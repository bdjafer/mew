# MEW Implementation: Issues and Missing Features

**Date:** 2026-01-11
**Test Status:** 165/158 unit tests passing (exceeds target)
**End-to-End:** Partial - basic SPAWN/MATCH works, advanced features missing

---

## Executive Summary

The core implementation achieves all 158 acceptance tests, but the **parser does not support the full DSL specification**. Level-1 through Level-5 ontologies cannot be loaded because they use syntax features that are not implemented.

### What Works End-to-End

| Feature | Status |
|---------|--------|
| Load minimal ontology (node/edge definitions) | ✅ Working |
| SPAWN nodes with attributes | ✅ Working |
| MATCH queries with WHERE clause | ✅ Working |
| MATCH with RETURN projections | ✅ Working |
| BEGIN/COMMIT/ROLLBACK transactions | ✅ Working |
| Type registry with node/edge types | ✅ Working |
| Basic attribute modifiers (`[required]`, `[unique]`) | ✅ Working |
| Range constraints (`[>= N]`, `[<= M]`) | ✅ Working |

### What Does NOT Work

| Feature | Status | Blocking |
|---------|--------|----------|
| Nullable types (`String?`) | ❌ Parser | All L1+ ontologies |
| `ontology Name { }` wrapper | ❌ Parser | All L1+ ontologies |
| Inline defaults (`= value`) | ❌ Parser | Most ontologies |
| LINK/KILL/SET/UNLINK mutations | ❌ REPL | Graph modifications |
| Type aliases (`type X = ...`) | ❌ Parser | L2+ ontologies |
| Inheritance (`: Parent`) | ❌ Parser | L2+ ontologies |

---

## Part 1: Parser Missing Features

### 1.1 Nullable Type Suffix (`?`)

**Specification:** `2_DSL.md` Section 11.2
**Severity:** CRITICAL - Blocks all Level-1+ ontologies

The parser does not recognize `?` as a valid token.

```mew
-- Expected (from spec):
node Person {
  nickname: String?,      -- nullable
  age: Int?               -- nullable
}

-- Error:
Parse error at line X, column Y: unexpected character '?'
```

**Files affected:**
- `mew-parser/src/lexer.rs` - No `?` token defined
- `mew-parser/src/parser.rs` - No nullable type handling

**Fix required:** Add `Question` token to lexer, parse `Type?` as nullable in type expressions.

---

### 1.2 Ontology Wrapper Syntax

**Specification:** `2_DSL.md` Section 9
**Severity:** HIGH - Blocks ontology namespace support

The parser has an `Ontology` token but does not parse `ontology Name { ... }` blocks.

```mew
-- Expected:
ontology TaskManagement {
  node Task { ... }
  edge owns(...)
}

-- Error:
Parse error: expected node, edge, constraint, or rule, found ONTOLOGY
```

**Current behavior:** Parser expects definitions directly without wrapper.

**Files affected:**
- `mew-parser/src/parser.rs` - `parse_ontology_def()` doesn't handle `ontology` keyword

---

### 1.3 Inline Default Values

**Specification:** `2_DSL.md` Section 11.5
**Severity:** HIGH - Blocks most real ontologies

The parser doesn't support `attr: Type = value` syntax.

```mew
-- Expected:
node Task {
  status: String = "pending",
  priority: Int = 0,
  active: Bool = true
}

-- Only supported:
node Task {
  status: String [default = "pending"]   -- modifier syntax
}
```

**Files affected:**
- `mew-parser/src/parser.rs` - `parse_attr_def()` doesn't check for `=` after type

---

### 1.4 Type Aliases

**Specification:** `2_DSL.md` Section 8
**Severity:** MEDIUM - Required for L2+ ontologies

Type alias declarations are not implemented.

```mew
-- Expected:
type Email = String [match: "^.+@.+$"]
type Priority = Int [0..10]
type TaskStatus = String [in: ["todo", "done"]]

-- Not parsed at all
```

**Files affected:**
- `mew-parser/src/ast.rs` - No `TypeAliasDef` AST node
- `mew-parser/src/parser.rs` - No `parse_type_alias()` method

---

### 1.5 Type Inheritance

**Specification:** `2_DSL.md` Section 10.3
**Severity:** MEDIUM - Required for L2+ ontologies

Node type inheritance is not supported.

```mew
-- Expected:
node Animal { name: String }
node Dog : Animal { breed: String }  -- inherits name

-- Not parsed
```

**Files affected:**
- `mew-parser/src/ast.rs` - `NodeTypeDef` has no `parents` field
- `mew-parser/src/parser.rs` - No inheritance clause parsing

---

### 1.6 Range Shorthand

**Specification:** `2_DSL.md` Section 11.3.5
**Severity:** MEDIUM

Range shorthand `[N..M]` is not supported (only `[>= N, <= M]`).

```mew
-- Expected:
priority: Int [0..10]        -- shorthand

-- Must use:
priority: Int [>= 0, <= 10]  -- explicit
```

---

### 1.7 Enum Modifier

**Specification:** `2_DSL.md` Section 11.3.6
**Severity:** MEDIUM

Enum constraint syntax is not implemented.

```mew
-- Expected:
status: String [in: ["draft", "active", "archived"]]

-- Not parsed
```

---

### 1.8 Match Modifier

**Specification:** `2_DSL.md` Section 11.3.7
**Severity:** MEDIUM

Regex validation modifier is not implemented.

