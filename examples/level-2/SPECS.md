# Level 2 Coverage

Specification coverage for Level 2: Structure features.

---

## Schema: Type Aliases

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Type alias definition | declarations/type_alias.md | ✓ | ecommerce/type_aliases |
| Type alias with [match:] | modifiers/regex_validation.md | ✓ | ecommerce/type_aliases |
| Type alias with [in:] | modifiers/enum_constraint.md | ✓ | ecommerce/type_aliases |
| Type alias with [>= N]/[<= M] | modifiers/range_constraint.md | ✓ | ecommerce/type_aliases |
| Alias chaining | declarations/type_alias.md | ✓ | ecommerce/alias_chaining |

---

## Schema: Inheritance

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Single inheritance (`:`) | declarations/node.md | ✓ | ecommerce/inheritance |
| Multiple inheritance | declarations/node.md | ✓ | humanresources/multiple_inheritance |
| Deep inheritance (4+ levels) | declarations/node.md | ✓ | humanresources/deep_inheritance |
| Diamond resolution | declarations/node.md | ✓ | humanresources/diamond_inheritance |
| `[abstract]` modifier | declarations/node_modifiers.md | ✓ | ecommerce/ontology |
| Polymorphic queries | statements/match.md | ✓ | ecommerce/inheritance |

---

## Schema: Attribute Constraints

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `[unique]` constraint | modifiers/unique.md | ✓ | ecommerce/uniqueness |
| `[indexed]` modifier | modifiers/indexed.md | ✓ | ecommerce/type_aliases |
| `[format: email]` | modifiers/format_validation.md | ✓ | ecommerce/format_validation |
| `[format: url]` | modifiers/format_validation.md | ✓ | ecommerce/format_validation |
| `[format: uuid]` | modifiers/format_validation.md | ✓ | ecommerce/format_validation |
| `[format: slug]` | modifiers/format_validation.md | ✓ | ecommerce/format_slug |
| `[format: phone]` | modifiers/format_validation.md | ✓ | ecommerce/format_advanced |
| `[format: iso_date]` | modifiers/format_validation.md | ✓ | ecommerce/format_advanced |
| `[format: iso_datetime]` | modifiers/format_validation.md | ✓ | ecommerce/format_advanced |
| `[format: ipv4]` | modifiers/format_validation.md | ✓ | ecommerce/format_advanced |
| `[format: ipv6]` | modifiers/format_validation.md | ✓ | ecommerce/format_advanced |
| `[match: "regex"]` | modifiers/regex_validation.md | ✓ | ecommerce/format_validation |
| `[in: [...]]` enum | modifiers/enum_constraint.md | ✓ | ecommerce/format_validation |
| `[>= N]` constraint | modifiers/range_constraint.md | ✓ | ecommerce/format_validation |
| `[<= M]` constraint | modifiers/range_constraint.md | ✓ | ecommerce/format_validation |
| Range `[N..M]` constraint | modifiers/range_constraint.md | ✓ | ecommerce/format_validation |
| `[length: N..M]` | modifiers/length_constraint.md | ✓ | ecommerce/type_aliases |

---

## Schema: Edge Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `[no_self]` edge modifier | modifiers/no_self.md | ✓ | humanresources/no_self |
| Edge attributes | declarations/edge.md | ✓ | humanresources/edge_attributes |
| Edge attribute defaults | declarations/edge.md | ✓ | humanresources/edge_attributes |
| Edge cardinality | modifiers/cardinality.md | ✓ | humanresources/cardinality |
| Referential actions | modifiers/referential_actions.md | ✓ | tasks/referential_actions |

---

