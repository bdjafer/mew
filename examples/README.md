# MEW Integration Test Examples

Test data (ontologies, seeds, operations) organized by complexity level. Test scenarios (Rust code) live in `mew/tests/tests/`.

## Structure

```
examples/
├── level-1/                        # Fundamentals: basic CRUD, simple types
│   ├── bookmarks/
│   │   ├── ontology.mew            # Type definitions
│   │   ├── seeds/
│   │   │   ├── minimal.mew         # Minimal starting state
│   │   │   └── populated.mew       # Rich starting state
│   │   └── operations/
│   │       ├── queries.mew         # Query operations
│   │       ├── edge_operations.mew # Link/unlink operations
│   │       └── ...
│   ├── contacts/
│   └── library/
├── level-2/                        # Structure: inheritance, validation
│   ├── ecommerce/
│   ├── humanresources/
│   └── tasks/
├── level-3/                        # Dynamics: constraints, rules
├── level-4/                        # Higher-order: edges about edges
└── level-5/                        # Meta-systems: self-reference

mew/tests/tests/                    # Test scenarios (Rust code)
├── level1_bookmarks.rs
├── level1_contacts.rs
├── level1_library.rs
├── level2_ecommerce.rs
├── level2_humanresources.rs
└── level2_tasks.rs
```

## Running Tests

```bash
./test.sh                    # All tests (unit + integration + testgen)
./test.sh integration        # Integration tests only
./test.sh integration -v     # Verbose output

# Filter by name
cargo test -p mew-tests bookmarks
cargo test -p mew-tests level1
```

## Writing Tests

### 1. Create operations file

Operations are MEW statements with step markers (`--# step_name`):

```mew
--# spawn_item
SPAWN item: Item { name = "Test" }

--# query_count
MATCH i: Item RETURN count(i) AS total

--# delete_item
KILL item
```

### 2. Create scenario in Rust

```rust
use mew_tests::prelude::*;

mod my_test {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("my_test")
            .ontology("level-1/domain/ontology.mew")
            .seed("level-1/domain/seeds/minimal.mew")      // Optional
            .operations("level-1/domain/operations/my_ops.mew")
            .step("spawn_item", |a| a.created(1))
            .step("query_count", |a| a.scalar("total", 1i64))
            .step("delete_item", |a| a.deleted(1))
    }

    #[test]
    fn test() {
        scenario().run().unwrap();
    }
}
```

### 3. Create seed file (optional)

Seeds set up initial state:

```mew
--# setup_user
SPAWN admin: User { name = "Admin", role = "admin" }

--# setup_items
SPAWN item1: Item { name = "First" }
SPAWN item2: Item { name = "Second" }
```

## Assertion API

### Mutations

```rust
.step("spawn", |a| a.created(1))
.step("update", |a| a.modified(1))
.step("delete", |a| a.deleted(1))
.step("link", |a| a.linked(1))
.step("unlink", |a| a.unlinked(1))

// Combined
.step("complex", |a| a.created(2).linked(1).deleted(1))
```

### Queries - Single Value

```rust
// Strict: verifies column name + value
.step("count", |a| a.scalar("total", 42i64))
.step("avg", |a| a.scalar("average", 3.14))

// Legacy: value only (no column verification)
.step("count", |a| a.value(42))
.step("range", |a| a.value_min(0).value_max(100))
```

### Queries - Rows

```rust
// Row count
.step("all", |a| a.rows(5))
.step("some", |a| a.rows_min(1))
.step("bounded", |a| a.rows_min(1).rows_max(10))
.step("none", |a| a.empty())
.step("exists", |a| a.not_empty())

// Column verification
.step("select", |a| a.columns(&["id", "name", "email"]).rows(5))
```

### Queries - Row Matching

```rust
// First/last row (partial match)
.step("first", |a| a.first(row!{ name: "Alice" }))
.step("last", |a| a.last(row!{ name: "Zoe" }))

// Exact rows (unordered by default)
.step("all_users", |a| a.returns(vec![
    row!{ name: "Alice", age: 30i64 },
    row!{ name: "Bob", age: 25i64 },
]))

// Ordered rows
.step("sorted", |a| a
    .returns(vec![row!{ name: "Alice" }, row!{ name: "Bob" }])
    .ordered()
)
```

### Errors

```rust
.step("missing_required", |a| a.error("required"))
.step("type_mismatch", |a| a.error("type"))
.step("pattern", |a| a.error_matches(".*missing.*"))

// Error after partial success
.step("spawn_then_fail", |a| a.created(1).error("constraint"))
```

### Custom Assertions

```rust
.step("complex", |a| a.assert_fn(|result| {
    // Custom validation logic
    true
}))
```

## The `row!` Macro

Build row expectations with type inference:

```rust
row!{ name: "Alice", age: 30i64, active: true }
row!{ title: "Test", count: 0i64 }
row!{ description: Option::<String>::None }  // Explicit null
```

Supported types: `i64`, `f64`, `&str`/`String`, `bool`, `Option<T>` for null.
