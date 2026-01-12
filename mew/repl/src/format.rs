//! Output formatting utilities for the REPL.

use mew_core::Value;

/// Format a value for display.
pub fn format_value(v: &Value) -> String {
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

/// Print help information.
pub fn print_help() {
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
