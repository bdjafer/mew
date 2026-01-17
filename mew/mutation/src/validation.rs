//! Attribute validation helpers for mutation operations.

use mew_core::{EdgeTypeId, NodeId, TypeId, Value};
use mew_graph::Graph;
use mew_registry::Registry;

use crate::error::{MutationError, MutationResult};

/// Format a value for display in error messages.
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Timestamp(t) => t.to_string(),
        Value::Duration(d) => d.to_string(),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        Value::NodeRef(id) => format!("node:{:?}", id),
        Value::EdgeRef(id) => format!("edge:{:?}", id),
    }
}

/// Validate an attribute assignment against the registry.
/// Use `is_update` = true for SET operations to enforce readonly constraints.
pub fn validate_attribute(
    registry: &Registry,
    type_name: &str,
    type_id: TypeId,
    attr_name: &str,
    value: &Value,
    is_update: bool,
) -> MutationResult<()> {
    // Use get_type_attr to check inherited attributes
    if let Some(attr_def) = registry.get_type_attr(type_id, attr_name) {
        // Check if trying to modify a readonly attribute during SET
        if is_update && attr_def.readonly {
            return Err(MutationError::readonly_attribute_violation(type_name, attr_name));
        }

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

        // Check format constraint
        if let Some(ref format) = attr_def.format {
            validate_format(attr_name, value, format)?;
        }

        // Check match pattern constraint
        if let Some(ref pattern) = attr_def.match_pattern {
            validate_match_pattern(attr_name, value, pattern)?;
        }

        // Check allowed values constraint (in: [...])
        if let Some(ref allowed_values) = attr_def.allowed_values {
            validate_allowed_values(attr_name, value, allowed_values)?;
        }

        // Check length constraint (for strings)
        if attr_def.length_min.is_some() || attr_def.length_max.is_some() {
            validate_length(attr_name, value, attr_def.length_min, attr_def.length_max)?;
        }
    } else {
        return Err(MutationError::unknown_attribute(type_name, attr_name));
    }

    Ok(())
}

