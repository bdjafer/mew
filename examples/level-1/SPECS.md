# Level 1 Coverage

Coverage map for Level 1 (Fundamentals) features.

---

## Schema

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `node` declaration | 3_SCHEMA.md §4 | ✓ | bookmarks/ontology |
| `edge` declaration | 3_SCHEMA.md §6 | ✓ | bookmarks/ontology |
| `String` type | 1_LANGUAGE.md §3.2 | ✓ | expressions/types |
| `Int` type | 1_LANGUAGE.md §3.3 | ✓ | expressions/types |
| `Float` type | 1_LANGUAGE.md §3.4 | ✓ | expressions/types |
| `Bool` type | 1_LANGUAGE.md §3.5 | ✓ | expressions/types |
| `Timestamp` type | 1_LANGUAGE.md §3.6 | ✓ | expressions/timestamps |
| `Duration` type | 1_LANGUAGE.md §3.7 | ✓ | expressions/types |
| `[required]` modifier | 3_SCHEMA.md §5.3.1 | ✓ | bookmarks/errors_comprehensive |
| Default values (`= value`) | 3_SCHEMA.md §5.5 | ✓ | bookmarks/spawn_variants |
| Optional types (`T?`) | 1_LANGUAGE.md §4.3 | ✓ | expressions/nulls |
| Doc comments (`---`) | 1_LANGUAGE.md §2.3.3 | ✓ | contacts/ontology |

---

## Mutations

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SPAWN basic | 5_MUTATIONS.md §1.1 | ✓ | bookmarks/spawn_variants |
| SPAWN with all attributes | 5_MUTATIONS.md §1.3 | ✓ | bookmarks/spawn_variants |
| SPAWN with defaults applied | 5_MUTATIONS.md §1.4.3 | ✓ | bookmarks/spawn_variants |
| SPAWN RETURNING id | 5_MUTATIONS.md §1.6 | ✓ | expressions/returning |
| SPAWN RETURNING * | 5_MUTATIONS.md §1.6 | ✓ | expressions/returning |
| SPAWN RETURNING attributes | 5_MUTATIONS.md §1.6 | ✓ | expressions/returning |
| SPAWN validation - missing required | 5_MUTATIONS.md §1.5 | ✓ | bookmarks/errors_comprehensive |
| SPAWN validation - type mismatch | 5_MUTATIONS.md §1.5 | ✓ | bookmarks/errors_comprehensive |
| KILL by ID | 5_MUTATIONS.md §2.2 | ✓ | library/loan_management |
| KILL - edge cascade behavior | 5_MUTATIONS.md §2.4 | ✓ | bookmarks/extreme_cases |
| KILL RETURNING id | 5_MUTATIONS.md §2.3 | ✓ | expressions/returning |
| KILL validation - not found | 5_MUTATIONS.md §2.3 | ✓ | bookmarks/errors_comprehensive |
| LINK basic | 5_MUTATIONS.md §3.1 | ✓ | bookmarks/edge_operations |
| LINK with attributes | 5_MUTATIONS.md §3.5 | ✓ | contacts/edge_attributes |
| LINK RETURNING id | 5_MUTATIONS.md §3.9 | ✓ | expressions/returning |
| LINK RETURNING * | 5_MUTATIONS.md §3.9 | ✓ | expressions/returning |
| LINK validation - node not found | 5_MUTATIONS.md §3.6 | ✓ | bookmarks/errors_comprehensive |
| LINK validation - type mismatch | 5_MUTATIONS.md §3.6 | ✓ | bookmarks/errors_comprehensive |
| UNLINK by edge ID | 5_MUTATIONS.md §4.2 | ✓ | bookmarks/edge_operations |
| UNLINK by pattern | 5_MUTATIONS.md §4.3 | ✓ | library/loan_management |
| UNLINK validation - not found | 5_MUTATIONS.md §4.3 | ✓ | bookmarks/errors_comprehensive |
| SET single attribute | 5_MUTATIONS.md §5.2 | ✓ | bookmarks/set_variants |
| SET multiple attributes | 5_MUTATIONS.md §5.4.1 | ✓ | bookmarks/set_variants |
| SET edge attribute | 5_MUTATIONS.md §5.3 | ✓ | contacts/edge_attributes |
| SET RETURNING | 5_MUTATIONS.md §5.3 | ✓ | expressions/returning |
| SET validation - attribute not found | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |
| SET validation - type mismatch | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |
| SET validation - required to null | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |

