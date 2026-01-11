//! MEW REPL - Read-Eval-Print-Loop for MEW hypergraph database.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::Path;

use mew_compiler::compile;
use mew_core::{EntityId, Value};
use mew_graph::Graph;
use mew_mutation::MutationExecutor;
use mew_parser::{parse_stmt, MatchStmt, Stmt, Target, TargetRef, TxnStmt};
use mew_pattern::{Binding, Bindings};
use mew_query::QueryExecutor;
use mew_registry::{Registry, RegistryBuilder};

/// REPL state
struct Repl {
    registry: Registry,
    graph: Graph,
    in_transaction: bool,
    verbose: bool,
    bindings: HashMap<String, EntityId>,
}

impl Repl {
    fn new() -> Self {
        Self {
            registry: RegistryBuilder::new().build().unwrap(),
            graph: Graph::new(),
            in_transaction: false,
            verbose: false,
            bindings: HashMap::new(),
        }
    }

    fn load_ontology(&mut self, source: &str) -> Result<String, String> {
        // Compile directly from source
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
        if trimmed.to_lowercase().starts_with("load ontology") {
            let source = self.extract_ontology_source(trimmed)?;
            return self.load_ontology(&source);
        }
        if trimmed.starts_with("ontology ") {
            return self.load_ontology(trimmed);
        }

        // Parse and execute statement
        let stmt = parse_stmt(trimmed).map_err(|e| format!("Parse error: {}", e))?;

        match stmt {
            Stmt::Match(ref match_stmt) => self.execute_match(match_stmt),
            Stmt::Spawn(ref spawn_stmt) => self.execute_spawn(spawn_stmt),
            Stmt::Kill(ref kill_stmt) => self.execute_kill(kill_stmt),
            Stmt::Link(ref link_stmt) => self.execute_link(link_stmt),
            Stmt::Unlink(ref unlink_stmt) => self.execute_unlink(unlink_stmt),
            Stmt::Set(ref set_stmt) => self.execute_set(set_stmt),
            Stmt::Txn(ref txn_stmt) => self.execute_txn(txn_stmt),
            Stmt::Walk(_) => Ok("WALK not yet implemented".to_string()),
        }
    }

    fn execute_match(&self, stmt: &MatchStmt) -> Result<String, String> {
        let executor = QueryExecutor::new(&self.registry, &self.graph);
        let initial_bindings = self.pattern_bindings();
        let results = executor
            .execute_match_with_bindings(stmt, &initial_bindings)
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
        let bindings = self.pattern_bindings();
        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);

        let result = executor
            .execute_spawn(stmt, &bindings)
            .map_err(|e| format!("Spawn error: {}", e))?;

