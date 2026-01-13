//! KILL operation - deletes nodes with cascade support.

use mew_core::{EntityId, NodeId};
use mew_graph::Graph;
use mew_parser::KillStmt;
use mew_registry::{EdgeTypeDef, OnKillAction, Registry};
use std::collections::{HashSet, VecDeque};

use crate::error::{MutationError, MutationResult};
use crate::result::{DeletedEntities, MutationOutcome};

/// Execute a KILL statement to delete a node and handle cascades.
pub fn execute_kill(
    registry: &Registry,
    graph: &mut Graph,
    stmt: &KillStmt,
    target_id: NodeId,
) -> MutationResult<MutationOutcome> {
    // Check node exists
    if graph.get_node(target_id).is_none() {
        return Err(MutationError::NodeNotFound(target_id));
    }

    let cascade = stmt.cascade.unwrap_or(true);
    let mut deleted_edges = HashSet::new();
    let mut deleted_nodes = HashSet::new();
    let mut to_delete = VecDeque::new();
    to_delete.push_back(target_id);

    while let Some(node_id) = to_delete.pop_front() {
        if deleted_nodes.contains(&node_id) {
            continue;
        }

        if cascade {
            process_cascade(registry, graph, node_id, &mut to_delete, &deleted_nodes)?;
        }

        // Collect edges to delete (cascade)
        let edges_from: Vec<_> = graph.edges_from(node_id, None).collect();
        let edges_to: Vec<_> = graph.edges_to(node_id, None).collect();

        for edge_id in edges_from.into_iter().chain(edges_to) {
            // Also delete higher-order edges about this edge
            let ho_edges: Vec<_> = graph.edges_about(edge_id).collect();
            for ho_edge_id in ho_edges {
                if graph.delete_edge(ho_edge_id).is_ok() {
                    deleted_edges.insert(ho_edge_id);
                }
            }

            if graph.delete_edge(edge_id).is_ok() {
                deleted_edges.insert(edge_id);
            }
        }

        // Delete the node
        let _ = graph.delete_node(node_id);
        deleted_nodes.insert(node_id);
    }

    Ok(MutationOutcome::Deleted(
        DeletedEntities::nodes(deleted_nodes.into_iter().collect())
            .with_cascade_edges(deleted_edges.into_iter().collect()),
    ))
}

/// Process cascade deletions for incident edges of a node.
fn process_cascade(
    registry: &Registry,
    graph: &Graph,
    node_id: NodeId,
    to_delete: &mut VecDeque<NodeId>,
    deleted_nodes: &HashSet<NodeId>,
) -> MutationResult<()> {
    let incident_edges: Vec<_> = graph
        .edges_from(node_id, None)
        .chain(graph.edges_to(node_id, None))
        .collect();

    for edge_id in incident_edges {
        if let Some(edge) = graph.get_edge(edge_id) {
            let edge_type = registry
                .get_edge_type(edge.type_id)
                .ok_or_else(|| MutationError::unknown_edge_type(edge.type_id.0.to_string()))?;
            let target_index = edge
                .targets
                .iter()
                .position(|target| matches!(target, EntityId::Node(id) if *id == node_id));
            if let Some(index) = target_index {
                let action = on_kill_action(edge_type, index);
                match action {
                    OnKillAction::Cascade => {
                        for target in &edge.targets {
                            if let EntityId::Node(other_id) = target {
                                if *other_id != node_id && !deleted_nodes.contains(other_id) {
                                    to_delete.push_back(*other_id);
                                }
                            }
                        }
                    }
                    OnKillAction::Restrict => {
                        return Err(MutationError::on_kill_restrict(edge_type.name.clone()));
                    }
                    OnKillAction::SetNull | OnKillAction::Delete => {}
                }
            }
        }
    }

    Ok(())
}

/// Determine the on-kill action for an edge at a specific target index.
fn on_kill_action(edge_type: &EdgeTypeDef, index: usize) -> OnKillAction {
    if edge_type.on_kill.is_empty() {
        return OnKillAction::Delete;
    }
    if edge_type.on_kill.len() == 1 {
        return edge_type.on_kill[0];
    }
    edge_type
        .on_kill
        .get(index)
        .copied()
        .unwrap_or(OnKillAction::Delete)
}
