---
spec: timestamp_functions
version: "1.0"
status: draft
category: expression
capability: temporal operations
requires: []
priority: common
---

# Spec: Timestamp Functions

## Overview

Timestamp functions provide operations for working with temporal values. These functions enable querying the current time, extracting date/time components from timestamps, and parsing timestamp values from strings. All timestamp operations use UTC (Coordinated Universal Time) as the reference timezone. Timestamps are stored internally as milliseconds since the Unix epoch (1970-01-01T00:00:00Z).

## Syntax

### Grammar

```ebnf
TimestampFunctionCall =
    "now" "(" ")"
  | "year" "(" Expr ")"
  | "month" "(" Expr ")"
  | "day" "(" Expr ")"
  | "hour" "(" Expr ")"
  | "minute" "(" Expr ")"
  | "second" "(" Expr ")"
  | "millisecond" "(" Expr ")"
  | "day_of_week" "(" Expr ")"
  | "timestamp" "(" Expr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `now` | Expression - current timestamp |
| `year` | Expression - extract year component |
| `month` | Expression - extract month component (1-12) |
| `day` | Expression - extract day of month (1-31) |
| `hour` | Expression - extract hour component (0-23) |
| `minute` | Expression - extract minute component (0-59) |
| `second` | Expression - extract second component (0-59) |
| `millisecond` | Expression - extract millisecond component (0-999) |
| `day_of_week` | Expression - extract day of week (0-6, Sunday=0) |
| `timestamp` | Expression - parse string to timestamp |

### Examples

```
-- Get current time
now()

-- Extract year from a timestamp
year(t.created_at)

-- Check if event occurred on a weekend
day_of_week(e.timestamp) = 0 OR day_of_week(e.timestamp) = 6

-- Parse an ISO timestamp string
timestamp("2024-01-15T10:30:00Z")
```

## Semantics

### Function Signatures and Behavior

| Function | Signature | Description |
|----------|-----------|-------------|
| `now()` | `() -> Timestamp` | Returns the current time as a timestamp |
| `year(t)` | `Timestamp -> Int` | Extracts the year (e.g., 2024) |
| `month(t)` | `Timestamp -> Int` | Extracts the month (1 = January, 12 = December) |
| `day(t)` | `Timestamp -> Int` | Extracts the day of month (1-31) |
| `hour(t)` | `Timestamp -> Int` | Extracts the hour in 24-hour format (0-23) |
| `minute(t)` | `Timestamp -> Int` | Extracts the minute (0-59) |
| `second(t)` | `Timestamp -> Int` | Extracts the second (0-59) |
| `millisecond(t)` | `Timestamp -> Int` | Extracts the millisecond (0-999) |
| `day_of_week(t)` | `Timestamp -> Int` | Extracts the day of week (0 = Sunday, 6 = Saturday) |
| `timestamp(s)` | `String -> Timestamp` | Parses an ISO 8601 formatted string to a timestamp |

### Type Rules

- Extraction functions (`year`, `month`, `day`, etc.) require a `Timestamp` or `Timestamp?` argument
- All extraction functions return `Int` (or `Int?` if input is nullable)
- The `timestamp` function requires a `String` argument and returns `Timestamp`
- The `now` function takes no arguments and returns `Timestamp`
- If any argument is `null`, the function returns `null` (null propagation)

### now() Semantics

- Returns the current UTC time at the moment of evaluation
- Within a single statement, `now()` may return different values if evaluated multiple times (implementation-defined)
- For transactional consistency, consider capturing `now()` once and referencing the result

### Component Ranges

| Component | Range | Notes |
|-----------|-------|-------|
| `year` | 1970 - 9999+ | Supports dates from Unix epoch onward |
| `month` | 1 - 12 | 1 = January |
| `day` | 1 - 31 | Varies by month |
| `hour` | 0 - 23 | 24-hour format |
| `minute` | 0 - 59 | |
| `second` | 0 - 59 | Leap seconds not represented |
| `millisecond` | 0 - 999 | |
| `day_of_week` | 0 - 6 | 0 = Sunday |

### timestamp() Parsing

The `timestamp` function parses ISO 8601 formatted strings:

**Supported formats:**
```
"2024-01-15"                    -- Date only (midnight UTC)
"2024-01-15T10:30:00Z"          -- Full with UTC
"2024-01-15T10:30:00+05:30"     -- With timezone offset
"2024-01-15T10:30:00.500Z"      -- With milliseconds
"2024-01-15T10:30:00"           -- No timezone (assumed UTC)
```

**Behavior:**
- If no timezone is specified, UTC is assumed
- Timezone offsets are converted to UTC for storage
- Invalid format strings produce a runtime error

### Timezone Handling

All operations use UTC:
- `now()` returns UTC time
- Extraction functions extract UTC components
- `timestamp()` normalizes all inputs to UTC
- No timezone conversion functions are provided (handle at application layer)

### Leap Seconds

Leap seconds are not explicitly represented. The `second` component is always 0-59.

## Layer 0

None.

## Examples

### Filtering by Time

```
-- Find events from the last 24 hours
MATCH e: Event
WHERE e.timestamp > now() - 86400000   -- 24 hours in milliseconds
RETURN e

