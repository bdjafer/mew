//! Mutation result types.

use mew_core::{EdgeId, NodeId, Value};
use std::collections::HashMap;

/// Outcome of a mutation operation.
#[derive(Debug, Clone)]
pub enum MutationOutcome {
    /// Created a node.
    Created(CreatedEntity),
    /// Deleted entities.
    Deleted(DeletedEntities),
    /// Updated entities.
    Updated(UpdatedEntities),
    /// Empty result.
    Empty,
}

impl MutationOutcome {
    /// Get created node ID if this is a Created result.
    pub fn created_node(&self) -> Option<NodeId> {
        match self {
            MutationOutcome::Created(e) => e.node_id,
            _ => None,
        }
    }

    /// Get created edge ID if this is a Created result.
    pub fn created_edge(&self) -> Option<EdgeId> {
        match self {
            MutationOutcome::Created(e) => e.edge_id,
            _ => None,
        }
    }

    /// Get deleted node count.
    pub fn deleted_nodes(&self) -> usize {
        match self {
            MutationOutcome::Deleted(d) => d.node_ids.len(),
            _ => 0,
        }
    }

    /// Get deleted edge count.
    pub fn deleted_edges(&self) -> usize {
        match self {
            MutationOutcome::Deleted(d) => d.edge_ids.len(),
            _ => 0,
        }
    }
}

/// Result of creating a node or edge.
#[derive(Debug, Clone, Default)]
pub struct CreatedEntity {
    /// Created node ID (if SPAWN single).
    pub node_id: Option<NodeId>,
    /// Created node IDs (if SPAWN multiple/chained).
    pub node_ids: Vec<NodeId>,
    /// Created edge ID (if LINK).
    pub edge_id: Option<EdgeId>,
    /// Returned attributes.
    pub attributes: HashMap<String, Value>,
}

impl CreatedEntity {
    pub fn node(id: NodeId) -> Self {
        Self {
            node_id: Some(id),
            node_ids: vec![id],
            edge_id: None,
            attributes: HashMap::new(),
        }
    }

    pub fn nodes(ids: Vec<NodeId>) -> Self {
        Self {
            node_id: ids.first().copied(),
            node_ids: ids,
            edge_id: None,
            attributes: HashMap::new(),
        }
    }

    pub fn edge(id: EdgeId) -> Self {
        Self {
            node_id: None,
            node_ids: Vec::new(),
            edge_id: Some(id),
            attributes: HashMap::new(),
        }
    }

    pub fn with_attrs(mut self, attrs: HashMap<String, Value>) -> Self {
        self.attributes = attrs;
        self
    }
}

/// Result of deleting nodes or edges.
#[derive(Debug, Clone, Default)]
pub struct DeletedEntities {
    /// Deleted node IDs.
    pub node_ids: Vec<NodeId>,
    /// Deleted edge IDs.
    pub edge_ids: Vec<EdgeId>,
}

impl DeletedEntities {
    pub fn node(id: NodeId) -> Self {
        Self {
            node_ids: vec![id],
            edge_ids: Vec::new(),
        }
    }

    pub fn edge(id: EdgeId) -> Self {
        Self {
            node_ids: Vec::new(),
            edge_ids: vec![id],
        }
    }

    pub fn nodes(ids: Vec<NodeId>) -> Self {
        Self {
            node_ids: ids,
            edge_ids: Vec::new(),
        }
    }

    pub fn with_cascade_edges(mut self, edge_ids: Vec<EdgeId>) -> Self {
        self.edge_ids = edge_ids;
        self
    }
}

/// Result of updating entities.
#[derive(Debug, Clone, Default)]
pub struct UpdatedEntities {
    /// Updated node IDs.
    pub node_ids: Vec<NodeId>,
    /// Updated edge IDs.
    pub edge_ids: Vec<EdgeId>,
}

impl UpdatedEntities {
    pub fn nodes(ids: Vec<NodeId>) -> Self {
        Self {
            node_ids: ids,
            edge_ids: Vec::new(),
        }
    }

    pub fn edges(ids: Vec<EdgeId>) -> Self {
        Self {
            node_ids: Vec::new(),
            edge_ids: ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_created_entity_node() {
        // GIVEN
        let node_id = NodeId::new(1);
        let result = MutationOutcome::Created(CreatedEntity::node(node_id));

        // THEN
        assert_eq!(result.created_node(), Some(node_id));
        assert_eq!(result.created_edge(), None);
    }

    #[test]
    fn test_deleted_entities() {
        // GIVEN
        let node_ids = vec![NodeId::new(1), NodeId::new(2)];
        let edge_ids = vec![EdgeId::new(1)];
        let result =
            MutationOutcome::Deleted(DeletedEntities::nodes(node_ids).with_cascade_edges(edge_ids));

        // THEN
        assert_eq!(result.deleted_nodes(), 2);
        assert_eq!(result.deleted_edges(), 1);
    }
}
