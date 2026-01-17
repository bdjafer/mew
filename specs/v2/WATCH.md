# MEW Watch System

**Version:** 1.0
**Status:** Specification
**Scope:** Reactive queries, push notifications, message consumption

---

# Part I: Context & Motivation

## 1.1 The Problem

Point-in-time queries are insufficient for many applications:

```
-- Point-in-time: snapshot at query moment
MATCH t: Task WHERE t.status = "done" RETURN t
-- Returns: [task_1, task_2, task_3]
-- Client must poll to detect changes
```

Applications need to react to changes:
- Dashboards updating in real-time
- Notifications when conditions are met
- Work queues distributing jobs
- Event-driven architectures
- Collaborative editing

Without native support, clients must poll, which is inefficient and introduces latency.

## 1.2 The Insight

Both "watching changes" and "receiving messages" are push mechanisms:

| Traditional View | Unified View |
|-----------------|--------------|
| Subscription = Watch pattern | Watch = Watch pattern |
| Channel = Message queue | Channel = Watch "messages to me" pattern |
| Different primitives | Same primitive, different modes |

The difference is **what happens after delivery**:
- **Watch:** See the change, graph unchanged
- **Consume:** Receive the message, message removed from graph

## 1.3 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Single primitive** | Watch handles both observing and messaging |
| **Pattern-based** | Full query power for defining scope |
| **Modal** | Watch vs consume determines semantics |
| **Composable** | Options combine orthogonally |
| **Graph-native** | Messages are nodes/edges, not separate system |

---

# Part II: Core Model

## 2.1 Watch Definition

A watch is a persistent query that pushes results to a client:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        WATCH ANATOMY                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WATCH                                                              │
│    pattern       -- what to match                                   │
│    [options]     -- how to behave                                   │
│    RETURN        -- what to return                                  │
│                                                                      │
│  Components:                                                        │
│                                                                      │
│    Pattern:   Standard MATCH pattern                                │
│    Options:   Mode, ordering, delivery, grouping, windowing        │
│    Return:    Projection of matched data                           │
│                                                                      │
│  Result:                                                            │
│                                                                      │
│    Watch handle (for management)                                    │
│    Stream of events (changes matching pattern)                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Watch Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WATCH LIFECYCLE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CREATE                                                             │
│  ──────                                                             │
│     WATCH pattern [options] RETURN projection                      │
│     → Returns: watch_handle                                        │
│     → Sends: initial results (current matches)                     │
│                                                                      │
│  ACTIVE                                                             │
│  ──────                                                             │
│     Graph changes → Check pattern → Push if matches                │
│     Continuous until cancelled or disconnected                     │
│                                                                      │
│  PAUSE / RESUME                                                     │
│  ─────────────                                                      │
│     PAUSE WATCH #handle                                            │
│     RESUME WATCH #handle                                           │
│     Paused: changes accumulate or are dropped (configurable)       │
│                                                                      │
│  CANCEL                                                             │
│  ──────                                                             │
│     CANCEL WATCH #handle                                           │
│     Watch removed, no more events                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.3 Event Types

Watches emit events describing changes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                       EVENT TYPES                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  INITIAL                                                            │
│  ───────                                                            │
│  Sent once on watch creation.                                      │
│  Contains all current matches.                                     │
│                                                                      │
│    { type: "initial", matches: [...] }                             │
│                                                                      │
│  ADDED                                                              │
│  ─────                                                              │
│  New match appeared (node/edge created, or now matches filter).    │
│                                                                      │
│    { type: "added", match: {...}, tick: N }                        │
│                                                                      │
│  REMOVED                                                            │
│  ───────                                                            │
│  Match disappeared (deleted, or no longer matches filter).         │
│                                                                      │
│    { type: "removed", match: {...}, tick: N }                      │
│                                                                      │
│  CHANGED                                                            │
│  ───────                                                            │
│  Match still exists but projected attributes changed.              │
│                                                                      │
│    { type: "changed", match: {...}, prev: {...}, tick: N }         │
│                                                                      │
│  CONSUMED (consume mode only)                                       │
│  ────────                                                           │
│  Match delivered for consumption.                                  │
│  Requires ACK.                                                     │
│                                                                      │
│    { type: "consumed", match: {...}, delivery_id: D, tick: N }     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part III: Watch Modes

