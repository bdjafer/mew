//! SET operation - updates node and edge attributes.

use mew_core::{EdgeId, NodeId};
use mew_graph::Graph;
use mew_parser::SetStmt;
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};
use crate::result::{MutationOutcome, UpdatedEntities};
use crate::validation;

/// Execute a SET statement to update node attributes.
pub fn execute_set(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &SetStmt,
    node_ids: Vec<NodeId>,
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    let mut updated_ids = Vec::new();

    for node_id in node_ids {
        let node = graph
            .get_node(node_id)
            .ok_or(MutationError::NodeNotFound(node_id))?;

        let type_id = node.type_id;
        let type_name = registry
            .get_type(type_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Validate and collect new attributes
        let mut new_attrs = mew_core::Attributes::new();

        for assign in &stmt.assignments {
            // Evaluate the value
            let value = evaluator.eval(&assign.value, bindings, graph)?;

            // Validate attribute (is_update=true since we're modifying an existing node)
            validation::validate_attribute(registry, &type_name, type_id, &assign.name, &value, true)?;

            new_attrs.insert(assign.name.clone(), value);
        }

        // Check uniqueness constraints, excluding the current node
        validation::check_unique_constraints(registry, graph, &type_name, type_id, &new_attrs, Some(node_id))?;

        // Apply updates
        for (name, value) in new_attrs.into_iter() {
            graph
                .set_node_attr(node_id, &name, value)
                .map_err(|e| MutationError::pattern_error(e.to_string()))?;
        }

        updated_ids.push(node_id);
    }

    Ok(MutationOutcome::Updated(UpdatedEntities::nodes(updated_ids)))
}

/// Execute a SET statement to update edge attributes.
pub fn execute_set_edge(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &SetStmt,
    edge_ids: Vec<EdgeId>,
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    let mut updated_ids = Vec::new();

    for edge_id in edge_ids {
        let edge = graph
            .get_edge(edge_id)
            .ok_or(MutationError::EdgeNotFound(edge_id))?;

        let type_id = edge.type_id;
        let type_name = registry
            .get_edge_type(type_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Validate and collect new attributes
        let mut new_attrs = Vec::new();

        for assign in &stmt.assignments {
            // Evaluate the value
            let value = evaluator.eval(&assign.value, bindings, graph)?;

            // Validate attribute for edge type (is_update=true since we're modifying an existing edge)
            validation::validate_edge_attribute(registry, &type_name, type_id, &assign.name, &value, true)?;

            new_attrs.push((assign.name.clone(), value));
        }

        // Apply updates
        for (name, value) in new_attrs {
            graph
                .set_edge_attr(edge_id, &name, value)
                .map_err(|e| MutationError::pattern_error(e.to_string()))?;
        }

        updated_ids.push(edge_id);
    }

    Ok(MutationOutcome::Updated(UpdatedEntities::edges(updated_ids)))
}
