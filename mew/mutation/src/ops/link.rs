//! LINK operation - creates edges between entities.

use mew_core::{EdgeId, EdgeTypeId, EntityId, NodeId};
use mew_graph::Graph;
use mew_parser::LinkStmt;
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;
use std::collections::{HashSet, VecDeque};

use crate::error::{MutationError, MutationResult};
use crate::result::{CreatedEntity, MutationOutcome};

/// Execute a LINK statement to create an edge.
pub fn execute_link(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &LinkStmt,
    target_ids: Vec<EntityId>,
) -> MutationResult<MutationOutcome> {
    // Look up the edge type
    let edge_type_id = registry
        .get_edge_type_id(&stmt.edge_type)
        .ok_or_else(|| MutationError::unknown_edge_type(&stmt.edge_type))?;

    // If IF NOT EXISTS, check if edge already exists
    if stmt.if_not_exists {
        if find_existing_edge(graph, edge_type_id, &target_ids).is_some() {
            // Edge already exists - no new edge created
            return Ok(MutationOutcome::Empty);
        }
    }

    // Validate arity and target types
    if let Some(edge_type) = registry.get_edge_type(edge_type_id) {
        let expected = edge_type.params.len();
        let actual = target_ids.len();
        if expected != actual {
            return Err(MutationError::invalid_arity(
                &stmt.edge_type,
                expected,
                actual,
            ));
        }

        // Validate target types
        for (i, (param, target_id)) in edge_type.params.iter().zip(target_ids.iter()).enumerate() {
            if let EntityId::Node(node_id) = target_id {
                if let Some(node) = graph.get_node(*node_id) {
                    // Check if node type matches expected parameter type
                    // "any" means any type is allowed
                    if param.type_constraint != "any" {
                        let expected_type_id = registry.get_type_id(&param.type_constraint);
                        if let Some(expected_id) = expected_type_id {
                            if !registry.is_subtype(node.type_id, expected_id) {
                                let actual_name = registry
                                    .get_type(node.type_id)
                                    .map(|t| t.name.clone())
                                    .unwrap_or_else(|| "unknown".to_string());
                                return Err(MutationError::target_type_mismatch(
                                    i,
                                    &param.type_constraint,
                                    actual_name,
                                ));
                            }
                        }
                    }
                }
            }
        }

        if edge_type.acyclic {
            ensure_acyclic(graph, edge_type_id, &stmt.edge_type, &target_ids)?;
        }
    }

    // Build attributes
    let mut attrs = mew_core::Attributes::new();
    let bindings = Bindings::new();

    for assign in &stmt.attrs {
        let value = evaluator.eval(&assign.value, &bindings, graph)?;
        attrs.insert(assign.name.clone(), value);
    }

    // Apply default values for missing edge attributes
    crate::validation::apply_edge_defaults(registry, edge_type_id, &mut attrs)?;

    // Create the edge
    let edge_id = graph
        .create_edge(edge_type_id, target_ids, attrs)
        .map_err(|e| MutationError::pattern_error(e.to_string()))?;

    Ok(MutationOutcome::Created(CreatedEntity::edge(edge_id)))
}

/// Ensure creating this edge wouldn't create a cycle (for acyclic edge types).
fn ensure_acyclic(
    graph: &Graph,
    edge_type_id: EdgeTypeId,
    edge_type_name: &str,
    target_ids: &[EntityId],
) -> MutationResult<()> {
    if target_ids.len() < 2 {
        return Ok(());
    }

    let source = match target_ids[0] {
        EntityId::Node(node_id) => node_id,
        _ => return Ok(()),
    };
    let target = match target_ids[1] {
        EntityId::Node(node_id) => node_id,
        _ => return Ok(()),
    };

    if source == target {
        return Err(MutationError::acyclic_violation(edge_type_name));
    }

    if path_exists(graph, edge_type_id, target, source) {
        return Err(MutationError::acyclic_violation(edge_type_name));
    }

    Ok(())
}

/// Find an existing edge with the given type and exact targets.
/// Returns the edge ID if found, None otherwise.
fn find_existing_edge(graph: &Graph, edge_type_id: EdgeTypeId, target_ids: &[EntityId]) -> Option<EdgeId> {
    // Get candidate edges based on first target type
    let candidate_edges: Vec<_> = match target_ids.first() {
        Some(EntityId::Node(source_id)) => {
            graph.edges_from(*source_id, Some(edge_type_id)).collect()
        }
        Some(EntityId::Edge(source_edge_id)) => {
            // For higher-order edges, check edges about this edge
            graph
                .edges_about(*source_edge_id)
                .filter(|e| {
                    graph
                        .get_edge(*e)
                        .map(|edge| edge.type_id == edge_type_id)
                        .unwrap_or(false)
                })
                .collect()
        }
        None => return None,
    };

    // Check if any candidate has exact matching targets
    for edge_id in candidate_edges {
        if let Some(edge) = graph.get_edge(edge_id) {
            if edge.targets.len() == target_ids.len() {
                let matches = edge
                    .targets
                    .iter()
                    .zip(target_ids.iter())
                    .all(|(a, b)| a == b);
                if matches {
                    return Some(edge_id);
                }
            }
        }
    }
    None
}

/// Check if a path exists from start to goal using edges of the given type.
fn path_exists(graph: &Graph, edge_type_id: EdgeTypeId, start: NodeId, goal: NodeId) -> bool {
    let mut visited = HashSet::new();
    let mut stack = VecDeque::new();
    stack.push_back(start);

    while let Some(node_id) = stack.pop_front() {
        if node_id == goal {
            return true;
        }
        if !visited.insert(node_id) {
            continue;
        }
        for edge_id in graph.edges_from(node_id, Some(edge_type_id)) {
            if let Some(edge) = graph.get_edge(edge_id) {
                if let Some(EntityId::Node(next_id)) = edge.targets.get(1) {
                    if !visited.contains(next_id) {
                        stack.push_back(*next_id);
                    }
                }
            }
        }
    }

    false
}
