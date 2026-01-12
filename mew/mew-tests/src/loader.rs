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
}

impl Operations {
    /// Parse an operations file from a string.
    pub fn parse(source: &str) -> ExampleResult<Self> {
        let mut steps = HashMap::new();
        let mut step_order = Vec::new();
        let mut current_step: Option<String> = None;
        let mut current_content = String::new();

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
            } else if current_step.is_some() {
                // Accumulate content for current step
                // Skip metadata comments (-- @...)
                if !trimmed.starts_with("-- @") {
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
        }

        Ok(Self {
            source: source.to_string(),
            steps,
            step_order,
        })
    }

    /// Load and parse an operations file from disk.
    pub fn load(path: &Path) -> ExampleResult<Self> {
        let source = std::fs::read_to_string(path).map_err(|e| ExampleError::file_read(path, e))?;
        Self::parse(&source).map_err(|e| ExampleError::operations_parse(path, e.to_string()))
    }

    /// Get the MEW statement for a step.
    pub fn get_step(&self, name: &str) -> Option<&str> {
        self.steps.get(name).map(|s| s.as_str())
    }

    /// Get all step names in order.
    pub fn step_names(&self) -> &[String] {
        &self.step_order
    }
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
