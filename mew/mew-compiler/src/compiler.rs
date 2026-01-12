//! Main compiler implementation.

use crate::{CompileError, CompileResult};
use mew_core::{TypeId, Value};
use mew_parser::{
    parse_ontology, AttrModifier, EdgeModifier, EdgeTypeDef as AstEdgeTypeDef,
    NodeTypeDef as AstNodeTypeDef, OntologyDef, TypeAliasDef,
};
use mew_registry::{AttrDef, OnKillAction, Registry, RegistryBuilder};
use std::collections::{HashMap, HashSet};

/// The Compiler transforms ontology source into Registry.
pub struct Compiler {
    /// Collected type names for validation.
    type_names: HashSet<String>,
    /// Collected edge type names for validation.
    edge_type_names: HashSet<String>,
    /// Type alias definitions: name -> base type
    type_aliases: HashMap<String, TypeAliasDef>,
    /// Generated constraints from modifiers (for node types).
    generated_type_constraints: Vec<GeneratedConstraint>,
    /// Generated constraints from modifiers (for edge types).
    generated_edge_constraints: Vec<GeneratedConstraint>,
}

/// A constraint generated from modifier expansion.
#[derive(Debug)]
struct GeneratedConstraint {
    name: String,
    on_type: String,
    condition: String,
}

impl Compiler {
    /// Create a new compiler.
    pub fn new() -> Self {
        Self {
            type_names: HashSet::new(),
            edge_type_names: HashSet::new(),
            type_aliases: HashMap::new(),
            generated_type_constraints: Vec::new(),
            generated_edge_constraints: Vec::new(),
        }
    }

    /// Compile ontology source into a Registry.
    pub fn compile(&mut self, source: &str) -> CompileResult<Registry> {
        // Parse the ontology
        let defs = parse_ontology(source)?;

        // First pass: collect all type names, edge type names, and type aliases
        for def in &defs {
            match def {
                OntologyDef::TypeAlias(alias) => {
                    self.type_aliases.insert(alias.name.clone(), alias.clone());
                }
                OntologyDef::Node(n) => {
                    if !self.type_names.insert(n.name.clone()) {
                        return Err(CompileError::duplicate_type(&n.name, n.span));
                    }
                }
                OntologyDef::Edge(e) => {
                    if !self.edge_type_names.insert(e.name.clone()) {
                        return Err(CompileError::duplicate_edge_type(&e.name, e.span));
                    }
                }
                _ => {}
            }
        }

        // Build the registry
        let mut builder = RegistryBuilder::new();

        // Second pass: add types and edges
        for def in &defs {
            match def {
                OntologyDef::TypeAlias(_) => {
                    // Type aliases are expanded at use site - nothing to register
                    // The modifiers are applied when the alias is used in an attribute
                }
                OntologyDef::Node(n) => {
                    self.add_node_type(&mut builder, n)?;
                }
                OntologyDef::Edge(e) => {
                    self.add_edge_type(&mut builder, e)?;
                }
                OntologyDef::Constraint(c) => {
                    // Extract the primary type from the pattern (first node pattern)
                    let primary_type = self.extract_primary_type(&c.pattern)?;

                    // Validate that the type exists
                    if !self.type_names.contains(&primary_type) {
                        return Err(CompileError::unknown_type(&primary_type, c.span));
                    }
                    // Note: Constraints are stored as strings in the registry
                    // The actual constraint checking is done at runtime
                    builder
                        .add_constraint(&c.name, format!("{:?}", c.condition))
                        .for_type(&primary_type)
                        .done()?;
                }
                OntologyDef::Rule(r) => {
                    // Extract the primary type from the pattern (first node pattern)
                    let primary_type = self.extract_primary_type(&r.pattern)?;

                    // Validate that the type exists
                    if !self.type_names.contains(&primary_type) {
                        return Err(CompileError::unknown_type(&primary_type, r.span));
                    }
                    let mut rule_builder = builder
                        .add_rule(&r.name, format!("{:?}", r.production))
                        .for_type(&primary_type);
                    if r.auto {
                        rule_builder = rule_builder.auto();
                    }
                    if let Some(p) = r.priority {
                        rule_builder = rule_builder.priority(p as i32);
                    }
                    rule_builder.done()?;
                }
            }
        }

        // Add generated constraints from modifier expansion (for node types)
        for gc in &self.generated_type_constraints {
            builder
                .add_constraint(&gc.name, &gc.condition)
                .for_type(&gc.on_type)
                .done()?;
        }

        // Add generated constraints from modifier expansion (for edge types)
        for gc in &self.generated_edge_constraints {
            builder
                .add_constraint(&gc.name, &gc.condition)
                .for_edge_type(&gc.on_type)
                .done()?;
        }

        builder.build().map_err(CompileError::from)
    }

