//! Session result types.

use mew_core::Value;

/// Result of executing a statement.
#[derive(Debug, Clone)]
pub enum StatementResult {
    /// Query result with rows.
    Query(QueryResult),
    /// Mutation result with affected counts.
    Mutation(MutationSummary),
    /// Mixed result containing both mutations and queries.
    Mixed {
        mutations: MutationSummary,
        queries: QueryResult,
    },
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
#[derive(Debug, Clone, Default)]
pub struct MutationSummary {
    /// Number of nodes created.
    pub nodes_created: usize,
    /// Number of nodes modified.
    pub nodes_modified: usize,
    /// Number of nodes deleted.
    pub nodes_deleted: usize,
    /// Number of edges created.
    pub edges_created: usize,
    /// Number of edges modified.
    pub edges_modified: usize,
    /// Number of edges deleted.
    pub edges_deleted: usize,
    /// Any returned values.
    pub returned: Vec<Value>,
}

impl MutationSummary {
    /// Create a new mutation summary (legacy compat: nodes/edges affected).
    pub fn new(nodes_affected: usize, edges_affected: usize) -> Self {
        Self {
            nodes_created: nodes_affected,
            nodes_modified: 0,
            nodes_deleted: 0,
            edges_created: edges_affected,
            edges_modified: 0,
            edges_deleted: 0,
            returned: Vec::new(),
        }
    }

    /// Create a summary for node creation.
    pub fn created_nodes(count: usize) -> Self {
        Self {
            nodes_created: count,
            ..Default::default()
        }
    }

    /// Create a summary for node modification.
    pub fn modified_nodes(count: usize) -> Self {
        Self {
            nodes_modified: count,
            ..Default::default()
        }
    }

    /// Create a summary for node deletion.
    pub fn deleted_nodes(count: usize) -> Self {
        Self {
            nodes_deleted: count,
            ..Default::default()
        }
    }

    /// Create a summary for edge creation.
    pub fn created_edges(count: usize) -> Self {
        Self {
            edges_created: count,
            ..Default::default()
        }
    }

    /// Create a summary for edge deletion.
    pub fn deleted_edges(count: usize) -> Self {
        Self {
            edges_deleted: count,
            ..Default::default()
        }
    }

    /// Create with returned values.
    pub fn with_returned(mut self, returned: Vec<Value>) -> Self {
        self.returned = returned;
        self
    }

    /// Total nodes affected.
    pub fn nodes_affected(&self) -> usize {
        self.nodes_created + self.nodes_modified + self.nodes_deleted
    }

    /// Total edges affected.
    pub fn edges_affected(&self) -> usize {
        self.edges_created + self.edges_deleted
    }

    /// Get total affected count.
    pub fn total_affected(&self) -> usize {
        self.nodes_affected() + self.edges_affected()
    }

    /// Merge another summary into this one.
    pub fn merge(&mut self, other: &MutationSummary) {
        self.nodes_created += other.nodes_created;
        self.nodes_modified += other.nodes_modified;
        self.nodes_deleted += other.nodes_deleted;
        self.edges_created += other.edges_created;
        self.edges_deleted += other.edges_deleted;
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
