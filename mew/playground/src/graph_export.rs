use mew_core::{EntityId, Value};
use mew_graph::Graph;
use mew_registry::Registry;
use serde::Serialize;
use std::collections::HashMap;

fn entity_id_to_raw(id: &EntityId) -> u64 {
    match id {
        EntityId::Node(n) => n.raw(),
        EntityId::Edge(e) => e.raw(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeData {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_name: String,
    pub attrs: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EdgeData {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_name: String,
    pub targets: Vec<u64>,
    pub attrs: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<JsonValue>),
}

impl From<&Value> for JsonValue {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(*b),
            Value::Int(i) => JsonValue::Int(*i),
            Value::Float(f) => JsonValue::Float(*f),
            Value::String(s) => JsonValue::String(s.clone()),
            Value::List(items) => JsonValue::List(items.iter().map(JsonValue::from).collect()),
            Value::NodeRef(id) => JsonValue::Int(id.raw() as i64),
            Value::EdgeRef(id) => JsonValue::Int(id.raw() as i64),
            Value::Timestamp(ts) => JsonValue::Int(*ts),
            Value::Duration(d) => JsonValue::Int(*d),
        }
    }
}

pub fn export_full_graph(graph: &Graph, registry: &Registry) -> GraphData {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for id in graph.all_node_ids() {
        if let Some(node) = graph.get_node(id) {
            let type_name = registry
                .get_type(node.type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("Unknown<{}>", node.type_id));
            let attrs: HashMap<String, JsonValue> = node
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                .collect();
            nodes.push(NodeData {
                id: id.raw(),
                type_name,
                attrs,
            });
        }
    }
    for id in graph.all_edge_ids() {
        if let Some(edge) = graph.get_edge(id) {
            let type_name = registry
                .get_edge_type(edge.type_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("Unknown<{}>", edge.type_id));
            let targets: Vec<u64> = edge.targets.iter().map(entity_id_to_raw).collect();
            let attrs: HashMap<String, JsonValue> = edge
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                .collect();
            edges.push(EdgeData {
                id: id.raw(),
                type_name,
                targets,
                attrs,
            });
        }
    }
    GraphData { nodes, edges }
}

pub fn export_nodes_by_ids(graph: &Graph, registry: &Registry, node_ids: &[u64]) -> GraphData {
    use mew_core::NodeId;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_edges = std::collections::HashSet::new();
    for &raw_id in node_ids {
        let id = NodeId::new(raw_id);
        if let Some(node) = graph.get_node(id) {
            let type_name = registry
                .get_type(node.type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("Unknown<{}>", node.type_id));
            let attrs: HashMap<String, JsonValue> = node
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                .collect();
            nodes.push(NodeData {
                id: raw_id,
                type_name,
                attrs,
            });
        }
    }
    let node_id_set: std::collections::HashSet<u64> = node_ids.iter().copied().collect();
    for edge_id in graph.all_edge_ids() {
        if let Some(edge) = graph.get_edge(edge_id) {
            let targets_in_set = edge
                .targets
                .iter()
                .any(|t| node_id_set.contains(&entity_id_to_raw(t)));
            if targets_in_set && !seen_edges.contains(&edge_id) {
                seen_edges.insert(edge_id);
                let type_name = registry
                    .get_edge_type(edge.type_id)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| format!("Unknown<{}>", edge.type_id));
                let targets: Vec<u64> = edge.targets.iter().map(entity_id_to_raw).collect();
                let attrs: HashMap<String, JsonValue> = edge
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                    .collect();
                edges.push(EdgeData {
                    id: edge_id.raw(),
                    type_name,
                    targets,
                    attrs,
                });
            }
        }
    }
    GraphData { nodes, edges }
}

pub fn get_neighbors(graph: &Graph, registry: &Registry, node_ids: &[u64]) -> GraphData {
    use mew_core::NodeId;
    let input_set: std::collections::HashSet<u64> = node_ids.iter().copied().collect();
    let mut neighbor_ids = std::collections::HashSet::new();
    let mut relevant_edges = Vec::new();
    for edge_id in graph.all_edge_ids() {
        if let Some(edge) = graph.get_edge(edge_id) {
            let targets_raw: Vec<u64> = edge.targets.iter().map(entity_id_to_raw).collect();
            let has_input = targets_raw.iter().any(|t| input_set.contains(t));
            if has_input {
                for &t in &targets_raw {
                    if !input_set.contains(&t) {
                        neighbor_ids.insert(t);
                    }
                }
                let type_name = registry
                    .get_edge_type(edge.type_id)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| format!("Unknown<{}>", edge.type_id));
                let attrs: HashMap<String, JsonValue> = edge
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                    .collect();
                relevant_edges.push(EdgeData {
                    id: edge_id.raw(),
                    type_name,
                    targets: targets_raw,
                    attrs,
                });
            }
        }
    }
    let mut nodes = Vec::new();
    for raw_id in neighbor_ids {
        let id = NodeId::new(raw_id);
        if let Some(node) = graph.get_node(id) {
            let type_name = registry
                .get_type(node.type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("Unknown<{}>", node.type_id));
            let attrs: HashMap<String, JsonValue> = node
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                .collect();
            nodes.push(NodeData {
                id: raw_id,
                type_name,
                attrs,
            });
        }
    }
    GraphData {
        nodes,
        edges: relevant_edges,
    }
}
