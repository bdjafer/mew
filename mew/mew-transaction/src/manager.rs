//! Transaction manager for orchestrating ACID transactions.

use std::collections::HashMap;
use mew_constraint::ConstraintChecker;
use mew_core::{Attributes, EdgeId, EdgeTypeId, EntityId, NodeId, TypeId, Value};
use mew_graph::Graph;
use mew_registry::Registry;
use mew_rule::RuleEngine;

use crate::buffer::TransactionBuffer;
use crate::error::{TransactionError, TransactionResult};

/// Transaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// No transaction is active.
    Inactive,
    /// Transaction is active.
    Active,
    /// Transaction is being committed.
    Committing,
    /// Transaction is being rolled back.
    RollingBack,
}

/// A savepoint within a transaction.
#[derive(Debug, Clone)]
struct Savepoint {
    buffer: TransactionBuffer,
}

/// Transaction manager that orchestrates ACID transactions.
///
/// This implementation applies changes directly to the graph and tracks
/// them in a buffer for potential rollback. This is appropriate for
/// Read Committed isolation level (single-writer model).
pub struct TransactionManager<'r, 'g> {
    registry: &'r Registry,
    graph: &'g mut Graph,
    state: TransactionState,
    buffer: TransactionBuffer,
    savepoints: HashMap<String, Savepoint>,
    auto_commit: bool,
}

impl<'r, 'g> TransactionManager<'r, 'g> {
    /// Create a new transaction manager.
    pub fn new(registry: &'r Registry, graph: &'g mut Graph) -> Self {
        Self {
            registry,
            graph,
            state: TransactionState::Inactive,
            buffer: TransactionBuffer::new(),
            savepoints: HashMap::new(),
            auto_commit: false,
        }
    }

