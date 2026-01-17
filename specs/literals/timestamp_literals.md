---
spec: timestamp_literals
version: "1.0"
status: stable
category: literal
requires: []
priority: essential
---

# Spec: Timestamp Literals

## Overview

Timestamp literals provide human-readable syntax for timestamp values, avoiding manual millisecond calculations. Instead of `1705312200000`, write `@2024-01-15T10:30:00Z`.

---

## Syntax

### Grammar
```ebnf
Literal = ... | TimestampLiteral

TimestampLiteral = "@" Date ("T" Time Timezone?)?

Date = Year "-" Month "-" Day
Year = Digit Digit Digit Digit
Month = Digit Digit
Day = Digit Digit

Time = Hour ":" Minute (":" Second ("." Millisecond)?)?
Hour = Digit Digit
Minute = Digit Digit
Second = Digit Digit
Millisecond = Digit Digit Digit

Timezone = "Z" | ("+" | "-") Hour ":" Minute
```

### Keywords

None.

### Examples
```
@2024-01-15                     -- Date only (midnight UTC)
@2024-01-15T10:30:00Z           -- Full timestamp UTC
@2024-01-15T10:30:00+05:30      -- With timezone offset
@2024-01-15T10:30:00.500Z       -- With milliseconds
```

---

## Semantics

### Evaluation

Timestamp literals evaluate to `Timestamp` (milliseconds since Unix epoch).

| Literal | Value (ms) |
|---------|------------|
| `@1970-01-01T00:00:00Z` | 0 |
| `@2024-01-15T00:00:00Z` | 1705276800000 |

### Date-Only Form

Date-only literals default to midnight UTC:
```
@2024-01-15
-- Equivalent to: @2024-01-15T00:00:00Z
```

### Timezone Handling

- `Z` suffix means UTC
- `+HH:MM` or `-HH:MM` specifies offset from UTC
- No timezone defaults to UTC

All values are stored as UTC. The timezone offset is applied during parsing, not stored.
```
@2024-01-15T10:30:00+05:30
-- Stored as: @2024-01-15T05:00:00Z (UTC equivalent)
```

### Validation

Invalid dates are compile-time errors:
```
@2024-02-30    -- ERROR: invalid date
@2024-13-01    -- ERROR: month out of range
@2024-01-15T25:00:00Z  -- ERROR: hour out of range
```

---

## Layer 0

None. Timestamp literals compile directly to `_LiteralExpr` with `value_type: "Timestamp"`.

---

## Compilation
```
@2024-01-15T10:30:00Z
```

Compiles to:
```
_LiteralExpr node:
  value_type: "Timestamp"
  value_string: "1705315800000"
```

---

## Examples

### Attribute Defaults
```
node Event {
  created_at: Timestamp = now(),
  scheduled_for: Timestamp = @2024-12-31T23:59:59Z
}
```

### Constraints
```
constraint future_deadline:
  t: Task WHERE t.deadline != null
  => t.deadline > @2024-01-01T00:00:00Z
```

### Queries
```
MATCH e: Event
WHERE e.timestamp >= @2024-01-01 AND e.timestamp < @2025-01-01
RETURN e
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Invalid date | `"Invalid date: YYYY-MM-DD"` |
| Invalid time | `"Invalid time: HH:MM:SS"` |
| Day out of range | `"Day N invalid for month M"` |

---

*End of Spec: Timestamp Literals*