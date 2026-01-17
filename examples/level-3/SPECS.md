# Level 3 Coverage

Specification coverage for Level 3: Dynamics features.

---

## Constraints

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Hard constraint declaration | declarations/constraint.md | ✓ | eventchain/ontology |
| Soft constraint `[soft]` | modifiers/soft_constraints.md | ✓ | projectmanagement/ontology |
| Constraint with `[message:]` | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint pattern matching | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint with WHERE | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint `=> false` (prohibition) | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint `=> condition` | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint violation error | declarations/constraint.md | ✗ | — |
| Soft constraint warning | modifiers/soft_constraints.md | ✗ | — |

---

## Rules

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Rule declaration | declarations/rule.md | ✓ | eventchain/ontology |
| Rule `[priority: N]` | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule `[auto]` (implicit default) | declarations/rule.md | ✓ | eventchain/ontology |
| Rule `[manual]` | declarations/rule.md | ✓ | workflow/ontology |
| Rule pattern matching | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule with WHERE | declarations/rule.md | ✓ | eventchain/ontology |
| Rule `=> SET` action | declarations/rule.md | ✓ | eventchain/ontology |
| Rule `=> SPAWN` action | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule `=> LINK` action | declarations/rule.md | ✓ | workflow/ontology |
| Rule `=> KILL` action | declarations/rule.md | ✗ | — |
| Rule `=> UNLINK` action | declarations/rule.md | ✗ | — |
| Rule with EXISTS | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule with NOT EXISTS | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule auto-execution (tested) | declarations/rule.md | ✗ | — |
| TRIGGER manual rule | statements/trigger.md | ✗ | — |
| Rule priority ordering (tested) | declarations/rule.md | ✗ | — |

---

## Edge Modifiers (Advanced)

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `[acyclic]` modifier | modifiers/acyclic_edges.md | ✓ | projectmanagement/ontology |
| `[symmetric]` modifier | modifiers/edge_symmetry.md | ✗ | — |
| Cardinality `[a -> N]` | modifiers/cardinality.md | ✓ | projectmanagement/ontology |
| Cardinality `[a -> N..M]` | modifiers/cardinality.md | ✓ | projectmanagement/ontology |
| `[on_kill_target: cascade]` | modifiers/referential_actions.md | ✓ | projectmanagement/ontology |
| `[on_kill_target: unlink]` | modifiers/referential_actions.md | ✓ | (default behavior) |
| `[on_kill_target: prevent]` | modifiers/referential_actions.md | ✓ | workflow/ontology |
| `[on_kill_source: cascade]` | modifiers/referential_actions.md | ✓ | workflow/ontology |
| `[on_kill_source: unlink]` | modifiers/referential_actions.md | ✓ | (default behavior) |
| `[on_kill_source: prevent]` | modifiers/referential_actions.md | ✗ | — |
| Cascade behavior (tested) | modifiers/referential_actions.md | ✗ | — |
| Prevent behavior (tested) | modifiers/referential_actions.md | ✗ | — |
| Cardinality violation (tested) | modifiers/cardinality.md | ✗ | — |
| Acyclic violation (tested) | modifiers/acyclic_edges.md | ✗ | — |

---

## Transitive Patterns

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `edge+` (one or more hops) | patterns/transitive_patterns.md | ✓ | eventchain/ontology (constraint) |
| `edge*` (zero or more hops) | patterns/transitive_patterns.md | ✗ | — |
| Transitive with `[depth: N]` | patterns/transitive_patterns.md | ✗ | — |
| Transitive with `[depth: N..M]` | patterns/transitive_patterns.md | ✗ | — |
| Transitive query (tested) | patterns/transitive_patterns.md | ✗ | — |
| Cycle detection via transitive | patterns/transitive_patterns.md | ✓ | eventchain/ontology |

---

## Negative Patterns

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| NOT EXISTS in constraint | patterns/negative_patterns.md | ✓ | projectmanagement/ontology |
| NOT EXISTS in rule | patterns/negative_patterns.md | ✓ | projectmanagement/ontology |
| NOT EXISTS with WHERE | patterns/negative_patterns.md | ✓ | projectmanagement/ontology |
| Nested NOT EXISTS | patterns/negative_patterns.md | ✗ | — |

---

## Higher-Order Edges

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `edge<T>` type reference | types/edge_references.md | ✗ | — |
| Edge targeting edge | types/edge_references.md | ✗ | — |
| Higher-order in LINK | types/edge_references.md | ✗ | — |
| Query edges about edges | types/edge_references.md | ✗ | — |
| Confidence/meta edge pattern | types/edge_references.md | ✗ | — |

---

