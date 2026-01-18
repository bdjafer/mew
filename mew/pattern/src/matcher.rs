//! Pattern matching against the graph.

use crate::{Binding, Bindings, CompiledPattern, Evaluator, PatternOp, PatternResult};
use mew_core::NodeId;
use mew_graph::Graph;
use mew_registry::Registry;

/// Pattern matcher that finds all matches in a graph.
pub struct Matcher<'r, 'g> {
    #[allow(dead_code)]
    registry: &'r Registry,
    graph: &'g Graph,
    evaluator: Evaluator<'r>,
}

impl<'r, 'g> Matcher<'r, 'g> {
    /// Create a new matcher.
    pub fn new(registry: &'r Registry, graph: &'g Graph) -> Self {
        Self {
            registry,
            graph,
            evaluator: Evaluator::new(registry),
        }
    }

    /// Find all matches of the pattern.
    pub fn find_all(&self, pattern: &CompiledPattern) -> PatternResult<Vec<Bindings>> {
        self.find_all_with_initial(pattern, Bindings::new())
    }

    /// Check if any match exists.
    /// This is a convenience method that delegates to find_all_with_initial.
    pub fn exists(&self, pattern: &CompiledPattern, initial: Bindings) -> PatternResult<bool> {
        let matches = self.find_all_with_initial(pattern, initial)?;
        Ok(!matches.is_empty())
    }

    /// Find all matches starting with initial bindings.
    pub fn find_all_with_initial(
        &self,
        pattern: &CompiledPattern,
        initial: Bindings,
    ) -> PatternResult<Vec<Bindings>> {
        // Start with the initial bindings
        let mut candidates = vec![initial];

        // Track edge variables that need deduplication (for symmetric edges)
        let mut edge_vars_for_dedup: Vec<String> = Vec::new();

        // Process each operation
        for op in &pattern.ops {
            let mut new_candidates = Vec::new();

            for bindings in candidates {
                let matches = self.execute_op(op, &bindings)?;
                new_candidates.extend(matches);
            }

            candidates = new_candidates;

            // Track edge variables from symmetric edge FollowEdge operations
            if let PatternOp::FollowEdge {
                edge_type_id,
                edge_var,
                ..
            } = op
            {
                if let Some(ref var) = edge_var {
                    if let Some(edge_type) = self.registry.get_edge_type(*edge_type_id) {
                        if edge_type.symmetric {
                            edge_vars_for_dedup.push(var.clone());
                        }
                    }
                }
            }
        }

        // Deduplicate by edge_id for symmetric edges
        // This ensures each physical edge appears only once in results
        // We prefer forward (stored-order) matches over reverse matches
        for edge_var in &edge_vars_for_dedup {
            // First pass: collect all matches, grouping by edge_id
            let mut edge_matches: std::collections::HashMap<mew_core::EdgeId, Vec<Bindings>> =
                std::collections::HashMap::new();
            let mut non_edge_bindings = Vec::new();

            for bindings in candidates {
                if let Some(binding) = bindings.get(edge_var) {
                    if let Some(edge_id) = binding.as_edge() {
                        edge_matches.entry(edge_id).or_default().push(bindings);
                    } else {
                        non_edge_bindings.push(bindings);
                    }
                } else {
                    non_edge_bindings.push(bindings);
                }
            }

            // Second pass: for each edge_id, prefer forward match over reverse match
            let reverse_marker = format!("_reverse_{}", edge_var);
            let mut deduped = non_edge_bindings;
            for (_edge_id, matches) in edge_matches {
                if matches.len() == 1 {
                    // Only one match, use it
                    deduped.push(matches.into_iter().next().unwrap());
                } else {
                    // Multiple matches - prefer the one WITHOUT reverse marker
                    let mut forward = None;
                    let mut reverse = None;
                    for m in matches {
                        if m.get(&reverse_marker).is_some() {
                            reverse = Some(m);
                        } else {
                            forward = Some(m);
                        }
                    }
                    // Prefer forward, fall back to reverse
                    if let Some(m) = forward {
                        deduped.push(m);
                    } else if let Some(m) = reverse {
                        deduped.push(m);
                    }
                }
            }
            candidates = deduped;
        }

        Ok(candidates)
    }

