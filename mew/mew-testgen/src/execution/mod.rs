//! Test execution against MEW

use crate::types::*;
use crate::oracle::Oracle;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;

const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Executes test cases against MEW
pub struct TestExecutor {
    pub results: Vec<TestResult>,
    pub timeout: Duration,
    pub verbose: bool,
}

impl TestExecutor {
    pub fn new() -> Self {
        Self { results: Vec::new(), timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS), verbose: false }
    }

    pub fn verbose(mut self) -> Self { self.verbose = true; self }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self { self.timeout = timeout; self }

    pub fn execute(&mut self, suite: &TestSuite) -> Result<TestSummary, TestGenError> {
        self.results.clear();

        for (i, test) in suite.test_cases.iter().enumerate() {
            if self.verbose {
                eprint!("    Test {}/{} [{}]... ", i + 1, suite.test_cases.len(), test.id);
            }
            
            let result = self.execute_test(test, &suite.ontology_source)?;
            
            if self.verbose {
                eprintln!("{}", if result.passed { "PASS" } else { "FAIL" });
            }
            
            self.results.push(result);
        }

        Ok(self.summarize())
    }

    fn execute_test(&self, test: &TestCase, ontology_source: &str) -> Result<TestResult, TestGenError> {
        let start = Instant::now();
        let source = ontology_source.to_string();
        let setup = test.setup.clone();
        let statement = test.statement.clone();
        let setup_len = setup.len();
        let timeout = self.timeout;

        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let _ = tx.send(Self::run_in_session(&source, &setup, &statement));
        });

        let actual = match rx.recv_timeout(timeout) {
            Ok(result) => result?,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                drop(handle);
                ActualResult::Error(format!("Timed out after {:?} (setup={} stmts)", timeout, setup_len))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                ActualResult::Error("Thread panicked".to_string())
            }
        };

        let passed = Oracle::verify(&test.expected, &actual).is_pass();

        Ok(TestResult {
            test_id: test.id.clone(),
            passed,
            expected: test.expected.clone(),
            actual,
            duration_us: start.elapsed().as_micros() as u64,
            trust_level: test.trust_level,
        })
    }

    fn run_in_session(source: &str, setup: &[String], statement: &str) -> Result<ActualResult, TestGenError> {
        use mew_compiler::Compiler;
        use mew_session::Session;

        let registry = Compiler::new().compile(source)
            .map_err(|e| TestGenError::ExecutionError(format!("Compile error: {}", e)))?;

        let mut session = Session::new(1, &registry);

        for stmt in setup {
            let _ = session.execute(stmt); // Ignore setup errors
        }

        match session.execute(statement) {
            Ok(result) => Self::convert_result(result),
            Err(e) => Ok(ActualResult::Error(e.to_string())),
        }
    }

    fn convert_result(result: mew_session::StatementResult) -> Result<ActualResult, TestGenError> {
        use mew_session::StatementResult;

        Ok(match result {
            StatementResult::Query(qr) => {
                ActualResult::Rows(qr.rows.into_iter()
                    .map(|row| Row { columns: row.into_iter().map(|v| Value::from(&v)).collect() })
                    .collect())
            }
            StatementResult::Mutation(_) | StatementResult::Transaction(_) | StatementResult::Empty => {
                ActualResult::Success
            }
        })
    }

    fn summarize(&self) -> TestSummary {
        use std::collections::HashMap;

        let passed = self.results.iter().filter(|r| r.passed).count();
        let mut by_trust_level: HashMap<TrustLevel, (usize, usize)> = HashMap::new();

        for r in &self.results {
            let e = by_trust_level.entry(r.trust_level).or_insert((0, 0));
            e.1 += 1;
            if r.passed { e.0 += 1; }
        }

        TestSummary {
            total: self.results.len(),
            passed,
            failed: self.results.len() - passed,
            by_trust_level,
            by_complexity: HashMap::new(),
            by_tag: HashMap::new(),
            total_duration_us: self.results.iter().map(|r| r.duration_us).sum(),
        }
    }
}

impl Default for TestExecutor {
    fn default() -> Self { Self::new() }
}
