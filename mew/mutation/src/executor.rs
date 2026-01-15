//! Mutation executor - coordinates mutation operations.
//!
//! The executor delegates to specialized operation modules in `ops/`:
//! - `ops/spawn.rs` - SPAWN (node creation)
//! - `ops/kill.rs` - KILL (node deletion with cascade)
//! - `ops/link.rs` - LINK (edge creation)
//! - `ops/unlink.rs` - UNLINK (edge deletion)
//! - `ops/set.rs` - SET (attribute updates)

use mew_core::{EdgeId, EntityId, NodeId};
use mew_graph::Graph;
use mew_parser::{KillStmt, LinkStmt, SetStmt, SpawnItem, SpawnStmt, UnlinkStmt};
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::MutationResult;
use crate::ops;
use crate::result::MutationOutcome;

/// Mutation executor.
pub struct MutationExecutor<'r, 'g> {
    registry: &'r Registry,
    graph: &'g mut Graph,
    evaluator: Evaluator<'r>,
}

impl<'r, 'g> MutationExecutor<'r, 'g> {
    /// Create a new executor.
    pub fn new(registry: &'r Registry, graph: &'g mut Graph) -> Self {
        Self {
            registry,
            graph,
            evaluator: Evaluator::new(registry),
        }
    }

    /// Execute a SPAWN statement.
    pub fn execute_spawn(
        &mut self,
        stmt: &SpawnStmt,
        bindings: &Bindings,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_spawn(self.registry, self.graph, &self.evaluator, stmt, bindings)
    }

    /// Execute a spawn from a SpawnItem (used by multi-spawn).
    pub fn execute_spawn_item(
        &mut self,
        item: &SpawnItem,
        bindings: &Bindings,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_spawn_item(self.registry, self.graph, &self.evaluator, item, bindings)
    }

    /// Execute a KILL statement (node deletion).
    pub fn execute_kill(
        &mut self,
        stmt: &KillStmt,
        target_id: NodeId,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_kill(self.registry, self.graph, stmt, target_id)
    }

    /// Execute a LINK statement (edge creation).
    pub fn execute_link(
        &mut self,
        stmt: &LinkStmt,
        target_ids: Vec<EntityId>,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_link(self.registry, self.graph, &self.evaluator, stmt, target_ids)
    }

    /// Execute an UNLINK statement (edge deletion).
    pub fn execute_unlink(
        &mut self,
        stmt: &UnlinkStmt,
        target_id: EdgeId,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_unlink(self.graph, stmt, target_id)
    }

    /// Execute a SET statement (attribute update) for nodes.
    pub fn execute_set(
        &mut self,
        stmt: &SetStmt,
        node_ids: Vec<NodeId>,
        bindings: &Bindings,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_set(
            self.registry,
            self.graph,
            &self.evaluator,
            stmt,
            node_ids,
            bindings,
        )
    }

    /// Execute a SET statement (attribute update) for edges.
    pub fn execute_set_edge(
        &mut self,
        stmt: &SetStmt,
        edge_ids: Vec<EdgeId>,
        bindings: &Bindings,
    ) -> MutationResult<MutationOutcome> {
        ops::execute_set_edge(
            self.registry,
            self.graph,
            &self.evaluator,
            stmt,
            edge_ids,
            bindings,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::{attrs, Value};
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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::MutationError::UnknownType { .. }
        ));
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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::MutationError::MissingRequired { .. }
        ));
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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::MutationError::InvalidAttrType { .. }
        ));
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
        assert!(matches!(
            result.unwrap_err(),
            crate::error::MutationError::NodeNotFound(_)
        ));
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
            if_not_exists: false,
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
            if_not_exists: false,
            span: Span::default(),
        };

        // WHEN
        let result = executor.execute_link(&stmt, vec![person.into()]);

        // THEN
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::MutationError::InvalidArity { .. }
        ));
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
        let edge = graph
            .create_edge(owns_type_id, vec![person.into(), task.into()], attrs! {})
            .unwrap();

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
        assert_eq!(
            node.get_attr("title"),
            Some(&Value::String("Updated".to_string()))
        );
    }
}