    /// Execute a single pattern operation.
    fn execute_op(&self, op: &PatternOp, bindings: &Bindings) -> PatternResult<Vec<Bindings>> {
        match op {
            PatternOp::ScanNodes { var, type_id } => {
                // Scan all nodes of the given type AND all subtypes (polymorphic matching)
                let mut node_ids: Vec<NodeId> = self.graph.nodes_by_type(*type_id).collect();

                // Also include nodes of all subtypes
                for subtype_id in self.registry.get_subtypes(*type_id) {
                    node_ids.extend(self.graph.nodes_by_type(subtype_id));
                }

                let matches: Vec<Bindings> = node_ids
                    .into_iter()
                    .map(|id| bindings.extend_with(var, Binding::Node(id)))
                    .collect();

                Ok(matches)
            }

            PatternOp::FollowEdge {
                edge_type_id,
                from_vars,
                edge_var,
            } => {
                // Get the source node from bindings
                if from_vars.is_empty() {
                    return Ok(vec![bindings.clone()]);
                }

                let first_var = &from_vars[0];
                let source_binding = bindings
                    .get(first_var)
                    .ok_or_else(|| crate::PatternError::unbound_variable(first_var))?;

                let source_id = source_binding
                    .as_node()
                    .ok_or_else(|| crate::PatternError::type_error("expected node binding"))?;

                // Check if this edge type is symmetric
                let is_symmetric = self
                    .registry
                    .get_edge_type(*edge_type_id)
                    .map(|et| et.symmetric)
                    .unwrap_or(false);

                // Find edges from this node with matching type
                let edges: Vec<_> = self
                    .graph
                    .edges_from(source_id, Some(*edge_type_id))
                    .collect();

                let mut matches = Vec::new();

                for edge_id in edges {
                    if let Some(edge) = self.graph.get_edge(edge_id) {
                        // Check that all other target variables match (if bound)
                        let mut all_match = true;
                        for (i, var) in from_vars.iter().enumerate() {
                            if let Some(binding) = bindings.get(var) {
                                if let Some(expected_id) = binding.as_node() {
                                    if i < edge.targets.len() {
                                        let actual_id = edge.targets[i].as_node();
                                        if actual_id != Some(expected_id) {
                                            all_match = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if all_match {
                            let mut new_bindings = bindings.clone();
                            if let Some(alias) = edge_var {
                                new_bindings.insert(alias, Binding::Edge(edge_id));
                            }
                            matches.push(new_bindings);
                        }
                    }
                }

                // For symmetric edges, also search edges where source_id is at position 1
                // This handles the case: friend_of(alice, bob) stored, query friend_of(bob, x)
                //
                // Note: This does NOT cause duplicates because:
                // - Forward search uses edges_from(source_id) - finds edges where source_id is at pos 0
                // - Reverse search uses edges_to(source_id) - finds edges where source_id is at pos > 0
                // For a given candidate, only ONE of these will match, never both.
                if is_symmetric && from_vars.len() == 2 {
                    let reverse_edges: Vec<_> = self
                        .graph
                        .edges_to(source_id, Some(*edge_type_id))
                        .collect();

                    for edge_id in reverse_edges {
                        if let Some(edge) = self.graph.get_edge(edge_id) {
                            // For symmetric edge, the query pattern edge(a, b) should match
                            // stored edge(x, y) when a=y (position 1).
                            // So we need to check: from_vars[0] matches edge.targets[1]
                            //                      from_vars[1] matches edge.targets[0]
                            let mut all_match = true;

                            // Check first variable against position 1
                            if let Some(binding) = bindings.get(&from_vars[0]) {
                                if let Some(expected_id) = binding.as_node() {
                                    if edge.targets.len() > 1 {
                                        let actual_id = edge.targets[1].as_node();
                                        if actual_id != Some(expected_id) {
                                            all_match = false;
                                        }
                                    }
                                }
                            }

                            // Check second variable against position 0 (if bound)
                            if all_match {
                                if let Some(binding) = bindings.get(&from_vars[1]) {
                                    if let Some(expected_id) = binding.as_node() {
                                        let actual_id = edge.targets[0].as_node();
                                        if actual_id != Some(expected_id) {
                                            all_match = false;
                                        }
                                    }
                                }
                            }

                            if all_match {
                                let mut new_bindings = bindings.clone();
                                if let Some(alias) = edge_var {
                                    new_bindings.insert(alias, Binding::Edge(edge_id));
                                    // Mark as reverse match for deduplication preference
                                    new_bindings.insert(
                                        &format!("_reverse_{}", alias),
                                        Binding::Value(mew_core::Value::Bool(true)),
                                    );
                                }
                                matches.push(new_bindings);
                            }
                        }
                    }
                }

                Ok(matches)
            }

            PatternOp::CheckEdge {
                edge_type_id,
                target_vars,
            } => {
                // Check that an edge exists between the bound variables
                // Build a vector of Option<NodeId> where None represents wildcards
                let mut target_ids: Vec<Option<NodeId>> = Vec::new();
                for var in target_vars {
                    // Wildcards become None
                    if var == "_" {
                        target_ids.push(None);
                    } else {
                        let binding = bindings
                            .get(var)
                            .ok_or_else(|| crate::PatternError::unbound_variable(var))?;
                        let node_id = binding.as_node().ok_or_else(|| {
                            crate::PatternError::type_error("expected node binding")
                        })?;
                        target_ids.push(Some(node_id));
                    }
                }

                // Find the first non-wildcard node to use for edge lookup
                let first_bound_idx = target_ids.iter().position(|id| id.is_some());

                if let Some(idx) = first_bound_idx {
                    let bound_id = target_ids[idx].unwrap();
                    // If bound node is at position 0, use edges_from (it's the source)
                    // Otherwise use edges_to (it's a target)
                    let edges: Vec<_> = if idx == 0 {
                        self.graph
                            .edges_from(bound_id, Some(*edge_type_id))
                            .collect()
                    } else {
                        self.graph.edges_to(bound_id, Some(*edge_type_id)).collect()
                    };

                    for edge_id in edges {
                        if let Some(edge) = self.graph.get_edge(edge_id) {
                            // Check all non-wildcard targets match
                            let mut all_match = true;
                            for (i, expected_id_opt) in target_ids.iter().enumerate() {
                                if let Some(expected_id) = expected_id_opt {
                                    if i < edge.targets.len() {
                                        if edge.targets[i].as_node() != Some(*expected_id) {
                                            all_match = false;
                                            break;
                                        }
                                    } else {
                                        all_match = false;
                                        break;
                                    }
                                }
                                // If None (wildcard), accept any value
                            }
                            if all_match {
                                return Ok(vec![bindings.clone()]);
                            }
                        }
                    }
                }

                // No matching edge found
                Ok(vec![])
            }

            PatternOp::Filter { condition } => {
                // Evaluate the filter condition
                let result = self.evaluator.eval_bool(condition, bindings, self.graph)?;

                if result {
                    Ok(vec![bindings.clone()])
                } else {
                    Ok(vec![])
                }
            }

            PatternOp::NotExists { subpattern } => {
                // Check that the subpattern does NOT match
                let matches = self.find_all_with_initial(subpattern, bindings.clone())?;

                if matches.is_empty() {
                    // Subpattern doesn't match, so NOT EXISTS is true
                    Ok(vec![bindings.clone()])
                } else {
                    // Subpattern matches, so NOT EXISTS is false
                    Ok(vec![])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;
    use mew_parser::{EdgePattern, NodePattern, PatternElem};
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .attr(AttrDef::new("priority", "Int"))
            .done()
            .unwrap();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("owns")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_match_single_type() {
        // GIVEN graph with Task A, Task B, Person C
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let person_type_id = registry.get_type_id("Person").unwrap();

        let _task_a = graph.create_node(task_type_id, attrs! { "title" => "Task A" });
        let _task_b = graph.create_node(task_type_id, attrs! { "title" => "Task B" });
        let _person_c = graph.create_node(person_type_id, attrs! { "name" => "Carol" });

        // PATTERN: t: Task
        let elements = vec![PatternElem::Node(NodePattern {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            span: Default::default(),
        })];
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // WHEN
        let matcher = Matcher::new(&registry, &graph);
        let matches = matcher.find_all(&pattern).unwrap();

        // THEN: expect 2 matches (Task A, Task B)
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_match_with_edge() {
        // GIVEN graph with Person Alice, Person Bob, owns(Alice, Task1)
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let owns_type_id = registry.get_edge_type_id("owns").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let _bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });
        let task1 = graph.create_node(task_type_id, attrs! { "title" => "Task 1" });

        let _ = graph.create_edge(owns_type_id, vec![alice.into(), task1.into()], attrs! {});

        // PATTERN: p: Person, t: Task, owns(p, t)
        let elements = vec![
            PatternElem::Node(NodePattern {
                var: "p".to_string(),
                type_name: "Person".to_string(),
                span: Default::default(),
            }),
            PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Default::default(),
            }),
            PatternElem::Edge(EdgePattern {
                edge_type: "owns".to_string(),
                targets: vec!["p".to_string(), "t".to_string()],
                alias: None,
                transitive: None,
                span: Default::default(),
            }),
        ];
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // WHEN
        let matcher = Matcher::new(&registry, &graph);
        let matches = matcher.find_all(&pattern).unwrap();

        // THEN: expect 1 match (Alice owns Task1)
        assert_eq!(matches.len(), 1);
        let bindings = &matches[0];
        assert_eq!(bindings.get("p").unwrap().as_node(), Some(alice));
        assert_eq!(bindings.get("t").unwrap().as_node(), Some(task1));
    }

    #[test]
    fn test_match_with_where_filter() {
        // GIVEN graph with Task A (priority=1), Task B (priority=5), Task C (priority=10)
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        graph.create_node(task_type_id, attrs! { "title" => "A", "priority" => 1 });
        let task_b = graph.create_node(task_type_id, attrs! { "title" => "B", "priority" => 5 });
        let task_c = graph.create_node(task_type_id, attrs! { "title" => "C", "priority" => 10 });

        // PATTERN: t: Task WHERE t.priority > 3
        let elements = vec![PatternElem::Node(NodePattern {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            span: Default::default(),
        })];
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // Add filter
        let filter = mew_parser::Expr::BinaryOp(
            mew_parser::BinaryOp::Gt,
            Box::new(mew_parser::Expr::AttrAccess(
                Box::new(mew_parser::Expr::Var("t".to_string(), Default::default())),
                "priority".to_string(),
                Default::default(),
            )),
            Box::new(mew_parser::Expr::Literal(mew_parser::Literal {
                kind: mew_parser::LiteralKind::Int(3),
                span: Default::default(),
            })),
            Default::default(),
        );
        let pattern = pattern.with_filter(filter);

        // WHEN
        let matcher = Matcher::new(&registry, &graph);
        let matches = matcher.find_all(&pattern).unwrap();

        // THEN: expect 2 matches (B and C)
        assert_eq!(matches.len(), 2);

        // Verify the matched nodes are B and C
        let node_ids: Vec<NodeId> = matches
            .iter()
            .map(|b| b.get("t").unwrap().as_node().unwrap())
            .collect();
        assert!(node_ids.contains(&task_b));
        assert!(node_ids.contains(&task_c));
    }

    #[test]
    fn test_match_symmetric_edge_reverse() {
        // GIVEN registry with symmetric edge friend_of(Person, Person)
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("friend_of")
            .param("a", "Person")
            .param("b", "Person")
            .symmetric()
            .done()
            .unwrap();
        let registry = builder.build().unwrap();

        // GIVEN graph with alice, bob and friend_of(alice, bob)
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let friend_of_type_id = registry.get_edge_type_id("friend_of").unwrap();

        let alice = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let bob = graph.create_node(person_type_id, attrs! { "name" => "Bob" });

        graph
            .create_edge(friend_of_type_id, vec![alice.into(), bob.into()], attrs! {})
            .unwrap();

        // PATTERN: a: Person, b: Person, friend_of(a, b)
        let elements = vec![
            PatternElem::Node(NodePattern {
                var: "a".to_string(),
                type_name: "Person".to_string(),
                span: Default::default(),
            }),
            PatternElem::Node(NodePattern {
                var: "b".to_string(),
                type_name: "Person".to_string(),
                span: Default::default(),
            }),
            PatternElem::Edge(EdgePattern {
                edge_type: "friend_of".to_string(),
                targets: vec!["a".to_string(), "b".to_string()],
                alias: None,
                transitive: None,
                span: Default::default(),
            }),
        ];
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // WHEN
        let matcher = Matcher::new(&registry, &graph);
        let matches = matcher.find_all(&pattern).unwrap();

        // THEN: For symmetric edge with cross-join, should return 1 match (not 2).
        // Per spec: "Only one edge is stored" - queries should return each edge once.
        assert_eq!(
            matches.len(),
            1,
            "Symmetric edge cross-join should return one row per physical edge"
        );

        // Verify the match contains the edge in stored order (alice, bob)
        let binding = &matches[0];
        assert_eq!(binding.get("a").unwrap().as_node(), Some(alice));
        assert_eq!(binding.get("b").unwrap().as_node(), Some(bob));
    }
}
