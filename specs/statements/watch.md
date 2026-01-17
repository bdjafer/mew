---
spec: watch
version: "1.0"
status: draft
category: statement
capability: watch
requires: [pattern, expression, return_clause]
priority: essential
---

# Spec: WATCH

## Overview

WATCH creates a persistent query that pushes matching changes to clients. It supports two modes: watch (non-destructive observation) and consume (destructive receipt with acknowledgment). This unified primitive handles both real-time subscriptions and message queue semantics through pattern-based matching with configurable options.

## Syntax

### Grammar

```ebnf
WatchStmt        = "WATCH" MatchPattern WatchOptions? ReturnClause ;

WatchOptions     = "[" WatchOption ("," WatchOption)* "]" ;

WatchOption      = "mode" ":" ("watch" | "consume")
                 | "group" ":" StringLiteral
                 | "ordering" ":" OrderingSpec
                 | "delivery" ":" ("best_effort" | "reliable")
                 | "visibility" ":" ("committed" | "immediate")
                 | "window" ":" WindowSpec
                 | "buffer" ":" IntLiteral
                 | "on_full" ":" ("drop" | "block" | "error")
                 | "ack_timeout" ":" Duration
                 | "max_redeliveries" ":" IntLiteral
                 | "dead_letter" ":" NodeRef
                 | "initial" ":" InitialSpec
                 | "filter" ":" Expr
                 | "branch" ":" BranchRef
                 | "branches" ":" "all"
                 | "deliver_on" ":" ("tick" | "commit") ;

OrderingSpec     = "arrival"
                 | "causal"
                 | "attribute" "(" OrderExpr ("," OrderExpr)* ")" ;

OrderExpr        = Expr ("ASC" | "DESC")? ;

WindowSpec       = "none"
                 | "tumbling" "(" Duration ")"
                 | "sliding" "(" Duration "," Duration ")" ;

InitialSpec      = "full"
                 | "none"
                 | "since" "(" IntLiteral ")" ;

(* Acknowledgment Statements *)
AckStmt          = "ACK" DeliveryId
                 | "NACK" DeliveryId NoRetryClause? ;

NoRetryClause    = "[" "no_retry" "]" ;

(* Watch Management Statements *)
PauseStmt        = "PAUSE" "WATCH" NodeRef ;
ResumeStmt       = "RESUME" "WATCH" NodeRef ;
CancelStmt       = "CANCEL" "WATCH" NodeRef ;
AlterWatchStmt   = "ALTER" "WATCH" NodeRef "SET" WatchOptions ;

(* Watch Triggers - server-side actions *)
WatchTrigger     = "ON" "MATCH" "DO" Action+ ;

(* Context Functions - valid in watch filters *)
WatchContextFunc = "current_actor" "(" ")"
                 | "current_session" "(" ")" ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `WATCH` | Statement - initiates persistent query |
| `mode` | Option - watch (observe) or consume (destructive) |
| `watch` | Mode value - non-destructive observation |
| `consume` | Mode value - destructive receipt with ACK |
| `group` | Option - consumer group name |
| `ordering` | Option - event ordering strategy |
| `arrival` | Ordering value - detection order |
| `causal` | Ordering value - happened-before order |
| `attribute` | Ordering value - order by attribute |
| `delivery` | Option - delivery guarantee level |
| `best_effort` | Delivery value - may lose events |
| `reliable` | Delivery value - buffered delivery |
| `visibility` | Option - when changes are visible |
| `committed` | Visibility value - after transaction commit |
| `immediate` | Visibility value - during transaction |
| `window` | Option - time-based batching |
| `tumbling` | Window value - non-overlapping batches |
| `sliding` | Window value - overlapping windows |
| `buffer` | Option - max buffered events |
| `on_full` | Option - buffer overflow behavior |
| `drop` | on_full value - drop oldest events |
| `block` | on_full value - backpressure |
| `error` | on_full value - cancel watch |
| `ack_timeout` | Option - acknowledgment timeout |
| `max_redeliveries` | Option - retry limit before dead letter |
| `dead_letter` | Option - failed message destination |
| `initial` | Option - initial snapshot behavior |
| `full` | Initial value - all current matches |
| `none` | Initial/Window value - no snapshot/batching |
| `since` | Initial value - changes since tick |
| `filter` | Option - server-side filter expression |
| `deliver_on` | Option - delivery timing |
| `tick` | deliver_on value - at tick boundary |
| `commit` | deliver_on value - on transaction commit |
| `branch` | Option - watch specific branch (v3+) |
| `branches` | Option - watch all branches (v3+) |
| `all` | branches value - observe all branches |
| `ASC` | Modifier - ascending order in attribute ordering |
| `DESC` | Modifier - descending order in attribute ordering |
| `ACK` | Statement - acknowledge consumption |
| `NACK` | Statement - negative acknowledgment |
| `no_retry` | Modifier - skip redelivery |
| `PAUSE` | Statement - pause watch |
| `RESUME` | Statement - resume watch |
| `CANCEL` | Statement - cancel watch |
| `ALTER` | Statement - modify watch options |
| `SET` | Clause - set options in ALTER WATCH |
| `ON` | Clause - watch trigger prefix |
| `MATCH` | Clause - watch trigger event |
| `DO` | Clause - watch trigger action |
| `current_actor` | Function - returns session's bound actor |
| `current_session` | Function - returns current session reference |

### Examples

```
-- Simple watch for completed tasks
WATCH t: Task WHERE t.status = "done"
RETURN t

