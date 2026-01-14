//! SPAWN operation - creates new nodes.

use mew_graph::Graph;
use mew_parser::SpawnStmt;
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
    // Look up the type
    let type_id = registry
        .get_type_id(&stmt.type_name)
        .ok_or_else(|| MutationError::unknown_type(&stmt.type_name))?;

    // Check that type is not abstract
    if let Some(type_def) = registry.get_type(type_id) {
        if type_def.is_abstract {
            return Err(MutationError::abstract_type(&stmt.type_name));
        }
    }

    // Build attributes
    let mut attrs = mew_core::Attributes::new();

    for assign in &stmt.attrs {
        // Evaluate the value expression
        let value = evaluator.eval(&assign.value, bindings, graph)?;

        // Validate attribute exists and type matches (is_update=false since this is a new node)
        validation::validate_attribute(registry, &stmt.type_name, type_id, &assign.name, &value, false)?;

        attrs.insert(assign.name.clone(), value);
    }

    // Check required attributes
    validation::check_required_attributes(registry, &stmt.type_name, type_id, &attrs)?;

    // Apply default values
    validation::apply_defaults(registry, type_id, &mut attrs)?;

    // Check uniqueness constraints
    validation::check_unique_constraints(registry, graph, &stmt.type_name, type_id, &attrs, None)?;

    // Create the node
    let node_id = graph.create_node(type_id, attrs);

    Ok(MutationOutcome::Created(CreatedEntity::node(node_id)))
}
