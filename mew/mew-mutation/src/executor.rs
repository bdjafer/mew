//! Mutation execution.

use mew_core::{EdgeId, EntityId, NodeId, Value};
use mew_graph::Graph;
use mew_parser::{KillStmt, LinkStmt, SetStmt, SpawnStmt, UnlinkStmt};
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};
use crate::result::{CreatedEntity, DeletedEntities, MutationResult as MutationOutput, UpdatedEntities};

/// Mutation executor.
pub struct MutationExecutor<'r, 'g> {
    registry: &'r Registry,
    graph: &'g mut Graph,
}

impl<'r, 'g> MutationExecutor<'r, 'g> {
    /// Create a new executor.
    pub fn new(registry: &'r Registry, graph: &'g mut Graph) -> Self {
        Self { registry, graph }
    }

    /// Execute a SPAWN statement.
    pub fn execute_spawn(&mut self, stmt: &SpawnStmt, bindings: &Bindings) -> MutationResult<MutationOutput> {
        // Look up the type
        let type_id = self
            .registry
            .get_type_id(&stmt.type_name)
            .ok_or_else(|| MutationError::unknown_type(&stmt.type_name))?;

        // Check that type is not abstract
        if let Some(type_def) = self.registry.get_type(type_id) {
            if type_def.is_abstract {
                return Err(MutationError::abstract_type(&stmt.type_name));
            }
        }

        // Build attributes
        let mut attrs = mew_core::Attributes::new();

        // Create evaluator with immutable graph reference
        let evaluator = Evaluator::new(self.registry, unsafe {
            &*(self.graph as *const Graph)
        });

        for assign in &stmt.attrs {
            // Evaluate the value expression
            let value = evaluator.eval(&assign.value, bindings)?;

            // Validate attribute exists and type matches
            self.validate_attribute(&stmt.type_name, type_id, &assign.name, &value)?;

            attrs.insert(assign.name.clone(), value);
        }

        // Check required attributes
        self.check_required_attributes(&stmt.type_name, type_id, &attrs)?;

        // Apply default values
        self.apply_defaults(&stmt.type_name, type_id, &mut attrs, bindings)?;

        // Create the node
        let node_id = self.graph.create_node(type_id, attrs);

        Ok(MutationOutput::Created(CreatedEntity::node(node_id)))
    }

    /// Execute a KILL statement (node deletion).
    pub fn execute_kill(&mut self, stmt: &KillStmt, target_id: NodeId) -> MutationResult<MutationOutput> {
        // Check node exists
        if self.graph.get_node(target_id).is_none() {
            return Err(MutationError::NodeNotFound(target_id));
        }

        // Collect edges to delete (cascade)
        let cascade = stmt.cascade.unwrap_or(true);
        let mut deleted_edges = Vec::new();

        if cascade {
            // Delete all incident edges
            let edges_from: Vec<_> = self.graph.edges_from(target_id, None).collect();
            let edges_to: Vec<_> = self.graph.edges_to(target_id, None).collect();

            for edge_id in edges_from.into_iter().chain(edges_to) {
                // Also delete higher-order edges about this edge
                let ho_edges: Vec<_> = self.graph.edges_about(edge_id).collect();
                for ho_edge_id in ho_edges {
                    if self.graph.delete_edge(ho_edge_id).is_ok() {
                        deleted_edges.push(ho_edge_id);
                    }
                }

                if self.graph.delete_edge(edge_id).is_ok() {
                    deleted_edges.push(edge_id);
                }
            }
        }

        // Delete the node
        self.graph.delete_node(target_id);

        Ok(MutationOutput::Deleted(
            DeletedEntities::node(target_id).with_cascade_edges(deleted_edges),
        ))
    }

