//! UNLINK operation - deletes edges.

use mew_core::EdgeId;
use mew_graph::Graph;
use mew_parser::UnlinkStmt;

use crate::error::{MutationError, MutationResult};
use crate::result::{DeletedEntities, MutationResult as MutationOutput};

/// Execute an UNLINK statement to delete an edge.
pub fn execute_unlink(
    graph: &mut Graph,
    _stmt: &UnlinkStmt,
    target_id: EdgeId,
) -> MutationResult<MutationOutput> {
    // Check edge exists
    if graph.get_edge(target_id).is_none() {
        return Err(MutationError::EdgeNotFound(target_id));
    }

    // Delete higher-order edges about this edge first
    let ho_edges: Vec<_> = graph.edges_about(target_id).collect();
    let mut deleted_edges = Vec::new();

    for ho_edge_id in ho_edges {
        if graph.delete_edge(ho_edge_id).is_ok() {
            deleted_edges.push(ho_edge_id);
        }
    }

    // Delete the edge
    let _ = graph.delete_edge(target_id);
    deleted_edges.push(target_id);

    Ok(MutationOutput::Deleted(
        DeletedEntities::edge(target_id).with_cascade_edges(deleted_edges),
    ))
}
