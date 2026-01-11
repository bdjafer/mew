//! Indexes for efficient graph lookups.

use mew_core::{EdgeId, EdgeTypeId, EntityId, NodeId, TypeId, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Type index: TypeId -> Set<NodeId>
#[derive(Debug, Default)]
pub struct TypeIndex {
    index: HashMap<TypeId, HashSet<NodeId>>,
}

impl TypeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, type_id: TypeId, node_id: NodeId) {
        self.index.entry(type_id).or_default().insert(node_id);
    }

    pub fn remove(&mut self, type_id: TypeId, node_id: NodeId) {
        if let Some(set) = self.index.get_mut(&type_id) {
            set.remove(&node_id);
            if set.is_empty() {
                self.index.remove(&type_id);
            }
        }
    }

    pub fn get(&self, type_id: TypeId) -> impl Iterator<Item = NodeId> + '_ {
        self.index
            .get(&type_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }
}

/// Edge type index: EdgeTypeId -> Set<EdgeId>
#[derive(Debug, Default)]
pub struct EdgeTypeIndex {
    index: HashMap<EdgeTypeId, HashSet<EdgeId>>,
}

impl EdgeTypeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, type_id: EdgeTypeId, edge_id: EdgeId) {
        self.index.entry(type_id).or_default().insert(edge_id);
    }

    pub fn remove(&mut self, type_id: EdgeTypeId, edge_id: EdgeId) {
        if let Some(set) = self.index.get_mut(&type_id) {
            set.remove(&edge_id);
            if set.is_empty() {
                self.index.remove(&type_id);
            }
        }
    }

    pub fn get(&self, type_id: EdgeTypeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.index
            .get(&type_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }
}

/// Key for attribute index: (TypeId, attribute name, value)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttrKey {
    pub type_id: TypeId,
    pub attr_name: String,
    pub value: AttrValue,
}

/// Simplified value for attribute indexing.
/// We only index exact matches for now (no range queries on floats).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AttrValue {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
}

impl AttrValue {
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(AttrValue::Null),
            Value::Bool(b) => Some(AttrValue::Bool(*b)),
            Value::Int(i) => Some(AttrValue::Int(*i)),
            Value::String(s) => Some(AttrValue::String(s.clone())),
            // Float, Timestamp, Duration, NodeRef, EdgeRef are not indexed for exact match
            _ => None,
        }
    }
}

/// Attribute index: (TypeId, attr_name, value) -> Set<NodeId>
#[derive(Debug, Default)]
pub struct AttributeIndex {
    /// Exact match index
    exact: HashMap<AttrKey, HashSet<NodeId>>,
    /// Range index for integers: (TypeId, attr_name) -> BTreeMap<i64, Set<NodeId>>
    range: HashMap<(TypeId, String), BTreeMap<i64, HashSet<NodeId>>>,
}

impl AttributeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, type_id: TypeId, attr_name: &str, value: &Value, node_id: NodeId) {
        // Add to exact index
        if let Some(attr_value) = AttrValue::from_value(value) {
            let key = AttrKey {
                type_id,
                attr_name: attr_name.to_string(),
                value: attr_value,
            };
            self.exact.entry(key).or_default().insert(node_id);
        }

        // Add to range index for integers
        if let Value::Int(i) = value {
            self.range
                .entry((type_id, attr_name.to_string()))
                .or_default()
                .entry(*i)
                .or_default()
                .insert(node_id);
        }
    }

    pub fn remove(&mut self, type_id: TypeId, attr_name: &str, value: &Value, node_id: NodeId) {
        // Remove from exact index
        if let Some(attr_value) = AttrValue::from_value(value) {
            let key = AttrKey {
                type_id,
                attr_name: attr_name.to_string(),
                value: attr_value,
            };
            if let Some(set) = self.exact.get_mut(&key) {
                set.remove(&node_id);
                if set.is_empty() {
                    self.exact.remove(&key);
                }
            }
        }

        // Remove from range index
        if let Value::Int(i) = value {
            let range_key = (type_id, attr_name.to_string());
            if let Some(btree) = self.range.get_mut(&range_key) {
                if let Some(set) = btree.get_mut(i) {
                    set.remove(&node_id);
                    if set.is_empty() {
                        btree.remove(i);
                    }
                }
                if btree.is_empty() {
                    self.range.remove(&range_key);
                }
            }
        }
    }

    pub fn find_exact(
        &self,
        type_id: TypeId,
        attr_name: &str,
        value: &Value,
    ) -> impl Iterator<Item = NodeId> + '_ {
        AttrValue::from_value(value)
            .and_then(|attr_value| {
                let key = AttrKey {
                    type_id,
                    attr_name: attr_name.to_string(),
                    value: attr_value,
                };
                self.exact.get(&key)
            })
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    pub fn find_range(
        &self,
        type_id: TypeId,
        attr_name: &str,
        min: i64,
        max: i64,
    ) -> impl Iterator<Item = NodeId> + '_ {
        let range_key = (type_id, attr_name.to_string());
        self.range
            .get(&range_key)
            .into_iter()
            .flat_map(move |btree| {
                btree
                    .range(min..=max)
                    .flat_map(|(_, set)| set.iter().copied())
            })
    }
}

