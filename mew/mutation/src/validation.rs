//! Attribute validation helpers for mutation operations.

use mew_core::{TypeId, Value};
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};

/// Validate an attribute assignment against the registry.
/// Use `is_update` = true for SET operations (currently unused but kept for future extensions).
pub fn validate_attribute(
    registry: &Registry,
    type_name: &str,
    type_id: TypeId,
    attr_name: &str,
    value: &Value,
    _is_update: bool,
) -> MutationResult<()> {
    // Use get_type_attr to check inherited attributes
    if let Some(attr_def) = registry.get_type_attr(type_id, attr_name) {
        // Check if trying to set a required attribute to null
        if attr_def.required && matches!(value, Value::Null) {
            return Err(MutationError::required_null_violation(type_name, attr_name));
        }

        // Check type compatibility
        let expected_type = &attr_def.type_name;
        let actual_type = value_type_name(value);

        if !types_compatible(expected_type, &actual_type) {
            return Err(MutationError::invalid_attr_type(
                attr_name,
                expected_type,
                actual_type,
            ));
        }

        // Check range constraints (min/max)
        validate_range(attr_name, value, &attr_def.min, &attr_def.max)?;
    } else {
        return Err(MutationError::unknown_attribute(type_name, attr_name));
    }

    Ok(())
}

/// Validate range constraints (min/max) for a value.
pub fn validate_range(
    attr_name: &str,
    value: &Value,
    min: &Option<Value>,
    max: &Option<Value>,
) -> MutationResult<()> {
    // Skip null values - they don't violate range constraints
    if matches!(value, Value::Null) {
        return Ok(());
    }

    // Check minimum constraint
    if let Some(min_val) = min {
        if !value.gte(min_val) {
            let range_desc = match max {
                Some(max_val) => format!(" [{:?}..{:?}]", min_val, max_val),
                None => format!(" [>= {:?}]", min_val),
            };
            return Err(MutationError::range_violation(
                attr_name,
                format!("{:?}", value),
                range_desc,
            ));
        }
    }

    // Check maximum constraint
    if let Some(max_val) = max {
        if !value.lte(max_val) {
            let range_desc = match min {
                Some(min_val) => format!(" [{:?}..{:?}]", min_val, max_val),
                None => format!(" [<= {:?}]", max_val),
            };
            return Err(MutationError::range_violation(
                attr_name,
                format!("{:?}", value),
                range_desc,
            ));
        }
    }

    Ok(())
}

/// Check that all required attributes are present.
pub fn check_required_attributes(
    registry: &Registry,
    type_name: &str,
    type_id: TypeId,
    attrs: &mew_core::Attributes,
) -> MutationResult<()> {
    // Get all attrs including inherited ones
    for attr_def in registry.get_all_type_attrs(type_id) {
        if attr_def.required && !attrs.contains_key(&attr_def.name) && attr_def.default.is_none() {
            return Err(MutationError::missing_required(type_name, &attr_def.name));
        }
    }
    Ok(())
}

/// Apply default values to missing node attributes.
pub fn apply_defaults(
    registry: &Registry,
    type_id: TypeId,
    attrs: &mut mew_core::Attributes,
) -> MutationResult<()> {
    // Get all attrs including inherited ones
    for attr_def in registry.get_all_type_attrs(type_id) {
        if !attrs.contains_key(&attr_def.name) {
            if let Some(ref default_value) = attr_def.default {
                attrs.insert(attr_def.name.clone(), default_value.clone());
            }
        }
    }
    Ok(())
}

/// Apply default values to missing edge attributes.
pub fn apply_edge_defaults(
    registry: &Registry,
    edge_type_id: mew_core::EdgeTypeId,
    attrs: &mut mew_core::Attributes,
) -> MutationResult<()> {
    if let Some(edge_type) = registry.get_edge_type(edge_type_id) {
        for (name, attr_def) in &edge_type.attributes {
            if !attrs.contains_key(name) {
                if let Some(ref default_value) = attr_def.default {
                    attrs.insert(name.clone(), default_value.clone());
                }
            }
        }
    }
    Ok(())
}

/// Get the type name of a value.
pub fn value_type_name(value: &Value) -> String {
    value.type_name().to_string()
}

/// Check if types are compatible.
pub fn types_compatible(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    // Null is compatible with anything
    if actual == "Null" {
        return true;
    }
    // Int can be used where Float is expected
    if expected == "Float" && actual == "Int" {
        return true;
    }
    // Int can be used where Duration/Timestamp are expected (they are Int-based types)
    if (expected == "Duration" || expected == "Timestamp") && actual == "Int" {
        return true;
    }
    false
}

