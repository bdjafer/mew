//! Assertion types and builders for verifying step results.

use mew_core::Value;
use mew_session::{MutationResult, QueryResult, StatementResult};
use std::collections::HashMap;

use crate::error::{ExampleError, ExampleResult};

/// A row of values, keyed by column name.
pub type Row = HashMap<String, Value>;

/// A complete assertion for a step result.
pub struct Assertion {
    // Mutation assertions
    pub created: Option<usize>,
    pub modified: Option<usize>,
    pub deleted: Option<usize>,
    pub linked: Option<usize>,

    // Query assertions - single value
    pub value: Option<Value>,
    pub value_min: Option<i64>,
    pub value_max: Option<i64>,

    // Query assertions - rows
    pub rows: Option<usize>,
    pub rows_min: Option<usize>,
    pub rows_max: Option<usize>,
    pub empty: Option<bool>,
    pub contains: Vec<Row>,
    pub exactly: Option<Vec<Row>>,
    pub ordered: bool,
    pub first: Option<Row>,
    pub last: Option<Row>,

    // Error assertions
    pub error: Option<String>,
    pub error_pattern: Option<String>,

    // Capture (for future use with variables)
    pub capture: Option<String>,

    // Custom assertion function
    #[allow(clippy::type_complexity)]
    pub custom: Option<Box<dyn Fn(&StatementResult) -> bool + Send + Sync>>,
}

impl Default for Assertion {
    fn default() -> Self {
        Self {
            created: None,
            modified: None,
            deleted: None,
            linked: None,
            value: None,
            value_min: None,
            value_max: None,
            rows: None,
            rows_min: None,
            rows_max: None,
            empty: None,
            contains: Vec::new(),
            exactly: None,
            ordered: false,
            first: None,
            last: None,
            error: None,
            error_pattern: None,
            capture: None,
            custom: None,
        }
    }
}

