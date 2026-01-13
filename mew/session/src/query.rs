//! Query execution helpers for MEW sessions.
//!
//! This module contains shared logic for executing queries and
//! converting query results to the session result format.

use mew_core::Value;
use mew_query::QueryResults;

use crate::result::QueryResult;

/// Convert an internal query result to a session QueryResult.
///
/// This extracts column names and row data into the format expected
/// by session clients.
pub fn convert_query_result(result: &QueryResults) -> QueryResult {
    let columns: Vec<String> = result.column_names().to_vec();
    let types = vec!["any".to_string(); columns.len()]; // Simplified type info

    let mut rows = Vec::new();
    for row in result.rows() {
        let mut values = Vec::with_capacity(columns.len());
        for col in &columns {
            values.push(row.get_by_name(col).cloned().unwrap_or(Value::Null));
        }
        rows.push(values);
    }

    QueryResult::new(columns, types, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mew_query::{QueryResults, QueryRow};

    #[test]
    fn test_convert_empty_result() {
        // GIVEN
        let result = QueryResults::with_columns(vec!["a".to_string(), "b".to_string()]);

        // WHEN
        let converted = convert_query_result(&result);

        // THEN
        assert_eq!(converted.columns, vec!["a", "b"]);
        assert!(converted.rows.is_empty());
    }

    #[test]
    fn test_convert_with_rows() {
        // GIVEN
        let mut result = QueryResults::with_columns(vec!["x".to_string()]);
        let mut row = QueryRow::new();
        row.push("x", Value::Int(42));
        result.push(row);

        // WHEN
        let converted = convert_query_result(&result);

        // THEN
        assert_eq!(converted.columns, vec!["x"]);
        assert_eq!(converted.rows.len(), 1);
        assert_eq!(converted.rows[0][0], Value::Int(42));
    }
}
