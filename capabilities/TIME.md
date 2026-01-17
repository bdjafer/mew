# MEW Time Model

**Version:** 1.0
**Status:** Capability
**Deferred to:** v2
**Scope:** Time representation, tick semantics, logical time

---

# Part I: Overview

## 1.1 The Problem

MEW graphs evolve through mutations and rules. But *when* do things happen?

- What does `now()` return?
- When do rules fire?
- How does external time relate to internal time?
- Can execution be replayed deterministically?

## 1.2 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Dual time** | Both logical and wall-clock time available |
| **Explicit advancement** | Logical time advances via tick, not implicitly |
| **Semantics over strategy** | Spec defines observable behavior, not execution strategy |
| **Determinism when needed** | Logical time enables replay |

## 1.3 What This Spec Defines vs. Leaves to Implementation

| Spec Defines (Semantics) | Implementation Decides (Strategy) |
|--------------------------|-----------------------------------|
| What `wall_time()` returns | When ticks are scheduled |
| What `logical_time()` returns | How mutations are batched |
| What happens during a tick | Fixed vs variable tick rate |
| Safety limits (recommendations) | Specific limit values |
| Determinism requirements | Execution optimizations |

---

# Part II: Time Model

## 2.1 Two Kinds of Time

MEW provides two distinct time concepts:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TIME CONCEPTS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WALL TIME                           LOGICAL TIME                   │
│  ─────────                           ────────────                   │
│                                                                      │
│  wall_time() → Timestamp             logical_time() → Int           │
│                                                                      │
│  • Real-world clock                  • Abstract counter             │
│  • Monotonic                         • Monotonic                    │
│  • Advances continuously             • Advances via tick            │
│  • Non-deterministic                 • Deterministic                │
│  • For: timestamps, TTLs,            • For: ordering, causality,    │
│         real-world deadlines               simulation, replay       │
│                                                                      │
│  Cannot control                      Controlled by tick             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 wall_time()

```
wall_time() → Timestamp
```

Returns the current real-world time (milliseconds since epoch).

**Properties:**
- Always available
- Monotonic (never goes backward)
- Not controlled by MEW
- Non-deterministic (replaying same mutations yields different wall times)

**Use cases:**
- `created_at: Timestamp = wall_time()`
- `WHERE wall_time() > t.expires_at`
- Audit trails, logging

**Not suitable for:**
- Deterministic simulation
- Replay/debugging
- Ordering guarantees

## 2.3 logical_time()

```
logical_time() → Int
logical_time(ScopeRef) → Int   -- v2+, for interior scopes
```

Returns the current logical tick counter.

**Properties:**
- Starts at 0
- Only advances via tick
- Same within a tick (all operations see same value)
- Deterministic (same inputs → same logical times)

**Use cases:**
- `turn: Int = logical_time()`
- `WHERE logical_time() - t.created_tick > 100`
- Game turns, simulation steps
- Causal ordering

**Scoped form (v2+):** `logical_time(SELF)`, `logical_time(PARENT)`, `logical_time(ROOT)`, `logical_time(#node)` query specific interior scopes. See [INTERIOR.md](./v2/INTERIOR.md).

## 2.4 now()

For convenience, `now()` is an alias with configurable binding:

```
now() → Timestamp | Int
```

**Binding options:**
- `wall` (default): `now()` = `wall_time()`
- `logical`: `now()` = `logical_time()`

**Rationale:** Most users expect `now()` to mean wall clock. Simulations can rebind it to logical time for determinism.

## 2.5 Time in Different Contexts

| Context | `wall_time()` | `logical_time()` |
|---------|---------------|------------------|
| Attribute default | Allowed | Allowed |
| Rule condition | Allowed | Allowed |
| Rule action | Allowed | Allowed |
| Query condition | Allowed | Allowed |
| Constraint condition | **Not allowed** | Allowed |

**Constraint restriction:** Constraints must be deterministic. `wall_time()` in constraints would make validation non-repeatable.

```
-- COMPILE ERROR
constraint recent_only:
  t: Task
  => wall_time() - t.created_at < 86400000
-- Error: wall_time() not allowed in constraints

-- OK: use logical time
constraint not_too_old:
  t: Task
  => logical_time() - t.created_tick < 100
```

---

# Part III: Tick Model

## 3.1 What Is a Tick?

A **tick** is the fundamental unit of logical time advancement.

