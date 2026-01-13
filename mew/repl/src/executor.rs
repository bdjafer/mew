//! Statement execution for the REPL.

use std::collections::HashMap;

use mew_core::{messages, EntityId};
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{InspectStmt, MatchMutateStmt, MatchStmt, MutationAction, Target, TargetRef, TxnStmt, WalkStmt};
use mew_pattern::{target, Binding, Bindings};
use mew_query::QueryExecutor;
use mew_registry::Registry;

use crate::format::format_value;

/// Execute a MATCH statement and return formatted results.
pub fn execute_match(
    registry: &Registry,
    graph: &Graph,
    bindings: &HashMap<String, EntityId>,
    stmt: &MatchStmt,
) -> Result<String, String> {
    let executor = QueryExecutor::new(registry, graph);
    let initial_bindings = to_pattern_bindings(bindings);
    let results = executor
        .execute_match_with_bindings(stmt, &initial_bindings)
        .map_err(|e| format!("Query error: {}", e))?;

    if results.is_empty() {
        return Ok("(no results)".to_string());
    }

    let mut output = String::new();

    // Header
    let columns = results.column_names();
    output.push_str(&columns.join(" | "));
    output.push('\n');
    output.push_str(&"-".repeat(columns.len() * 15));
    output.push('\n');

    // Rows
    for row in results.rows() {
        let values: Vec<String> = columns
            .iter()
            .map(|c| {
                row.get_by_name(c)
                    .map(format_value)
                    .unwrap_or_else(|| "NULL".to_string())
            })
            .collect();
        output.push_str(&values.join(" | "));
        output.push('\n');
    }

    output.push_str(&format!("\n({} rows)", results.len()));

    Ok(output)
}

/// Execute a WALK statement and return formatted results.
pub fn execute_walk(
    registry: &Registry,
    graph: &Graph,
    bindings: &HashMap<String, EntityId>,
    stmt: &WalkStmt,
) -> Result<String, String> {
    let executor = QueryExecutor::new(registry, graph);
    let _initial_bindings = to_pattern_bindings(bindings);
    let results = executor
        .execute_walk(stmt)
        .map_err(|e| format!("Walk error: {}", e))?;

    if results.is_empty() {
        return Ok("(no paths found)".to_string());
    }

    let mut output = String::new();

    // Header
    let columns = results.column_names();
    output.push_str(&columns.join(" | "));
    output.push('\n');
    output.push_str(&"-".repeat(columns.len() * 15));
    output.push('\n');

    // Rows
    for row in results.rows() {
        let values: Vec<String> = columns
            .iter()
            .map(|c| {
                row.get_by_name(c)
                    .map(format_value)
                    .unwrap_or_else(|| "NULL".to_string())
            })
            .collect();
        output.push_str(&values.join(" | "));
        output.push('\n');
    }

    output.push_str(&format!("\n({} paths)", results.len()));

    Ok(output)
}

/// Execute an INSPECT statement and return formatted results.
pub fn execute_inspect(
    registry: &Registry,
    graph: &Graph,
    stmt: &InspectStmt,
) -> Result<String, String> {
    use mew_core::{NodeId, Value};

    // Try to parse the ID as a node ID (format: "node_N" or just a number)
    let id_str = &stmt.id;
    let node_id = if let Some(num_str) = id_str.strip_prefix("node_") {
        num_str.parse::<u64>().ok().map(NodeId::new)
    } else {
        id_str.parse::<u64>().ok().map(NodeId::new)
    };

    // Try to look up as node first
    if let Some(nid) = node_id {
        if let Some(node) = graph.get_node(nid) {
            // Get the type name from registry
            let type_name = registry
                .get_type(node.type_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let mut output = String::new();

            // Build columns based on projections or all attributes
            if let Some(ref projections) = stmt.projections {
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

                    let value = match col_name.as_str() {
                        "_type" => Value::String(type_name.clone()),
                        "_id" => Value::NodeRef(nid),
                        "*" => {
                            for (attr_name, attr_val) in node.attributes.iter() {
                                output.push_str(&format!("{}: {}\n", attr_name, format_value(attr_val)));
                            }
                            continue;
                        }
                        attr => node.get_attr(attr).cloned().unwrap_or(Value::Null),
                    };

                    output.push_str(&format!("{}: {}\n", col_name, format_value(&value)));
                }
            } else {
                // Default: return all attributes plus _type and _id
                output.push_str(&format!("_type: {}\n", type_name));
                output.push_str(&format!("_id: #{}\n", nid.raw()));

                for (attr_name, attr_val) in node.attributes.iter() {
                    output.push_str(&format!("{}: {}\n", attr_name, format_value(attr_val)));
                }
            }

            return Ok(output.trim_end().to_string());
        }
    }

    // Entity not found
    Ok(format!("Entity #{} not found", id_str))
}

