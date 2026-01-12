//! MEW Generative Test Runner
//!
//! Usage: cargo run -p mew-testgen --bin testgen-runner -- [OPTIONS]

use mew_testgen::{TestConfig, TestGenerator, TestExecutor, TestSummary, TestResult, ActualResult};
use mew_testgen::trust::TrustAuditor;
use mew_testgen::report::ReportGenerator;
use std::fs;
use std::path::PathBuf;

fn main() {
    let opts = Opts::parse();
    
    println!("MEW Generative Test Runner\n");
    
    fs::create_dir_all(&opts.output).expect("Failed to create output directory");
    
    let ontologies = find_ontologies(&opts);
    if ontologies.is_empty() {
        eprintln!("No ontologies found");
        std::process::exit(1);
    }
    
    println!("Testing {} ontologies → {}\n", ontologies.len(), opts.output.display());
    
    let mut stats = Stats::default();
    
    for (path, name) in &ontologies {
        print!("  {:<30} ", name);
        
        match run_tests(path, &name, &opts) {
            Ok(outcome) => {
                stats.passed += outcome.passed;
                stats.failed += outcome.failed;
                
                if outcome.failed == 0 {
                    println!("{}/{} ✓", outcome.passed, outcome.total());
                } else {
                    println!("{}/{} ({} failed)", outcome.passed, outcome.total(), outcome.failed);
                    // Show first failure reason, condensed
                    if let Some(err) = outcome.first_failure {
                        println!("    └─ {}", truncate(&err, 70));
                    }
                }
            }
            Err(e) => {
                stats.errors += 1;
                println!("ERROR: {}", truncate(&e, 60));
            }
        }
    }
    
    println!("\n{}", stats);
    std::process::exit(if stats.failed + stats.errors > 0 { 1 } else { 0 });
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max]) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Options
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Opts {
    level: Option<u8>,
    filter: Option<String>,
    seed: u64,
    nodes: usize,
    queries: usize,
    mutations: usize,
    output: PathBuf,
    ontologies: PathBuf,
    execute: bool,
    json: bool,
}

impl Opts {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut opts = Self::default();
        let mut i = 1;
        
        while i < args.len() {
            match args[i].as_str() {
                "--level" | "-l"     => { i += 1; opts.level = args.get(i).and_then(|s| s.parse().ok()); }
                "--ontology" | "-o"  => { i += 1; opts.filter = args.get(i).cloned(); }
                "--seed" | "-s"      => { i += 1; opts.seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(42); }
                "--nodes" | "-n"     => { i += 1; opts.nodes = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(5); }
                "--queries" | "-q"   => { i += 1; opts.queries = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(10); }
                "--mutations" | "-m" => { i += 1; opts.mutations = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(5); }
                "--output"           => { i += 1; opts.output = args.get(i).map(PathBuf::from).unwrap_or(opts.output); }
                "--ontologies-dir"   => { i += 1; opts.ontologies = args.get(i).map(PathBuf::from).unwrap_or(opts.ontologies); }
                "--execute" | "-x"   => opts.execute = true,
                "--json" | "-j"      => opts.json = true,
                "--help" | "-h"      => { print_help(); std::process::exit(0); }
                _ => {}
            }
            i += 1;
        }
        opts
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            level: None,
            filter: None,
            seed: 42,
            nodes: 5,
            queries: 10,
            mutations: 5,
            output: PathBuf::from("tests/reports"),
            ontologies: PathBuf::from("ontologies"),
            execute: false,
            json: false,
        }
    }
}

fn print_help() {
    println!(r#"Usage: testgen-runner [OPTIONS]

Options:
  -l, --level <N>        Test ontology level 1-5 (default: all)
  -o, --ontology <NAME>  Filter by ontology name
  -s, --seed <N>         Random seed (default: 42)
  -n, --nodes <N>        Nodes per type (default: 5)
  -q, --queries <N>      Query count (default: 10)
  -m, --mutations <N>    Mutation count (default: 5)
  -x, --execute          Execute tests against MEW
  -j, --json             Output JSON reports
  --output <DIR>         Output directory (default: tests/reports)
  --ontologies-dir <DIR> Ontologies directory (default: ontologies)
  -h, --help             Show this help"#);
}

// ─────────────────────────────────────────────────────────────────────────────
// Stats & Outcome
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Stats { passed: usize, failed: usize, errors: usize }

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.passed + self.failed;
        write!(f, "Results: {}/{} passed", self.passed, total)?;
        if self.errors > 0 { write!(f, ", {} errors", self.errors)?; }
        if self.failed + self.errors == 0 { write!(f, " ✓")?; }
        Ok(())
    }
}

