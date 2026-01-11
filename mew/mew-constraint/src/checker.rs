//! Constraint checking.

use mew_core::{EdgeId, NodeId, Value};
use mew_graph::Graph;
use mew_pattern::{Bindings, Evaluator, Matcher};
use mew_registry::{ConstraintDef, Registry};

use crate::error::ConstraintResult;
use crate::violation::{Violation, ViolationSeverity, Violations};

/// Constraint checker.
pub struct ConstraintChecker<'r, 'g> {
    registry: &'r Registry,
    graph: &'g Graph,
    matcher: Matcher<'r, 'g>,
    evaluator: Evaluator<'r, 'g>,
}

impl<'r, 'g> ConstraintChecker<'r, 'g> {
    /// Create a new constraint checker.
    pub fn new(registry: &'r Registry, graph: &'g Graph) -> Self {
        Self {
            registry,
            graph,
            matcher: Matcher::new(registry, graph),
            evaluator: Evaluator::new(registry, graph),
        }
    }

    /// Check immediate constraints after a node mutation.
    pub fn check_node_immediate(&self, node_id: NodeId) -> ConstraintResult<Violations> {
        let mut violations = Violations::new();

        // Get the node's type
        let node = match self.graph.get_node(node_id) {
            Some(n) => n,
            None => return Ok(violations),
        };

        // Find applicable constraints for this type
        let constraints = self.registry.get_constraints_for_type(node.type_id);

        for constraint in constraints {
            if !constraint.deferred {
                if let Some(violation) = self.check_constraint(constraint, Some(node_id), None)? {
                    violations.push(violation);
                }
            }
        }

        Ok(violations)
    }

    /// Check immediate constraints after an edge mutation.
    pub fn check_edge_immediate(&self, edge_id: EdgeId) -> ConstraintResult<Violations> {
        let mut violations = Violations::new();

        // Get the edge's type
        let edge = match self.graph.get_edge(edge_id) {
            Some(e) => e,
            None => return Ok(violations),
        };

        // Find applicable constraints for this edge type
        let constraints = self.registry.get_constraints_for_edge_type(edge.type_id);

        for constraint in constraints {
            if !constraint.deferred {
                if let Some(violation) = self.check_constraint(constraint, None, Some(edge_id))? {
                    violations.push(violation);
                }
            }
        }

        Ok(violations)
    }

