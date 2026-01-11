//! Transaction buffer for tracking pending changes.

use std::collections::{HashMap, HashSet};
use mew_core::{Attributes, EdgeId, EdgeTypeId, NodeId, TypeId, Value};

/// A pending node creation.
#[derive(Debug, Clone)]
pub struct PendingNode {
    /// The node ID (allocated but not committed).
    pub id: NodeId,
    /// The node type.
    pub type_id: TypeId,
    /// The node attributes.
    pub attrs: Attributes,
}

/// A pending edge creation.
#[derive(Debug, Clone)]
pub struct PendingEdge {
    /// The edge ID (allocated but not committed).
    pub id: EdgeId,
    /// The edge type.
    pub type_id: EdgeTypeId,
    /// The edge targets.
    pub targets: Vec<mew_core::EntityId>,
    /// The edge attributes.
    pub attrs: Attributes,
}

/// A pending attribute update.
#[derive(Debug, Clone)]
pub struct PendingUpdate {
    /// The node ID.
    pub node_id: NodeId,
    /// The attribute name.
    pub attr_name: String,
    /// The old value (for rollback).
    pub old_value: Option<Value>,
    /// The new value.
    pub new_value: Value,
}

/// Transaction buffer that tracks uncommitted changes.
#[derive(Debug, Clone, Default)]
pub struct TransactionBuffer {
    /// Nodes created in this transaction.
    created_nodes: HashMap<NodeId, PendingNode>,
    /// Nodes deleted in this transaction.
    deleted_nodes: HashSet<NodeId>,
    /// Edges created in this transaction.
    created_edges: HashMap<EdgeId, PendingEdge>,
    /// Edges deleted in this transaction.
    deleted_edges: HashSet<EdgeId>,
    /// Attribute updates in this transaction.
    updates: Vec<PendingUpdate>,
    /// Next node ID to allocate.
    next_node_id: u64,
    /// Next edge ID to allocate.
    next_edge_id: u64,
}

