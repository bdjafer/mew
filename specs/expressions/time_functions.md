---
spec: time-functions
version: "1.0"
status: draft
category: expression
capability: time-clock
requires: [expressions]
priority: essential
---

# Spec: Time Functions

## Overview

Time functions provide access to temporal values within MEW expressions. MEW provides two distinct time concepts: wall time (real-world clock) and logical time (abstract tick counter), plus a configurable `now()` alias. These functions enable time-based attribute defaults, rule conditions, query filters, and temporal constraints.

Wall time is non-deterministic and reflects the real-world clock. Logical time is deterministic and only advances explicitly via tick operations. The `now()` function is a configurable alias that defaults to wall time but can be rebound to logical time for deterministic simulations.

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

> **Note:** Grammar matches [TIME_CLOCK.md](../../capabilities/v2/TIME_CLOCK.md) Part VIII exactly.

### Keywords

| Keyword | Context |
|---------|---------|
| `wall_time` | Expression - returns current wall clock timestamp |
| `logical_time` | Expression - returns logical tick counter |
| `now` | Expression - configurable alias (defaults to wall_time) |
| `SELF` | ScopeRef - current interior scope |
| `PARENT` | ScopeRef - parent interior scope |
| `ROOT` | ScopeRef - root/global scope |

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

### Function Signatures and Behavior

| Function | Returns | Description | Example Result |
|----------|---------|-------------|----------------|
| `wall_time()` | Timestamp | Current real-world time (ms since epoch) | `1705500000000` |
| `logical_time()` | Int | Current global logical tick counter | `42` |
| `logical_time(scope)` | Int | Logical tick counter for specified scope | `10` |
| `now()` | Timestamp \| Int | Configurable alias (see below) | depends on binding |

### wall_time()

Returns the current real-world wall clock time as a Timestamp (milliseconds since Unix epoch).

**Properties:**
- Always available
- Monotonic (never goes backward within a session)
- Not controlled by MEW
- Same call in same transaction may return different values
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
- Starts at 0 (or configured initial value)
- Only advances via explicit `TICK` or configured triggers
- Same within a tick (all operations in tick see same value)
- Deterministic (same inputs produce same logical times)

**Use cases:**
- `turn: Int = logical_time()` - game turns
- `WHERE logical_time() - t.created_tick > 100` - tick-based expiry
- Causal ordering
- Simulation steps

### logical_time(ScopeRef)

Returns the logical tick counter for a specific interior scope. This is a v2+ feature that requires the INTERIOR capability.

> **Cross-reference:** See [INTERIOR.md](../../capabilities/v2/INTERIOR.md) Part II Section 2.4 for time configuration options (`shared`, `independent`, `ratio(N)`).

**Scope References:**
- `SELF` - current interior's tick counter
- `PARENT` - parent interior's tick counter
- `ROOT` - global/root tick counter
- `NodeRef` - tick counter for the interior owned by that node

**Example:**
```
-- Query interior-specific time
MATCH e: Entity
WHERE logical_time(SELF) - e.local_tick > 10
RETURN e

-- Compare parent and local time
MATCH s: Simulation
WHERE logical_time(PARENT) > logical_time(SELF)
RETURN s
```

### now()

A configurable alias that defaults to `wall_time()` but can be rebound.

**Configuration:**
```
SET time.now_source = "wall"      -- now() = wall_time()
SET time.now_source = "logical"   -- now() = logical_time()
SET time.now_source = "hybrid"    -- now() = (wall_time(), logical_time())
```

**Default:** `"wall"` (now() returns wall_time())

**Rationale:** Most users expect `now()` to mean wall clock. Simulations and games can rebind it to logical time for determinism.

**Return type:**
- `"wall"` mode: Returns Timestamp
- `"logical"` mode: Returns Int
- `"hybrid"` mode: Returns Tuple(Timestamp, Int)

### Evaluation Context

Time functions are evaluated at different points depending on context:

| Context | When Evaluated |
|---------|----------------|
| Attribute default | At SPAWN/LINK time |
| Rule condition (WHERE) | At match time |
| Rule action (SET) | At execution time |
| Query (MATCH WHERE) | At query time |
| Constraint condition | At validation time |

### Purity

Time functions are NOT pure in the strict sense:
- `wall_time()` may return different values on each call
- `logical_time()` returns the same value within a tick but changes across ticks
- `now()` behavior depends on configuration

However, `logical_time()` is deterministic: given the same tick sequence, it produces the same values.

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

### Deterministic Mode

When `execution.deterministic = true`, additional restrictions apply:

- `wall_time()` is prohibited in rules
- `now()` must be bound to `"logical"`
- Violation produces compile-time or runtime error