/// Validate an edge attribute assignment against the registry.
/// Use `is_update` = true for SET operations to enforce readonly constraints.
pub fn validate_edge_attribute(
    registry: &Registry,
    edge_type_name: &str,
    edge_type_id: EdgeTypeId,
    attr_name: &str,
    value: &Value,
    is_update: bool,
) -> MutationResult<()> {
    // Get the edge type definition
    let edge_type = registry
        .get_edge_type(edge_type_id)
        .ok_or_else(|| MutationError::unknown_attribute(edge_type_name, attr_name))?;

    // Look up the attribute in the edge type
    if let Some(attr_def) = edge_type.attributes.get(attr_name) {
        // Check if trying to modify a readonly attribute during SET
        if is_update && attr_def.readonly {
            return Err(MutationError::readonly_attribute_violation(edge_type_name, attr_name));
        }

        // Check if trying to set a required attribute to null
        if attr_def.required && matches!(value, Value::Null) {
            return Err(MutationError::required_null_violation(edge_type_name, attr_name));
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
        return Err(MutationError::unknown_attribute(edge_type_name, attr_name));
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

/// Check that all required edge attributes are present.
pub fn check_required_edge_attributes(
    registry: &Registry,
    edge_type_name: &str,
    edge_type_id: EdgeTypeId,
    attrs: &mew_core::Attributes,
) -> MutationResult<()> {
    if let Some(edge_type) = registry.get_edge_type(edge_type_id) {
        for (name, attr_def) in &edge_type.attributes {
            if attr_def.required && !attrs.contains_key(name) && attr_def.default.is_none() {
                return Err(MutationError::missing_required(edge_type_name, name));
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

/// Check uniqueness constraints on attributes.
/// Excludes `exclude_node` when checking (used for SET operations on existing nodes).
pub fn check_unique_constraints(
    registry: &Registry,
    graph: &Graph,
    type_name: &str,
    type_id: TypeId,
    attrs: &mew_core::Attributes,
    exclude_node: Option<NodeId>,
) -> MutationResult<()> {
    // Get all attrs including inherited ones and check for unique constraints
    for attr_def in registry.get_all_type_attrs(type_id) {
        if !attr_def.unique {
            continue;
        }

        // Get the value for this attribute
        let value = match attrs.get(&attr_def.name) {
            Some(v) => v,
            None => continue, // Attribute not being set
        };

        // Skip null values - they don't violate uniqueness
        if matches!(value, Value::Null) {
            continue;
        }

        // Find the type that declares this unique attribute
        // For inheritance, uniqueness is checked across the declaring type and all subtypes
        let declaring_type_id = find_declaring_type(registry, type_id, &attr_def.name);

        // Check all nodes of the declaring type and its subtypes
        if value_exists_in_nodes(
            registry,
            graph,
            declaring_type_id,
            &attr_def.name,
            value,
            exclude_node,
        ) {
            return Err(MutationError::unique_constraint_violation(
                type_name,
                &attr_def.name,
                format_value(value),
            ));
        }
    }
    Ok(())
}

/// Find the type that originally declares an attribute (for inheritance).
/// Recursively walks up the type hierarchy to find the topmost type.
fn find_declaring_type(registry: &Registry, type_id: TypeId, attr_name: &str) -> TypeId {
    // Check each supertype recursively
    for supertype_id in registry.get_supertypes(type_id) {
        if let Some(supertype) = registry.get_type(supertype_id) {
            if supertype.attributes.contains_key(attr_name) {
                // This supertype declares the attribute, but check if there's an even higher one
                return find_declaring_type(registry, supertype_id, attr_name);
            }
        }
    }
    // No supertype declares it, so this type is the declarer
    type_id
}

/// Check if a value exists in any node of the given type or its subtypes.
fn value_exists_in_nodes(
    registry: &Registry,
    graph: &Graph,
    type_id: TypeId,
    attr_name: &str,
    value: &Value,
    exclude_node: Option<NodeId>,
) -> bool {
    // Collect all type IDs to check (the type itself plus all subtypes)
    let mut types_to_check = vec![type_id];
    types_to_check.extend(registry.get_subtypes(type_id));

    // Check all nodes of these types
    for check_type_id in types_to_check {
        for node_id in graph.nodes_by_type(check_type_id) {
            // Skip the excluded node
            if Some(node_id) == exclude_node {
                continue;
            }

            if let Some(node) = graph.get_node(node_id) {
                if let Some(node_value) = node.attributes.get(attr_name) {
                    if node_value == value {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Validate a value against a format constraint.
pub fn validate_format(attr_name: &str, value: &Value, format: &str) -> MutationResult<()> {
    // Skip null values
    if matches!(value, Value::Null) {
        return Ok(());
    }

    let s = match value {
        Value::String(s) => s,
        _ => return Ok(()), // Format only applies to strings
    };

    let valid = match format {
        "slug" => is_valid_slug(s),
        "email" => is_valid_email(s),
        "url" => is_valid_url(s),
        "uuid" => is_valid_uuid(s),
        _ => true, // Unknown format - don't validate
    };

    if !valid {
        return Err(MutationError::format_constraint_violation(
            attr_name,
            s.clone(),
            format,
        ));
    }

    Ok(())
}

/// Validate a value against a match pattern constraint.
pub fn validate_match_pattern(attr_name: &str, value: &Value, pattern: &str) -> MutationResult<()> {
    // Skip null values
    if matches!(value, Value::Null) {
        return Ok(());
    }

    let s = match value {
        Value::String(s) => s,
        _ => return Ok(()), // Match pattern only applies to strings
    };

    // Compile and test the regex
    match regex_lite::Regex::new(pattern) {
        Ok(re) => {
            if !re.is_match(s) {
                return Err(MutationError::pattern_constraint_violation(
                    attr_name,
                    s.clone(),
                    pattern,
                ));
            }
        }
        Err(_) => {
            // Invalid regex pattern - skip validation but could log warning
        }
    }

    Ok(())
}

/// Check if a string is a valid slug (lowercase letters, numbers, and hyphens).
/// Slugs must not start or end with a hyphen, and cannot have consecutive hyphens.
fn is_valid_slug(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('-') || s.ends_with('-') {
        return false;
    }
    if s.contains("--") {
        return false;
    }
    s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check if a string looks like a valid email (simple validation).
fn is_valid_email(s: &str) -> bool {
    // Simple email validation: contains @ with text before and after
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    !parts[0].is_empty() && parts[1].contains('.') && !parts[1].starts_with('.') && !parts[1].ends_with('.')
}

/// Check if a string looks like a valid URL.
fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Check if a string is a valid UUID format.
fn is_valid_uuid(s: &str) -> bool {
    // UUID format: 8-4-4-4-12 hex digits
    let pattern = regex_lite::Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$");
    pattern.map(|re| re.is_match(s)).unwrap_or(false)
}

/// Validate a value against allowed values constraint (in: [...]).
pub fn validate_allowed_values(attr_name: &str, value: &Value, allowed_values: &[Value]) -> MutationResult<()> {
    // Skip null values
    if matches!(value, Value::Null) {
        return Ok(());
    }

    // Check if value is in the allowed list
    if !allowed_values.contains(value) {
        return Err(MutationError::allowed_values_violation(
            attr_name,
            format_value(value),
        ));
    }

    Ok(())
}

/// Validate a value against length constraints (for strings).
pub fn validate_length(
    attr_name: &str,
    value: &Value,
    length_min: Option<i64>,
    length_max: Option<i64>,
) -> MutationResult<()> {
    // Skip null values
    if matches!(value, Value::Null) {
        return Ok(());
    }

    let s = match value {
        Value::String(s) => s,
        _ => return Ok(()), // Length only applies to strings
    };

    let actual_length = s.chars().count();
    let min = length_min.unwrap_or(0);
    let max = length_max.unwrap_or(i64::MAX);

    if (actual_length as i64) < min || (actual_length as i64) > max {
        return Err(MutationError::length_constraint_violation(
            attr_name,
            actual_length,
            min,
            max,
        ));
    }

    Ok(())
}