-- Consume mode with consumer group
WATCH j: Job [mode: consume, group: "workers"]
RETURN j

-- Watch with attribute ordering (priority queue)
WATCH t: Task [ordering: attribute(t.priority DESC)]
RETURN t

-- Watch with windowed aggregation
WATCH t: Task [window: tumbling(10s)]
RETURN COUNT(t)

-- Full options example
WATCH m: Message, inbox(#self, m)
  [mode: consume, group: "processors", ordering: attribute(m.sent_at ASC),
   delivery: reliable, ack_timeout: 30s, max_redeliveries: 3]
RETURN m.content, m.sent_at
```

## Semantics

### Watch Lifecycle

When WATCH executes:
1. A watch handle is returned for management
2. Initial results are sent based on the `initial` option (default: full)
3. The watch remains active, pushing events as graph changes match the pattern
4. The watch continues until cancelled or disconnected

### Event Types

Watches emit events describing changes:

| Event Type | Description |
|------------|-------------|
| `INITIAL` | Sent once on watch creation; contains all current matches |
| `ADDED` | New match appeared (node/edge created, or now matches filter) |
| `REMOVED` | Match disappeared (deleted, or no longer matches filter) |
| `CHANGED` | Match still exists but projected attributes changed |
| `CONSUMED` | Match delivered for consumption (consume mode only); requires ACK |

Event payload format:
```
{ type: "added", match: {...}, tick: N }
{ type: "changed", match: {...}, prev: {...}, tick: N }
{ type: "consumed", match: {...}, delivery_id: D, tick: N }
```

### Mode Comparison

| Aspect | Watch Mode | Consume Mode |
|--------|------------|--------------|
| Graph effect | None | Deletes matched after ACK |
| Multiple watchers | All see all | Each sees different (per group) |
| Acknowledgment | Not required | Required (ACK/NACK) |
| Use case | Dashboards, monitoring | Queues, messaging |
| Delivery | Best-effort or reliable | Exactly-once |
| Events | initial, added, removed, changed | initial (empty), consumed |

### Option Defaults

| Option | Default | Alternatives |
|--------|---------|--------------|
| mode | watch | consume |
| group | none | "name" |
| ordering | arrival | causal, attribute(...) |
| delivery | best_effort | reliable |
| visibility | committed | immediate |
| window | none | tumbling(...), sliding(...) |
| buffer | 1000 | N |
| on_full | drop | block, error |
| ack_timeout | 30s | Duration |
| max_redeliveries | 3 | N |
| initial | full | none, since(...) |
| deliver_on | commit | tick |

### Delivery Guarantees

| Guarantee | Description | Configuration |
|-----------|-------------|---------------|
| At-most-once | May lose events, never duplicate | `[delivery: best_effort]` in watch mode |
| At-least-once | Never lose, may duplicate | `[delivery: reliable]` in watch mode |
| Exactly-once | Never lose, never duplicate | `[mode: consume]` with ACK |

### Ordering Guarantees

| Ordering | Description |
|----------|-------------|
| `arrival` | Order events were detected; approximate FIFO; no strict guarantees |
| `causal` | If A caused B, deliver A before B; may delay events |
| `attribute(expr)` | Order by attribute value; useful for priority queues |

### Consume Protocol

In consume mode:
1. Match found and marked "pending" (invisible to other consumers)
2. Event delivered with `delivery_id`
3. Watcher processes the message
4. Acknowledgment sent:
   - `ACK #delivery_id` - match deleted from graph
   - `NACK #delivery_id` - match returned to available pool for redelivery
   - `NACK #delivery_id [no_retry]` - match moved to dead letter or deleted
5. If no ACK/NACK within `ack_timeout`, treated as NACK

### Group Semantics

Without group:
- Watch mode: each watcher independent, all see all
- Consume mode: single consumer

With group:
- Watch mode: group ignored, all see all
- Consume mode: competing consumers; each match to exactly one group member; load balanced; failover on disconnect

## Layer 0

### Nodes

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
  on_full: String = "drop",          -- "drop" | "block" | "error"
  created_at: Timestamp [required],
  events_delivered: Int = 0,         -- counter
  events_pending: Int = 0,           -- current buffer size
  last_event_at: Timestamp?,
  doc: String?
}

