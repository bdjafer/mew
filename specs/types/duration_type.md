---
spec: duration_type
version: "1.0"
status: stable
category: type
requires: ["timestamp_literals"]
---

# Spec: Duration Type

## Overview

The `Duration` type represents time spans in milliseconds. It enables type-safe arithmetic with timestamps: adding a duration to a timestamp yields a timestamp, subtracting two timestamps yields a duration.

---

## Syntax

### Grammar
```ebnf
ScalarType = ... | "Duration"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `Duration` | Type expression |

### Examples
```
node Token {
  ttl: Duration [required],
  refresh_interval: Duration = 3600000  -- 1 hour in ms
}
```

---

## Semantics

### Representation

Duration is stored as a 64-bit signed integer representing milliseconds.

| Range | Value |
|-------|-------|
| Minimum | −2⁶³ ms (~292 million years) |
| Maximum | 2⁶³−1 ms (~292 million years) |

Negative durations are valid (represent "before").

### Operations

| Operation | Types | Result |
|-----------|-------|--------|
| `d1 + d2` | Duration + Duration | Duration |
| `d1 - d2` | Duration − Duration | Duration |
| `d * n` | Duration × Int | Duration |
| `d / n` | Duration ÷ Int | Duration |
| `t + d` | Timestamp + Duration | Timestamp |
| `t - d` | Timestamp − Duration | Timestamp |
| `t1 - t2` | Timestamp − Timestamp | Duration |
| `d1 = d2` | Duration = Duration | Bool |
| `d1 < d2` | Duration < Duration | Bool |

### Built-in Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `to_milliseconds(d)` | Duration → Int | Raw milliseconds |
| `to_seconds(d)` | Duration → Int | Truncated to seconds |
| `to_minutes(d)` | Duration → Int | Truncated to minutes |
| `to_hours(d)` | Duration → Int | Truncated to hours |
| `to_days(d)` | Duration → Int | Truncated to days |
| `abs(d)` | Duration → Duration | Absolute value |

---

## Layer 0

### Scalar Type

Duration is added to the set of scalar types:
```
ScalarType = "String" | "Int" | "Float" | "Bool" | "Timestamp" | "Duration"
```

`_ScalarTypeExpr.scalar_type` can now be `"Duration"`.

---

## Compilation
```
node Token {
  ttl: Duration [required]
}
```

Compiles to:
```
_AttributeDef node:
  name: "ttl"
  scalar_type: "Duration"
  required: true
```

---

## Examples

### Token Expiration
```
node Token {
  issued_at: Timestamp = now(),
  ttl: Duration [required],
  
  -- Computed in query, not stored
  -- expires_at would be: issued_at + ttl
}

constraint token_valid_ttl:
  t: Token
  => t.ttl > 0
```

### Scheduling
```
node ScheduledJob {
  interval: Duration [required],
  last_run: Timestamp?,
  next_run: Timestamp
}

constraint next_after_last:
  j: ScheduledJob WHERE j.last_run != null
  => j.next_run = j.last_run + j.interval
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Duration ÷ 0 | `"Division by zero"` |
| Overflow | `"Duration overflow"` |

---

*End of Spec: Duration Type*