## Watch & Subscriptions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| WATCH basic | statements/watch.md | ✗ | — |
| WATCH watch mode | statements/watch.md | ✗ | — |
| WATCH consume mode | statements/watch.md | ✗ | — |
| Competing consumers (group) | statements/watch.md | ✗ | — |
| Windowing | statements/watch.md | ✗ | — |
| Buffering | statements/watch.md | ✗ | — |
| PAUSE watch | statements/watch_management.md | ✗ | — |
| RESUME watch | statements/watch_management.md | ✗ | — |
| CANCEL watch | statements/watch_management.md | ✗ | — |
| ALTER watch | statements/watch_management.md | ✗ | — |
| ACK delivery | statements/ack.md | ✗ | — |
| NACK delivery | statements/ack.md | ✗ | — |

---

## Versioning & Time-Travel

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| CREATE SNAPSHOT | statements/versioning.md | ✗ | — |
| CHECKOUT version | statements/versioning.md | ✗ | — |
| DIFF between versions | statements/versioning.md | ✗ | — |
| CREATE BRANCH | statements/versioning.md | ✗ | — |
| MERGE branch | statements/versioning.md | ✗ | — |
| Version references (HEAD, HEAD~N) | statements/versioning.md | ✗ | — |
| VERSIONS list | statements/versioning.md | ✗ | — |

---

## Policy & Authorization

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Policy declaration | declarations/policy.md | ✗ | — |
| ALLOW rule | declarations/policy.md | ✗ | — |
| DENY rule | declarations/policy.md | ✗ | — |
| `current_actor()` | expressions/context_functions.md | ✗ | — |
| `operation()` | expressions/context_functions.md | ✗ | — |
| `target()` | expressions/context_functions.md | ✗ | — |
| `target_type()` | expressions/context_functions.md | ✗ | — |
| `target_attr()` | expressions/context_functions.md | ✗ | — |
| BEGIN SESSION AS | statements/session.md | ✗ | — |
| END SESSION | statements/session.md | ✗ | — |
| RBAC pattern | declarations/policy.md | ✓ | workflow/ontology (schema only) |

---

## Transactions (Advanced)

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SAVEPOINT | statements/transactions.md | ✗ | — |
| ROLLBACK TO savepoint | statements/transactions.md | ✗ | — |
| Nested savepoints | statements/transactions.md | ✗ | — |

---

## Coverage Summary

| Category | Covered | Total | Coverage |
|----------|---------|-------|----------|
| Constraints | 7 | 9 | 78% |
| Rules | 12 | 16 | 75% |
| Edge Modifiers | 8 | 14 | 57% |
| Transitive Patterns | 2 | 6 | 33% |
| Negative Patterns | 3 | 4 | 75% |
| Higher-Order Edges | 0 | 5 | 0% |
| Watch & Subscriptions | 0 | 12 | 0% |
| Versioning | 0 | 7 | 0% |
| Policy & Authorization | 1 | 11 | 9% |
| Transactions (Advanced) | 0 | 3 | 0% |
| **Total** | **33** | **87** | **38%** |

---

## Gaps to Address

### Critical Gaps (0% coverage - need new ontologies/scenarios)

**Higher-Order Edges** - Need ontology with `edge<T>` patterns:
- Add confidence edges to EventChain (edge targeting causes edge)
- Scenario: `eventchain/edge_references.mew`

**Watch & Subscriptions** - Need dedicated ontology:
- New ontology: `notifications/` with message queue patterns
- Scenarios: `notifications/watch.mew`, `notifications/consume.mew`

**Versioning** - Add to existing ontology:
- Scenario: `projectmanagement/versioning.mew`

### High Priority Gaps

**Transitive Query Tests** - Ontologies define transitive patterns but no query tests:
- Scenario: `eventchain/transitive.mew` - test causes+, causes* queries

**Constraint Violation Tests** - Constraints defined but not tested:
- Scenario: `eventchain/constraint_violations.mew`

**Rule Execution Tests** - Rules defined but auto-execution not tested:
- Scenario: `eventchain/rule_execution.mew`

**TRIGGER Statement** - Manual rules defined but not triggered:
- Scenario: `workflow/trigger.mew`

**[symmetric] Edge Modifier** - Not used in any ontology:
- Need new ontology with mutual relationships (social graph or argumentation)

### Medium Priority Gaps

- `[on_kill_source: prevent]` - Not demonstrated
- Aggregates in WHERE for constraints
- Cardinality/acyclic violation testing
- Nested savepoints

---

## Ontology Focus

| Ontology | Primary Focus | Unique Features |
|----------|---------------|-----------------|
| eventchain | Causation, temporal constraints | transitive patterns, auto rules |
| projectmanagement | Dependencies, cardinality | cascade behavior, soft constraints |
| workflow | State machines, RBAC | manual rules, prevent actions |
| (needed) notifications | Real-time subscriptions | WATCH, ACK/NACK |
| (needed) social | Symmetric relationships | [symmetric] edges |

---

*Spec references point to files under `specs/` directory.*