    /// Check deferred constraints at commit.
    pub fn check_deferred(&self, affected_nodes: &[NodeId], affected_edges: &[EdgeId]) -> ConstraintResult<Violations> {
        let mut violations = Violations::new();

        // Check deferred constraints for affected nodes
        for &node_id in affected_nodes {
            if let Some(node) = self.graph.get_node(node_id) {
                let constraints = self.registry.get_constraints_for_type(node.type_id);
                for constraint in constraints {
                    if constraint.deferred {
                        if let Some(violation) = self.check_constraint(constraint, Some(node_id), None)? {
                            violations.push(violation);
                        }
                    }
                }
            }
        }

        // Check deferred constraints for affected edges
        for &edge_id in affected_edges {
            if let Some(edge) = self.graph.get_edge(edge_id) {
                let constraints = self.registry.get_constraints_for_edge_type(edge.type_id);
                for constraint in constraints {
                    if constraint.deferred {
                        if let Some(violation) = self.check_constraint(constraint, None, Some(edge_id))? {
                            violations.push(violation);
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check all constraints (for testing or full validation).
    pub fn check_all(&self) -> ConstraintResult<Violations> {
        let mut violations = Violations::new();

        // Check all node constraints
        for node_id in self.graph.all_node_ids() {
            if let Some(node) = self.graph.get_node(node_id) {
                let constraints = self.registry.get_constraints_for_type(node.type_id);
                for constraint in constraints {
                    if let Some(violation) = self.check_constraint(constraint, Some(node_id), None)? {
                        violations.push(violation);
                    }
                }
            }
        }

        // Check all edge constraints
        for edge_id in self.graph.all_edge_ids() {
            if let Some(edge) = self.graph.get_edge(edge_id) {
                let constraints = self.registry.get_constraints_for_edge_type(edge.type_id);
                for constraint in constraints {
                    if let Some(violation) = self.check_constraint(constraint, None, Some(edge_id))? {
                        violations.push(violation);
                    }
                }
            }
        }

        Ok(violations)
    }

    // ========== Internal checking methods ==========

    /// Check a single constraint.
    fn check_constraint(
        &self,
        constraint: &ConstraintDef,
        node_id: Option<NodeId>,
        edge_id: Option<EdgeId>,
    ) -> ConstraintResult<Option<Violation>> {
        // Parse the condition string and check it
        // For now, we support simple condition patterns:
        // - "required:attr_name" - check that attribute is present
        // - "unique:attr_name" - check that attribute is unique
        // - "no_self" - check that edge doesn't target itself
        // - Other conditions are treated as expressions to evaluate

        let condition = &constraint.condition;

        if let Some(attr) = condition.strip_prefix("required:") {
            return self.check_required(constraint, node_id, attr.trim());
        }

        if let Some(attr) = condition.strip_prefix("unique:") {
            return self.check_unique(constraint, node_id, attr.trim());
        }

        if condition == "no_self" {
            return self.check_no_self(constraint, edge_id);
        }

        // For other conditions, we'd need to parse and evaluate them
        // For now, return no violation (constraint passes)
        Ok(None)
    }

    /// Check required attribute constraint.
    fn check_required(
        &self,
        constraint: &ConstraintDef,
        node_id: Option<NodeId>,
        attr: &str,
    ) -> ConstraintResult<Option<Violation>> {
        let node_id = match node_id {
            Some(id) => id,
            None => return Ok(None),
        };

        if let Some(node) = self.graph.get_node(node_id) {
            let has_value = node.get_attr(attr)
                .map(|v| !matches!(v, Value::Null))
                .unwrap_or(false);

            if !has_value {
                let severity = if constraint.hard {
                    ViolationSeverity::Error
                } else {
                    ViolationSeverity::Warning
                };

                return Ok(Some(
                    Violation::new(
                        &constraint.name,
                        severity,
                        format!("Required attribute '{}' is missing", attr),
                    )
                    .with_node(node_id),
                ));
            }
        }

        Ok(None)
    }

    /// Check unique attribute constraint.
    fn check_unique(
        &self,
        constraint: &ConstraintDef,
        node_id: Option<NodeId>,
        attr: &str,
    ) -> ConstraintResult<Option<Violation>> {
        let node_id = match node_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let node = match self.graph.get_node(node_id) {
            Some(n) => n,
            None => return Ok(None),
        };

        let value = match node.get_attr(attr) {
            Some(v) if !matches!(v, Value::Null) => v,
            _ => return Ok(None),
        };

        // Check if any other node of the same type has the same value
        for other_id in self.graph.nodes_by_attr(node.type_id, attr, value) {
            if other_id != node_id {
                let severity = if constraint.hard {
                    ViolationSeverity::Error
                } else {
                    ViolationSeverity::Warning
                };

                return Ok(Some(
                    Violation::new(
                        &constraint.name,
                        severity,
                        format!("Duplicate value for unique attribute '{}'", attr),
                    )
                    .with_node(node_id),
                ));
            }
        }

        Ok(None)
    }

    /// Check no_self constraint for an edge.
    fn check_no_self(
        &self,
        constraint: &ConstraintDef,
        edge_id: Option<EdgeId>,
    ) -> ConstraintResult<Option<Violation>> {
        let edge_id = match edge_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let edge = match self.graph.get_edge(edge_id) {
            Some(e) => e,
            None => return Ok(None),
        };

        // Check if any two targets are the same
        let targets = &edge.targets;
        for i in 0..targets.len() {
            for j in (i + 1)..targets.len() {
                if targets[i] == targets[j] {
                    let severity = if constraint.hard {
                        ViolationSeverity::Error
                    } else {
                        ViolationSeverity::Warning
                    };

                    return Ok(Some(
                        Violation::new(
                            &constraint.name,
                            severity,
                            "Self-referential edge not allowed",
                        )
                        .with_edge(edge_id),
                    ));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String").required())
            .attr(AttrDef::new("priority", "Int"))
            .done()
            .unwrap();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String").required())
            .attr(AttrDef::new("email", "String").unique())
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
    fn test_check_no_violations_for_valid_data() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();

        let node_id = graph.create_node(task_type_id, attrs! { "title" => "Test Task" });

        let checker = ConstraintChecker::new(&registry, &graph);

        // WHEN
        let violations = checker.check_node_immediate(node_id).unwrap();

        // THEN
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_all_no_violations() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let person_type_id = registry.get_type_id("Person").unwrap();

        graph.create_node(task_type_id, attrs! { "title" => "Task 1" });
        graph.create_node(person_type_id, attrs! { "name" => "Alice" });

        let checker = ConstraintChecker::new(&registry, &graph);

        // WHEN
        let violations = checker.check_all().unwrap();

        // THEN
        assert!(violations.is_empty());
    }

    #[test]
    fn test_violations_collection() {
        // GIVEN
        let mut violations = Violations::new();

        // WHEN
        violations.push(Violation::error("test1", "Error 1"));
        violations.push(Violation::warning("test2", "Warning 1"));

        // THEN
        assert_eq!(violations.len(), 2);
        assert!(violations.has_errors());
        assert_eq!(violations.errors().count(), 1);
        assert_eq!(violations.warnings().count(), 1);
    }
}