```
┌─────────────────────────────────────────────────────────────────────┐
│                          TICK SEMANTICS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Before tick N:                                                     │
│    logical_time() = N                                               │
│    Graph in state S_N                                               │
│                                                                      │
│  During tick N:                                                     │
│    1. PROCESS - Apply pending mutations                             │
│    2. REACT   - Fire rules (to quiescence or limit)                │
│    3. VALIDATE - Check constraints                                  │
│                                                                      │
│  After tick N:                                                      │
│    logical_time() = N + 1                                          │
│    Graph in state S_{N+1}                                          │
│                                                                      │
│  All operations within tick N see logical_time() = N               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Tick Processing Cycle

Each tick executes these phases in order:

| Phase | Description |
|-------|-------------|
| **PROCESS** | Apply pending mutations to graph |
| **REACT** | Fire rules until quiescence (no rules can fire) or limit reached |
| **VALIDATE** | Check constraints; violation triggers rollback |
| **ADVANCE** | Increment logical time counter |

If any phase fails, the tick rolls back and logical time does not advance.

## 3.3 Quiescence

**Quiescence** = state where no rules can fire (no patterns match, or all matches processed).

Rules fire repeatedly within a tick until quiescence or a safety limit is reached. This ensures the graph reaches a stable state before time advances.

## 3.4 Manual Tick Control

The `TICK` statement provides explicit time control:

```
TICK        -- advance by 1
TICK 10     -- advance by 10 (runs 10 tick cycles)
```

Use cases:
- Simulations with explicit time steps
- Turn-based games
- Testing with controlled time

## 3.5 Tick Scope

By default, MEW uses a **global tick** — one tick counter for the entire graph.

For scoped timing with independent tick rates, use **world interiors** from [INTERIOR.md](./v2/INTERIOR.md). Each interior can have its own time relationship to its parent (shared, independent, ratio).

---

# Part IV: Execution Strategy

## 4.1 Implementation Choice

**The spec defines tick semantics. How and when ticks are scheduled is an implementation decision.**

Valid implementation strategies:

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Immediate** | Tick after every mutation | Traditional database |
| **Fixed** | Tick every Xms regardless of load | Game servers (60fps) |
| **Variable** | Tick when processing completes | Adaptive, best throughput |
| **Manual** | Only tick on explicit `TICK` statement | Simulations, testing |

All are valid. The spec only requires that tick semantics (the processing cycle) are respected.

## 4.2 Safety Limits (Recommendations)

Implementations SHOULD enforce limits to prevent runaway execution:

| Limit | Description |
|-------|-------------|
| **max_rule_depth** | Maximum nested rule trigger depth |
| **max_rule_actions** | Maximum rule actions per tick |
| **max_execution_time** | Wall-clock budget per tick |

These are implementation decisions. The spec does not mandate specific values.

---

# Part V: Determinism

## 5.1 Determinism Requirements

For deterministic execution (same inputs → same outputs):

**Required:**
- Use logical time, not wall time
- Fixed rule priority ordering
- Deterministic ID allocation
- Same mutation order

**Prohibited:**
- `wall_time()` in rules/constraints
- Non-deterministic functions (`random()`, `uuid()`)
- Time-based limits (use count-based instead)

## 5.2 Replay

With deterministic configuration, execution can be replayed:

```
Record:
  • Initial graph state
  • Sequence of (tick, mutations) pairs

Replay:
  • Load initial state
  • For each (tick, mutations):
      Apply mutations
      TICK
  • Final state matches original
```

Use cases:
- Debugging ("what happened at tick 1000?")
- Testing (deterministic test cases)
- Replication (followers replay leader's log)

---

# Part VI: Interaction with Other Systems

## 6.1 Rules

> **Cross-reference:** See [RULES.md](./RULES.md) for rule semantics.

Rules fire during the REACT phase of each tick. All rules see the same `logical_time()` within a tick.

## 6.2 Watches

> **Cross-reference:** See [WATCH.md](./WATCH.md) for watch semantics.

Watch visibility options (`committed`, `immediate`) determine when subscribers see changes relative to tick boundaries.

## 6.3 Interiors (v2+)

> **Cross-reference:** See [INTERIOR.md](./v2/INTERIOR.md) for interior semantics.

World interiors can have independent tick rates. `logical_time(ScopeRef)` queries a specific interior's tick counter.

---

# Part VII: Grammar

```ebnf
(* Time Expressions *)
TimeExpr         = "wall_time" "(" ")"
                 | "logical_time" "(" ScopeRef? ")"
                 | "now" "(" ")" ;

ScopeRef         = "SELF" | "PARENT" | "ROOT" | NodeRef ;

(* Tick Statement *)
TickStmt         = "TICK"
                 | "TICK" IntLiteral ;
```

---

# Part VIII: Layer 0

## 8.1 Nodes

```mew
node _TickState [sealed, singleton] {
  global_tick: Int = 0,
  last_tick_at: Timestamp?
}
```

The `_TickState` singleton tracks the current logical time counter.

## 8.2 Constraints

```mew
constraint _tick_non_negative:
  s: _TickState => s.global_tick >= 0
```

---

# Part IX: Errors

## 9.1 Tick Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9001 | INVALID_TICK_AMOUNT | Zero or negative tick amount | `INVALID_TICK_AMOUNT: Tick amount must be positive, got {amount}` |

## 9.2 Determinism Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9031 | WALL_TIME_IN_DETERMINISTIC | wall_time() used in deterministic context | `WALL_TIME_IN_DETERMINISTIC: wall_time() is not allowed in deterministic mode` |
| E9034 | NONDETERMINISTIC_FUNCTION | Non-deterministic function in constraint | `NONDETERMINISTIC_FUNCTION: Function '{name}' is non-deterministic and cannot be used in constraints` |

## 9.3 Execution Limit Errors (Implementation-Defined)

Implementations may define errors for safety limit violations:

| Suggested Code | Name | Condition |
|----------------|------|-----------|
| E9021 | RULE_DEPTH_EXCEEDED | Rule nesting exceeds limit |
| E9022 | RULE_ACTIONS_EXCEEDED | Rule actions exceed limit |
| E9023 | EXECUTION_TIMEOUT | Execution time exceeded |

Specific error codes and messages are implementation decisions.

---

# Part X: Summary

## 10.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Wall time** | Real-world clock, continuous, non-deterministic |
| **Logical time** | Abstract counter, advances via tick, deterministic |
| **Tick** | Unit of logical time; executes PROCESS → REACT → VALIDATE → ADVANCE |
| **Quiescence** | State where no rules can fire |

## 10.2 What Spec Defines

- `wall_time()`, `logical_time()`, `now()` semantics
- Tick processing cycle (phases and ordering)
- `TICK` statement for manual time control
- Constraint restriction on `wall_time()`
- Determinism requirements

## 10.3 What Implementation Decides

- When ticks occur (immediate, fixed, variable, manual)
- How mutations are batched
- Specific safety limit values
- Execution optimizations

---

*End of MEW Time Model Capability*