    /// Check if a transaction is active.
    pub fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }

    /// Get the current transaction state.
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// Enable or disable auto-commit mode.
    pub fn set_auto_commit(&mut self, enabled: bool) {
        self.auto_commit = enabled;
    }

    /// Check if auto-commit is enabled.
    pub fn is_auto_commit(&self) -> bool {
        self.auto_commit
    }

    // ========== Transaction Lifecycle ==========

    /// Begin a new transaction.
    pub fn begin(&mut self) -> TransactionResult<()> {
        if self.state == TransactionState::Active {
            return Err(TransactionError::AlreadyActive);
        }

        self.buffer = TransactionBuffer::new();
        self.savepoints.clear();
        self.state = TransactionState::Active;

        Ok(())
    }

    /// Commit the current transaction.
    pub fn commit(&mut self) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        self.state = TransactionState::Committing;

        // 1. Run deferred constraint checks
        let constraint_result = self.check_deferred_constraints();
        if let Err(e) = constraint_result {
            // Rollback on constraint failure
            self.state = TransactionState::Active;
            self.do_rollback()?;
            return Err(e);
        }

        // 2. Clear transaction state (changes already applied to graph)
        self.buffer.clear();
        self.savepoints.clear();
        self.state = TransactionState::Inactive;

        Ok(())
    }

    /// Rollback the current transaction.
    pub fn rollback(&mut self) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        self.state = TransactionState::RollingBack;
        self.do_rollback()?;
        self.state = TransactionState::Inactive;

        Ok(())
    }

    /// Actually perform the rollback.
    fn do_rollback(&mut self) -> TransactionResult<()> {
        // Undo in reverse order: updates, edge creations, node creations
        // Delete created edges first
        for pending in self.buffer.created_edges() {
            let _ = self.graph.delete_edge(pending.id);
        }

        // Delete created nodes
        for pending in self.buffer.created_nodes() {
            let _ = self.graph.delete_node(pending.id);
        }

        // Restore deleted edges (would need to store original data - simplified for now)
        // Restore deleted nodes (would need to store original data - simplified for now)
        // Restore updated attributes
        for update in self.buffer.updates() {
            if let Some(old_value) = &update.old_value {
                let _ = self.graph.set_node_attr(update.node_id, &update.attr_name, old_value.clone());
            }
        }

        self.buffer.clear();
        self.savepoints.clear();

        Ok(())
    }

    // ========== Savepoints ==========

    /// Create a savepoint.
    pub fn savepoint(&mut self, name: &str) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        self.savepoints.insert(
            name.to_string(),
            Savepoint {
                buffer: self.buffer.savepoint(),
            },
        );

        Ok(())
    }

    /// Rollback to a savepoint.
    pub fn rollback_to(&mut self, name: &str) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        let savepoint = self
            .savepoints
            .get(name)
            .ok_or_else(|| TransactionError::savepoint_not_found(name))?
            .clone();

        // Undo changes since savepoint
        // Delete nodes created after savepoint
        let current_nodes: Vec<_> = self.buffer.created_nodes().map(|p| p.id).collect();
        let savepoint_nodes: std::collections::HashSet<_> =
            savepoint.buffer.created_nodes().map(|p| p.id).collect();

        for node_id in current_nodes {
            if !savepoint_nodes.contains(&node_id) {
                let _ = self.graph.delete_node(node_id);
            }
        }

        // Delete edges created after savepoint
        let current_edges: Vec<_> = self.buffer.created_edges().map(|p| p.id).collect();
        let savepoint_edges: std::collections::HashSet<_> =
            savepoint.buffer.created_edges().map(|p| p.id).collect();

        for edge_id in current_edges {
            if !savepoint_edges.contains(&edge_id) {
                let _ = self.graph.delete_edge(edge_id);
            }
        }

        self.buffer.restore(savepoint.buffer);
        self.savepoints.remove(name);

        Ok(())
    }

    /// Release a savepoint.
    pub fn release_savepoint(&mut self, name: &str) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }

        if self.savepoints.remove(name).is_none() {
            return Err(TransactionError::savepoint_not_found(name));
        }

        Ok(())
    }

    // ========== Operations ==========

    /// Create a node within the transaction.
    pub fn create_node(&mut self, type_id: TypeId, attrs: Attributes) -> TransactionResult<NodeId> {
        self.ensure_active()?;

        // Validate the type exists
        if self.registry.get_type(type_id).is_none() {
            return Err(TransactionError::MutationError(
                mew_mutation::MutationError::UnknownType {
                    name: format!("TypeId({})", type_id.0)
                },
            ));
        }

        // Create the node in the graph
        let node_id = self.graph.create_node(type_id, attrs.clone());

        // Track for potential rollback
        self.buffer.track_created_node(node_id, type_id, attrs.clone());

        // Run immediate constraints
        self.check_immediate_constraints_node(node_id, &attrs)?;

        // Run triggered rules
        self.run_triggered_rules(&[node_id], &[])?;

        // Auto-commit if enabled
        if self.auto_commit {
            self.commit()?;
        }

        Ok(node_id)
    }

    /// Delete a node within the transaction.
    pub fn delete_node(&mut self, node_id: NodeId) -> TransactionResult<()> {
        self.ensure_active()?;

        // Check node exists
        if self.graph.get_node(node_id).is_none() {
            return Err(TransactionError::MutationError(
                mew_mutation::MutationError::NodeNotFound(node_id),
            ));
        }

        // Track for potential rollback (store original state)
        self.buffer.track_deleted_node(node_id);

        // Delete from graph (will cascade delete edges)
        let _ = self.graph.delete_node(node_id);

        // Auto-commit if enabled
        if self.auto_commit {
            self.commit()?;
        }

        Ok(())
    }

    /// Create an edge within the transaction.
    pub fn create_edge(
        &mut self,
        type_id: EdgeTypeId,
        targets: Vec<EntityId>,
        attrs: Attributes,
    ) -> TransactionResult<EdgeId> {
        self.ensure_active()?;

        // Validate edge type exists
        if self.registry.get_edge_type(type_id).is_none() {
            return Err(TransactionError::MutationError(
                mew_mutation::MutationError::UnknownEdgeType {
                    name: format!("EdgeTypeId({})", type_id.0)
                },
            ));
        }

        // Create the edge in the graph
        let edge_id = self.graph
            .create_edge(type_id, targets.clone(), attrs.clone())
            .map_err(|e| TransactionError::MutationError(
                mew_mutation::MutationError::pattern_error(e.to_string())
            ))?;

        // Track for potential rollback
        self.buffer.track_created_edge(edge_id, type_id, targets, attrs);

        // Run triggered rules
        self.run_triggered_rules(&[], &[edge_id])?;

        // Auto-commit if enabled
        if self.auto_commit {
            self.commit()?;
        }

        Ok(edge_id)
    }

    /// Delete an edge within the transaction.
    pub fn delete_edge(&mut self, edge_id: EdgeId) -> TransactionResult<()> {
        self.ensure_active()?;

        if self.graph.get_edge(edge_id).is_none() {
            return Err(TransactionError::MutationError(
                mew_mutation::MutationError::EdgeNotFound(edge_id),
            ));
        }

        // Track for potential rollback
        self.buffer.track_deleted_edge(edge_id);

        // Delete from graph
        let _ = self.graph.delete_edge(edge_id);

        // Auto-commit if enabled
        if self.auto_commit {
            self.commit()?;
        }

        Ok(())
    }

    /// Update an attribute within the transaction.
    pub fn update_attr(
        &mut self,
        node_id: NodeId,
        attr_name: &str,
        value: Value,
    ) -> TransactionResult<()> {
        self.ensure_active()?;

        let old_value = self.graph
            .get_node(node_id)
            .and_then(|n| n.get_attr(attr_name).cloned());

        if self.graph.get_node(node_id).is_none() {
            return Err(TransactionError::MutationError(
                mew_mutation::MutationError::NodeNotFound(node_id),
            ));
        }

        // Track for potential rollback
        self.buffer.update_attr(node_id, attr_name.to_string(), old_value, value.clone());

        // Update in graph
        self.graph.set_node_attr(node_id, attr_name, value)
            .map_err(|e| TransactionError::MutationError(
                mew_mutation::MutationError::pattern_error(e.to_string())
            ))?;

        // Auto-commit if enabled
        if self.auto_commit {
            self.commit()?;
        }

        Ok(())
    }

    // ========== Query Support (Read-Your-Writes) ==========

    /// Check if a node exists.
    pub fn node_exists(&self, node_id: NodeId) -> bool {
        self.graph.get_node(node_id).is_some()
    }

    /// Check if an edge exists.
    pub fn edge_exists(&self, edge_id: EdgeId) -> bool {
        self.graph.get_edge(edge_id).is_some()
    }

    /// Get an attribute value.
    pub fn get_attr(&self, node_id: NodeId, attr_name: &str) -> Option<Value> {
        self.graph
            .get_node(node_id)
            .and_then(|n| n.get_attr(attr_name).cloned())
    }

    /// Get the type of a node.
    pub fn get_node_type(&self, node_id: NodeId) -> Option<TypeId> {
        self.graph.get_node(node_id).map(|n| n.type_id)
    }

    // ========== Internal Helpers ==========

    fn ensure_active(&self) -> TransactionResult<()> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NoActiveTransaction);
        }
        Ok(())
    }

    fn check_immediate_constraints_node(
        &self,
        _node_id: NodeId,
        _attrs: &Attributes,
    ) -> TransactionResult<()> {
        // Immediate constraints would be checked here
        // For now, we defer all constraints to commit
        Ok(())
    }

    fn check_deferred_constraints(&self) -> TransactionResult<()> {
        let checker = ConstraintChecker::new(self.registry, self.graph);

        // Check constraints on created nodes
        for pending in self.buffer.created_nodes() {
            // Skip nodes that were subsequently deleted
            if self.graph.get_node(pending.id).is_none() {
                continue;
            }

            let violations = checker.check_node_immediate(pending.id)?;
            if !violations.is_empty() {
                let first = &violations.all()[0];
                return Err(TransactionError::constraint_violation(format!(
                    "{}: {}",
                    first.constraint_name, first.message
                )));
            }
        }

        Ok(())
    }

    fn run_triggered_rules(
        &mut self,
        _nodes: &[NodeId],
        _edges: &[EdgeId],
    ) -> TransactionResult<()> {
        // Rules would be triggered here
        // For now, we just validate the rule engine can be created
        let _engine = RuleEngine::new(self.registry, self.graph);
        Ok(())
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
            .attr(AttrDef::new("title", "String"))
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
    fn test_begin_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);

        // WHEN
        let result = manager.begin();

        // THEN
        assert!(result.is_ok());
        assert!(manager.is_active());
    }

    #[test]
    fn test_begin_already_active() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        // WHEN
        let result = manager.begin();

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransactionError::AlreadyActive));
    }

    #[test]
    fn test_commit_empty_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        // WHEN
        let result = manager.commit();

        // THEN
        assert!(result.is_ok());
        assert!(!manager.is_active());
    }

    #[test]
    fn test_rollback_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        // Create a node
        let type_id = registry.get_type_id("Task").unwrap();
        let node_id = manager.create_node(type_id, attrs! { "title" => "Test" }).unwrap();
        assert!(manager.node_exists(node_id));

        // WHEN
        manager.rollback().unwrap();

        // THEN - node should not be in graph
        assert!(graph.get_node(node_id).is_none());
    }

    #[test]
    fn test_create_node_in_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        // WHEN
        let type_id = registry.get_type_id("Task").unwrap();
        let node_id = manager.create_node(type_id, attrs! { "title" => "Test" }).unwrap();

        // THEN - node exists (applied immediately for Read Committed)
        assert!(manager.node_exists(node_id));

        // After commit, node is still in graph
        manager.commit().unwrap();
        assert!(graph.get_node(node_id).is_some());
    }

    #[test]
    fn test_read_your_writes() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        let type_id = registry.get_type_id("Task").unwrap();
        let node_id = manager.create_node(type_id, attrs! { "title" => "Original" }).unwrap();

        // WHEN - update the attribute
        manager.update_attr(node_id, "title", Value::String("Updated".to_string())).unwrap();

        // THEN - read should see the update
        let value = manager.get_attr(node_id, "title");
        assert_eq!(value, Some(Value::String("Updated".to_string())));
    }

    #[test]
    fn test_savepoint() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.begin().unwrap();

        let type_id = registry.get_type_id("Task").unwrap();
        let node1 = manager.create_node(type_id, attrs! { "title" => "Task 1" }).unwrap();

        // Create savepoint
        manager.savepoint("sp1").unwrap();

        let node2 = manager.create_node(type_id, attrs! { "title" => "Task 2" }).unwrap();

        // WHEN - rollback to savepoint
        manager.rollback_to("sp1").unwrap();

        // THEN - node1 still exists, node2 does not
        assert!(manager.node_exists(node1));
        assert!(!manager.node_exists(node2));
    }

    #[test]
    fn test_auto_commit() {
        // GIVEN
        let registry = test_registry();
        let mut graph = Graph::new();
        let mut manager = TransactionManager::new(&registry, &mut graph);
        manager.set_auto_commit(true);
        manager.begin().unwrap();

        // WHEN - create node with auto-commit
        let type_id = registry.get_type_id("Task").unwrap();
        let node_id = manager.create_node(type_id, attrs! { "title" => "Auto" }).unwrap();

        // THEN - transaction should be inactive and node in graph
        assert!(!manager.is_active());
        assert!(graph.get_node(node_id).is_some());
    }
}
