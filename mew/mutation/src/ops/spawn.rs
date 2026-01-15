//! SPAWN operation - creates new nodes.

use mew_graph::Graph;
use mew_parser::{SpawnItem, SpawnStmt};
use mew_pattern::{Bindings, Evaluator};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};
use crate::result::{CreatedEntity, MutationOutcome};
use crate::validation;

/// Execute a SPAWN statement to create a new node.
pub fn execute_spawn(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    stmt: &SpawnStmt,
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    execute_spawn_core(registry, graph, evaluator, &stmt.type_name, &stmt.attrs, bindings)
}

/// Execute a spawn from a SpawnItem (used by multi-spawn).
pub fn execute_spawn_item(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    item: &SpawnItem,
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    execute_spawn_core(registry, graph, evaluator, &item.type_name, &item.attrs, bindings)
}

/// Core spawn logic shared by execute_spawn and execute_spawn_item.
fn execute_spawn_core(
    registry: &Registry,
    graph: &mut Graph,
    evaluator: &Evaluator,
    type_name: &str,
    attr_assignments: &[mew_parser::AttrAssignment],
    bindings: &Bindings,
) -> MutationResult<MutationOutcome> {
    // Look up the type
    let type_id = registry
        .get_type_id(type_name)
        .ok_or_else(|| MutationError::unknown_type(type_name))?;

    // Check that type is not abstract
    if let Some(type_def) = registry.get_type(type_id) {
        if type_def.is_abstract {
            return Err(MutationError::abstract_type(type_name));
        }
    }

    // Build attributes
    let mut attrs = mew_core::Attributes::new();

    for assign in attr_assignments {
        // Evaluate the value expression
        let value = evaluator.eval(&assign.value, bindings, graph)?;

        // Validate attribute exists and type matches (is_update=false since this is a new node)
        validation::validate_attribute(registry, type_name, type_id, &assign.name, &value, false)?;

        attrs.insert(assign.name.clone(), value);
    }

    // Check required attributes
    validation::check_required_attributes(registry, type_name, type_id, &attrs)?;

    // Apply default values
    validation::apply_defaults(registry, type_id, &mut attrs)?;

    // Check uniqueness constraints
    validation::check_unique_constraints(registry, graph, type_name, type_id, &attrs, None)?;

    // Create the node
    let node_id = graph.create_node(type_id, attrs);

    Ok(MutationOutcome::Created(CreatedEntity::node(node_id)))
}
