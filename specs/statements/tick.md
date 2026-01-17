---
spec: tick
version: "1.0"
status: draft
category: statement
capability: time
capability_source: capabilities/v2/TIME_CLOCK.md
requires: [int_literal]
priority: essential
---

# Spec: TICK

## Overview

TICK advances the logical time counter and triggers the tick processing cycle. This is the fundamental mechanism for explicit time control in MEW, enabling deterministic simulations, turn-based systems, and controlled rule execution. The tick processing cycle applies pending mutations, fires rules to quiescence (or limit), and checks constraints before advancing the logical time counter.

## Syntax

### Grammar

```ebnf
TickStmt         = "TICK"
                 | "TICK" IntLiteral ;

IntLiteral       = Digit+ ;

Digit            = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
```

> **Note:** Scoped tick statements (`TICK IN #scope`, `TICK ALL`, `TICK CHILDREN`) are v2+ features defined in [INTERIOR.md](../../capabilities/v2/INTERIOR.md) for world interior timing control.

### Keywords

| Keyword | Context |
|---------|---------|
| `TICK` | Statement - advances logical time |

### Examples

```
-- Advance logical time by 1
TICK

-- Advance logical time by 10
TICK 10

-- Game loop simulation
TICK         -- Turn 1
TICK         -- Turn 2
TICK 100     -- Skip to turn 102
```

## Semantics

### Tick Execution Cycle

When TICK executes, the following phases occur in order:

```
1. PROCESS   - Apply pending mutations to graph
2. REACT     - Fire rules (to quiescence or configured limit)
3. VALIDATE  - Check constraints (hard fail triggers rollback)
4. ADVANCE   - Increment logical time counter
```

All operations within a tick see the same `logical_time()` value. The counter advances only after successful completion of phases 1-3.

### Behavior by Statement Form

| Form | Behavior |
|------|----------|
| `TICK` | Advance logical time by 1 |
| `TICK N` | Advance logical time by N (executes N tick cycles) |

### Tick Amount Semantics

When `TICK N` is specified with N > 1:
- N complete tick cycles execute sequentially
- Each cycle processes mutations, fires rules, and checks constraints
- If any cycle fails (constraint violation, limit exceeded), the entire TICK statement fails
- Logical time advances by N only on complete success

### Interaction with Configuration

The tick behavior is influenced by execution configuration:

| Configuration | Effect on TICK |
|---------------|----------------|
| `execution.rule_mode = "eager"` | Rules fire immediately during each tick cycle |
| `execution.rule_mode = "deferred"` | Rules fire during the TICK processing phase |
| `execution.quiescence = "strict"` | TICK fails if quiescence not reached within limits |
| `execution.quiescence = "bounded"` | TICK commits partial state, defers remaining rules |
| `execution.on_limit = "rollback"` | TICK fails and rolls back if limits exceeded |
| `execution.on_limit = "commit"` | TICK commits partial state on limit |

### Manual vs Automatic Ticking

TICK statements are used when `time.tick_on = "manual"`. In other modes:

| Mode | TICK Statement Behavior |
|------|-------------------------|
| `manual` | Required for time advancement |
| `commit` | TICK still works; adds to automatic per-commit ticks |
| `interval` | TICK still works; adds to automatic periodic ticks |

### Return Value

TICK does not return a value. Use `logical_time()` after TICK to query the new time:

```
TICK
MATCH () RETURN logical_time() AS current_tick
```

## Layer 0

The tick statement interacts with the `_TickState` singleton node defined in [TIME_CLOCK.md](../../capabilities/v2/TIME_CLOCK.md).

### Nodes

```
node _TickState [sealed, singleton] {
  global_tick: Int = 0,          -- current logical time counter
  last_tick_at: Timestamp?       -- wall-clock time of last tick (for diagnostics)
}
```

> **Note:** As a `singleton` node, exactly one `_TickState` exists per ontology. No explicit edge is required to associate it with the ontology.

### Edges

None. The `_TickState` singleton is implicitly associated with the ontology.

### Constraints

```
constraint _tick_non_negative:
  s: _TickState => s.global_tick >= 0
```

## Examples

### Basic Time Advancement

```
-- Check initial time
MATCH () RETURN logical_time()   -- Returns 0

-- Advance by 1
TICK
MATCH () RETURN logical_time()   -- Returns 1

-- Advance by 5
TICK 5
MATCH () RETURN logical_time()   -- Returns 6
```

### Turn-Based Game

```
-- Game setup
SPAWN player: Player { name = "Alice", turn_created = logical_time() }

-- Turn 1: Player moves
SPAWN move: Move { player = #player, position = "A1", turn = logical_time() }
TICK

-- Turn 2: Another move
SPAWN move: Move { player = #player, position = "B2", turn = logical_time() }
TICK

-- Query moves by turn
MATCH m: Move WHERE m.turn = 1 RETURN m
```

### Simulation with Rule Processing

```
-- Physics rule (fires each tick)
rule apply_gravity:
  p: Particle WHERE p.grounded = false
  => SET p.velocity_y = p.velocity_y - 9.8

-- Run 100 simulation steps
TICK 100
```

### Deterministic Replay

```
-- Configure for determinism
SET execution.deterministic = true

-- Record mutations with tick numbers
SPAWN t: Task { title = "Task 1" }   -- tick 0
TICK
SPAWN t: Task { title = "Task 2" }   -- tick 1
TICK
SPAWN t: Task { title = "Task 3" }   -- tick 2

-- Replay produces identical state
```

### Testing Rule Execution

```
-- Rule that propagates status
rule cascade_status:
  parent: Task, child: Task, subtask(parent, child)
  WHERE parent.status = "cancelled"
  => SET child.status = "cancelled"

-- Setup
SPAWN parent: Task { status = "active" }
SPAWN child: Task { status = "active" }
LINK subtask(#parent, #child)

-- Trigger rule
SET #parent.status = "cancelled"

-- With deferred rules, need TICK to fire
TICK

-- Verify cascade
MATCH t: Task WHERE t.id = #child RETURN t.status  -- "cancelled"
```

## Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9001 | INVALID_TICK_AMOUNT | Tick amount is zero | `INVALID_TICK_AMOUNT: Tick amount must be positive, got {amount}` |

### Error Contexts

**E9001: INVALID_TICK_AMOUNT**

Occurs when the tick amount is not a positive integer. The grammar accepts `IntLiteral` (digits only), so this error applies to zero values. Expressions like `TICK -5` would use a unary minus which is semantically invalid for tick amounts.

```
-- Error: zero amount
TICK 0
-- Error E9001: INVALID_TICK_AMOUNT: Tick amount must be positive, got 0
```

### Related Errors (from TIME_CLOCK capability)

The following errors may occur during tick execution but are defined in the TIME_CLOCK capability:

| Code | Name | Condition |
|------|------|-----------|
| E9021 | RULE_DEPTH_EXCEEDED | Rule nesting exceeds `max_rule_depth` |
| E9022 | RULE_ACTIONS_EXCEEDED | Rule actions exceed `max_rule_actions` |
| E9023 | EXECUTION_TIMEOUT | Execution exceeds `max_execution_time` |
| E9026 | QUIESCENCE_NOT_REACHED | Strict quiescence mode, limits hit before quiescence |

> **Note:** Scoped tick errors (invalid scope reference, circular time relationships) are v2+ features defined in [INTERIOR.md](../../capabilities/v2/INTERIOR.md).
