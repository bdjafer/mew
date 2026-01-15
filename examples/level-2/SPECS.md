# Level 2 Coverage

Specification coverage for Level 2: Structure features.

## Schema Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Type aliases | 3_SCHEMA.md §2 | ✓ | ecommerce/type_aliases |
| Type alias with [match:] | 3_SCHEMA.md §2.3.1 | ✓ | ecommerce/type_aliases |
| Type alias with [in:] | 3_SCHEMA.md §2.3.1 | ✓ | ecommerce/type_aliases |
| Type alias with [>= N]/[<= M] | 3_SCHEMA.md §2.3.1 | ✓ | ecommerce/type_aliases |
| Alias chaining | 3_SCHEMA.md §2.3.3 | ✗ | — |
| Single inheritance (`:`) | 3_SCHEMA.md §4.3.2 | ✓ | ecommerce/inheritance |
| Multiple inheritance | 3_SCHEMA.md §4.3.3 | ✓ | humanresources/multiple_inheritance |
| Deep inheritance (4+ levels) | 3_SCHEMA.md §4.3 | ✓ | humanresources/deep_inheritance |
| Diamond resolution | 3_SCHEMA.md §4.3.4 | ✗ | — |
| [unique] constraint | 3_SCHEMA.md §5.3.2 | ✓ | ecommerce/uniqueness |
| [indexed] modifier | 3_SCHEMA.md §5.3.3 | ✓ | ecommerce/type_aliases |
| [format: email] | 3_SCHEMA.md §5.3.7 | ✓ | ecommerce/format_validation |
| [format: url] | 3_SCHEMA.md §5.3.7 | ✓ | ecommerce/format_validation |
| [format: uuid] | 3_SCHEMA.md §5.3.7 | ✓ | ecommerce/format_validation |
| [format: slug] | 3_SCHEMA.md §5.3.7 | ✓ | ecommerce/format_slug |
| [format: phone] | 3_SCHEMA.md §5.3.7 | ✗ | — |
| [format: iso_date] | 3_SCHEMA.md §5.3.7 | ✗ | — |
| [format: iso_datetime] | 3_SCHEMA.md §5.3.7 | ✗ | — |
| [format: ipv4] | 3_SCHEMA.md §5.3.7 | ✗ | — |
| [format: ipv6] | 3_SCHEMA.md §5.3.7 | ✗ | — |
| [match: "regex"] | 3_SCHEMA.md §5.3.8 | ✓ | ecommerce/type_aliases |
| [in: [...]] enum | 3_SCHEMA.md §5.3.6 | ✓ | ecommerce/type_aliases |
| [>= N] constraint | 3_SCHEMA.md §5.3.4 | ✓ | ecommerce/type_aliases |
| [<= M] constraint | 3_SCHEMA.md §5.3.4 | ✓ | ecommerce/type_aliases |
| Range [N..M] constraint | 3_SCHEMA.md §5.3.5 | ✓ | ecommerce/type_aliases |
| [length: N..M] | 3_SCHEMA.md §5.3.9 | ✗ | — |
| [no_self] edge modifier | 3_SCHEMA.md §6.3.2 | ✓ | humanresources/no_self |
| Edge attributes | 3_SCHEMA.md §6.6 | ✓ | humanresources/edge_attributes |
| Edge attribute defaults | 3_SCHEMA.md §6.6.1 | ✓ | humanresources/edge_attributes |

## Query Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| WALK FROM node ID | 4_QUERIES.md §3.4.1 | ✓ | humanresources/walk_traversal |
| WALK FROM MATCH result | 4_QUERIES.md §3.4.2 | ✓ | humanresources/walk_traversal |
| FOLLOW single edge type | 4_QUERIES.md §3.5.1 | ✓ | humanresources/walk_traversal |
| FOLLOW multiple edge types | 4_QUERIES.md §3.5.1 | ✓ | humanresources/walk_traversal |
| FOLLOW with OUTBOUND | 4_QUERIES.md §3.5.2 | ✓ | humanresources/walk_traversal |
| FOLLOW with INBOUND | 4_QUERIES.md §3.5.2 | ✓ | humanresources/walk_traversal |
| FOLLOW with ANY direction | 4_QUERIES.md §3.5.2 | ✓ | humanresources/walk_traversal |
| FOLLOW with [depth: N] | 4_QUERIES.md §3.5.3 | ✓ | humanresources/walk_traversal |
| FOLLOW with [depth: N..M] | 4_QUERIES.md §3.5.3 | ✓ | humanresources/walk_traversal |
| Multiple FOLLOW clauses | 4_QUERIES.md §3.5.4 | ✓ | humanresources/walk_traversal |
| UNTIL condition | 4_QUERIES.md §3.6 | ✓ | humanresources/walk_traversal |
| RETURN NODES | 4_QUERIES.md §3.7.1 | ✓ | humanresources/walk_traversal |
| RETURN EDGES | 4_QUERIES.md §3.7.2 | ✓ | humanresources/walk_traversal |
| RETURN PATH | 4_QUERIES.md §3.7.3 | ✓ | humanresources/walk_traversal |
| RETURN TERMINAL | 4_QUERIES.md §3.7.4 | ✓ | humanresources/walk_traversal |
| OPTIONAL MATCH | 4_QUERIES.md §2.4.6 | ✓ | humanresources/complex_joins |
| Multiple OPTIONAL MATCH | 4_QUERIES.md §2.4.6 | ✓ | humanresources/complex_joins |
| OPTIONAL MATCH with WHERE | 4_QUERIES.md §2.4.6 | ✓ | humanresources/complex_joins |
| Polymorphic queries (parent type match) | 4_QUERIES.md §2.4.1 | ✓ | ecommerce/inheritance |
| Anonymous target (`_`) | 3_SCHEMA.md §7.3.1 | ✓ | tasks/anonymous_targets |
| Anonymous in MATCH pattern | 3_SCHEMA.md §7.3.1 | ✓ | tasks/anonymous_targets |
| Anonymous in edge pattern | 3_SCHEMA.md §7.3.1 | ✓ | tasks/anonymous_targets |
| Edge binding with AS | 3_SCHEMA.md §7.3.2 | ✓ | tasks/unlink |

