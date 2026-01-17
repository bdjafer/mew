# MEW

**Minimum Executable World** — A hypergraph rewriting kernel

[![CI](https://github.com/bdjafer/mew/actions/workflows/ci.yml/badge.svg)](https://github.com/bdjafer/mew/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

> ⚠️ **Development Status**: This project uses TDD-first development. Tests are written before implementation, so many tests are expected to fail until features are implemented.

## What is MEW?

Declare an ontology — types, relations, constraints, rules — compile it, then query and mutate a typed higher-order hypergraph where constraints hold and rules fire automatically.

```mew
node Person {
    name: String @required
    email: String @unique
}

edge knows: Person -> Person {
    since: Date
}

constraint email_format:
    p: Person => p.email MATCHES ".*@.*\\..*"

rule auto_timestamp:
    WHEN SPAWN p: Person
    THEN SET p.created_at = now()
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

- [CONTEXT.md](CONTEXT.md) — Vision and philosophy
- [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute
- [examples/LEVELS.md](examples/LEVELS.md) — Ontology complexity levels
- [specs/](specs/) — Technical specifications

## License

[MIT](LICENSE)

## Acknowledgments

Built with inspiration from:
- [Wolfram Physics Project](https://www.wolframphysics.org/) — Hypergraph rewriting
- [Category Theory](https://ncatlab.org/) — Compositionality and structure
- [Description Logics](https://www.w3.org/TR/owl2-overview/) — Formal ontologies
