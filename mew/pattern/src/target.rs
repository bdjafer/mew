//! Target resolution utilities.
//!
//! Shared logic for resolving targets (variables, edge patterns) to entity IDs.
//! Used by both session and REPL to avoid code duplication.

use std::collections::HashMap;

use mew_core::EntityId;
use mew_graph::Graph;
use mew_parser::{Target, TargetRef};
use mew_registry::Registry;
use thiserror::Error;

/// Error returned when target resolution fails.
#[derive(Debug, Clone, Error)]
pub enum TargetError {
    /// Variable not found in bindings.
    #[error("Variable not found: {name}")]
    UnknownVariable { name: String },

    /// Target type not supported (Pattern).
    #[error("Unsupported target type: {target_type}")]
    UnsupportedTarget { target_type: String },

    /// Edge type not found in registry.
    #[error("Unknown edge type: {name}")]
    UnknownEdgeType { name: String },

    /// Edge pattern requires at least 2 targets.
    #[error("Edge pattern requires at least 2 targets, got {actual}")]
    InsufficientTargets { actual: usize },

    /// Source in edge pattern must be a node.
    #[error("Source at position {position} must be a node, got {actual_type}")]
    SourceNotNode {
        position: usize,
        actual_type: String,
    },

    /// Target in edge pattern must be a node.
    #[error("Target at position {position} must be a node, got {actual_type}")]
    TargetNotNode {
        position: usize,
        actual_type: String,
    },

    /// No edge found matching the pattern.
    #[error("Edge not found: no edge of type '{edge_type}' between specified nodes")]
    EdgeNotFound { edge_type: String },
}

/// Resolve a binding name to an EntityId.
///
/// Used for both Var and Id targets which reference bound variables.
fn resolve_binding(name: &str, bindings: &HashMap<String, EntityId>) -> Result<EntityId, TargetError> {
    bindings
        .get(name)
        .copied()
        .ok_or_else(|| TargetError::UnknownVariable {
            name: name.to_string(),
        })
}

/// Resolve a target to an EntityId.
///
/// For simple variable targets, just looks up in bindings.
/// For edge patterns, searches the graph for matching edges.
pub fn resolve_target(
    target: &Target,
    bindings: &HashMap<String, EntityId>,
    registry: &Registry,
    graph: &Graph,
) -> Result<EntityId, TargetError> {
    match target {
        Target::Var(var_name) => resolve_binding(var_name, bindings),

        Target::Id(id_name) => resolve_binding(id_name, bindings),

        Target::Pattern(_) => Err(TargetError::UnsupportedTarget {
            target_type: "Pattern".to_string(),
        }),

        Target::EdgePattern { edge_type, targets } => {
            resolve_edge_pattern(edge_type, targets, bindings, registry, graph)
        }
    }
}

/// Resolve a target reference to an EntityId.
///
/// Simpler than resolve_target - only handles variables.
pub fn resolve_target_ref(
    target_ref: &TargetRef,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, TargetError> {
    match target_ref {
        TargetRef::Var(var_name) => resolve_binding(var_name, bindings),

        TargetRef::Id(id_name) => resolve_binding(id_name, bindings),

        TargetRef::Pattern(_) => Err(TargetError::UnsupportedTarget {
            target_type: "Pattern".to_string(),
        }),
    }
}

/// Resolve a variable-only target (no edge patterns).
///
/// Use this when you don't have access to registry/graph and only support variable targets.
pub fn resolve_var_target(
    target: &Target,
    bindings: &HashMap<String, EntityId>,
) -> Result<EntityId, TargetError> {
    match target {
        Target::Var(var_name) => resolve_binding(var_name, bindings),

        Target::Id(id_name) => resolve_binding(id_name, bindings),

        Target::Pattern(_) => Err(TargetError::UnsupportedTarget {
            target_type: "Pattern".to_string(),
        }),

        Target::EdgePattern { .. } => Err(TargetError::UnsupportedTarget {
            target_type: "EdgePattern".to_string(),
        }),
    }
}

/// Resolve an edge pattern to an edge ID by searching the graph.
fn resolve_edge_pattern(
    edge_type: &str,
    target_vars: &[String],
    bindings: &HashMap<String, EntityId>,
    registry: &Registry,
    graph: &Graph,
) -> Result<EntityId, TargetError> {
    // Resolve target variables to entity IDs
    let mut target_ids = Vec::new();
    for target_var in target_vars {
        let id = bindings
            .get(target_var)
            .copied()
            .ok_or_else(|| TargetError::UnknownVariable {
                name: target_var.clone(),
            })?;
        target_ids.push(id);
    }

    // Find edge type ID
    let edge_type_id = registry
        .get_edge_type_id(edge_type)
        .ok_or_else(|| TargetError::UnknownEdgeType {
            name: edge_type.to_string(),
        })?;

    // Need at least 2 targets
    if target_ids.len() < 2 {
        return Err(TargetError::InsufficientTargets {
            actual: target_ids.len(),
        });
    }

    let source_node_id = target_ids[0].as_node().ok_or_else(|| {
        TargetError::SourceNotNode {
            position: 0,
            actual_type: format!("{:?}", target_ids[0]),
        }
    })?;
    let target_node_id = target_ids[1].as_node().ok_or_else(|| {
        TargetError::TargetNotNode {
            position: 1,
            actual_type: format!("{:?}", target_ids[1]),
        }
    })?;

    // Search for matching edge
    for edge_id in graph.edges_from(source_node_id, None) {
        if let Some(edge) = graph.get_edge(edge_id) {
            if edge.type_id == edge_type_id {
                let targets = &edge.targets;
                if targets.len() >= 2
                    && targets[0].as_node() == Some(source_node_id)
                    && targets[1].as_node() == Some(target_node_id)
                {
                    return Ok(edge_id.into());
                }
            }
        }
    }

    Err(TargetError::EdgeNotFound {
        edge_type: edge_type.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_core::NodeId;

    #[test]
    fn test_resolve_var_target() {
        // GIVEN
        let mut bindings = HashMap::new();
        let node_id = NodeId::new(1);
        bindings.insert("x".to_string(), EntityId::Node(node_id));

        let target = Target::Var("x".to_string());

        // Dummy registry and graph (not used for var target)
        let registry = mew_registry::RegistryBuilder::new().build().unwrap();
        let graph = Graph::new();

        // WHEN
        let result = resolve_target(&target, &bindings, &registry, &graph);

        // THEN
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EntityId::Node(node_id));
    }

    #[test]
    fn test_resolve_unknown_var() {
        // GIVEN
        let bindings = HashMap::new();
        let target = Target::Var("unknown".to_string());
        let registry = mew_registry::RegistryBuilder::new().build().unwrap();
        let graph = Graph::new();

        // WHEN
        let result = resolve_target(&target, &bindings, &registry, &graph);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result, Err(TargetError::UnknownVariable { .. })));
    }

    #[test]
    fn test_resolve_target_ref_var() {
        // GIVEN
        let mut bindings = HashMap::new();
        let node_id = NodeId::new(1);
        bindings.insert("y".to_string(), EntityId::Node(node_id));

        let target_ref = TargetRef::Var("y".to_string());

        // WHEN
        let result = resolve_target_ref(&target_ref, &bindings);

        // THEN
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EntityId::Node(node_id));
    }
}
