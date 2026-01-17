# MEW

**Minimum Executable World** — A hypergraph rewriting kernel

[![CI](https://github.com/bdjafer/mew/actions/workflows/ci.yml/badge.svg)](https://github.com/bdjafer/mew/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

> ⚠️ **Development Status**: TDD - Tests are written before implementation, so many tests are expected to fail until features are implemented.

## What is MEW?

Declare an ontology — types, relations, constraints, rules — compile it, then query and mutate a typed higher-order hypergraph where constraints hold and rules fire automatically.

```mew
-- Schema: types and relations
node Task { 
  title: String [required],
  status: String = "todo"
}
node Person { 
  name: String [required] 
}
edge assigned(task: Task, person: Person)

-- Hyperedge: n-ary relation
edge reviewed(task: Task, reviewer: Person, approver: Person) {
  approved: Bool = false
}

-- Higher-order edge: edge about an edge
edge confidence(about: edge<assigned>) {
  level: Float [>= 0.0, <= 1.0]
}

-- Constraint, Rule, Policy
constraint done_needs_timestamp:
  t: Task WHERE t.status = "done" AND t.completed_at = null
  => FAIL "Completed tasks need completed_at"

rule auto_archive:
  t: Task WHERE t.status = "done" => SET t.archived = true

policy assignee_only:
  ON SET(t: Task, "status") ALLOW IF assigned(t, current_actor())
```

```mew
-- Mutations
SPAWN Task { title: "Write docs" } AS t
LINK assigned(t, current_actor()) AS a   -- capture edge
LINK confidence(a) { level = 0.9 }       -- edge about edge

-- Query all tasks assigned with confidence > 0.8
MATCH t: Task, assigned(t, p) AS a, confidence(a) AS c
WHERE c.level > 0.8
RETURN t.title, p.name, c.level

-- Query all non self-reviewed tasks
MATCH reviewed(t, reviewer, approver) WHERE approver != reviewer
RETURN t.title, reviewer.name, approver.name

-- Subscribe to changes
WATCH t: Task WHERE t.status = "done"
RETURN t.title, t.completed_at
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [just](https://github.com/casey/just) (optional, for task running)

### Setup

```bash
git clone https://github.com/bdjafer/mew.git
cd mew

# Install dev tools (optional)
just setup

# Build
just build

# Run the REPL
just mew
```

### Using the REPL

```bash
# Start interactive REPL
just mew

# Load an ontology file
just repl-file examples/level-1/contacts/ontology.mew

# Or without just:
cd mew && cargo run --bin mew
```

## Development

### Commands

```bash
just              # List all commands
just build        # Build all crates
just test         # Run quick tests (unit + integration)
just test-all     # Run all tests including testgen
just lint         # Run clippy
just fmt          # Format code
just ci           # Run all CI checks locally
just watch        # Watch mode - run tests on change
just docs         # Generate and open documentation
```

### Project Structure

```
mew/
├── mew/                    # Rust workspace
│   ├── core/              # Core types and values
│   ├── graph/             # Hypergraph storage
│   ├── parser/            # MEW language parser
│   ├── analyzer/          # Semantic analysis
│   ├── compiler/          # Ontology compiler
│   ├── registry/          # Type registry
│   ├── pattern/           # Pattern matching
│   ├── query/             # Query execution
│   ├── mutation/          # Mutation execution
│   ├── constraint/        # Constraint checking
│   ├── rule/              # Rule engine
│   ├── transaction/       # Transaction management
│   ├── journal/           # Write-ahead logging
│   ├── session/           # Session management
│   ├── repl/              # Interactive REPL
│   ├── tests/             # Integration test framework
│   └── testgen/           # Test generation
├── examples/              # Example ontologies by level
│   ├── level-1/          # Fundamentals
│   ├── level-2/          # Structure
│   ├── level-3/          # Dynamics
│   ├── level-4/          # Reflection
│   └── level-5/          # Meta-Systems
├── specs/                 # Specifications
└── .github/              # CI/CD workflows
```

### TDD Workflow

This project follows Test-Driven Development:

1. **Specs** define the behavior (`specs/`)
2. **Examples** show usage patterns (`examples/`)
3. **Tests** verify implementation (`mew/tests/`)
4. **Implementation** makes tests pass

Tests failing is normal and expected during development. Progress is measured by increasing pass rate.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Before submitting a PR
just ci
```

## Documentation

- [examples/](examples/) — Scenarios examples
- [specs/](specs/) — Technical specifications

## License

[AGPLv3](LICENSE)
