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

    /// Find all matches starting with initial bindings.
    pub fn find_all_with_initial(
        &self,
        pattern: &CompiledPattern,
        initial: Bindings,
    ) -> PatternResult<Vec<Bindings>> {
        // Start with the initial bindings
        let mut candidates = vec![initial];

        // Process each operation
        for op in &pattern.ops {
            let mut new_candidates = Vec::new();

            for bindings in candidates {
                let matches = self.execute_op(op, &bindings)?;
                new_candidates.extend(matches);
            }

            candidates = new_candidates;
        }

        Ok(candidates)
    }

    /// Execute a single pattern operation.
    fn execute_op(&self, op: &PatternOp, bindings: &Bindings) -> PatternResult<Vec<Bindings>> {
        match op {
            PatternOp::ScanNodes { var, type_id } => {
                // Scan all nodes of the given type
                let node_ids: Vec<NodeId> = self.graph.nodes_by_type(*type_id).collect();

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

                Ok(matches)
            }

            PatternOp::CheckEdge {
                edge_type_id,
                target_vars,
            } => {
                // Check that an edge exists between the bound variables
                // Get all bound node IDs from target_vars
                let mut target_ids = Vec::new();
                for var in target_vars {
                    let binding = bindings
                        .get(var)
                        .ok_or_else(|| crate::PatternError::unbound_variable(var))?;
                    let node_id = binding
                        .as_node()
                        .ok_or_else(|| crate::PatternError::type_error("expected node binding"))?;
                    target_ids.push(node_id);
                }

                // Check if such an edge exists
                if target_ids.len() >= 2 {
                    let edges: Vec<_> = self
                        .graph
                        .edges_from(target_ids[0], Some(*edge_type_id))
                        .collect();

                    for edge_id in edges {
                        if let Some(edge) = self.graph.get_edge(edge_id) {
                            // Check all targets match
                            let mut all_match = true;
                            for (i, &expected_id) in target_ids.iter().enumerate() {
                                if i < edge.targets.len() {
                                    if edge.targets[i].as_node() != Some(expected_id) {
                                        all_match = false;
                                        break;
                                    }
                                }
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
}