node _WatchEvent [sealed] {
  event_type: String [required],     -- "initial" | "added" | "removed" | "changed" | "consumed"
  tick: Int [required],
  delivery_id: String?,              -- for consume mode
  created_at: Timestamp [required]
}

node _DeadLetter [sealed] {
  original_match: String [required], -- serialized match data
  failure_reason: String [required],
  delivery_attempts: Int [required],
  created_at: Timestamp [required]
}
```

### Edges

```
edge _session_has_watch(session: _Session, watch: _Watch) {}

edge _watch_event(watch: _Watch, event: _WatchEvent) {
  position: Int [required]           -- ordering within watch
}

edge _watch_dead_letter(watch: _Watch, letter: _DeadLetter) {}

edge _ontology_declares_watch(ontology: _Ontology, watch: _Watch) {}
```

### Constraints

```
constraint _watch_has_pattern:
  w: _Watch => w.pattern != null AND length(w.pattern) > 0

constraint _watch_valid_mode:
  w: _Watch => w.mode = "watch" OR w.mode = "consume"

constraint _watch_valid_status:
  w: _Watch => w.status = "active" OR w.status = "paused" OR w.status = "cancelled"

constraint _watch_valid_on_full:
  w: _Watch => w.on_full = "drop" OR w.on_full = "block" OR w.on_full = "error"
```

## Examples

### Dashboard Real-Time Updates

```
-- Watch all tasks in a project for dashboard
WATCH t: Task, belongs_to(t, #project_1)
  [mode: watch, delivery: reliable]
RETURN t.id, t.title, t.status, t.priority
```

### Work Queue with Competing Consumers

```
-- Worker 1
WATCH j: Job WHERE j.status = "pending"
  [mode: consume, group: "workers", ordering: attribute(j.priority DESC, j.created_at ASC)]
RETURN j

-- Worker 2 (same group - jobs distributed between workers)
WATCH j: Job WHERE j.status = "pending"
  [mode: consume, group: "workers", ordering: attribute(j.priority DESC, j.created_at ASC)]
RETURN j

-- Process and acknowledge
ACK #delivery_abc123
```

### Point-to-Point Messaging (Inbox Pattern)

```
-- Schema
node Message { content: String, sent_at: Timestamp }
edge inbox(recipient: Actor, message: Message)

-- Send
SPAWN m: Message { content = "hello", sent_at = now() }
LINK inbox(#bob, m)

-- Receive (Bob's watch)
WATCH m: Message, inbox(#self, m)
  [mode: consume, ordering: attribute(m.sent_at ASC)]
RETURN m.content, m.sent_at
```

### Windowed Aggregation

```
-- Aggregate task stats every 10 seconds
WATCH t: Task, belongs_to(t, #project_1)
  [window: tumbling(10s)]
RETURN COUNT(t) AS total,
       COUNT(t WHERE t.status = "done") AS done,
       AVG(t.priority) AS avg_priority
```

### Watch Management

```
-- Create watch and store handle
w = WATCH t: Task WHERE t.status = "done"
      [mode: watch]
    RETURN t

-- Pause watch
PAUSE WATCH #w

-- Resume watch
RESUME WATCH #w

-- Modify options
ALTER WATCH #w SET [buffer: 2000]

-- Cancel watch
CANCEL WATCH #w
```

## Errors

| Code | Condition | Message |
|------|-----------|---------|
| E8001 | Policy denies watch pattern | `WATCH_PATTERN_DENIED: Watch pattern rejected by policy` |
| E8002 | Invalid watch handle | `WATCH_NOT_FOUND: Handle references non-existent or cancelled watch` |
| E8003 | Acknowledgment timeout | `ACK_TIMEOUT: Consume acknowledgment timed out; message will be redelivered` |
| E8004 | Invalid delivery ID | `INVALID_DELIVERY_ID: ACK/NACK for unknown or already-processed delivery` |
| E8005 | Buffer overflow with error mode | `WATCH_BUFFER_OVERFLOW: Buffer full and on_full: error configured; watch cancelled` |
| E8006 | Too many watches | `WATCH_LIMIT_EXCEEDED: Too many active watches per session or globally` |
| E8007 | Dead letter failure | `DEAD_LETTER_FAILED: Message could not be moved to dead letter destination` |
