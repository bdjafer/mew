//! SET operation - updates node attributes.

use mew_core::NodeId;
use mew_graph::Graph;
use mew_parser::SetStmt;
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};
use crate::result::{MutationResult as MutationOutput, UpdatedEntities};
use crate::validation;

/// Execute a SET statement to update node attributes.
pub fn execute_set(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &SetStmt,
    node_ids: Vec<NodeId>,
    bindings: &Bindings,
) -> MutationResult<MutationOutput> {
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
        let mut new_attrs = Vec::new();

        for assign in &stmt.assignments {
            // Evaluate the value
            let value = evaluator.eval(&assign.value, bindings, graph)?;

            // Validate attribute
            validation::validate_attribute(registry, &type_name, type_id, &assign.name, &value)?;

            new_attrs.push((assign.name.clone(), value));
        }

        // Apply updates
        for (name, value) in new_attrs {
            graph
                .set_node_attr(node_id, &name, value)
                .map_err(|e| MutationError::pattern_error(e.to_string()))?;
        }

        updated_ids.push(node_id);
    }

    Ok(MutationOutput::Updated(UpdatedEntities::nodes(updated_ids)))
}
