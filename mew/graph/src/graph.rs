//! Core graph storage implementation.

use crate::index::{AdjacencyIndex, AttributeIndex, EdgeTypeIndex, HigherOrderIndex, TypeIndex};
use mew_core::{
    Attributes, Edge, EdgeId, EdgeTypeId, EntityId, GraphError, GraphResult, Node, NodeId, TypeId,
    Value,
};
use std::collections::HashMap;

/// ID allocator for nodes and edges.
#[derive(Debug, Default)]
struct IdAllocator {
    next_node_id: u64,
    next_edge_id: u64,
}

impl IdAllocator {
    fn new() -> Self {
        Self {
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    fn alloc_edge_id(&mut self) -> EdgeId {
        let id = EdgeId::new(self.next_edge_id);
        self.next_edge_id += 1;
        id
    }
}

/// The in-memory graph storage.
#[derive(Debug)]
pub struct Graph {
    /// Node storage
    nodes: HashMap<NodeId, Node>,
    /// Edge storage
    edges: HashMap<EdgeId, Edge>,
    /// ID allocator
    id_alloc: IdAllocator,
    /// Type index
    type_index: TypeIndex,
    /// Edge type index
    edge_type_index: EdgeTypeIndex,
    /// Attribute index
    attr_index: AttributeIndex,
    /// Adjacency index
    adj_index: AdjacencyIndex,
    /// Higher-order index
    ho_index: HigherOrderIndex,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            id_alloc: IdAllocator::new(),
            type_index: TypeIndex::new(),
            edge_type_index: EdgeTypeIndex::new(),
            attr_index: AttributeIndex::new(),
            adj_index: AdjacencyIndex::new(),
            ho_index: HigherOrderIndex::new(),
        }
    }

    // ==================== Node Operations ====================

