//! Compiled pattern representation.

use mew_core::{EdgeTypeId, TypeId};
use mew_parser::{EdgePattern, Expr, NodePattern, PatternElem};
use mew_registry::Registry;

use crate::{PatternError, PatternResult};

/// A compiled pattern operation.
#[derive(Debug, Clone)]
pub enum PatternOp {
    /// Scan nodes of a type and bind to variable.
    ScanNodes {
        var: String,
        type_id: TypeId,
    },
    /// Follow an edge from bound variables.
    FollowEdge {
        edge_type_id: EdgeTypeId,
        from_vars: Vec<String>,
        edge_var: Option<String>,
    },
    /// Check an edge exists between bound variables.
    CheckEdge {
        edge_type_id: EdgeTypeId,
        target_vars: Vec<String>,
    },
    /// Filter by expression.
    Filter {
        condition: Expr,
    },
    /// Check NOT EXISTS subpattern.
    NotExists {
        subpattern: CompiledPattern,
    },
}

/// A compiled pattern ready for matching.
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Operations to execute in order.
    pub ops: Vec<PatternOp>,
    /// Variables that must be bound by this pattern.
    pub output_vars: Vec<String>,
}

impl CompiledPattern {
    /// Create a new empty pattern.
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            output_vars: Vec::new(),
        }
    }

    /// Compile a pattern from AST elements.
    pub fn compile(elements: &[PatternElem], registry: &Registry) -> PatternResult<Self> {
        let mut ops = Vec::new();
        let mut bound_vars = Vec::new();

        for elem in elements {
            match elem {
                PatternElem::Node(node) => {
                    let op = compile_node_pattern(node, registry)?;
                    ops.push(op);
                    bound_vars.push(node.var.clone());
                }
                PatternElem::Edge(edge) => {
                    let op = compile_edge_pattern(edge, registry, &bound_vars)?;
                    ops.push(op);
                    if let Some(alias) = &edge.alias {
                        bound_vars.push(alias.clone());
                    }
                }
            }
        }

        Ok(Self {
            ops,
            output_vars: bound_vars,
        })
    }

    /// Add a filter operation.
    pub fn with_filter(mut self, condition: Expr) -> Self {
        self.ops.push(PatternOp::Filter { condition });
        self
    }

    /// Get the output variables.
    pub fn output_vars(&self) -> &[String] {
        &self.output_vars
    }

    /// Check if a variable will be bound by this pattern.
    pub fn binds(&self, var: &str) -> bool {
        self.output_vars.contains(&var.to_string())
    }
}

impl Default for CompiledPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile a node pattern element.
fn compile_node_pattern(node: &NodePattern, registry: &Registry) -> PatternResult<PatternOp> {
    let type_id = registry
        .get_type_id(&node.type_name)
        .ok_or_else(|| PatternError::unknown_type(&node.type_name))?;

    Ok(PatternOp::ScanNodes {
        var: node.var.clone(),
        type_id,
    })
}

/// Compile an edge pattern element.
fn compile_edge_pattern(
    edge: &EdgePattern,
    registry: &Registry,
    bound_vars: &[String],
) -> PatternResult<PatternOp> {
    let edge_type_id = registry
        .get_edge_type_id(&edge.edge_type)
        .ok_or_else(|| PatternError::unknown_edge_type(&edge.edge_type))?;

    // Check that all target variables are bound
    for target in &edge.targets {
        if !bound_vars.contains(target) {
            return Err(PatternError::unbound_variable(target));
        }
    }

    // If the first target is bound, this is a follow operation
    // Otherwise it's a check operation
    if bound_vars.contains(&edge.targets[0]) {
        Ok(PatternOp::FollowEdge {
            edge_type_id,
            from_vars: edge.targets.clone(),
            edge_var: edge.alias.clone(),
        })
    } else {
        Ok(PatternOp::CheckEdge {
            edge_type_id,
            target_vars: edge.targets.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_registry::{AttrDef, RegistryBuilder};

    fn test_registry() -> Registry {
        let mut builder = RegistryBuilder::new();
        builder
            .add_type("Task")
            .attr(AttrDef::new("title", "String"))
            .done()
            .unwrap();
        builder
            .add_type("Person")
            .attr(AttrDef::new("name", "String"))
            .done()
            .unwrap();
        builder
            .add_edge_type("owns")
            .param("owner", "Person")
            .param("task", "Task")
            .done()
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_compile_single_node_pattern() {
        // GIVEN
        let registry = test_registry();
        let elements = vec![PatternElem::Node(NodePattern {
            var: "t".to_string(),
            type_name: "Task".to_string(),
            span: Default::default(),
        })];

        // WHEN
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // THEN
        assert_eq!(pattern.ops.len(), 1);
        assert!(matches!(&pattern.ops[0], PatternOp::ScanNodes { var, .. } if var == "t"));
        assert!(pattern.binds("t"));
    }

    #[test]
    fn test_compile_multi_node_with_edge() {
        // GIVEN
        let registry = test_registry();
        let elements = vec![
            PatternElem::Node(NodePattern {
                var: "p".to_string(),
                type_name: "Person".to_string(),
                span: Default::default(),
            }),
            PatternElem::Node(NodePattern {
                var: "t".to_string(),
                type_name: "Task".to_string(),
                span: Default::default(),
            }),
            PatternElem::Edge(EdgePattern {
                edge_type: "owns".to_string(),
                targets: vec!["p".to_string(), "t".to_string()],
                alias: None,
                transitive: None,
                span: Default::default(),
            }),
        ];

        // WHEN
        let pattern = CompiledPattern::compile(&elements, &registry).unwrap();

        // THEN
        assert_eq!(pattern.ops.len(), 3);
        assert!(matches!(&pattern.ops[0], PatternOp::ScanNodes { var, .. } if var == "p"));
        assert!(matches!(&pattern.ops[1], PatternOp::ScanNodes { var, .. } if var == "t"));
        assert!(matches!(&pattern.ops[2], PatternOp::FollowEdge { .. }));
    }

    #[test]
    fn test_compile_unknown_type_error() {
        // GIVEN
        let registry = test_registry();
        let elements = vec![PatternElem::Node(NodePattern {
            var: "x".to_string(),
            type_name: "Unknown".to_string(),
            span: Default::default(),
        })];

        // WHEN
        let result = CompiledPattern::compile(&elements, &registry);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PatternError::UnknownType { .. }));
    }
}
