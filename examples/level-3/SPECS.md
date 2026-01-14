# Level 3 Coverage

Specification coverage for Level 3: Dynamics features.

## Schema Features: Constraints

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Hard constraint | 3_SCHEMA.md §8.2 | ✓ | eventchain/ontology, projectmanagement/ontology |
| Soft constraint | 3_SCHEMA.md §8.3 | ✓ | projectmanagement/ontology |
| Constraint with [message:] | 3_SCHEMA.md §8.1 | ✓ | eventchain/ontology |
| Constraint pattern matching | 3_SCHEMA.md §8.4 | ✓ | eventchain/ontology |
| Constraint with WHERE | 3_SCHEMA.md §8.4 | ✓ | eventchain/ontology |
| Constraint with => false | 3_SCHEMA.md §8.5 | ✓ | eventchain/ontology |
| Constraint with => condition | 3_SCHEMA.md §8.5 | ✓ | eventchain/ontology |
| Constraint with transitive edge | 3_SCHEMA.md §8.4 | ✓ | eventchain/ontology |
| Constraint violation (tested) | 3_SCHEMA.md §8 | ✗ | — |
| Soft constraint warning | 3_SCHEMA.md §8.3 | ✗ | — |

## Schema Features: Rules

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Rule definition | 3_SCHEMA.md §9.1 | ✓ | eventchain/ontology |
| Rule [priority: N] | 3_SCHEMA.md §9.2 | ✓ | projectmanagement/ontology |
| Rule [auto] (implicit) | 3_SCHEMA.md §9.3 | ✓ | eventchain/ontology |
| Rule [manual] | 3_SCHEMA.md §9.3 | ✓ | workflow/ontology |
| Rule pattern matching | 3_SCHEMA.md §9.4 | ✓ | projectmanagement/ontology |
| Rule with WHERE | 3_SCHEMA.md §9.4 | ✓ | eventchain/ontology |
| Rule => SET | 3_SCHEMA.md §9.5 | ✓ | eventchain/ontology |
| Rule => SPAWN | 3_SCHEMA.md §9.5 | ✓ | projectmanagement/ontology |
| Rule => LINK | 3_SCHEMA.md §9.5 | ✓ | workflow/ontology |
| Rule with EXISTS | 3_SCHEMA.md §9.4 | ✓ | projectmanagement/ontology |
| Rule with NOT EXISTS | 3_SCHEMA.md §9.4 | ✓ | projectmanagement/ontology |
| Rule execution (tested) | 3_SCHEMA.md §9 | ✗ | — |
| Manual rule invocation | 3_SCHEMA.md §9.3 | ✗ | — |
| Rule priority ordering | 3_SCHEMA.md §9.2 | ✗ | — |

## Schema Features: Edge Modifiers

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| [acyclic] | 3_SCHEMA.md §6.3.3 | ✓ | projectmanagement/ontology |
| [symmetric] | 3_SCHEMA.md §6.3.4 | ✗ | — |
| Cardinality [a -> N] | 3_SCHEMA.md §6.4.1 | ✓ | projectmanagement/ontology |
| Cardinality [a -> N..M] | 3_SCHEMA.md §6.4.2 | ✓ | projectmanagement/ontology |
| [on_kill_target: cascade] | 3_SCHEMA.md §6.5.1 | ✓ | projectmanagement/ontology |
| [on_kill_target: unlink] | 3_SCHEMA.md §6.5.2 | ✓ | (default behavior) |
| [on_kill_target: prevent] | 3_SCHEMA.md §6.5.3 | ✓ | workflow/ontology |
| [on_kill_source: cascade] | 3_SCHEMA.md §6.5.1 | ✓ | workflow/ontology |
| [on_kill_source: unlink] | 3_SCHEMA.md §6.5.2 | ✓ | (default behavior) |
| [on_kill_source: prevent] | 3_SCHEMA.md §6.5.3 | ✗ | — |
| Cascade behavior (tested) | 5_MUTATIONS.md §2.4 | ✗ | — |
| Prevent behavior (tested) | 5_MUTATIONS.md §2.4.3 | ✗ | — |
| Cardinality violation (tested) | 3_SCHEMA.md §6.4 | ✗ | — |
| Acyclic violation (tested) | 3_SCHEMA.md §6.3.3 | ✗ | — |

## Query Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Transitive edge+ (one or more) | 4_QUERIES.md §2.4.4 | ✓ | eventchain/ontology (constraint) |
| Transitive edge* (zero or more) | 4_QUERIES.md §2.4.4 | ✗ | — |
| Transitive with [depth: N] | 4_QUERIES.md §2.4.4 | ✗ | — |
| Transitive with [depth: N..M] | 4_QUERIES.md §2.4.4 | ✗ | — |
| Transitive query (tested) | 4_QUERIES.md §2.4.4 | ✗ | — |
| EXISTS in WHERE | 4_QUERIES.md §2.5.3 | ✓ | projectmanagement/ontology, workflow/ontology |
| NOT EXISTS in WHERE | 4_QUERIES.md §2.5.3 | ✓ | projectmanagement/ontology |
| EXISTS with nested pattern | 4_QUERIES.md §2.5.3 | ✓ | projectmanagement/ontology |
| EXISTS with WHERE inside | 4_QUERIES.md §2.5.3 | ✓ | projectmanagement/ontology |
| EXISTS query (tested) | 4_QUERIES.md §2.5.3 | ✗ | — |
| Aggregate COUNT in WHERE | 4_QUERIES.md §2.5.4 | ✗ | — |
| Aggregate SUM in WHERE | 4_QUERIES.md §2.5.4 | ✗ | — |
| Aggregate AVG in WHERE | 4_QUERIES.md §2.5.4 | ✗ | — |

