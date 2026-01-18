use mew_compiler::compile;
use mew_graph::Graph;
use mew_registry::Registry;
use mew_session::Session;
use std::collections::HashMap;
use std::sync::Arc;

pub type SessionId = u32;

pub struct SessionData {
    pub registry: Arc<Registry>,
    pub graph: Graph,
    pub ontology_source: String,
}

pub struct SessionManager {
    next_id: SessionId,
    sessions: HashMap<SessionId, SessionData>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, ontology_source: &str) -> Result<SessionId, String> {
        let registry = compile(ontology_source).map_err(|e| e.to_string())?;
        let id = self.next_id;
        self.next_id += 1;
        self.sessions.insert(
            id,
            SessionData {
                registry: Arc::new(registry),
                graph: Graph::new(),
                ontology_source: ontology_source.to_string(),
            },
        );
        Ok(id)
    }

    pub fn delete_session(&mut self, id: SessionId) -> bool {
        self.sessions.remove(&id).is_some()
    }

    pub fn get_session(&self, id: SessionId) -> Option<&SessionData> {
        self.sessions.get(&id)
    }

    pub fn with_session<F, R>(&mut self, id: SessionId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session<'_>) -> R,
    {
        if let Some(data) = self.sessions.get_mut(&id) {
            let mut session =
                Session::with_graph(id as u64, &data.registry, std::mem::take(&mut data.graph));
            let result = f(&mut session);
            data.graph = std::mem::take(session.graph_mut());
            Some(result)
        } else {
            None
        }
    }

    pub fn list_sessions(&self) -> Vec<SessionId> {
        self.sessions.keys().copied().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
