# Feature: Null Handling

**Version:** 1.0
**Status:** Essential
**Requires:** Core (Parts I, II, III)

---

## 1. Overview

Null handling provides expressions for working with optional values (`T?` types).

**Why essential:** Optional types are useless without ergonomic null handling. Every optional attribute access needs a way to provide defaults or check for null.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
CoalesceExpr = OrExpr ("??" OrExpr)*

CoalesceCallExpr = "COALESCE" "(" Expr ("," Expr)+ ")"

NullCheckExpr = Expr "IS" "NULL"
              | Expr "IS" "NOT" "NULL"
```

The `??` operator is the lowest precedence binary operator:
```ebnf
Expr = CoalesceExpr
CoalesceExpr = OrExpr ("??" OrExpr)*
```

Null check and COALESCE function are primary expressions:
```ebnf
PrimaryExpr = ... | CoalesceCallExpr | NullCheckExpr
```

### 2.2 Keywords Added

| Keyword | Context |
|---------|---------|
| `coalesce` | Function call |

Note: `is`, `not`, `null` are already core keywords.

### 2.3 Operator Added

| Operator | Precedence | Associativity | Description |
|----------|------------|---------------|-------------|
| `??` | 9 (below `or`) | Right | Null coalesce |

### 2.4 Examples
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

---

## 3. Semantics

### 3.1 Coalesce Operator (??)

`a ?? b` evaluates to:
- `a` if `a` is not null
- `b` if `a` is null

Right-associative: `a ?? b ?? c` = `a ?? (b ?? c)`

Short-circuit: `b` is not evaluated if `a` is not null.

### 3.2 COALESCE Function

`COALESCE(a, b, c, ...)` returns the first non-null argument.

- If all arguments are null, returns null
- Evaluates arguments left-to-right, stops at first non-null

### 3.3 IS NULL / IS NOT NULL

`x IS NULL` evaluates to:
- `true` if `x` is null
- `false` otherwise

`x IS NOT NULL` evaluates to:
- `true` if `x` is not null
- `false` otherwise

### 3.4 Typing

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

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _CoalesceExpr : _Expr [sealed] {
  -- Arguments linked via _coalesce_arg edges
}

node _NullCheckExpr : _Expr [sealed] {
  negated: Bool = false    -- true for IS NOT NULL
}
```

### 4.2 Edge Types
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

### 4.3 Constraints
```
constraint _coalesce_has_args:
  e: _CoalesceExpr
  => EXISTS(a: _Expr, _coalesce_arg(e, a) AS c WHERE c.position = 0)
     AND EXISTS(a: _Expr, _coalesce_arg(e, a) AS c WHERE c.position = 1)

constraint _null_check_has_operand:
  e: _NullCheckExpr
  => EXISTS(o: _Expr, _null_check_operand(e, o))
```

---

## 5. Compilation

### 5.1 Coalesce Operator
```
a ?? b ?? c
```

Compiles to:
```
_CoalesceExpr node

_coalesce_arg edges:
  (expr, a) { position: 0 }
  (expr, b) { position: 1 }
  (expr, c) { position: 2 }
```

### 5.2 COALESCE Function

Identical compilation to `??` operator.

### 5.3 Null Check
```
x IS NOT NULL
```

Compiles to:
```
_NullCheckExpr node:
  negated: true

_null_check_operand edge:
  (expr, x_expr)
```

---

## 6. Examples

### 6.1 Default Values in Queries
```
MATCH p: Person
RETURN p.name, p.nickname ?? "(no nickname)"
```

### 6.2 Constraint with Null Check
```
constraint active_has_email:
  p: Person WHERE p.status = "active"
  => p.email IS NOT NULL
```

### 6.3 Coalesce Chain
```
MATCH t: Task
RETURN COALESCE(t.assigned_to_name, t.created_by_name, "Unassigned")
```

### 6.4 Filtering Nulls
```
MATCH t: Task
WHERE t.completed_at IS NOT NULL
RETURN t
```

---

*End of Feature: Null Handling*