//! MEW REPL - Read-Eval-Print-Loop for MEW hypergraph database.
//!
//! This is the entry point for the MEW REPL binary.

use std::env;
use std::io::{self, IsTerminal, Read};
use std::path::Path;

use mew_repl::Repl;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut repl = Repl::new();

    // Load any files passed as arguments
    for arg in &args[1..] {
        if arg == "-v" || arg == "--verbose" {
            repl.set_verbose(true);
            continue;
        }

        if let Err(e) = repl.run_file(Path::new(arg)) {
            eprintln!("Error loading {}: {}", arg, e);
            std::process::exit(1);
        }
    }

    // Enter interactive mode if stdin is a terminal
    let stdin = io::stdin();
    if stdin.is_terminal() {
        repl.interactive();
    } else if args.len() == 1 {
        // Only read from stdin pipe if no files were passed
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
