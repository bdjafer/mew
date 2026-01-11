# MEW

**Minimum Executable World** — A nested hypergraph database.

[![CI](https://github.com/bryandjafer/mew/actions/workflows/ci.yml/badge.svg)](https://github.com/bryandjafer/mew/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What is MEW?

A **nested hypergraph database** where:
- **Hyperedges** connect any number of nodes (not just pairs)
- **Edges can connect to edges** (higher-order relationships)
- **Schema is stored as data** (self-describing)
- **ACID transactions** with crash recovery

```
Standard Graph:        Hypergraph:           Higher-Order:
  A ──── B              A ──┬── B              A ──┬── B
                            │                      │
                            C                 [confidence: 0.9]
```

## Quick Start

```bash
git clone https://github.com/bryandjafer/mew.git
cd mew/mew
cargo build --release
cargo run --bin mew-repl
```

**Define an ontology** (`contacts.mew`):
```mew
node Person {
    name: String @required
    email: String @unique
}

edge knows: Person -> Person {
    since: Date
}
```

**Query**:
```gql
MATCH (p:Person)-[:knows]->(friend:Person)
RETURN p.name, friend.name
```

## Development

```bash
cargo install just          # Task runner
just test                   # Run tests
just lint                   # Clippy
just fmt                    # Format
```

## Documentation

- [Foundations](specs/1_FOUNDATIONS.md) — Core concepts
- [DSL](specs/2_DSL.md) — Ontology language
- [GQL](specs/3_GQL.md) — Query language
- [Architecture](specs/architecture.md) — System design
- [AGENTS.md](AGENTS.md) — AI agent operation

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Run `just ci` before submitting PRs.

## License

[MIT](LICENSE)
