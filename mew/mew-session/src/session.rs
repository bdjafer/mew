//! Session manager.

use std::collections::HashMap;
use mew_core::{EntityId, Value};
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{parse_stmt, MatchStmt, Stmt, TxnStmt, TargetRef};
use mew_pattern::Bindings;
use mew_query::QueryExecutor;
use mew_registry::Registry;

use crate::error::{SessionError, SessionResult};
use crate::result::{MutationResult, QueryResult, StatementResult, TransactionResult};

/// Session ID type.
pub type SessionId = u64;

/// A MEW session.
pub struct Session<'r> {
    /// Unique session ID.
    id: SessionId,
    /// The registry (shared).
    registry: &'r Registry,
    /// Session-specific graph.
    graph: Graph,
    /// Auto-commit mode.
    auto_commit: bool,
    /// Whether a transaction is active.
    in_transaction: bool,
    /// Variable bindings (var_name -> EntityId) for mutation targets.
    bindings: HashMap<String, EntityId>,
}

impl<'r> Session<'r> {
    /// Create a new session.
    pub fn new(id: SessionId, registry: &'r Registry) -> Self {
        Self {
            id,
            registry,
            graph: Graph::new(),
            auto_commit: true,
            in_transaction: false,
            bindings: HashMap::new(),
        }
    }

    /// Create a session with an existing graph.
    pub fn with_graph(id: SessionId, registry: &'r Registry, graph: Graph) -> Self {
        Self {
            id,
            registry,
            graph,
            auto_commit: true,
            in_transaction: false,
            bindings: HashMap::new(),
        }
    }

    /// Get the session ID.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Get the registry.
    pub fn registry(&self) -> &Registry {
        self.registry
    }

    /// Get a reference to the graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get a mutable reference to the graph.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Check if auto-commit is enabled.
    pub fn is_auto_commit(&self) -> bool {
        self.auto_commit
    }

    /// Set auto-commit mode.
    pub fn set_auto_commit(&mut self, enabled: bool) {
        self.auto_commit = enabled;
    }

    /// Check if a transaction is active.
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    /// Execute a statement string.
    pub fn execute(&mut self, input: &str) -> SessionResult<StatementResult> {
        // Parse the input
        let stmt = parse_stmt(input)?;

        self.execute_statement(&stmt)
    }

    /// Execute a parsed statement.
    fn execute_statement(&mut self, stmt: &Stmt) -> SessionResult<StatementResult> {
        match stmt {
            Stmt::Match(match_stmt) => {
                let result = self.execute_match(match_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Spawn(spawn_stmt) => {
                let result = self.execute_spawn(spawn_stmt)?;
                Ok(StatementResult::Mutation(result))
            }

            Stmt::Kill(kill_stmt) => {
                let result = self.execute_kill(kill_stmt)?;
                Ok(StatementResult::Mutation(result))
            }

            Stmt::Link(link_stmt) => {
                let result = self.execute_link(link_stmt)?;
                Ok(StatementResult::Mutation(result))
            }

            Stmt::Unlink(unlink_stmt) => {
                let result = self.execute_unlink(unlink_stmt)?;
                Ok(StatementResult::Mutation(result))
            }

            Stmt::Set(set_stmt) => {
                let result = self.execute_set(set_stmt)?;
                Ok(StatementResult::Mutation(result))
            }

            Stmt::Walk(_) => {
                // Walk statements for traversal
                Ok(StatementResult::Empty)
            }

            Stmt::Txn(txn_stmt) => self.execute_txn(txn_stmt),
        }
    }

    /// Execute a MATCH statement.
    fn execute_match(&self, stmt: &MatchStmt) -> SessionResult<QueryResult> {
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_match(stmt)?;

        // Convert to QueryResult
        let columns: Vec<String> = result.column_names().iter().cloned().collect();
        let types = vec!["any".to_string(); columns.len()]; // Simplified

        let mut rows = Vec::new();
        for row in result.rows() {
            let mut values = Vec::with_capacity(columns.len());
            for col in &columns {
                values.push(row.get_by_name(col).cloned().unwrap_or(Value::Null));
            }
            rows.push(values);
        }

        Ok(QueryResult::new(columns, types, rows))
    }

    /// Execute a SPAWN statement.
    fn execute_spawn(&mut self, stmt: &mew_parser::SpawnStmt) -> SessionResult<MutationResult> {
        let pattern_bindings = Bindings::new();
        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_spawn(stmt, &pattern_bindings)?;

        // Store the created node ID with the variable name
        if let Some(node_id) = result.created_node() {
            self.bindings.insert(stmt.var.clone(), node_id.into());
        }

        let nodes = if result.created_node().is_some() { 1 } else { 0 };
        let edges = if result.created_edge().is_some() { 1 } else { 0 };
        Ok(MutationResult::new(nodes, edges))
    }

    /// Execute a KILL statement.
    fn execute_kill(&mut self, stmt: &mew_parser::KillStmt) -> SessionResult<MutationResult> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id.as_node()
            .ok_or_else(|| SessionError::invalid_statement_type("KILL requires a node target"))?;

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_kill(stmt, node_id)?;

        Ok(MutationResult::new(result.deleted_nodes(), result.deleted_edges()))
    }

    /// Execute a LINK statement.
    fn execute_link(&mut self, stmt: &mew_parser::LinkStmt) -> SessionResult<MutationResult> {
        // Resolve all targets
        let mut target_ids = Vec::new();
        for target_ref in &stmt.targets {
            let entity_id = self.resolve_target_ref(target_ref)?;
            target_ids.push(entity_id);
        }

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_link(stmt, target_ids)?;

        // Store the created edge ID with the variable name if present
        if let Some(ref var) = stmt.var {
            if let Some(edge_id) = result.created_edge() {
                self.bindings.insert(var.clone(), edge_id.into());
            }
        }

        let edges = if result.created_edge().is_some() { 1 } else { 0 };
        Ok(MutationResult::new(0, edges))
    }

    /// Execute an UNLINK statement.
    fn execute_unlink(&mut self, stmt: &mew_parser::UnlinkStmt) -> SessionResult<MutationResult> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let edge_id = target_id.as_edge()
            .ok_or_else(|| SessionError::invalid_statement_type("UNLINK requires an edge target"))?;

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_unlink(stmt, edge_id)?;

        Ok(MutationResult::new(0, result.deleted_edges()))
    }