impl TransactionBuffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a buffer starting from given IDs.
    pub fn with_starting_ids(next_node_id: u64, next_edge_id: u64) -> Self {
        Self {
            next_node_id,
            next_edge_id,
            ..Default::default()
        }
    }

    /// Allocate a new node ID.
    pub fn allocate_node_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    /// Allocate a new edge ID.
    pub fn allocate_edge_id(&mut self) -> EdgeId {
        let id = EdgeId::new(self.next_edge_id);
        self.next_edge_id += 1;
        id
    }

    /// Record a node creation (allocates new ID).
    pub fn create_node(&mut self, type_id: TypeId, attrs: Attributes) -> NodeId {
        let id = self.allocate_node_id();
        self.created_nodes.insert(
            id,
            PendingNode {
                id,
                type_id,
                attrs,
            },
        );
        id
    }

    /// Track a node that was created (with externally provided ID).
    pub fn track_created_node(&mut self, id: NodeId, type_id: TypeId, attrs: Attributes) {
        self.created_nodes.insert(
            id,
            PendingNode {
                id,
                type_id,
                attrs,
            },
        );
    }

    /// Track a node deletion (for rollback tracking).
    pub fn track_deleted_node(&mut self, node_id: NodeId) {
        self.deleted_nodes.insert(node_id);
    }

    /// Record a node deletion.
    pub fn delete_node(&mut self, node_id: NodeId) {
        // If this node was created in this transaction, just remove it
        if self.created_nodes.remove(&node_id).is_some() {
            return;
        }
        // Otherwise mark for deletion
        self.deleted_nodes.insert(node_id);
    }

    /// Record an edge creation (allocates new ID).
    pub fn create_edge(
        &mut self,
        type_id: EdgeTypeId,
        targets: Vec<mew_core::EntityId>,
        attrs: Attributes,
    ) -> EdgeId {
        let id = self.allocate_edge_id();
        self.created_edges.insert(
            id,
            PendingEdge {
                id,
                type_id,
                targets,
                attrs,
            },
        );
        id
    }

    /// Track an edge that was created (with externally provided ID).
    pub fn track_created_edge(
        &mut self,
        id: EdgeId,
        type_id: EdgeTypeId,
        targets: Vec<mew_core::EntityId>,
        attrs: Attributes,
    ) {
        self.created_edges.insert(
            id,
            PendingEdge {
                id,
                type_id,
                targets,
                attrs,
            },
        );
    }

    /// Track an edge deletion (for rollback tracking).
    pub fn track_deleted_edge(&mut self, edge_id: EdgeId) {
        self.deleted_edges.insert(edge_id);
    }

    /// Record an edge deletion.
    pub fn delete_edge(&mut self, edge_id: EdgeId) {
        // If this edge was created in this transaction, just remove it
        if self.created_edges.remove(&edge_id).is_some() {
            return;
        }
        // Otherwise mark for deletion
        self.deleted_edges.insert(edge_id);
    }

    /// Record an attribute update.
    pub fn update_attr(&mut self, node_id: NodeId, attr_name: String, old_value: Option<Value>, new_value: Value) {
        // If node was created in this transaction, update it directly
        if let Some(pending) = self.created_nodes.get_mut(&node_id) {
            pending.attrs.insert(attr_name, new_value);
            return;
        }

        self.updates.push(PendingUpdate {
            node_id,
            attr_name,
            old_value,
            new_value,
        });
    }

    /// Check if a node was created in this transaction.
    pub fn is_created_node(&self, node_id: NodeId) -> bool {
        self.created_nodes.contains_key(&node_id)
    }

    /// Check if a node was deleted in this transaction.
    pub fn is_deleted_node(&self, node_id: NodeId) -> bool {
        self.deleted_nodes.contains(&node_id)
    }

    /// Check if an edge was created in this transaction.
    pub fn is_created_edge(&self, edge_id: EdgeId) -> bool {
        self.created_edges.contains_key(&edge_id)
    }

    /// Check if an edge was deleted in this transaction.
    pub fn is_deleted_edge(&self, edge_id: EdgeId) -> bool {
        self.deleted_edges.contains(&edge_id)
    }

    /// Get a created node.
    pub fn get_created_node(&self, node_id: NodeId) -> Option<&PendingNode> {
        self.created_nodes.get(&node_id)
    }

    /// Get a created edge.
    pub fn get_created_edge(&self, edge_id: EdgeId) -> Option<&PendingEdge> {
        self.created_edges.get(&edge_id)
    }

    /// Get the buffered value for an attribute.
    pub fn get_buffered_attr(&self, node_id: NodeId, attr_name: &str) -> Option<&Value> {
        // Check created nodes first
        if let Some(pending) = self.created_nodes.get(&node_id) {
            return pending.attrs.get(attr_name);
        }

        // Check updates (last update wins)
        for update in self.updates.iter().rev() {
            if update.node_id == node_id && update.attr_name == attr_name {
                return Some(&update.new_value);
            }
        }

        None
    }

    /// Get all created nodes.
    pub fn created_nodes(&self) -> impl Iterator<Item = &PendingNode> {
        self.created_nodes.values()
    }

    /// Get all deleted nodes.
    pub fn deleted_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.deleted_nodes.iter().copied()
    }

    /// Get all created edges.
    pub fn created_edges(&self) -> impl Iterator<Item = &PendingEdge> {
        self.created_edges.values()
    }

    /// Get all deleted edges.
    pub fn deleted_edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.deleted_edges.iter().copied()
    }

    /// Get all attribute updates.
    pub fn updates(&self) -> &[PendingUpdate] {
        &self.updates
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.created_nodes.is_empty()
            && self.deleted_nodes.is_empty()
            && self.created_edges.is_empty()
            && self.deleted_edges.is_empty()
            && self.updates.is_empty()
    }

    /// Clear the buffer (for rollback).
    pub fn clear(&mut self) {
        self.created_nodes.clear();
        self.deleted_nodes.clear();
        self.created_edges.clear();
        self.deleted_edges.clear();
        self.updates.clear();
    }

    /// Create a savepoint by cloning current state.
    pub fn savepoint(&self) -> Self {
        self.clone()
    }

    /// Restore from a savepoint.
    pub fn restore(&mut self, savepoint: Self) {
        *self = savepoint;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;

    #[test]
    fn test_buffer_creation() {
        // GIVEN/WHEN
        let buffer = TransactionBuffer::new();

        // THEN
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_create_node() {
        // GIVEN
        let mut buffer = TransactionBuffer::with_starting_ids(100, 200);

        // WHEN
        let node_id = buffer.create_node(TypeId(1), attrs! { "name" => "Test" });

        // THEN
        assert_eq!(node_id.raw(), 100);
        assert!(buffer.is_created_node(node_id));
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_delete_created_node() {
        // GIVEN
        let mut buffer = TransactionBuffer::new();
        let node_id = buffer.create_node(TypeId(1), attrs! {});

        // WHEN - delete node created in same transaction
        buffer.delete_node(node_id);

        // THEN - should be removed entirely
        assert!(!buffer.is_created_node(node_id));
        assert!(!buffer.is_deleted_node(node_id));
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_delete_existing_node() {
        // GIVEN
        let mut buffer = TransactionBuffer::new();
        let existing_node = NodeId::new(42);

        // WHEN
        buffer.delete_node(existing_node);

        // THEN
        assert!(buffer.is_deleted_node(existing_node));
    }

    #[test]
    fn test_update_created_node_attr() {
        // GIVEN
        let mut buffer = TransactionBuffer::new();
        let node_id = buffer.create_node(TypeId(1), attrs! { "name" => "Original" });

        // WHEN
        buffer.update_attr(node_id, "name".to_string(), None, Value::String("Updated".to_string()));

        // THEN - update applied directly to pending node
        let buffered = buffer.get_buffered_attr(node_id, "name");
        assert_eq!(buffered, Some(&Value::String("Updated".to_string())));
    }

    #[test]
    fn test_savepoint_restore() {
        // GIVEN
        let mut buffer = TransactionBuffer::new();
        buffer.create_node(TypeId(1), attrs! {});

        let savepoint = buffer.savepoint();

        buffer.create_node(TypeId(2), attrs! {});

        // WHEN
        buffer.restore(savepoint);

        // THEN
        assert_eq!(buffer.created_nodes().count(), 1);
    }
}