## Higher-Order Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| edge<T> type reference | 4_QUERIES.md §2.4.3 | ✗ | — |
| Edge targeting edge | 3_SCHEMA.md §6.7 | ✗ | — |
| Edge binding with AS | 4_QUERIES.md §2.4.2 | ✗ | — |
| Query edges about edges | 4_QUERIES.md §2.4.3 | ✗ | — |
| Higher-order in LINK | 5_MUTATIONS.md §3.3 | ✗ | — |
| Confidence edge pattern | — | ✗ | — |

## Subscription Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SUBSCRIBE basic | 6_SYSTEM.md §X | ✗ | — |
| SUBSCRIBE watch mode | 6_SYSTEM.md §X | ✗ | — |
| SUBSCRIBE consume mode | 6_SYSTEM.md §X | ✗ | — |
| Competing consumers (group) | 6_SYSTEM.md §X | ✗ | — |
| Windowing | 6_SYSTEM.md §X | ✗ | — |
| Buffering | 6_SYSTEM.md §X | ✗ | — |
| ACK/NACK delivery | 6_SYSTEM.md §X | ✗ | — |

## Time Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| logical_time() | 6_SYSTEM.md §X | ✗ | — |
| TICK advancement | 6_SYSTEM.md §X | ✗ | — |
| Tick-based rule execution | 6_SYSTEM.md §X | ✗ | — |
| Tick trigger: manual | 6_SYSTEM.md §X | ✗ | — |
| Tick trigger: per-transaction | 6_SYSTEM.md §X | ✗ | — |
| Tick trigger: periodic | 6_SYSTEM.md §X | ✗ | — |

## Authorization Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Role-based (RBAC) | — | ✓ | workflow/ontology (Role, has_role, can_perform) |
| Relationship-based auth | — | ✗ | — |
| Attribute-based (ABAC) | — | ✗ | — |
| operation() function | — | ✗ | — |
| target() function | — | ✗ | — |
| target_type() function | — | ✗ | — |
| target_attr() function | — | ✗ | — |
| RBAC enforcement (tested) | — | ✗ | — |

## Transaction Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SAVEPOINT | 5_MUTATIONS.md §6.10 | ✗ | — |
| ROLLBACK TO savepoint | 5_MUTATIONS.md §6.10 | ✗ | — |
| READ COMMITTED isolation | 5_MUTATIONS.md §6.7.1 | ✗ | — |
| SERIALIZABLE isolation | 5_MUTATIONS.md §6.7.2 | ✗ | — |

## Coverage Summary

| Category | Covered | Total | Coverage |
|----------|---------|-------|----------|
| Constraints | 8 | 10 | 80% |
| Rules | 11 | 14 | 79% |
| Edge Modifiers | 9 | 14 | 64% |
| Queries | 6 | 13 | 46% |
| Higher-Order | 0 | 6 | 0% |
| Subscriptions | 0 | 7 | 0% |
| Time | 0 | 6 | 0% |
| Authorization | 1 | 8 | 13% |
| Transactions | 0 | 4 | 0% |
| **Total** | **35** | **82** | **43%** |

## Gaps to Address

### Critical Gaps (0% coverage)
- **Higher-Order Edges** — No scenarios test edge<T>, edges about edges, or higher-order patterns. Need to add confidence/meta edges to EventChain or create dedicated scenario.
- **Subscriptions** — No SUBSCRIBE scenarios exist. Need dedicated pub/sub testing file.
- **Time Features** — No logical_time() or TICK scenarios. Need temporal test suite.
- **Transactions** — No SAVEPOINT or isolation level scenarios. Need transaction testing file.

### High Priority Gaps
- **Transitive Query Tests** — Ontologies define transitive patterns (causes+) but no operation files test them. Need:
  - `eventchain/operations/transitive.mew` for causes+ queries
  - `projectmanagement/operations/dependencies.mew` for depends_on chain queries
- **Constraint Violation Tests** — Constraints are defined but not tested for violation behavior
- **Rule Execution Tests** — Rules are defined but auto-execution not tested
- **Edge Modifier Tests** — [acyclic], cascade, prevent defined but behavior not tested

### Medium Priority Gaps
- [symmetric] edge modifier — Not used in any ontology
- [on_kill_source: prevent] — Not demonstrated
- Aggregates in WHERE — Not tested
- Manual rule invocation — Rule [manual] defined but not invoked

### Ontology Improvements
- **EventChain** — Add edge<causes> confidence pattern for higher-order demo
- **ProjectManagement** — Add transitive dependency queries, cardinality violation tests
- **Workflow** — Add RBAC enforcement tests, transition authorization scenarios

### Missing Ontology (from LEVELS.md)
- **Argumentation** — Listed in LEVELS.md but not present in level-3. Should include:
  - attack/support edges between arguments
  - symmetric "related_to" edges
  - transitive rebuttal chains

### Recommended New Scenario Files
1. `eventchain/operations/transitive.mew` — Test causes+, causes* queries
2. `eventchain/operations/constraints.mew` — Test temporal_order violation, no_causal_loop violation
3. `projectmanagement/operations/rules.mew` — Test auto-timestamp rules, milestone completion
4. `projectmanagement/operations/cascade.mew` — Test on_kill_target: cascade behavior
5. `workflow/operations/authorization.mew` — Test RBAC with can_perform edge
6. `workflow/operations/transitions.mew` — Test manual rule invocation
7. `level-3/subscriptions/basic.mew` — SUBSCRIBE scenarios (may need dedicated ontology)
8. `level-3/transactions/savepoints.mew` — SAVEPOINT and isolation scenarios
