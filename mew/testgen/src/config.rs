//! Configuration for test generation

/// Configuration for test generation
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Random seed for reproducibility
    pub seed: u64,
    /// Number of nodes to generate per type
    pub nodes_per_type: usize,
    /// Number of edges to generate per type
    pub edges_per_type: usize,
    /// Number of query test cases to generate
    pub query_count: usize,
    /// Number of mutation test cases to generate
    pub mutation_count: usize,
    /// Include negative tests (expected failures)
    pub include_negative_tests: bool,
    /// Maximum complexity to generate (0-100)
    pub max_complexity: u8,
    /// Include property-based tests
    pub include_property_tests: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            nodes_per_type: 5,
            edges_per_type: 3,
            query_count: 20,
            mutation_count: 10,
            include_negative_tests: true,
            max_complexity: 80,
            include_property_tests: true,
        }
    }
}

impl TestConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_nodes_per_type(mut self, count: usize) -> Self {
        self.nodes_per_type = count;
        self
    }

    pub fn with_edges_per_type(mut self, count: usize) -> Self {
        self.edges_per_type = count;
        self
    }

    pub fn with_query_count(mut self, count: usize) -> Self {
        self.query_count = count;
        self
    }

    pub fn with_mutation_count(mut self, count: usize) -> Self {
        self.mutation_count = count;
        self
    }

    pub fn minimal() -> Self {
        Self {
            seed: 42,
            nodes_per_type: 2,
            edges_per_type: 1,
            query_count: 5,
            mutation_count: 3,
            include_negative_tests: false,
            max_complexity: 50,
            include_property_tests: false,
        }
    }
}