## 3.1 Watch Mode (Default)

Non-destructive observation of graph changes.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        WATCH MODE                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WATCH t: Task WHERE t.status = "done"                             │
│    [mode: watch]                                                   │
│    RETURN t.id, t.title, t.completed_at                           │
│                                                                      │
│  Semantics:                                                         │
│  ──────────                                                         │
│  • Multiple watchers all see all matches                           │
│  • Matches persist in graph (not affected by watch)               │
│  • Events: initial, added, removed, changed                       │
│  • No acknowledgment required                                      │
│                                                                      │
│  Use cases:                                                         │
│  ──────────                                                         │
│  • Dashboards                                                      │
│  • Monitoring                                                       │
│  • Derived views                                                    │
│  • Notifications                                                    │
│  • Audit logging                                                    │
│                                                                      │
│  Diagram:                                                           │
│                                                                      │
│    Graph ───changes───┬──▶ Watcher A (sees all)                   │
│                       ├──▶ Watcher B (sees all)                   │
│                       └──▶ Watcher C (sees all)                   │
│                                                                      │
│    Graph unchanged by watches                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Consume Mode

Destructive consumption — each match delivered once, then removed.

```
┌─────────────────────────────────────────────────────────────────────┐
│                       CONSUME MODE                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WATCH j: Job WHERE j.status = "pending"                           │
│    [mode: consume]                                                 │
│    RETURN j                                                        │
│                                                                      │
│  Semantics:                                                         │
│  ──────────                                                         │
│  • Each match delivered to exactly one watcher (in group)          │
│  • Match removed from graph after acknowledgment                   │
│  • Events: initial (empty), consumed                              │
│  • Acknowledgment required (ACK/NACK)                              │
│  • Unacknowledged = redelivery after timeout                       │
│                                                                      │
│  Use cases:                                                         │
│  ──────────                                                         │
│  • Work queues                                                      │
│  • Message processing                                               │
│  • Task distribution                                                │
│  • Event handling                                                   │
│                                                                      │
│  Diagram:                                                           │
│                                                                      │
│    Graph ───match───▶ Watcher A ───ACK───▶ Match deleted           │
│                                                                      │
│    With competing consumers (group):                               │
│                                                                      │
│    Graph ───match1───▶ Watcher A (in group "workers")             │
│          ───match2───▶ Watcher B (in group "workers")             │
│          ───match3───▶ Watcher A                                   │
│                                                                      │
│    Each match goes to exactly one group member                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.3 Consume Delivery Protocol

```
┌─────────────────────────────────────────────────────────────────────┐
│                   CONSUME DELIVERY PROTOCOL                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. MATCH FOUND                                                     │
│     Match satisfies watch pattern                                  │
│     Match marked "pending" (invisible to other consumers)          │
│                                                                      │
│  2. DELIVER                                                         │
│     Event sent to watcher:                                         │
│     { type: "consumed", match: {...}, delivery_id: "abc123" }     │
│                                                                      │
│  3. WATCHER PROCESSES                                               │
│     Application handles the message                                │
│                                                                      │
│  4. ACKNOWLEDGE                                                     │
│                                                                      │
│     ACK #delivery_id                                               │
│     → Match deleted from graph                                     │
│     → Processing complete                                          │
│                                                                      │
│     NACK #delivery_id                                              │
│     → Match returned to available pool                             │
│     → Will be redelivered (to same or different consumer)         │
│                                                                      │
│     NACK #delivery_id [no_retry]                                   │
│     → Match moved to dead letter (if configured)                   │
│     → Or deleted                                                   │
│                                                                      │
│  5. TIMEOUT                                                         │
│     If no ACK/NACK within timeout:                                 │
│     → Treated as NACK                                              │
│     → Redelivered                                                  │
│                                                                      │
│  Configuration:                                                     │
│                                                                      │
│     [ack_timeout: 30s]          -- time to acknowledge            │
│     [max_redeliveries: 3]       -- before dead letter             │
│     [dead_letter: #dlq_node]    -- where failed messages go       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part IV: Watch Options

## 4.1 Complete Options Reference

```
WATCH pattern
  [mode: watch | consume]
  [group: "group_name"]
  [ordering: arrival | causal | attribute(expr)]
  [delivery: best_effort | reliable]
  [visibility: committed | immediate]
  [window: none | tumbling(duration) | sliding(duration, interval)]
  [buffer: N]
  [ack_timeout: duration]
  [max_redeliveries: N]
  [dead_letter: node_ref]
  [initial: full | none | since(tick)]
  RETURN projection
```

## 4.2 Mode

```
[mode: watch | consume]

  watch (default):
    Non-destructive observation
    All watchers see all matches
    No acknowledgment

  consume:
    Destructive consumption
    Each match delivered once (per group)
    Requires acknowledgment
    Removed after ACK
```

## 4.3 Group

```
[group: "group_name"]

  No group (default):
    Each watcher independent
    In watch mode: all see all
    In consume mode: single consumer

  With group:
    Watchers in same group coordinate
    In watch mode: all see all (group ignored)
    In consume mode: competing consumers
      - Each match to exactly one group member
      - Load balanced
      - Failover on disconnect
```

Example: Work queue with competing consumers

```
-- Worker 1
WATCH j: Job WHERE j.status = "pending"
  [mode: consume, group: "workers"]
  RETURN j

-- Worker 2 (same group)
WATCH j: Job WHERE j.status = "pending"
  [mode: consume, group: "workers"]
  RETURN j

-- Jobs distributed between workers
-- Each job processed by exactly one worker
```

## 4.4 Ordering

```
[ordering: arrival | causal | attribute(expr)]

  arrival (default):
    Order events were detected
    Approximate FIFO
    No strict guarantees

  causal:
    If A caused B, deliver A before B
    Respects happened-before
    May delay events to maintain order

  attribute(expr):
    Order by attribute value
    e.g., [ordering: attribute(t.priority DESC)]
    Useful for priority queues
```

## 4.5 Delivery

```
[delivery: best_effort | reliable]

  best_effort (default):
    Events may be lost on disconnect
    No buffering during outage
    Lower resource usage

  reliable:
    Events buffered during disconnect
    Redelivered on reconnect
    Exactly-once within session (watch mode)
    Exactly-once ever (consume mode with ACK)
    Higher resource usage
```

## 4.6 Visibility

```
[visibility: committed | immediate]

  committed (default):
    See changes only after transaction commits
    Consistent view
    No phantom reads

  immediate:
    See changes during transaction
    May see uncommitted data
    May see data that gets rolled back
    Use for: real-time collaboration drafts
```

## 4.7 Windowing

```
[window: none | tumbling(duration) | sliding(duration, interval)]

  none (default):
    Each event delivered individually

  tumbling(duration):
    Collect events for duration
    Deliver as batch at window end
    Non-overlapping windows

    [window: tumbling(1s)]
    → Events batched per second

  sliding(duration, interval):
    Window of duration, evaluated every interval
    Overlapping windows

    [window: sliding(10s, 1s)]
    → 10-second windows, updated every second
```

## 4.8 Buffer

```
[buffer: N]

  Maximum events to buffer when watcher is slow.
  Behavior when full:

    [buffer: 1000]                    -- default behavior: drop oldest
    [buffer: 1000, on_full: drop]     -- drop oldest
    [buffer: 1000, on_full: block]    -- backpressure (slow producer)
    [buffer: 1000, on_full: error]    -- cancel watch
```

## 4.9 Initial Results

```
[initial: full | none | since(tick)]

  full (default):
    Send all current matches on watch creation
    Then incremental updates

  none:
    No initial snapshot
    Only new changes from now

  since(tick):
    Send changes since specified tick
    Requires event retention (history)
```

---

# Part V: Patterns as Channels

## 5.1 The Channel Pattern

Channels are watches on "messages addressed to me":

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CHANNEL AS PATTERN                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Traditional channel:                                               │
│                                                                      │
│    SEND "hello" TO #alice_inbox                                    │
│    RECEIVE FROM #alice_inbox → msg                                 │
│                                                                      │
│  As MEW pattern:                                                    │
│                                                                      │
│    Schema:                                                          │
│      node Inbox { owner: Actor }                                   │
│      node Message { content: String, sent_at: Timestamp }          │
│      edge pending(inbox: Inbox, message: Message)                  │
│                                                                      │
│    Send:                                                            │
│      SPAWN m: Message { content = "hello", sent_at = now() }       │
│      LINK pending(#alice_inbox, m)                                 │
│                                                                      │
│    Receive:                                                         │
│      WATCH m: Message, pending(#my_inbox, m)                       │
│        [mode: consume]                                             │
│        [ordering: attribute(m.sent_at)]                            │
│        RETURN m                                                    │
│                                                                      │
│  Benefits:                                                          │
│  • Messages are queryable graph structure                          │
│  • Full pattern power (filter, join)                               │
│  • Existing policy applies                                         │
│  • No separate messaging system                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Standard Channel Patterns

### Point-to-Point (Inbox)

```
-- Schema
node Actor { name: String }
node Message { content: String, sent_at: Timestamp }
edge inbox(recipient: Actor, message: Message)

-- Send
SPAWN m: Message { content = "hello", sent_at = wall_time() }
LINK inbox(#bob, m)

-- Receive (Bob's watch)
WATCH m: Message, inbox(#self, m)
  [mode: consume, ordering: attribute(m.sent_at)]
  RETURN m.content, m.sent_at
```

### Topic (Pubsub)

```
-- Schema
node Topic { name: String }
node Publication { content: String, published_at: Timestamp }
edge published(topic: Topic, pub: Publication)
edge subscribed(actor: Actor, topic: Topic)

-- Publish
SPAWN p: Publication { content = "news!", published_at = wall_time() }
LINK published(#news_topic, p)

-- Watch topic (watch mode - all see all)
WATCH p: Publication, published(#news_topic, p)
  [mode: watch]
  RETURN p

-- Fan-out: each watcher independently sees all publications
```

### Work Queue (Competing Consumers)

```
-- Schema
node Job { payload: String, priority: Int, created_at: Timestamp }
edge pending_job(job: Job)

-- Enqueue
SPAWN j: Job { payload = "...", priority = 5, created_at = wall_time() }
LINK pending_job(j)

-- Workers (competing)
WATCH j: Job, pending_job(j)
  [mode: consume]
  [group: "workers"]
  [ordering: attribute(j.priority DESC, j.created_at ASC)]
  RETURN j

-- Each job goes to exactly one worker
-- Higher priority first, FIFO within priority
```

### Request-Reply

```
-- Schema
node Request { payload: String, correlation_id: String }
node Response { payload: String, correlation_id: String }
edge request_to(req: Request, service: Actor)
edge response_to(resp: Response, requester: Actor)

-- Send request
SPAWN req: Request { 
  payload = "get user 123", 
  correlation_id = "corr_abc" 
}
LINK request_to(req, #user_service)

-- Service watch
WATCH r: Request, request_to(r, #self)
  [mode: consume]
  RETURN r

-- Client waits for specific response
WATCH r: Response, response_to(r, #self)
  WHERE r.correlation_id = "corr_abc"
  [mode: consume]
  RETURN r
```

---

# Part VI: Watch Management

## 6.1 Watch Handle

Creating a watch returns a handle:

```
w = WATCH t: Task WHERE t.status = "done"
      [mode: watch]
      RETURN t

-- w is a handle: #watch_abc123

-- Handle is a node in the graph (queryable!)
MATCH s: _Watch WHERE s.id = #w
RETURN s.pattern, s.mode, s.created_at
```

## 6.2 Management Operations

```
-- Pause (stop receiving, optionally buffer)
PAUSE WATCH #w

-- Resume
RESUME WATCH #w

-- Cancel (permanent)
CANCEL WATCH #w

-- Modify (some options changeable)
ALTER WATCH #w SET [buffer: 2000]

-- List active watches
MATCH s: _Watch WHERE s.session = current_session()
RETURN s
```

## 6.3 Watch Metadata

Watches are nodes in Layer 0:

```
node _Watch {
  pattern: String,          -- serialized pattern
  mode: String,             -- "watch" | "consume"
  group: String?,
  status: String,           -- "active" | "paused" | "cancelled"
  created_at: Timestamp,
  session: Session,
  events_delivered: Int,    -- counter
  events_pending: Int,      -- buffer size
  last_event_at: Timestamp?
}
```

This allows:
- Querying watch status
- Monitoring watch health
- Administrative management
- Self-describing system

---

# Part VII: Server-Side Processing

## 7.1 Aggregating Watches

Watches can include aggregation:

```
WATCH t: Task, belongs_to(t, #project_1)
  [window: tumbling(10s)]
  RETURN COUNT(t) AS total,
         COUNT(t WHERE t.status = "done") AS done,
         AVG(t.priority) AS avg_priority

-- Every 10 seconds, receive aggregated stats
-- Not individual task events
```

## 7.2 Watch Triggers (Actions)

Watches can trigger server-side actions:

```
WATCH t: Task WHERE t.priority > 8
  [mode: watch]
  ON MATCH DO
    SPAWN n: Notification { 
      message = "High priority task: " ++ t.title 
    }
    LINK notify(#ops_team, n)

-- When high-priority task appears:
-- 1. Notification created automatically
-- 2. No client needed to be connected
```

This bridges watches and rules:
- Rules: Always active, fire on any matching change
- Watch triggers: Active while watch exists, client-controlled

## 7.3 Watch Filters (Server-Side)

Reduce data sent to client:

```
WATCH t: Task
  [filter: t.assignee = current_actor()]
  RETURN t

-- Only tasks assigned to me
-- Filter evaluated server-side
-- Reduces bandwidth
```

---

# Part VIII: Delivery Semantics

## 8.1 Delivery Guarantees

```
┌─────────────────────────────────────────────────────────────────────┐
│                   DELIVERY GUARANTEES                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  AT-MOST-ONCE                                                       │
│  ─────────────                                                      │
│  • May lose events                                                 │
│  • Never duplicate                                                 │
│  • Lowest latency                                                  │
│                                                                      │
│  [delivery: best_effort]  -- in watch mode                         │
│                                                                      │
│                                                                      │
│  AT-LEAST-ONCE                                                      │
│  ──────────────                                                     │
│  • Never lose events                                               │
│  • May duplicate (on retry)                                        │
│  • Client must be idempotent                                       │
│                                                                      │
│  [delivery: reliable]  -- in watch mode without dedup              │
│                                                                      │
│                                                                      │
│  EXACTLY-ONCE                                                       │
│  ────────────                                                       │
│  • Never lose, never duplicate                                     │
│  • Requires acknowledgment + deduplication                         │
│  • Highest overhead                                                │
│                                                                      │
│  [mode: consume]  -- consume mode with ACK is exactly-once         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.2 Ordering Guarantees

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ORDERING GUARANTEES                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ARRIVAL ORDER                                                      │
│  ─────────────                                                      │
│  Events delivered in order detected.                               │
│  May not match causal order.                                       │
│  Lowest latency.                                                   │
│                                                                      │
│  [ordering: arrival]                                               │
│                                                                      │
│                                                                      │
│  CAUSAL ORDER                                                       │
│  ────────────                                                       │
│  If A happened-before B, deliver A before B.                       │
│  May delay events to maintain order.                               │
│  Higher latency, stronger guarantees.                              │
│                                                                      │
│  [ordering: causal]                                                │
│                                                                      │
│                                                                      │
│  ATTRIBUTE ORDER                                                    │
│  ───────────────                                                    │
│  Order by specified attribute(s).                                  │
│  Useful for priority queues.                                       │
│  May reorder events.                                               │
│                                                                      │
│  [ordering: attribute(t.priority DESC)]                            │
│                                                                      │
│                                                                      │
│  TOTAL ORDER (consume mode with single consumer)                   │
│  ───────────                                                        │
│  All events in single global order.                                │
│  Only one consumer receives at a time.                             │
│  Strongest guarantee, lowest throughput.                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part IX: Interaction with Other Systems

## 9.1 Policy

Watches are subject to policy:

```
-- Watch pattern must pass policy check
WATCH t: Task WHERE t.secret = true
  RETURN t

-- Policy checks:
-- 1. Can current_actor() create watches? (META READ-like)
-- 2. Can current_actor() MATCH Task? (data access)
-- 3. Can current_actor() see secret tasks? (field-level)

-- If any check fails, watch rejected

-- For consume mode:
-- Also checks: can current_actor() delete matched items?
```

## 9.2 Branching

In branching execution:

```
-- Default: watch sees current branch only
WATCH t: Task
  RETURN t

-- Explicit branch:
WATCH t: Task
  [branch: #branch_123]
  RETURN t

-- All branches:
WATCH t: Task
  [branches: all]
  RETURN t, branch.id, branch.weight
```

## 9.3 Timing

When are watch events delivered?

```
With tick-based execution:
──────────────────────────

  SET time.tick_interval = 16ms

  Option A: Deliver on tick
    Events batched, delivered at tick boundary
    Lower overhead, higher latency

  Option B: Deliver on commit
    Events delivered as transactions commit
    Lower latency, higher overhead

  Configuration:
    [deliver_on: tick | commit]
    Default: commit


With streaming execution (best_effort quiescence):
──────────────────────────────────────────────────

  Events delivered continuously as detected
  No batching
```

## 9.4 Transactions

```
Watcher sees consistent snapshots:

  BEGIN
    SPAWN t: Task { status = "todo" }    -- not visible to watchers
    SET t.status = "done"                -- not visible to watchers
  COMMIT                                 -- watcher sees: added (status=done)

Watcher never sees intermediate state (status=todo).
Unless [visibility: immediate] specified.
```

---

# Part X: Grammar

```
WatchStmt =
    "WATCH" MatchPattern WatchOptions? ReturnClause

WatchOptions = "[" WatchOption ("," WatchOption)* "]"

WatchOption =
    "mode" ":" ("watch" | "consume")
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
  | "deliver_on" ":" ("tick" | "commit")

OrderingSpec =
    "arrival"
  | "causal"
  | "attribute" "(" OrderExpr ("," OrderExpr)* ")"

OrderExpr = Expr ("ASC" | "DESC")?

WindowSpec =
    "none"
  | "tumbling" "(" Duration ")"
  | "sliding" "(" Duration "," Duration ")"

InitialSpec =
    "full"
  | "none"
  | "since" "(" IntLiteral ")"

AckStmt =
    "ACK" DeliveryId
  | "NACK" DeliveryId NoRetryClause?

NoRetryClause = "[" "no_retry" "]"

PauseStmt = "PAUSE" "WATCH" NodeRef
ResumeStmt = "RESUME" "WATCH" NodeRef
CancelStmt = "CANCEL" "WATCH" NodeRef
AlterWatchStmt = "ALTER" "WATCH" NodeRef "SET" WatchOptions

WatchTrigger = "ON" "MATCH" "DO" Action+
```

---

# Part XI: Summary

## 11.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Watch** | Persistent query that pushes matching changes |
| **Watch mode** | Non-destructive observation, broadcast |
| **Consume mode** | Destructive receipt, exactly-once per group |
| **Group** | Named set of competing consumers |
| **Acknowledgment** | Confirmation of successful consumption |
| **Window** | Time-based batching of events |

## 11.2 Mode Comparison

| Aspect | Watch | Consume |
|--------|-------|---------|
| Graph effect | None | Deletes matched |
| Multiple watchers | All see all | Each sees different |
| Acknowledgment | No | Required |
| Use case | Dashboards, monitoring | Queues, messaging |
| Delivery | Best-effort or reliable | Exactly-once |

## 11.3 Option Defaults

| Option | Default | Alternatives |
|--------|---------|--------------|
| mode | watch | consume |
| group | none | "name" |
| ordering | arrival | causal, attribute(...) |
| delivery | best_effort | reliable |
| visibility | committed | immediate |
| window | none | tumbling(...), sliding(...) |
| buffer | 1000 | N |
| initial | full | none, since(...) |

## 11.4 The Unified Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                      UNIFIED VIEW                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  WATCH is the single push primitive.                                │
│                                                                      │
│  What traditional systems call:           In MEW:                   │
│  ─────────────────────────────────────────────────────────────     │
│  Live query / real-time query     →   WATCH [mode: watch]          │
│  Message queue / channel          →   WATCH [mode: consume]        │
│  Topic / pubsub                   →   WATCH pattern [watch]        │
│  Work queue                       →   WATCH [consume, group]       │
│  Request-reply                    →   Pattern with correlation     │
│  Event stream                     →   WATCH [watch, causal]        │
│                                                                      │
│  All unified under pattern-based watch with modal options.         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of MEW Watch System Specification*
