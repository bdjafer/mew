//! Test execution against MEW

use crate::types::*;
use crate::oracle::{Oracle, VerifyResult};
use std::time::Instant;

/// Executes test cases against MEW and collects results
pub struct TestExecutor {
    /// Results from executed tests
    pub results: Vec<TestResult>,
}

impl TestExecutor {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Execute a test suite and return results
    pub fn execute(&mut self, suite: &TestSuite) -> Result<TestSummary, TestGenError> {
        self.results.clear();

        for test_case in &suite.test_cases {
            let result = self.execute_test(test_case, &suite.ontology_source)?;
            self.results.push(result);
        }

        Ok(self.summarize())
    }

    /// Execute a single test case
    fn execute_test(
        &self,
        test: &TestCase,
        ontology_source: &str,
    ) -> Result<TestResult, TestGenError> {
        let start = Instant::now();

        // Create a fresh session for each test
        let actual = self.run_in_session(ontology_source, &test.setup, &test.statement)?;

        let duration_us = start.elapsed().as_micros() as u64;

        // Verify using oracle
        let verify_result = Oracle::verify(&test.expected, &actual);
        let passed = verify_result.is_pass();

        Ok(TestResult {
            test_id: test.id.clone(),
            passed,
            expected: test.expected.clone(),
            actual,
            duration_us,
            trust_level: test.trust_level,
        })
    }

    /// Run statements in a MEW session
    fn run_in_session(
        &self,
        ontology_source: &str,
        setup: &[String],
        statement: &str,
    ) -> Result<ActualResult, TestGenError> {
        use mew_compiler::Compiler;
        use mew_session::Session;

        // 1. Compile ontology source to registry
        let mut compiler = Compiler::new();
        let registry = compiler.compile(ontology_source)
            .map_err(|e| TestGenError::ExecutionError(format!("Compile error: {}", e)))?;

        // 3. Create session
        let mut session = Session::new(1, &registry);

        // 4. Run setup statements
        for stmt in setup {
            if let Err(e) = session.execute(stmt) {
                // Setup failures might be expected - log but continue
                eprintln!("Setup warning: {} - {}", stmt, e);
            }
        }

        // 5. Execute the main statement
        match session.execute(statement) {
            Ok(result) => {
                // Convert result to ActualResult
                self.convert_result(result)
            }
            Err(e) => {
                Ok(ActualResult::Error(e.to_string()))
            }
        }
    }

    /// Convert MEW result to ActualResult
    fn convert_result(
        &self,
        result: mew_session::StatementResult,
    ) -> Result<ActualResult, TestGenError> {
        use mew_session::StatementResult;

        match result {
            StatementResult::Query(query_result) => {
                let converted: Vec<Row> = query_result.rows.into_iter()
                    .map(|row| {
                        let columns: Vec<Value> = row.into_iter()
                            .map(|v| Value::from(&v))
                            .collect();
                        Row { columns }
                    })
                    .collect();
                Ok(ActualResult::Rows(converted))
            }
            StatementResult::Mutation(mutation_result) => {
                if mutation_result.nodes_affected > 0 || mutation_result.edges_affected > 0 {
                    Ok(ActualResult::Success)
                } else {
                    Ok(ActualResult::Success)
                }
            }
            StatementResult::Transaction(_) => {
                Ok(ActualResult::Success)
            }
            StatementResult::Empty => {
                Ok(ActualResult::Success)
            }
        }
    }

    /// Summarize test results
    fn summarize(&self) -> TestSummary {
        use std::collections::HashMap;

        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let mut by_trust_level: HashMap<TrustLevel, (usize, usize)> = HashMap::new();
        let mut by_complexity: HashMap<u8, (usize, usize)> = HashMap::new();
        let by_tag: HashMap<String, (usize, usize)> = HashMap::new();

        for result in &self.results {
            let entry = by_trust_level.entry(result.trust_level).or_insert((0, 0));
            entry.1 += 1;
            if result.passed {
                entry.0 += 1;
            }
        }

        let total_duration_us: u64 = self.results.iter().map(|r| r.duration_us).sum();

        TestSummary {
            total,
            passed,
            failed,
            by_trust_level,
            by_complexity,
            by_tag,
            total_duration_us,
        }
    }
}

impl Default for TestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here, but they depend on mew-session
    // which we can test in the actual test runs
}
