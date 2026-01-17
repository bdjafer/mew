---
spec: numeric_functions
version: "1.0"
status: draft
category: expression
capability: numeric computation
requires: []
priority: common
---

# Spec: Numeric Functions

## Overview

Numeric functions provide mathematical operations on `Int` and `Float` values. These functions support absolute value computation, finding minimum and maximum values, and rounding operations. They are essential for data analysis, aggregation, and mathematical transformations in queries and rules.

## Syntax

### Grammar

```ebnf
NumericFunctionCall =
    "abs" "(" Expr ")"
  | "min" "(" Expr "," Expr ")"
  | "max" "(" Expr "," Expr ")"
  | "floor" "(" Expr ")"
  | "ceil" "(" Expr ")"
  | "round" "(" Expr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `abs` | Expression - absolute value |
| `min` | Expression - minimum of two values |
| `max` | Expression - maximum of two values |
| `floor` | Expression - round down to integer |
| `ceil` | Expression - round up to integer |
| `round` | Expression - round to nearest integer |

### Examples

```
-- Get absolute value of a difference
abs(t.actual - t.estimated)

-- Find the smaller of two values
min(t.priority, 10)

-- Round a floating point value
round(t.rating)

-- Floor a value to truncate decimals
floor(price / 100.0) * 100
```

## Semantics

### Function Signatures and Behavior

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs(n)` | `Int -> Int` or `Float -> Float` | Returns the absolute (non-negative) value |
| `min(a, b)` | `Int x Int -> Int` or `Float x Float -> Float` | Returns the smaller of two values |
| `max(a, b)` | `Int x Int -> Int` or `Float x Float -> Float` | Returns the larger of two values |
| `floor(f)` | `Float -> Int` | Rounds down toward negative infinity |
| `ceil(f)` | `Float -> Int` | Rounds up toward positive infinity |
| `round(f)` | `Float -> Int` | Rounds to the nearest integer (half rounds up) |

### Type Rules

**abs function:**
- If argument is `Int`, returns `Int`
- If argument is `Float`, returns `Float`
- Type is preserved

**min/max functions:**
- Both arguments must have compatible numeric types
- If both are `Int`, returns `Int`
- If both are `Float`, returns `Float`
- If one is `Int` and one is `Float`, the `Int` is coerced to `Float` and result is `Float`

**Rounding functions (floor, ceil, round):**
- Input must be `Float` (or `Float?`)
- Output is `Int` (or `Int?` if input is nullable)
- These are the explicit conversion functions from `Float` to `Int`

**Null propagation:**
- If any argument is `null`, the function returns `null`
- Return type becomes nullable if any input is nullable

### Detailed Behavior

**abs:**
```
abs(5)     -> 5
abs(-5)    -> 5
abs(0)     -> 0
abs(-3.14) -> 3.14
abs(3.14)  -> 3.14
```

**min/max:**
```
min(3, 7)     -> 3
max(3, 7)     -> 7
min(-5, -3)   -> -5
max(-5, -3)   -> -3
min(3.5, 2.1) -> 2.1
max(3.5, 2.1) -> 3.5
```

**floor (toward negative infinity):**
```
floor(3.7)  -> 3
floor(3.2)  -> 3
floor(-3.2) -> -4
floor(-3.7) -> -4
floor(3.0)  -> 3
```

**ceil (toward positive infinity):**
```
ceil(3.2)  -> 4
ceil(3.7)  -> 4
ceil(-3.2) -> -3
ceil(-3.7) -> -3
ceil(3.0)  -> 3
```

**round (to nearest, half rounds up):**
```
round(3.4)  -> 3
round(3.5)  -> 4
round(3.6)  -> 4
round(-3.4) -> -3
round(-3.5) -> -3   -- Note: half rounds toward positive infinity
round(-3.6) -> -4
```

### Special Float Values

**NaN (Not a Number):**
- `abs(NaN)` returns `NaN`
- `min(NaN, x)` returns `NaN`
- `max(NaN, x)` returns `NaN`
- `floor(NaN)`, `ceil(NaN)`, `round(NaN)` produce a runtime error

**Infinity:**
- `abs(-Infinity)` returns `Infinity`
- `min(Infinity, x)` returns `x` (for finite x)
- `max(-Infinity, x)` returns `x` (for finite x)
- `floor(Infinity)`, `ceil(Infinity)`, `round(Infinity)` produce a runtime error

### Implicit Int to Float Coercion

When `min` or `max` receives mixed `Int` and `Float` arguments, the `Int` is implicitly coerced to `Float`:

```
min(5, 3.2)   -> 3.2   (5 coerced to 5.0, result is Float)
max(5, 3.2)   -> 5.0   (result is Float, not Int)
```

Explicit conversion from `Float` to `Int` requires `floor`, `ceil`, or `round`:

```
-- VALID: Explicit conversion
x: Int = floor(3.7)

-- INVALID: Implicit conversion not allowed
x: Int = 3.7   -- Type error
```

## Layer 0

None.

## Examples

### Absolute Value for Differences

```
-- Find tasks where estimate was off by more than 2 hours
MATCH t: Task
WHERE abs(t.actual_hours - t.estimated_hours) > 2
RETURN t.title,
       t.estimated_hours,
       t.actual_hours,
       abs(t.actual_hours - t.estimated_hours) AS variance
```

### Clamping Values with Min/Max

```
-- Clamp priority to valid range [1, 10]
MATCH t: Task
RETURN t.title,
       max(1, min(t.priority, 10)) AS clamped_priority

-- Ensure non-negative values
MATCH t: Transaction
RETURN t.id, max(0, t.amount) AS safe_amount
```

### Rounding for Display or Calculation

```
-- Round ratings to whole numbers
MATCH p: Product
RETURN p.name, round(p.average_rating) AS rating_stars

-- Calculate price tiers (floor to nearest 100)
MATCH p: Product
RETURN p.name,
       p.price,
       floor(p.price / 100.0) * 100 AS price_tier

-- Round up for allocation (always allocate at least ceiling)
MATCH r: Resource
RETURN r.name, ceil(r.demand / r.unit_size) AS units_needed
```

### Combining with Aggregations

```
-- Find maximum absolute deviation from average
MATCH t: Task
RETURN max(abs(t.priority - 5), abs(t.priority - 5)) AS max_deviation

-- Get the floor of the average priority
MATCH t: Task, p: Project, belongs_to(t, p)
RETURN p.name, floor(AVG(t.priority)) AS avg_priority_floor
```

### Type-Safe Numeric Conversions

```
-- Convert float measurement to integer for storage
MATCH m: Measurement
SET m.value_int = round(m.value_float)

-- Truncate decimal for display
MATCH t: Transaction
RETURN t.id, floor(t.amount) AS amount_dollars

-- Round up for billing (never undercharge)
MATCH u: Usage
RETURN u.id, ceil(u.hours) AS billable_hours
```

## Errors

| Condition | Message |
|-----------|---------|
| Non-numeric argument | Type error: `abs` expects Int or Float, got String |
| Incompatible types in min/max | Type error: `min` expects matching numeric types, got Int and String |
| Float->Int without explicit conversion | Type error: cannot assign Float to Int. Use floor(), ceil(), or round() |
| floor/ceil/round on Int | Type error: `floor` expects Float, got Int (Int is already an integer) |
| floor/ceil/round on NaN | Runtime error: cannot convert NaN to Int |
| floor/ceil/round on Infinity | Runtime error: cannot convert Infinity to Int |
