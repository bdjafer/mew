---
spec: null_handling
version: "1.0"
status: stable
category: expression
capability: null_handling
requires: [literals, types]
priority: essential
---

# Spec: Null Handling

## Overview

Null handling provides expressions for working with optional values (`T?` types). Optional types require ergonomic null handling - every optional attribute access needs a way to provide defaults or check for null.

## Syntax

### Grammar

```ebnf
CoalesceExpr     = OrExpr ("??" OrExpr)*

CoalesceCallExpr = "COALESCE" "(" Expr ("," Expr)+ ")"

NullCheckExpr    = Expr "IS" "NULL"
                 | Expr "IS" "NOT" "NULL"

Expr             = CoalesceExpr

PrimaryExpr      = ... | CoalesceCallExpr | NullCheckExpr
```

### Keywords

| Keyword | Context |
|---------|---------|
| `COALESCE` | Expression - function call |
| `IS` | Expression - null check (core keyword) |
| `NULL` | Expression - null literal (core keyword) |
| `NOT` | Modifier - negation (core keyword) |

### Operators

| Operator | Precedence | Associativity | Description |
|----------|------------|---------------|-------------|
| `??` | 9 (below `OR`) | Right | Null coalesce |

### Examples

```
-- Coalesce operator
t.nickname ?? t.name ?? "Anonymous"

-- Coalesce function
COALESCE(t.nickname, t.name, "Anonymous")

-- Null check
t.deleted_at IS NULL
t.email IS NOT NULL

-- Combined
(t.nickname ?? t.name) IS NOT NULL
```

## Semantics

### Coalesce Operator (??)

`a ?? b` evaluates to:
- `a` if `a` is not null
- `b` if `a` is null

Right-associative: `a ?? b ?? c` = `a ?? (b ?? c)`

Short-circuit: `b` is not evaluated if `a` is not null.

### COALESCE Function

`COALESCE(a, b, c, ...)` returns the first non-null argument.

- If all arguments are null, returns null
- Evaluates arguments left-to-right, stops at first non-null

### IS NULL / IS NOT NULL

`x IS NULL` evaluates to:
- `true` if `x` is null
- `false` otherwise

`x IS NOT NULL` evaluates to:
- `true` if `x` is not null
- `false` otherwise

### Typing

**Coalesce:**
- All arguments must have compatible types
- If last argument is non-null type `T`, result is `T`
- If all arguments are nullable, result is nullable

```
-- t.nickname: String?, t.name: String?, "default": String
t.nickname ?? t.name ?? "default"   -- Type: String (last is non-null)

t.nickname ?? t.alt_name            -- Type: String? (both nullable)
```

**Null check:**
- Left side: any type
- Result: `Bool`

## Layer 0

### Nodes

```
node _CoalesceExpr : _Expr [sealed] {
  -- Arguments linked via _coalesce_arg edges
}

node _NullCheckExpr : _Expr [sealed] {
  negated: Bool = false    -- true for IS NOT NULL
}
```

### Edges

```
edge _coalesce_arg(
  expr: _CoalesceExpr,
  arg: _Expr
) {
  position: Int [required]    -- 0-indexed
}

edge _null_check_operand(
  expr: _NullCheckExpr,
  operand: _Expr
) {}
```

### Constraints

```
constraint _coalesce_has_args:
  e: _CoalesceExpr
  => EXISTS(a: _Expr, _coalesce_arg(e, a) AS c WHERE c.position = 0)
     AND EXISTS(a: _Expr, _coalesce_arg(e, a) AS c WHERE c.position = 1)

constraint _null_check_has_operand:
  e: _NullCheckExpr
  => EXISTS(o: _Expr, _null_check_operand(e, o))
```

## Examples

### Default Values in Queries

```
MATCH p: Person
RETURN p.name, p.nickname ?? "(no nickname)"
```

### Constraint with Null Check

```
constraint active_has_email:
  p: Person WHERE p.status = "active"
  => p.email IS NOT NULL
```

### Coalesce Chain

```
MATCH t: Task
RETURN COALESCE(t.assigned_to_name, t.created_by_name, "Unassigned")
```

### Filtering Nulls

```
MATCH t: Task
WHERE t.completed_at IS NOT NULL
RETURN t
```

## Errors

| Condition | Message |
|-----------|---------|
| Incompatible types in coalesce | `Coalesce arguments must have compatible types` |
| Single argument to COALESCE | `COALESCE requires at least 2 arguments` |