## Query: WALK Traversal

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| WALK FROM node ID | statements/walk.md | ✓ | humanresources/walk_traversal |
| WALK FROM MATCH result | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW single edge type | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW multiple edge types | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW with OUTBOUND | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW with INBOUND | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW with ANY direction | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW with [depth: N] | statements/walk.md | ✓ | humanresources/walk_traversal |
| FOLLOW with [depth: N..M] | statements/walk.md | ✓ | humanresources/walk_traversal |
| Multiple FOLLOW clauses | statements/walk.md | ✓ | humanresources/walk_traversal |
| UNTIL condition | statements/walk.md | ✓ | humanresources/walk_traversal |
| RETURN NODES | statements/walk.md | ✓ | humanresources/walk_traversal |
| RETURN EDGES | statements/walk.md | ✓ | humanresources/walk_traversal |
| RETURN PATH | statements/walk.md | ✓ | humanresources/walk_traversal |
| RETURN TERMINAL | statements/walk.md | ✓ | humanresources/walk_traversal |

---

## Query: Patterns

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| OPTIONAL MATCH | statements/optional_match.md | ✓ | humanresources/complex_joins |
| Multiple OPTIONAL MATCH | statements/optional_match.md | ✓ | humanresources/complex_joins |
| OPTIONAL MATCH with WHERE | statements/optional_match.md | ✓ | humanresources/complex_joins |
| Anonymous target (`_`) | patterns/edge_patterns.md | ✓ | tasks/anonymous_targets |
| Anonymous in MATCH pattern | patterns/edge_patterns.md | ✓ | tasks/anonymous_targets |
| Anonymous in edge pattern | patterns/edge_patterns.md | ✓ | tasks/anonymous_targets |
| Edge binding with AS | patterns/edge_patterns.md | ✓ | tasks/unlink |
| Multi-hop joins | statements/match.md | ✓ | humanresources/complex_joins |

---

## Mutation: Bulk Operations

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Bulk SET (MATCH + SET) | statements/set.md | ✓ | tasks/bulk_mutations |
| Bulk SET multiple attributes | statements/set.md | ✓ | tasks/bulk_mutations |
| Bulk SET with computed values | statements/set.md | ✓ | tasks/bulk_mutations |
| Bulk KILL (MATCH + KILL) | statements/kill.md | ✓ | tasks/bulk_mutations |
| Bulk KILL with conditions | statements/kill.md | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING id | statements/returning.md | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING * | statements/returning.md | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING specific fields | statements/returning.md | ✓ | tasks/bulk_mutations |
| SET with RETURNING | statements/returning.md | ✓ | tasks/bulk_mutations |

---

## Mutation: LINK Variants

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| LINK IF NOT EXISTS basic | statements/link.md | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS idempotency | statements/link.md | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS multiple targets | statements/link.md | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS vs LINK | statements/link.md | ✓ | tasks/link_if_not_exists |
| Inline SPAWN in LINK | statements/link.md | ✓ | ecommerce/inline_spawn_link |
| Inline SPAWN with AS binding | statements/link.md | ✓ | ecommerce/inline_spawn_link |
| Multiple inline SPAWNs in LINK | statements/link.md | ✓ | ecommerce/inline_spawn_link |

---

## Mutation: UNLINK

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| UNLINK by edge alias | statements/unlink.md | ✓ | tasks/unlink |
| UNLINK with pattern matching | statements/unlink.md | ✓ | tasks/unlink |
| UNLINK selective (with filter) | statements/unlink.md | ✓ | tasks/unlink |
| UNLINK preserves entities | statements/unlink.md | ✓ | tasks/unlink |

---

## Parameters

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| $param in WHERE (string) | expressions/parameters.md | ✓ | tasks/parameters |
| $param in WHERE (int) | expressions/parameters.md | ✓ | tasks/parameters |
| $param multiple in WHERE | expressions/parameters.md | ✓ | tasks/parameters |
| $param in pattern filter | expressions/parameters.md | ✓ | tasks/parameters |
| $param in SPAWN | expressions/parameters.md | ✓ | tasks/parameters |
| $param in SET | expressions/parameters.md | ✓ | tasks/parameters |
| $param in list (IN clause) | expressions/parameters.md | ✓ | tasks/parameters |
| $param type inference | expressions/parameters.md | ✗ | — |
| $param bool type | expressions/parameters.md | ✓ | tasks/parameters |
| Missing parameter error | expressions/parameters.md | ✓ | tasks/parameters |

---

