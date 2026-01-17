# Level 1 Coverage

Coverage map for Level 1 (Fundamentals) features.

---

## Schema

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| `ontology` declaration | declarations/ontology.md | ✓ | all ontologies (implicit) |
| `node` declaration | declarations/node.md | ✓ | bookmarks/ontology |
| `edge` declaration | declarations/edge.md | ✓ | bookmarks/ontology |
| Node patterns (`var: Type`) | patterns/node_patterns.md | ✓ | all MATCH queries (implicit) |
| `String` type | core/1_LANGUAGE.md §3 | ✓ | expressions/types |
| `Int` type | core/1_LANGUAGE.md §3 | ✓ | expressions/types |
| `Float` type | core/1_LANGUAGE.md §3 | ✓ | expressions/types |
| `Bool` type | core/1_LANGUAGE.md §3 | ✓ | expressions/types |
| `Timestamp` type | core/1_LANGUAGE.md §3 | ✓ | expressions/timestamps |
| `Duration` type | literals/duration_literals.md | ✓ | expressions/types |
| `[required]` modifier | modifiers/required.md | ✓ | bookmarks/errors_comprehensive |
| Default values (`= value`) | modifiers/default_values.md | ✓ | bookmarks/spawn_variants |
| Optional types (`T?`) | types/optional_type.md | ✓ | expressions/nulls |
| Doc comments (`---`) | core/3_DSL.md §7 | ✓ | contacts/ontology |

---

## Mutations

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SPAWN basic | statements/spawn.md | ✓ | bookmarks/spawn_variants |
| SPAWN with all attributes | statements/spawn.md | ✓ | bookmarks/spawn_variants |
| SPAWN with defaults applied | statements/spawn.md | ✓ | bookmarks/spawn_variants |
| SPAWN RETURNING id | statements/returning.md | ✓ | expressions/returning |
| SPAWN RETURNING * | statements/returning.md | ✓ | expressions/returning |
| SPAWN RETURNING attributes | statements/returning.md | ✓ | expressions/returning |
| SPAWN validation - missing required | statements/spawn.md | ✓ | bookmarks/errors_comprehensive |
| SPAWN validation - type mismatch | statements/spawn.md | ✓ | bookmarks/errors_comprehensive |
| KILL by ID | statements/kill.md | ✓ | library/loan_management |
| KILL - edge cascade behavior | statements/kill.md | ✓ | bookmarks/extreme_cases |
| KILL RETURNING id | statements/returning.md | ✓ | expressions/returning |
| KILL validation - not found | statements/kill.md | ✓ | bookmarks/errors_comprehensive |
| LINK basic | statements/link.md | ✓ | bookmarks/edge_operations |
| LINK with attributes | statements/link.md | ✓ | contacts/edge_attributes |
| LINK RETURNING id | statements/returning.md | ✓ | expressions/returning |
| LINK RETURNING * | statements/returning.md | ✓ | expressions/returning |
| LINK validation - node not found | statements/link.md | ✓ | bookmarks/errors_comprehensive |
| LINK validation - type mismatch | statements/link.md | ✓ | bookmarks/errors_comprehensive |
| UNLINK by edge ID | statements/unlink.md | ✓ | bookmarks/edge_operations |
| UNLINK by pattern | statements/unlink.md | ✓ | library/loan_management |
| UNLINK validation - not found | statements/unlink.md | ✓ | bookmarks/errors_comprehensive |
| SET single attribute | statements/set.md | ✓ | bookmarks/set_variants |
| SET multiple attributes | statements/set.md | ✓ | bookmarks/set_variants |
| SET edge attribute | statements/set.md | ✓ | contacts/edge_attributes |
| SET RETURNING | statements/returning.md | ✓ | expressions/returning |
| SET validation - attribute not found | statements/set.md | ✓ | bookmarks/errors_comprehensive |
| SET validation - type mismatch | statements/set.md | ✓ | bookmarks/errors_comprehensive |
| SET validation - required to null | statements/set.md | ✓ | bookmarks/errors_comprehensive |

---

