---
spec: optional_type
version: "1.0"
status: draft
category: type
capability: nullable_values
requires: []
priority: essential
---

# Spec: Optional Type

## Overview

The optional type (`T?`) enables nullable semantics for any base type, allowing attributes and expressions to hold either a value of type T or null. Optional types are essential for representing missing, unknown, or inapplicable data in graph schemas and queries, supporting null propagation through expressions and providing safe handling of absent values via coalesce operations.

## Syntax

### Grammar

```ebnf
TypeExpr         = UnionType
UnionType        = OptionalType ("|" OptionalType)*
OptionalType     = PrimaryType "?"?
PrimaryType      = NamedType | EdgeRefType | AnyType | ScalarType | "(" TypeExpr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `?` | Type modifier (postfix operator) |
| `null` | Literal value for optional types |
| `is` | Null check operator (`IS NULL`, `IS NOT NULL`) |
| `coalesce` | Function for null handling |
| `??` | Infix coalesce operator |

### Examples

```
-- Optional scalar types
name: String?                   -- String or null
age: Int?                       -- Int or null
score: Float?                   -- Float or null

-- Optional named types
assignee: Person?               -- Person or null
parent: Task?                   -- Task or null

-- Optional with union (union binds looser)
item: Task | Project?           -- means: Task | (Project?)
owner: (Person | Bot)?          -- means: (Person | Bot) or null

-- Null literal assignment
description: String? = null     -- explicit null default
```

## Semantics

### Nullability Rules

1. **Required vs Optional:**
   - Required attribute (`T`): MUST have a non-null value
   - Optional attribute (`T?`): MAY be null

2. **Type Compatibility:**
   - `T <: T?` (non-null is subtype of optional)
   - `null <: T?` (null is valid for any optional type)
   - `T? </: T` (optional is NOT subtype of required)

3. **Double Optional:**
   - `T??` is equivalent to `T?` (idempotent)
   - The parser normalizes `T??` to `T?`

### Null Propagation

Operations involving null propagate null through the expression:

| Expression | Result |
|------------|--------|
| `null + 1` | `null` |
| `null * x` | `null` |
| `length(null)` | `null` |
| `null ++ "text"` | `null` |
| `x.attr` where `x` is `null` | `null` |

### Null Comparison

| Expression | Result | Notes |
|------------|--------|-------|
| `null = null` | `true` | Equality with null |
| `null != null` | `false` | Inequality with null |
| `null < x` | `false` | Ordering with null |
| `null > x` | `false` | Ordering with null |
| `x IS NULL` | `true` if x is null | SQL-style check |
| `x IS NOT NULL` | `true` if x is non-null | SQL-style check |

### Null in Boolean Context

| Expression | Result |
|------------|--------|
| `null and x` | `false` |
| `null or true` | `true` |
| `null or false` | `false` |
| `not null` | `null` |

### Coalesce Semantics

The coalesce operation returns the first non-null value:

```
coalesce(x, y, z)    -- first non-null of x, y, z
x ?? y               -- shorthand for coalesce(x, y)
x ?? y ?? z          -- right-associative: x ?? (y ?? z)
```

**Type inference for coalesce:**
- If the last argument is non-null type T, result is T
- If all arguments are nullable, result is T?

```
coalesce(x: String?, "default")     -- Result: String (last is non-null)
coalesce(x: String?, y: String?)    -- Result: String? (both nullable)
```

### Modifier Interaction with NULL

Value constraint modifiers only apply when the attribute is non-null:

```
age: Int? [>= 0]     -- NULL is valid; if non-null, must be >= 0
```

**Behavior:**
| Value | Constraint Check |
|-------|------------------|
| `age = 25` | Checked: 25 >= 0 passes |
| `age = -5` | Checked: -5 >= 0 fails |
| `age = null` | Skipped: constraint does not apply |

**To require non-null AND validate:**
```
age: Int [required, >= 0]   -- Must be present AND >= 0
```

## Layer 0

### Nodes

```
_OptionalTypeExpr node:
  -- Represents an optional type in the AST
```

### Edges

```
_optional_inner edge:
  (optional_type_expr, inner_type_expr)
  -- Links optional type to its base type
```

### Constraints

None. Optional type behavior is built into the type system.

## Examples

### Schema Definition with Optional Types

```
node Person {
  -- Required fields
  name: String [required],
  email: String [required, unique],

  -- Optional fields
  phone: String?,
  age: Int? [>= 0, <= 150],
  bio: String? [length: 0..2000],

  -- Optional with default (uses null if not provided)
  nickname: String? = null,

  -- Optional timestamp
  last_login: Timestamp?
}

node Task {
  title: String [required],
  description: String?,
  due_date: Timestamp?,
  completed_at: Timestamp?,
  priority: Int [required] = 5
}
```

### Query with Null Handling

```
-- Find tasks without due dates
MATCH t: Task
WHERE t.due_date IS NULL
RETURN t.title

-- Use coalesce for default values in output
MATCH p: Person
RETURN p.name, coalesce(p.nickname, p.name) AS display_name

-- Filter by optional field when present
MATCH t: Task
WHERE t.due_date IS NOT NULL AND t.due_date < now()
RETURN t.title AS overdue_task

-- Chained coalesce
MATCH p: Person
RETURN coalesce(p.phone, p.email, "no contact") AS contact
```

### Rule with Optional Handling

```
-- Auto-populate completed_at when status changes
rule set_completed_timestamp [priority: 100]:
  t: Task
  WHERE t.status = "done" AND t.completed_at IS NULL
  =>
  SET t.completed_at = now()

-- Clear optional field conditionally
rule clear_assignee_on_archive:
  t: Task, p: Person, assigned_to(t, p)
  WHERE t.status = "archived"
  =>
  UNLINK assigned_to(t, p)
```

### Edge with Optional Attributes

```
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp = now(),
  role: String = "owner",
  notes: String?              -- Optional notes
}

-- Link with optional attribute omitted
LINK assigned_to(t, p) { role = "reviewer" }
-- notes will be null

-- Link with optional attribute provided
LINK assigned_to(t, p) { role = "owner", notes = "Primary assignee" }
```

## Errors

| Condition | Message |
|-----------|---------|
| Assigning null to non-optional type | `Type error: Cannot assign null to non-nullable type 'T'` |
| Missing required attribute at SPAWN | `Constraint violation: Required attribute 'attr' not provided for type 'T'` |
| Nullable + required modifier | `Compile error: Attribute 'attr' cannot be both nullable (?) and [required]` |
| SET required attribute to null | `Constraint violation: Cannot set required attribute 'attr' to null` |
| Type mismatch in coalesce | `Type error: Incompatible types in coalesce: 'T' and 'U'` |
