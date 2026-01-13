//! Assertion types and builders for verifying step results.

use mew_core::Value;
use mew_session::{MutationSummary, QueryResult, StatementResult};
use std::collections::HashMap;

use crate::error::{ExampleError, ExampleResult};

/// A row of values, keyed by column name.
pub type Row = HashMap<String, Value>;

/// A complete assertion for a step result.
#[derive(Default)]
pub struct Assertion {
    // Mutation assertions
    pub created: Option<usize>,
    pub modified: Option<usize>,
    pub deleted: Option<usize>,
    pub linked: Option<usize>,
    pub unlinked: Option<usize>,

    // Query assertions - single value
    pub value: Option<Value>,
    pub value_min: Option<i64>,
    pub value_max: Option<i64>,

    // Query assertions - scalar (column name + value)
    pub scalar_column: Option<String>,
    pub scalar_value: Option<Value>,

    // Query assertions - columns
    pub columns: Option<Vec<String>>,

    // Query assertions - rows
    pub rows: Option<usize>,
    pub rows_min: Option<usize>,
    pub rows_max: Option<usize>,
    pub empty: Option<bool>,
    pub contains: Vec<Row>,
    pub exactly: Option<Vec<Row>>,
    pub returns: Option<Vec<Row>>,
    pub ordered: bool,
    pub first: Option<Row>,
    pub last: Option<Row>,

    // Error assertions
    pub error: Option<String>,
    pub error_pattern: Option<String>,

    // Custom assertion function
    #[allow(clippy::type_complexity)]
    pub custom: Option<Box<dyn Fn(&StatementResult) -> bool + Send + Sync>>,
}


