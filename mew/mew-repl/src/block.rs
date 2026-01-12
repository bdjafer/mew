//! Block collection utilities for multi-line input.

use std::io::{self, BufRead, Write};

/// Collect a block (e.g., ontology definition) from an iterator of lines.
///
/// This reads lines until the braces are balanced.
pub fn collect_block_from_lines<I>(first_line: &str, lines: &mut I) -> Result<String, String>
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

/// Collect a block from stdin interactively.
///
/// This prompts the user for continuation lines until braces are balanced.
pub fn collect_block_from_stdin(first_line: &str, stdin: &io::Stdin) -> Result<String, String> {
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

/// Extract the ontology source from a LOAD ONTOLOGY block.
///
/// This extracts the content between the outermost braces.
pub fn extract_ontology_source(block: &str) -> Result<String, String> {
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

/// Check if a parse error indicates incomplete input that needs more lines.
pub fn should_continue_parse(err: &str) -> bool {
    err.contains("unexpected end of input") || err.contains("found end of input")
}