    /// Add a node type to the registry builder.
    fn add_node_type(
        &mut self,
        builder: &mut RegistryBuilder,
        node_def: &AstNodeTypeDef,
    ) -> CompileResult<TypeId> {
        let mut type_builder = builder.add_type(&node_def.name);

        // Add parent types for inheritance
        for parent in &node_def.parents {
            type_builder = type_builder.extends(parent);
        }

        for attr_def in &node_def.attrs {
            // Resolve type aliases to their base types
            let resolved_type = self.resolve_type_name(&attr_def.type_name);
            let mut attr = AttrDef::new(&attr_def.name, &resolved_type);

            // Handle nullable types
            if attr_def.nullable {
                attr = attr.nullable();
            }

            // Handle inline default value (= expr)
            if let Some(default_expr) = &attr_def.default_value {
                if let Some(value) = expr_to_value(default_expr) {
                    attr = attr.with_default(value);
                }
                // For now(), we need special handling - just mark as having default
                // The actual now() value would be computed at runtime
                if is_now_call(default_expr) {
                    // Mark as having a timestamp default (computed at runtime)
                    attr = attr.with_default(Value::Timestamp(0)); // Placeholder
                }
            }

            // Process modifiers and expand to constraints
            for modifier in &attr_def.modifiers {
                match modifier {
                    AttrModifier::Required => {
                        attr = attr.required();
                        // Generate required constraint
                        self.generated_type_constraints.push(GeneratedConstraint {
                            name: format!("_{}_{}_{}", node_def.name, attr_def.name, "required"),
                            on_type: node_def.name.clone(),
                            condition: format!("t.{} IS NOT NULL", attr_def.name),
                        });
                    }
                    AttrModifier::Unique => {
                        attr = attr.unique();
                        // Generate uniqueness constraint
                        self.generated_type_constraints.push(GeneratedConstraint {
                            name: format!("_{}_{}_{}", node_def.name, attr_def.name, "unique"),
                            on_type: node_def.name.clone(),
                            condition: format!(
                                "NOT EXISTS t2: {} WHERE t != t2 AND t.{} = t2.{}",
                                node_def.name, attr_def.name, attr_def.name
                            ),
                        });
                    }
                    AttrModifier::Default(expr) => {
                        // For now, only handle simple literals
                        if let Some(value) = expr_to_value(expr) {
                            attr = attr.with_default(value);
                        }
                    }
                    AttrModifier::Range { min, max } => {
                        let min_val = min.as_ref().and_then(expr_to_value);
                        let max_val = max.as_ref().and_then(expr_to_value);
                        attr = attr.with_range(min_val.clone(), max_val.clone());

                        // Generate range constraint
                        let mut conditions = Vec::new();
                        if let Some(min) = min_val {
                            conditions.push(format!("t.{} >= {:?}", attr_def.name, min));
                        }
                        if let Some(max) = max_val {
                            conditions.push(format!("t.{} <= {:?}", attr_def.name, max));
                        }
                        if !conditions.is_empty() {
                            self.generated_type_constraints.push(GeneratedConstraint {
                                name: format!("_{}_{}_{}", node_def.name, attr_def.name, "range"),
                                on_type: node_def.name.clone(),
                                condition: conditions.join(" AND "),
                            });
                        }
                    }
                    AttrModifier::InValues(values) => {
                        // Generate enum constraint
                        let value_strs: Vec<String> = values
                            .iter()
                            .filter_map(expr_to_value)
                            .map(|v| format!("{:?}", v))
                            .collect();
                        if !value_strs.is_empty() {
                            self.generated_type_constraints.push(GeneratedConstraint {
                                name: format!("_{}_{}_{}", node_def.name, attr_def.name, "enum"),
                                on_type: node_def.name.clone(),
                                condition: format!(
                                    "t.{} IN [{}]",
                                    attr_def.name,
                                    value_strs.join(", ")
                                ),
                            });
                        }
                    }
                    AttrModifier::Match(pattern) => {
                        // Generate regex constraint
                        self.generated_type_constraints.push(GeneratedConstraint {
                            name: format!("_{}_{}_{}", node_def.name, attr_def.name, "match"),
                            on_type: node_def.name.clone(),
                            condition: format!(
                                "t.{} MATCHES \"{}\"",
                                attr_def.name, pattern
                            ),
                        });
                    }
                }
            }

            type_builder = type_builder.attr(attr);
        }

        type_builder.done().map_err(CompileError::from)
    }