/// Execute a SPAWN statement.
pub fn execute_spawn(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &mut HashMap<String, EntityId>,
    stmt: &mew_parser::SpawnStmt,
) -> Result<String, String> {
    let pattern_bindings = to_pattern_bindings(bindings);
    let mut executor = MutationExecutor::new(registry, graph);

    let result = executor
        .execute_spawn(stmt, &pattern_bindings)
        .map_err(|e| format!("Spawn error: {}", e))?;

    if let Some(node_id) = result.created_node() {
        bindings.insert(stmt.var.clone(), node_id.into());
        Ok(format!("Created {} with id {}", stmt.var, node_id.raw()))
    } else {
        Ok("Spawn completed".to_string())
    }
}

/// Execute a KILL statement.
pub fn execute_kill(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &mut HashMap<String, EntityId>,
    stmt: &mew_parser::KillStmt,
) -> Result<String, String> {
    let target_id = resolve_target(&stmt.target, bindings)?;
    let node_id = target_id
        .as_node()
        .ok_or_else(|| messages::ERR_KILL_REQUIRES_NODE.to_string())?;

    let mut executor = MutationExecutor::new(registry, graph);
    let result = executor
        .execute_kill(stmt, node_id)
        .map_err(|e| format!("Kill error: {}", e))?;

    remove_bindings_for_entity(bindings, target_id);
    Ok(format!(
        "Deleted {} nodes, {} edges",
        result.deleted_nodes(),
        result.deleted_edges()
    ))
}

/// Execute a LINK statement.
pub fn execute_link(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &mut HashMap<String, EntityId>,
    stmt: &mew_parser::LinkStmt,
) -> Result<String, String> {
    let mut targets = Vec::new();
    for target_ref in &stmt.targets {
        targets.push(resolve_target_ref(target_ref, bindings)?);
    }

    let mut executor = MutationExecutor::new(registry, graph);
    let result = executor
        .execute_link(stmt, targets)
        .map_err(|e| format!("Link error: {}", e))?;

    if let (Some(var), Some(edge_id)) = (&stmt.var, result.created_edge()) {
        bindings.insert(var.clone(), edge_id.into());
    }

    Ok("Link created".to_string())
}

/// Execute an UNLINK statement.
pub fn execute_unlink(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &mut HashMap<String, EntityId>,
    stmt: &mew_parser::UnlinkStmt,
) -> Result<String, String> {
    let target_id = resolve_target(&stmt.target, bindings)?;
    let edge_id = target_id
        .as_edge()
        .ok_or_else(|| messages::ERR_UNLINK_REQUIRES_EDGE.to_string())?;

    let mut executor = MutationExecutor::new(registry, graph);
    let result = executor
        .execute_unlink(stmt, edge_id)
        .map_err(|e| format!("Unlink error: {}", e))?;

    remove_bindings_for_entity(bindings, target_id);
    Ok(format!("Deleted {} edges", result.deleted_edges()))
}

/// Execute a SET statement.
pub fn execute_set(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &HashMap<String, EntityId>,
    stmt: &mew_parser::SetStmt,
) -> Result<String, String> {
    let target_id = resolve_target(&stmt.target, bindings)?;
    let node_id = target_id
        .as_node()
        .ok_or_else(|| messages::ERR_SET_REQUIRES_NODE.to_string())?;

    let pattern_bindings = to_pattern_bindings(bindings);
    let mut executor = MutationExecutor::new(registry, graph);
    let result = executor
        .execute_set(stmt, vec![node_id], &pattern_bindings)
        .map_err(|e| format!("Set error: {}", e))?;

    let updated = match result {
        mew_mutation::MutationOutcome::Updated(ref updated) => updated.node_ids.len(),
        _ => 0,
    };
    Ok(format!("Updated {} nodes", updated))
}

/// Execute a transaction statement.
pub fn execute_txn(in_transaction: &mut bool, stmt: &TxnStmt) -> Result<String, String> {
    match stmt {
        TxnStmt::Begin { .. } => {
            if *in_transaction {
                return Err("Transaction already active".to_string());
            }
            *in_transaction = true;
            Ok("BEGIN".to_string())
        }
        TxnStmt::Commit => {
            if !*in_transaction {
                return Err("No transaction active".to_string());
            }
            *in_transaction = false;
            Ok("COMMIT".to_string())
        }
        TxnStmt::Rollback => {
            if !*in_transaction {
                return Err("No transaction active".to_string());
            }
            *in_transaction = false;
            Ok("ROLLBACK".to_string())
        }
    }
}

