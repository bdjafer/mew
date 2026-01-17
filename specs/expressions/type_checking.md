---
spec: type_checking
version: "1.0"
status: stable
category: expression
requires: []
priority: common
---

# Spec: Type Checking Expression

## Overview

The type check expression `x:Type` tests whether a value is of a specific type at runtime. Returns `true` if the value's type equals or inherits from the specified type.

**Why needed:** When a variable has union type or `any` type, code may need to branch on the actual runtime type before accessing type-specific attributes.

---

## Syntax

### Grammar
```ebnf
PostfixExpr = PrimaryExpr (("." Identifier) | (":" Identifier))*
```

### Keywords

None. Uses existing `:` operator in postfix position.

### Examples
```
x:Person              -- true if x is Person or subtype
entity:Task           -- true if entity is Task
node:Event            -- true if node is Event or subtype
```

---

## Semantics

### Evaluation

`expr:TypeName` evaluates to `Bool`:
- `true` if expr's runtime type equals TypeName or inherits from TypeName
- `false` otherwise

### Inheritance

Type checking respects inheritance:
```
abstract node Entity { }
node Person : Entity { }
node Employee : Person { }

-- Given e: Employee
e:Employee  -- true
e:Person    -- true (Employee inherits Person)
e:Entity    -- true (Employee inherits Entity)
e:Task      -- false (unrelated type)
```

### Null Handling

Type check on null returns false:
```
x: Person? = null
x:Person    -- false
```

### Non-Node Values

Type checking on non-node values (scalars) returns false:
```
name: String = "Alice"
name:Person   -- false (String is not a node type)
```

### Union Types

Useful for narrowing union types:
```
x: Task | Project

-- In WHERE clause:
WHERE x:Task AND x.priority > 5
-- x.priority is valid because we've checked x:Task
```

### Precedence

`:` for type checking has same precedence as `.` (highest):
```
x:Person AND y:Task
-- Parses as: (x:Person) AND (y:Task)

x.manager:Employee
-- Parses as: (x.manager):Employee
```

---

## Layer 0

### Nodes
```
node _TypeCheckExpr : _Expr [sealed] {
  type_name: String [required]   -- Name of type to check against
}
```

### Edges
```
edge _type_check_operand(
  expr: _TypeCheckExpr,
  operand: _Expr
) {}
```

### Constraints
```
constraint _type_check_has_operand:
  e: _TypeCheckExpr
  => EXISTS(op: _Expr, _type_check_operand(e, op))
```

---

## Compilation
```
x:Person
```

Compiles to:
```
_TypeCheckExpr node:
  type_name: "Person"

_VarRefExpr node:
  var_name: "x"

_type_check_operand edge:
  (type_check_expr, var_ref_expr)
```

---

## Examples

### Filtering by Type
```
edge tagged(entity: any, tag: Tag)

MATCH tagged(e, t)
WHERE e:Task
RETURN e, t
-- Only tagged Tasks
```

### Conditional Constraints
```
constraint employee_needs_department:
  p: Person WHERE p:Employee
  => EXISTS(d: Department, works_in(p, d))
-- Only applies to Employees, not all Persons
```

### Union Type Handling
```
type Assignable = Task | Issue | Story

edge assigned_to(item: Assignable, person: Person)

constraint task_assignment_limit:
  p: Person, a: Assignable,
  assigned_to(a, p)
  WHERE a:Task
  => count_assignments(p, "Task") <= 10
-- Limit only applies to Tasks
```

### Chained Access
```
MATCH e: Employee
WHERE e.manager:Director
RETURN e
-- Employees whose manager is a Director
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Unknown type | `"Unknown type 'X'"` |
| Type check on edge variable | `"Type check not supported on edge variables"` |

---

*End of Spec: Type Checking Expression*