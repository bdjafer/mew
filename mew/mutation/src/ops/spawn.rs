//! SPAWN operation - creates new nodes.

use mew_core::NodeId;
use mew_graph::Graph;
use mew_parser::{SpawnItem, SpawnStmt};
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};
use crate::result::{CreatedEntity, MutationOutcome};
use crate::validation;

/// Execute a single spawn item to create a new node.
fn execute_spawn_item(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    item: &SpawnItem,
    bindings: &Bindings,
) -> MutationResult<NodeId> {
    // Look up the type
    let type_id = registry
        .get_type_id(&item.type_name)
        .ok_or_else(|| MutationError::unknown_type(&item.type_name))?;

    // Check that type is not abstract
    if let Some(type_def) = registry.get_type(type_id) {
        if type_def.is_abstract {
            return Err(MutationError::abstract_type(&item.type_name));
        }
    }

    // Build attributes
    let mut attrs = mew_core::Attributes::new();

    for assign in &item.attrs {
        // Evaluate the value expression
        let value = evaluator.eval(&assign.value, bindings, graph)?;

        // Validate attribute exists and type matches (is_update=false since this is a new node)
        validation::validate_attribute(registry, &item.type_name, type_id, &assign.name, &value, false)?;

        attrs.insert(assign.name.clone(), value);
    }

    // Check required attributes
    validation::check_required_attributes(registry, &item.type_name, type_id, &attrs)?;

    // Apply default values
    validation::apply_defaults(registry, type_id, &mut attrs)?;

    // Check uniqueness constraints
    validation::check_unique_constraints(registry, graph, &item.type_name, type_id, &attrs, None)?;

    // Create the node
    let node_id = graph.create_node(type_id, attrs);

    Ok(node_id)
}

/// Execute a SPAWN statement to create new node(s).
/// Supports both single spawns and chained spawns (SPAWN a: T, SPAWN b: U).
pub fn execute_spawn(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &SpawnStmt,
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    let mut created_nodes = Vec::new();

    for item in &stmt.items {
        let node_id = execute_spawn_item(registry, graph, evaluator, item, bindings)?;
        created_nodes.push(node_id);
    }

    // For single spawns, return just the node; for multiple, return all
    if created_nodes.len() == 1 {
        Ok(MutationOutcome::Created(CreatedEntity::node(created_nodes[0])))
    } else {
        Ok(MutationOutcome::Created(CreatedEntity::nodes(created_nodes)))
    }
}