## Queries

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| MATCH single node type | statements/match.md | ✓ | bookmarks/queries |
| MATCH multiple node types | statements/match.md | ✓ | contacts/multi_entity |
| MATCH with edge pattern | patterns/edge_patterns.md | ✓ | bookmarks/edge_operations |
| MATCH with edge alias (AS) | patterns/edge_patterns.md | ✓ | contacts/edge_attributes |
| WHERE with equality (=) | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with inequality (!=) | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with < operator | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with > operator | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with <= operator | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with >= operator | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with AND | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with OR | statements/match.md | ✓ | bookmarks/query_filtering |
| WHERE with NOT | statements/match.md | ✓ | contacts/query_complex |
| WHERE with null check | expressions/null_handling.md | ✓ | bookmarks/query_filtering |
| WHERE with parentheses | statements/match.md | ✓ | bookmarks/query_filtering |
| EXISTS subquery | expressions/exists.md | ✓ | bookmarks/query_exists |
| NOT EXISTS subquery | expressions/exists.md | ✓ | bookmarks/query_exists |
| EXISTS with conditions | expressions/exists.md | ✓ | bookmarks/query_exists |
| RETURN whole nodes | statements/match.md | ✓ | bookmarks/queries |
| RETURN specific attributes | statements/match.md | ✓ | bookmarks/queries |
| RETURN with alias (AS) | statements/match.md | ✓ | bookmarks/query_ordering |
| RETURN * | statements/match.md | ✓ | contacts/multi_entity |
| DISTINCT | statements/distinct.md | ✓ | bookmarks/query_ordering |
| ORDER BY single field | statements/order_by.md | ✓ | bookmarks/query_ordering |
| ORDER BY multiple fields | statements/order_by.md | ✓ | bookmarks/query_ordering |
| ORDER BY ASC | statements/order_by.md | ✓ | bookmarks/query_ordering |
| ORDER BY DESC | statements/order_by.md | ✓ | bookmarks/query_ordering |
| LIMIT | statements/limit_offset.md | ✓ | bookmarks/query_ordering |
| OFFSET | statements/limit_offset.md | ✓ | bookmarks/query_ordering |

---

## Aggregations

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| COUNT(x) | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| COUNT(*) | expressions/aggregations.md | ✓ | contacts/edge_attributes |
| COUNT(DISTINCT x) | expressions/aggregations.md | ✓ | bookmarks/query_ordering |
| SUM(x) | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| AVG(x) | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| MIN(x) | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| MAX(x) | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| COLLECT(x) | expressions/aggregations.md | ✓ | expressions/aggregations |
| COLLECT with limit | expressions/aggregations.md | ✓ | expressions/aggregations |
| Multiple aggregations | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| Aggregation with grouping | expressions/aggregations.md | ✓ | bookmarks/query_aggregations |
| Aggregation with ORDER BY | statements/order_by.md | ✓ | expressions/aggregations |
| Aggregation on empty set | expressions/aggregations.md | ✓ | expressions/aggregations |

---

## Expressions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Arithmetic: addition (+) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: subtraction (-) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: multiplication (*) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: division (/) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: modulo (%) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: unary minus (-) | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Arithmetic: operator precedence | core/1_LANGUAGE.md §5 | ✓ | expressions/arithmetic |
| Comparison: equality (=) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Comparison: inequality (!=) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Comparison: less than (<) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Comparison: greater than (>) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Comparison: less or equal (<=) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Comparison: greater or equal (>=) | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Logical: AND | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Logical: OR | core/1_LANGUAGE.md §5 | ✓ | bookmarks/query_filtering |
| Logical: NOT | core/1_LANGUAGE.md §5 | ✓ | expressions/nulls |
| String: concatenation (++) | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: length() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: starts_with() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: ends_with() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: contains() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: lower() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: upper() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: trim() | expressions/string_functions.md | ✓ | bookmarks/string_functions |
| String: substring() | expressions/string_functions.md | ✓ | expressions/strings |
| Numeric: abs() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Numeric: min() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Numeric: max() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Numeric: floor() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Numeric: ceil() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Numeric: round() | expressions/numeric_functions.md | ✓ | expressions/arithmetic |
| Timestamp: now() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: year() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: month() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: day() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: hour() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: minute() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp: second() | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp arithmetic (+/-) | expressions/timestamp_functions.md | ✓ | expressions/timestamps |
| Timestamp literals | literals/timestamp_literals.md | ✓ | expressions/timestamps |
| Duration literals | literals/duration_literals.md | ✓ | expressions/types |
| Null: IS NULL | expressions/null_handling.md | ✓ | expressions/nulls |
| Null: coalesce() | expressions/null_handling.md | ✓ | expressions/nulls |
| Null: ?? operator | expressions/null_handling.md | ✓ | expressions/nulls |
| Null propagation | expressions/null_handling.md | ✓ | expressions/nulls |

