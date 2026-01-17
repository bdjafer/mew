---
spec: time-functions
version: "1.0"
status: draft
category: expression
capability: time
requires: [expressions]
priority: essential
---

# Spec: Time Functions

## Overview

Time functions provide access to temporal values within MEW expressions. MEW provides two distinct time concepts: wall time (real-world clock) and logical time (abstract tick counter), plus a configurable `now()` alias.

Wall time is non-deterministic and reflects the real-world clock. Logical time is deterministic and only advances via tick operations.

## Syntax

### Grammar

```ebnf
TimeExpr =
    "wall_time" "(" ")"
  | "logical_time" "(" ScopeRef? ")"
  | "now" "(" ")"

ScopeRef =
    "SELF" | "PARENT" | "ROOT" | NodeRef
```

### Keywords

| Keyword | Context |
|---------|---------|
| `wall_time` | Expression - returns current wall clock timestamp |
| `logical_time` | Expression - returns logical tick counter |
| `now` | Expression - configurable alias (defaults to wall_time) |
| `SELF` | ScopeRef - current interior scope (v2+) |
| `PARENT` | ScopeRef - parent interior scope (v2+) |
| `ROOT` | ScopeRef - root/global scope (v2+) |

### Examples

```
-- Attribute default using wall time
node Task {
  created_at: Timestamp = wall_time()
}

-- Query with wall time comparison
MATCH t: Task
WHERE wall_time() > t.expires_at
RETURN t

-- Game turn tracking with logical time
node GameEntity {
  spawn_tick: Int = logical_time()
}

-- Rule using logical time for age check
rule expire_old_entities:
  e: GameEntity
  WHERE logical_time() - e.spawn_tick > 100
  => KILL e

-- Using now() (defaults to wall_time)
node Event {
  timestamp: Timestamp = now()
}
```

## Semantics

### Function Signatures

| Function | Returns | Description |
|----------|---------|-------------|
| `wall_time()` | Timestamp | Current real-world time (ms since epoch) |
| `logical_time()` | Int | Current global logical tick counter |
| `logical_time(scope)` | Int | Logical tick counter for specified scope (v2+) |
| `now()` | Timestamp \| Int | Configurable alias (see below) |

### wall_time()

Returns the current real-world wall clock time as a Timestamp (milliseconds since Unix epoch).

**Properties:**
- Always available
- Monotonic (never goes backward within a session)
- Not controlled by MEW
- Non-deterministic (replaying same mutations yields different wall times)

**Use cases:**
- `created_at: Timestamp = wall_time()` - audit timestamps
- `WHERE wall_time() > t.expires_at` - TTL checks
- Logging and audit trails

**Not suitable for:**
- Deterministic simulation
- Replay/debugging
- Causal ordering guarantees

### logical_time()

Returns the current logical tick counter as an Int.

**Properties:**
- Starts at 0
- Only advances via tick
- Same within a tick (all operations see same value)
- Deterministic (same inputs produce same logical times)

**Use cases:**
- `turn: Int = logical_time()` - game turns
- `WHERE logical_time() - t.created_tick > 100` - tick-based expiry
- Causal ordering
- Simulation steps

### logical_time(ScopeRef)

Returns the logical tick counter for a specific interior scope. This is a v2+ feature that requires the INTERIOR capability.

> **Cross-reference:** See [INTERIOR.md](../../capabilities/v2/INTERIOR.md) for time configuration options (`shared`, `independent`, `ratio(N)`).

**Scope References:**
- `SELF` - current interior's tick counter
- `PARENT` - parent interior's tick counter
- `ROOT` - global/root tick counter
- `NodeRef` - tick counter for the interior owned by that node

### now()

A configurable alias that defaults to `wall_time()`.

**Binding options:**
- `wall` (default): `now()` = `wall_time()`, returns Timestamp
- `logical`: `now()` = `logical_time()`, returns Int

**Rationale:** Most users expect `now()` to mean wall clock. Simulations can rebind it to logical time for determinism.

### Evaluation Context

Time functions are evaluated at different points depending on context:

| Context | When Evaluated |
|---------|----------------|
| Attribute default | At SPAWN/LINK time |
| Rule condition (WHERE) | At match time |
| Rule action (SET) | At execution time |
| Query (MATCH WHERE) | At query time |
| Constraint condition | At validation time |

## Context Restrictions

### wall_time() in Constraints

`wall_time()` is **not allowed** in constraint conditions. Constraints must be deterministic for validation to be repeatable.

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

### Valid Usage Summary

| Context | wall_time() | logical_time() | now() |
|---------|-------------|----------------|-------|
| Attribute default | Yes | Yes | Yes |
| Rule condition | Yes | Yes | Yes |
| Rule action | Yes | Yes | Yes |
| Query condition | Yes | Yes | Yes |
| Constraint condition | **No** | Yes | Only if bound to logical |

## Layer 0

The `_TickState` singleton tracks the current logical time:

```mew
node _TickState [sealed, singleton] {
  global_tick: Int = 0,
  last_tick_at: Timestamp?
}
```

> **Cross-reference:** See [TIME.md](../../capabilities/TIME.md) Part VIII for complete Layer 0 definition.

## Examples

### Timestamp Tracking

```
node Document {
  created_at: Timestamp = wall_time(),
  updated_at: Timestamp = wall_time()
}

rule track_updates:
  d: Document
  WHERE d.content != d._prev.content
  => SET d.updated_at = wall_time()
```

### Game Turn System

```
node GameEntity {
  spawn_tick: Int = logical_time(),
  last_action_tick: Int = logical_time()
}

node Effect {
  applied_tick: Int = logical_time(),
  duration: Int
}

rule expire_effects:
  e: Effect
  WHERE logical_time() - e.applied_tick >= e.duration
  => KILL e
```

### TTL-Based Expiration

```
node CacheEntry {
  expires_at: Timestamp,
  value: String
}

-- Query for expired entries
MATCH e: CacheEntry
WHERE wall_time() > e.expires_at
RETURN e

-- Rule to clean up expired entries
rule cleanup_expired:
  e: CacheEntry
  WHERE wall_time() > e.expires_at
  => KILL e
```

### Interior-Scoped Time (v2+)

> **Cross-reference:** See [INTERIOR.md](../../capabilities/v2/INTERIOR.md) for full interior semantics.

```
node Simulation [has_interior] {
  interior: ontology [time: independent] {
    node SimEntity {
      local_tick: Int = logical_time(SELF)
    }
  }
}

-- Query comparing scopes
MATCH e: SimEntity
WHERE logical_time(PARENT) > 100 AND logical_time(SELF) < 50
RETURN e
```

## Errors

### Determinism Errors

| Condition | Error Code | Message |
|-----------|------------|---------|
| `wall_time()` in constraint | E9034 | `NONDETERMINISTIC_FUNCTION: Function 'wall_time' is non-deterministic and cannot be used in constraints` |

### Scope Reference Errors (v2+)

| Condition | Error Code | Message |
|-----------|------------|---------|
| Invalid ScopeRef | E904x | `INVALID_SCOPE_REF: Scope reference '{ref}' does not refer to a valid interior` |
| `logical_time(scope)` on non-interior node | E904x | `NOT_AN_INTERIOR: Node '{node}' does not have an interior scope` |
