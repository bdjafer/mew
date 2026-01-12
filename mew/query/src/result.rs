//! Query result types.

use mew_core::Value;
use std::collections::HashMap;

/// A single row of query results.
#[derive(Debug, Clone, Default)]
pub struct QueryRow {
    /// Column values in order.
    columns: Vec<Value>,
    /// Column names.
    names: Vec<String>,
}

impl QueryRow {
    /// Create a new empty row.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            names: Vec::new(),
        }
    }

    /// Create a row from columns and names.
    pub fn from_columns(names: Vec<String>, columns: Vec<Value>) -> Self {
        Self { names, columns }
    }

    /// Add a column to the row.
    pub fn push(&mut self, name: impl Into<String>, value: Value) {
        self.names.push(name.into());
        self.columns.push(value);
    }

    /// Get a column value by index.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.columns.get(index)
    }

    /// Get a column value by name.
    pub fn get_by_name(&self, name: &str) -> Option<&Value> {
        self.names
            .iter()
            .position(|n| n == name)
            .and_then(|i| self.columns.get(i))
    }

    /// Get the number of columns.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Check if the row is empty.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Get column names.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Get column values.
    pub fn values(&self) -> &[Value] {
        &self.columns
    }

    /// Convert to a hashmap.
    pub fn to_map(&self) -> HashMap<String, Value> {
        self.names
            .iter()
            .cloned()
            .zip(self.columns.iter().cloned())
            .collect()
    }
}

/// Query results collection.
#[derive(Debug, Clone, Default)]
pub struct QueryResults {
    /// Column names.
    column_names: Vec<String>,
    /// Rows of results.
    rows: Vec<QueryRow>,
}

impl QueryResults {
    /// Create a new empty result set.
    pub fn new() -> Self {
        Self {
            column_names: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Create a result set with column names.
    pub fn with_columns(names: Vec<String>) -> Self {
        Self {
            column_names: names,
            rows: Vec::new(),
        }
    }

    /// Add a row to the results.
    pub fn push(&mut self, row: QueryRow) {
        if self.column_names.is_empty() && !row.names().is_empty() {
            self.column_names = row.names().to_vec();
        }
        self.rows.push(row);
    }

    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if results are empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get column names.
    pub fn column_names(&self) -> &[String] {
        &self.column_names
    }

    /// Get all rows.
    pub fn rows(&self) -> &[QueryRow] {
        &self.rows
    }

    /// Iterate over rows.
    pub fn iter(&self) -> impl Iterator<Item = &QueryRow> {
        self.rows.iter()
    }

    /// Convert to vector of rows.
    pub fn into_rows(self) -> Vec<QueryRow> {
        self.rows
    }
}

impl IntoIterator for QueryResults {
    type Item = QueryRow;
    type IntoIter = std::vec::IntoIter<QueryRow>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryResults {
    type Item = &'a QueryRow;
    type IntoIter = std::slice::Iter<'a, QueryRow>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_row() {
        // GIVEN
        let mut row = QueryRow::new();

        // WHEN
        row.push("name", Value::String("Alice".to_string()));
        row.push("age", Value::Int(30));

        // THEN
        assert_eq!(row.len(), 2);
        assert_eq!(
            row.get_by_name("name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(row.get_by_name("age"), Some(&Value::Int(30)));
    }

    #[test]
    fn test_query_results() {
        // GIVEN
        let mut results = QueryResults::new();

        // WHEN
        let mut row1 = QueryRow::new();
        row1.push("id", Value::Int(1));
        results.push(row1);

        let mut row2 = QueryRow::new();
        row2.push("id", Value::Int(2));
        results.push(row2);

        // THEN
        assert_eq!(results.len(), 2);
        assert!(!results.is_empty());
    }
}
