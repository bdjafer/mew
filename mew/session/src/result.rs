//! Session result types.

use mew_core::Value;

/// Result of executing a statement.
#[derive(Debug, Clone)]
pub enum StatementResult {
    /// Query result with rows.
    Query(QueryResult),
    /// Mutation result with affected counts.
    Mutation(MutationSummary),
    /// Transaction control result.
    Transaction(TransactionResult),
    /// Empty result (for comments, etc.).
    Empty,
}

/// Result of a query execution.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Column names.
    pub columns: Vec<String>,
    /// Column types.
    pub types: Vec<String>,
    /// Data rows.
    pub rows: Vec<Vec<Value>>,
}

impl QueryResult {
    /// Create a new query result.
    pub fn new(columns: Vec<String>, types: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        Self {
            columns,
            types,
            rows,
        }
    }

    /// Create an empty query result.
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            types: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Get the number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Check if the result is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Summary of a mutation execution.
#[derive(Debug, Clone)]
pub struct MutationSummary {
    /// Number of nodes affected.
    pub nodes_affected: usize,
    /// Number of edges affected.
    pub edges_affected: usize,
    /// Any returned values.
    pub returned: Vec<Value>,
}

impl MutationSummary {
    /// Create a new mutation summary.
    pub fn new(nodes_affected: usize, edges_affected: usize) -> Self {
        Self {
            nodes_affected,
            edges_affected,
            returned: Vec::new(),
        }
    }

    /// Create with returned values.
    pub fn with_returned(mut self, returned: Vec<Value>) -> Self {
        self.returned = returned;
        self
    }

    /// Get total affected count.
    pub fn total_affected(&self) -> usize {
        self.nodes_affected + self.edges_affected
    }
}

/// Result of a transaction control statement.
#[derive(Debug, Clone)]
pub enum TransactionResult {
    /// Transaction began.
    Begun,
    /// Transaction committed.
    Committed,
    /// Transaction rolled back.
    RolledBack,
    /// Savepoint created.
    SavepointCreated { name: String },
    /// Rolled back to savepoint.
    RolledBackTo { name: String },
    /// Savepoint released.
    SavepointReleased { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_empty() {
        // GIVEN/WHEN
        let result = QueryResult::empty();

        // THEN
        assert!(result.is_empty());
        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn test_mutation_summary() {
        // GIVEN
        let result = MutationSummary::new(3, 5);

        // WHEN/THEN
        assert_eq!(result.total_affected(), 8);
    }
}
