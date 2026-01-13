# Missing Level 1 Test Coverage

Based on the analysis of specifications (`1_LANGUAGE.md`, `3_SCHEMA.md`, `4_QUERIES.md`, `5_MUTATIONS.md`) and existing test files, the following areas require test coverage.

### 1. Data Types Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `Float` type | ❌ NOT TESTED | 1_LANGUAGE.md §3.4 |
| Duration type literals (`1.day`, `30.minutes`) | ❌ NOT TESTED | 1_LANGUAGE.md §3.7 |
| Timestamp literals (`@2024-01-15`) | ❌ NOT TESTED | 1_LANGUAGE.md §3.6.2 |

### 2. String Functions/Operations Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `++` (string concatenation) | ❌ NOT TESTED | 1_LANGUAGE.md §5.5.3 |
| `starts_with(s, prefix)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `ends_with(s, suffix)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `contains(s, substring)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `lower(s)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `upper(s)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `trim(s)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |
| `substring(s, start, len)` | ❌ NOT TESTED | 1_LANGUAGE.md §3.2.2 |

### 3. Arithmetic/Numeric Functions Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Arithmetic in queries (`+`, `-`, `*`, `/`, `%`) | ❌ NOT TESTED | 1_LANGUAGE.md §5.5.2 |
| Unary minus `-x` | ❌ NOT TESTED | 1_LANGUAGE.md §5.6 |
| `abs(n)` | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| `min(a, b)` / `max(a, b)` (scalar functions) | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| `floor(f)` / `ceil(f)` / `round(f)` | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |

### 4. Aggregation Gaps
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `COLLECT(x)` aggregation | ❌ NOT TESTED | 4_QUERIES.md §2.6.4 |
| `COUNT(DISTINCT x)` | ❌ NOT TESTED | 4_QUERIES.md §2.6.4 |
| `SUM`/`AVG` with Float values | ❌ NOT TESTED | 4_QUERIES.md §2.6.4 |
| `MIN`/`MAX` with String values (lexicographic) | ❌ NOT TESTED | 4_QUERIES.md §2.6.4 |
| Aggregation on empty set | ❌ NOT TESTED | - |

### 5. Null Handling Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `coalesce(x, default)` | ❌ NOT TESTED | 1_LANGUAGE.md §5.10.1 |
| `??` operator | ❌ NOT TESTED | 1_LANGUAGE.md §5.10.1 |
| `is_null(x)` function | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| Null propagation (`null + 1` → `null`) | ❌ NOT TESTED | 1_LANGUAGE.md §3.9 |
| `null = null` → `true` | ❌ NOT TESTED | 1_LANGUAGE.md §3.9 |

### 6. Timestamp Functions Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `now()` in attribute defaults | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| `year(t)`, `month(t)`, `day(t)` | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| `hour(t)`, `minute(t)`, `second(t)` | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| Timestamp arithmetic (`t + duration`, `t - t`) | ❌ NOT TESTED | 1_LANGUAGE.md §3.6.1 |

### 7. RETURNING Clause Variants Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `SPAWN ... RETURNING id` | ❌ NOT TESTED | 5_MUTATIONS.md §1.6 |
| `SPAWN ... RETURNING *` | ❌ NOT TESTED | 5_MUTATIONS.md §1.6 |
| `SPAWN ... RETURNING attr1, attr2` | ❌ NOT TESTED | 5_MUTATIONS.md §1.6 |
| `LINK ... RETURNING id` | ❌ NOT TESTED | 5_MUTATIONS.md §3.9 |
| `KILL ... RETURNING id` | ❌ NOT TESTED | 5_MUTATIONS.md §2.3 |
| `SET ... RETURNING attr` | ❌ NOT TESTED | 5_MUTATIONS.md §5.3 |

### 8. Transaction Control Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `BEGIN ... COMMIT` | ❌ NOT TESTED | 5_MUTATIONS.md §6 |
| `BEGIN ... ROLLBACK` | ❌ NOT TESTED | 5_MUTATIONS.md §6.6 |
| Auto-commit mode (implicit) | ❌ NOT TESTED | 5_MUTATIONS.md §6.4 |
| Transaction failure rollback | ❌ NOT TESTED | 5_MUTATIONS.md §6.6.2 |

### 9. Debug Statements Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `EXPLAIN` | ❌ NOT TESTED | 4_QUERIES.md §5.2 |
| `PROFILE` | ❌ NOT TESTED | 4_QUERIES.md §5.3 |

