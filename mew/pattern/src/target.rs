//! Target resolution utilities.
//!
//! Shared logic for resolving targets (variables, edge patterns) to entity IDs.
//! Used by both session and REPL to avoid code duplication.

use std::collections::HashMap;

use mew_core::EntityId;
use mew_graph::Graph;
use mew_parser::{Target, TargetRef};
use mew_registry::Registry;

/// Error returned when target resolution fails.
#[derive(Debug, Clone)]
pub enum TargetError {
    /// Variable not found in bindings.
    UnknownVariable(String),
    /// Target type not supported (Id, Pattern).
    UnsupportedTarget,
    /// Edge type not found in registry.
    UnknownEdgeType(String),
    /// Edge pattern requires at least 2 targets.
    InsufficientTargets,
    /// Source in edge pattern must be a node.
    SourceNotNode,
    /// Target in edge pattern must be a node.
    TargetNotNode,
    /// No edge found matching the pattern.
    EdgeNotFound(String),
}

impl std::fmt::Display for TargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetError::UnknownVariable(name) => write!(f, "Unknown variable: {}", name),
            TargetError::UnsupportedTarget => {
                write!(f, "Only variable targets are supported")
            }
            TargetError::UnknownEdgeType(name) => write!(f, "Unknown edge type: {}", name),
            TargetError::InsufficientTargets => {
                write!(f, "Edge pattern requires at least 2 targets")
            }
            TargetError::SourceNotNode => write!(f, "Source must be a node"),
            TargetError::TargetNotNode => write!(f, "Target must be a node"),
            TargetError::EdgeNotFound(edge_type) => {
                write!(
                    f,
                    "No edge of type '{}' found between specified nodes",
                    edge_type
                )
            }
        }
    }
}

impl std::error::Error for TargetError {}

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
        Target::Var(var_name) => bindings
            .get(var_name)
            .copied()
            .ok_or_else(|| TargetError::UnknownVariable(var_name.clone())),

        Target::Id(_) | Target::Pattern(_) => Err(TargetError::UnsupportedTarget),

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
        TargetRef::Var(var_name) => bindings
            .get(var_name)
            .copied()
            .ok_or_else(|| TargetError::UnknownVariable(var_name.clone())),

        TargetRef::Id(_) | TargetRef::Pattern(_) => Err(TargetError::UnsupportedTarget),
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
        Target::Var(var_name) => bindings
            .get(var_name)
            .copied()
            .ok_or_else(|| TargetError::UnknownVariable(var_name.clone())),

        Target::Id(_) | Target::Pattern(_) | Target::EdgePattern { .. } => {
            Err(TargetError::UnsupportedTarget)
        }
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
            .ok_or_else(|| TargetError::UnknownVariable(target_var.clone()))?;
        target_ids.push(id);
    }

    // Find edge type ID
    let edge_type_id = registry
        .get_edge_type_id(edge_type)
        .ok_or_else(|| TargetError::UnknownEdgeType(edge_type.to_string()))?;

    // Need at least 2 targets
    if target_ids.len() < 2 {
        return Err(TargetError::InsufficientTargets);
    }

    let source_node_id = target_ids[0]
        .as_node()
        .ok_or(TargetError::SourceNotNode)?;
    let target_node_id = target_ids[1]
        .as_node()
        .ok_or(TargetError::TargetNotNode)?;

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

    Err(TargetError::EdgeNotFound(edge_type.to_string()))
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
        assert!(matches!(result, Err(TargetError::UnknownVariable(_))));
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