/// Adjacency index: NodeId -> { outbound: Map<EdgeTypeId, Set<EdgeId>>, inbound: ... }
#[derive(Debug, Default)]
pub struct AdjacencyIndex {
    /// Edges where the node is at position 0 (outbound for binary edges)
    outbound: HashMap<NodeId, HashMap<EdgeTypeId, HashSet<EdgeId>>>,
    /// Edges where the node is at any position > 0 (inbound for binary edges)
    inbound: HashMap<NodeId, HashMap<EdgeTypeId, HashSet<EdgeId>>>,
    /// All edges involving a node (any position)
    all: HashMap<NodeId, HashSet<EdgeId>>,
}

impl AdjacencyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, edge_id: EdgeId, edge_type_id: EdgeTypeId, targets: &[EntityId]) {
        for (pos, target) in targets.iter().enumerate() {
            if let EntityId::Node(node_id) = target {
                // Add to all
                self.all.entry(*node_id).or_default().insert(edge_id);

                // Add to outbound/inbound
                if pos == 0 {
                    self.outbound
                        .entry(*node_id)
                        .or_default()
                        .entry(edge_type_id)
                        .or_default()
                        .insert(edge_id);
                } else {
                    self.inbound
                        .entry(*node_id)
                        .or_default()
                        .entry(edge_type_id)
                        .or_default()
                        .insert(edge_id);
                }
            }
        }
    }

    pub fn remove(&mut self, edge_id: EdgeId, edge_type_id: EdgeTypeId, targets: &[EntityId]) {
        for (pos, target) in targets.iter().enumerate() {
            if let EntityId::Node(node_id) = target {
                // Remove from all
                if let Some(set) = self.all.get_mut(node_id) {
                    set.remove(&edge_id);
                    if set.is_empty() {
                        self.all.remove(node_id);
                    }
                }

                // Remove from outbound/inbound
                let index = if pos == 0 {
                    &mut self.outbound
                } else {
                    &mut self.inbound
                };

                if let Some(type_map) = index.get_mut(node_id) {
                    if let Some(set) = type_map.get_mut(&edge_type_id) {
                        set.remove(&edge_id);
                        if set.is_empty() {
                            type_map.remove(&edge_type_id);
                        }
                    }
                    if type_map.is_empty() {
                        index.remove(node_id);
                    }
                }
            }
        }
    }

    /// Get edges from a node (position 0).
    pub fn edges_from(
        &self,
        node_id: NodeId,
        edge_type: Option<EdgeTypeId>,
    ) -> impl Iterator<Item = EdgeId> + '_ {
        self.outbound
            .get(&node_id)
            .into_iter()
            .flat_map(move |type_map| {
                if let Some(et) = edge_type {
                    type_map
                        .get(&et)
                        .into_iter()
                        .flat_map(|set| set.iter().copied())
                        .collect::<Vec<_>>()
                } else {
                    type_map
                        .values()
                        .flat_map(|set| set.iter().copied())
                        .collect::<Vec<_>>()
                }
            })
    }

    /// Get edges to a node (any position > 0).
    pub fn edges_to(
        &self,
        node_id: NodeId,
        edge_type: Option<EdgeTypeId>,
    ) -> impl Iterator<Item = EdgeId> + '_ {
        self.inbound
            .get(&node_id)
            .into_iter()
            .flat_map(move |type_map| {
                if let Some(et) = edge_type {
                    type_map
                        .get(&et)
                        .into_iter()
                        .flat_map(|set| set.iter().copied())
                        .collect::<Vec<_>>()
                } else {
                    type_map
                        .values()
                        .flat_map(|set| set.iter().copied())
                        .collect::<Vec<_>>()
                }
            })
    }

    /// Get all edges involving a node.
    pub fn edges_involving(&self, node_id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.all
            .get(&node_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }
}

/// Higher-order index: EdgeId -> Set<EdgeId> (edges that target this edge)
#[derive(Debug, Default)]
pub struct HigherOrderIndex {
    index: HashMap<EdgeId, HashSet<EdgeId>>,
}

impl HigherOrderIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, target_edge_id: EdgeId, meta_edge_id: EdgeId) {
        self.index
            .entry(target_edge_id)
            .or_default()
            .insert(meta_edge_id);
    }

    pub fn remove(&mut self, target_edge_id: EdgeId, meta_edge_id: EdgeId) {
        if let Some(set) = self.index.get_mut(&target_edge_id) {
            set.remove(&meta_edge_id);
            if set.is_empty() {
                self.index.remove(&target_edge_id);
            }
        }
    }

    /// Get edges that target the given edge.
    pub fn edges_about(&self, edge_id: EdgeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.index
            .get(&edge_id)
            .into_iter()
            .flat_map(|set| set.iter().copied())
    }

    /// Check if an edge has higher-order edges about it.
    #[allow(dead_code)]
    pub fn has_higher_order(&self, edge_id: EdgeId) -> bool {
        self.index
            .get(&edge_id)
            .map(|set| !set.is_empty())
            .unwrap_or(false)
    }
}