    /// Execute a SET statement.
    fn execute_set(&mut self, stmt: &mew_parser::SetStmt) -> SessionResult<MutationResult> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id.as_node()
            .ok_or_else(|| SessionError::invalid_statement_type("SET requires a node target"))?;

        let pattern_bindings = Bindings::new();
        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_set(stmt, vec![node_id], &pattern_bindings)?;

        // Count updated nodes from the result
        use mew_mutation::MutationOutput;
        let nodes = match result {
            MutationOutput::Updated(ref u) => u.node_ids.len(),
            _ => 0,
        };
        Ok(MutationResult::new(nodes, 0))
    }

    /// Resolve a target to an EntityId.
    fn resolve_target(&self, target: &mew_parser::Target) -> SessionResult<EntityId> {
        match target {
            mew_parser::Target::Var(var_name) => {
                self.bindings.get(var_name)
                    .copied()
                    .ok_or_else(|| SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name)))
            }
            mew_parser::Target::Id(_) | mew_parser::Target::Pattern(_) => {
                Err(SessionError::invalid_statement_type("Only variable targets are supported"))
            }
        }
    }

    /// Resolve a target reference to an EntityId.
    fn resolve_target_ref(&self, target_ref: &TargetRef) -> SessionResult<EntityId> {
        match target_ref {
            TargetRef::Var(var_name) => {
                self.bindings.get(var_name)
                    .copied()
                    .ok_or_else(|| SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name)))
            }
            TargetRef::Id(_) | TargetRef::Pattern(_) => {
                Err(SessionError::invalid_statement_type("Only variable targets are supported"))
            }
        }
    }

    /// Execute a transaction statement.
    fn execute_txn(&mut self, stmt: &TxnStmt) -> SessionResult<StatementResult> {
        match stmt {
            TxnStmt::Begin { .. } => {
                if self.in_transaction {
                    return Err(SessionError::TransactionError(
                        mew_transaction::TransactionError::AlreadyActive,
                    ));
                }
                self.in_transaction = true;
                Ok(StatementResult::Transaction(TransactionResult::Begun))
            }

            TxnStmt::Commit => {
                if !self.in_transaction {
                    return Err(SessionError::TransactionError(
                        mew_transaction::TransactionError::NoActiveTransaction,
                    ));
                }
                self.in_transaction = false;
                Ok(StatementResult::Transaction(TransactionResult::Committed))
            }

            TxnStmt::Rollback => {
                if !self.in_transaction {
                    return Err(SessionError::TransactionError(
                        mew_transaction::TransactionError::NoActiveTransaction,
                    ));
                }
                self.in_transaction = false;
                Ok(StatementResult::Transaction(TransactionResult::RolledBack))
            }
        }
    }
}

/// Session manager for handling multiple sessions.
#[derive(Default)]
pub struct SessionManager {
    /// Next session ID to assign.
    next_id: SessionId,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Allocate a new session ID.
    pub fn alloc_id(&mut self) -> SessionId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Create a new session with the given registry.
    pub fn create_session<'r>(&mut self, registry: &'r Registry) -> Session<'r> {
        let id = self.alloc_id();
        Session::new(id, registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        builder.build().unwrap()
    }

    #[test]
    fn test_session_creation() {
        // GIVEN
        let registry = test_registry();

        // WHEN
        let session = Session::new(1, &registry);

        // THEN
        assert_eq!(session.id(), 1);
        assert!(session.is_auto_commit());
        assert!(!session.in_transaction());
    }

    #[test]
    fn test_session_manager() {
        // GIVEN
        let registry = test_registry();
        let mut manager = SessionManager::new();

        // WHEN
        let session1 = manager.create_session(&registry);
        let session2 = manager.create_session(&registry);

        // THEN
        assert_eq!(session1.id(), 1);
        assert_eq!(session2.id(), 2);
    }

    #[test]
    fn test_begin_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN
        let result = session.execute("BEGIN");

        // THEN
        assert!(result.is_ok());
        assert!(session.in_transaction());
    }

    #[test]
    fn test_commit_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut session = Session::new(1, &registry);
        session.execute("BEGIN").unwrap();

        // WHEN
        let result = session.execute("COMMIT");

        // THEN
        assert!(result.is_ok());
        assert!(!session.in_transaction());
    }

    #[test]
    fn test_rollback_transaction() {
        // GIVEN
        let registry = test_registry();
        let mut session = Session::new(1, &registry);
        session.execute("BEGIN").unwrap();

        // WHEN
        let result = session.execute("ROLLBACK");

        // THEN
        assert!(result.is_ok());
        assert!(!session.in_transaction());
    }

    #[test]
    fn test_commit_without_transaction_fails() {
        // GIVEN
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN
        let result = session.execute("COMMIT");

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_commit_mode() {
        // GIVEN
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN
        session.set_auto_commit(false);

        // THEN
        assert!(!session.is_auto_commit());
    }
}
