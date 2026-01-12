//! Scenario definition and builder.

use std::path::{Path, PathBuf};

use crate::assertion::{Assertion, AssertionBuilder};
use crate::error::{ExampleError, ExampleResult};
use crate::loader::Operations;
use crate::runner::Runner;

/// A step in a scenario with its assertion.
#[derive(Debug)]
pub struct Step {
    /// Step name (matches `--# name` in operations file).
    pub name: String,
    /// Assertion to verify the result.
    pub assertion: Assertion,
}

/// A complete test scenario.
pub struct Scenario {
    /// Scenario name (for reporting).
    name: String,
    /// Path to the ontology file.
    ontology_path: Option<PathBuf>,
    /// Path to the seed file (optional).
    seed_path: Option<PathBuf>,
    /// Path to the operations file.
    operations_path: Option<PathBuf>,
    /// Parsed operations (if loaded inline).
    operations: Option<Operations>,
    /// Steps with assertions.
    steps: Vec<Step>,
    /// Base path for resolving relative paths.
    base_path: PathBuf,
}

impl Scenario {
    /// Create a new scenario with the given name.
    ///
    /// The name should match the operations file name (without extension).
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ontology_path: None,
            seed_path: None,
            operations_path: None,
            operations: None,
            steps: Vec::new(),
            base_path: examples_root(),
        }
    }

    /// Set the base path for resolving relative paths.
    pub fn base_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.base_path = path.into();
        self
    }

    /// Set the ontology file path (relative to examples/).
    pub fn ontology(mut self, path: impl Into<PathBuf>) -> Self {
        self.ontology_path = Some(path.into());
        self
    }

    /// Set the seed file path (relative to examples/).
    pub fn seed(mut self, path: impl Into<PathBuf>) -> Self {
        self.seed_path = Some(path.into());
        self
    }

    /// Set the operations file path (relative to examples/).
    pub fn operations(mut self, path: impl Into<PathBuf>) -> Self {
        self.operations_path = Some(path.into());
        self
    }

    /// Load operations from a string (for testing).
    pub fn operations_source(mut self, source: &str) -> ExampleResult<Self> {
        self.operations = Some(Operations::parse(source)?);
        Ok(self)
    }

    /// Add a step with an assertion.
    ///
    /// The step name must match a `--# name` marker in the operations file.
    pub fn step<F>(mut self, name: impl Into<String>, assertion_fn: F) -> Self
    where
        F: FnOnce(AssertionBuilder) -> AssertionBuilder,
    {
        let name = name.into();
        let assertion = assertion_fn(AssertionBuilder::new()).build();
        self.steps.push(Step { name, assertion });
        self
    }

    /// Run the scenario and return the result.
    pub fn run(&self) -> ExampleResult<()> {
        let runner = Runner::new(self)?;
        runner.run()
    }

    /// Get the scenario name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the ontology path (resolved).
    pub fn ontology_path(&self) -> ExampleResult<PathBuf> {
        match &self.ontology_path {
            Some(p) => Ok(self.resolve_path(p)),
            None => Err(ExampleError::missing_ontology(&self.name)),
        }
    }

    /// Get the seed path (resolved), if any.
    pub fn seed_path(&self) -> Option<PathBuf> {
        self.seed_path.as_ref().map(|p| self.resolve_path(p))
    }

    /// Get the operations, loading from file if needed.
    pub fn load_operations(&self) -> ExampleResult<Operations> {
        if let Some(ref ops) = self.operations {
            return Ok(ops.clone());
        }

        let path = match &self.operations_path {
            Some(p) => self.resolve_path(p),
            None => {
                // Default: look for {name}.mew in the current scenario directory
                return Err(ExampleError::missing_operations(PathBuf::from(&self.name)));
            }
        };

        Operations::load(&path)
    }

    /// Get the steps.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Resolve a path relative to the base path.
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        }
    }
}

/// Get the examples root directory.
///
/// This looks for the `examples/` directory relative to the workspace root.
fn examples_root() -> PathBuf {
    // Try to find examples/ relative to CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        // mew/mew-examples -> mew -> repo root -> examples
        if let Some(workspace) = manifest_path.parent().and_then(|p| p.parent()) {
            let examples = workspace.join("examples");
            if examples.exists() {
                return examples;
            }
        }
    }

    // Fallback: try current directory
    let cwd = std::env::current_dir().unwrap_or_default();

    // Check if we're in the repo root
    let examples = cwd.join("examples");
    if examples.exists() {
        return examples;
    }

    // Check if we're in mew/
    let examples = cwd.parent().map(|p| p.join("examples")).unwrap_or_default();
    if examples.exists() {
        return examples;
    }

    // Give up and return a relative path
    PathBuf::from("examples")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder() {
        let scenario = Scenario::new("test")
            .ontology("level-1/bookmarks/ontology.mew")
            .seed("level-1/bookmarks/seeds/empty.mew")
            .step("spawn", |a| a.created(1))
            .step("query", |a| a.rows(1));

        assert_eq!(scenario.name(), "test");
        assert_eq!(scenario.steps().len(), 2);
        assert_eq!(scenario.steps()[0].name, "spawn");
        assert_eq!(scenario.steps()[1].name, "query");
    }
}
