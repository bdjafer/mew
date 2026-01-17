---
spec: exists
version: "1.0"
status: stable
category: expression
capability: expression
requires: []
priority: essential
---

## 1. Overview

Exists patterns test for the presence or absence of graph structure within constraint conditions and WHERE clauses.

**Why essential:** Most useful constraints need to express "there exists" or "there does not exist" conditions. Without this, constraints can only reason about bound variables, not about the existence of related structure.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
ExistsExpr = "EXISTS" "(" ExistsPattern ")"
           | "NOT" "EXISTS" "(" ExistsPattern ")"

ExistsPattern = PatternElement ("," PatternElement)* WhereClause?
```

Exists expressions are a new primary expression form:
```ebnf
PrimaryExpr = ... | ExistsExpr
```

### 2.2 Keywords Added

| Keyword | Context |
|---------|---------|
| `exists` | Expression |

Note: `not` is already a core keyword.

### 2.3 Examples
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

---

## 3. Semantics

### 3.1 Evaluation

`EXISTS(pattern)` evaluates to:
- `true` if at least one match for `pattern` exists
- `false` if no matches exist

`NOT EXISTS(pattern)` evaluates to:
- `true` if no matches exist
- `false` if at least one match exists

### 3.2 Variable Scoping

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

### 3.3 Typing

EXISTS expressions have type `Bool`.

### 3.4 Nesting

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

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _ExistsExpr : _Expr [sealed] {
  negated: Bool = false    -- true for NOT EXISTS
}
```

### 4.2 Edge Types
```
edge _exists_pattern(
  expr: _ExistsExpr,
  pattern: _PatternDef
) {}
```

### 4.3 Constraints
```
constraint _exists_has_pattern:
  e: _ExistsExpr
  => EXISTS(p: _PatternDef, _exists_pattern(e, p))
```

---

## 5. Compilation

### 5.1 Surface to Layer 0
```
EXISTS(p: Person, assigned_to(task, p))
```

Compiles to:
```
_ExistsExpr node:
  negated: false

_PatternDef node:
  (contains node pattern for p, edge pattern for assigned_to)

_exists_pattern edge:
  (exists_expr, pattern_def)
```

### 5.2 Variable Resolution

The compiler must:
1. Collect all variables in scope at the EXISTS expression
2. Make outer variables available for reference in the inner pattern
3. Verify no shadowing occurs
4. Verify inner variables are not referenced after the EXISTS

---

## 6. Examples

### 6.1 Constraint: Every Task Has Owner
```
constraint task_has_owner:
  t: Task
  => EXISTS(p: Person, assigned_to(t, p) AS a WHERE a.role = "owner")
```

### 6.2 Constraint: No Orphan Tasks
```
constraint task_belongs_to_project:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))
```

### 6.3 Constraint: No Circular Dependencies
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

### 6.4 In WHERE Clause
```
MATCH t: Task
WHERE EXISTS(p: Person, assigned_to(t, p))
RETURN t
```

---

*End of Feature: Exists Patterns*