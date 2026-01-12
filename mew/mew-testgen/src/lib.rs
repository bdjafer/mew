//! Generative Testing Framework for MEW
//!
//! This crate implements a schema-aware, constructive testing framework that:
//! - Analyzes ontology schemas to understand structure
//! - Generates valid world states (nodes, edges, attributes)
//! - Synthesizes queries and mutations with expected results
//! - Provides trust levels for test verification
//! - Reports comprehensive test results

pub mod config;
pub mod execution;
pub mod mutation;
pub mod oracle;
pub mod query;
pub mod report;
pub mod schema;
pub mod trust;
pub mod types;
pub mod world;

pub use config::TestConfig;
pub use execution::TestExecutor;
pub use mutation::MutationGenerator;
pub use oracle::Oracle;
pub use query::QueryGenerator;
pub use report::ReportGenerator;
pub use schema::SchemaAnalyzer;
pub use trust::TrustAuditor;
pub use types::*;
pub use world::WorldGenerator;

use rand::rngs::StdRng;
use rand::SeedableRng;

/// Main entry point for the test generator
pub struct TestGenerator {
    pub config: TestConfig,
    pub rng: StdRng,
}

impl TestGenerator {
    pub fn new(config: TestConfig) -> Self {
        let rng = StdRng::seed_from_u64(config.seed);
        Self { config, rng }
    }

    /// Generate a complete test suite for an ontology
    pub fn generate_suite(&mut self, ontology_source: &str) -> Result<TestSuite, TestGenError> {
        // 1. Analyze schema
        let schema = SchemaAnalyzer::analyze(ontology_source)?;

        // 2. Generate world state
        let mut world_gen = WorldGenerator::new(&schema, self.config.clone());
        let world = world_gen.generate(&mut self.rng)?;

        // 3. Generate queries
        let mut query_gen = QueryGenerator::new(&schema, &world);
        let queries = query_gen.generate(self.config.query_count, &mut self.rng)?;

        // 4. Generate mutations
        let mut mutation_gen = MutationGenerator::new(&schema, &world);
        let mutations = mutation_gen.generate(self.config.mutation_count, &mut self.rng)?;

        // 5. Combine into test cases
        let mut test_cases = Vec::new();

        for (i, query) in queries.into_iter().enumerate() {
            test_cases.push(TestCase {
                id: format!("query_{}", i),
                statement: query.statement,
                setup: query.required_setup,
                expected: query.expected,
                trust_level: query.trust_level,
                complexity: query.complexity,
                tags: query.tags,
            });
        }

        for (i, mutation) in mutations.into_iter().enumerate() {
            test_cases.push(TestCase {
                id: format!("mutation_{}", i),
                statement: mutation.statement,
                setup: mutation.required_setup,
                expected: mutation.expected,
                trust_level: mutation.trust_level,
                complexity: mutation.complexity,
                tags: mutation.tags,
            });
        }

        Ok(TestSuite {
            ontology_source: ontology_source.to_string(),
            schema,
            world,
            test_cases,
            seed: self.config.seed,
        })
    }
}