-- Find events from today
MATCH e: Event
WHERE year(e.timestamp) = year(now())
  AND month(e.timestamp) = month(now())
  AND day(e.timestamp) = day(now())
RETURN e

-- Using duration literals (if supported)
MATCH e: Event
WHERE e.timestamp > now() - 1.day
RETURN e
```

### Grouping by Time Components

```
-- Count events by month
MATCH e: Event
RETURN year(e.timestamp) AS yr,
       month(e.timestamp) AS mo,
       COUNT(e) AS event_count
ORDER BY yr, mo

-- Group tasks by day of week created
MATCH t: Task
RETURN day_of_week(t.created_at) AS weekday, COUNT(t) AS task_count
ORDER BY weekday
```

### Working Hours and Weekends

```
-- Find events during business hours (9 AM - 5 PM UTC)
MATCH e: Event
WHERE hour(e.timestamp) >= 9 AND hour(e.timestamp) < 17
RETURN e

-- Find weekend events
MATCH e: Event
WHERE day_of_week(e.timestamp) = 0 OR day_of_week(e.timestamp) = 6
RETURN e

-- Find events outside business hours
MATCH e: Event
WHERE hour(e.timestamp) < 9 OR hour(e.timestamp) >= 17
RETURN e
```

### Parsing and Creating Timestamps

```
-- Parse a timestamp from a string field
MATCH r: Record
WHERE r.date_string IS NOT NULL
SET r.parsed_date = timestamp(r.date_string)

-- Compare against a specific date
MATCH t: Task
WHERE t.due_date < timestamp("2024-06-01T00:00:00Z")
RETURN t.title, t.due_date
```

### Date Comparisons

```
-- Find overdue tasks
MATCH t: Task
WHERE t.due_date < now() AND t.status != "done"
RETURN t.title, t.due_date

-- Find tasks due this month
MATCH t: Task
WHERE year(t.due_date) = year(now())
  AND month(t.due_date) = month(now())
RETURN t

-- Find events in Q1 (January-March)
MATCH e: Event
WHERE year(e.timestamp) = 2024 AND month(e.timestamp) <= 3
RETURN e
```

### Timestamp Arithmetic

```
-- Set reminder for 1 week before due date
MATCH t: Task
SET t.reminder_at = t.due_date - 604800000   -- 7 days in milliseconds

-- Calculate age in days
MATCH t: Task
RETURN t.title, (now() - t.created_at) / 86400000 AS age_days

-- Using duration literals
MATCH t: Task
SET t.reminder_at = t.due_date - 7.days
RETURN t.title, (now() - t.created_at) / 1.day AS age_days
```

### Combining with Other Functions

```
-- Find tasks created in the last hour of each day
MATCH t: Task
WHERE hour(t.created_at) = 23
RETURN t

-- Find events with specific minute precision
MATCH e: Event
WHERE minute(e.timestamp) = 0 AND second(e.timestamp) = 0
RETURN e  -- Events exactly on the hour
```

## Errors

| Condition | Message |
|-----------|---------|
| Non-timestamp argument to extraction function | Type error: `year` expects Timestamp, got String |
| Non-string argument to timestamp() | Type error: `timestamp` expects String, got Int |
| Invalid timestamp format | Runtime error: cannot parse "invalid" as timestamp. Expected ISO 8601 format |
| Arguments to now() | Syntax error: `now` takes no arguments |
| Timestamp before epoch (negative) | Runtime error: timestamp values before 1970-01-01 not supported |