impl std::fmt::Debug for Assertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assertion")
            .field("created", &self.created)
            .field("modified", &self.modified)
            .field("deleted", &self.deleted)
            .field("linked", &self.linked)
            .field("value", &self.value)
            .field("scalar_column", &self.scalar_column)
            .field("columns", &self.columns)
            .field("rows", &self.rows)
            .field("empty", &self.empty)
            .field("returns", &self.returns)
            .field("ordered", &self.ordered)
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

    fn verify_mutation(&self, step: &str, result: &MutationSummary) -> ExampleResult<()> {
        if let Some(expected) = self.created {
            if result.nodes_created != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} created, got {}",
                        expected, result.nodes_created
                    ),
                ));
            }
        }

        if let Some(expected) = self.modified {
            if result.nodes_modified != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} modified, got {}",
                        expected, result.nodes_modified
                    ),
                ));
            }
        }

        if let Some(expected) = self.deleted {
            if result.nodes_deleted != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} deleted, got {}",
                        expected, result.nodes_deleted
                    ),
                ));
            }
        }

        if let Some(expected) = self.linked {
            if result.edges_created != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} linked, got {}",
                        expected, result.edges_created
                    ),
                ));
            }
        }

        if let Some(expected) = self.unlinked {
            if result.edges_deleted != expected {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected {} unlinked, got {}",
                        expected, result.edges_deleted
                    ),
                ));
            }
        }

        Ok(())
    }

    fn verify_query(&self, step: &str, result: &QueryResult) -> ExampleResult<()> {
        // Check columns first (if specified)
        if let Some(ref expected_columns) = self.columns {
            if result.columns.len() != expected_columns.len() {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "column count mismatch:\n  expected: {:?}\n  actual:   {:?}",
                        expected_columns, result.columns
                    ),
                ));
            }
            for (i, expected_col) in expected_columns.iter().enumerate() {
                if result.columns.get(i) != Some(expected_col) {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!(
                            "column mismatch at index {}:\n  expected: {:?}\n  actual:   {:?}",
                            i, expected_columns, result.columns
                        ),
                    ));
                }
            }
        }

        // Check scalar (column name + value for single-row, single-column result)
        if let Some(ref expected_column) = self.scalar_column {
            if result.rows.len() != 1 {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "scalar() expects 1 row, got {}\n  columns: {:?}",
                        result.rows.len(),
                        result.columns
                    ),
                ));
            }
            if result.columns.len() != 1 {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "scalar() expects 1 column, got {:?}",
                        result.columns
                    ),
                ));
            }
            if result.columns[0] != *expected_column {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "scalar() column mismatch:\n  expected: \"{}\"\n  actual:   \"{}\"",
                        expected_column, result.columns[0]
                    ),
                ));
            }
            if let Some(ref expected_value) = self.scalar_value {
                let actual = &result.rows[0][0];
                if !values_equal(actual, expected_value) {
                    return Err(ExampleError::assertion_failed(
                        step,
                        format!(
                            "scalar() value mismatch for column \"{}\":\n  expected: {}\n  actual:   {}",
                            expected_column,
                            format_value(expected_value),
                            format_value(actual)
                        ),
                    ));
                }
            }
        }

        // Check single value (legacy - no column verification)
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
                    format!(
                        "value mismatch:\n  expected: {}\n  actual:   {}",
                        format_value(expected_value),
                        format_value(actual)
                    ),
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

        // Check contains (deprecated - loose matching)
        for expected_row in &self.contains {
            if !result_contains_row(result, expected_row) {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "expected result to contain row:\n  expected: {}\n  columns:  {:?}\n  actual rows: {}",
                        format_row(expected_row),
                        result.columns,
                        result.rows.len()
                    ),
                ));
            }
        }

        // Check returns (strict row matching with ordered support)
        if let Some(ref expected_rows) = self.returns {
            if result.rows.len() != expected_rows.len() {
                return Err(ExampleError::assertion_failed(
                    step,
                    format!(
                        "returns() row count mismatch:\n  expected: {} rows\n  actual:   {} rows\n  columns:  {:?}",
                        expected_rows.len(),
                        result.rows.len(),
                        result.columns
                    ),
                ));
            }

            if self.ordered {
                // Ordered comparison - rows must match in sequence
                for (i, expected_row) in expected_rows.iter().enumerate() {
                    if !row_matches(&result.columns, &result.rows[i], expected_row) {
                        return Err(ExampleError::assertion_failed(
                            step,
                            format!(
                                "returns().ordered() row mismatch at index {}:\n  expected: {}\n  actual:   {}\n  columns:  {:?}",
                                i,
                                format_row(expected_row),
                                format_row_values(&result.columns, &result.rows[i]),
                                result.columns
                            ),
                        ));
                    }
                }
            } else {
                // Unordered comparison - multiset matching
                let mut remaining_rows: Vec<(usize, &[Value])> = result
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(i, r)| (i, r.as_slice()))
                    .collect();

                for (expected_idx, expected_row) in expected_rows.iter().enumerate() {
                    let pos = remaining_rows
                        .iter()
                        .position(|(_, row)| row_matches(&result.columns, row, expected_row));
                    match pos {
                        Some(idx) => {
                            remaining_rows.remove(idx);
                        }
                        None => {
                            return Err(ExampleError::assertion_failed(
                                step,
                                format!(
                                    "returns() missing expected row at index {}:\n  expected: {}\n  columns:  {:?}\n  remaining actual rows: {}",
                                    expected_idx,
                                    format_row(expected_row),
                                    result.columns,
                                    remaining_rows.len()
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // Check exactly (legacy - same as returns but kept for compatibility)
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

            if self.ordered {
                // Ordered comparison
                for (i, expected_row) in expected_rows.iter().enumerate() {
                    if !row_matches(&result.columns, &result.rows[i], expected_row) {
                        return Err(ExampleError::assertion_failed(
                            step,
                            format!(
                                "exactly().ordered() row mismatch at index {}:\n  expected: {}\n  actual:   {}\n  columns:  {:?}",
                                i,
                                format_row(expected_row),
                                format_row_values(&result.columns, &result.rows[i]),
                                result.columns
                            ),
                        ));
                    }
                }
            } else {
                // Multiset comparison
                let mut remaining_rows: Vec<&[Value]> =
                    result.rows.iter().map(|r| r.as_slice()).collect();
                for expected_row in expected_rows {
                    let pos = remaining_rows
                        .iter()
                        .position(|row| row_matches(&result.columns, row, expected_row));
                    match pos {
                        Some(idx) => {
                            remaining_rows.remove(idx);
                        }
                        None => {
                            return Err(ExampleError::assertion_failed(
                                step,
                                format!(
                                    "expected result to contain row:\n  expected: {}\n  columns:  {:?}",
                                    format_row(expected_row),
                                    result.columns
                                ),
                            ));
                        }
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
                        "first() row mismatch:\n  expected: {}\n  actual:   {}\n  columns:  {:?}",
                        format_row(expected_first),
                        format_row_values(&result.columns, &result.rows[0]),
                        result.columns
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
                        "last() row mismatch:\n  expected: {}\n  actual:   {}\n  columns:  {:?}",
                        format_row(expected_last),
                        format_row_values(&result.columns, last_row),
                        result.columns
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

    /// Assert that N edges were removed (unlinked).
    pub fn unlinked(mut self, n: usize) -> Self {
        self.assertion.unlinked = Some(n);
        self
    }

    // ========== Query assertions - columns ==========

    /// Assert that the result has exactly these columns (in order).
    ///
    /// # Example
    /// ```ignore
    /// .step("query", |a| a.columns(&["id", "name", "email"]).rows(5))
    /// ```
    pub fn columns(mut self, names: &[&str]) -> Self {
        self.assertion.columns = Some(names.iter().map(|s| s.to_string()).collect());
        self
    }

    // ========== Query assertions - scalar ==========

    /// Assert a single-row, single-column result with column name verification.
    ///
    /// This is the strict replacement for `contains_value()` - it verifies:
    /// - Exactly 1 row
    /// - Exactly 1 column
    /// - Column name matches
    /// - Value matches
    ///
    /// # Example
    /// ```ignore
    /// .step("count_products", |a| a.scalar("total_products", 5i64))
    /// ```
    pub fn scalar<V: IntoValue>(mut self, column: &str, value: V) -> Self {
        self.assertion.scalar_column = Some(column.to_string());
        self.assertion.scalar_value = Some(value.into_value());
        self
    }

    // ========== Query assertions - single value (legacy) ==========

    /// Assert that the result is a single value equal to the given value.
    ///
    /// Note: This does not verify the column name. Consider using `scalar()` instead.
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

    /// Alias for rows_min - assert at least N rows.
    pub fn rows_gte(self, n: usize) -> Self {
        self.rows_min(n)
    }

    /// Alias for rows_max - assert at most N rows.
    pub fn rows_lte(self, n: usize) -> Self {
        self.rows_max(n)
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

    // ========== Query assertions - row matching ==========

    /// Assert that the result contains exactly these rows.
    ///
    /// By default, order does not matter (multiset comparison).
    /// Use `.ordered()` to require rows in exact sequence.
    ///
    /// # Example
    /// ```ignore
    /// .step("list_users", |a| a.returns(vec![
    ///     row!{ name: "Alice", age: 30 },
    ///     row!{ name: "Bob", age: 25 },
    /// ]))
    ///
    /// // With ordering
    /// .step("sorted_users", |a| a
    ///     .returns(vec![row!{ name: "Alice" }, row!{ name: "Bob" }])
    ///     .ordered()
    /// )
    /// ```
    pub fn returns(mut self, rows: Vec<Row>) -> Self {
        self.assertion.returns = Some(rows);
        self
    }

    /// Assert that order matters for `returns()` or `exactly()` checks.
    ///
    /// When set, rows must match in exact sequence rather than as a multiset.
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

    // ========== Deprecated methods ==========

    /// Assert that the result contains exactly these rows (order-independent).
    ///
    /// **Deprecated**: Use `returns()` instead for clearer semantics.
    #[deprecated(since = "0.1.0", note = "use returns() instead")]
    pub fn exactly(mut self, rows: Vec<Row>) -> Self {
        self.assertion.exactly = Some(rows);
        self
    }

    /// Assert that the result contains a row matching the given fields.
    ///
    /// **Deprecated**: This is a loose check that ignores extra columns.
    /// Use `returns(vec![row])` for strict single-row matching.
    #[deprecated(since = "0.1.0", note = "use returns(vec![row]) instead")]
    pub fn contains(mut self, row: Row) -> Self {
        self.assertion.contains.push(row);
        self
    }

    /// Assert that the result contains a specific value (anywhere in any row).
    ///
    /// **Deprecated**: This is very loose - it doesn't verify which column or row.
    /// Use `scalar(column, value)` for single-value results, or `returns()` for row matching.
    #[deprecated(since = "0.1.0", note = "use scalar() or returns() instead")]
    pub fn contains_value<V: IntoValue>(self, v: V) -> Self {
        let value = v.into_value();
        self.assert_fn(move |result| {
            if let StatementResult::Query(q) = result {
                for row in &q.rows {
                    for cell in row {
                        if values_equal(cell, &value) {
                            return true;
                        }
                    }
                }
            }
            false
        })
    }

    /// Assert that the result contains a NULL value (anywhere in any row).
    ///
    /// **Deprecated**: Use `returns()` with explicit null values instead.
    #[deprecated(since = "0.1.0", note = "use returns() with explicit null instead")]
    pub fn contains_null(self) -> Self {
        self.assert_fn(move |result| {
            if let StatementResult::Query(q) = result {
                for row in &q.rows {
                    for cell in row {
                        if matches!(cell, Value::Null) {
                            return true;
                        }
                    }
                }
            }
            false
        })
    }

    /// Assert that the first row's first column equals the given value.
    ///
    /// **Deprecated**: Use `scalar(column, value)` or `first(row)` instead.
    #[deprecated(since = "0.1.0", note = "use scalar() or first() instead")]
    pub fn first_value<V: IntoValue>(self, v: V) -> Self {
        let value = v.into_value();
        self.assert_fn(move |result| {
            if let StatementResult::Query(q) = result {
                if let Some(first_row) = q.rows.first() {
                    if let Some(first_cell) = first_row.first() {
                        return values_equal(first_cell, &value);
                    }
                }
            }
            false
        })
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

/// Format a Value for display in error messages.
fn format_value(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{:.6}", f),
        Value::String(s) => format!("\"{}\"", s),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Timestamp(ts) => format!("timestamp({})", ts),
        Value::Duration(d) => format!("duration({})", d),
        Value::NodeRef(id) => format!("node({})", id),
        Value::EdgeRef(id) => format!("edge({})", id),
    }
}

/// Format a Row (HashMap) for display in error messages.
fn format_row(row: &Row) -> String {
    let mut parts: Vec<String> = row
        .iter()
        .map(|(k, v)| format!("{}: {}", k, format_value(v)))
        .collect();
    parts.sort(); // Consistent ordering
    format!("{{ {} }}", parts.join(", "))
}

/// Format actual row values with column names for display in error messages.
fn format_row_values(columns: &[String], values: &[Value]) -> String {
    let parts: Vec<String> = columns
        .iter()
        .zip(values.iter())
        .map(|(col, val)| format!("{}: {}", col, format_value(val)))
        .collect();
    format!("{{ {} }}", parts.join(", "))
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
