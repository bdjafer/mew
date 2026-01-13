//! Loader for operations files.
//!
//! Parses `.mew` files with step markers (`--# step_name`).

use std::collections::HashMap;
use std::path::Path;

use crate::error::{ExampleError, ExampleResult};

/// A parsed operations file.
#[derive(Debug, Clone)]
pub struct Operations {
    /// The raw file content.
    pub source: String,
    /// Map of step name to MEW statement(s).
    pub steps: HashMap<String, String>,
    /// Steps in order of appearance.
    pub step_order: Vec<String>,
    /// Parameters per step (step name -> (param name -> value))
    pub step_params: HashMap<String, HashMap<String, String>>,
}

impl Operations {
    /// Parse an operations file from a string.
    pub fn parse(source: &str) -> ExampleResult<Self> {
        let mut steps = HashMap::new();
        let mut step_order = Vec::new();
        let mut step_params: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut current_step: Option<String> = None;
        let mut current_content = String::new();
        let mut current_params: HashMap<String, String> = HashMap::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // Check for step marker
            if let Some(suffix) = trimmed.strip_prefix("--#") {
                // Save previous step if any
                if let Some(ref step_name) = current_step {
                    let content = current_content.trim().to_string();
                    if !content.is_empty() {
                        steps.insert(step_name.clone(), content);
                    }
                    if !current_params.is_empty() {
                        step_params.insert(step_name.clone(), current_params.clone());
                    }
                }

                // Start new step
                let step_name = suffix.trim().to_string();
                if step_name.is_empty() {
                    return Err(ExampleError::operations_parse(
                        "<inline>",
                        "empty step name after --#",
                    ));
                }
                step_order.push(step_name.clone());
                current_step = Some(step_name);
                current_content = String::new();
                current_params = HashMap::new();
            } else if current_step.is_some() {
                // Check for parameter definition: -- @param $name = value
                if let Some(param_str) = trimmed.strip_prefix("-- @param ") {
                    if let Some((name, value)) = parse_param_def(param_str) {
                        current_params.insert(name, value);
                    }
                } else if !trimmed.starts_with("-- @") {
                    // Accumulate content for current step (skip other metadata comments)
                    current_content.push_str(line);
                    current_content.push('\n');
                }
            }
            // Lines before the first step marker are ignored (file-level comments)
        }

        // Save last step
        if let Some(ref step_name) = current_step {
            let content = current_content.trim().to_string();
            if !content.is_empty() {
                steps.insert(step_name.clone(), content);
            }
            if !current_params.is_empty() {
                step_params.insert(step_name.clone(), current_params);
            }
        }

        Ok(Self {
            source: source.to_string(),
            steps,
            step_order,
            step_params,
        })
    }

    /// Load and parse an operations file from disk.
    pub fn load(path: &Path) -> ExampleResult<Self> {
        let source = std::fs::read_to_string(path).map_err(|e| ExampleError::file_read(path, e))?;
        Self::parse(&source).map_err(|e| ExampleError::operations_parse(path, e.to_string()))
    }

    /// Get the MEW statement for a step with parameters substituted.
    pub fn get_step(&self, name: &str) -> Option<String> {
        self.steps.get(name).map(|statement| {
            // Apply parameter substitutions if any
            if let Some(params) = self.step_params.get(name) {
                let mut result = statement.clone();
                for (param_name, param_value) in params {
                    // Replace $param_name with the value
                    let placeholder = format!("${}", param_name);
                    result = result.replace(&placeholder, param_value);
                }
                result
            } else {
                statement.clone()
            }
        })
    }

    /// Get the raw MEW statement for a step (without parameter substitution).
    pub fn get_step_raw(&self, name: &str) -> Option<&str> {
        self.steps.get(name).map(|s| s.as_str())
    }

    /// Get all step names in order.
    pub fn step_names(&self) -> &[String] {
        &self.step_order
    }
}

/// Parse a parameter definition from `$name = value` format.
/// Returns (name_without_dollar, value_as_mew_literal) or None if invalid.
fn parse_param_def(s: &str) -> Option<(String, String)> {
    // Format: $name = value
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim().trim_start_matches('$').to_string();
    let value = parts[1].trim().to_string();

    if name.is_empty() {
        return None;
    }

    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_operations() {
        let source = r#"
-- This is a test operations file

--# spawn_bookmark
SPAWN b: Bookmark {
  url: "https://example.com",
  title: "Example"
}

--# query_count
MATCH b: Bookmark RETURN COUNT(*)

--# query_all
MATCH b: Bookmark RETURN b.title, b.url
"#;

        let ops = Operations::parse(source).unwrap();

        assert_eq!(
            ops.step_order,
            vec!["spawn_bookmark", "query_count", "query_all"]
        );
        assert!(ops.get_step("spawn_bookmark").unwrap().contains("SPAWN"));
        assert!(ops.get_step("query_count").unwrap().contains("COUNT"));
        assert!(ops.get_step("query_all").unwrap().contains("RETURN"));
    }

    #[test]
    fn test_parse_with_metadata() {
        let source = r#"
-- @seed: minimal
-- @description: Test scenario

--# step1
-- @expect: created 1
SPAWN x: Thing { name: "test" }
"#;

        let ops = Operations::parse(source).unwrap();

        assert_eq!(ops.step_order, vec!["step1"]);
        // Metadata comments should be stripped
        let step = ops.get_step("step1").unwrap();
        assert!(!step.contains("@expect"));
        assert!(step.contains("SPAWN"));
    }
}