---

## Queries

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| MATCH single node type | 4_QUERIES.md §2.3 | ✓ | bookmarks/queries |
| MATCH multiple node types | 4_QUERIES.md §2.4.2 | ✓ | contacts/multi_entity |
| MATCH with edge pattern | 4_QUERIES.md §2.4.2 | ✓ | bookmarks/edge_operations |
| MATCH with edge alias (AS) | 4_QUERIES.md §2.4.2 | ✓ | contacts/edge_attributes |
| WHERE with equality (=) | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/query_filtering |
| WHERE with inequality (!=) | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/query_filtering |
| WHERE with < operator | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/query_filtering |
| WHERE with > operator | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/query_filtering |
| WHERE with <= operator | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/query_filtering |
| WHERE with >= operator | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/query_filtering |
| WHERE with AND | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/query_filtering |
| WHERE with OR | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/query_filtering |
| WHERE with NOT | 4_QUERIES.md §2.5.1 | ✓ | contacts/query_complex |
| WHERE with null check | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/query_filtering |
| WHERE with parentheses | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/query_filtering |
| EXISTS subquery | 4_QUERIES.md §2.5.3 | ✓ | bookmarks/query_exists |
| NOT EXISTS subquery | 4_QUERIES.md §2.5.3 | ✓ | bookmarks/query_exists |
| EXISTS with conditions | 4_QUERIES.md §2.5.3 | ✓ | bookmarks/query_exists |
| RETURN whole nodes | 4_QUERIES.md §2.6.1 | ✓ | bookmarks/queries |
| RETURN specific attributes | 4_QUERIES.md §2.6.1 | ✓ | bookmarks/queries |
| RETURN with alias (AS) | 4_QUERIES.md §2.6.1 | ✓ | bookmarks/query_ordering |
| RETURN * | 4_QUERIES.md §2.6.1 | ✓ | contacts/multi_entity |
| DISTINCT | 4_QUERIES.md §2.6.2 | ✓ | bookmarks/query_ordering |
| ORDER BY single field | 4_QUERIES.md §2.7 | ✓ | bookmarks/query_ordering |
| ORDER BY multiple fields | 4_QUERIES.md §2.7 | ✓ | bookmarks/query_ordering |
| ORDER BY ASC | 4_QUERIES.md §2.7 | ✓ | bookmarks/query_ordering |
| ORDER BY DESC | 4_QUERIES.md §2.7 | ✓ | bookmarks/query_ordering |
| LIMIT | 4_QUERIES.md §2.8 | ✓ | bookmarks/query_ordering |
| OFFSET | 4_QUERIES.md §2.8 | ✓ | bookmarks/query_ordering |

---

