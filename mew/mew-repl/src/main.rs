//! MEW REPL - Read-Eval-Print-Loop for MEW hypergraph database.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use mew_compiler::compile;
use mew_core::Value;
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{parse_stmt, MatchStmt, Stmt, TxnStmt};
use mew_query::QueryExecutor;
use mew_registry::{Registry, RegistryBuilder};

/// REPL state
struct Repl {
    registry: Registry,
    graph: Graph,
    in_transaction: bool,
    verbose: bool,
}

impl Repl {
    fn new() -> Self {
        Self {
            registry: RegistryBuilder::new().build().unwrap(),
            graph: Graph::new(),
            in_transaction: false,
            verbose: false,
        }
    }

    fn load_ontology(&mut self, source: &str) -> Result<String, String> {
        // Compile directly from source
        let registry = compile(source).map_err(|e| format!("Compile error: {}", e))?;

        let type_count = registry.type_count();
        let edge_type_count = registry.edge_type_count();

        self.registry = registry;

        if self.verbose {
            self.print_registry_summary();
        }

        Ok(format!(
            "Ontology loaded: {} types, {} edge types",
            type_count, edge_type_count
        ))
    }

    fn print_registry_summary(&self) {
        let type_names: Vec<&str> = self.registry.all_types().map(|t| t.name.as_str()).collect();
        let edge_type_names: Vec<&str> = self
            .registry
            .all_edge_types()
            .map(|e| e.name.as_str())
            .collect();

        println!("  Types: {:?}", type_names);
        println!("  Edge types: {:?}", edge_type_names);
    }

    fn execute(&mut self, input: &str) -> Result<String, String> {
        let trimmed = input.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") {
            return Ok(String::new());
        }

        // Handle ontology loading (entire block)
        if trimmed.starts_with("ontology ") {
            return self.load_ontology(trimmed);
        }

        // Parse and execute statement
        let stmt = parse_stmt(trimmed).map_err(|e| format!("Parse error: {}", e))?;

