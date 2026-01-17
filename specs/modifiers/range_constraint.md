---
spec: range_constraint
version: "1.0"
status: draft
category: modifier
capability: range_constraint
requires: []
priority: common
---

# Spec: Range Constraint

## Overview

Range constraint modifiers (`>=`, `<=`, `>`, `<`, and `N..M`) constrain numeric attribute values to specified bounds. They enforce domain-specific limits on integers and floats.

**Why needed:** Numeric attributes often have valid ranges (ages 0-150, priorities 1-10, percentages 0-100). Range constraints enforce these limits at the schema level.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | ComparisonModifier | RangeModifier

ComparisonModifier =
    ">=" Literal
  | "<=" Literal
  | ">" Literal
  | "<" Literal

RangeModifier = NumericLiteral ".." NumericLiteral

NumericLiteral = IntLiteral | FloatLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `>=` | Greater than or equal (inclusive minimum) |
| `<=` | Less than or equal (inclusive maximum) |
| `>` | Greater than (exclusive minimum) |
| `<` | Less than (exclusive maximum) |
| `..` | Range shorthand (inclusive both ends) |

### Examples
```
node Person {
  age: Int [>= 0, <= 150],
  score: Float [>= 0.0, <= 1.0]
}

node Task {
  priority: Int [0..10],              -- shorthand for [>= 0, <= 10]
  progress: Float [>= 0.0, < 100.0]   -- exclusive upper bound
}
```

---

## Semantics

### Comparison Modifiers

| Modifier | Meaning | Example |
|----------|---------|---------|
| `>= N` | Value must be >= N | `age: Int [>= 0]` |
| `<= N` | Value must be <= N | `age: Int [<= 150]` |
| `> N` | Value must be > N | `temp: Float [> -273.15]` |
| `< N` | Value must be < N | `percent: Float [< 100.0]` |

### Range Shorthand

The `N..M` syntax is shorthand for `[>= N, <= M]`:
```
priority: Int [0..10]
-- Equivalent to:
priority: Int [>= 0, <= 10]
```

### Combining Bounds

Multiple comparison modifiers can be combined:
```
-- Min and max
age: Int [>= 0, <= 150]

-- Exclusive bounds
score: Float [> 0.0, < 1.0]

-- Mixed inclusive/exclusive
percentage: Float [>= 0.0, < 100.0]
```

### Validation Timing

Range validation occurs at:
1. **SPAWN time**: Value must satisfy bounds
2. **SET time**: New value must satisfy bounds

### Null Handling

Range validation is skipped for null values:
```
node Person {
  age: Int? [>= 0, <= 150]
}

SPAWN p: Person { name = "Alice" }  -- age = null, OK
SPAWN p: Person { name = "Alice", age = null }  -- OK
SPAWN p: Person { name = "Alice", age = -5 }  -- ERROR
```

**To require a value AND validate range**, combine with `[required]`:
```
age: Int [required, >= 0, <= 150]
```

### Type Compatibility

Range modifiers work with numeric types:

| Type | Supported Bounds |
|------|------------------|
| `Int` | Integer literals |
| `Float` | Float or integer literals |

```
-- Int with Int bounds
count: Int [>= 0, <= 1000]

-- Float with Float bounds
ratio: Float [>= 0.0, <= 1.0]

-- Float with Int bounds (promoted)
temperature: Float [>= -50, <= 50]
```

### Compilation to Constraints
```
age: Int [>= 0, <= 150]
```

Compiles to:
```
constraint <type>_<attr>_min:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> >= 0

constraint <type>_<attr>_max:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> <= 150
```

---

## Layer 0

None. Range constraints compile to standard comparison constraints.

---

## Examples

### Age and Ratings
```
ontology Users {
  node User {
    -- Age: 0 to 150 years
    age: Int? [>= 0, <= 150],

    -- Rating: 0.0 to 5.0
    rating: Float [>= 0.0, <= 5.0] = 0.0,

    -- Reputation: no upper limit
    reputation: Int [>= 0] = 0
  }
}

-- Valid
SPAWN u: User { name = "Alice", age = 25, rating = 4.5 }

-- Invalid: age out of range
SPAWN u: User { name = "Bob", age = -1 }
-- ERROR: Attribute 'age' value -1 is below minimum 0

SPAWN u: User { name = "Charlie", age = 200 }
-- ERROR: Attribute 'age' value 200 exceeds maximum 150
```

### Priority Systems
```
ontology Tasks {
  type Priority = Int [0..10]

  node Task {
    priority: Priority = 5
  }

  node Epic {
    priority: Priority = 7,
    urgency: Int [1..5] = 3
  }
}
```

### Scientific Measurements
```
ontology Physics {
  node Measurement {
    -- Temperature in Kelvin (absolute zero minimum)
    temperature_k: Float [>= 0.0],

    -- Probability (0 to 1, exclusive of endpoints)
    probability: Float [> 0.0, < 1.0],

    -- Percentage (0 to 100 inclusive)
    confidence_pct: Float [0.0..100.0]
  }
}
```

### Financial Constraints
```
ontology Finance {
  node Account {
    -- Balance can be negative (overdraft)
    balance: Float,

    -- Credit limit is non-negative
    credit_limit: Float [>= 0.0] = 0.0,

    -- Interest rate as decimal
    interest_rate: Float [>= 0.0, <= 1.0]
  }

  node Transaction {
    -- Amount must be positive
    amount: Float [> 0.0],

    -- Tax rate as percentage
    tax_rate: Float [>= 0.0, <= 100.0] = 0.0
  }
}
```

### Type Alias with Range
```
ontology Types {
  type Percentage = Float [0.0..100.0]
  type PositiveInt = Int [>= 1]
  type NonNegativeInt = Int [>= 0]
  type Rating = Float [0.0..5.0]

  node Task {
    completion: Percentage = 0.0,
    priority: PositiveInt,
    views: NonNegativeInt = 0
  }

  node Review {
    rating: Rating [required],
    helpfulness: Percentage?
  }
}
```

### Overriding Alias Bounds
```
type Priority = Int [0..10]

node Task {
  -- Use alias defaults
  standard_priority: Priority = 5,

  -- Override with stricter bounds (usage wins on conflict)
  high_priority: Priority [<= 5] = 3
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Below minimum (>=) | `"Attribute '<attr>' value <val> is below minimum <min>"` |
| Above maximum (<=) | `"Attribute '<attr>' value <val> exceeds maximum <max>"` |
| Below exclusive (>) | `"Attribute '<attr>' value <val> must be greater than <min>"` |
| Above exclusive (<) | `"Attribute '<attr>' value <val> must be less than <max>"` |
| Invalid range | `"Range minimum <min> cannot exceed maximum <max>"` |
| Type mismatch | `"Range constraint on '<attr>' requires numeric type, got <type>"` |

---

*End of Spec: Range Constraint*
