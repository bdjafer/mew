//! Schema analysis - extracts structure from ontology source

use crate::types::*;
use std::collections::HashMap;

/// Analyzes an ontology source to extract schema information
pub struct SchemaAnalyzer;

impl SchemaAnalyzer {
    /// Analyze ontology source and extract schema information
    ///
    /// This is independent of MEW's parser - we do our own parsing
    /// to maintain oracle independence.
    pub fn analyze(source: &str) -> Result<AnalyzedSchema, TestGenError> {
        let mut schema = AnalyzedSchema {
            node_types: HashMap::new(),
            edge_types: HashMap::new(),
            constraints: Vec::new(),
            rules: Vec::new(),
        };

        // Simple line-by-line parsing for independence from MEW parser
        let mut in_block = false;
        let mut current_block: Option<BlockType> = None;
        let mut current_name = String::new();
        let mut current_attrs = Vec::new();
        let mut current_parents = Vec::new();
        let mut current_params = Vec::new();
        let mut edge_modifiers = EdgeModifiers::default();

        for line in source.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("--") {
                continue;
            }

            // Handle ontology wrapper
            if line.starts_with("ontology ") && line.ends_with('{') {
                continue;
            }

            // Detect block start
            if line.starts_with("node ") {
                let rest = &line[5..];
                let (name, parents) = Self::parse_node_header(rest);
                current_name = name;
                current_parents = parents;
                current_block = Some(BlockType::Node);
                current_attrs.clear();
                in_block = line.contains('{') && !line.contains('}');

                // Single-line definition
                if line.contains('}') {
                    schema.node_types.insert(current_name.clone(), NodeTypeInfo {
                        name: current_name.clone(),
                        attrs: current_attrs.clone(),
                        parents: current_parents.clone(),
                        applicable_constraints: Vec::new(),
                    });
                    current_block = None;
                }
                continue;
            }

            if line.starts_with("edge ") {
                let rest = &line[5..];
                let (name, params) = Self::parse_edge_header(rest);
                current_name = name;
                current_params = params;
                current_block = Some(BlockType::Edge);
                current_attrs.clear();
                edge_modifiers = EdgeModifiers::default();
                in_block = line.contains('{');

                // Parse edge modifiers from header
                if let Some(bracket_start) = line.find('[') {
                    if let Some(bracket_end) = line.find(']') {
                        let mods = &line[bracket_start + 1..bracket_end];
                        edge_modifiers = Self::parse_edge_modifiers(mods);
                    }
                }

                // Edge without body
                if !line.contains('{') || line.trim_end().ends_with("{}") {
                    schema.edge_types.insert(current_name.clone(), EdgeTypeInfo {
                        name: current_name.clone(),
                        params: current_params.clone(),
                        attrs: Vec::new(),
                        acyclic: edge_modifiers.acyclic,
                        unique: edge_modifiers.unique,
                        symmetric: edge_modifiers.symmetric,
                        no_self: edge_modifiers.no_self,
                    });
                    current_block = None;
                    in_block = false;
                }
                continue;
            }

            if line.starts_with("constraint ") {
                let rest = &line[11..];
                if let Some((name, on_type)) = Self::parse_constraint_header(rest) {
                    schema.constraints.push(ConstraintInfo {
                        name,
                        on_type,
                        description: String::new(),
                    });
                }
                continue;
            }

            if line.starts_with("rule ") {
                let rest = &line[5..];
                if let Some((name, on_type, auto)) = Self::parse_rule_header(rest) {
                    schema.rules.push(RuleInfo {
                        name,
                        on_type,
                        auto,
                    });
                }
                continue;
            }

            // Handle block end
            if line == "}" {
                if let Some(ref block_type) = current_block {
                    match block_type {
                        BlockType::Node => {
                            schema.node_types.insert(current_name.clone(), NodeTypeInfo {
                                name: current_name.clone(),
                                attrs: current_attrs.clone(),
                                parents: current_parents.clone(),
                                applicable_constraints: Vec::new(),
                            });
                        }
                        BlockType::Edge => {
                            schema.edge_types.insert(current_name.clone(), EdgeTypeInfo {
                                name: current_name.clone(),
                                params: current_params.clone(),
                                attrs: current_attrs.clone(),
                                acyclic: edge_modifiers.acyclic,
                                unique: edge_modifiers.unique,
                                symmetric: edge_modifiers.symmetric,
                                no_self: edge_modifiers.no_self,
                            });
                        }
                    }
                }
                current_block = None;
                in_block = false;
                continue;
            }

            // Parse attribute in block
            if in_block && line.contains(':') {
                if let Some(attr) = Self::parse_attr(line) {
                    current_attrs.push(attr);
                }
            }
        }

