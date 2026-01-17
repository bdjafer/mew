---
spec: tick
version: "1.0"
status: draft
category: statement
capability: time
requires: [int_literal]
priority: essential
---

# Spec: TICK

## Overview

TICK advances the logical time counter and triggers the tick processing cycle. This is the fundamental mechanism for explicit time control in MEW, enabling deterministic simulations, turn-based systems, and controlled rule execution.

## Syntax

### Grammar

```ebnf
TickStmt         = "TICK"
                 | "TICK" IntLiteral ;

IntLiteral       = Digit+ ;
```

> **Note:** Scoped tick statements (`TICK IN #scope`, `TICK ALL`, `TICK CHILDREN`) are v2+ features defined in [INTERIOR.md](../../capabilities/v2/INTERIOR.md).

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

### Tick Processing Cycle

When TICK executes, the following phases occur in order:

```
1. PROCESS   - Apply pending mutations to graph
2. REACT     - Fire rules (to quiescence or limit)
3. VALIDATE  - Check constraints (violation triggers rollback)
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

### Return Value

TICK does not return a value. Use `logical_time()` after TICK to query the new time:

```
TICK
MATCH () RETURN logical_time() AS current_tick
```

## Layer 0

### Nodes

```
node _TickState [sealed, singleton] {
  global_tick: Int = 0,          -- current logical time counter
  last_tick_at: Timestamp?       -- wall-clock time of last tick
}
```

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
-- Record mutations with tick numbers
SPAWN t: Task { title = "Task 1" }   -- tick 0
TICK
SPAWN t: Task { title = "Task 2" }   -- tick 1
TICK
SPAWN t: Task { title = "Task 3" }   -- tick 2

-- Replay with same sequence produces identical state
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
TICK

-- Verify cascade
MATCH t: Task WHERE t.id = #child RETURN t.status  -- "cancelled"
```

## Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E9001 | INVALID_TICK_AMOUNT | Tick amount is zero or negative | `INVALID_TICK_AMOUNT: Tick amount must be positive, got {amount}` |

### Error Context

**E9001: INVALID_TICK_AMOUNT**

Occurs when the tick amount is not a positive integer.

```
-- Error: zero amount
TICK 0
-- Error E9001: INVALID_TICK_AMOUNT: Tick amount must be positive, got 0
```

### Related Errors

The following errors may occur during tick execution (implementation-defined):

| Suggested Code | Name | Condition |
|----------------|------|-----------|
| E9021 | RULE_DEPTH_EXCEEDED | Rule nesting exceeds implementation limit |
| E9022 | RULE_ACTIONS_EXCEEDED | Rule actions exceed implementation limit |
| E9023 | EXECUTION_TIMEOUT | Execution exceeds implementation time limit |

> **Note:** Specific limit values and error codes are implementation decisions. See [TIME.md](../../capabilities/TIME.md) Part IV for discussion of safety limits.