### 10. Expression Edge Cases Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Operator precedence (`a + b * c`) | ❌ NOT TESTED | 1_LANGUAGE.md §2.8.1 |
| Parenthesized expressions | ❌ NOT TESTED | 1_LANGUAGE.md §5.9 |
| Boolean short-circuit (`false and x`) | ❌ NOT TESTED | 1_LANGUAGE.md §3.5 |

### 11. ID Reference Syntax Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `#"uuid-format-id"` quoted IDs | ❌ NOT TESTED | 4_QUERIES.md §4.2.1 |

### 12. SET Block Syntax Not Tested
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `SET #id { attr1 = v1, attr2 = v2 }` | ❌ NOT TESTED | 5_MUTATIONS.md §5.4.1 |

---

## Summary

*   **Total missing test categories:** 12
*   **Estimated missing individual test cases:** 50+

### Most Critical Gaps
1.  **Transactions:** Core ACID feature not tested.
2.  **String functions:** Half the string API is untested.
3.  **Null handling:** `Coalesce` and null semantics untested.
4.  **Float/Duration types:** Scalar types have no coverage.
5.  **RETURNING clause:** Mutation result handling is untested.
6.  **COLLECT aggregation:** Key aggregation function untested.


---
Here is the **Level 2 Gap Analysis** report, reformatted into clean, structured Markdown.

***

# Level 2 Gap Analysis

## What Level 2 Should Cover (from LEVELS.md)

*   **Schema:** Type aliases, inheritance (`:`), multiple inheritance, `[unique]`, `[indexed]`, `[format: email|url|uuid|...]`, `[match: "regex"]`, `[in: [...]]`, `[>= N]/[<= M]`, `[length: N..M]`, `[no_self]`, edge attributes.
*   **Queries:** `WALK` traversal (`FOLLOW`, `UNTIL`, `RETURN NODES/EDGES/PATH/TERMINAL`), `OPTIONAL MATCH`, polymorphic queries, anonymous targets (`_`).
*   **Mutations:** Bulk operations (`KILL`/`SET` with `MATCH` subquery), `LINK IF NOT EXISTS`, inline `SPAWN` in `LINK`.
*   **Time:** `now()`, `wall_time()` for real-world timestamps.
*   **Authorization:** Simple ownership-based access control, `current_actor()`.
*   **Parameters:** `$param` syntax, prepared statements (`PREPARE`/`EXECUTE`).

---

## ⚠️ CRITICAL: Misplaced Tests (Level 3 Features in Level 2)

The following features are currently tested in Level 2 files but belong to Level 3 according to `LEVELS.md`.

| Feature | Current Location | Per LEVELS.md |
| :--- | :--- | :--- |
| Transitive `+`/`*` patterns | `tasks/transitive.mew` | **Level 3** |
| `EXISTS` / `NOT EXISTS` | `humanresources/exists_patterns.mew`<br>`tasks/not_exists.mew` | **Level 3** |

> **Action:** These tests should be moved to Level 3 or the ontologies redesigned.

---

## Currently Tested (Correctly Placed)

| Feature | Status | File(s) | Spec Reference |
| :--- | :---: | :--- | :--- |
| Single inheritance (`:`) | ✅ Tested | `inheritance.mew`, `deep_inheritance.mew` | 3_SCHEMA.md §2.3 |
| Polymorphic queries | ✅ Tested | `inheritance.mew` | 4_QUERIES.md §2.3 |
| `[unique]` constraint | ✅ Tested | `uniqueness.mew` | 3_SCHEMA.md §4.2.1 |
| `[no_self]` constraint | ✅ Tested | `no_self.mew`, `blocking.mew` | 3_SCHEMA.md §4.3.2 |
| Edge attributes | ✅ Tested | `edge_attributes.mew` | 3_SCHEMA.md §3.3 |
| `OPTIONAL MATCH` | ✅ Tested | `advanced_queries.mew` | 4_QUERIES.md §2.5 |
| Bulk `KILL`/`SET` | ✅ Tested | `bulk_mutations.mew` | 5_MUTATIONS.md §2.6, §5.6 |
| `SPAWN ... RETURNING` | ✅ Tested | `bulk_mutations.mew` | 5_MUTATIONS.md §1.6 |

---

## Missing Level 2 Tests

### 1. Schema: Type Aliases
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Type alias definition | ❌ NOT TESTED | 3_SCHEMA.md §2.2 |
| Type alias in attribute declaration | ❌ NOT TESTED | 3_SCHEMA.md §2.2 |
| Type alias validation (alias inherits constraints) | ❌ NOT TESTED | 3_SCHEMA.md §2.2 |

