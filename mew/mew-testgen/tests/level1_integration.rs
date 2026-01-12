//! Integration tests for level-1 ontologies
//!
//! These tests verify the test generation framework against real ontology files.

use mew_testgen::{QueryGenerator, SchemaAnalyzer, TestConfig, TestGenerator, WorldGenerator};
use std::fs;
use std::path::Path;

// Helper to find ontology files in a level
fn find_ontologies(level: u8) -> Vec<(String, String)> {
    let level_dir = format!("../ontologies/level-{}", level);
    let path = Path::new(&level_dir);

    if !path.exists() {
        return Vec::new();
    }

    let mut ontologies = Vec::new();
    for entry in fs::read_dir(path).unwrap().flatten() {
        let file_path = entry.path();
        if file_path.extension().map(|e| e == "mew").unwrap_or(false) {
            let name = file_path.file_name().unwrap().to_string_lossy().to_string();
            let source = fs::read_to_string(&file_path).unwrap();
            ontologies.push((name, source));
        }
    }
    ontologies
}

#[test]
fn test_schema_analysis_level1() {
    for (name, source) in find_ontologies(1) {
        let result = SchemaAnalyzer::analyze(&source);
        assert!(
            result.is_ok(),
            "Failed to analyze {}: {:?}",
            name,
            result.err()
        );

        let schema = result.unwrap();
        assert!(!schema.node_types.is_empty(), "No node types in {}", name);
    }
}

#[test]
fn test_world_generation_level1() {
    use rand::SeedableRng;

    for (name, source) in find_ontologies(1) {
        let schema = SchemaAnalyzer::analyze(&source).unwrap();
        let config = TestConfig::default().with_nodes_per_type(3);
        let mut gen = WorldGenerator::new(&schema, config);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let result = gen.generate(&mut rng);
        assert!(
            result.is_ok(),
            "Failed to generate world for {}: {:?}",
            name,
            result.err()
        );

        let world = result.unwrap();
        assert!(!world.nodes.is_empty(), "No nodes generated for {}", name);
    }
}

#[test]
fn test_query_generation_level1() {
    use rand::SeedableRng;

    for (name, source) in find_ontologies(1) {
        let schema = SchemaAnalyzer::analyze(&source).unwrap();
        let config = TestConfig::default().with_nodes_per_type(3);
        let mut gen = WorldGenerator::new(&schema, config);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let world = gen.generate(&mut rng).unwrap();

        let mut query_gen = QueryGenerator::new(&schema, &world);
        let result = query_gen.generate(5, &mut rng);
        assert!(
            result.is_ok(),
            "Failed to generate queries for {}: {:?}",
            name,
            result.err()
        );

        let queries = result.unwrap();
        assert_eq!(queries.len(), 5, "Wrong query count for {}", name);
    }
}

#[test]
fn test_full_suite_generation_level1() {
    for (name, source) in find_ontologies(1) {
        let config = TestConfig::default()
            .with_seed(42)
            .with_nodes_per_type(3)
            .with_query_count(5)
            .with_mutation_count(3);

        let mut generator = TestGenerator::new(config);
        let result = generator.generate_suite(&source);

        assert!(
            result.is_ok(),
            "Failed to generate suite for {}: {:?}",
            name,
            result.err()
        );

        let suite = result.unwrap();
        assert!(!suite.test_cases.is_empty(), "No test cases for {}", name);

        // Should have queries
        let query_count = suite
            .test_cases
            .iter()
            .filter(|t| t.tags.contains(&"match".to_string()))
            .count();
        assert!(query_count > 0, "No queries for {}", name);
    }
}

#[test]
fn test_all_levels_parse() {
    // Test that all ontologies can at least be parsed
    for level in 1..=5 {
        for (name, source) in find_ontologies(level) {
            let result = SchemaAnalyzer::analyze(&source);
            assert!(
                result.is_ok(),
                "Level {} - Failed to analyze {}: {:?}",
                level,
                name,
                result.err()
            );
        }
    }
}
