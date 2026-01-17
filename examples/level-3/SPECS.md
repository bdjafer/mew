# Level 3 Coverage

Specification coverage for Level 3: Dynamics features.

---

## Constraints

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Hard constraint declaration | declarations/constraint.md | ✓ | eventchain/ontology |
| Soft constraint `[soft]` | modifiers/soft_constraints.md | ✓ | projectmanagement/ontology, auth/ontology |
| Constraint with `[message:]` | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint pattern matching | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint with WHERE | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint `=> false` (prohibition) | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint `=> condition` | declarations/constraint.md | ✓ | eventchain/ontology |
| Constraint violation error | declarations/constraint.md | ✓ | eventchain/constraint_violations |
| Soft constraint warning | modifiers/soft_constraints.md | ✓ | auth/ontology |

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
| Rule `=> UNLINK` action | declarations/rule.md | ✓ | social/ontology |
| Rule with EXISTS | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule with NOT EXISTS | declarations/rule.md | ✓ | projectmanagement/ontology |
| Rule auto-execution (tested) | declarations/rule.md | ✓ | eventchain/rule_execution |
| TRIGGER manual rule | statements/trigger.md | ✓ | workflow/trigger |
| Rule priority ordering (tested) | declarations/rule.md | ✓ | eventchain/rule_execution |

---

## Edge Modifiers (Advanced)

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `[acyclic]` modifier | modifiers/acyclic_edges.md | ✓ | projectmanagement/ontology |
| `[symmetric]` modifier | modifiers/edge_symmetry.md | ✓ | social/symmetric |
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
| `edge+` (one or more hops) | patterns/transitive_patterns.md | ✓ | eventchain/transitive |
| `edge*` (zero or more hops) | patterns/transitive_patterns.md | ✓ | eventchain/transitive, social/transitive |
| Transitive with `[depth: N]` | patterns/transitive_patterns.md | ✓ | eventchain/transitive |
| Transitive with `[depth: N..M]` | patterns/transitive_patterns.md | ✓ | eventchain/transitive |
| Transitive query (tested) | patterns/transitive_patterns.md | ✓ | eventchain/transitive |
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
| `edge<T>` type reference | types/edge_references.md | ✓ | social/ontology |
| Edge targeting edge | types/edge_references.md | ✓ | social/higher_order |
| Higher-order in LINK | types/edge_references.md | ✓ | social/higher_order |
| Query edges about edges | types/edge_references.md | ✓ | social/higher_order |
| Confidence/meta edge pattern | types/edge_references.md | ✓ | social/ontology (trust_score) |

---

## Watch & Subscriptions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| WATCH basic | statements/watch.md | ✓ | notifications/watch |
| WATCH watch mode | statements/watch.md | ✓ | notifications/watch |
| WATCH consume mode | statements/watch.md | ✓ | notifications/consume |
| Competing consumers (group) | statements/watch.md | ✓ | notifications/consume |
| Windowing | statements/watch.md | ✓ | notifications/watch |
| Buffering | statements/watch.md | ✓ | notifications/watch |
| PAUSE watch | statements/watch_management.md | ✓ | notifications/management |
| RESUME watch | statements/watch_management.md | ✓ | notifications/management |
| CANCEL watch | statements/watch_management.md | ✓ | notifications/management |
| ALTER watch | statements/watch_management.md | ✓ | notifications/management |
| ACK delivery | statements/ack.md | ✓ | notifications/consume |
| NACK delivery | statements/ack.md | ✓ | notifications/consume |

---

## Versioning & Time-Travel

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| CREATE SNAPSHOT | statements/versioning.md | ✓ | audit/versioning |
| CHECKOUT version | statements/versioning.md | ✓ | audit/versioning |
| DIFF between versions | statements/versioning.md | ✓ | audit/versioning |
| CREATE BRANCH | statements/versioning.md | ✓ | audit/versioning |
| MERGE branch | statements/versioning.md | ✓ | audit/versioning |
| Version references (HEAD, HEAD~N) | statements/versioning.md | ✓ | audit/versioning |
| VERSIONS list | statements/versioning.md | ✓ | audit/versioning |

---

## Policy & Authorization

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Policy declaration | declarations/policy.md | ✓ | auth/ontology |
| ALLOW rule | declarations/policy.md | ✓ | auth/policy |
| DENY rule | declarations/policy.md | ✓ | auth/policy |
| `current_actor()` | expressions/context_functions.md | ✓ | auth/context_functions |
| `operation()` | expressions/context_functions.md | ✓ | auth/context_functions |
| `target()` | expressions/context_functions.md | ✓ | auth/context_functions |
| `target_type()` | expressions/context_functions.md | ✓ | auth/context_functions |
| `target_attr()` | expressions/context_functions.md | ✓ | auth/context_functions |
| BEGIN SESSION AS | statements/session.md | ✓ | auth/session |
| END SESSION | statements/session.md | ✓ | auth/session |
| RBAC pattern | declarations/policy.md | ✓ | auth/ontology |

---

## Transactions (Advanced)

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| BEGIN/COMMIT | statements/transactions.md | ✓ | audit/transactions |
| ROLLBACK | statements/transactions.md | ✓ | audit/transactions |
| SAVEPOINT | statements/transactions.md | ✓ | audit/transactions |
| ROLLBACK TO savepoint | statements/transactions.md | ✓ | audit/transactions |
| Nested savepoints | statements/transactions.md | ✓ | audit/transactions |
| Isolation levels | statements/transactions.md | ✓ | audit/transactions |

---

## Coverage Summary

| Category | Covered | Total | Coverage |
|----------|---------|-------|----------|
| Constraints | 9 | 9 | 100% |
| Rules | 15 | 16 | 94% |
| Edge Modifiers | 10 | 14 | 71% |
| Transitive Patterns | 6 | 6 | 100% |
| Negative Patterns | 3 | 4 | 75% |
| Higher-Order Edges | 5 | 5 | 100% |
| Watch & Subscriptions | 12 | 12 | 100% |
| Versioning | 7 | 7 | 100% |
| Policy & Authorization | 11 | 11 | 100% |
| Transactions | 6 | 6 | 100% |
| **Total** | **84** | **90** | **93%** |

---

## Remaining Gaps

### Rules
- Rule `=> KILL` action (not demonstrated in any ontology)

### Edge Modifiers
- `[on_kill_source: prevent]` - Not demonstrated
- Cascade behavior (tested) - Need operational tests
- Prevent behavior (tested) - Need operational tests
- Cardinality violation (tested) - Need error scenario
- Acyclic violation (tested) - Need error scenario

### Negative Patterns
- Nested NOT EXISTS - Not demonstrated

---

## Ontology Focus

| Ontology | Primary Focus | Unique Features |
|----------|---------------|-----------------|
| eventchain | Causation, temporal constraints | transitive patterns, auto rules, constraint violations |
| projectmanagement | Dependencies, cardinality | cascade behavior, soft constraints |
| workflow | State machines, RBAC | manual rules, prevent actions, TRIGGER |
| notifications | Real-time subscriptions | WATCH, ACK/NACK, consumer groups |
| social | Symmetric relationships | [symmetric] edges, higher-order edges, transitive social |
| auth | Policy & Authorization | ALLOW/DENY rules, sessions, context functions |
| audit | Versioning & Time-Travel | SNAPSHOT, CHECKOUT, BRANCH, MERGE, transactions |

---

*Spec references point to files under `specs/` directory.*