> *Note: Type aliases exist in ontologies (e.g., `type SKU = String [match: "..."]`) but are NOT directly tested as a feature.*

### 2. Schema: Multiple Inheritance
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Node inheriting from 2+ parent types | ❌ NOT TESTED | 3_SCHEMA.md §2.3.2 |
| Diamond inheritance resolution | ❌ NOT TESTED | 3_SCHEMA.md §2.3.2 |
| Querying multi-parent types polymorphically | ❌ NOT TESTED | 3_SCHEMA.md §2.3.2 |

> *Note: No ontology currently has multiple inheritance. Need to add to an ontology.*

### 3. Schema: [indexed] Modifier
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `[indexed]` declaration | ❌ NOT TESTED | 3_SCHEMA.md §4.2.2 |
| Index performance (implicit, not directly testable) | ❌ NOT TESTED | 3_SCHEMA.md §4.2.2 |

### 4. Schema: [format: ...] Validators
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `[format: email]` validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.3 |
| `[format: url]` validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.3 |
| `[format: uuid]` validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.3 |
| `[format: ...]` with invalid values | ❌ NOT TESTED | 3_SCHEMA.md §4.1.3 |

### 5. Schema: [match: "regex"] Validation
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Valid regex match | ❌ NOT TESTED | 3_SCHEMA.md §4.1.4 |
| Invalid regex match (rejection) | ❌ NOT TESTED | 3_SCHEMA.md §4.1.4 |
| Regex boundary cases | ❌ NOT TESTED | 3_SCHEMA.md §4.1.4 |

> *Note: Regex patterns exist in ontologies (SKU, email, slug) but validation is NOT tested.*

### 6. Schema: [in: [...]] Enum Validation
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Valid enum value | ❌ NOT TESTED | 3_SCHEMA.md §4.1.5 |
| Invalid enum value (rejection) | ❌ NOT TESTED | 3_SCHEMA.md §4.1.5 |
| SET to invalid enum value | ❌ NOT TESTED | 3_SCHEMA.md §4.1.5 |

> *Note: Enums exist (e.g., ProductStatus, EmploymentStatus, Status) but validation NOT tested.*

### 7. Schema: Range Constraints [>= N] / [<= M]
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `[>= N]` minimum value check | ❌ NOT TESTED | 3_SCHEMA.md §4.1.6 |
| `[<= M]` maximum value check | ❌ NOT TESTED | 3_SCHEMA.md §4.1.6 |
| `[N..M]` range check (inclusive) | ❌ NOT TESTED | 3_SCHEMA.md §4.1.6 |
| Boundary value testing | ❌ NOT TESTED | 3_SCHEMA.md §4.1.6 |

> *Note: Range constraints exist (Price `[>= 0]`, Rating `[1..5]`, management_level `[1..5]`) but are NOT tested.*

### 8. Schema: [length: N..M] String Length
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Minimum length validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.7 |
| Maximum length validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.7 |
| Exact length validation | ❌ NOT TESTED | 3_SCHEMA.md §4.1.7 |

> *Note: No current ontology uses `[length: ...]`. May need to add.*

### 9. Queries: WALK Traversal
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `WALK` with `FOLLOW` clause | ❌ NOT TESTED | 4_QUERIES.md §3 |
| `WALK` with `UNTIL` termination | ❌ NOT TESTED | 4_QUERIES.md §3.2 |
| `RETURN NODES` | ❌ NOT TESTED | 4_QUERIES.md §3.3 |
| `RETURN EDGES` | ❌ NOT TESTED | 4_QUERIES.md §3.3 |
| `RETURN PATH` | ❌ NOT TESTED | 4_QUERIES.md §3.3 |
| `RETURN TERMINAL` | ❌ NOT TESTED | 4_QUERIES.md §3.3 |
| `WALK` with depth limits | ❌ NOT TESTED | 4_QUERIES.md §3.4 |

### 10. Queries: Anonymous Targets (_)
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `_` as edge target | ❌ NOT TESTED | 4_QUERIES.md §2.2.3 |
| `_` in pattern matching | ❌ NOT TESTED | 4_QUERIES.md §2.2.3 |
| `_` with `NOT EXISTS` | ❌ NOT TESTED | 4_QUERIES.md §2.2.3 |