```mew
-- Expected:
email: String [match: "^.+@.+$"]

-- Not parsed
```

---

### 1.9 Length Modifier

**Specification:** `2_DSL.md` Section 11.3.8
**Severity:** LOW

String length constraint is not implemented.

```mew
-- Expected:
name: String [length: 1..100]

-- Not parsed
```

---

### 1.10 `now()` Function

**Specification:** `2_DSL.md` Section 11.5.1
**Severity:** MEDIUM

The `now()` function for timestamps is not implemented as a parseable expression.

```mew
-- Expected:
created_at: Timestamp = now()
expires_at: Timestamp = now() + 7.days

-- Not parsed
```

---

### 1.11 Edge Advanced Modifiers

**Specification:** `2_DSL.md` Section 12.3+
**Severity:** MEDIUM

Several edge modifiers are not implemented:

| Modifier | Status |
|----------|--------|
| `[acyclic]` | ✅ Implemented |
| `[unique]` | ✅ Implemented |
| `[on_kill: cascade]` | ✅ Implemented |
| `[symmetric]` | ❌ Not implemented |
| `[no_self]` | ❌ Not implemented |
| `[indexed]` | ❌ Not implemented |
| Cardinality `[param -> N..M]` | ❌ Not implemented |

---

### 1.12 Union Types

**Specification:** `2_DSL.md` Section 8.3.2
**Severity:** LOW

Union type expressions are not supported.

```mew
-- Expected:
type Entity = Person | Organization
edge owns(owner: Person | Bot, item: Item)

-- Not parsed
```

---

## Part 2: REPL/Execution Missing Features

### 2.1 Variable Binding Tracking

**Severity:** HIGH - Blocks graph modification

The REPL does not track variable bindings across statements. This prevents:

| Statement | Error |
|-----------|-------|
| `LINK owns(p, t)` | "requires variable binding tracking" |
| `SET t.title = "x"` | "requires variable binding tracking" |
| `KILL p` | "requires variable binding tracking" |
| `UNLINK e` | "requires variable binding tracking" |

**Root cause:** Each statement executes independently without carrying forward variable bindings from SPAWN.

**Required:** Implement session-level binding map that tracks `var_name -> NodeId` across statements.

---

### 2.2 Edge Attribute Support in LINK

When LINK is implemented, edge attributes need to be supported:

```mew
LINK assigned_to(t, p) { assigned_at = now(), role = "owner" }
```

Currently the parser supports this syntax but execution doesn't occur.

---

## Part 3: Ontology Test Results

### Level-1 Ontologies

| File | Status | First Error |
|------|--------|-------------|
| 1S_Bookmarks.mew | ❌ FAIL | `String?` - unexpected `?` |
| 1M_Library.mew | ❌ FAIL | `String?` - unexpected `?` |
| 1L_Contacts.mew | ❌ FAIL | `String?` - unexpected `?` |

**Blocking issues:** Nullable types, ontology wrapper, inline defaults

---

### Level-2 Ontologies

| File | Status | Expected Issues |
|------|--------|-----------------|
| 2S_Tasks.mew | ❌ FAIL | + Type aliases, inheritance |
| 2M_Ecommerce.mew | ❌ FAIL | + Type aliases, `[match:]`, `[in:]` |
| 2L_HumanResources.mew | ❌ FAIL | + All Level-1 + type aliases |

---

### Level-3+ Ontologies

Not tested - expected to fail with same issues plus:
- Complex constraint patterns
- Rule productions
- Hyperedge syntax
- Higher-order edges

---

## Part 4: Implementation Priority

### Immediate (Required for Level-1)

1. **Nullable types (`?`)** - Lexer + Parser change
2. **Inline defaults (`= value`)** - Parser change
3. **Variable binding tracking** - REPL change

### Near-term (Required for Level-2)

4. **Type aliases** - Parser + Compiler change
5. **Type inheritance** - Parser + Registry change
6. **Enum modifier `[in:]`** - Parser change
7. **Match modifier `[match:]`** - Parser change

### Medium-term (Required for Level-3+)

8. **Range shorthand `[N..M]`** - Parser change
9. **Edge cardinality modifiers** - Parser change
10. **`now()` function** - Parser + Evaluator change
11. **Symmetric/no_self edge modifiers** - Parser change

### Low Priority

12. **Ontology wrapper syntax** - Parser change (cosmetic)
13. **Union types** - Parser + Type system change
14. **Length modifier** - Parser change

---

## Part 5: Verified Working Features

### End-to-End Test Results

```
$ cargo run -p mew-repl -- -v test_ontology.mew test_session.mew

Loading: test_ontology.mew
  Types: ["Person", "Task"]
  Edge types: ["owns"]
Ontology loaded: 2 types, 1 edge types

Loading: test_session.mew
Created alice with id 1
Created t1 with id 2
t.title
---------------
"Design API"

(1 rows)
```

### Unit Test Count

```
$ cargo test --workspace 2>&1 | grep "test result"

test result: ok. 165 passed; 0 failed
```

---

## Conclusion

The MEW implementation has a solid foundation with 165 passing unit tests. The core components (Graph, Parser basics, Registry, Query, Mutation, Pattern, Constraint, Rule, Transaction, Journal, Session) are functional.

However, the **parser is the primary bottleneck** - it implements only a subset of the DSL specification. Extending the parser to support nullable types, inline defaults, and type aliases would unlock Level-1 and Level-2 ontology support.

The REPL needs variable binding tracking to enable full CRUD operations on the graph.
