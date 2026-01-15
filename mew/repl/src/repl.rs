//! Core REPL state and execution.

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use mew_compiler::compile;
use mew_core::EntityId;
use mew_graph::Graph;
use mew_parser::{parse_stmt, Stmt};
use mew_registry::{Registry, RegistryBuilder};

use crate::block::{
    collect_block_from_lines, collect_block_from_stdin, extract_ontology_source,
    should_continue_parse,
};
use crate::executor::{
    execute_explain, execute_inspect, execute_kill, execute_link, execute_match, execute_match_mutate,
    execute_match_walk, execute_profile, execute_set, execute_spawn, execute_txn, execute_unlink, execute_walk,
};
use crate::format::print_help;

/// REPL state.
pub struct Repl {
    registry: Registry,
    graph: Graph,
    in_transaction: bool,
    verbose: bool,
    bindings: HashMap<String, EntityId>,
}

impl Repl {
    /// Create a new REPL instance.
    pub fn new() -> Self {
        Self {
            registry: RegistryBuilder::new().build().unwrap(),
            graph: Graph::new(),
            in_transaction: false,
            verbose: false,
            bindings: HashMap::new(),
        }
    }

    /// Set verbose mode.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Load an ontology from source.
    pub fn load_ontology(&mut self, source: &str) -> Result<String, String> {
        let registry = compile(source).map_err(|e| format!("Compile error: {}", e))?;

        let type_count = registry.type_count();
        let edge_type_count = registry.edge_type_count();

        self.registry = registry;
        self.graph = Graph::new();
        self.bindings.clear();

        if self.verbose {
            self.print_registry_summary();
        }

        Ok(format!(
            "Ontology loaded: {} types, {} edge types",
            type_count, edge_type_count
        ))
    }

    /// Print a summary of the registry.
    pub fn print_registry_summary(&self) {
        let type_names: Vec<&str> = self.registry.all_types().map(|t| t.name.as_str()).collect();
        let edge_type_names: Vec<&str> = self
            .registry
            .all_edge_types()
            .map(|e| e.name.as_str())
            .collect();

        println!("  Types: {:?}", type_names);
        println!("  Edge types: {:?}", edge_type_names);
    }

    /// Print graph statistics.
    pub fn print_graph_stats(&self) {
        println!("Nodes: {}", self.graph.node_count());
        println!("Edges: {}", self.graph.edge_count());
    }

    /// Toggle verbose mode.
    pub fn toggle_verbose(&mut self) {
        self.verbose = !self.verbose;
        println!("Verbose mode: {}", self.verbose);
    }

    /// Execute a statement or command.
    pub fn execute(&mut self, input: &str) -> Result<String, String> {
        let trimmed = input.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("--") {
            return Ok(String::new());
        }

        // Handle ontology loading (entire block)
        if trimmed.to_lowercase().starts_with("load ontology") {
            let source = extract_ontology_source(trimmed)?;
            return self.load_ontology(&source);
        }
        if trimmed.starts_with("ontology ") {
            return self.load_ontology(trimmed);
        }

        // Parse and execute statement
        let stmt = parse_stmt(trimmed).map_err(|e| format!("Parse error: {}", e))?;

        match stmt {
            Stmt::Match(ref match_stmt) => {
                execute_match(&self.registry, &self.graph, &self.bindings, match_stmt)
            }
            Stmt::Spawn(ref spawn_stmt) => execute_spawn(
                &self.registry,
                &mut self.graph,
                &mut self.bindings,
                spawn_stmt,
            ),
            Stmt::Kill(ref kill_stmt) => execute_kill(
                &self.registry,
                &mut self.graph,
                &mut self.bindings,
                kill_stmt,
            ),
            Stmt::Link(ref link_stmt) => execute_link(
                &self.registry,
                &mut self.graph,
                &mut self.bindings,
                link_stmt,
            ),
            Stmt::Unlink(ref unlink_stmt) => execute_unlink(
                &self.registry,
                &mut self.graph,
                &mut self.bindings,
                unlink_stmt,
            ),
            Stmt::Set(ref set_stmt) => {
                execute_set(&self.registry, &mut self.graph, &self.bindings, set_stmt)
            }
            Stmt::Txn(ref txn_stmt) => execute_txn(&mut self.in_transaction, txn_stmt),
            Stmt::Walk(ref walk_stmt) => {
                execute_walk(&self.registry, &self.graph, &self.bindings, walk_stmt)
            }
            Stmt::Inspect(ref inspect_stmt) => {
                execute_inspect(&self.registry, &self.graph, inspect_stmt)
            }
            Stmt::MatchMutate(ref match_mutate_stmt) => execute_match_mutate(
                &self.registry,
                &mut self.graph,
                &mut self.bindings,
                match_mutate_stmt,
            ),
            Stmt::MatchWalk(ref match_walk_stmt) => execute_match_walk(
                &self.registry,
                &self.graph,
                &self.bindings,
                match_walk_stmt,
            ),
            Stmt::Explain(ref explain_stmt) => {
                execute_explain(&self.registry, &self.graph, explain_stmt)
            }
            Stmt::Profile(ref profile_stmt) => {
                execute_profile(&self.registry, &mut self.graph, profile_stmt)
            }
            Stmt::Prepare(_) | Stmt::Execute(_) | Stmt::DropPrepared(_) => {
                // Prepared statements require session context
                Err("Prepared statements (PREPARE/EXECUTE/DROP PREPARED) require session mode".into())
            }
        }
    }

