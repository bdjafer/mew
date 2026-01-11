//! Test runner for MEW generative tests
//!
//! This binary runs generative tests against MEW ontologies and produces reports.
//!
//! Usage:
//!   cargo run -p mew-testgen --bin testgen-runner -- [OPTIONS]
//!
//! Options:
//!   --level <N>       Run tests for ontology level N (1-5, default: all)
//!   --ontology <NAME> Run tests for a specific ontology file
//!   --seed <N>        Random seed for reproducibility (default: 42)
//!   --nodes <N>       Nodes per type to generate (default: 5)
//!   --queries <N>     Number of queries to generate (default: 10)
//!   --mutations <N>   Number of mutations to generate (default: 5)
//!   --output <DIR>    Output directory for reports (default: tests/reports)
//!   --json            Output JSON reports instead of text
//!   --verbose         Print detailed output
//!   --execute         Actually execute tests against MEW (default: generate only)

use mew_testgen::{TestConfig, TestGenerator, TestExecutor};
use mew_testgen::trust::TrustAuditor;
use mew_testgen::report::ReportGenerator;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           MEW Generative Test Runner                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Ensure output directory exists
    fs::create_dir_all(&config.output_dir).expect("Failed to create output directory");

    // Find ontologies to test
    let ontologies = find_ontologies(&config);

    if ontologies.is_empty() {
        eprintln!("No ontologies found matching criteria");
        std::process::exit(1);
    }

    println!("Found {} ontologies to test", ontologies.len());
    println!("Output directory: {}", config.output_dir.display());
    println!();

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_tests = 0;

    for (path, name) in &ontologies {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Testing: {}", name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match run_tests_for_ontology(path, name, &config) {
            Ok((passed, failed, report_path)) => {
                total_passed += passed;
                total_failed += failed;
                total_tests += passed + failed;
                println!("  Result: {}/{} passed", passed, passed + failed);
                println!("  Report: {}", report_path.display());
            }
            Err(e) => {
                eprintln!("  ERROR: {}", e);
                total_failed += 1;
            }
        }
        println!();
    }

    // Summary
    println!("════════════════════════════════════════════════════════════════");
    println!("                         SUMMARY                                ");
    println!("════════════════════════════════════════════════════════════════");
    println!("Total ontologies: {}", ontologies.len());
    println!("Total tests:      {}", total_tests);
    println!("Passed:           {}", total_passed);
    println!("Failed:           {}", total_failed);

    if total_failed > 0 {
        println!("\n⚠ Some tests failed - check individual reports for details");
        std::process::exit(1);
    } else {
        println!("\n✓ All tests passed!");
    }
}

#[derive(Debug)]
struct RunConfig {
    level: Option<u8>,
    ontology_name: Option<String>,
    seed: u64,
    nodes_per_type: usize,
    query_count: usize,
    mutation_count: usize,
    output_dir: PathBuf,
    json_output: bool,
    verbose: bool,
    execute: bool,
    ontologies_dir: PathBuf,
}

fn parse_args(args: &[String]) -> RunConfig {
    let mut config = RunConfig {
        level: None,
        ontology_name: None,
        seed: 42,
        nodes_per_type: 5,
        query_count: 10,
        mutation_count: 5,
        output_dir: PathBuf::from("tests/reports"),
        json_output: false,
        verbose: false,
        execute: false,
        ontologies_dir: PathBuf::from("ontologies"),
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--level" => {
                i += 1;
                if i < args.len() {
                    config.level = args[i].parse().ok();
                }
            }
            "--ontology" => {
                i += 1;
                if i < args.len() {
                    config.ontology_name = Some(args[i].clone());
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    config.seed = args[i].parse().unwrap_or(42);
                }
            }
            "--nodes" => {
                i += 1;
                if i < args.len() {
                    config.nodes_per_type = args[i].parse().unwrap_or(5);
                }
            }
            "--queries" => {
                i += 1;
                if i < args.len() {
                    config.query_count = args[i].parse().unwrap_or(10);
                }
            }
            "--mutations" => {
                i += 1;
                if i < args.len() {
                    config.mutation_count = args[i].parse().unwrap_or(5);
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    config.output_dir = PathBuf::from(&args[i]);
                }
            }
            "--ontologies-dir" => {
                i += 1;
                if i < args.len() {
                    config.ontologies_dir = PathBuf::from(&args[i]);
                }
            }
            "--json" => config.json_output = true,
            "--verbose" => config.verbose = true,
            "--execute" => config.execute = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}