## Time Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| now() function | expressions/timestamp_functions.md | ✓ | tasks/bulk_mutations |
| Duration type | types/duration_type.md | ✓ | (level-1: expressions/types) |

---

## Type System

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Union types (`T \| U`) | types/union_type.md | ✓ | humanresources/union_types |
| `any` type in edges | types/any_type.md | ✓ | humanresources/any_types |
| Type checking (`:Type`) | expressions/type_checking.md | ✓ | ecommerce/type_checking |

---

## ID References & Inspection

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `#id` reference syntax | expressions/id_references.md | ✓ | tasks/inspect |
| `#"uuid"` quoted syntax | expressions/id_references.md | ✓ | tasks/inspect |
| INSPECT by ID | statements/inspect.md | ✓ | tasks/inspect |
| INSPECT with projection | statements/inspect.md | ✓ | tasks/inspect |

---

## Administration

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SHOW TYPES | statements/admin.md | ✓ | tasks/admin |
| SHOW EDGES | statements/admin.md | ✓ | tasks/admin |
| SHOW CONSTRAINTS | statements/admin.md | ✓ | tasks/admin |
| CREATE INDEX | statements/admin.md | ✓ | tasks/admin |
| DROP INDEX | statements/admin.md | ✓ | tasks/admin |

---

## Policy & Authorization

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Policy declaration | declarations/policy.md | ✓ | tasks/policy |
| current_actor() | expressions/context_functions.md | ✓ | tasks/policy |
| BEGIN SESSION AS | statements/session.md | ✓ | tasks/policy |
| Ownership-based access | declarations/policy.md | ✓ | tasks/policy |

---

## Error Cases

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `[unique]` violation | modifiers/unique.md | ✓ | ecommerce/uniqueness |
| `[format:]` violation | modifiers/format_validation.md | ✓ | ecommerce/format_validation |
| `[match:]` violation | modifiers/regex_validation.md | ✓ | ecommerce/format_validation |
| `[in:]` violation | modifiers/enum_constraint.md | ✓ | ecommerce/format_validation |
| `[>= N]` violation | modifiers/range_constraint.md | ✓ | ecommerce/format_validation |
| `[<= M]` violation | modifiers/range_constraint.md | ✓ | ecommerce/format_validation |
| `[no_self]` violation | modifiers/no_self.md | ✓ | humanresources/no_self |
| Missing parameter error | expressions/parameters.md | ✓ | tasks/parameters |

---

## Coverage Summary

| Category | Covered | Total | Coverage |
|----------|---------|-------|----------|
| Type Aliases | 5 | 5 | 100% |
| Inheritance | 6 | 6 | 100% |
| Attribute Constraints | 17 | 17 | 100% |
| Edge Features | 5 | 5 | 100% |
| WALK Traversal | 15 | 15 | 100% |
| Query Patterns | 8 | 8 | 100% |
| Bulk Operations | 9 | 9 | 100% |
| LINK Variants | 7 | 7 | 100% |
| UNLINK | 4 | 4 | 100% |
| Parameters | 9 | 10 | 90% |
| Time | 2 | 2 | 100% |
| Type System | 3 | 3 | 100% |
| ID Refs & Inspection | 4 | 4 | 100% |
| Administration | 5 | 5 | 100% |
| Policy/Authorization | 4 | 4 | 100% |
| Errors | 8 | 8 | 100% |
| **Total** | **111** | **112** | **99%** |

---

## Gaps to Address

### Remaining Gap
- `$param` type inference (1 item in Parameters category)

### Completed This Session
- Administration commands (SHOW, CREATE INDEX, DROP INDEX)
- Format validators: phone, iso_date, iso_datetime, ipv4, ipv6
- `[length: N..M]` string length validation
- `#"uuid"` quoted syntax for ID references
- `$param` in list parameters (IN clause)
- Alias chaining (chained type aliases)
- Diamond inheritance resolution
- Edge cardinality constraints
- Referential actions (cascade, unlink, prevent)
- Policy/Authorization (BEGIN SESSION, END SESSION)

---

*Spec references point to files under `specs/` directory.*