    /// Create a new node with the given type and attributes.
    pub fn create_node(&mut self, type_id: TypeId, attributes: Attributes) -> NodeId {
        let id = self.id_alloc.alloc_node_id();
        let node = Node::new(id, type_id, attributes);

        // Update indexes
        self.type_index.insert(type_id, id);
        for (attr_name, value) in &node.attributes {
            self.attr_index.insert(type_id, attr_name, value, id);
        }

        self.nodes.insert(id, node);
        id
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Delete a node and all edges involving it.
    pub fn delete_node(&mut self, id: NodeId) -> GraphResult<()> {
        let node = self.nodes.get(&id).ok_or(GraphError::NodeNotFound(id))?;
        let type_id = node.type_id;

        // First, collect all edges that need to be deleted
        let edges_to_delete: Vec<EdgeId> = self.adj_index.edges_involving(id).collect();

        // Delete all edges involving this node
        for edge_id in edges_to_delete {
            self.delete_edge(edge_id)?;
        }

        // Now remove the node
        let node = self.nodes.remove(&id).unwrap();

        // Update indexes
        self.type_index.remove(type_id, id);
        for (attr_name, value) in &node.attributes {
            self.attr_index.remove(type_id, attr_name, value, id);
        }

        Ok(())
    }

    /// Set an attribute on a node.
    pub fn set_node_attr(&mut self, id: NodeId, attr_name: &str, value: Value) -> GraphResult<()> {
        let node = self
            .nodes
            .get_mut(&id)
            .ok_or(GraphError::NodeNotFound(id))?;
        let type_id = node.type_id;

        // Remove old value from index
        if let Some(old_value) = node.attributes.get(attr_name) {
            self.attr_index.remove(type_id, attr_name, old_value, id);
        }

        // Add new value to index
        self.attr_index.insert(type_id, attr_name, &value, id);

        // Update node
        node.set_attr(attr_name.to_string(), value);

        Ok(())
    }

    // ==================== Edge Operations ====================

    /// Create a new edge with the given type, targets, and attributes.
    pub fn create_edge(
        &mut self,
        type_id: EdgeTypeId,
        targets: Vec<EntityId>,
        attributes: Attributes,
    ) -> GraphResult<EdgeId> {
        // Validate that all targets exist
        for target in &targets {
            match target {
                EntityId::Node(node_id) => {
                    if !self.nodes.contains_key(node_id) {
                        return Err(GraphError::NodeNotFound(*node_id));
                    }
                }
                EntityId::Edge(edge_id) => {
                    if !self.edges.contains_key(edge_id) {
                        return Err(GraphError::EdgeNotFound(*edge_id));
                    }
                }
            }
        }

        let id = self.id_alloc.alloc_edge_id();
        let edge = Edge::new(id, type_id, targets.clone(), attributes);

        // Update indexes
        self.edge_type_index.insert(type_id, id);
        self.adj_index.insert(id, type_id, &targets);

        // Update higher-order index for edge targets
        for target in &targets {
            if let EntityId::Edge(target_edge_id) = target {
                self.ho_index.insert(*target_edge_id, id);
            }
        }

        self.edges.insert(id, edge);
        Ok(id)
    }

    /// Get an edge by ID.
    pub fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    /// Get a mutable reference to an edge by ID.
    pub fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut Edge> {
        self.edges.get_mut(&id)
    }

    /// Delete an edge and all higher-order edges about it.
    pub fn delete_edge(&mut self, id: EdgeId) -> GraphResult<()> {
        let edge = self.edges.get(&id).ok_or(GraphError::EdgeNotFound(id))?;
        let type_id = edge.type_id;
        let targets = edge.targets.clone();

        // First, collect all higher-order edges that need to be deleted
        let higher_order_to_delete: Vec<EdgeId> = self.ho_index.edges_about(id).collect();

        // Delete all higher-order edges
        for ho_edge_id in higher_order_to_delete {
            self.delete_edge(ho_edge_id)?;
        }

        // Now remove the edge
        self.edges.remove(&id);

        // Update indexes
        self.edge_type_index.remove(type_id, id);
        self.adj_index.remove(id, type_id, &targets);

        // Remove from higher-order index if this edge targeted other edges
        for target in &targets {
            if let EntityId::Edge(target_edge_id) = target {
                self.ho_index.remove(*target_edge_id, id);
            }
        }

        Ok(())
    }

    /// Set an attribute on an edge.
    pub fn set_edge_attr(&mut self, id: EdgeId, attr_name: &str, value: Value) -> GraphResult<()> {
        let edge = self
            .edges
            .get_mut(&id)
            .ok_or(GraphError::EdgeNotFound(id))?;

        edge.set_attr(attr_name.to_string(), value);

        Ok(())
    }

    // ==================== Query Operations ====================

    /// Find nodes by type.
    pub fn nodes_by_type(&self, type_id: TypeId) -> impl Iterator<Item = NodeId> + '_ {
        self.type_index.get(type_id)
    }

