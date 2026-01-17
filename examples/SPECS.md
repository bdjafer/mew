# Examples Specification Coverage

This directory contains example ontologies and test scenarios organized by complexity level.

---

## Level Organization

| Level | Focus | Coverage |
|-------|-------|----------|
| **Level 1** | Fundamentals: CRUD, expressions, transactions | [level-1/SPECS.md](level-1/SPECS.md) |
| **Level 2** | Structure: inheritance, constraints, WALK | [level-2/SPECS.md](level-2/SPECS.md) |
| **Level 3** | Advanced: rules, triggers, versioning | [level-3/SPECS.md](level-3/SPECS.md) |

---

## Coverage Summary

### Level 1: Fundamentals (100% coverage)

Core operations and expressions:
- **Schema**: ontology/node/edge declarations, scalar types, modifiers
- **Mutations**: SPAWN, KILL, LINK, UNLINK, SET with RETURNING
- **Queries**: MATCH, WHERE, ORDER BY, LIMIT/OFFSET, DISTINCT
- **Aggregations**: COUNT, SUM, AVG, MIN, MAX, COLLECT
- **Expressions**: arithmetic, string functions, null handling, timestamps
- **Transactions**: BEGIN, COMMIT, ROLLBACK, isolation levels
- **Debug**: EXPLAIN, PROFILE

### Level 2: Structure (81% coverage)

Type system and advanced patterns:
- **Type Aliases**: constrained types with [match:], [in:], ranges
- **Inheritance**: single, multiple, deep (4+ levels), polymorphic queries
- **Constraints**: [unique], [indexed], [format:], [no_self]
- **WALK Traversal**: FOLLOW, UNTIL, RETURN NODES/EDGES/PATH
- **Patterns**: OPTIONAL MATCH, anonymous targets (_), edge binding
- **Bulk Ops**: MATCH + SET/KILL, LINK IF NOT EXISTS, inline SPAWN
- **Type System**: union types (T | U), any type, type checking (:Type)
- **Parameters**: $param syntax in queries and mutations
- **INSPECT**: direct entity retrieval by ID

**Gaps**: Administration (SHOW, CREATE INDEX), Policy, some formats

### Level 3: Dynamics (38% coverage)

Reactive and system features:
- **Constraints**: hard/soft, pattern matching, transitive cycle detection
- **Rules**: auto/manual, priority, SPAWN/SET/LINK actions
- **Edge Modifiers**: [acyclic], cardinality, referential actions
- **Transitive Patterns**: edge+, edge*, cycle detection

**Gaps**: Watch/Subscriptions, Versioning, Higher-Order Edges, Policy/Session

---

## Spec Reference Migration

All spec references now point to the new modular spec structure under `specs/`:

| Category | Location |
|----------|----------|
| Core Language | `specs/core/1_LANGUAGE.md`, `2_LAYER0.md`, `3_DSL.md` |
| Declarations | `specs/declarations/` (node, edge, ontology, rule, constraint, policy) |
| Statements | `specs/statements/` (match, spawn, kill, link, set, walk, etc.) |
| Modifiers | `specs/modifiers/` (required, unique, indexed, format, etc.) |
| Expressions | `specs/expressions/` (aggregations, functions, parameters, etc.) |
| Patterns | `specs/patterns/` (node, edge, negative, transitive) |
| Types | `specs/types/` (optional, union, any, duration) |
| Literals | `specs/literals/` (duration, timestamp) |

---

## Running Tests

```bash
# Run all tests
./test.sh

# Run specific level
cargo test -p mew-tests -- level_1
cargo test -p mew-tests -- level_2
```

---

*See individual level SPECS.md files for detailed feature coverage.*