/// Resolve a target to an entity ID.
///
/// Note: REPL only supports variable targets, not edge patterns.
/// For edge pattern support, use session which has registry/graph access.
pub fn resolve_target(
    t: &Target,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, String> {
    target::resolve_var_target(t, bindings)
        .map_err(|e| e.to_string())
}

/// Resolve a target reference to an entity ID.
pub fn resolve_target_ref(
    target_ref: &TargetRef,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, String> {
    target::resolve_target_ref(target_ref, bindings)
        .map_err(|e| e.to_string())
}

/// Convert REPL bindings to pattern bindings.
pub fn to_pattern_bindings(bindings: &HashMap<String, EntityId>) -> Bindings {
    let mut pattern_bindings = Bindings::new();
    for (name, entity) in bindings {
        match entity {
            EntityId::Node(node_id) => {
                pattern_bindings.insert(name.clone(), Binding::Node(*node_id))
            }
            EntityId::Edge(edge_id) => {
                pattern_bindings.insert(name.clone(), Binding::Edge(*edge_id))
            }
        }
    }
    pattern_bindings
}

/// Remove bindings for a deleted entity.
pub fn remove_bindings_for_entity(bindings: &mut HashMap<String, EntityId>, entity_id: EntityId) {
    bindings.retain(|_, value| *value != entity_id);
}

/// Execute a MATCH...mutation compound statement.
pub fn execute_match_mutate(
    registry: &Registry,
    graph: &mut Graph,
    bindings: &mut HashMap<String, EntityId>,
    stmt: &MatchMutateStmt,
) -> Result<String, String> {
    use mew_pattern::{CompiledPattern, Matcher};

    // Compile the pattern
    let mut pattern = CompiledPattern::compile(&stmt.pattern, registry)
        .map_err(|e| format!("Pattern compile error: {}", e))?;

    // Add WHERE clause as filter if present
    if let Some(ref where_expr) = stmt.where_clause {
        pattern = pattern.with_filter(where_expr.clone());
    }

    // Execute the pattern match to get all bindings
    let matcher = Matcher::new(registry, graph);
    let bindings_list = matcher
        .find_all(&pattern)
        .map_err(|e| format!("Match error: {}", e))?;

    let mut total_nodes = 0usize;
    let mut total_edges = 0usize;

    // For each set of bindings from the match, execute the mutations
    for pattern_bindings in bindings_list {
        // Convert pattern bindings to a HashMap for variable lookup
        let mut local_bindings: HashMap<String, EntityId> = HashMap::new();

        // Copy existing bindings
        for (k, v) in bindings.iter() {
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
                        let entity_id = resolve_target_ref(target_ref, &local_bindings)?;
                        targets.push(entity_id);
                    }

                    let mut executor = MutationExecutor::new(registry, graph);
                    let result = executor
                        .execute_link(link_stmt, targets)
                        .map_err(|e| format!("Link error: {}", e))?;

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
                    let target_id = resolve_target(&set_stmt.target, &local_bindings)?;
                    let node_id = target_id
                        .as_node()
                        .ok_or_else(|| messages::ERR_SET_REQUIRES_NODE.to_string())?;

                    let pb = Bindings::new();
                    let mut executor = MutationExecutor::new(registry, graph);
                    let result = executor
                        .execute_set(set_stmt, vec![node_id], &pb)
                        .map_err(|e| format!("Set error: {}", e))?;

                    use mew_mutation::MutationOutcome;
                    if let MutationOutcome::Updated(ref u) = result {
                        total_nodes += u.node_ids.len();
                    }
                }
                MutationAction::Kill(kill_stmt) => {
                    let target_id = resolve_target(&kill_stmt.target, &local_bindings)?;
                    let node_id = target_id
                        .as_node()
                        .ok_or_else(|| messages::ERR_KILL_REQUIRES_NODE.to_string())?;

                    let mut executor = MutationExecutor::new(registry, graph);
                    let result = executor
                        .execute_kill(kill_stmt, node_id)
                        .map_err(|e| format!("Kill error: {}", e))?;

                    total_nodes += result.deleted_nodes();
                    total_edges += result.deleted_edges();
                }
                MutationAction::Unlink(unlink_stmt) => {
                    let target_id = resolve_target(&unlink_stmt.target, &local_bindings)?;
                    let edge_id = target_id
                        .as_edge()
                        .ok_or_else(|| messages::ERR_UNLINK_REQUIRES_EDGE.to_string())?;

                    let mut executor = MutationExecutor::new(registry, graph);
                    let result = executor
                        .execute_unlink(unlink_stmt, edge_id)
                        .map_err(|e| format!("Unlink error: {}", e))?;

                    total_edges += result.deleted_edges();
                }
            }
        }
    }

    Ok(format!(
        "Affected {} nodes and {} edges",
        total_nodes, total_edges
    ))
}