    /// Find nodes by attribute value (exact match).
    pub fn nodes_by_attr(
        &self,
        type_id: TypeId,
        attr_name: &str,
        value: &Value,
    ) -> impl Iterator<Item = NodeId> + '_ {
        self.attr_index.find_exact(type_id, attr_name, value)
    }

    /// Find nodes by attribute range (integers only).
    pub fn nodes_by_attr_range(
        &self,
        type_id: TypeId,
        attr_name: &str,
        min: i64,
        max: i64,
    ) -> impl Iterator<Item = NodeId> + '_ {
        self.attr_index.find_range(type_id, attr_name, min, max)
    }

    /// Find edges by type.
    pub fn edges_by_type(&self, type_id: EdgeTypeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.edge_type_index.get(type_id)
    }

    /// Find edges from a node (where node is at position 0).
    pub fn edges_from(
        &self,
        node_id: NodeId,
        edge_type: Option<EdgeTypeId>,
    ) -> impl Iterator<Item = EdgeId> + '_ {
        self.adj_index.edges_from(node_id, edge_type)
    }

    /// Find edges to a node (where node is at any position > 0).
    pub fn edges_to(
        &self,
        node_id: NodeId,
        edge_type: Option<EdgeTypeId>,
    ) -> impl Iterator<Item = EdgeId> + '_ {
        self.adj_index.edges_to(node_id, edge_type)
    }

    /// Find edges about an edge (higher-order edges targeting this edge).
    pub fn edges_about(&self, edge_id: EdgeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.ho_index.edges_about(edge_id)
    }

    // ==================== Statistics ====================

    /// Get the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all node IDs.
    pub fn all_node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Get all edge IDs.
    pub fn all_edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.edges.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::attrs;

    // ========== TEST: create_node_returns_unique_id ==========
    #[test]
    fn test_create_node_returns_unique_id() {
        // GIVEN empty graph
        let mut graph = Graph::new();

        // WHEN create node with type=1, attrs={name: "Alice"}
        let id = graph.create_node(TypeId::new(1), attrs! { "name" => "Alice" });

        // THEN returns NodeId
        // AND get_node(id) returns node with type=1, name="Alice"
        let node = graph.get_node(id).expect("Node should exist");
        assert_eq!(node.type_id, TypeId::new(1));
        assert_eq!(node.get_attr("name"), Some(&Value::String("Alice".into())));
    }

    // ========== TEST: create_multiple_nodes_unique_ids ==========
    #[test]
    fn test_create_multiple_nodes_unique_ids() {
        // GIVEN empty graph
        let mut graph = Graph::new();

        // WHEN create node A AND create node B
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());

        // THEN A.id != B.id
        assert_ne!(id_a, id_b);
    }

    // ========== TEST: get_nonexistent_node_returns_none ==========
    #[test]
    fn test_get_nonexistent_node_returns_none() {
        // GIVEN empty graph
        let graph = Graph::new();

        // WHEN get_node(NodeId(999))
        // THEN returns None
        assert!(graph.get_node(NodeId::new(999)).is_none());
    }

    // ========== TEST: delete_node_removes_it ==========
    #[test]
    fn test_delete_node_removes_it() {
        // GIVEN graph with node A
        let mut graph = Graph::new();
        let id = graph.create_node(TypeId::new(1), attrs!());

        // WHEN delete_node(A.id)
        graph.delete_node(id).expect("Delete should succeed");

        // THEN get_node(A.id) returns None
        assert!(graph.get_node(id).is_none());
    }

    // ========== TEST: delete_node_cascades_to_edges ==========
    #[test]
    fn test_delete_node_cascades_to_edges() {
        // GIVEN graph with nodes A, B AND edge E from A to B
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let edge_id = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .expect("Edge creation should succeed");

        // WHEN delete_node(A.id)
        graph.delete_node(id_a).expect("Delete should succeed");

        // THEN get_node(A.id) returns None AND get_edge(E.id) returns None
        assert!(graph.get_node(id_a).is_none());
        assert!(graph.get_edge(edge_id).is_none());
    }

    // ========== TEST: set_attribute_updates_value ==========
    #[test]
    fn test_set_attribute_updates_value() {
        // GIVEN graph with node A where name="Alice"
        let mut graph = Graph::new();
        let id = graph.create_node(TypeId::new(1), attrs! { "name" => "Alice" });

        // WHEN set_attr(A.id, "name", "Bob")
        graph
            .set_node_attr(id, "name", Value::String("Bob".into()))
            .expect("Set attr should succeed");

        // THEN get_node(A.id).attrs["name"] == "Bob"
        let node = graph.get_node(id).expect("Node should exist");
        assert_eq!(node.get_attr("name"), Some(&Value::String("Bob".into())));
    }

    // ========== TEST: create_edge_returns_unique_id ==========
    #[test]
    fn test_create_edge_returns_unique_id() {
        // GIVEN graph with nodes A, B
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());

        // WHEN create edge with type=1, targets=[A.id, B.id]
        let edge_id = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .expect("Edge creation should succeed");

        // THEN returns EdgeId AND get_edge(id) returns edge with targets=[A.id, B.id]
        let edge = graph.get_edge(edge_id).expect("Edge should exist");
        assert_eq!(edge.targets.len(), 2);
        assert_eq!(edge.targets[0], EntityId::Node(id_a));
        assert_eq!(edge.targets[1], EntityId::Node(id_b));
    }

    // ========== TEST: create_higher_order_edge ==========
    #[test]
    fn test_create_higher_order_edge() {
        // GIVEN graph with nodes A, B AND edge E1 connecting A to B
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let e1_id = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .expect("Edge creation should succeed");

        // WHEN create edge E2 with targets=[E1.id]
        let e2_id = graph
            .create_edge(EdgeTypeId::new(2), vec![EntityId::Edge(e1_id)], attrs!())
            .expect("Edge creation should succeed");

        // THEN get_edge(E2.id).targets == [E1.id] AND E2 is marked as higher_order
        let e2 = graph.get_edge(e2_id).expect("Edge should exist");
        assert_eq!(e2.targets, vec![EntityId::Edge(e1_id)]);
        assert!(e2.is_higher_order());
    }

    // ========== TEST: delete_edge_removes_it ==========
    #[test]
    fn test_delete_edge_removes_it() {
        // GIVEN graph with edge E
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let edge_id = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .expect("Edge creation should succeed");

        // WHEN delete_edge(E.id)
        graph.delete_edge(edge_id).expect("Delete should succeed");

        // THEN get_edge(E.id) returns None
        assert!(graph.get_edge(edge_id).is_none());
    }

    // ========== TEST: delete_edge_cascades_to_higher_order ==========
    #[test]
    fn test_delete_edge_cascades_to_higher_order() {
        // GIVEN graph with edge E1 AND higher-order edge E2 about E1
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let e1_id = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .expect("Edge creation should succeed");
        let e2_id = graph
            .create_edge(EdgeTypeId::new(2), vec![EntityId::Edge(e1_id)], attrs!())
            .expect("Edge creation should succeed");

        // WHEN delete_edge(E1.id)
        graph.delete_edge(e1_id).expect("Delete should succeed");

        // THEN get_edge(E1.id) returns None AND get_edge(E2.id) returns None
        assert!(graph.get_edge(e1_id).is_none());
        assert!(graph.get_edge(e2_id).is_none());
    }

    // ========== TEST: find_nodes_by_type ==========
    #[test]
    fn test_find_nodes_by_type() {
        // GIVEN graph with: node A type=1, node B type=1, node C type=2
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let _id_c = graph.create_node(TypeId::new(2), attrs!());

        // WHEN find_by_type(1)
        let mut result: Vec<NodeId> = graph.nodes_by_type(TypeId::new(1)).collect();
        result.sort();

        // THEN returns [A, B] (order unspecified)
        let mut expected = vec![id_a, id_b];
        expected.sort();
        assert_eq!(result, expected);
    }

    // ========== TEST: find_nodes_by_type_empty ==========
    #[test]
    fn test_find_nodes_by_type_empty() {
        // GIVEN graph with node A type=1
        let mut graph = Graph::new();
        let _id = graph.create_node(TypeId::new(1), attrs!());

        // WHEN find_by_type(2)
        let result: Vec<NodeId> = graph.nodes_by_type(TypeId::new(2)).collect();

        // THEN returns []
        assert!(result.is_empty());
    }

    // ========== TEST: find_nodes_by_attribute_value ==========
    #[test]
    fn test_find_nodes_by_attribute_value() {
        // GIVEN graph with nodes with status attribute
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs! { "status" => "active" });
        let _id_b = graph.create_node(TypeId::new(1), attrs! { "status" => "inactive" });
        let id_c = graph.create_node(TypeId::new(1), attrs! { "status" => "active" });

        // WHEN find_by_attr(type=1, attr="status", value="active")
        let mut result: Vec<NodeId> = graph
            .nodes_by_attr(TypeId::new(1), "status", &Value::String("active".into()))
            .collect();
        result.sort();

        // THEN returns [A, C]
        let mut expected = vec![id_a, id_c];
        expected.sort();
        assert_eq!(result, expected);
    }

    // ========== TEST: find_nodes_by_attribute_range ==========
    #[test]
    fn test_find_nodes_by_attribute_range() {
        // GIVEN graph with nodes with priority attribute
        let mut graph = Graph::new();
        let _id_a = graph.create_node(TypeId::new(1), attrs! { "priority" => 1i64 });
        let id_b = graph.create_node(TypeId::new(1), attrs! { "priority" => 5i64 });
        let _id_c = graph.create_node(TypeId::new(1), attrs! { "priority" => 10i64 });

        // WHEN find_by_attr_range(attr="priority", min=3, max=7)
        let result: Vec<NodeId> = graph
            .nodes_by_attr_range(TypeId::new(1), "priority", 3, 7)
            .collect();

        // THEN returns [B]
        assert_eq!(result, vec![id_b]);
    }

    // ========== TEST: find_edges_from_node ==========
    #[test]
    fn test_find_edges_from_node() {
        // GIVEN graph with nodes A, B, C and edges E1 from A to B, E2 from A to C, E3 from B to C
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let id_c = graph.create_node(TypeId::new(1), attrs!());
        let e1 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .unwrap();
        let e2 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_c)],
                attrs!(),
            )
            .unwrap();
        let _e3 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_b), EntityId::Node(id_c)],
                attrs!(),
            )
            .unwrap();

        // WHEN find_edges_from(A.id)
        let mut result: Vec<EdgeId> = graph.edges_from(id_a, None).collect();
        result.sort();

        // THEN returns [E1, E2]
        let mut expected = vec![e1, e2];
        expected.sort();
        assert_eq!(result, expected);
    }

    // ========== TEST: find_edges_to_node ==========
    #[test]
    fn test_find_edges_to_node() {
        // GIVEN graph with nodes A, B, C and edges E1 from A to C, E2 from B to C
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let id_c = graph.create_node(TypeId::new(1), attrs!());
        let e1 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_c)],
                attrs!(),
            )
            .unwrap();
        let e2 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_b), EntityId::Node(id_c)],
                attrs!(),
            )
            .unwrap();

        // WHEN find_edges_to(C.id)
        let mut result: Vec<EdgeId> = graph.edges_to(id_c, None).collect();
        result.sort();

        // THEN returns [E1, E2]
        let mut expected = vec![e1, e2];
        expected.sort();
        assert_eq!(result, expected);
    }

    // ========== TEST: find_edges_about_edge ==========
    #[test]
    fn test_find_edges_about_edge() {
        // GIVEN graph with edge E1 (base), E2 about E1, E3 about E1, E4 about E2
        let mut graph = Graph::new();
        let id_a = graph.create_node(TypeId::new(1), attrs!());
        let id_b = graph.create_node(TypeId::new(1), attrs!());
        let e1 = graph
            .create_edge(
                EdgeTypeId::new(1),
                vec![EntityId::Node(id_a), EntityId::Node(id_b)],
                attrs!(),
            )
            .unwrap();
        let e2 = graph
            .create_edge(EdgeTypeId::new(2), vec![EntityId::Edge(e1)], attrs!())
            .unwrap();
        let e3 = graph
            .create_edge(EdgeTypeId::new(2), vec![EntityId::Edge(e1)], attrs!())
            .unwrap();
        let _e4 = graph
            .create_edge(EdgeTypeId::new(2), vec![EntityId::Edge(e2)], attrs!())
            .unwrap();

        // WHEN find_edges_about(E1.id)
        let mut result: Vec<EdgeId> = graph.edges_about(e1).collect();
        result.sort();

        // THEN returns [E2, E3]
        let mut expected = vec![e2, e3];
        expected.sort();
        assert_eq!(result, expected);
    }
}