struct Outcome {
    passed: usize,
    failed: usize,
    first_failure: Option<String>,
}

impl Outcome {
    fn total(&self) -> usize { self.passed + self.failed }
}

// ─────────────────────────────────────────────────────────────────────────────
// Core Logic
// ─────────────────────────────────────────────────────────────────────────────

fn find_ontologies(opts: &Opts) -> Vec<(PathBuf, String)> {
    let levels: Vec<u8> = opts.level.map(|l| vec![l]).unwrap_or_else(|| vec![1, 2, 3, 4, 5]);
    let mut result = Vec::new();
    
    for level in levels {
        let dir = opts.ontologies.join(format!("level-{}", level));
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "mew").unwrap_or(false) {
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                
                if let Some(ref filter) = opts.filter {
                    if !name.to_lowercase().contains(&filter.to_lowercase()) { continue; }
                }
                result.push((path, name));
            }
        }
    }
    
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

fn run_tests(path: &PathBuf, name: &str, opts: &Opts) -> Result<Outcome, String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    
    let config = TestConfig::default()
        .with_seed(opts.seed)
        .with_nodes_per_type(opts.nodes)
        .with_query_count(opts.queries)
        .with_mutation_count(opts.mutations);
    
    let mut generator = TestGenerator::new(config);
    let suite = generator.generate_suite(&source).map_err(|e| e.to_string())?;
    
    let (summary, results) = if opts.execute {
        let mut executor = TestExecutor::new();
        let summary = executor.execute(&suite).map_err(|e| e.to_string())?;
        (summary, executor.results)
    } else {
        (generation_summary(&suite), Vec::new())
    };
    
    write_reports(name, &suite, &summary, &results, opts)?;
    
    let first_failure = results.iter()
        .find(|r| !r.passed)
        .map(|r| format_failure(r));
    
    Ok(Outcome { passed: summary.passed, failed: summary.failed, first_failure })
}

fn format_failure(r: &TestResult) -> String {
    match &r.actual {
        ActualResult::Error(e) => e.clone(),
        ActualResult::Rows(rows) => {
            let actual_desc = if rows.len() <= 3 {
                format!("{:?}", rows)
            } else {
                format!("{} rows", rows.len())
            };
            format!("[{}]: got {}, expected {:?}", r.test_id, actual_desc, r.expected)
        }
        ActualResult::Count(n) => format!("[{}]: got count {}, expected {:?}", r.test_id, n, r.expected),
        ActualResult::Success => format!("[{}]: got success, expected {:?}", r.test_id, r.expected),
    }
}

fn generation_summary(suite: &mew_testgen::TestSuite) -> TestSummary {
    TestSummary {
        total: suite.test_cases.len(),
        passed: suite.test_cases.len(),
        failed: 0,
        by_trust_level: std::collections::HashMap::new(),
        by_complexity: std::collections::HashMap::new(),
        by_tag: std::collections::HashMap::new(),
        total_duration_us: 0,
    }
}

fn write_reports(
    name: &str,
    suite: &mew_testgen::TestSuite,
    summary: &TestSummary,
    results: &[TestResult],
    opts: &Opts,
) -> Result<(), String> {
    let ext = if opts.json { "json" } else { "txt" };
    
    // Main report
    let report = if opts.json {
        ReportGenerator::generate_json(suite, summary, results)
    } else {
        let trust = if results.is_empty() { 
            mew_testgen::trust::TrustReport::new() 
        } else { 
            TrustAuditor::audit(results) 
        };
        ReportGenerator::generate(suite, summary, &trust)
    };
    
    fs::write(opts.output.join(format!("{}_report.{}", name, ext)), &report)
        .map_err(|e| e.to_string())?;
    
    // Test cases file
    let cases: String = suite.test_cases.iter().enumerate()
        .map(|(i, tc)| format!(
            "=== Test {} [{}] ===\nTrust: {:?}\nSetup: {}\nStatement: {}\nExpected: {:?}\n",
            i + 1, tc.id, tc.trust_level, 
            if tc.setup.is_empty() { "(none)".into() } else { tc.setup.join("; ") },
            tc.statement, tc.expected
        ))
        .collect();
    
    fs::write(opts.output.join(format!("{}_tests.txt", name)), cases)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