        // Link constraints to types
        for constraint in &schema.constraints {
            if let Some(node_type) = schema.node_types.get_mut(&constraint.on_type) {
                node_type.applicable_constraints.push(constraint.name.clone());
            }
        }

        if schema.node_types.is_empty() && schema.edge_types.is_empty() {
            return Err(TestGenError::SchemaError(
                "No types found in ontology".to_string()
            ));
        }

        Ok(schema)
    }

    fn parse_node_header(rest: &str) -> (String, Vec<String>) {
        let rest = rest.trim();
        let mut parents = Vec::new();

        // Find name (before : or {)
        let name_end = rest.find(|c| c == ':' || c == '{' || c == ' ').unwrap_or(rest.len());
        let name = rest[..name_end].trim().to_string();

        // Check for inheritance
        if let Some(colon_pos) = rest.find(':') {
            // Make sure it's inheritance, not attr definition
            let before_colon = &rest[..colon_pos];
            if !before_colon.contains('{') {
                let after_colon = &rest[colon_pos + 1..];
                let parent_end = after_colon.find('{').unwrap_or(after_colon.len());
                let parent = after_colon[..parent_end].trim();
                if !parent.is_empty() && !parent.contains(':') {
                    parents.push(parent.to_string());
                }
            }
        }

        (name, parents)
    }

    fn parse_edge_header(rest: &str) -> (String, Vec<(String, String)>) {
        let rest = rest.trim();

        // Find name (before ()
        let name_end = rest.find('(').unwrap_or(rest.len());
        let name = rest[..name_end].trim().to_string();

        let mut params = Vec::new();

        // Extract parameters
        if let Some(paren_start) = rest.find('(') {
            if let Some(paren_end) = rest.find(')') {
                let param_str = &rest[paren_start + 1..paren_end];
                for param in param_str.split(',') {
                    let param = param.trim();
                    if let Some(colon_pos) = param.find(':') {
                        let param_name = param[..colon_pos].trim().to_string();
                        let param_type = param[colon_pos + 1..].trim().to_string();
                        params.push((param_name, param_type));
                    }
                }
            }
        }

        (name, params)
    }

    fn parse_edge_modifiers(mods: &str) -> EdgeModifiers {
        let mut result = EdgeModifiers::default();
        for m in mods.split(',') {
            let m = m.trim().to_lowercase();
            match m.as_str() {
                "acyclic" => result.acyclic = true,
                "unique" => result.unique = true,
                "symmetric" => result.symmetric = true,
                "no_self" => result.no_self = true,
                _ => {}
            }
        }
        result
    }

    fn parse_constraint_header(rest: &str) -> Option<(String, String)> {
        let rest = rest.trim();
        // constraint name ON Type { ... }
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("on") {
            let name = parts[0].to_string();
            let on_type = parts[2].trim_end_matches('{').trim().to_string();
            return Some((name, on_type));
        }
        None
    }

    fn parse_rule_header(rest: &str) -> Option<(String, String, bool)> {
        let rest = rest.trim();
        // rule name ON Type [auto] { ... }
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("on") {
            let name = parts[0].to_string();
            let on_type = parts[2].trim_end_matches('{').trim().to_string();
            let auto = rest.contains("[auto]");
            return Some((name, on_type, auto));
        }
        None
    }

    fn parse_attr(line: &str) -> Option<AttrInfo> {
        let line = line.trim().trim_end_matches(',');

        // name: Type? = default [modifiers]
        let colon_pos = line.find(':')?;
        let name = line[..colon_pos].trim().to_string();

        let after_colon = &line[colon_pos + 1..];

        // Find type name
        let mut type_end = after_colon.len();
        for (i, c) in after_colon.char_indices() {
            if c == '?' || c == '=' || c == '[' || c == ',' {
                type_end = i;
                break;
            }
        }
        let type_name = after_colon[..type_end].trim().to_string();

        // Check nullable
        let nullable = after_colon.contains('?');

        // Parse modifiers
        let mut required = false;
        let mut unique = false;
        let mut default = None;
        let mut min = None;
        let mut max = None;
        let mut allowed_values = None;
        let mut pattern = None;

        // Find inline default
        if let Some(eq_pos) = after_colon.find('=') {
            let after_eq = &after_colon[eq_pos + 1..];
            let val_end = after_eq.find('[').unwrap_or(after_eq.len());
            let val_str = after_eq[..val_end].trim();
            default = Self::parse_value(val_str);
        }

        // Parse bracket modifiers
        if let Some(bracket_start) = after_colon.find('[') {
            if let Some(bracket_end) = after_colon.rfind(']') {
                let mods = &after_colon[bracket_start + 1..bracket_end];
                for modifier in Self::split_modifiers(mods) {
                    let modifier = modifier.trim();
                    if modifier == "required" {
                        required = true;
                    } else if modifier == "unique" {
                        unique = true;
                    } else if modifier.starts_with("default") {
                        if let Some(eq) = modifier.find('=') {
                            default = Self::parse_value(modifier[eq + 1..].trim());
                        }
                    } else if modifier.starts_with(">=") {
                        min = Self::parse_value(modifier[2..].trim());
                    } else if modifier.starts_with("<=") {
                        max = Self::parse_value(modifier[2..].trim());
                    } else if modifier.starts_with("in:") {
                        let values_str = modifier[3..].trim();
                        if values_str.starts_with('[') && values_str.ends_with(']') {
                            let inner = &values_str[1..values_str.len() - 1];
                            let values: Vec<Value> = inner.split(',')
                                .filter_map(|v| Self::parse_value(v.trim()))
                                .collect();
                            if !values.is_empty() {
                                allowed_values = Some(values);
                            }
                        }
                    } else if modifier.starts_with("match:") {
                        pattern = Some(modifier[6..].trim().trim_matches('"').to_string());
                    } else if modifier.contains("..") {
                        // Range shorthand: 0..10
                        let parts: Vec<&str> = modifier.split("..").collect();
                        if parts.len() == 2 {
                            min = Self::parse_value(parts[0].trim());
                            max = Self::parse_value(parts[1].trim());
                        }
                    }
                }
            }
        }

        Some(AttrInfo {
            name,
            type_name,
            nullable,
            required,
            unique,
            default,
            min,
            max,
            allowed_values,
            pattern,
        })
    }

    fn split_modifiers(mods: &str) -> Vec<&str> {
        // Smart split that handles nested brackets
        let mut result = Vec::new();
        let mut depth = 0;
        let mut start = 0;

        for (i, c) in mods.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => {
                    result.push(&mods[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        if start < mods.len() {
            result.push(&mods[start..]);
        }
        result
    }

    fn parse_value(s: &str) -> Option<Value> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if s == "null" {
            return Some(Value::Null);
        }
        if s == "true" {
            return Some(Value::Bool(true));
        }
        if s == "false" {
            return Some(Value::Bool(false));
        }
        // Function call like now()
        if s.ends_with("()") && !s.starts_with('"') {
            let func_name = &s[..s.len() - 2];
            return Some(Value::FunctionCall(func_name.to_string()));
        }
        if let Ok(i) = s.parse::<i64>() {
            return Some(Value::Int(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Some(Value::Float(f));
        }
        // String literal
        let s = s.trim_matches('"');
        Some(Value::String(s.to_string()))
    }
}

enum BlockType {
    Node,
    Edge,
}

#[derive(Default)]
struct EdgeModifiers {
    acyclic: bool,
    unique: bool,
    symmetric: bool,
    no_self: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_simple_ontology() {
        let source = r#"
            node Person {
                name: String [required],
                age: Int?
            }
            edge knows(a: Person, b: Person)
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();

        assert!(schema.node_types.contains_key("Person"));
        let person = &schema.node_types["Person"];
        assert_eq!(person.attrs.len(), 2);
        assert_eq!(person.attrs[0].name, "name");
        assert!(person.attrs[0].required);
        assert_eq!(person.attrs[1].name, "age");
        assert!(person.attrs[1].nullable);

        assert!(schema.edge_types.contains_key("knows"));
        let knows = &schema.edge_types["knows"];
        assert_eq!(knows.params.len(), 2);
    }

    #[test]
    fn test_analyze_with_wrapper() {
        let source = r#"
            ontology Test {
                node Item {
                    name: String
                }
            }
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        assert!(schema.node_types.contains_key("Item"));
    }

    #[test]
    fn test_analyze_inheritance() {
        let source = r#"
            node Animal { name: String }
            node Dog : Animal { breed: String }
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let dog = &schema.node_types["Dog"];
        assert_eq!(dog.parents, vec!["Animal"]);
    }

    #[test]
    fn test_analyze_edge_modifiers() {
        let source = r#"
            node A {}
            edge rel(x: A, y: A) [acyclic, no_self]
        "#;

        let schema = SchemaAnalyzer::analyze(source).unwrap();
        let rel = &schema.edge_types["rel"];
        assert!(rel.acyclic);
        assert!(rel.no_self);
        assert!(!rel.symmetric);
    }
}
