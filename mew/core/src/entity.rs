//! Entity structures for MEW.
//!
//! Nodes and edges are the two fundamental entity types in a hypergraph.

use crate::{Attributes, EdgeId, EdgeTypeId, EntityId, NodeId, TypeId, Value};

/// A node in the hypergraph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Type of this node (reference to registry).
    pub type_id: TypeId,
    /// MVCC version number.
    pub version: u64,
    /// Attribute values.
    pub attributes: Attributes,
}

impl Node {
    /// Create a new node with the given properties.
    pub fn new(id: NodeId, type_id: TypeId, attributes: Attributes) -> Self {
        Self {
            id,
            type_id,
            version: 1,
            attributes,
        }
    }

    /// Get an attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name)
    }

    /// Set an attribute value.
    pub fn set_attr(&mut self, name: String, value: Value) {
        self.attributes.insert(name, value);
        self.version += 1;
    }

    /// Remove an attribute.
    pub fn remove_attr(&mut self, name: &str) -> Option<Value> {
        let result = self.attributes.remove(name);
        if result.is_some() {
            self.version += 1;
        }
        result
    }
}

/// An edge in the hypergraph.
///
/// Edges connect one or more entities (nodes or other edges).
/// Higher-order edges are edges that target other edges.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Unique identifier for this edge.
    pub id: EdgeId,
    /// Type of this edge (reference to registry).
    pub type_id: EdgeTypeId,
    /// Ordered list of targets (can be NodeId or EdgeId).
    pub targets: Vec<EntityId>,
    /// MVCC version number.
    pub version: u64,
    /// Attribute values.
    pub attributes: Attributes,
}

impl Edge {
    /// Create a new edge with the given properties.
    pub fn new(
        id: EdgeId,
        type_id: EdgeTypeId,
        targets: Vec<EntityId>,
        attributes: Attributes,
    ) -> Self {
        Self {
            id,
            type_id,
            targets,
            version: 1,
            attributes,
        }
    }

    /// Returns true if this is a higher-order edge (targets any edges).
    pub fn is_higher_order(&self) -> bool {
        self.targets.iter().any(|t| t.is_edge())
    }

    /// Get the arity (number of targets) of this edge.
    pub fn arity(&self) -> usize {
        self.targets.len()
    }

    /// Get the target at a specific position.
    pub fn target(&self, position: usize) -> Option<&EntityId> {
        self.targets.get(position)
    }

    /// Get all node IDs that this edge targets.
    pub fn node_targets(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.targets.iter().filter_map(|t| t.as_node())
    }

    /// Get all edge IDs that this edge targets (higher-order).
    pub fn edge_targets(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.targets.iter().filter_map(|t| t.as_edge())
    }

    /// Get an attribute value by name.
    pub fn get_attr(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name)
    }

    /// Set an attribute value.
    pub fn set_attr(&mut self, name: String, value: Value) {
        self.attributes.insert(name, value);
        self.version += 1;
    }

    /// Remove an attribute.
    pub fn remove_attr(&mut self, name: &str) -> Option<Value> {
        let result = self.attributes.remove(name);
        if result.is_some() {
            self.version += 1;
        }
        result
    }

    /// Check if this edge involves a specific entity (node or edge) as a target.
    pub fn involves(&self, entity_id: EntityId) -> bool {
        self.targets.contains(&entity_id)
    }

    /// Check if this edge involves a specific node as a target.
    pub fn involves_node(&self, node_id: NodeId) -> bool {
        self.targets.contains(&EntityId::Node(node_id))
    }

    /// Check if this edge involves a specific edge as a target (higher-order).
    pub fn involves_edge(&self, edge_id: EdgeId) -> bool {
        self.targets.contains(&EntityId::Edge(edge_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrs;

    #[test]
    fn test_node_creation() {
        let node = Node::new(NodeId::new(1), TypeId::new(1), attrs! { "name" => "Alice" });

        assert_eq!(node.id, NodeId::new(1));
        assert_eq!(node.type_id, TypeId::new(1));
        assert_eq!(node.version, 1);
        assert_eq!(node.get_attr("name"), Some(&Value::String("Alice".into())));
    }

    #[test]
    fn test_node_attribute_operations() {
        let mut node = Node::new(NodeId::new(1), TypeId::new(1), attrs!());

        node.set_attr("name".to_string(), Value::String("Alice".into()));
        assert_eq!(node.get_attr("name"), Some(&Value::String("Alice".into())));
        assert_eq!(node.version, 2);

        let removed = node.remove_attr("name");
        assert_eq!(removed, Some(Value::String("Alice".into())));
        assert_eq!(node.version, 3);
    }

    #[test]
    fn test_edge_creation() {
        let targets = vec![
            EntityId::Node(NodeId::new(1)),
            EntityId::Node(NodeId::new(2)),
        ];
        let edge = Edge::new(
            EdgeId::new(1),
            EdgeTypeId::new(1),
            targets,
            attrs! { "weight" => 5i64 },
        );

        assert_eq!(edge.id, EdgeId::new(1));
        assert_eq!(edge.type_id, EdgeTypeId::new(1));
        assert_eq!(edge.arity(), 2);
        assert!(!edge.is_higher_order());
        assert_eq!(edge.get_attr("weight"), Some(&Value::Int(5)));
    }

    #[test]
    fn test_higher_order_edge() {
        let targets = vec![EntityId::Edge(EdgeId::new(1))];
        let edge = Edge::new(EdgeId::new(2), EdgeTypeId::new(2), targets, attrs!());

        assert!(edge.is_higher_order());
        assert_eq!(
            edge.edge_targets().collect::<Vec<_>>(),
            vec![EdgeId::new(1)]
        );
    }

    #[test]
    fn test_edge_involves() {
        let targets = vec![
            EntityId::Node(NodeId::new(1)),
            EntityId::Node(NodeId::new(2)),
        ];
        let edge = Edge::new(EdgeId::new(1), EdgeTypeId::new(1), targets, attrs!());

        assert!(edge.involves_node(NodeId::new(1)));
        assert!(edge.involves_node(NodeId::new(2)));
        assert!(!edge.involves_node(NodeId::new(3)));
        assert!(!edge.involves_edge(EdgeId::new(99)));
    }
}