```
SET execution.deterministic = true

-- This will error:
rule log_time:
  t: Task WHERE t.pending
  => SET t.checked_at = wall_time()  -- ERROR: wall_time() not allowed in deterministic mode
```

### Valid Usage Summary

| Context | wall_time() | logical_time() | now() |
|---------|-------------|----------------|-------|
| Attribute default | Yes | Yes | Yes |
| Rule condition | Yes* | Yes | Yes* |
| Rule action | Yes* | Yes | Yes* |
| Query condition | Yes | Yes | Yes |
| Constraint condition | **No** | Yes | Depends on binding** |

\* Prohibited when `execution.deterministic = true`
\** Allowed only if bound to `"logical"`

## Layer 0

> **Cross-reference:** See [TIME_CLOCK.md](../../capabilities/v2/TIME_CLOCK.md) Appendix B for complete Layer 0 extensions including `_ExecutionConfig`.

Time function behavior is configured via the `_TimingConfig` singleton node:

```mew
node _TimingConfig [sealed, singleton] {
  now_source: String [in: ["wall", "logical", "hybrid"]] = "wall",
  tick_on: String [in: ["manual", "commit", "interval"]] = "commit",
  tick_interval_ms: Int?,
  doc: String?
}
```

The `now()` function reads `now_source` to determine which underlying function to call:
- `"wall"`: `now()` delegates to `wall_time()`
- `"logical"`: `now()` delegates to `logical_time()`
- `"hybrid"`: `now()` returns a tuple of both

Current tick state is tracked in `_TickState`:

```mew
node _TickState [sealed, singleton] {
  global_tick: Int = 0,
  last_tick_at: Timestamp?
}
```

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

-- Configure for game mode
CONFIGURE PRESET game
-- Equivalent to:
-- SET time.now_source = "logical"
-- SET time.tick_interval = 16ms
```

### TTL-Based Expiration

```
node CacheEntry {
  expires_at: Timestamp,
  value: String
}

-- Query for expired entries (not in constraint due to wall_time)
MATCH e: CacheEntry
WHERE wall_time() > e.expires_at
RETURN e

-- Rule to clean up expired entries
rule cleanup_expired:
  e: CacheEntry
  WHERE wall_time() > e.expires_at
  => KILL e
```

### Deterministic Simulation

```
-- Configure for deterministic mode
SET execution.deterministic = true
SET time.now_source = "logical"

node Particle {
  created_tick: Int = now(),  -- now() = logical_time()
  position: Float
}

-- Constraint using logical time (allowed)
constraint particle_lifespan:
  p: Particle
  => logical_time() - p.created_tick <= 1000

-- Manual time advancement
TICK       -- advance by 1
TICK 10    -- advance by 10
```

### Interior-Scoped Time

> **Cross-reference:** See [INTERIOR.md](../../capabilities/v2/INTERIOR.md) for full interior semantics and time configuration options.

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

### Configurable now() Usage

```
-- Default mode (wall time)
node Event {
  timestamp: Timestamp = now()  -- wall_time()
}

-- Switch to logical mode for testing
SET time.now_source = "logical"

node TestEvent {
  tick: Int = now()  -- logical_time()
}

-- Hybrid mode for auditing
SET time.now_source = "hybrid"

node AuditEvent {
  time_info: (Timestamp, Int) = now()  -- (wall_time(), logical_time())
}
```

## Errors

### Determinism Errors (from TIME_CLOCK.md)

| Condition | Error Code | Message |
|-----------|------------|---------|
| `wall_time()` in constraint | E9034 | `NONDETERMINISTIC_FUNCTION: Function 'wall_time' is non-deterministic and cannot be used in constraints` |
| `wall_time()` in deterministic mode rule | E9031 | `WALL_TIME_IN_DETERMINISTIC: wall_time() is not allowed in deterministic mode` |
| `now()` in deterministic mode with wall binding | E9031 | `WALL_TIME_IN_DETERMINISTIC: now() resolves to wall_time() which is not allowed in deterministic mode` |

### Scope Reference Errors (v2+ with INTERIOR capability)

> **Cross-reference:** See [INTERIOR.md](../../capabilities/v2/INTERIOR.md) for scoped tick error definitions. Error codes below are provisional pending INTERIOR.md error code assignment.

| Condition | Error Code | Message |
|-----------|------------|---------|
| Invalid ScopeRef | E904x | `INVALID_SCOPE_REF: Scope reference '{ref}' does not refer to a valid interior` |
| `logical_time(scope)` on non-interior node | E904x | `NOT_AN_INTERIOR: Node '{node}' does not have an interior scope` |

### Type Errors

| Condition | Error Code | Message |
|-----------|------------|---------|
| Type mismatch with now() return | E1xxx | `TYPE_MISMATCH: Expected {expected}, got {actual} from now() in '{mode}' mode` |
