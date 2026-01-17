---
spec: watch-management
version: "1.0"
status: draft
category: statement
capability: watch
requires: [watch, node_reference, watch_options]
priority: common
---

# Spec: Watch Management

## Overview

Watch management statements control the lifecycle of active watches. PAUSE temporarily stops event delivery, RESUME restarts it, CANCEL permanently removes the watch, and ALTER modifies watch options at runtime. These statements operate on watch handles obtained when creating a watch with the WATCH statement.

## Syntax

### Grammar

```ebnf
PauseStmt       = "PAUSE" "WATCH" NodeRef ;

ResumeStmt      = "RESUME" "WATCH" NodeRef ;

CancelStmt      = "CANCEL" "WATCH" NodeRef ;

AlterWatchStmt  = "ALTER" "WATCH" NodeRef "SET" AlterOptions ;

AlterOptions    = "[" AlterOption ("," AlterOption)* "]" ;

AlterOption     = "buffer" ":" IntLiteral
                | "on_full" ":" ("drop" | "block" | "error")
                | "delivery" ":" ("best_effort" | "reliable")
                | "filter" ":" Expr ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `PAUSE` | Statement - temporarily stops event delivery |
| `RESUME` | Statement - restarts event delivery after pause |
| `CANCEL` | Statement - permanently removes a watch |
| `ALTER` | Statement - modifies watch options |
| `WATCH` | Keyword - identifies the target as a watch handle |
| `SET` | Clause - introduces new option values |

### Examples

```
-- Create a watch and get handle
w = WATCH t: Task RETURN t

-- Pause event delivery
PAUSE WATCH #w

-- Resume event delivery
RESUME WATCH #w

-- Modify buffer size
ALTER WATCH #w SET [buffer: 2000]

-- Modify multiple options
ALTER WATCH #w SET [buffer: 5000, on_full: drop]

-- Permanently cancel
CANCEL WATCH #w
```

## Semantics

### Watch Handle

A watch handle is a node reference (e.g., `#watch_abc123`) returned when creating a watch. The handle references a `_Watch` node in Layer 0 that tracks the watch state and configuration.

```
-- Creating a watch returns a handle
w = WATCH t: Task WHERE t.status = "pending"
      [mode: watch]
      RETURN t.id, t.title

-- w is now a handle: #watch_abc123
```

### PAUSE Statement

PAUSE temporarily stops event delivery for a watch.

**Behavior:**
- Sets `_Watch.status` to `"paused"`
- Events continue to be detected but are not delivered
- Depending on watch configuration:
  - If `buffer` is configured: events accumulate in the buffer
  - If buffer overflows: behavior determined by `on_full` setting (drop oldest, block producer, or error)
- Watch can be resumed later
- Pausing an already-paused watch is a no-op (idempotent)

```
PAUSE WATCH #w

-- While paused:
-- - Pattern matching continues
-- - Events buffer (up to buffer limit)
-- - No events delivered to client
```

### RESUME Statement

RESUME restarts event delivery for a paused watch.

**Behavior:**
- Sets `_Watch.status` to `"active"`
- Buffered events are delivered immediately (in order)
- Normal event delivery resumes
- Resuming an already-active watch is a no-op (idempotent)
- Cannot resume a cancelled watch (error E8002)

```
RESUME WATCH #w

-- After resume:
-- 1. Buffered events delivered first
-- 2. New events delivered as they occur
```

### CANCEL Statement

CANCEL permanently removes a watch.

**Behavior:**
- Sets `_Watch.status` to `"cancelled"`
- No more events will be delivered
- Buffered events are discarded
- Watch cannot be resumed (RESUME returns E8002)
- The `_Watch` node is marked as cancelled but retained for audit
- Cancelling an already-cancelled watch is a no-op (idempotent)

```
CANCEL WATCH #w

-- After cancel:
-- - No more events
-- - Cannot be resumed
-- - Watch metadata retained for queries
```

### ALTER Statement

ALTER modifies watch options at runtime. Not all options are mutable.

**Mutable Options:**

| Option | Description |
|--------|-------------|
| `buffer` | Maximum events to buffer when watcher is slow |
| `on_full` | Buffer overflow behavior: `drop`, `block`, `error` |
| `delivery` | Delivery guarantee: `best_effort`, `reliable` |
| `filter` | Server-side filter expression |

**Immutable Options (cannot be changed after creation):**

| Option | Reason |
|--------|--------|
| `mode` | Fundamental watch semantics (watch vs consume) |
| `group` | Affects competing consumer coordination |
| `ordering` | May require reordering existing events |
| `visibility` | Affects transaction boundary behavior |
| `window` | Changes event batching semantics |

