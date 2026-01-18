//! Session manager.

use mew_analyzer::Analyzer;
use mew_constraint::ConstraintChecker;
use mew_core::{messages, EntityId, Value};
use mew_graph::Graph;
use mew_mutation::{MutationExecutor, MutationOutcome};
use mew_parser::{
    parse_stmt, parse_stmts, InspectStmt, MatchMutateStmt, MatchStmt, MutationAction, Stmt,
    TargetRef, WalkStmt,
};
use mew_pattern::{target, Binding, Bindings};
use mew_query::QueryExecutor;
use mew_registry::Registry;
use std::collections::HashMap;

use crate::error::{SessionError, SessionResult};
use crate::query::convert_query_result;
use crate::result::{MutationSummary, QueryResult, StatementResult, TransactionResult};
use crate::transaction::{self, TransactionState};

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
    /// Transaction state.
    txn_state: TransactionState,
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
            txn_state: TransactionState::new(),
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
            txn_state: TransactionState::new(),
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
        self.txn_state.in_transaction
    }

    /// Reset transaction state (used after errors to clean up).
    ///
    /// This rolls back any pending changes and resets the transaction flag.
    /// Note: This doesn't undo changes already applied to the graph - those
    /// would need proper rollback support at the graph level.
    pub fn reset_transaction(&mut self) {
        if self.txn_state.in_transaction {
            // Roll back created entities
            for edge_id in self.txn_state.created_edges.drain(..) {
                let _ = self.graph.delete_edge(edge_id);
            }
            for node_id in self.txn_state.created_nodes.drain(..) {
                let _ = self.graph.delete_node(node_id);
            }
            self.txn_state.in_transaction = false;
        }
    }

    /// Execute a statement string.
    pub fn execute(&mut self, input: &str) -> SessionResult<StatementResult> {
        // Parse the input
        let stmt = parse_stmt(input)?;

        self.execute_statement(&stmt)
    }

    /// Execute multiple statements from a string.
    ///
    /// Aggregation behavior:
    /// - Mutations: sums all created/deleted/modified counts
    /// - Queries: combines rows when columns match; different columns replace accumulated result
    pub fn execute_all(&mut self, input: &str) -> SessionResult<StatementResult> {
        let stmts = parse_stmts(input)?;

        if stmts.is_empty() {
            return Ok(StatementResult::Mutation(MutationSummary::default()));
        }

        // If there's only one statement, just execute it normally
        if stmts.len() == 1 {
            return self.execute_statement(&stmts[0]);
        }

        // Execute all statements and aggregate results
        let mut total_mutations = MutationSummary::default();
        let mut combined_query: Option<QueryResult> = None;

        for stmt in &stmts {
            let result = self.execute_statement(stmt)?;
            match result {
                StatementResult::Mutation(m) => {
                    total_mutations.merge(&m);
                }
                StatementResult::Transaction(TransactionResult::RolledBack) => {
                    // On ROLLBACK, clear accumulated mutation counts since changes were undone
                    total_mutations = MutationSummary::default();
                }
                StatementResult::Query(q) => {
                    // Combine query results: append rows if columns match, or replace if different
                    combined_query = Some(match combined_query {
                        None => q,
                        Some(mut existing) => {
                            if existing.columns == q.columns {
                                // Same columns - append rows
                                existing.rows.extend(q.rows);
                                existing
                            } else {
                                // Different columns - use the new result
                                q
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        // Return results based on what was executed
        match (total_mutations.total_affected() > 0, combined_query) {
            (true, Some(query_result)) => {
                // Both mutations and queries occurred - return Mixed
                Ok(StatementResult::Mixed {
                    mutations: total_mutations,
                    queries: query_result,
                })
            }
            (true, None) => {
                // Only mutations occurred
                Ok(StatementResult::Mutation(total_mutations))
            }
            (false, Some(query_result)) => {
                // Only queries occurred
                Ok(StatementResult::Query(query_result))
            }
            (false, None) => {
                // No mutations or queries (empty statements)
                Ok(StatementResult::Mutation(MutationSummary::default()))
            }
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

            Stmt::MatchWalk(match_walk_stmt) => {
                let result = self.execute_match_walk(match_walk_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Inspect(inspect_stmt) => {
                let result = self.execute_inspect(inspect_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Txn(txn_stmt) => self.execute_txn(txn_stmt),

            Stmt::Explain(explain_stmt) => {
                let result = self.execute_explain(explain_stmt)?;
                Ok(StatementResult::Query(result))
            }

            Stmt::Profile(profile_stmt) => {
                let result = self.execute_profile(profile_stmt)?;
                Ok(StatementResult::Query(result))
            }
        }
    }

    /// Execute a MATCH statement.
    fn execute_match(&self, stmt: &MatchStmt) -> SessionResult<QueryResult> {
        // Run analyzer for type checking before execution
        let mut analyzer = Analyzer::new(self.registry);
        analyzer.analyze_stmt(&Stmt::Match(stmt.clone()))?;

        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_match(stmt)?;
        Ok(convert_query_result(&result))
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

        let mut nodes_created = 0usize;
        let mut nodes_modified = 0usize;
        let mut edges_modified = 0usize;
        let mut edges_created = 0usize;
        let mut nodes_deleted = 0usize;
        let mut edges_deleted = 0usize;

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
                    MutationAction::Spawn(spawn_stmt) => {
                        let summary = self.execute_spawn(spawn_stmt)?;
                        nodes_created += summary.nodes_created;

                        // Add spawned nodes to local bindings (handles both single and chained)
                        for item in &spawn_stmt.items {
                            if let Some(entity_id) = self.bindings.get(&item.var) {
                                local_bindings.insert(item.var.clone(), *entity_id);
                            }
                        }
                    }
                    MutationAction::Link(link_stmt) => {
                        let mut targets = Vec::new();
                        for target_ref in &link_stmt.targets {
                            let entity_id = self.resolve_or_spawn_target_ref_with_bindings(
                                target_ref,
                                &mut local_bindings,
                                &mut nodes_created,
                            )?;
                            targets.push(entity_id);
                        }

                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_link(link_stmt, targets)?;

                        if let Some(edge_id) = result.created_edge() {
                            self.check_edge_constraints(edge_id)?;
                        }

                        // Store edge binding if variable present
                        if let Some(ref var) = link_stmt.var {
                            if let Some(edge_id) = result.created_edge() {
                                local_bindings.insert(var.clone(), edge_id.into());
                            }
                        }

                        if result.created_edge().is_some() {
                            edges_created += 1;
                        }
                    }
                    MutationAction::Set(set_stmt) => {
                        let target_id =
                            self.resolve_target_with_bindings(&set_stmt.target, &local_bindings)?;

                        // Convert local_bindings to pattern Bindings for expression evaluation
                        let pb = to_pattern_bindings(&local_bindings);
                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);

                        use mew_mutation::MutationOutcome;
                        if let Some(node_id) = target_id.as_node() {
                            let result = executor.execute_set(set_stmt, vec![node_id], &pb)?;
                            if let MutationOutcome::Updated(ref u) = result {
                                nodes_modified += u.node_ids.len();
                            }
                        } else if let Some(edge_id) = target_id.as_edge() {
                            let result = executor.execute_set_edge(set_stmt, vec![edge_id], &pb)?;
                            if let MutationOutcome::Updated(ref u) = result {
                                edges_modified += u.edge_ids.len();
                            }
                        } else {
                            return Err(SessionError::invalid_statement_type(
                                messages::ERR_SET_REQUIRES_NODE,
                            ));
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

                        nodes_deleted += result.deleted_nodes();
                        edges_deleted += result.deleted_edges();
                    }
                    MutationAction::Unlink(unlink_stmt) => {
                        let target_id = self
                            .resolve_target_with_bindings(&unlink_stmt.target, &local_bindings)?;
                        let edge_id = target_id.as_edge().ok_or_else(|| {
                            SessionError::invalid_statement_type(messages::ERR_UNLINK_REQUIRES_EDGE)
                        })?;

                        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
                        let result = executor.execute_unlink(unlink_stmt, edge_id)?;

                        edges_deleted += result.deleted_edges();
                    }
                }
            }
        }

        Ok(MutationSummary {
            nodes_created,
            nodes_modified,
            edges_modified,
            edges_created,
            nodes_deleted,
            edges_deleted,
            ..Default::default()
        })
    }

    /// Resolve a target using provided bindings.
    fn resolve_target_with_bindings(
        &self,
        t: &mew_parser::Target,
        bindings: &HashMap<String, EntityId>,
    ) -> SessionResult<EntityId> {
        Ok(target::resolve_target(
            t,
            bindings,
            self.registry,
            &self.graph,
        )?)
    }

    /// Resolve a target reference using provided bindings.
    fn resolve_target_ref_with_bindings(
        &self,
        target_ref: &TargetRef,
        bindings: &HashMap<String, EntityId>,
    ) -> SessionResult<EntityId> {
        Ok(target::resolve_target_ref(target_ref, bindings)?)
    }

    /// Resolve a target reference, handling inline spawns with provided bindings.
    fn resolve_or_spawn_target_ref_with_bindings(
        &mut self,
        target_ref: &TargetRef,
        bindings: &mut HashMap<String, EntityId>,
        nodes_created: &mut usize,
    ) -> SessionResult<EntityId> {
        match target_ref {
            TargetRef::InlineSpawn(spawn_stmt) => {
                // Execute the spawn and return the created node ID
                let summary = self.execute_spawn(spawn_stmt)?;
                *nodes_created += summary.nodes_created;

                // Get the variable name for inline spawn (first item)
                let var_name = spawn_stmt.var();

                // Get the node ID from session bindings (spawn stores it with var name)
                let entity_id = self.bindings.get(var_name).cloned().ok_or_else(|| {
                    SessionError::invalid_statement_type(format!(
                        "inline spawn did not create binding for '{}'",
                        var_name
                    ))
                })?;

                // Also add to local bindings for MATCH context
                bindings.insert(var_name.to_string(), entity_id);
                Ok(entity_id)
            }
            _ => self.resolve_target_ref_with_bindings(target_ref, bindings),
        }
    }

    /// Execute a WALK statement.
    fn execute_walk(&self, stmt: &WalkStmt) -> SessionResult<QueryResult> {
        // Convert session bindings to pattern bindings so ID refs can be resolved
        let pattern_bindings = to_pattern_bindings(&self.bindings);
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_walk_with_bindings(stmt, Some(&pattern_bindings))?;
        Ok(convert_query_result(&result))
    }

    /// Execute a MATCH...WALK compound statement.
    fn execute_match_walk(&self, stmt: &mew_parser::MatchWalkStmt) -> SessionResult<QueryResult> {
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let result = executor.execute_match_walk(stmt)?;
        Ok(convert_query_result(&result))
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
                let (columns, values): (Vec<String>, Vec<Value>) = if let Some(ref projections) =
                    stmt.projections
                {
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
                            attr => node.get_attr(attr).cloned().unwrap_or(Value::Null),
                        };

                        cols.push(col_name);
                        vals.push(value);
                    }
                    (cols, vals)
                } else {
                    // Default: return all attributes plus _type and _id
                    let mut cols = vec!["_type".to_string(), "_id".to_string()];
                    let mut vals: Vec<Value> = vec![Value::String(type_name), Value::NodeRef(nid)];

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

        // Handle created nodes - for chained spawns, bind each variable
        let mut nodes_created = 0;
        if let MutationOutcome::Created(ref created) = result {
            // Bind each spawn item's variable to its created node
            for (i, item) in stmt.items.iter().enumerate() {
                if let Some(&node_id) = created.node_ids.get(i) {
                    self.bindings.insert(item.var.clone(), node_id.into());
                    // Track for transaction rollback
                    self.txn_state.track_created_node(node_id);
                    nodes_created += 1;
                }
            }
        }

        let edges_created = if let Some(edge_id) = result.created_edge() {
            // Track for transaction rollback
            self.txn_state.track_created_edge(edge_id);
            1
        } else {
            0
        };

        Ok(MutationSummary {
            nodes_created,
            edges_created,
            ..Default::default()
        })
    }

    /// Execute a KILL statement.
    fn execute_kill(&mut self, stmt: &mew_parser::KillStmt) -> SessionResult<MutationSummary> {
        // Handle pattern-based KILL specially
        if let mew_parser::Target::Pattern(match_stmt) = &stmt.target {
            return self.execute_kill_pattern(stmt, match_stmt);
        }

        // Resolve the target
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id.as_node().ok_or_else(|| {
            SessionError::invalid_statement_type(messages::ERR_KILL_REQUIRES_NODE)
        })?;

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_kill(stmt, node_id)?;

        Ok(MutationSummary {
            nodes_deleted: result.deleted_nodes(),
            edges_deleted: result.deleted_edges(),
            ..Default::default()
        })
    }

    /// Execute a KILL statement with a pattern target.
    /// Runs the MATCH query first, then deletes all matching nodes.
    fn execute_kill_pattern(
        &mut self,
        stmt: &mew_parser::KillStmt,
        match_stmt: &MatchStmt,
    ) -> SessionResult<MutationSummary> {
        // Execute the MATCH query to get matching entities
        let executor = QueryExecutor::new(self.registry, &self.graph);
        let query_result = executor.execute_match(match_stmt)?;

        // Collect all node IDs to delete
        let mut node_ids = Vec::new();
        for row in query_result.rows() {
            // Get the first column value which should be the node reference
            if let Some(value) = row.get(0) {
                if let Some(node_id) = value.as_node_ref() {
                    node_ids.push(node_id);
                }
            }
        }

        // Delete each node
        let mut total_nodes_deleted = 0usize;
        let mut total_edges_deleted = 0usize;

        for node_id in node_ids {
            let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
            let result = executor.execute_kill(stmt, node_id)?;
            total_nodes_deleted += result.deleted_nodes();
            total_edges_deleted += result.deleted_edges();
        }

        Ok(MutationSummary {
            nodes_deleted: total_nodes_deleted,
            edges_deleted: total_edges_deleted,
            ..Default::default()
        })
    }

    /// Execute a LINK statement.
    fn execute_link(&mut self, stmt: &mew_parser::LinkStmt) -> SessionResult<MutationSummary> {
        // Resolve all targets, handling inline spawns
        let mut target_ids = Vec::new();
        let mut nodes_created = 0;

        for target_ref in &stmt.targets {
            let entity_id = self.resolve_or_spawn_target_ref(target_ref, &mut nodes_created)?;
            target_ids.push(entity_id);
        }

        let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
        let result = executor.execute_link(stmt, target_ids)?;

        // Store the created edge ID with the variable name if present
        let edges_created = if let Some(edge_id) = result.created_edge() {
            self.check_edge_constraints(edge_id)?;

            if let Some(ref var) = stmt.var {
                self.bindings.insert(var.clone(), edge_id.into());
            }
            // Track for transaction rollback
            self.txn_state.track_created_edge(edge_id);
            1
        } else {
            0
        };

        Ok(MutationSummary {
            nodes_created,
            edges_created,
            ..Default::default()
        })
    }

    /// Resolve a target reference to an entity ID, spawning if it's an inline spawn.
    fn resolve_or_spawn_target_ref(
        &mut self,
        target_ref: &TargetRef,
        nodes_created: &mut usize,
    ) -> SessionResult<EntityId> {
        match target_ref {
            TargetRef::InlineSpawn(spawn_stmt) => {
                // Execute the spawn and return the created node ID
                let summary = self.execute_spawn(spawn_stmt)?;
                *nodes_created += summary.nodes_created;

                // Get the variable name for inline spawn (first item)
                let var_name = spawn_stmt.var();

                // Get the node ID from bindings (spawn stores it with var name)
                let entity_id = self.bindings.get(var_name).cloned().ok_or_else(|| {
                    SessionError::invalid_statement_type(format!(
                        "inline spawn did not create binding for '{}'",
                        var_name
                    ))
                })?;
                Ok(entity_id)
            }
            _ => self.resolve_target_ref(target_ref),
        }
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

        Ok(MutationSummary {
            edges_deleted: result.deleted_edges(),
            ..Default::default()
        })
    }

    /// Execute a SET statement.
    fn execute_set(&mut self, stmt: &mew_parser::SetStmt) -> SessionResult<MutationSummary> {
        let target_id = self.resolve_target(&stmt.target)?;
        let pattern_bindings = Bindings::new();

        // Handle both node and edge targets
        if let Some(node_id) = target_id.as_node() {
            let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
            let result = executor.execute_set(stmt, vec![node_id], &pattern_bindings)?;

            let nodes_modified = match result {
                MutationOutcome::Updated(ref u) => u.node_ids.len(),
                _ => 0,
            };

            Ok(MutationSummary {
                nodes_modified,
                ..Default::default()
            })
        } else if let Some(edge_id) = target_id.as_edge() {
            let mut executor = MutationExecutor::new(self.registry, &mut self.graph);
            let result = executor.execute_set_edge(stmt, vec![edge_id], &pattern_bindings)?;

            let edges_modified = match result {
                MutationOutcome::Updated(ref u) => u.edge_ids.len(),
                _ => 0,
            };

            Ok(MutationSummary {
                edges_modified,
                ..Default::default()
            })
        } else {
            Err(SessionError::invalid_statement_type(
                messages::ERR_SET_REQUIRES_NODE,
            ))
        }
    }

    /// Resolve a target to an EntityId.
    fn resolve_target(&self, t: &mew_parser::Target) -> SessionResult<EntityId> {
        Ok(target::resolve_target(
            t,
            &self.bindings,
            self.registry,
            &self.graph,
        )?)
    }

    /// Check edge constraints and rollback if violated.
    ///
    /// Note: We check AFTER creation because the constraint checker needs
    /// the edge to exist in the graph to inspect its endpoints.
    fn check_edge_constraints(&mut self, edge_id: mew_core::EdgeId) -> SessionResult<()> {
        let checker = ConstraintChecker::new(self.registry, &self.graph);
        let violations = checker
            .check_edge_immediate(edge_id)
            .map_err(|e| SessionError::constraint_error(e.to_string()))?;

        if !violations.is_empty() {
            let _ = self.graph.delete_edge(edge_id);
            let first = &violations.all()[0];
            return Err(SessionError::constraint_error(format!(
                "{}: {}",
                first.constraint_name, first.message
            )));
        }
        Ok(())
    }

    /// Resolve a target reference to an EntityId.
    fn resolve_target_ref(&self, target_ref: &TargetRef) -> SessionResult<EntityId> {
        Ok(target::resolve_target_ref(target_ref, &self.bindings)?)
    }

    /// Execute a transaction statement.
    ///
    /// # ROLLBACK Semantics
    ///
    /// ROLLBACK currently only undoes SPAWN operations (node/edge creation).
    /// SET mutations and DELETE operations within the transaction are NOT
    /// automatically reverted. This is a known limitation - full MVCC or
    /// undo-logging would be required for complete rollback support.
    ///
    /// Errors during rollback cleanup are intentionally ignored because:
    /// - The entity may have already been deleted by a KILL within the transaction
    /// - Partial rollback is better than failing and leaving the DB in an
    ///   inconsistent state
    fn execute_txn(&mut self, stmt: &mew_parser::TxnStmt) -> SessionResult<StatementResult> {
        // For ROLLBACK, we need to undo created entities before processing
        if matches!(stmt, mew_parser::TxnStmt::Rollback) {
            // Take ownership to avoid clone - clear_tracked() would clear anyway
            let created_edges = std::mem::take(&mut self.txn_state.created_edges);
            let created_nodes = std::mem::take(&mut self.txn_state.created_nodes);

            // Delete edges first (they may reference nodes)
            for edge_id in created_edges {
                // Errors ignored intentionally - see doc comment above
                let _ = self.graph.delete_edge(edge_id);
            }
            // Then delete nodes
            for node_id in created_nodes {
                // Errors ignored intentionally - see doc comment above
                let _ = self.graph.delete_node(node_id);
            }
        }

        transaction::execute_txn(&mut self.txn_state, stmt)
    }

    /// Execute an EXPLAIN statement - returns the query plan without executing.
    fn execute_explain(&self, stmt: &mew_parser::ExplainStmt) -> SessionResult<QueryResult> {
        use mew_query::QueryPlanner;

        // Get the plan based on the inner statement type
        let plan_str = match stmt.statement.as_ref() {
            Stmt::Match(m) => {
                let planner = QueryPlanner::new(self.registry);
                match planner.plan_match(m) {
                    Ok(plan) => format!("{:#?}", plan),
                    Err(e) => format!("Plan error: {}", e),
                }
            }
            Stmt::Walk(w) => {
                let planner = QueryPlanner::new(self.registry);
                match planner.plan_walk(w) {
                    Ok(plan) => format!("{:#?}", plan),
                    Err(e) => format!("Plan error: {}", e),
                }
            }
            other => format!(
                "EXPLAIN not supported for {:?}",
                std::mem::discriminant(other)
            ),
        };

        Ok(QueryResult {
            columns: vec!["plan".to_string()],
            types: vec!["String".to_string()],
            rows: vec![vec![Value::String(plan_str)]],
        })
    }

    /// Execute a PROFILE statement - executes and returns actual results.
    /// PROFILE returns the query results (same as running the inner query directly).
    /// Future enhancement: add execution timing/metrics to output.
    fn execute_profile(&mut self, stmt: &mew_parser::ProfileStmt) -> SessionResult<QueryResult> {
        // Execute the inner statement and return its result
        let result = self.execute_statement(&stmt.statement)?;

        match result {
            StatementResult::Query(qr) => Ok(qr),
            StatementResult::Mixed { queries, .. } => {
                // For mixed results, return the query part
                Ok(queries)
            }
            StatementResult::Mutation(_) => {
                // For mutations, return empty query result
                Ok(QueryResult {
                    columns: vec![],
                    types: vec![],
                    rows: vec![],
                })
            }
            StatementResult::Transaction(_) | StatementResult::Empty => Ok(QueryResult {
                columns: vec![],
                types: vec![],
                rows: vec![],
            }),
        }
    }
}

/// Convert a HashMap of entity bindings to pattern Bindings for expression evaluation.
fn to_pattern_bindings(bindings: &HashMap<String, EntityId>) -> Bindings {
    let mut pattern_bindings = Bindings::new();
    for (name, entity) in bindings {
        match entity {
            EntityId::Node(node_id) => {
                pattern_bindings.insert(name.clone(), Binding::Node(*node_id));
            }
            EntityId::Edge(edge_id) => {
                pattern_bindings.insert(name.clone(), Binding::Edge(*edge_id));
            }
        }
    }
    pattern_bindings
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
                assert_eq!(mutation_result.nodes_created, 1);
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
        let _ = session
            .execute("SPAWN t: Task { title = \"Test Task\" }")
            .unwrap();

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
