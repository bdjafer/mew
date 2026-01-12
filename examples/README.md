# MEW Integration Test Examples

This directory contains test data (ontologies, seeds, operations) organized by complexity level and domain. The actual test scenarios (Rust code) live in `mew/mew-examples/tests/`.

## Structure

```
examples/                           # Test data (MEW files only)
├── level-1/                        # Fundamentals: basic CRUD, simple types
│   ├── bookmarks/
│   │   ├── ontology.mew           # Type definitions
│   │   ├── seeds/                 # Reusable starting states
│   │   │   ├── empty.mew
│   │   │   ├── minimal.mew
│   │   │   └── populated.mew
│   │   └── operations/            # Reusable operation sequences
│   │       ├── crud.mew
│   │       ├── queries.mew
│   │       └── errors.mew
│   ├── contacts/
│   └── library/
├── level-2/                        # Structure: inheritance, validation
├── level-3/                        # Dynamics: constraints, rules
├── level-4/                        # Higher-order: edges about edges
└── level-5/                        # Meta-systems: self-reference

mew/mew-examples/tests/             # Test scenarios (Rust code)
└── level1_bookmarks.rs            # Scenarios: seed + operations + assertions
```

## Concepts

| Component | Location | Format | Purpose |
|-----------|----------|--------|---------|
| **ontology** | `examples/` | MEW | Type definitions for the domain |
| **seeds** | `examples/` | MEW | Reusable starting states (data fixtures) |
| **operations** | `examples/` | MEW | Reusable operation sequences with step markers |
| **scenarios** | `mew/mew-examples/tests/` | Rust | Test orchestration: seed + operations + assertions |

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

## Writing Tests

### 1. Create operations file (`examples/.../operations/my_ops.mew`)

Operations are pure MEW statements with step markers:

```mew
--# spawn_item
SPAWN item: Item { name = "Test" }

--# query_count
MATCH i: Item RETURN count(i)

--# delete_item
KILL item
```

### 2. Create scenario in test file (`mew/mew-examples/tests/`)

Scenarios orchestrate seeds + operations + assertions:

```rust
use mew_examples::prelude::*;

mod my_test {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("my_test")
            .ontology("level-1/domain/ontology.mew")
            .seed("level-1/domain/seeds/minimal.mew")      // Optional
            .operations("level-1/domain/operations/my_ops.mew")
            .step("spawn_item", |a| a.created(1))
            .step("query_count", |a| a.value(1))
            .step("delete_item", |a| a.deleted(1))
    }

    #[test]
    fn test() {
        scenario().run().unwrap();
    }
}
```

### 3. Optionally create/reuse a seed (`examples/.../seeds/my_seed.mew`)

Seeds set up initial state:

```mew
--# setup_user
SPAWN admin: User { name = "Admin", role = "admin" }

--# setup_items
SPAWN item1: Item { name = "First" }
SPAWN item2: Item { name = "Second" }
```

## Design Principles

- **Full decoupling**: Data (examples/) and code (mew/mew-examples/) are separated
- **Operations in `.mew`**: Native syntax, editor support, reusable across scenarios
- **Seeds are composable**: Multiple scenarios can share the same starting state
- **Scenarios in Rust**: Type-safe assertions, IDE support, complex logic
- **Chaining**: A scenario can use multiple operations files (future)

## Assertion API

```rust
// Mutations
.step("spawn", |a| a.created(1))
.step("update", |a| a.modified(1))
.step("delete", |a| a.deleted(1))
.step("link", |a| a.linked(1))

// Queries - single value
.step("count", |a| a.value(42))
.step("range", |a| a.value_min(0).value_max(100))

// Queries - rows
.step("all", |a| a.rows(5))
.step("some", |a| a.rows_min(1))
.step("none", |a| a.empty())

// Errors
.step("invalid", |a| a.error("required"))
.step("pattern", |a| a.error_matches(".*missing.*"))

// Custom
.step("complex", |a| a.assert_fn(|result| { /* custom logic */ true }))
```
