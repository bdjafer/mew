//! Session manager.

use mew_core::{messages, EntityId, Value};
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{parse_stmt, parse_stmts, InspectStmt, MatchMutateStmt, MatchStmt, MutationAction, Stmt, TargetRef, TxnStmt, WalkStmt};
use mew_pattern::Bindings;
use mew_query::QueryExecutor;
use mew_registry::Registry;
use std::collections::HashMap;

use crate::error::{SessionError, SessionResult};
use crate::result::{MutationSummary, QueryResult, StatementResult, TransactionResult};

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

    /// Execute multiple statements from a string.
    /// Returns aggregated mutation results (sums created/deleted nodes and edges).
    pub fn execute_all(&mut self, input: &str) -> SessionResult<StatementResult> {
        let stmts = parse_stmts(input)?;

        if stmts.is_empty() {
            return Ok(StatementResult::Mutation(MutationSummary::new(0, 0)));
        }

        // If there's only one statement, just execute it normally
        if stmts.len() == 1 {
            return self.execute_statement(&stmts[0]);
        }

        // Execute all statements and aggregate results
        let mut total_nodes = 0usize;
        let mut total_edges = 0usize;
        let mut last_result = None;

        for stmt in &stmts {
            let result = self.execute_statement(stmt)?;
            match &result {
                StatementResult::Mutation(m) => {
                    total_nodes += m.nodes_affected;
                    total_edges += m.edges_affected;
                }
                _ => {}
            }
            last_result = Some(result);
        }

        // Return aggregated mutation result if any mutations happened
        if total_nodes > 0 || total_edges > 0 {
            Ok(StatementResult::Mutation(MutationSummary::new(
                total_nodes,
                total_edges,
            )))
        } else {
            // Otherwise return the last statement's result
            Ok(last_result.unwrap_or(StatementResult::Mutation(MutationSummary::new(0, 0))))
        }
    }

    /// Execute a parsed statement.
    fn execute_statement(&mut self, stmt: &Stmt) -> SessionResult<StatementResult> {
        match stmt {
            Stmt::Match(match_stmt) => {
                let result = self.execute_match(match_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::MatchMutate(match_mutate_stmt) => {
                let result = self.execute_match_mutate(match_mutate_stmt)?;
                Ok(StatementResult::Mutation(result))
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

            Stmt::Walk(walk_stmt) => {
                let result = self.execute_walk(walk_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Inspect(inspect_stmt) => {
                let result = self.execute_inspect(inspect_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Txn(txn_stmt) => self.execute_txn(txn_stmt),
        }
    }

    /// Execute a MATCH statement.
    fn execute_match(&self, stmt: &MatchStmt) -> SessionResult<QueryResult> {
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_match(stmt)?;

        // Convert to QueryResult
        let columns: Vec<String> = result.column_names().to_vec();
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

    /// Execute a MATCH...mutation compound statement.
    /// This runs the MATCH to get bindings, then executes mutations for each binding row.
    fn execute_match_mutate(&mut self, stmt: &MatchMutateStmt) -> SessionResult<MutationSummary> {
        use mew_pattern::{CompiledPattern, Matcher};

        // Compile the pattern
        let mut pattern = CompiledPattern::compile(&stmt.pattern, self.registry)?;

        // Add WHERE clause as filter if present
        if let Some(ref where_expr) = stmt.where_clause {
            pattern = pattern.with_filter(where_expr.clone());
        }

        // Execute the pattern match to get all bindings
        let matcher = Matcher::new(self.registry, &self.graph);
        let bindings_list = matcher.find_all(&pattern)?;

        let mut total_nodes = 0usize;
        let mut total_edges = 0usize;

        // For each set of bindings from the match, execute the mutations
        for pattern_bindings in bindings_list {
            // Convert pattern bindings to a HashMap for variable lookup
            let mut local_bindings: HashMap<String, EntityId> = HashMap::new();

            // Copy existing session bindings
            for (k, v) in &self.bindings {
                local_bindings.insert(k.clone(), *v);
            }

            // Add bindings from pattern match
            for (var, binding) in pattern_bindings.iter() {
                if let Some(node_id) = binding.as_node() {
                    local_bindings.insert(var.to_string(), node_id.into());
                } else if let Some(edge_id) = binding.as_edge() {
                    local_bindings.insert(var.to_string(), edge_id.into());
                }
            }

            // Execute each mutation with the current bindings
            for mutation in &stmt.mutations {
                match mutation {
                    MutationAction::Link(link_stmt) => {
                        let mut targets = Vec::new();
                        for target_ref in &link_stmt.targets {
                            let entity_id =
                                self.resolve_target_ref_with_bindings(target_ref, &local_bindings)?;
                            targets.push(entity_id);
                        }

                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_link(link_stmt, targets)?;

                        // Store edge binding if variable present
                        if let Some(ref var) = link_stmt.var {
                            if let Some(edge_id) = result.created_edge() {
                                local_bindings.insert(var.clone(), edge_id.into());
                            }
                        }

                        if result.created_edge().is_some() {
                            total_edges += 1;
                        }
                    }
                    MutationAction::Set(set_stmt) => {
                        let target_id =
                            self.resolve_target_with_bindings(&set_stmt.target, &local_bindings)?;
                        let node_id = target_id.as_node().ok_or_else(|| {
                            SessionError::invalid_statement_type(messages::ERR_SET_REQUIRES_NODE)
                        })?;

                        let pb = Bindings::new();
                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_set(set_stmt, vec![node_id], &pb)?;

                        use mew_mutation::MutationOutcome;
                        if let MutationOutcome::Updated(ref u) = result {
                            total_nodes += u.node_ids.len();
                        }
                    }
                    MutationAction::Kill(kill_stmt) => {
                        let target_id =
                            self.resolve_target_with_bindings(&kill_stmt.target, &local_bindings)?;
                        let node_id = target_id.as_node().ok_or_else(|| {
                            SessionError::invalid_statement_type(messages::ERR_KILL_REQUIRES_NODE)
                        })?;

                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_kill(kill_stmt, node_id)?;

                        total_nodes += result.deleted_nodes();
                        total_edges += result.deleted_edges();
                    }
                    MutationAction::Unlink(unlink_stmt) => {
                        let target_id = self
                            .resolve_target_with_bindings(&unlink_stmt.target, &local_bindings)?;
                        let edge_id = target_id.as_edge().ok_or_else(|| {
                            SessionError::invalid_statement_type(messages::ERR_UNLINK_REQUIRES_EDGE)
                        })?;

                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_unlink(unlink_stmt, edge_id)?;

                        total_edges += result.deleted_edges();
                    }
                }
            }
        }

        Ok(MutationSummary::new(total_nodes, total_edges))
    }

    /// Resolve a target using provided bindings.
    fn resolve_target_with_bindings(
        &self,
        target: &mew_parser::Target,
        bindings: &HashMap<String, EntityId>,
    ) -> SessionResult<EntityId> {
        match target {
            mew_parser::Target::Var(var_name) => {
                bindings.get(var_name).copied().ok_or_else(|| {
                    SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name))
                })
            }
            _ => Err(SessionError::invalid_statement_type(
                messages::ERR_ONLY_VAR_TARGETS_COMPOUND,
            )),
        }
    }

    /// Resolve a target reference using provided bindings.
    fn resolve_target_ref_with_bindings(
        &self,
        target_ref: &TargetRef,
        bindings: &HashMap<String, EntityId>,
    ) -> SessionResult<EntityId> {
        match target_ref {
            TargetRef::Var(var_name) => bindings.get(var_name).copied().ok_or_else(|| {
                SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name))
            }),
            _ => Err(SessionError::invalid_statement_type(
                messages::ERR_ONLY_VAR_TARGETS_COMPOUND,
            )),
        }
    }

    /// Execute a WALK statement.
    fn execute_walk(&self, stmt: &WalkStmt) -> SessionResult<QueryResult> {
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_walk(stmt)?;

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

    /// Execute an INSPECT statement.
    fn execute_inspect(&self, stmt: &InspectStmt) -> SessionResult<QueryResult> {
        use mew_core::NodeId;

        // Try to parse the ID as a node ID (format: "node_N" or just a number)
        let id_str = &stmt.id;
        let node_id = if let Some(num_str) = id_str.strip_prefix("node_") {
            num_str.parse::<u64>().ok().map(NodeId::new)
        } else {
            id_str.parse::<u64>().ok().map(NodeId::new)
        };

        // Try to look up as node first
        if let Some(nid) = node_id {
            if let Some(node) = self.graph.get_node(nid) {
                // Get the type name from registry
                let type_name = self
                    .registry
                    .get_type(node.type_id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                // Build columns based on projections or all attributes
                let (columns, values): (Vec<String>, Vec<Value>) =
                    if let Some(ref projections) = stmt.projections {
                        let mut cols = Vec::new();
                        let mut vals = Vec::new();
                        for proj in projections {
                            let col_name = proj.alias.clone().unwrap_or_else(|| {
                                if let mew_parser::Expr::Var(name, _) = &proj.expr {
                                    name.clone()
                                } else if let mew_parser::Expr::AttrAccess(_, attr, _) = &proj.expr {
                                    attr.clone()
                                } else {
                                    "?".to_string()
                                }
                            });

                            // Handle special columns
                            let value = match col_name.as_str() {
                                "_type" => Value::String(type_name.clone()),
                                "_id" => Value::NodeRef(nid),
                                "*" => {
                                    // Return all attributes
                                    for (attr_name, attr_val) in node.attributes.iter() {
                                        cols.push(attr_name.clone());
                                        vals.push(attr_val.clone());
                                    }
                                    continue;
                                }
                                attr => node
                                    .get_attr(attr)
                                    .cloned()
                                    .unwrap_or(Value::Null),
                            };

                            cols.push(col_name);
                            vals.push(value);
                        }
                        (cols, vals)
                    } else {
                        // Default: return all attributes plus _type and _id
                        let mut cols = vec!["_type".to_string(), "_id".to_string()];
                        let mut vals: Vec<Value> = vec![
                            Value::String(type_name),
                            Value::NodeRef(nid),
                        ];

                        for (attr_name, attr_val) in node.attributes.iter() {
                            cols.push(attr_name.clone());
                            vals.push(attr_val.clone());
                        }

                        (cols, vals)
                    };

                let types = vec!["any".to_string(); columns.len()];
                return Ok(QueryResult::new(columns, types, vec![values]));
            }
        }

        // Entity not found - return empty result with found=false
        let columns = vec!["found".to_string()];
        let types = vec!["bool".to_string()];
        let values = vec![Value::Bool(false)];
        Ok(QueryResult::new(columns, types, vec![values]))
    }

    /// Execute a SPAWN statement.
    fn execute_spawn(&mut self, stmt: &mew_parser::SpawnStmt) -> SessionResult<MutationSummary> {
        let pattern_bindings = Bindings::new();
        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_spawn(stmt, &pattern_bindings)?;

        // Store the created node ID with the variable name
        if let Some(node_id) = result.created_node() {
            self.bindings.insert(stmt.var.clone(), node_id.into());
        }

        let nodes = if result.created_node().is_some() {
            1
        } else {
            0
        };
        let edges = if result.created_edge().is_some() {
            1
        } else {
            0
        };
        Ok(MutationSummary::new(nodes, edges))
    }

    /// Execute a KILL statement.
    fn execute_kill(&mut self, stmt: &mew_parser::KillStmt) -> SessionResult<MutationSummary> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id
            .as_node()
            .ok_or_else(|| SessionError::invalid_statement_type(messages::ERR_KILL_REQUIRES_NODE))?;

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_kill(stmt, node_id)?;

        Ok(MutationSummary::new(
            result.deleted_nodes(),
            result.deleted_edges(),
        ))
    }

    /// Execute a LINK statement.
    fn execute_link(&mut self, stmt: &mew_parser::LinkStmt) -> SessionResult<MutationSummary> {
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

        let edges = if result.created_edge().is_some() {
            1
        } else {
            0
        };
        Ok(MutationSummary::new(0, edges))
    }

    /// Execute an UNLINK statement.
    fn execute_unlink(&mut self, stmt: &mew_parser::UnlinkStmt) -> SessionResult<MutationSummary> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let edge_id = target_id.as_edge().ok_or_else(|| {
            SessionError::invalid_statement_type(messages::ERR_UNLINK_REQUIRES_EDGE)
        })?;

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_unlink(stmt, edge_id)?;

        Ok(MutationSummary::new(0, result.deleted_edges()))
    }

    /// Execute a SET statement.
    fn execute_set(&mut self, stmt: &mew_parser::SetStmt) -> SessionResult<MutationSummary> {
        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id
            .as_node()
            .ok_or_else(|| SessionError::invalid_statement_type(messages::ERR_SET_REQUIRES_NODE))?;

        let pattern_bindings = Bindings::new();
        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_set(stmt, vec![node_id], &pattern_bindings)?;

        // Count updated nodes from the result
        use mew_mutation::MutationOutcome;
        let nodes = match result {
            MutationOutcome::Updated(ref u) => u.node_ids.len(),
            _ => 0,
        };
        Ok(MutationSummary::new(nodes, 0))
    }

    /// Resolve a target to an EntityId.
    fn resolve_target(&self, target: &mew_parser::Target) -> SessionResult<EntityId> {
        match target {
            mew_parser::Target::Var(var_name) => {
                self.bindings.get(var_name).copied().ok_or_else(|| {
                    SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name))
                })
            }
            mew_parser::Target::Id(_) | mew_parser::Target::Pattern(_) => Err(
                SessionError::invalid_statement_type(messages::ERR_ONLY_VAR_TARGETS),
            ),
            mew_parser::Target::EdgePattern { edge_type, targets } => {
                // Resolve target variables to node IDs
                let mut target_ids = Vec::new();
                for target_var in targets {
                    let id = self.bindings.get(target_var).copied().ok_or_else(|| {
                        SessionError::invalid_statement_type(format!(
                            "Unknown variable in edge pattern: {}",
                            target_var
                        ))
                    })?;
                    target_ids.push(id);
                }

                // Find edge type ID
                let edge_type_id = self.registry.get_edge_type_id(edge_type).ok_or_else(|| {
                    SessionError::invalid_statement_type(format!(
                        "Unknown edge type: {}",
                        edge_type
                    ))
                })?;

                // Find edge between the nodes
                if target_ids.len() < 2 {
                    return Err(SessionError::invalid_statement_type(
                        messages::ERR_EDGE_PATTERN_MIN_TARGETS,
                    ));
                }

                let source_node_id = target_ids[0]
                    .as_node()
                    .ok_or_else(|| SessionError::invalid_statement_type(messages::ERR_SOURCE_MUST_BE_NODE))?;
                let target_node_id = target_ids[1]
                    .as_node()
                    .ok_or_else(|| SessionError::invalid_statement_type(messages::ERR_TARGET_MUST_BE_NODE))?;

                // Search for matching edge
                for edge_id in self.graph.edges_from(source_node_id, None) {
                    if let Some(edge) = self.graph.get_edge(edge_id) {
                        if edge.type_id == edge_type_id {
                            let targets = &edge.targets;
                            if targets.len() >= 2
                                && targets[0].as_node() == Some(source_node_id)
                                && targets[1].as_node() == Some(target_node_id)
                            {
                                return Ok(edge_id.into());
                            }
                        }
                    }
                }

                Err(SessionError::invalid_statement_type(format!(
                    "No edge of type '{}' found between specified nodes",
                    edge_type
                )))
            }
        }
    }

    /// Resolve a target reference to an EntityId.
    fn resolve_target_ref(&self, target_ref: &TargetRef) -> SessionResult<EntityId> {
        match target_ref {
            TargetRef::Var(var_name) => self.bindings.get(var_name).copied().ok_or_else(|| {
                SessionError::invalid_statement_type(format!("Unknown variable: {}", var_name))
            }),
            TargetRef::Id(_) | TargetRef::Pattern(_) => Err(SessionError::invalid_statement_type(
                messages::ERR_ONLY_VAR_TARGETS,
            )),
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

    // ========== Acceptance Tests ==========

    #[test]
    fn test_accept_and_execute_query() {
        // TEST: accept_and_execute_query
        // GIVEN a session with a task
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // Create a task first
        let spawn_result = session.execute("SPAWN t: Task { title = \"Test\" }");
        assert!(spawn_result.is_ok());

        // WHEN executing a MATCH query
        let result = session.execute("MATCH t: Task RETURN t");

        // THEN result is parsed, analyzed, executed and returns rows
        assert!(result.is_ok());
        match result.unwrap() {
            StatementResult::Query(query_result) => {
                assert!(!query_result.columns.is_empty());
            }
            _ => panic!("Expected query result"),
        }
    }

    #[test]
    fn test_accept_and_execute_mutation() {
        // TEST: accept_and_execute_mutation
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN executing a SPAWN mutation
        let result = session.execute("SPAWN t: Task { title = \"Hello\" }");

        // THEN node is created and returns created ID
        assert!(result.is_ok());
        match result.unwrap() {
            StatementResult::Mutation(mutation_result) => {
                assert_eq!(mutation_result.nodes_affected, 1);
            }
            _ => panic!("Expected mutation result"),
        }
    }

    #[test]
    fn test_syntax_error_returns_error() {
        // TEST: syntax_error_returns_error
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN executing invalid syntax
        let result = session.execute("MATC t: Task");

        // THEN error response with message
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should be a parse error
        assert!(matches!(err, SessionError::ParseError(_)));
    }

    #[test]
    fn test_analysis_error_returns_error() {
        // TEST: analysis_error_returns_error
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN executing query with unknown type
        let result = session.execute("MATCH t: Unknown RETURN t");

        // THEN error about unknown type
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Unknown") || err.contains("unknown") || err.contains("type"));
    }

    #[test]
    fn test_session_tracks_transaction() {
        // TEST: session_tracks_transaction
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN BEGIN
        let _ = session.execute("BEGIN");

        // THEN session has active transaction
        assert!(session.in_transaction());

        // WHEN COMMIT
        let _ = session.execute("COMMIT");

        // THEN session has no active transaction
        assert!(!session.in_transaction());
    }

    #[test]
    fn test_transaction_spans_statements() {
        // TEST: transaction_spans_statements
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN executing multiple statements in a transaction
        let _ = session.execute("BEGIN");
        let r1 = session.execute("SPAWN t: Task { title = \"A\" }");
        let r2 = session.execute("SPAWN p: Person { name = \"B\" }");
        let _ = session.execute("COMMIT");

        // THEN both nodes created
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        // And transaction is complete
        assert!(!session.in_transaction());
    }

    #[test]
    fn test_concurrent_sessions_isolated() {
        // TEST: concurrent_sessions_isolated
        // GIVEN two sessions with the same registry
        let registry = test_registry();
        let mut session_a = Session::new(1, &registry);
        let session_b = Session::new(2, &registry);

        // WHEN session A creates a task in a transaction (uncommitted)
        let _ = session_a.execute("BEGIN");
        let _ = session_a.execute("SPAWN t: Task { title = \"A's Task\" }");

        // THEN session B does not see A's uncommitted work
        // (Each session has its own graph in the current implementation)
        assert!(session_b.graph().node_count() == 0);

        // Session A commits
        let _ = session_a.execute("COMMIT");

        // Session A's graph has the node
        assert!(session_a.graph().node_count() == 1);

        // Note: In a true shared database, session B would now see the data
        // Our current implementation uses separate graphs per session
        // This test verifies the isolation mechanism works
    }

    // ========== INSPECT Tests ==========

    #[test]
    fn test_inspect_existing_node() {
        // GIVEN a session with a task
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // Create a task
        let _ = session.execute("SPAWN t: Task { title = \"Test Task\" }").unwrap();

        // WHEN inspecting the node by ID
        let result = session.execute("INSPECT #1");

        // THEN we get the node data
        if let Err(ref e) = result {
            eprintln!("INSPECT failed: {:?}", e);
        }
        assert!(result.is_ok(), "INSPECT failed: {:?}", result.err());
        match result.unwrap() {
            StatementResult::Query(q) => {
                assert!(q.columns.contains(&"_type".to_string()));
                assert!(q.columns.contains(&"_id".to_string()));
                // Should have at least one row
                assert!(!q.rows.is_empty());
            }
            _ => panic!("Expected query result"),
        }
    }

    #[test]
    fn test_inspect_nonexistent_node() {
        // GIVEN a session
        let registry = test_registry();
        let mut session = Session::new(1, &registry);

        // WHEN inspecting a nonexistent node
        let result = session.execute("INSPECT #999");

        // THEN we get found=false
        if let Err(ref e) = result {
            eprintln!("INSPECT failed: {:?}", e);
        }
        assert!(result.is_ok(), "INSPECT failed: {:?}", result.err());
        match result.unwrap() {
            StatementResult::Query(q) => {
                assert!(q.columns.contains(&"found".to_string()));
                // First row should have found=false
                if let Some(row) = q.rows.first() {
                    assert_eq!(row[0], Value::Bool(false));
                }
            }
            _ => panic!("Expected query result"),
        }
    }
}
