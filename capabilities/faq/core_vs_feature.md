Excellent question. I made implicit choices that deserve explicit criteria.

## Core vs Feature: The Criteria

### Something is **Core** if ALL of these hold:

| Criterion | Rationale |
|-----------|-----------|
| **Compile-time only** | Expands to simple constraints; no special runtime behavior |
| **No Layer 0 additions** | Doesn't need new meta-schema node/edge types |
| **No new syntax** | Uses existing grammar (modifiers in `[]`, standard patterns) |
| **Predictable cost** | O(1) per operation, no warnings needed |
| **Universal** | Concept exists in SQL/every schema language |
| **No external dependencies** | Doesn't require regex engine, format validators, etc. |

### Something is a **Feature** if ANY of these hold:

| Criterion | Examples |
|-----------|----------|
| **Needs Layer 0 additions** | `_ExistsExpr`, `_TransitiveModifier`, `_EdgeRefTypeExpr` |
| **Needs new syntax** | `EXISTS(...)`, `causes+(a,b)`, `AS` binding, `??` operator |
| **Runtime behavior** | `[symmetric]` changes storage/matching; `[acyclic]` runs traversal |
| **Performance implications** | `[acyclic]` is O(V+E) per LINK — needs warning |
| **Complex timing semantics** | Cardinality checked at COMMIT, not per-operation |
| **Domain-specific/opinionated** | `[format: email]` — what is a "valid email"? |
| **External dependencies** | `[match: "regex"]` needs regex engine |
| **Changes enforcement model** | `[soft]` warns instead of rejects |

---

## Applying the Criteria

### Why these are **Core**:

| Modifier | Justification |
|----------|---------------|
| `[required]` | SQL `NOT NULL`. Compiles to simple null-check constraint. |
| `[unique]` | SQL `UNIQUE`. Compiles to pairwise inequality constraint. |
| `[indexed]` | Engine hint, no constraint, universal concept. |
| `[>= N]`, `[<= N]` | SQL `CHECK`. Simple comparison constraint. |
| `[N..M]` | Sugar for `>= N, <= M`. |
| `[in: [...]]` | SQL `CHECK ... IN (...)`. Expands to OR constraint. |
| `[length: N..M]` | Uses built-in `length()`. Simple constraint. |
| `[no_self_loops]` | Simple `a.id != b.id` constraint. |
| `[unique]` on edges | Same as node uniqueness. |

### Why these are **Features**:

| Modifier/Syntax | Violates | Justification |
|-----------------|----------|---------------|
| `[symmetric]` | Runtime behavior | Changes edge storage (canonicalization) and matching semantics |
| `[acyclic]` | Performance, Layer 0 | O(V+E) traversal; needs `_TransitiveModifier` |
| `[task -> 0..1]` | Timing semantics | Checked at COMMIT, not per-op; max vs min have different timing |
| `[on_kill_*: cascade]` | Runtime behavior, Layer 0 | Generates rules, cascade depth tracking |
| `[format: email]` | Domain-specific, dependencies | What's a valid email? Requires validator set. |
| `[match: "..."]` | External dependency | Requires regex engine |
| `[soft]` | Enforcement model | Changes constraint behavior fundamentally |
| `EXISTS(...)` | New syntax, Layer 0 | New expression type, scoping rules |
| `causes+(a,b)` | New syntax, Layer 0, performance | Transitive closure, depth limits |
| `edge<T>`, `AS` | New syntax, Layer 0 | New type expression, edge binding |
| `COALESCE`, `??` | New syntax, Layer 0 | New expression type |

---

## Gray Areas (Debatable)

| Item | Currently | Could argue |
|------|-----------|-------------|
| `[in: [...]]` | Core | Feature: enum validation is "opinionated" |
| `[length: N..M]` | Core | Feature: requires `length()` function |
| `[no_self_loops]` | Core | Feature: could be `[reflexive: false]` with more options |
| `now()` in defaults | Core | Feature: dynamic evaluation has subtle semantics |

---
