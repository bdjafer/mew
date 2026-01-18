use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionKind {
    Keyword,
    Type,
    Function,
    Property,
    Snippet,
}

pub fn get_ontology_completions(prefix: &str, line_context: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let prefix_lower = prefix.to_lowercase();
    let keywords = [
        ("node", "node", "Define a node type"),
        ("edge", "edge", "Define an edge type"),
        ("constraint", "constraint", "Define a constraint"),
        ("rule", "rule", "Define a rule"),
        ("type", "type", "Define a type alias"),
        ("ontology", "ontology", "Define an ontology"),
    ];
    for (kw, insert, detail) in keywords {
        if kw.starts_with(&prefix_lower) {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: CompletionKind::Keyword,
                detail: Some(detail.to_string()),
                insert_text: Some(insert.to_string()),
            });
        }
    }
    let types = [
        ("String", "String type"),
        ("Int", "64-bit signed integer"),
        ("Float", "64-bit floating point"),
        ("Bool", "Boolean value"),
        ("Timestamp", "Point in time"),
        ("ID", "Entity identifier"),
    ];
    for (t, detail) in types {
        if t.to_lowercase().starts_with(&prefix_lower) {
            items.push(CompletionItem {
                label: t.to_string(),
                kind: CompletionKind::Type,
                detail: Some(detail.to_string()),
                insert_text: Some(t.to_string()),
            });
        }
    }
    let modifiers = [
        ("required", "Attribute must have a value"),
        ("unique", "Value must be unique"),
    ];
    if line_context.contains('[') {
        for (m, detail) in modifiers {
            if m.starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: m.to_string(),
                    kind: CompletionKind::Property,
                    detail: Some(detail.to_string()),
                    insert_text: Some(m.to_string()),
                });
            }
        }
    }
    items
}

pub fn get_statement_completions(prefix: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let prefix_upper = prefix.to_uppercase();
    let statements = [
        ("MATCH", "Pattern matching query"),
        ("SPAWN", "Create a new node"),
        ("KILL", "Delete a node"),
        ("LINK", "Create an edge"),
        ("UNLINK", "Delete an edge"),
        ("SET", "Modify an attribute"),
        ("RETURN", "Return results"),
        ("WHERE", "Filter condition"),
        ("ORDER BY", "Sort results"),
        ("GROUP BY", "Group results"),
        ("BEGIN", "Start transaction"),
        ("COMMIT", "Commit transaction"),
        ("ROLLBACK", "Rollback transaction"),
    ];
    for (stmt, detail) in statements {
        if stmt.starts_with(&prefix_upper)
            || stmt.to_lowercase().starts_with(&prefix.to_lowercase())
        {
            items.push(CompletionItem {
                label: stmt.to_string(),
                kind: CompletionKind::Keyword,
                detail: Some(detail.to_string()),
                insert_text: Some(stmt.to_string()),
            });
        }
    }
    let functions = [
        ("count", "Count aggregation"),
        ("sum", "Sum aggregation"),
        ("avg", "Average aggregation"),
        ("min", "Minimum value"),
        ("max", "Maximum value"),
        ("now", "Current timestamp"),
    ];
    for (f, detail) in functions {
        if f.starts_with(&prefix.to_lowercase()) {
            items.push(CompletionItem {
                label: format!("{}()", f),
                kind: CompletionKind::Function,
                detail: Some(detail.to_string()),
                insert_text: Some(format!("{}($0)", f)),
            });
        }
    }
    items
}