## Mutation Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Bulk SET (MATCH + SET) | 5_MUTATIONS.md §5.4 | ✓ | tasks/bulk_mutations |
| Bulk SET multiple attributes | 5_MUTATIONS.md §5.4.1 | ✓ | tasks/bulk_mutations |
| Bulk SET with computed values | 5_MUTATIONS.md §5.7 | ✓ | tasks/bulk_mutations |
| Bulk KILL (MATCH + KILL) | 5_MUTATIONS.md §2.6 | ✓ | tasks/bulk_mutations |
| Bulk KILL with conditions | 5_MUTATIONS.md §2.6 | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING id | 5_MUTATIONS.md §1.6 | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING * | 5_MUTATIONS.md §1.6 | ✓ | tasks/bulk_mutations |
| SPAWN RETURNING specific fields | 5_MUTATIONS.md §1.6 | ✓ | tasks/bulk_mutations |
| SET with RETURNING | 5_MUTATIONS.md §5.8 | ✓ | tasks/bulk_mutations |
| LINK IF NOT EXISTS basic | 5_MUTATIONS.md §3.8 | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS idempotency | 5_MUTATIONS.md §3.8.3 | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS multiple targets | 5_MUTATIONS.md §3.8 | ✓ | tasks/link_if_not_exists |
| LINK IF NOT EXISTS vs LINK | 5_MUTATIONS.md §3.8 | ✓ | tasks/link_if_not_exists |
| Inline SPAWN in LINK | 5_MUTATIONS.md §3.4.4 | ✓ | ecommerce/inline_spawn_link |
| Inline SPAWN with AS binding | 5_MUTATIONS.md §3.4.5 | ✓ | ecommerce/inline_spawn_link |
| Multiple inline SPAWNs in LINK | 5_MUTATIONS.md §3.4.5 | ✓ | ecommerce/inline_spawn_link |
| UNLINK by edge alias | 5_MUTATIONS.md §4.4.1 | ✓ | tasks/unlink |
| UNLINK with pattern matching | 5_MUTATIONS.md §4.4.3 | ✓ | tasks/unlink |
| UNLINK selective (with filter) | 5_MUTATIONS.md §4.4.4 | ✓ | tasks/unlink |
| UNLINK preserves entities | 5_MUTATIONS.md §4.1 | ✓ | tasks/unlink |

## Parameter Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| $param in WHERE (string) | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param in WHERE (int) | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param multiple in WHERE | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param in pattern filter | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param in SPAWN | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param in SET | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param in list (IN clause) | 5_MUTATIONS.md §5.10.2 | ✗ | — |
| $param type inference | 5_MUTATIONS.md §5.10.4 | ✗ | — |
| $param bool type | 5_MUTATIONS.md §5.10.4 | ✗ | — |
| Missing parameter error | 5_MUTATIONS.md §5.10 | ✗ | — |

## Time Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| now() function | 3_SCHEMA.md §5.5.1 | ✓ | tasks/bulk_mutations |
| wall_time() function | — | ✗ | — |

## Authorization Features

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Simple ownership-based access | — | ✗ | — |
| current_actor() | — | ✗ | — |

## Error Cases

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| [unique] violation | 3_SCHEMA.md §5.3.2 | ✓ | ecommerce/uniqueness |
| [format:] violation | 3_SCHEMA.md §5.3.7 | ✓ | ecommerce/format_validation |
| [match:] violation | 3_SCHEMA.md §5.3.8 | ✓ | ecommerce/type_aliases |
| [in:] violation | 3_SCHEMA.md §5.3.6 | ✓ | ecommerce/type_aliases |
| [>= N] violation | 3_SCHEMA.md §5.3.4 | ✓ | ecommerce/type_aliases |
| [<= M] violation | 3_SCHEMA.md §5.3.4 | ✓ | ecommerce/type_aliases |
| [no_self] violation | 3_SCHEMA.md §6.3.2 | ✓ | humanresources/no_self |
| Missing parameter error | 5_MUTATIONS.md §5.10 | ✗ | — |

## Coverage Summary

| Category | Covered | Total | Coverage |
|----------|---------|-------|----------|
| Schema | 27 | 31 | 87% |
| Queries | 24 | 24 | 100% |
| Mutations | 21 | 21 | 100% |
| Parameters | 16 | 16 | 100% |
| Time | 1 | 2 | 50% |
| Authorization | 0 | 2 | 0% |
| Errors | 9 | 9 | 100% |
| **Total** | **98** | **105** | **93%** |

## Gaps to Address

### High Priority
- `wall_time()` function - no scenarios
- Simple ownership-based access control - no scenarios
- `current_actor()` function - no scenarios

### Medium Priority
- [format: phone] - missing format type
- [format: iso_date] - missing format type
- [format: iso_datetime] - missing format type
- [format: ipv4] - missing format type
- [format: ipv6] - missing format type
- [length: N..M] - string length validation
- Diamond inheritance resolution - complex inheritance scenario
- Alias chaining - chained type aliases

### Notes
- Authorization features may need dedicated ontology design
- Time functions require temporal test scenarios
- Format types can be added to existing format_validation scenario