    /// Run a file.
    pub fn run_file(&mut self, path: &Path) -> Result<(), String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        println!("Loading: {}", path.display());

        self.run_script(&content)
    }

    /// Run a script (series of statements).
    pub fn run_script(&mut self, content: &str) -> Result<(), String> {
        // Check if first meaningful line is a definition (node/edge/constraint/rule)
        // or a statement (SPAWN/MATCH/KILL/LINK/etc.)
        let first_meaningful = content
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with("--"));

        let starts_with_def = first_meaningful
            .map(|l| {
                l.starts_with("node ")
                    || l.starts_with("edge ")
                    || l.starts_with("constraint ")
                    || l.starts_with("rule ")
                    || l.starts_with("ontology ")
            })
            .unwrap_or(false);

        // Check if it's a statement file (starts with a MEW statement keyword)
        let starts_with_stmt = first_meaningful
            .map(|l| {
                let upper = l.to_uppercase();
                upper.starts_with("SPAWN ")
                    || upper.starts_with("MATCH ")
                    || upper.starts_with("KILL ")
                    || upper.starts_with("LINK ")
                    || upper.starts_with("UNLINK ")
                    || upper.starts_with("SET ")
                    || upper.starts_with("BEGIN")
                    || upper.starts_with("COMMIT")
                    || upper.starts_with("ROLLBACK")
                    || upper.starts_with("WALK ")
            })
            .unwrap_or(false);

        // Prefer checking content over extension - if it starts with statements, it's a script
        if starts_with_def && !starts_with_stmt {
            // It's an ontology definition - load entire content
            match self.load_ontology(content) {
                Ok(msg) => println!("{}", msg),
                Err(e) => return Err(e),
            }
            return Ok(());
        }

        // Otherwise, execute as statements line by line (with buffering)
        let mut buffer = String::new();
        let mut pending_match = false;
        let mut lines = content.lines().map(|line| line.to_string()).peekable();

        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            if pending_match && trimmed.to_uppercase().starts_with("ORDER BY") {
                buffer.push_str(&line);
                buffer.push('\n');
                pending_match = false;
                match parse_stmt(&buffer) {
                    Ok(_) => {
                        let stmt_text = buffer.trim().to_string();
                        buffer.clear();
                        match self.execute(&stmt_text) {
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
                    Err(e) => {
                        let err = format!("{}", e);
                        buffer.clear();
                        eprintln!("Error: {}", err);
                    }
                }
                continue;
            }

            if pending_match {
                let stmt_text = buffer.trim().to_string();
                buffer.clear();
                pending_match = false;
                match self.execute(&stmt_text) {
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

            if trimmed.to_lowercase().starts_with("load ontology") {
                let block = collect_block_from_lines(&line, &mut lines)?;
                let source = extract_ontology_source(&block)?;
                match self.load_ontology(&source) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
                continue;
            }

            if trimmed.starts_with("ontology ") {
                let block = collect_block_from_lines(&line, &mut lines)?;
                match self.load_ontology(&block) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => return Err(e),
                }
                continue;
            }

            buffer.push_str(&line);
            buffer.push('\n');

            match parse_stmt(&buffer) {
                Ok(stmt) => {
                    if let Stmt::Match(match_stmt) = stmt {
                        if match_stmt.order_by.is_none() {
                            pending_match = true;
                            continue;
                        }
                    }
                    let stmt_text = buffer.trim().to_string();
                    buffer.clear();
                    match self.execute(&stmt_text) {
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
                Err(e) => {
                    let err = format!("{}", e);
                    if !should_continue_parse(&err) {
                        buffer.clear();
                        pending_match = false;
                        eprintln!("Error: {}", err);
                    }
                }
            }
        }

        if pending_match && !buffer.trim().is_empty() {
            let stmt_text = buffer.trim().to_string();
            match self.execute(&stmt_text) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        } else if !buffer.trim().is_empty() {
            let err = parse_stmt(&buffer).map_err(|e| format!("{}", e));
            if let Err(e) = err {
                eprintln!("Error: {}", e);
            }
        }

        Ok(())
    }

    /// Run the interactive REPL.
    pub fn interactive(&mut self) {
        println!("MEW REPL v0.1.0");
        println!("Type 'help' for commands, 'quit' to exit");
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut buffer = String::new();
        let mut pending_match = false;

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
                    self.print_graph_stats();
                    continue;
                }
                "verbose" => {
                    self.toggle_verbose();
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

            if trimmed.to_lowercase().starts_with("load ontology") {
                let block = match collect_block_from_stdin(&line, &stdin) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                };
                let source = match extract_ontology_source(&block) {
                    Ok(source) => source,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                };
                match self.load_ontology(&source) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("Error: {}", e),
                }
                continue;
            }

            if trimmed.starts_with("ontology ") {
                let block = match collect_block_from_stdin(&line, &stdin) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                };
                match self.load_ontology(&block) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => eprintln!("Error: {}", e),
                }
                continue;
            }

            if pending_match && trimmed.to_uppercase().starts_with("ORDER BY") {
                buffer.push_str(trimmed);
                buffer.push('\n');
                pending_match = false;
                let stmt_text = buffer.trim().to_string();
                buffer.clear();
                match self.execute(&stmt_text) {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
                continue;
            }

            if pending_match {
                let stmt_text = buffer.trim().to_string();
                buffer.clear();
                pending_match = false;
                match self.execute(&stmt_text) {
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

            // Execute statement (with buffering)
            buffer.push_str(trimmed);
            buffer.push('\n');

            loop {
                match parse_stmt(&buffer) {
                    Ok(stmt) => {
                        if let Stmt::Match(match_stmt) = stmt {
                            if match_stmt.order_by.is_none() {
                                pending_match = true;
                                break;
                            }
                        }
                        let stmt_text = buffer.trim().to_string();
                        buffer.clear();
                        match self.execute(&stmt_text) {
                            Ok(output) => {
                                if !output.is_empty() {
                                    println!("{}", output);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                        break;
                    }
                    Err(e) => {
                        let err = format!("{}", e);
                        if should_continue_parse(&err) {
                            print!("....> ");
                            stdout.flush().unwrap();
                            let mut continuation = String::new();
                            if stdin.lock().read_line(&mut continuation).unwrap() == 0 {
                                break;
                            }
                            buffer.push_str(continuation.trim_end());
                            buffer.push('\n');
                            continue;
                        }
                        buffer.clear();
                        pending_match = false;
                        eprintln!("Error: {}", err);
                        break;
                    }
                }
            }
        }

        println!("Goodbye!");
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_ONTOLOGY: &str = r#"
        ontology Test {
            node Task {
                title: String
            }
            node Person {
                name: String
            }
            edge assigned(task: Task, person: Person)
        }
    "#;

    #[test]
    fn loads_ontology_with_summary() {
        let mut repl = Repl::new();
        let output = repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();
        assert_eq!(output, "Ontology loaded: 2 types, 1 edge types");
    }

    #[test]
    fn reports_empty_match_results() {
        let mut repl = Repl::new();
        repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();

        let output = repl.execute("MATCH t: Task RETURN t").unwrap();
        assert_eq!(output, "(no results)");
    }

    #[test]
    fn surfaces_parse_errors() {
        let mut repl = Repl::new();
        repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();

        let err = repl.execute("MATC t: Task").unwrap_err();
        assert!(err.starts_with("Parse error:"));
    }

    #[test]
    fn supports_query_and_mutation_feedback() {
        let mut repl = Repl::new();
        repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();

        let spawn_output = repl.execute("SPAWN t: Task { title = \"Hello\" }").unwrap();
        assert!(spawn_output.starts_with("Created t with id "));

        let match_output = repl.execute("MATCH t: Task RETURN t").unwrap();
        assert!(match_output.contains("t"));
        assert!(match_output.contains("node#"));
        assert!(match_output.contains("(1 rows)"));
    }

    #[test]
    fn supports_mutations_links_and_transactions() {
        let mut repl = Repl::new();
        repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();

        let begin_output = repl.execute("BEGIN").unwrap();
        assert_eq!(begin_output, "BEGIN");

        repl.execute("SPAWN t: Task { title = \"Hello\" }").unwrap();
        repl.execute("SPAWN p: Person { name = \"Ada\" }").unwrap();

        let link_output = repl.execute("LINK e: assigned(t, p)").unwrap();
        assert_eq!(link_output, "Link created");

        let set_output = repl.execute("SET t { title = \"Updated\" }").unwrap();
        assert_eq!(set_output, "Updated 1 nodes");

        let unlink_output = repl.execute("UNLINK e").unwrap();
        assert_eq!(unlink_output, "Deleted 1 edges");

        let kill_output = repl.execute("KILL t").unwrap();
        assert_eq!(kill_output, "Deleted 1 nodes, 0 edges");

        let commit_output = repl.execute("COMMIT").unwrap();
        assert_eq!(commit_output, "COMMIT");
    }

    #[test]
    fn rejects_transaction_commands_outside_txn() {
        let mut repl = Repl::new();
        repl.load_ontology(SIMPLE_ONTOLOGY).unwrap();

        let err = repl.execute("COMMIT").unwrap_err();
        assert_eq!(err, "No transaction active");
    }
}