Attempting to alter an immutable option results in error E8008.

```
-- Change buffer size
ALTER WATCH #w SET [buffer: 5000]

-- Change overflow behavior
ALTER WATCH #w SET [on_full: block]

-- Change multiple options
ALTER WATCH #w SET [buffer: 2000, delivery: reliable]

-- Error: cannot change immutable option
ALTER WATCH #w SET [mode: consume]
-- Error: E8008 IMMUTABLE_WATCH_OPTION
```

Cannot alter a cancelled watch (error E8002).

### Watch Status

The `_Watch.status` field reflects the watch lifecycle:

| Status | Description |
|--------|-------------|
| `"active"` | Watch is running and delivering events |
| `"paused"` | Watch is temporarily suspended |
| `"cancelled"` | Watch is permanently removed |

```
-- Query watch status
MATCH s: _Watch WHERE s.id = #w
RETURN s.status, s.events_delivered, s.events_pending
```

## Layer 0

### Nodes

Watch management operates on the existing `_Watch` node type:

```
node _Watch [sealed] {
  pattern: String [required],        -- serialized pattern
  mode: String [required],           -- "watch" | "consume"
  group: String?,                    -- group name for competing consumers
  status: String [required],         -- "active" | "paused" | "cancelled"
  ordering: String = "arrival",      -- ordering mode
  delivery: String = "best_effort",  -- delivery guarantee
  visibility: String = "committed",  -- visibility mode
  buffer_size: Int = 1000,           -- max buffered events
  created_at: Timestamp [required],
  events_delivered: Int = 0,         -- counter
  events_pending: Int = 0,           -- current buffer size
  last_event_at: Timestamp?,
  doc: String?
}
```

### Edges

```
edge _session_has_watch(session: _Session, watch: _Watch) {}
```

### Constraints

```
constraint _watch_valid_status:
  w: _Watch => w.status = "active" OR w.status = "paused" OR w.status = "cancelled"
```

## Examples

### Complete Lifecycle Example

```
-- Create a watch for high-priority tasks
w = WATCH t: Task WHERE t.priority > 8
      [mode: watch, buffer: 1000]
      RETURN t.id, t.title, t.priority

-- Later, pause during maintenance
PAUSE WATCH #w

-- Check watch status
MATCH s: _Watch WHERE s.id = #w
RETURN s.status, s.events_pending
-- { status: "paused", events_pending: 47 }

-- Increase buffer before resuming
ALTER WATCH #w SET [buffer: 5000]

-- Resume delivery
RESUME WATCH #w

-- Eventually, cancel when no longer needed
CANCEL WATCH #w
```

### Administrative Watch Management

```
-- List all paused watches for current session
MATCH s: _Watch, _session_has_watch(current_session(), s)
WHERE s.status = "paused"
RETURN s.id, s.pattern, s.events_pending

-- Cancel all watches with excessive buffered events
MATCH s: _Watch, _session_has_watch(current_session(), s)
WHERE s.events_pending > 10000
DO CANCEL WATCH s.id
```

### Graceful Shutdown Pattern

```
-- Pause all watches before disconnecting
MATCH s: _Watch, _session_has_watch(current_session(), s)
WHERE s.status = "active"
DO PAUSE WATCH s.id

-- Process remaining buffered events...

-- Cancel all watches
MATCH s: _Watch, _session_has_watch(current_session(), s)
WHERE s.status = "paused"
DO CANCEL WATCH s.id
```

## Errors

Errors from WATCH.md Part X:

| Condition | Code | Message |
|-----------|------|---------|
| Watch handle not found | E8002 | `WATCH_NOT_FOUND: Watch handle '#xxx' does not exist` |
| Resume cancelled watch | E8002 | `WATCH_NOT_FOUND: Cannot resume cancelled watch '#xxx'` |
| Alter cancelled watch | E8002 | `WATCH_NOT_FOUND: Cannot alter cancelled watch '#xxx'` |

Additional errors specific to watch management:

| Condition | Code | Message |
|-----------|------|---------|
| Alter immutable option | E8008 | `IMMUTABLE_WATCH_OPTION: Option 'mode' cannot be changed after watch creation` |
| Invalid option value | E8009 | `INVALID_WATCH_OPTION: Invalid value for option 'buffer': expected positive integer` |
| Watch not owned by session | E8010 | `WATCH_ACCESS_DENIED: Watch '#xxx' is not owned by current session` |

Note: E8008-E8010 extend the error codes defined in WATCH.md Part X (E8001-E8007) to cover watch management operations.
