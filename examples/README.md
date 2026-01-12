# MEW Integration Test Examples

This directory contains integration tests organized by complexity level and domain.

## Structure

```
examples/
├── level-1/              # Fundamentals: basic CRUD, simple types
│   ├── bookmarks/
│   │   ├── ontology.mew  # The ontology definition
│   │   ├── seeds/        # Reusable starting states
│   │   │   ├── empty.mew
│   │   │   └── populated.mew
│   │   └── scenarios/    # Test scenarios
│   │       ├── crud.mew  # Operations (MEW statements)
│   │       └── crud.rs   # Expectations (Rust assertions)
│   ├── contacts/
│   └── library/
├── level-2/              # Structure: inheritance, validation
├── level-3/              # Dynamics: constraints, rules
├── level-4/              # Higher-order: edges about edges
└── level-5/              # Meta-systems: self-reference
```

## Running Tests

```bash
# Run all integration tests
cargo test -p mew-examples

# Run specific level
cargo test -p mew-examples level_1

# Run specific ontology
cargo test -p mew-examples bookmarks

# Run specific scenario
cargo test -p mew-examples crud
```

## Writing a New Scenario

1. Create `scenarios/my_test.mew` with operations:

```mew
--# step_name
SPAWN Bookmark { url: "https://example.com", title: "Example" }

--# another_step
MATCH b: Bookmark RETURN COUNT(*)
```

2. Create `scenarios/my_test.rs` with expectations:

```rust
use mew_examples::prelude::*;

pub fn scenario() -> Scenario {
    Scenario::new("my_test")
        .ontology("level-1/bookmarks/ontology.mew")
        .seed("level-1/bookmarks/seeds/empty.mew")
        .step("step_name", |a| a.created(1))
        .step("another_step", |a| a.value(1))
}

#[test]
fn test() {
    scenario().run().unwrap();
}
```

## Design Principles

- **Operations in `.mew`**: Native syntax, editor support, no embedded strings
- **Expectations in Rust**: Type-safe, IDE support, complex logic possible
- **Seeds are reusable**: Multiple scenarios can share the same starting state
- **Convention over configuration**: Minimal boilerplate needed