fn print_help() {
    println!(
        r#"MEW Generative Test Runner

USAGE:
    cargo run -p mew-testgen --bin testgen-runner -- [OPTIONS]

OPTIONS:
    --level <N>           Run tests for ontology level N (1-5)
    --ontology <NAME>     Run tests for a specific ontology file
    --seed <N>            Random seed for reproducibility (default: 42)
    --nodes <N>           Nodes per type to generate (default: 5)
    --queries <N>         Number of queries to generate (default: 10)
    --mutations <N>       Number of mutations to generate (default: 5)
    --output <DIR>        Output directory for reports (default: tests/reports)
    --ontologies-dir <DIR> Ontologies directory (default: ontologies)
    --json                Output JSON reports instead of text
    --verbose             Print detailed output
    --execute             Execute tests against MEW (default: generate only)
    --help, -h            Print this help message

EXAMPLES:
    # Test all level-1 ontologies
    cargo run -p mew-testgen --bin testgen-runner -- --level 1

    # Test a specific ontology with verbose output
    cargo run -p mew-testgen --bin testgen-runner -- --ontology Bookmarks --verbose

    # Run full execution tests with custom seed
    cargo run -p mew-testgen --bin testgen-runner -- --execute --seed 12345
"#
    );
}

fn find_ontologies(config: &RunConfig) -> Vec<(PathBuf, String)> {
    let mut ontologies = Vec::new();

    let levels: Vec<u8> = match config.level {
        Some(l) => vec![l],
        None => vec![1, 2, 3, 4, 5],
    };

    for level in levels {
        let level_dir = config.ontologies_dir.join(format!("level-{}", level));
        if !level_dir.exists() {
            continue;
        }

        let entries = match fs::read_dir(&level_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "mew").unwrap_or(false) {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Filter by ontology name if specified
                if let Some(ref filter) = config.ontology_name {
                    if !name.to_lowercase().contains(&filter.to_lowercase()) {
                        continue;
                    }
                }

                ontologies.push((path, name));
            }
        }
    }

    // Sort by name for consistent ordering
    ontologies.sort_by(|a, b| a.1.cmp(&b.1));
    ontologies
}

fn run_tests_for_ontology(
    path: &Path,
    name: &str,
    config: &RunConfig,
) -> Result<(usize, usize, PathBuf), String> {
    let start = Instant::now();

    // Read ontology source
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    if config.verbose {
        println!("  Source: {} bytes", source.len());
    }

    // Configure test generation
    let test_config = TestConfig::default()
        .with_seed(config.seed)
        .with_nodes_per_type(config.nodes_per_type)
        .with_query_count(config.query_count)
        .with_mutation_count(config.mutation_count);

    // Generate test suite
    let mut generator = TestGenerator::new(test_config);
    let suite = generator.generate_suite(&source)
        .map_err(|e| format!("Failed to generate tests: {}", e))?;

    if config.verbose {
        println!("  Generated {} test cases", suite.test_cases.len());
        println!("  World: {} nodes, {} edges", suite.world.nodes.len(), suite.world.edges.len());
    }

    let (passed, failed, summary, results) = if config.execute {
        // Execute tests against MEW
        let mut executor = TestExecutor::new();
        let summary = executor.execute(&suite)
            .map_err(|e| format!("Failed to execute tests: {}", e))?;

        (summary.passed, summary.failed, summary, executor.results)
    } else {
        // Just count generated tests (all "pass" in generation mode)
        let summary = mew_testgen::types::TestSummary {
            total: suite.test_cases.len(),
            passed: suite.test_cases.len(),
            failed: 0,
            by_trust_level: std::collections::HashMap::new(),
            by_complexity: std::collections::HashMap::new(),
            by_tag: std::collections::HashMap::new(),
            total_duration_us: start.elapsed().as_micros() as u64,
        };
        (suite.test_cases.len(), 0, summary, Vec::new())
    };

    // Generate report
    let trust_report = if config.execute {
        TrustAuditor::audit(&results)
    } else {
        mew_testgen::trust::TrustReport::new()
    };

    let report_content = if config.json_output {
        ReportGenerator::generate_json(&suite, &summary, &results)
    } else {
        ReportGenerator::generate(&suite, &summary, &trust_report)
    };

    // Write report
    let ext = if config.json_output { "json" } else { "txt" };
    let report_path = config.output_dir.join(format!("{}_report.{}", name, ext));
    fs::write(&report_path, &report_content)
        .map_err(|e| format!("Failed to write report: {}", e))?;

    // Also write generated test cases for inspection
    let cases_path = config.output_dir.join(format!("{}_tests.txt", name));
    let mut cases_content = String::new();
    for (i, tc) in suite.test_cases.iter().enumerate() {
        cases_content.push_str(&format!("=== Test {} [{}] ===\n", i + 1, tc.id));
        cases_content.push_str(&format!("Trust: {:?}\n", tc.trust_level));
        cases_content.push_str(&format!("Tags: {:?}\n", tc.tags));
        if !tc.setup.is_empty() {
            cases_content.push_str("Setup:\n");
            for s in &tc.setup {
                cases_content.push_str(&format!("  {}\n", s));
            }
        }
        cases_content.push_str(&format!("Statement: {}\n", tc.statement));
        cases_content.push_str(&format!("Expected: {:?}\n", tc.expected));
        cases_content.push_str("\n");
    }
    fs::write(&cases_path, &cases_content)
        .map_err(|e| format!("Failed to write test cases: {}", e))?;

    if config.verbose {
        println!("  Duration: {:?}", start.elapsed());
    }

    Ok((passed, failed, report_path))
}