## Aggregations

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| COUNT(x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| COUNT(*) | 4_QUERIES.md §2.6.4 | ✓ | contacts/edge_attributes |
| COUNT(DISTINCT x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_ordering |
| SUM(x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| AVG(x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| MIN(x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| MAX(x) | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| COLLECT(x) | 4_QUERIES.md §2.6.4 | ✓ | expressions/aggregations |
| COLLECT with limit | 4_QUERIES.md §2.6.4.1 | ✓ | expressions/aggregations |
| Multiple aggregations | 4_QUERIES.md §2.6.4 | ✓ | bookmarks/query_aggregations |
| Aggregation with grouping | 4_QUERIES.md §2.6.5 | ✓ | bookmarks/query_aggregations |
| Aggregation with ORDER BY | 4_QUERIES.md §2.7 | ✓ | expressions/aggregations |
| Aggregation on empty set | 4_QUERIES.md §2.6.4 | ✓ | expressions/aggregations |

---

## Expressions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Arithmetic: addition (+) | 1_LANGUAGE.md §5.5.2 | ✓ | expressions/arithmetic |
| Arithmetic: subtraction (-) | 1_LANGUAGE.md §5.5.2 | ✓ | expressions/arithmetic |
| Arithmetic: multiplication (*) | 1_LANGUAGE.md §5.5.2 | ✓ | expressions/arithmetic |
| Arithmetic: division (/) | 1_LANGUAGE.md §5.5.2 | ✓ | expressions/arithmetic |
| Arithmetic: modulo (%) | 1_LANGUAGE.md §5.5.2 | ✓ | expressions/arithmetic |
| Arithmetic: unary minus (-) | 1_LANGUAGE.md §5.6 | ✓ | expressions/arithmetic |
| Arithmetic: operator precedence | 1_LANGUAGE.md §2.8.1 | ✓ | expressions/arithmetic |
| Comparison: equality (=) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Comparison: inequality (!=) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Comparison: less than (<) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Comparison: greater than (>) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Comparison: less or equal (<=) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Comparison: greater or equal (>=) | 1_LANGUAGE.md §5.5.1 | ✓ | bookmarks/query_filtering |
| Logical: AND | 1_LANGUAGE.md §5.5.4 | ✓ | bookmarks/query_filtering |
| Logical: OR | 1_LANGUAGE.md §5.5.4 | ✓ | bookmarks/query_filtering |
| Logical: NOT | 1_LANGUAGE.md §5.6 | ✓ | expressions/nulls |
| String: concatenation (++) | 1_LANGUAGE.md §5.5.3 | ✓ | bookmarks/string_functions |
| String: length() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: starts_with() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: ends_with() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: contains() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: lower() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: upper() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: trim() | 1_LANGUAGE.md §3.2.2 | ✓ | bookmarks/string_functions |
| String: substring() | 1_LANGUAGE.md §3.2.2 | ✓ | expressions/strings |
| Numeric: abs() | 1_LANGUAGE.md §3.3.1 | ✓ | expressions/arithmetic |
| Numeric: min() | 1_LANGUAGE.md §3.3.1 | ✓ | expressions/arithmetic |
| Numeric: max() | 1_LANGUAGE.md §3.3.1 | ✓ | expressions/arithmetic |
| Numeric: floor() | 1_LANGUAGE.md §3.4.1 | ✓ | expressions/arithmetic |
| Numeric: ceil() | 1_LANGUAGE.md §3.4.1 | ✓ | expressions/arithmetic |
| Numeric: round() | 1_LANGUAGE.md §3.4.1 | ✓ | expressions/arithmetic |
| Timestamp: now() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: year() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: month() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: day() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: hour() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: minute() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp: second() | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp arithmetic (+/-) | 1_LANGUAGE.md §3.6.1 | ✓ | expressions/timestamps |
| Timestamp literals | 1_LANGUAGE.md §3.6.2 | ✓ | expressions/timestamps |
| Duration literals | 1_LANGUAGE.md §3.7.1 | ✓ | expressions/types |
| Null: is_null() | 1_LANGUAGE.md §5.7.1 | ✓ | expressions/nulls |
| Null: coalesce() | 1_LANGUAGE.md §5.10.1 | ✓ | expressions/nulls |
| Null: ?? operator | 1_LANGUAGE.md §5.10.1 | ✓ | expressions/nulls |
| Null propagation | 1_LANGUAGE.md §3.9 | ✓ | expressions/nulls |

---

## Transactions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Auto-commit mode | 5_MUTATIONS.md §6.4 | ✓ | expressions/transactions |
| BEGIN | 5_MUTATIONS.md §6.2 | ✓ | expressions/transactions |
| COMMIT | 5_MUTATIONS.md §6.2 | ✓ | expressions/transactions |
| ROLLBACK | 5_MUTATIONS.md §6.6.1 | ✓ | expressions/transactions |
| BEGIN READ COMMITTED | 5_MUTATIONS.md §6.7.1 | ✓ | expressions/transactions |
| BEGIN SERIALIZABLE | 5_MUTATIONS.md §6.7.2 | ✓ | expressions/transactions |
| Transaction atomicity | 5_MUTATIONS.md §6.3 | ✓ | expressions/transactions |
| Multiple operations in txn | 5_MUTATIONS.md §6.5 | ✓ | expressions/transactions |

---

## Debug

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| EXPLAIN basic query | 6_SYSTEM.md §3.2 | ✓ | expressions/debug |
| EXPLAIN with WHERE | 6_SYSTEM.md §3.2 | ✓ | expressions/debug |
| EXPLAIN with join | 6_SYSTEM.md §3.2 | ✓ | expressions/debug |
| EXPLAIN with aggregation | 6_SYSTEM.md §3.2 | ✓ | expressions/debug |
| EXPLAIN with ORDER BY/LIMIT | 6_SYSTEM.md §3.2 | ✓ | expressions/debug |
| PROFILE basic query | 6_SYSTEM.md §3.3 | ✓ | expressions/debug |
| PROFILE with WHERE | 6_SYSTEM.md §3.3 | ✓ | expressions/debug |
| PROFILE with join | 6_SYSTEM.md §3.3 | ✓ | expressions/debug |
| PROFILE with aggregation | 6_SYSTEM.md §3.3 | ✓ | expressions/debug |
| PROFILE with ORDER BY/LIMIT | 6_SYSTEM.md §3.3 | ✓ | expressions/debug |

---

## Errors

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Missing required field | 5_MUTATIONS.md §1.5 | ✓ | bookmarks/errors_comprehensive |
| Type mismatch in SPAWN | 5_MUTATIONS.md §1.5 | ✓ | bookmarks/errors_comprehensive |
| Type mismatch in SET | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |
| Required set to null | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |
| Invalid attribute access | 5_MUTATIONS.md §5.5 | ✓ | bookmarks/errors_comprehensive |
| Node not found | 5_MUTATIONS.md §2.3 | ✓ | bookmarks/errors_comprehensive |
| Edge not found | 5_MUTATIONS.md §4.3 | ✓ | bookmarks/errors_comprehensive |
| Invalid LINK types | 5_MUTATIONS.md §3.6 | ✓ | bookmarks/errors_comprehensive |
| Invalid LINK arity | 5_MUTATIONS.md §3.6 | ✓ | bookmarks/errors_comprehensive |
| Query type not found | 4_QUERIES.md §2.4.1 | ✓ | bookmarks/errors_comprehensive |
| Query attribute not found | 4_QUERIES.md §2.5.2 | ✓ | bookmarks/errors_comprehensive |
| Query edge not found | 4_QUERIES.md §2.4.2 | ✓ | bookmarks/errors_comprehensive |
| Query type mismatch | 4_QUERIES.md §2.5.1 | ✓ | bookmarks/errors_comprehensive |

---

## Coverage Summary

| Category | Total Features | Covered | Coverage |
|----------|----------------|---------|----------|
| Schema | 12 | 12 | 100% |
| Mutations | 30 | 30 | 100% |
| Queries | 29 | 29 | 100% |
| Aggregations | 13 | 13 | 100% |
| Expressions | 46 | 46 | 100% |
| Transactions | 8 | 8 | 100% |
| Debug | 10 | 10 | 100% |
| Errors | 13 | 13 | 100% |
| **Total** | **161** | **161** | **100%** |

---

*Generated from specs/specification/*.md and examples/level-1/**/operations/*.mew*
