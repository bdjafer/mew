//! Identity types for MEW entities.
//!
//! All identifiers are 64-bit values that are:
//! - Unique within their namespace
//! - Immutable once assigned
//! - Opaque to external users

use std::fmt;

/// Unique identifier for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new NodeId from a raw value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// Unique identifier for an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(pub u64);

impl EdgeId {
    /// Create a new EdgeId from a raw value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// Unified identifier that can refer to either a node or an edge.
/// This is used for edge targets in higher-order hypergraphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityId {
    Node(NodeId),
    Edge(EdgeId),
}

impl EntityId {
    /// Returns true if this is a node ID.
    pub fn is_node(&self) -> bool {
        matches!(self, EntityId::Node(_))
    }

    /// Returns true if this is an edge ID.
    pub fn is_edge(&self) -> bool {
        matches!(self, EntityId::Edge(_))
    }

    /// Get as a NodeId if this is a node reference.
    pub fn as_node(&self) -> Option<NodeId> {
        match self {
            EntityId::Node(id) => Some(*id),
            EntityId::Edge(_) => None,
        }
    }

    /// Get as an EdgeId if this is an edge reference.
    pub fn as_edge(&self) -> Option<EdgeId> {
        match self {
            EntityId::Node(_) => None,
            EntityId::Edge(id) => Some(*id),
        }
    }
}

impl From<NodeId> for EntityId {
    fn from(id: NodeId) -> Self {
        EntityId::Node(id)
    }
}

impl From<EdgeId> for EntityId {
    fn from(id: EdgeId) -> Self {
        EntityId::Edge(id)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityId::Node(id) => write!(f, "{}", id),
            EntityId::Edge(id) => write!(f, "{}", id),
        }
    }
}

/// Identifier for a node type in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Identifier for an edge type in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeTypeId(pub u32);

impl EdgeTypeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for EdgeTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "et{}", self.0)
    }
}

/// Identifier for an attribute within a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttrId(pub u32);

impl AttrId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for AttrId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::new(1);
        let id2 = NodeId::new(1);
        let id3 = NodeId::new(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_edge_id_equality() {
        let id1 = EdgeId::new(1);
        let id2 = EdgeId::new(1);
        let id3 = EdgeId::new(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_entity_id_conversion() {
        let node_id = NodeId::new(42);
        let edge_id = EdgeId::new(99);

        let entity_from_node: EntityId = node_id.into();
        let entity_from_edge: EntityId = edge_id.into();

        assert!(entity_from_node.is_node());
        assert!(!entity_from_node.is_edge());
        assert!(entity_from_edge.is_edge());
        assert!(!entity_from_edge.is_node());

        assert_eq!(entity_from_node.as_node(), Some(node_id));
        assert_eq!(entity_from_edge.as_edge(), Some(edge_id));
    }
}
