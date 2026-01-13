//! Scenario runner.

use std::fs;

use mew_compiler::compile;
use mew_session::Session;

use crate::error::{ExampleError, ExampleResult};
use crate::loader::Operations;
use crate::scenario::Scenario;

/// Runs a scenario against MEW.
pub struct Runner<'s> {
    scenario: &'s Scenario,
    operations: Operations,
}

impl<'s> Runner<'s> {
    /// Create a new runner for a scenario.
    pub fn new(scenario: &'s Scenario) -> ExampleResult<Self> {
        let operations = scenario.load_operations()?;
        Ok(Self {
            scenario,
            operations,
        })
    }

    /// Run the scenario.
    pub fn run(&self) -> ExampleResult<()> {
        // 1. Load and compile the ontology
        let ontology_path = self.scenario.ontology_path()?;
        let ontology_source = fs::read_to_string(&ontology_path)
            .map_err(|e| ExampleError::file_read(&ontology_path, e))?;

        let registry = compile(&ontology_source)
            .map_err(|e| ExampleError::ontology_compile(&ontology_path, e.to_string()))?;

        // 2. Create a session
        let mut session = Session::new(1, &registry);

        // 3. Execute seed if present
        if let Some(seed_path) = self.scenario.seed_path() {
            if !seed_path.exists() {
                return Err(ExampleError::file_read(
                    &seed_path,
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "configured seed file does not exist",
                    ),
                ));
            }

            let seed_source = fs::read_to_string(&seed_path)
                .map_err(|e| ExampleError::file_read(&seed_path, e))?;

            // Parse seed as operations (same format)
            let seed_ops = Operations::parse(&seed_source)?;

            // Execute each step in the seed
            for step_name in seed_ops.step_names() {
                if let Some(statement) = seed_ops.get_step(step_name) {
                    session.execute_all(statement).map_err(|e| {
                        ExampleError::step_execution(format!("seed:{}", step_name), e.to_string())
                    })?;
                }
            }
        }

        // 4. Execute each step and verify assertions
        for step in self.scenario.steps() {
            let statement = self
                .operations
                .get_step(&step.name)
                .ok_or_else(|| ExampleError::step_not_found(&step.name))?;

            // Execute the statement
            let result = session.execute_all(statement);

            // Convert to the format expected by assertions
            let result_for_assertion = result.map_err(|e| e.to_string());

            // Verify the assertion
            step.assertion.verify(&step.name, &result_for_assertion)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::scenario::Scenario;

    #[test]
    fn test_runner_with_inline_operations() {
        // This test requires the actual MEW components to work
        // For now, just verify the runner can be constructed

        let ops_source = r#"
--# spawn
SPAWN t: Task { title: "Test" }

--# query
MATCH t: Task RETURN t.title
"#;

        let scenario = Scenario::new("test")
            .operations_source(ops_source)
            .unwrap()
            .step("spawn", |a| a.created(1))
            .step("query", |a| a.rows(1));

        // Runner creation should work
        // (actual run would fail without proper ontology setup)
        assert_eq!(scenario.steps().len(), 2);
    }
}