        if let Some(node_id) = result.created_node() {
            self.bindings.insert(stmt.var.clone(), node_id.into());
            Ok(format!("Created {} with id {}", stmt.var, node_id.raw()))
        } else {
            Ok("Spawn completed".to_string())
        }
    }

    fn execute_kill(&mut self, stmt: &mew_parser::KillStmt) -> Result<String, String> {
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id
            .as_node()
            .ok_or_else(|| "KILL requires a node target".to_string())?;

        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);
        let result = executor
            .execute_kill(stmt, node_id)
            .map_err(|e| format!("Kill error: {}", e))?;

        self.remove_bindings_for_entity(target_id);
        Ok(format!(
            "Deleted {} nodes, {} edges",
            result.deleted_nodes(),
            result.deleted_edges()
        ))
    }

    fn execute_link(&mut self, stmt: &mew_parser::LinkStmt) -> Result<String, String> {
        let mut targets = Vec::new();
        for target_ref in &stmt.targets {
            targets.push(self.resolve_target_ref(target_ref)?);
        }

        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);
        let result = executor
            .execute_link(stmt, targets)
            .map_err(|e| format!("Link error: {}", e))?;

        if let (Some(var), Some(edge_id)) = (&stmt.var, result.created_edge()) {
            self.bindings.insert(var.clone(), edge_id.into());
        }

        Ok("Link created".to_string())
    }

    fn execute_unlink(&mut self, stmt: &mew_parser::UnlinkStmt) -> Result<String, String> {
        let target_id = self.resolve_target(&stmt.target)?;
        let edge_id = target_id
            .as_edge()
            .ok_or_else(|| "UNLINK requires an edge target".to_string())?;

        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);
        let result = executor
            .execute_unlink(stmt, edge_id)
            .map_err(|e| format!("Unlink error: {}", e))?;

        self.remove_bindings_for_entity(target_id);
        Ok(format!("Deleted {} edges", result.deleted_edges()))
    }

    fn execute_set(&mut self, stmt: &mew_parser::SetStmt) -> Result<String, String> {
        let target_id = self.resolve_target(&stmt.target)?;
        let node_id = target_id
            .as_node()
            .ok_or_else(|| "SET requires a node target".to_string())?;

        let bindings = self.pattern_bindings();
        let mut executor = MutationExecutor::new(&self.registry, &mut self.graph);
        let result = executor
            .execute_set(stmt, vec![node_id], &bindings)
            .map_err(|e| format!("Set error: {}", e))?;

        let updated = match result {
            mew_mutation::MutationOutput::Updated(ref updated) => updated.node_ids.len(),
            _ => 0,
        };
        Ok(format!("Updated {} nodes", updated))
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

    fn resolve_target(&self, target: &Target) -> Result<EntityId, String> {
        match target {
            Target::Var(name) => self
                .bindings
                .get(name)
                .copied()
                .ok_or_else(|| format!("Unknown variable: {}", name)),
            Target::Id(_) | Target::Pattern(_) => {
                Err("Only variable targets are supported".to_string())
            }
        }
    }

    fn resolve_target_ref(&self, target_ref: &TargetRef) -> Result<EntityId, String> {
        match target_ref {
            TargetRef::Var(name) => self
                .bindings
                .get(name)
                .copied()
                .ok_or_else(|| format!("Unknown variable: {}", name)),
            TargetRef::Id(_) | TargetRef::Pattern(_) => {
                Err("Only variable targets are supported".to_string())
            }
        }
    }

    fn pattern_bindings(&self) -> Bindings {
        let mut bindings = Bindings::new();
        for (name, entity) in &self.bindings {
            match entity {
                EntityId::Node(node_id) => bindings.insert(name.clone(), Binding::Node(*node_id)),
                EntityId::Edge(edge_id) => bindings.insert(name.clone(), Binding::Edge(*edge_id)),
            }
        }
        bindings
    }

    fn remove_bindings_for_entity(&mut self, entity_id: EntityId) {
        self.bindings.retain(|_, value| *value != entity_id);
    }

    fn extract_ontology_source(&self, block: &str) -> Result<String, String> {
        let lower = block.trim_start().to_lowercase();
        if !lower.starts_with("load ontology") {
            return Ok(block.to_string());
        }

        let open_index = block
            .find('{')
            .ok_or_else(|| "LOAD ONTOLOGY requires a '{' block".to_string())?;
        let mut depth = 0usize;
        let mut close_index = None;

        for (idx, ch) in block.char_indices().skip(open_index) {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_index = Some(idx);
                    break;
                }
            }
        }

        let close_index =
            close_index.ok_or_else(|| "LOAD ONTOLOGY requires a matching '}'".to_string())?;

        let inner = &block[(open_index + 1)..close_index];
        Ok(inner.trim().to_string())
    }

    fn should_continue_parse(err: &str) -> bool {
        err.contains("unexpected end of input") || err.contains("found end of input")
    }

    fn collect_block_from_lines<I>(&self, first_line: &str, lines: &mut I) -> Result<String, String>
    where
        I: Iterator<Item = String>,
    {
        let mut block = String::new();
        block.push_str(first_line);
        block.push('\n');

        let mut depth = 0usize;
        let mut started = false;

        let mut current_line = first_line.to_string();
        loop {
            for ch in current_line.chars() {
                if ch == '{' {
                    depth += 1;
                    started = true;
                } else if ch == '}' && started {
                    depth = depth.saturating_sub(1);
                }
            }

            if started && depth == 0 {
                break;
            }

            let next = lines
                .next()
                .ok_or_else(|| "LOAD ONTOLOGY block did not terminate".to_string())?;
            for ch in next.chars() {
                if ch == '{' {
                    depth += 1;
                    started = true;
                } else if ch == '}' {
                    depth = depth.saturating_sub(1);
                }
            }
            block.push_str(&next);
            block.push('\n');
            current_line = next;
            if started && depth == 0 {
                break;
            }
        }

        Ok(block)
    }

    fn run_file(&mut self, path: &Path) -> Result<(), String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        println!("Loading: {}", path.display());

        self.run_script(&content)
    }

    fn run_script(&mut self, content: &str) -> Result<(), String> {
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
            match self.load_ontology(&content) {
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
                let block = self.collect_block_from_lines(&line, &mut lines)?;
                let source = self.extract_ontology_source(&block)?;
                match self.load_ontology(&source) {
                    Ok(msg) => println!("{}", msg),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
                continue;
            }

            if trimmed.starts_with("ontology ") {
                let block = self.collect_block_from_lines(&line, &mut lines)?;
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
                    if !Self::should_continue_parse(&err) {
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

    fn interactive(&mut self) {
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

            if trimmed.to_lowercase().starts_with("load ontology") {
                let block = match self.collect_block_from_stdin(&line, &stdin) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                };
                let source = match self.extract_ontology_source(&block) {
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
                let block = match self.collect_block_from_stdin(&line, &stdin) {
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
                        if Self::should_continue_parse(&err) {
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

    fn collect_block_from_stdin(
        &self,
        first_line: &str,
        stdin: &io::Stdin,
    ) -> Result<String, String> {
        let mut block = String::new();
        block.push_str(first_line);
        block.push('\n');

        let mut depth = 0usize;
        let mut started = false;

        let mut current_line = first_line.to_string();
        loop {
            for ch in current_line.chars() {
                if ch == '{' {
                    depth += 1;
                    started = true;
                } else if ch == '}' && started {
                    depth = depth.saturating_sub(1);
                }
            }
            if started && depth == 0 {
                break;
            }

            print!("....> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            if stdin.lock().read_line(&mut line).unwrap() == 0 {
                return Err("LOAD ONTOLOGY block did not terminate".to_string());
            }
            for ch in line.chars() {
                if ch == '{' {
                    depth += 1;
                    started = true;
                } else if ch == '}' {
                    depth = depth.saturating_sub(1);
                }
            }
            block.push_str(line.trim_end());
            block.push('\n');
            current_line = line;
            if started && depth == 0 {
                break;
            }
        }

        Ok(block)
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
        let stdin = io::stdin();
        if stdin.is_terminal() {
            repl.interactive();
        } else {
            let mut input = String::new();
            if let Err(e) = stdin.lock().read_to_string(&mut input) {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = repl.run_script(&input) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