        match stmt {
            Stmt::Match(ref match_stmt) => self.execute_match(match_stmt),
            Stmt::Spawn(ref spawn_stmt) => self.execute_spawn(spawn_stmt),
            Stmt::Kill(ref _kill_stmt) => {
                Err("KILL requires variable binding tracking (not yet implemented)".to_string())
            }
            Stmt::Link(ref _link_stmt) => {
                Err("LINK requires variable binding tracking (not yet implemented)".to_string())
            }
            Stmt::Unlink(ref _unlink_stmt) => {
                Err("UNLINK requires variable binding tracking (not yet implemented)".to_string())
            }
            Stmt::Set(ref _set_stmt) => {
                Err("SET requires variable binding tracking (not yet implemented)".to_string())
            }
            Stmt::Txn(ref txn_stmt) => self.execute_txn(txn_stmt),
            Stmt::Walk(_) => Ok("WALK not yet implemented".to_string()),
        }
    }

    fn execute_match(&self, stmt: &MatchStmt) -> Result<String, String> {
        let executor = QueryExecutor::new(&self.registry, &self.graph);
        let results = executor
            .execute_match(stmt)
            .map_err(|e| format!("Query error: {}", e))?;

        if results.is_empty() {
            return Ok("(no results)".to_string());
        }

        let mut output = String::new();

        // Header
        let columns = results.column_names();
        output.push_str(&columns.join(" | "));
        output.push('\n');
        output.push_str(&"-".repeat(columns.len() * 15));
        output.push('\n');

        // Rows
        for row in results.rows() {
            let values: Vec<String> = columns
                .iter()
                .map(|c| {
                    row.get_by_name(c)
                        .map(|v| format_value(v))
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect();
            output.push_str(&values.join(" | "));
            output.push('\n');
        }

        output.push_str(&format!("\n({} rows)", results.len()));

        Ok(output)
    }

    fn execute_spawn(&mut self, stmt: &mew_parser::SpawnStmt) -> Result<String, String> {
        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);
        let bindings = mew_pattern::Bindings::new();

        let result = executor
            .execute_spawn(stmt, &bindings)
            .map_err(|e| format!("Spawn error: {}", e))?;

        if let Some(node_id) = result.created_node() {
            Ok(format!("Created {} with id {}", stmt.var, node_id.raw()))
        } else {
            Ok("Spawn completed".to_string())
        }
    }

    fn execute_txn(&mut self, stmt: &TxnStmt) -> Result<String, String> {
        match stmt {
            TxnStmt::Begin { .. } => {
                if self.in_transaction {
                    return Err("Transaction already active".to_string());
                }
                self.in_transaction = true;
                Ok("BEGIN".to_string())
            }
            TxnStmt::Commit => {
                if !self.in_transaction {
                    return Err("No transaction active".to_string());
                }
                self.in_transaction = false;
                Ok("COMMIT".to_string())
            }
            TxnStmt::Rollback => {
                if !self.in_transaction {
                    return Err("No transaction active".to_string());
                }
                self.in_transaction = false;
                Ok("ROLLBACK".to_string())
            }
        }
    }

    fn run_file(&mut self, path: &Path) -> Result<(), String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        println!("Loading: {}", path.display());

        // Check if first meaningful line is a definition (node/edge/constraint/rule)
        // or a statement (SPAWN/MATCH/KILL/LINK/etc.)
        let first_meaningful = content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with("--"));

        let starts_with_def = first_meaningful
            .map(|l| l.starts_with("node ") || l.starts_with("edge ") || l.starts_with("constraint ") || l.starts_with("rule ") || l.starts_with("ontology "))
            .unwrap_or(false);

        // Check if it's a statement file (starts with a MEW statement keyword)
        let starts_with_stmt = first_meaningful
            .map(|l| {
                let upper = l.to_uppercase();
                upper.starts_with("SPAWN ") || upper.starts_with("MATCH ") ||
                upper.starts_with("KILL ") || upper.starts_with("LINK ") ||
                upper.starts_with("UNLINK ") || upper.starts_with("SET ") ||
                upper.starts_with("BEGIN") || upper.starts_with("COMMIT") ||
                upper.starts_with("ROLLBACK") || upper.starts_with("WALK ")
            })
            .unwrap_or(false);

        // Prefer checking content over extension - if it starts with statements, it's a script
        if starts_with_def && !starts_with_stmt {
            // It's an ontology definition - load entire content
            match self.load_ontology(&content) {
                Ok(msg) => println!("{}", msg),
                Err(e) => return Err(e),
            }
            return Ok(());
        }

        // Otherwise, execute as statements line by line
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            match self.execute(trimmed) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn interactive(&mut self) {
        println!("MEW REPL v0.1.0");
        println!("Type 'help' for commands, 'quit' to exit");
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Prompt
            let prompt = if self.in_transaction {
                "mew*> "
            } else {
                "mew> "
            };
            print!("{}", prompt);
            stdout.flush().unwrap();

            // Read line
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).unwrap() == 0 {
                break; // EOF
            }

            let trimmed = line.trim();

            // Handle special commands
            match trimmed.to_lowercase().as_str() {
                "quit" | "exit" | "\\q" => break,
                "help" | "\\h" => {
                    print_help();
                    continue;
                }
                "types" | "\\dt" => {
                    self.print_registry_summary();
                    continue;
                }
                "graph" | "\\dg" => {
                    println!("Nodes: {}", self.graph.node_count());
                    println!("Edges: {}", self.graph.edge_count());
                    continue;
                }
                "verbose" => {
                    self.verbose = !self.verbose;
                    println!("Verbose mode: {}", self.verbose);
                    continue;
                }
                _ => {}
            }

            // Handle file loading
            if trimmed.starts_with("\\i ") || trimmed.starts_with("load ") {
                let path = trimmed.split_whitespace().nth(1).unwrap_or("");
                if let Err(e) = self.run_file(Path::new(path)) {
                    eprintln!("Error: {}", e);
                }
                continue;
            }

            // Execute statement
            match self.execute(trimmed) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        println!("Goodbye!");
    }
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Timestamp(t) => format!("@{}", t),
        Value::Duration(d) => format!("{}ms", d),
        Value::NodeRef(id) => format!("node#{}", id.raw()),
        Value::EdgeRef(id) => format!("edge#{}", id.raw()),
    }
}

fn print_help() {
    println!("MEW REPL Commands:");
    println!("  \\i <file>      Load and execute a file");
    println!("  \\dt            Show types");
    println!("  \\dg            Show graph stats");
    println!("  verbose        Toggle verbose mode");
    println!("  help, \\h       Show this help");
    println!("  quit, \\q       Exit");
    println!();
    println!("MEW Statements:");
    println!("  MATCH ...      Query the graph");
    println!("  SPAWN ...      Create a node");
    println!("  LINK ...       Create an edge");
    println!("  KILL ...       Delete a node");
    println!("  UNLINK ...     Delete an edge");
    println!("  SET ...        Update attributes");
    println!("  BEGIN          Start transaction");
    println!("  COMMIT         Commit transaction");
    println!("  ROLLBACK       Rollback transaction");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut repl = Repl::new();

    if args.len() > 1 {
        // Run files from command line
        for arg in &args[1..] {
            if arg == "-v" || arg == "--verbose" {
                repl.verbose = true;
                continue;
            }

            if let Err(e) = repl.run_file(Path::new(arg)) {
                eprintln!("Error loading {}: {}", arg, e);
                std::process::exit(1);
            }
        }
    } else {
        // Interactive mode
        repl.interactive();
    }
}
