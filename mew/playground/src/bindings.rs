use crate::completions::{get_ontology_completions, get_statement_completions, CompletionItem};
use crate::graph_export::{export_full_graph, export_nodes_by_ids, get_neighbors, GraphData};
use crate::session::SessionManager;
use mew_parser::{parse_ontology, parse_stmt};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Playground {
    manager: SessionManager,
}

#[derive(Serialize, Deserialize)]
pub struct ParseResult {
    pub success: bool,
    pub errors: Vec<ParseError>,
}

#[derive(Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub result_type: String,
    pub columns: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    pub nodes_created: Option<usize>,
    pub nodes_modified: Option<usize>,
    pub nodes_deleted: Option<usize>,
    pub edges_created: Option<usize>,
    pub edges_modified: Option<usize>,
    pub edges_deleted: Option<usize>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SchemaData {
    pub types: Vec<TypeInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub type_graph: Vec<Connection>,
}

#[derive(Serialize, Deserialize)]
pub struct TypeInfo {
    pub id: u32,
    pub name: String,
    pub attributes: Vec<AttrInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct AttrInfo {
    pub name: String,
    pub type_name: String,
    pub required: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeTypeInfo {
    pub id: u32,
    pub name: String,
    pub arity: usize,
    pub params: Vec<ParamInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub type_constraint: String,
}

#[derive(Serialize, Deserialize)]
pub struct Connection {
    pub edge_type: String,
    pub from_type: String,
    pub to_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub node_count: usize,
    pub edge_count: usize,
    pub type_counts: Vec<(String, usize)>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSessionResult {
    pub success: bool,
    pub session_id: Option<u32>,
    pub error: Option<String>,
}

#[wasm_bindgen]
impl Playground {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            manager: SessionManager::new(),
        }
    }

    #[wasm_bindgen]
    pub fn parse(&self, source: &str, mode: &str) -> JsValue {
        let result = match mode {
            "ontology" => match parse_ontology(source) {
                Ok(_) => ParseResult {
                    success: true,
                    errors: vec![],
                },
                Err(e) => ParseResult {
                    success: false,
                    errors: vec![ParseError {
                        message: e.to_string(),
                        line: 1,
                        column: 1,
                    }],
                },
            },
            "statement" | _ => match parse_stmt(source) {
                Ok(_) => ParseResult {
                    success: true,
                    errors: vec![],
                },
                Err(e) => ParseResult {
                    success: false,
                    errors: vec![ParseError {
                        message: e.to_string(),
                        line: 1,
                        column: 1,
                    }],
                },
            },
        };
        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    #[wasm_bindgen]
    pub fn create_session(&mut self, ontology: &str) -> JsValue {
        match self.manager.create_session(ontology) {
            Ok(id) => serde_wasm_bindgen::to_value(&CreateSessionResult {
                success: true,
                session_id: Some(id),
                error: None,
            })
            .unwrap(),
            Err(e) => serde_wasm_bindgen::to_value(&CreateSessionResult {
                success: false,
                session_id: None,
                error: Some(e),
            })
            .unwrap(),
        }
    }

    #[wasm_bindgen]
    pub fn delete_session(&mut self, session_id: u32) -> bool {
        self.manager.delete_session(session_id)
    }

    #[wasm_bindgen]
    pub fn execute(&mut self, session_id: u32, statement: &str) -> JsValue {
        use mew_session::StatementResult;
        // Use execute_all to support multi-statement input
        let result = self
            .manager
            .with_session(session_id, |session| session.execute_all(statement));
        let execute_result = match result {
            None => ExecuteResult {
                success: false,
                result_type: "error".to_string(),
                columns: None,
                rows: None,
                nodes_created: None,
                nodes_modified: None,
                nodes_deleted: None,
                edges_created: None,
                edges_modified: None,
                edges_deleted: None,
                error: Some("Session not found".to_string()),
            },
            Some(Err(e)) => ExecuteResult {
                success: false,
                result_type: "error".to_string(),
                columns: None,
                rows: None,
                nodes_created: None,
                nodes_modified: None,
                nodes_deleted: None,
                edges_created: None,
                edges_modified: None,
                edges_deleted: None,
                error: Some(e.to_string()),
            },
            Some(Ok(stmt_result)) => match stmt_result {
                StatementResult::Query(qr) => {
                    let rows: Vec<Vec<serde_json::Value>> = qr
                        .rows
                        .iter()
                        .map(|row| row.iter().map(|v| value_to_json(v)).collect())
                        .collect();
                    ExecuteResult {
                        success: true,
                        result_type: "query".to_string(),
                        columns: Some(qr.columns),
                        rows: Some(rows),
                        nodes_created: None,
                        nodes_modified: None,
                        nodes_deleted: None,
                        edges_created: None,
                        edges_modified: None,
                        edges_deleted: None,
                        error: None,
                    }
                }
                StatementResult::Mutation(mr) => ExecuteResult {
                    success: true,
                    result_type: "mutation".to_string(),
                    columns: None,
                    rows: None,
                    nodes_created: Some(mr.nodes_created),
                    nodes_modified: Some(mr.nodes_modified),
                    nodes_deleted: Some(mr.nodes_deleted),
                    edges_created: Some(mr.edges_created),
                    edges_modified: Some(mr.edges_modified),
                    edges_deleted: Some(mr.edges_deleted),
                    error: None,
                },
                StatementResult::Transaction(_) => ExecuteResult {
                    success: true,
                    result_type: "transaction".to_string(),
                    columns: None,
                    rows: None,
                    nodes_created: None,
                    nodes_modified: None,
                    nodes_deleted: None,
                    edges_created: None,
                    edges_modified: None,
                    edges_deleted: None,
                    error: None,
                },
                StatementResult::Mixed { mutations, queries } => {
                    let rows: Vec<Vec<serde_json::Value>> = queries
                        .rows
                        .iter()
                        .map(|row| row.iter().map(|v| value_to_json(v)).collect())
                        .collect();
                    ExecuteResult {
                        success: true,
                        result_type: "mixed".to_string(),
                        columns: Some(queries.columns),
                        rows: Some(rows),
                        nodes_created: Some(mutations.nodes_created),
                        nodes_modified: Some(mutations.nodes_modified),
                        nodes_deleted: Some(mutations.nodes_deleted),
                        edges_created: Some(mutations.edges_created),
                        edges_modified: Some(mutations.edges_modified),
                        edges_deleted: Some(mutations.edges_deleted),
                        error: None,
                    }
                }
                StatementResult::Empty => ExecuteResult {
                    success: true,
                    result_type: "empty".to_string(),
                    columns: None,
                    rows: None,
                    nodes_created: None,
                    nodes_modified: None,
                    nodes_deleted: None,
                    edges_created: None,
                    edges_modified: None,
                    edges_deleted: None,
                    error: None,
                },
            },
        };
        // Use custom serializer to properly handle nested objects
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        execute_result.serialize(&serializer).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_graph(&self, session_id: u32) -> JsValue {
        let result = self
            .manager
            .get_session(session_id)
            .map(|data| export_full_graph(&data.graph, &data.registry));
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        match result {
            Some(graph_data) => graph_data.serialize(&serializer).unwrap(),
            None => GraphData {
                nodes: vec![],
                edges: vec![],
            }
            .serialize(&serializer)
            .unwrap(),
        }
    }

    #[wasm_bindgen]
    pub fn get_nodes(&self, session_id: u32, node_ids: Vec<u64>) -> JsValue {
        let result = self
            .manager
            .get_session(session_id)
            .map(|data| export_nodes_by_ids(&data.graph, &data.registry, &node_ids));
        match result {
            Some(graph_data) => serde_wasm_bindgen::to_value(&graph_data).unwrap(),
            None => serde_wasm_bindgen::to_value(&GraphData {
                nodes: vec![],
                edges: vec![],
            })
            .unwrap(),
        }
    }

    #[wasm_bindgen]
    pub fn get_neighbors(&self, session_id: u32, node_ids: Vec<u64>) -> JsValue {
        let result = self
            .manager
            .get_session(session_id)
            .map(|data| get_neighbors(&data.graph, &data.registry, &node_ids));
        match result {
            Some(graph_data) => serde_wasm_bindgen::to_value(&graph_data).unwrap(),
            None => serde_wasm_bindgen::to_value(&GraphData {
                nodes: vec![],
                edges: vec![],
            })
            .unwrap(),
        }
    }

    #[wasm_bindgen]
    pub fn get_schema(&self, session_id: u32) -> JsValue {
        let result = self
            .manager
            .get_session(session_id)
            .map(|data| build_schema_data(&data.registry));
        match result {
            Some(schema) => serde_wasm_bindgen::to_value(&schema).unwrap(),
            None => serde_wasm_bindgen::to_value(&SchemaData {
                types: vec![],
                edge_types: vec![],
                type_graph: vec![],
            })
            .unwrap(),
        }
    }

    #[wasm_bindgen]
    pub fn get_completions(&self, session_id: Option<u32>, prefix: &str, context: &str) -> JsValue {
        let mut items: Vec<CompletionItem> = Vec::new();
        let mode = if context.contains("ontology") {
            "ontology"
        } else {
            "statement"
        };
        if mode == "ontology" {
            items.extend(get_ontology_completions(prefix, context));
        } else {
            items.extend(get_statement_completions(prefix));
        }
        if let Some(sid) = session_id {
            if let Some(data) = self.manager.get_session(sid) {
                for type_def in data.registry.all_types() {
                    if type_def
                        .name
                        .to_lowercase()
                        .starts_with(&prefix.to_lowercase())
                    {
                        items.push(CompletionItem {
                            label: type_def.name.clone(),
                            kind: crate::completions::CompletionKind::Type,
                            detail: Some("Node type".to_string()),
                            insert_text: Some(type_def.name.clone()),
                        });
                    }
                }
                for edge_def in data.registry.all_edge_types() {
                    if edge_def
                        .name
                        .to_lowercase()
                        .starts_with(&prefix.to_lowercase())
                    {
                        items.push(CompletionItem {
                            label: edge_def.name.clone(),
                            kind: crate::completions::CompletionKind::Type,
                            detail: Some("Edge type".to_string()),
                            insert_text: Some(edge_def.name.clone()),
                        });
                    }
                }
            }
        }
        serde_wasm_bindgen::to_value(&items).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_stats(&self, session_id: u32) -> JsValue {
        let result = self.manager.get_session(session_id).map(|data| {
            let mut type_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for node_id in data.graph.all_node_ids() {
                if let Some(node) = data.graph.get_node(node_id) {
                    let type_name = data
                        .registry
                        .get_type(node.type_id)
                        .map(|t| t.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    *type_counts.entry(type_name).or_insert(0) += 1;
                }
            }
            Stats {
                node_count: data.graph.node_count(),
                edge_count: data.graph.edge_count(),
                type_counts: type_counts.into_iter().collect(),
            }
        });
        match result {
            Some(stats) => serde_wasm_bindgen::to_value(&stats).unwrap(),
            None => serde_wasm_bindgen::to_value(&Stats {
                node_count: 0,
                edge_count: 0,
                type_counts: vec![],
            })
            .unwrap(),
        }
    }
}

impl Default for Playground {
    fn default() -> Self {
        Self::new()
    }
}

fn value_to_json(v: &mew_core::Value) -> serde_json::Value {
    match v {
        mew_core::Value::Null => serde_json::Value::Null,
        mew_core::Value::Bool(b) => serde_json::Value::Bool(*b),
        mew_core::Value::Int(i) => serde_json::json!(*i),
        mew_core::Value::Float(f) => serde_json::json!(*f),
        mew_core::Value::String(s) => serde_json::Value::String(s.clone()),
        mew_core::Value::List(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        mew_core::Value::NodeRef(id) => serde_json::json!({ "_type": "node", "_id": id.raw() }),
        mew_core::Value::EdgeRef(id) => serde_json::json!({ "_type": "edge", "_id": id.raw() }),
        mew_core::Value::Timestamp(ts) => serde_json::json!({ "_type": "timestamp", "value": *ts }),
        mew_core::Value::Duration(d) => serde_json::json!({ "_type": "duration", "value": *d }),
    }
}

fn build_schema_data(registry: &mew_registry::Registry) -> SchemaData {
    let types: Vec<TypeInfo> = registry
        .all_types()
        .map(|t| TypeInfo {
            id: t.id.raw(),
            name: t.name.clone(),
            attributes: t
                .attributes
                .iter()
                .map(|(name, attr)| AttrInfo {
                    name: name.clone(),
                    type_name: attr.type_name.clone(),
                    required: attr.required,
                })
                .collect(),
        })
        .collect();
    let edge_types: Vec<EdgeTypeInfo> = registry
        .all_edge_types()
        .map(|e| EdgeTypeInfo {
            id: e.id.raw(),
            name: e.name.clone(),
            arity: e.params.len(),
            params: e
                .params
                .iter()
                .map(|p| ParamInfo {
                    name: p.name.clone(),
                    type_constraint: p.type_constraint.clone(),
                })
                .collect(),
        })
        .collect();
    let mut type_graph = Vec::new();
    for edge in registry.all_edge_types() {
        if edge.params.len() >= 2 {
            type_graph.push(Connection {
                edge_type: edge.name.clone(),
                from_type: edge.params[0].type_constraint.clone(),
                to_type: edge.params[1].type_constraint.clone(),
            });
        }
    }
    SchemaData {
        types,
        edge_types,
        type_graph,
    }
}