    /// Resolve a type name, expanding type aliases to their base types.
    fn resolve_type_name(&self, type_name: &str) -> String {
        if let Some(alias) = self.type_aliases.get(type_name) {
            // Recursively resolve in case of chained aliases
            self.resolve_type_name(&alias.base_type)
        } else {
            type_name.to_string()
        }
    }

    /// Add an edge type to the registry builder.
    fn add_edge_type(
        &mut self,
        builder: &mut RegistryBuilder,
        edge_def: &AstEdgeTypeDef,
    ) -> CompileResult<()> {
        // Validate that parameter types exist
        for (_, param_type) in &edge_def.params {
            if param_type != "any" && !self.type_names.contains(param_type) {
                return Err(CompileError::unknown_type(param_type, edge_def.span));
            }
        }

        let mut edge_builder = builder.add_edge_type(&edge_def.name);

        for (param_name, param_type) in &edge_def.params {
            edge_builder = edge_builder.param(param_name, param_type);
        }

        // Process edge attributes
        for attr_def in &edge_def.attrs {
            // Resolve type aliases to their base types
            let resolved_type = self.resolve_type_name(&attr_def.type_name);
            let mut attr = AttrDef::new(&attr_def.name, &resolved_type);

            // Handle nullable types
            if attr_def.nullable {
                attr = attr.nullable();
            }

            // Handle inline default value
            if let Some(default_expr) = &attr_def.default_value {
                if let Some(value) = expr_to_value(default_expr) {
                    attr = attr.with_default(value);
                }
                if is_now_call(default_expr) {
                    attr = attr.with_default(Value::Timestamp(0)); // Placeholder
                }
            }

            // Process modifiers
            for modifier in &attr_def.modifiers {
                match modifier {
                    AttrModifier::Required => {
                        attr = attr.required();
                    }
                    AttrModifier::Unique => {
                        attr = attr.unique();
                    }
                    AttrModifier::Default(expr) => {
                        if let Some(value) = expr_to_value(expr) {
                            attr = attr.with_default(value);
                        }
                    }
                    AttrModifier::Range { min, max } => {
                        let min_val = min.as_ref().and_then(expr_to_value);
                        let max_val = max.as_ref().and_then(expr_to_value);
                        attr = attr.with_range(min_val, max_val);
                    }
                    AttrModifier::InValues(_) | AttrModifier::Match(_) => {
                        // These are constraint-generating modifiers
                        // For edge attributes, we'd generate edge-level constraints
                        // For now, just skip - runtime constraint checking will handle them
                    }
                }
            }

            edge_builder = edge_builder.attr(attr);
        }

        // Process edge modifiers
        for modifier in &edge_def.modifiers {
            match modifier {
                EdgeModifier::Acyclic => {
                    edge_builder = edge_builder.acyclic();
                    // Generate acyclic constraint (for edge types)
                    self.generated_edge_constraints.push(GeneratedConstraint {
                        name: format!("_{}_acyclic", edge_def.name),
                        on_type: edge_def.name.clone(),
                        condition: format!("NOT path(a, b) via {}", edge_def.name),
                    });
                }
                EdgeModifier::Unique => {
                    edge_builder = edge_builder.unique_edge();
                }
                EdgeModifier::NoSelf => {
                    // Generate no-self constraint
                    self.generated_edge_constraints.push(GeneratedConstraint {
                        name: format!("_{}_no_self", edge_def.name),
                        on_type: edge_def.name.clone(),
                        condition: "source != target".to_string(),
                    });
                }
                EdgeModifier::Symmetric => {
                    edge_builder = edge_builder.symmetric();
                }
                EdgeModifier::Indexed => {
                    // Indexed modifier is an index hint, not yet implemented in registry
                }
                EdgeModifier::OnKillSource(action) | EdgeModifier::OnKillTarget(action) => {
                    let registry_action = match action {
                        mew_parser::ReferentialAction::Cascade => OnKillAction::Cascade,
                        mew_parser::ReferentialAction::Unlink => OnKillAction::SetNull, // Use SetNull as Unlink equivalent
                        mew_parser::ReferentialAction::Prevent => OnKillAction::Restrict,
                    };
                    edge_builder = edge_builder.on_kill(registry_action);
                }
                EdgeModifier::Cardinality { param, min, max } => {
                    // Cardinality constraints generate runtime constraints
                    // For now, just record them as generated constraints
                    let max_str = match max {
                        mew_parser::CardinalityMax::Value(v) => v.to_string(),
                        mew_parser::CardinalityMax::Unbounded => "*".to_string(),
                    };
                    self.generated_edge_constraints.push(GeneratedConstraint {
                        name: format!("_{}_{}_cardinality", edge_def.name, param),
                        on_type: edge_def.name.clone(),
                        condition: format!("cardinality({}, {}..{})", param, min, max_str),
                    });
                }
            }
        }

        edge_builder.done()?;
        Ok(())
    }

