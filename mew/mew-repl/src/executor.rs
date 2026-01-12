//! Statement execution for the REPL.

use std::collections::HashMap;

use mew_core::EntityId;
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{MatchStmt, Target, TargetRef, TxnStmt};
use mew_pattern::{Binding, Bindings};
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
        .ok_or_else(|| "KILL requires a node target".to_string())?;

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
        .ok_or_else(|| "UNLINK requires an edge target".to_string())?;

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
        .ok_or_else(|| "SET requires a node target".to_string())?;

    let pattern_bindings = to_pattern_bindings(bindings);
    let mut executor = MutationExecutor::new(registry, graph);
    let result = executor
        .execute_set(stmt, vec![node_id], &pattern_bindings)
        .map_err(|e| format!("Set error: {}", e))?;

    let updated = match result {
        mew_mutation::MutationOutput::Updated(ref updated) => updated.node_ids.len(),
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
pub fn resolve_target(
    target: &Target,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, String> {
    match target {
        Target::Var(name) => bindings
            .get(name)
            .copied()
            .ok_or_else(|| format!("Unknown variable: {}", name)),
        Target::Id(_) | Target::Pattern(_) => {
            Err("Only variable targets are supported".to_string())
        }
    }
}

/// Resolve a target reference to an entity ID.
pub fn resolve_target_ref(
    target_ref: &TargetRef,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, String> {
    match target_ref {
        TargetRef::Var(name) => bindings
            .get(name)
            .copied()
            .ok_or_else(|| format!("Unknown variable: {}", name)),
        TargetRef::Id(_) | TargetRef::Pattern(_) => {
            Err("Only variable targets are supported".to_string())
        }
    }
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
