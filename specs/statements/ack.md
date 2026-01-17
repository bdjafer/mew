---
spec: ack
version: "1.0"
status: draft
category: statement
capability: watch
requires: [watch, consume_mode]
priority: essential
---

# Spec: ACK / NACK

## Overview

ACK and NACK statements acknowledge or reject message deliveries in consume mode watches. ACK confirms successful processing, causing the message to be deleted from the graph. NACK indicates processing failure, causing the message to be redelivered or moved to dead letter.

## Syntax

### Grammar

```ebnf
AckStmt       = "ACK" DeliveryId
              | "NACK" DeliveryId NoRetryClause? ;

NoRetryClause = "[" "no_retry" "]" ;

DeliveryId    = StringLiteral | Variable ;

Variable      = "$" Identifier ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `ACK` | Statement - confirms successful message processing |
| `NACK` | Statement - indicates message processing failure |
| `no_retry` | Modifier - prevents redelivery, moves to dead letter |

### Examples

```
-- Acknowledge successful processing
ACK "delivery_abc123"

-- Acknowledge using variable
ACK $deliveryId

-- Reject and request redelivery
NACK "delivery_abc123"

-- Reject without redelivery (move to dead letter)
NACK "delivery_abc123" [no_retry]
```

## Semantics

### ACK Behavior

When ACK executes:
1. The delivery ID is validated against pending deliveries
2. The matched node/edge is deleted from the graph
3. The delivery is marked as completed
4. No further action is taken for this message

```
-- Consumer receives event:
-- { type: "consumed", match: {...}, delivery_id: "abc123" }

-- After successful processing:
ACK "abc123"
-- Match deleted from graph, processing complete
```

### NACK Behavior

When NACK executes without `[no_retry]`:
1. The delivery ID is validated against pending deliveries
2. The matched node/edge is returned to the available pool
3. The delivery attempt counter is incremented
4. The message will be redelivered (to same or different consumer)

```
-- Processing failed, request redelivery:
NACK "abc123"
-- Match returned to pool, will be redelivered
```

### NACK with no_retry

When NACK executes with `[no_retry]`:
1. The delivery ID is validated against pending deliveries
2. If dead letter is configured: match is moved to dead letter destination
3. If dead letter is not configured: match is deleted
4. No redelivery occurs

```
-- Unrecoverable error, do not retry:
NACK "abc123" [no_retry]
-- Match moved to dead letter (if configured) or deleted
```

### Timeout Behavior

If no ACK/NACK is received within `ack_timeout`:
1. The message is treated as NACK (implicit rejection)
2. The delivery attempt counter is incremented
3. The message is redelivered

Configuration in watch:
```
WATCH j: Job WHERE j.status = "pending"
  [mode: consume]
  [ack_timeout: 30s]
  RETURN j
```

### Redelivery Limits

After `max_redeliveries` failed attempts (NACK or timeout):
1. The message is automatically moved to dead letter (if `dead_letter` configured)
2. Or automatically deleted (if no `dead_letter` configured)
3. No further redelivery attempts occur
4. A `_DeadLetter` node is created to record the failure

Configuration in watch:
```
WATCH j: Job WHERE j.status = "pending"
  [mode: consume]
  [max_redeliveries: 3]
  [dead_letter: #dlq_node]
  RETURN j
```

When a message exceeds `max_redeliveries`, it is treated equivalently to `NACK [no_retry]`.

### Delivery ID

The delivery ID is a unique identifier provided in the consumed event:

```
{
  type: "consumed",
  match: {...},
  delivery_id: "delivery_abc123",  -- use this ID for ACK/NACK
  tick: 42
}
```

The delivery ID:
- Is unique per delivery attempt
- May differ across redeliveries of the same message
- Must be used exactly once (either ACK or NACK)
- Expires after `ack_timeout`

## Layer 0

ACK/NACK statements interact with Layer 0 types defined in the watch system (see WATCH.md Appendix B).

### References

| Type | Relevance to ACK/NACK |
|------|----------------------|
| `_WatchEvent` | Provides `delivery_id` in consumed events |
| `_DeadLetter` | Created when messages fail permanently |

### _WatchEvent (from WATCH.md)

The `delivery_id` field is populated for consume mode events:

```
node _WatchEvent [sealed] {
  event_type: String [required],     -- "initial" | "added" | "removed" | "changed" | "consumed"
  tick: Int [required],
  delivery_id: String?,              -- present for consume mode; used with ACK/NACK
  created_at: Timestamp [required]
}
```

### _DeadLetter (from WATCH.md)

Created when a message fails permanently (NACK [no_retry] or max_redeliveries exceeded):

```
node _DeadLetter [sealed] {
  original_match: String [required], -- serialized match data
  failure_reason: String [required], -- "no_retry" | "max_redeliveries_exceeded"
  delivery_attempts: Int [required], -- total attempts before dead letter
  created_at: Timestamp [required]
}
```

### Edge

```
edge _watch_dead_letter(watch: _Watch, letter: _DeadLetter) {}
```

Links the originating watch to its dead letter entries.

## Examples

### Basic Consume Workflow

```
-- Worker consumes jobs
WATCH j: Job WHERE j.status = "pending"
  [mode: consume]
  [ack_timeout: 30s]
  [max_redeliveries: 3]
  [dead_letter: #failed_jobs]
  RETURN j

-- Receive event:
-- { type: "consumed", match: { id: "job_1", payload: "..." }, delivery_id: "d123" }

-- Process successfully:
ACK "d123"

-- Or on failure, request retry:
NACK "d123"

-- Or on permanent failure:
NACK "d123" [no_retry]
```

### Error Handling Pattern

Client-side pseudocode for handling consumed events:

```
-- Receive consumed event:
-- { type: "consumed", match: {...}, delivery_id: "d123" }

-- Application processes the match, then acknowledges:

-- On success:
ACK "d123"

-- On transient/retryable failure:
NACK "d123"

-- On permanent/unrecoverable failure:
NACK "d123" [no_retry]
```

### Competing Consumers

```
-- Multiple workers in same group
WATCH j: Job WHERE j.status = "pending"
  [mode: consume]
  [group: "workers"]
  [ack_timeout: 60s]
  RETURN j

-- Worker A receives job_1:
-- { delivery_id: "d_a1" }
ACK "d_a1"

-- Worker B receives job_2:
-- { delivery_id: "d_b1" }
NACK "d_b1"  -- job_2 redelivered to A or B
```

## Errors

Error codes from WATCH.md Part X:

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E8003 | ACK_TIMEOUT | No ACK/NACK received within `ack_timeout` | `Acknowledgment timeout: delivery '{delivery_id}' not acknowledged within {timeout}` |
| E8004 | INVALID_DELIVERY_ID | ACK/NACK references unknown or already-processed delivery | `Invalid delivery ID: '{delivery_id}' not found or already processed` |
| E8007 | DEAD_LETTER_FAILED | Failed to move message to dead letter destination | `Dead letter failed: could not move delivery '{delivery_id}' to '{dead_letter}' - {reason}` |

### Error Scenarios

**E8003 - ACK_TIMEOUT:**
- Triggered automatically by the system when `ack_timeout` expires
- Message is implicitly NACKed and redelivered
- Delivery attempt counter is incremented

**E8004 - INVALID_DELIVERY_ID:**
- Returned when ACK/NACK is called with an invalid delivery ID
- Causes: typo, delivery already acknowledged, delivery expired

**E8007 - DEAD_LETTER_FAILED:**
- Triggered when dead letter destination is unavailable or violates constraints
- Original message may be lost if dead letter cannot be written
