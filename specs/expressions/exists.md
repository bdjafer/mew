---
spec: exists
version: "1.0"
status: stable
category: expression
capability: existence_testing
requires: []
priority: essential
---

# Spec: Exists

## Overview

Exists patterns test for the presence or absence of graph structure within constraint conditions and WHERE clauses. Most useful constraints need to express "there exists" or "there does not exist" conditions. Without this, constraints can only reason about bound variables, not about the existence of related structure.

## Syntax

### Grammar

```ebnf
ExistsExpr    = "EXISTS" "(" ExistsPattern ")"
              | "NOT" "EXISTS" "(" ExistsPattern ")"

ExistsPattern = PatternElement ("," PatternElement)* WhereClause?

PrimaryExpr   = ... | ExistsExpr
```

### Keywords

| Keyword | Context |
|---------|---------|
| `EXISTS` | Expression |
| `NOT` | Modifier (already a core keyword) |

### Examples

```
-- There exists an assignment
EXISTS(assigned_to(task, _))

-- There exists a person with specific role
EXISTS(p: Person, assigned_to(task, p) AS a WHERE a.role = "owner")

-- Negated: no assignment exists
NOT EXISTS(assigned_to(task, _))

-- Complex: no blocking dependency
NOT EXISTS(
  upstream: Task,
  depends_on(task, upstream)
  WHERE upstream.status != "done"
)
```

## Semantics

### Evaluation

`EXISTS(pattern)` evaluates to:
- `true` if at least one match for `pattern` exists
- `false` if no matches exist

`NOT EXISTS(pattern)` evaluates to:
- `true` if no matches exist
- `false` if at least one match exists

### Variable Scoping

**Outer variables are visible inside EXISTS:**
```
constraint task_needs_owner:
  t: Task
  => EXISTS(p: Person, assigned_to(t, p) AS a WHERE a.role = "owner")
--                                      ^ 't' from outer scope
```

**Inner variables are NOT visible outside:**
```
constraint bad:
  t: Task
  WHERE EXISTS(p: Person, assigned_to(t, p))
  => p.active = true    -- ERROR: 'p' not in scope
```

**Shadowing is forbidden:**
```
constraint bad:
  t: Task
  WHERE EXISTS(t: Project, ...)  -- ERROR: 't' already declared
```

Variable names must be unique across the entire pattern including nested EXISTS.

### Typing

EXISTS expressions have type `Bool`.

### Nesting

EXISTS can be nested:
```
EXISTS(
  p: Project,
  belongs_to(task, p)
  WHERE EXISTS(
    t: Team,
    owns(t, p)
    WHERE t.active = true
  )
)
```

Each level introduces its own scope. Outer variables remain visible at all levels.

## Layer 0

### Nodes

```
node _ExistsExpr : _Expr [sealed] {
  negated: Bool = false    -- true for NOT EXISTS
}
```

### Edges

```
edge _exists_pattern(
  expr: _ExistsExpr,
  pattern: _PatternDef
) {}
```

### Constraints

```
constraint _exists_has_pattern:
  e: _ExistsExpr
  => EXISTS(p: _PatternDef, _exists_pattern(e, p))
```

## Examples

### Constraint: Every Task Has Owner

```
constraint task_has_owner:
  t: Task
  => EXISTS(p: Person, assigned_to(t, p) AS a WHERE a.role = "owner")
```

### Constraint: No Orphan Tasks

```
constraint task_belongs_to_project:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))
```

### Constraint: No Circular Dependencies

```
constraint no_self_block:
  t: Task
  WHERE NOT EXISTS(
    upstream: Task,
    depends_on(t, upstream)
    WHERE upstream.status != "done"
  )
  => t.status != "blocked"
```

### In WHERE Clause

```
MATCH t: Task
WHERE EXISTS(p: Person, assigned_to(t, p))
RETURN t
```

## Errors

| Condition | Message |
|-----------|---------|
| Variable shadowing | `Variable 'x' already declared in outer scope` |
| Inner variable used outside | `Variable 'x' not in scope` |
| Invalid pattern in EXISTS | `Invalid pattern in EXISTS expression` |