impl std::fmt::Debug for Assertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assertion")
            .field("created", &self.created)
            .field("modified", &self.modified)
            .field("deleted", &self.deleted)
            .field("linked", &self.linked)
            .field("value", &self.value)
            .field("rows", &self.rows)
            .field("empty", &self.empty)
            .field("error", &self.error)
            .field("custom", &self.custom.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Assertion {
    /// Create a new empty assertion.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify the assertion against a result.
    pub fn verify(
        &self,
        step: &str,
        result: &Result<StatementResult, String>,
    ) -> ExampleResult<()> {
        // Check error expectations first
        if let Some(ref expected_error) = self.error {
            match result {
                Err(msg) if msg.contains(expected_error) => return Ok(()),
                Err(msg) => {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!(
                            "expected error containing '{}', got: {}",
                            expected_error, msg
                        ),
                    ))
                }
                Ok(_) => {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!(
                            "expected error containing '{}', but step succeeded",
                            expected_error
                        ),
                    ))
                }
            }
        }

        if let Some(ref pattern) = self.error_pattern {
            let re = regex_lite::Regex::new(pattern).map_err(|e| {
                ExampleError::assertion_failed(step, format!("invalid regex pattern: {}", e))
            })?;
            match result {
                Err(msg) if re.is_match(msg) => return Ok(()),
                Err(msg) => {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!("expected error matching '{}', got: {}", pattern, msg),
                    ))
                }
                Ok(_) => {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!("expected error matching '{}', but step succeeded", pattern),
                    ))
                }
            }
        }

        // If we expect an error but got success, that's a failure
        let result = result
            .as_ref()
            .map_err(|msg| ExampleError::assertion_failed(step, format!("step failed: {}", msg)))?;

        // Run custom assertion if present
        if let Some(ref custom) = self.custom {
            if !custom(result) {
                return Err(ExampleError::assertion_failed(
                    step,
                    "custom assertion failed",
                ));
            }
        }

        match result {
            StatementResult::Mutation(m) => self.verify_mutation(step, m),
            StatementResult::Query(q) => self.verify_query(step, q),
            StatementResult::Transaction(_) => Ok(()),
            StatementResult::Empty => Ok(()),
        }
    }

    fn verify_mutation(&self, step: &str, result: &MutationResult) -> ExampleResult<()> {
        if let Some(expected) = self.created {
            if result.nodes_affected != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} created, got {}",
                        expected, result.nodes_affected
                    ),
                ));
            }
        }

        if let Some(expected) = self.modified {
            // For now, modified is tracked via nodes_affected in SET operations
            if result.nodes_affected != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} modified, got {}",
                        expected, result.nodes_affected
                    ),
                ));
            }
        }

        if let Some(expected) = self.deleted {
            // Deleted nodes are tracked via nodes_affected in KILL operations
            if result.nodes_affected != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} deleted, got {}",
                        expected, result.nodes_affected
                    ),
                ));
            }
        }

        if let Some(expected) = self.linked {
            if result.edges_affected != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} linked, got {}",
                        expected, result.edges_affected
                    ),
                ));
            }
        }

        Ok(())
    }

    fn verify_query(&self, step: &str, result: &QueryResult) -> ExampleResult<()> {
        // Check single value
        if let Some(ref expected_value) = self.value {
            if result.rows.len() != 1 || result.rows[0].len() != 1 {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected single value, got {} rows with {} columns",
                        result.rows.len(),
                        result.rows.first().map(|r| r.len()).unwrap_or(0)
                    ),
                ));
            }
            let actual = &result.rows[0][0];
            if !values_equal(actual, expected_value) {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!("expected value {:?}, got {:?}", expected_value, actual),
                ));
            }
        }

        // Check value range
        if let Some(min) = self.value_min {
            if result.rows.len() != 1 || result.rows[0].len() != 1 {
                return Err(ExampleError::assertion_failed(
                    step,
                    "expected single value for range check",
                ));
            }
            if let Value::Int(v) = &result.rows[0][0] {
                if *v < min {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!("expected value >= {}, got {}", min, v),
                    ));
                }
            }
        }

        if let Some(max) = self.value_max {
            if result.rows.len() != 1 || result.rows[0].len() != 1 {
                return Err(ExampleError::assertion_failed(
                    step,
                    "expected single value for range check",
                ));
            }
            if let Value::Int(v) = &result.rows[0][0] {
                if *v > max {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!("expected value <= {}, got {}", max, v),
                    ));
                }
            }
        }

        // Check row count
        if let Some(expected) = self.rows {
            if result.rows.len() != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!("expected {} rows, got {}", expected, result.rows.len()),
                ));
            }
        }

        if let Some(min) = self.rows_min {
            if result.rows.len() < min {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!("expected at least {} rows, got {}", min, result.rows.len()),
                ));
            }
        }

        if let Some(max) = self.rows_max {
            if result.rows.len() > max {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!("expected at most {} rows, got {}", max, result.rows.len()),
                ));
            }
        }

        // Check empty
        if let Some(expected_empty) = self.empty {
            if result.rows.is_empty() != expected_empty {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} empty, got {} rows",
                        if expected_empty { "" } else { "not" },
                        result.rows.len()
                    ),
                ));
            }
        }

        // Check contains
        for expected_row in &self.contains {
            if !result_contains_row(result, expected_row) {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!("expected result to contain row {:?}", expected_row),
                ));
            }
        }

        // Check exactly (with multiplicity)
        if let Some(ref expected_rows) = self.exactly {
            if result.rows.len() != expected_rows.len() {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected exactly {} rows, got {}",
                        expected_rows.len(),
                        result.rows.len()
                    ),
                ));
            }
            // Use multiset comparison: match and remove each expected row from actual
            let mut remaining_rows: Vec<&[Value]> = result.rows.iter().map(|r| r.as_slice()).collect();
            for expected_row in expected_rows {
                let pos = remaining_rows.iter().position(|row| row_matches(&result.columns, row, expected_row));
                match pos {
                    Some(idx) => {
                        remaining_rows.remove(idx);
                    }
                    None => {
                        return Err(ExampleError::assertion_failed(
                            step,
                            format!("expected result to contain row {:?}", expected_row),
                        ));
                    }
                }
            }
        }

        // Check first row
        if let Some(ref expected_first) = self.first {
            if result.rows.is_empty() {
                return Err(ExampleError::assertion_failed(
                    step,
                    "expected first row but result is empty",
                ));
            }
            if !row_matches(&result.columns, &result.rows[0], expected_first) {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "first row {:?} does not match expected {:?}",
                        result.rows[0], expected_first
                    ),
                ));
            }
        }

        // Check last row
        if let Some(ref expected_last) = self.last {
            if result.rows.is_empty() {
                return Err(ExampleError::assertion_failed(
                    step,
                    "expected last row but result is empty",
                ));
            }
            let last_row = result.rows.last().unwrap();
            if !row_matches(&result.columns, last_row, expected_last) {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "last row {:?} does not match expected {:?}",
                        last_row, expected_last
                    ),
                ));
            }
        }

        Ok(())
    }
}