    /// Extract the primary type from a pattern (first node pattern).
    /// For constraints like `e: Event, causes(e, _) => ...`, returns "Event".
    fn extract_primary_type(&self, pattern: &mew_parser::Pattern) -> CompileResult<String> {
        for element in &pattern.elements {
            if let mew_parser::PatternElem::Node(node) = element {
                return Ok(node.type_name.clone());
            }
        }

        // No node pattern found - check if there's an edge pattern we can extract from
        for element in &pattern.elements {
            if let mew_parser::PatternElem::Edge(edge) = element {
                // For edge-only patterns, return a special type marker
                // In practice, this case needs more sophisticated handling
                return Ok(format!("edge:{}", edge.edge_type));
            }
        }

        // Fallback - empty pattern
        Err(CompileError::validation(
            "Constraint/rule pattern has no node types",
            pattern.span,
        ))
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an expression to a Value (only handles simple literals).
fn expr_to_value(expr: &mew_parser::Expr) -> Option<Value> {
    match expr {
        mew_parser::Expr::Literal(lit) => match &lit.kind {
            mew_parser::LiteralKind::Null => Some(Value::Null),
            mew_parser::LiteralKind::Bool(b) => Some(Value::Bool(*b)),
            mew_parser::LiteralKind::Int(i) => Some(Value::Int(*i)),
            mew_parser::LiteralKind::Float(f) => Some(Value::Float(*f)),
            mew_parser::LiteralKind::String(s) => Some(Value::String(s.clone())),
        },
        _ => None,
    }
}

/// Check if an expression is a `now()` function call.
fn is_now_call(expr: &mew_parser::Expr) -> bool {
    matches!(expr, mew_parser::Expr::FnCall(fc) if fc.name.eq_ignore_ascii_case("now") && fc.args.is_empty())
}

/// Compile ontology source into a Registry.
pub fn compile(source: &str) -> CompileResult<Registry> {
    Compiler::new().compile(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_node_type() {
        // GIVEN
        let source = r#"
            node Task {
                title: String
            }
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let task_type = registry.get_type_by_name("Task");
        assert!(task_type.is_some());
        assert_eq!(task_type.unwrap().name, "Task");
    }

    #[test]
    fn test_compile_node_with_required_modifier() {
        // GIVEN
        let source = r#"
            node Task {
                title: String [required]
            }
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let task_type = registry.get_type_by_name("Task").unwrap();
        let attr = task_type.get_attr("title").unwrap();
        assert!(attr.required);
    }

    #[test]
    fn test_compile_node_with_unique_modifier() {
        // GIVEN
        let source = r#"
            node Task {
                code: String [unique]
            }
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let task_type = registry.get_type_by_name("Task").unwrap();
        let attr = task_type.get_attr("code").unwrap();
        assert!(attr.unique);
    }

    #[test]
    fn test_compile_simple_edge_type() {
        // GIVEN
        let source = r#"
            node Person { name: String }
            node Task { title: String }
            edge owns(owner: Person, task: Task)
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let edge_type = registry.get_edge_type_by_name("owns");
        assert!(edge_type.is_some());
        assert_eq!(edge_type.unwrap().name, "owns");
        assert_eq!(edge_type.unwrap().arity(), 2);
    }

    #[test]
    fn test_compile_duplicate_type_error() {
        // GIVEN
        let source = r#"
            node Task { title: String }
            node Task { name: String }
        "#;

        // WHEN
        let result = compile(source);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CompileError::DuplicateType { .. }));
    }

    #[test]
    fn test_compile_unknown_type_in_edge() {
        // GIVEN
        let source = r#"
            node Person { name: String }
            edge owns(owner: Person, task: Unknown)
        "#;

        // WHEN
        let result = compile(source);

        // THEN
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CompileError::UnknownType { .. }));
    }

    #[test]
    fn test_compile_edge_with_acyclic_modifier() {
        // GIVEN
        let source = r#"
            node Task { title: String }
            edge depends_on(a: Task, b: Task) [acyclic]
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let edge_type = registry.get_edge_type_by_name("depends_on").unwrap();
        assert!(edge_type.acyclic);
    }

    #[test]
    fn test_compile_edge_with_cascade_modifier() {
        // GIVEN
        let source = r#"
            node Person { name: String }
            node Item { name: String }
            edge owns(owner: Person, item: Item) [on_kill_target: cascade]
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let edge_type = registry.get_edge_type_by_name("owns").unwrap();
        assert_eq!(edge_type.on_kill.len(), 1);
        assert_eq!(edge_type.on_kill[0], OnKillAction::Cascade);
    }

    #[test]
    fn test_compile_constraint() {
        // GIVEN
        let source = r#"
            node Task {
                priority: Int
            }
            constraint priority_positive: t: Task => t.priority >= 0
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let task_id = registry.get_type_id("Task").unwrap();
        let constraints = registry.get_constraints_for_type(task_id);
        assert!(!constraints.is_empty());
    }

    #[test]
    fn test_compile_rule() {
        // GIVEN
        let source = r#"
            node Task {
                status: String
            }
            rule auto_complete [auto, priority: 10]: t: Task => SET t.status = "done"
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        let task_id = registry.get_type_id("Task").unwrap();
        let rules = registry.get_rules_for_type(task_id);
        assert!(!rules.is_empty());
        assert!(rules[0].auto);
        assert_eq!(rules[0].priority, 10);
    }

    #[test]
    fn test_compile_registry_building() {
        // GIVEN
        let source = r#"
            node Task { title: String }
            edge owns(p: any, t: Task)
        "#;

        // WHEN
        let registry = compile(source).unwrap();

        // THEN
        assert!(registry.get_type_by_name("Task").is_some());
        assert!(registry.get_edge_type_by_name("owns").is_some());
    }
}
