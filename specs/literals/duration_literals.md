---
spec: duration_literals
version: "1.0"
status: stable
category: literal
requires: [duration_type]
priority: essential
---

# Spec: Duration Literals

## Overview

Duration literals provide human-readable syntax for duration values. Instead of `86400000`, write `1.day`. Instead of `3600000`, write `1.hour`.

---

## Syntax

### Grammar
```ebnf
Literal = ... | DurationLiteral

DurationLiteral = IntLiteral "." DurationUnit

DurationUnit = 
    "millisecond" | "milliseconds" | "ms"
  | "second" | "seconds" | "s"
  | "minute" | "minutes" | "min"
  | "hour" | "hours" | "h"
  | "day" | "days"
  | "week" | "weeks"
```

### Keywords

None. Duration units are contextual (only after `.` following an integer).

### Examples
```
1.millisecond
500.ms
1.second
30.seconds
5.minutes
15.min
1.hour
2.hours
1.day
7.days
2.weeks
```

---

## Semantics

### Unit Conversion

| Unit | Milliseconds |
|------|--------------|
| `millisecond`, `ms` | 1 |
| `second`, `s` | 1,000 |
| `minute`, `min` | 60,000 |
| `hour`, `h` | 3,600,000 |
| `day` | 86,400,000 |
| `week` | 604,800,000 |

### Evaluation

Duration literals evaluate to `Duration`:
```
1.day       -- 86400000
2.hours     -- 7200000
30.minutes  -- 1800000
```

### Arithmetic

Duration literals can be combined:
```
1.day + 2.hours + 30.minutes
-- 86400000 + 7200000 + 1800000 = 95400000

7.days - 1.day
-- 518400000
```

### Negative Durations

Use unary minus:
```
-1.day      -- -86400000
-30.minutes -- -1800000
```

### Singular vs Plural

Both forms are accepted:
```
1.day       -- OK
1.days      -- OK (same value)
2.day       -- OK (grammatically odd but valid)
2.days      -- OK
```

---

## Layer 0

None. Duration literals compile directly to `_LiteralExpr` with `value_type: "Duration"`.

---

## Compilation
```
7.days
```

Compiles to:
```
_LiteralExpr node:
  value_type: "Duration"
  value_string: "604800000"
```

---

## Examples

### Attribute Defaults
```
node Token {
  ttl: Duration = 24.hours,
  refresh_interval: Duration = 1.hour
}

node CacheEntry {
  max_age: Duration = 5.minutes
}
```

### Dynamic Defaults
```
node Invitation {
  created_at: Timestamp = now(),
  expires_at: Timestamp = now() + 7.days
}
```

### Constraints
```
constraint reasonable_ttl:
  t: Token
  => t.ttl >= 1.minute AND t.ttl <= 365.days
```

### Queries
```
MATCH t: Token
WHERE t.issued_at + t.ttl < now()
RETURN t
-- Find expired tokens

MATCH e: Event
WHERE e.timestamp > now() - 24.hours
RETURN e
-- Events in last 24 hours
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Unknown unit | `"Unknown duration unit 'x'"` |
| Non-integer base | `"Duration literal requires integer base"` |

---

*End of Spec: Duration Literals*