### 11. Mutations: LINK IF NOT EXISTS
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| Create edge when not exists | ❌ NOT TESTED | 5_MUTATIONS.md §3.8 |
| No-op when edge exists | ❌ NOT TESTED | 5_MUTATIONS.md §3.8 |
| `RETURNING CREATED` flag | ❌ NOT TESTED | 5_MUTATIONS.md §3.8.2 |
| Idempotency verification | ❌ NOT TESTED | 5_MUTATIONS.md §3.8.3 |

### 12. Mutations: Inline SPAWN in LINK
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `LINK edge(SPAWN Type {...}, #existing)` | ❌ NOT TESTED | 5_MUTATIONS.md §3.4.4 |
| `AS` binding for spawned nodes | ❌ NOT TESTED | 5_MUTATIONS.md §3.4.5 |
| Multiple inline SPAWNs | ❌ NOT TESTED | 5_MUTATIONS.md §3.4.5 |
| Rollback on LINK failure | ❌ NOT TESTED | 5_MUTATIONS.md §3.4.5 |

### 13. Time: wall_time()
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `wall_time()` function call | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |
| `wall_time()` vs `now()` distinction | ❌ NOT TESTED | 1_LANGUAGE.md §5.7.1 |

### 14. Authorization: Simple Ownership
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `current_actor()` function | ❌ NOT TESTED | 4_QUERIES.md §4.2 |
| Ownership-based read filtering | ❌ NOT TESTED | 4_QUERIES.md §4.2 |
| Ownership-based write authorization | ❌ NOT TESTED | 4_QUERIES.md §4.2 |

### 15. Parameters: $param Syntax
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `$param` in `WHERE` clause | ❌ NOT TESTED | 5_MUTATIONS.md §5.10 |
| `$param` in `SPAWN` attributes | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.2 |
| `$param` in `LINK` targets | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.2 |
| Parameter type inference | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.4 |
| Missing parameter error | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.4 |

### 16. Parameters: Prepared Statements
| Feature | Status | Spec Reference |
| :--- | :---: | :--- |
| `PREPARE` statement | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.6 |
| `EXECUTE` with parameters | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.6 |
| `DROP PREPARED` | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.6 |
| Plan caching behavior | ❌ NOT TESTED | 5_MUTATIONS.md §5.10.6 |

---

## Summary

| Category | Total Features | Tested | Missing | Misplaced |
| :--- | :--- | :--- | :--- | :--- |
| Schema (type aliases) | 3 | 0 | 3 | 0 |
| Schema (inheritance) | 3 | 1 | 2 | 0 |
| Schema (`[indexed]`) | 1 | 0 | 1 | 0 |
| Schema (`[format]`) | 4 | 0 | 4 | 0 |
| Schema (`[match]`) | 3 | 0 | 3 | 0 |
| Schema (`[in]`) | 3 | 0 | 3 | 0 |
| Schema (range) | 4 | 0 | 4 | 0 |
| Schema (`[length]`) | 3 | 0 | 3 | 0 |
| Queries (`WALK`) | 7 | 0 | 7 | 0 |
| Queries (anonymous `_`) | 3 | 0 | 3 | 0 |
| Mutations (`LINK IF NOT EXISTS`) | 4 | 0 | 4 | 0 |
| Mutations (inline `SPAWN`) | 4 | 0 | 4 | 0 |
| Time (`wall_time()`) | 2 | 0 | 2 | 0 |
| Authorization | 3 | 0 | 3 | 0 |
| Parameters (`$param`) | 5 | 0 | 5 | 0 |
| Parameters (`PREPARE`) | 4 | 0 | 4 | 0 |
| **TOTAL** | **56** | **1** | **55** | **0** |

**Additionally:** 3 operation files contain Level 3 features that should be moved:
*   `tasks/transitive.mew` → Move to Level 3
*   `tasks/not_exists.mew` → Move to Level 3
*   `humanresources/exists_patterns.mew` → Move to Level 3

---

## Recommended Action Plan

1.  **Move misplaced Level 3 tests** to appropriate Level 3 ontologies.
2.  **Add constraint validation tests** (`format_validation.mew`).
3.  **Add WALK traversal tests** (New file needed).
4.  **Add anonymous target tests** (New file needed).
5.  **Add LINK IF NOT EXISTS tests** (New file needed).
6.  **Add inline SPAWN in LINK tests** (New file needed).
7.  **Add parameterized query tests** (New file needed).
8.  **Add authorization tests** (if ontologies support it; may need ontology updates).
9.  **Consider adding multiple inheritance** to one ontology for testing.