---

## Transactions

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Auto-commit mode | statements/transactions.md | ✓ | expressions/transactions |
| BEGIN | statements/transactions.md | ✓ | expressions/transactions |
| COMMIT | statements/transactions.md | ✓ | expressions/transactions |
| ROLLBACK | statements/transactions.md | ✓ | expressions/transactions |
| BEGIN READ COMMITTED | statements/transactions.md | ✓ | expressions/transactions |
| BEGIN SERIALIZABLE | statements/transactions.md | ✓ | expressions/transactions |
| Transaction atomicity | statements/transactions.md | ✓ | expressions/transactions |
| Multiple operations in txn | statements/transactions.md | ✓ | expressions/transactions |

---

## Debug

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| EXPLAIN basic query | statements/explain.md | ✓ | expressions/debug |
| EXPLAIN with WHERE | statements/explain.md | ✓ | expressions/debug |
| EXPLAIN with join | statements/explain.md | ✓ | expressions/debug |
| EXPLAIN with aggregation | statements/explain.md | ✓ | expressions/debug |
| EXPLAIN with ORDER BY/LIMIT | statements/explain.md | ✓ | expressions/debug |
| PROFILE basic query | statements/explain.md | ✓ | expressions/debug |
| PROFILE with WHERE | statements/explain.md | ✓ | expressions/debug |
| PROFILE with join | statements/explain.md | ✓ | expressions/debug |
| PROFILE with aggregation | statements/explain.md | ✓ | expressions/debug |
| PROFILE with ORDER BY/LIMIT | statements/explain.md | ✓ | expressions/debug |

---

## Errors

| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| Missing required field | statements/spawn.md | ✓ | bookmarks/errors_comprehensive |
| Type mismatch in SPAWN | statements/spawn.md | ✓ | bookmarks/errors_comprehensive |
| Type mismatch in SET | statements/set.md | ✓ | bookmarks/errors_comprehensive |
| Required set to null | statements/set.md | ✓ | bookmarks/errors_comprehensive |
| Invalid attribute access | statements/set.md | ✓ | bookmarks/errors_comprehensive |
| Node not found | statements/kill.md | ✓ | bookmarks/errors_comprehensive |
| Edge not found | statements/unlink.md | ✓ | bookmarks/errors_comprehensive |
| Invalid LINK types | statements/link.md | ✓ | bookmarks/errors_comprehensive |
| Invalid LINK arity | statements/link.md | ✓ | bookmarks/errors_comprehensive |
| Query type not found | statements/match.md | ✓ | bookmarks/errors_comprehensive |
| Query attribute not found | statements/match.md | ✓ | bookmarks/errors_comprehensive |
| Query edge not found | statements/match.md | ✓ | bookmarks/errors_comprehensive |
| Query type mismatch | statements/match.md | ✓ | bookmarks/errors_comprehensive |

---

## Coverage Summary

| Category | Total Features | Covered | Coverage |
|----------|----------------|---------|----------|
| Schema | 12 | 12 | 100% |
| Mutations | 28 | 28 | 100% |
| Queries | 30 | 30 | 100% |
| Aggregations | 13 | 13 | 100% |
| Expressions | 46 | 46 | 100% |
| Transactions | 8 | 8 | 100% |
| Debug | 10 | 10 | 100% |
| Errors | 13 | 13 | 100% |
| **Total** | **160** | **160** | **100%** |

---

*Spec references point to files under `specs/` directory.*