    /// Execute a LINK statement (edge creation).
    pub fn execute_link(&mut self, stmt: &LinkStmt, target_ids: Vec<EntityId>) -> MutationResult<MutationOutput> {
        // Look up the edge type
        let edge_type_id = self
            .registry
            .get_edge_type_id(&stmt.edge_type)
            .ok_or_else(|| MutationError::unknown_edge_type(&stmt.edge_type))?;

        // Validate arity
        if let Some(edge_type) = self.registry.get_edge_type(edge_type_id) {
            let expected = edge_type.params.len();
            let actual = target_ids.len();
            if expected != actual {
                return Err(MutationError::invalid_arity(&stmt.edge_type, expected, actual));
            }

            // Validate target types
            for (i, (param, target_id)) in edge_type.params.iter().zip(target_ids.iter()).enumerate() {
                if let EntityId::Node(node_id) = target_id {
                    if let Some(node) = self.graph.get_node(*node_id) {
                        // Check if node type matches expected parameter type
                        // "any" means any type is allowed
                        if param.type_constraint != "any" {
                            let expected_type_id = self.registry.get_type_id(&param.type_constraint);
                            if let Some(expected_id) = expected_type_id {
                                if !self.registry.is_subtype(node.type_id, expected_id) {
                                    let actual_name = self.registry.get_type(node.type_id)
                                        .map(|t| t.name.clone())
                                        .unwrap_or_else(|| "unknown".to_string());
                                    return Err(MutationError::target_type_mismatch(
                                        i,
                                        &param.type_constraint,
                                        actual_name,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Build attributes
        let mut attrs = mew_core::Attributes::new();
        let bindings = Bindings::new();
        let evaluator = Evaluator::new(self.registry, unsafe {
            &*(self.graph as *const Graph)
        });

        for assign in &stmt.attrs {
            let value = evaluator.eval(&assign.value, &bindings)?;
            attrs.insert(assign.name.clone(), value);
        }

        // Create the edge
        let edge_id = self.graph.create_edge(edge_type_id, target_ids, attrs)
            .map_err(|e| MutationError::pattern_error(e.to_string()))?;

        Ok(MutationOutput::Created(CreatedEntity::edge(edge_id)))
    }

    /// Execute an UNLINK statement (edge deletion).
    pub fn execute_unlink(&mut self, _stmt: &UnlinkStmt, target_id: EdgeId) -> MutationResult<MutationOutput> {
        // Check edge exists
        if self.graph.get_edge(target_id).is_none() {
            return Err(MutationError::EdgeNotFound(target_id));
        }

        // Delete higher-order edges about this edge first
        let ho_edges: Vec<_> = self.graph.edges_about(target_id).collect();
        let mut deleted_edges = Vec::new();

        for ho_edge_id in ho_edges {
            if self.graph.delete_edge(ho_edge_id).is_ok() {
                deleted_edges.push(ho_edge_id);
            }
        }

        // Delete the edge
        let _ = self.graph.delete_edge(target_id);
        deleted_edges.push(target_id);

        Ok(MutationOutput::Deleted(DeletedEntities::edge(target_id).with_cascade_edges(deleted_edges)))
    }

    /// Execute a SET statement (attribute update).
    pub fn execute_set(
        &mut self,
        stmt: &SetStmt,
        node_ids: Vec<NodeId>,
        bindings: &Bindings,
    ) -> MutationResult<MutationOutput> {
        let mut updated_ids = Vec::new();

        let evaluator = Evaluator::new(self.registry, unsafe {
            &*(self.graph as *const Graph)
        });

        for node_id in node_ids {
            let node = self
                .graph
                .get_node(node_id)
                .ok_or(MutationError::NodeNotFound(node_id))?;

            let type_id = node.type_id;
            let type_name = self.registry.get_type(type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "unknown".to_string());

            // Validate and collect new attributes
            let mut new_attrs = Vec::new();

            for assign in &stmt.assignments {
                // Evaluate the value
                let value = evaluator.eval(&assign.value, bindings)?;

                // Validate attribute
                self.validate_attribute(&type_name, type_id, &assign.name, &value)?;

                new_attrs.push((assign.name.clone(), value));
            }

            // Apply updates
            for (name, value) in new_attrs {
                self.graph.set_node_attr(node_id, &name, value)
                    .map_err(|e| MutationError::pattern_error(e.to_string()))?;
            }

            updated_ids.push(node_id);
        }

        Ok(MutationOutput::Updated(UpdatedEntities::nodes(updated_ids)))
    }

    // ========== Validation helpers ==========

    /// Validate an attribute assignment.
    fn validate_attribute(
        &self,
        type_name: &str,
        type_id: mew_core::TypeId,
        attr_name: &str,
        value: &Value,
    ) -> MutationResult<()> {
        if let Some(type_def) = self.registry.get_type(type_id) {
            if let Some(attr_def) = type_def.get_attr(attr_name) {
                // Check type compatibility
                let expected_type = &attr_def.type_name;
                let actual_type = self.value_type_name(value);

                if !self.types_compatible(expected_type, &actual_type) {
                    return Err(MutationError::invalid_attr_type(
                        attr_name,
                        expected_type,
                        actual_type,
                    ));
                }
            } else {
                return Err(MutationError::unknown_attribute(type_name, attr_name));
            }
        }

        Ok(())
    }

    /// Check that all required attributes are present.
    fn check_required_attributes(
        &self,
        type_name: &str,
        type_id: mew_core::TypeId,
        attrs: &mew_core::Attributes,
    ) -> MutationResult<()> {
        if let Some(type_def) = self.registry.get_type(type_id) {
            for (attr_name, attr_def) in &type_def.attributes {
                if attr_def.required && !attrs.contains_key(attr_name) && attr_def.default.is_none() {
                    return Err(MutationError::missing_required(type_name, attr_name));
                }
            }
        }
        Ok(())
    }

    /// Apply default values to missing attributes.
    fn apply_defaults(
        &self,
        _type_name: &str,
        type_id: mew_core::TypeId,
        attrs: &mut mew_core::Attributes,
        _bindings: &Bindings,
    ) -> MutationResult<()> {
        if let Some(type_def) = self.registry.get_type(type_id) {
            for (attr_name, attr_def) in &type_def.attributes {
                if !attrs.contains_key(attr_name) {
                    if let Some(ref default_value) = attr_def.default {
                        attrs.insert(attr_name.clone(), default_value.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the type name of a value.
    fn value_type_name(&self, value: &Value) -> String {
        match value {
            Value::Null => "Null".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::Int(_) => "Int".to_string(),
            Value::Float(_) => "Float".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Timestamp(_) => "Timestamp".to_string(),
            Value::Duration(_) => "Duration".to_string(),
            Value::NodeRef(_) => "NodeRef".to_string(),
            Value::EdgeRef(_) => "EdgeRef".to_string(),
        }
    }

    /// Check if types are compatible.
    fn types_compatible(&self, expected: &str, actual: &str) -> bool {
        if expected == actual {
            return true;
        }
        // Null is compatible with anything
        if actual == "Null" {
            return true;
        }
        // Int can be used where Float is expected
        if expected == "Float" && actual == "Int" {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;
    use mew_parser::{AttrAssignment, Expr, Literal, LiteralKind, Span};
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
    fn test_spawn_valid_node() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut executor = MutationExecutor::new(&registry, &mut graph);
        let bindings = Bindings::new();

        let stmt = SpawnStmt {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            attrs: vec![AttrAssignment {
                name: "title".to_string(),
                value: Expr::Literal(Literal {
                    kind: LiteralKind::String("New Task".to_string()),
                    span: Span::default(),
                }),
                span: Span::default(),
            }],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_spawn(&stmt, &bindings);

        // THEN
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.created_node().is_some());
    }

    #[test]
    fn test_spawn_unknown_type() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut executor = MutationExecutor::new(&registry, &mut graph);
        let bindings = Bindings::new();

        let stmt = SpawnStmt {
            var: "x".to_string(),
            type_name: "Unknown".to_string(),
            attrs: vec![],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_spawn(&stmt, &bindings);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::UnknownType { .. }));
    }

    #[test]
    fn test_spawn_missing_required() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut executor = MutationExecutor::new(&registry, &mut graph);
        let bindings = Bindings::new();

        // Missing required 'title' attribute
        let stmt = SpawnStmt {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            attrs: vec![],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_spawn(&stmt, &bindings);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::MissingRequired { .. }));
    }

    #[test]
    fn test_spawn_wrong_attr_type() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut executor = MutationExecutor::new(&registry, &mut graph);
        let bindings = Bindings::new();

        // Wrong type for 'priority' - should be Int
        let stmt = SpawnStmt {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            attrs: vec![
                AttrAssignment {
                    name: "title".to_string(),
                    value: Expr::Literal(Literal {
                        kind: LiteralKind::String("Task".to_string()),
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
                AttrAssignment {
                    name: "priority".to_string(),
                    value: Expr::Literal(Literal {
                        kind: LiteralKind::String("high".to_string()), // Should be Int
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
            ],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_spawn(&stmt, &bindings);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::InvalidAttrType { .. }));
    }

    #[test]
    fn test_kill_existing_node() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node_id = graph.create_node(task_type_id, attrs! { "title" => "Test" });

        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = KillStmt {
            target: mew_parser::Target::Var("t".to_string()),
            cascade: Some(true),
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_kill(&stmt, node_id);

        // THEN
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deleted_nodes(), 1);
    }

    #[test]
    fn test_kill_nonexistent_node() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = KillStmt {
            target: mew_parser::Target::Var("t".to_string()),
            cascade: Some(true),
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_kill(&stmt, NodeId::new(999));

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::NodeNotFound(_)));
    }

    #[test]
    fn test_link_valid_edge() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let person_type_id = registry.get_type_id("Person").unwrap();

        let person = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let task = graph.create_node(task_type_id, attrs! { "title" => "Task 1" });

        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = LinkStmt {
            var: Some("e".to_string()),
            edge_type: "owns".to_string(),
            targets: vec![
                mew_parser::TargetRef::Var("p".to_string()),
                mew_parser::TargetRef::Var("t".to_string()),
            ],
            attrs: vec![],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_link(&stmt, vec![person.into(), task.into()]);

        // THEN
        assert!(result.is_ok());
        assert!(result.unwrap().created_edge().is_some());
    }

    #[test]
    fn test_link_wrong_arity() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let person = graph.create_node(person_type_id, attrs! { "name" => "Alice" });

        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = LinkStmt {
            var: None,
            edge_type: "owns".to_string(),
            targets: vec![mew_parser::TargetRef::Var("p".to_string())], // Missing second target
            attrs: vec![],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_link(&stmt, vec![person.into()]);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MutationError::InvalidArity { .. }));
    }

    #[test]
    fn test_unlink_existing_edge() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let person_type_id = registry.get_type_id("Person").unwrap();
        let owns_type_id = registry.get_edge_type_id("owns").unwrap();

        let person = graph.create_node(person_type_id, attrs! { "name" => "Alice" });
        let task = graph.create_node(task_type_id, attrs! { "title" => "Task 1" });
        let edge = graph.create_edge(owns_type_id, vec![person.into(), task.into()], attrs! {}).unwrap();

        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = UnlinkStmt {
            target: mew_parser::Target::Var("e".to_string()),
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_unlink(&stmt, edge);

        // THEN
        assert!(result.is_ok());
        assert!(result.unwrap().deleted_edges() >= 1);
    }

    #[test]
    fn test_set_valid_attribute() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let task_type_id = registry.get_type_id("Task").unwrap();
        let node_id = graph.create_node(task_type_id, attrs! { "title" => "Original" });

        let mut executor = MutationExecutor::new(&registry, &mut graph);

        let stmt = SetStmt {
            target: mew_parser::Target::Var("t".to_string()),
            assignments: vec![AttrAssignment {
                name: "title".to_string(),
                value: Expr::Literal(Literal {
                    kind: LiteralKind::String("Updated".to_string()),
                    span: Span::default(),
                }),
                span: Span::default(),
            }],
            returning: None,
            span: Span::default(),
        };

        // WHEN
        let bindings = Bindings::new();
        let result = executor.execute_set(&stmt, vec![node_id], &bindings);

        // THEN
        assert!(result.is_ok());

        // Verify the update
        let node = graph.get_node(node_id).unwrap();
        assert_eq!(node.get_attr("title"), Some(&Value::String("Updated".to_string())));
    }
}