/// Builder for fluent assertion construction.
pub struct AssertionBuilder {
    assertion: Assertion,
}

impl AssertionBuilder {
    /// Create a new assertion builder.
    pub fn new() -> Self {
        Self {
            assertion: Assertion::new(),
        }
    }

    /// Build the assertion.
    pub fn build(self) -> Assertion {
        self.assertion
    }

    // ========== Mutation assertions ==========

    /// Assert that N nodes were created.
    pub fn created(mut self, n: usize) -> Self {
        self.assertion.created = Some(n);
        self
    }

    /// Assert that N nodes were modified.
    pub fn modified(mut self, n: usize) -> Self {
        self.assertion.modified = Some(n);
        self
    }

    /// Assert that N nodes were deleted.
    pub fn deleted(mut self, n: usize) -> Self {
        self.assertion.deleted = Some(n);
        self
    }

    /// Assert that N edges were created (linked).
    pub fn linked(mut self, n: usize) -> Self {
        self.assertion.linked = Some(n);
        self
    }

    // ========== Query assertions - single value ==========

    /// Assert that the result is a single value equal to the given value.
    pub fn value<V: IntoValue>(mut self, v: V) -> Self {
        self.assertion.value = Some(v.into_value());
        self
    }

    /// Assert that the single value is at least min.
    pub fn value_min(mut self, min: i64) -> Self {
        self.assertion.value_min = Some(min);
        self
    }

    /// Assert that the single value is at most max.
    pub fn value_max(mut self, max: i64) -> Self {
        self.assertion.value_max = Some(max);
        self
    }

    // ========== Query assertions - rows ==========

    /// Assert that the result has exactly N rows.
    pub fn rows(mut self, n: usize) -> Self {
        self.assertion.rows = Some(n);
        self
    }

    /// Assert that the result has at least N rows.
    pub fn rows_min(mut self, n: usize) -> Self {
        self.assertion.rows_min = Some(n);
        self
    }

    /// Assert that the result has at most N rows.
    pub fn rows_max(mut self, n: usize) -> Self {
        self.assertion.rows_max = Some(n);
        self
    }

    /// Assert that the result is empty.
    pub fn empty(mut self) -> Self {
        self.assertion.empty = Some(true);
        self
    }

    /// Assert that the result is not empty.
    pub fn not_empty(mut self) -> Self {
        self.assertion.empty = Some(false);
        self
    }

    /// Assert that the result contains a row matching the given fields.
    pub fn contains(mut self, row: Row) -> Self {
        self.assertion.contains.push(row);
        self
    }

    /// Assert that the result contains exactly these rows (order-independent).
    pub fn exactly(mut self, rows: Vec<Row>) -> Self {
        self.assertion.exactly = Some(rows);
        self
    }

    /// Assert that order matters for contains checks.
    pub fn ordered(mut self) -> Self {
        self.assertion.ordered = true;
        self
    }

    /// Assert that the first row matches the given fields.
    pub fn first(mut self, row: Row) -> Self {
        self.assertion.first = Some(row);
        self
    }

    /// Assert that the last row matches the given fields.
    pub fn last(mut self, row: Row) -> Self {
        self.assertion.last = Some(row);
        self
    }

    // ========== Error assertions ==========

    /// Assert that the step fails with an error containing the given string.
    pub fn error(mut self, contains: impl Into<String>) -> Self {
        self.assertion.error = Some(contains.into());
        self
    }

    /// Assert that the step fails with an error matching the given regex.
    pub fn error_matches(mut self, pattern: impl Into<String>) -> Self {
        self.assertion.error_pattern = Some(pattern.into());
        self
    }

    // ========== Advanced ==========

    /// Capture a value for use in later steps (future feature).
    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.assertion.capture = Some(name.into());
        self
    }

    /// Custom assertion function.
    pub fn assert_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&StatementResult) -> bool + Send + Sync + 'static,
    {
        self.assertion.custom = Some(Box::new(f));
        self
    }
}

impl Default for AssertionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}

fn result_contains_row(result: &QueryResult, expected: &Row) -> bool {
    result
        .rows
        .iter()
        .any(|row| row_matches(&result.columns, row, expected))
}

fn row_matches(columns: &[String], row: &[Value], expected: &Row) -> bool {
    for (key, expected_value) in expected {
        let col_idx = columns.iter().position(|c| c == key);
        match col_idx {
            Some(idx) => {
                if !values_equal(&row[idx], expected_value) {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

/// Trait for converting values into MEW Values.
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoValue for usize {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}
