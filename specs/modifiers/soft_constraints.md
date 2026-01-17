---
spec: soft_constraints
version: "1.0"
status: draft
category: modifier
requires: []
priotity: convenience
---

# Spec: Soft Constraints

## Overview

Soft constraints warn instead of reject. They document best practices, data quality expectations, or guidelines that shouldn't block operations. Hard constraints are requirements; soft constraints are recommendations.

---

## Syntax

### Grammar
```ebnf
ConstraintDecl = 
  DocComment?
  "constraint" Identifier ConstraintModifiers? ":" Pattern "=>" Expr

ConstraintModifiers = "[" ConstraintModifier ("," ConstraintModifier)* "]"

ConstraintModifier = 
    "soft"
  | "message:" StringLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `soft` | Constraint modifier |
| `message` | Constraint modifier |

### Examples
```
-- Soft constraint: warns but allows
constraint prefer_description [soft]:
  t: Task
  => t.description != null

-- With custom message
constraint prefer_due_date [soft, message: "Tasks should have due dates"]:
  t: Task
  => t.due_date != null

-- Message on hard constraint (for better error messages)
constraint temporal_order [message: "Cause must precede effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp < e2.timestamp
```

---

## Semantics

### Hard vs Soft

| Modifier | On Violation |
|----------|--------------|
| (default) | Reject transaction |
| `[soft]` | Log warning, allow transaction |

### Evaluation Order

Both hard and soft constraints are evaluated at commit:
1. Evaluate all affected constraints
2. If any hard constraint fails → reject transaction
3. If any soft constraint fails → log warning
4. If all hard constraints pass → commit

Soft constraint failures do not prevent commit.

### Message Modifier

The `message` modifier provides a custom message for errors (hard) or warnings (soft):
```
constraint unique_email [message: "Email already in use"]:
  p1: Person, p2: Person WHERE p1.id != p2.id
  => p1.email != p2.email

-- Error: "Constraint 'unique_email' violated: Email already in use"
```

Without `message`, the default is:
```
-- Error: "Constraint 'unique_email' violated"
```

### Combining Modifiers
```
[soft, message: "Consider adding a description"]
```

Order doesn't matter.

### Introspection

Soft constraints are observable like hard constraints:
```
MATCH c: _ConstraintDef WHERE c.hard = false
RETURN c.name
-- Returns names of all soft constraints
```

---

## Layer 0

### Nodes

No new node types. Uses existing `_ConstraintDef`:
```
node _ConstraintDef [sealed] {
  name: String [required],
  hard: Bool = true,        -- false for soft constraints
  message: String?,         -- custom message
  doc: String?
}
```

Note: `hard` and `message` already exist in Layer 0 Core.

### Edges

None.

### Constraints

None.

---

## Compilation
```
constraint prefer_description [soft, message: "Tasks should have descriptions"]:
  t: Task
  => t.description != null
```

Compiles to:
```
_ConstraintDef node:
  name: "prefer_description"
  hard: false
  message: "Tasks should have descriptions"
  doc: null

_constraint_has_pattern edge: ...
_constraint_has_condition edge: ...
```

---

## Examples

### Data Quality Guidelines
```
ontology TaskManagement {
  -- Hard: required for correctness
  constraint task_has_title:
    t: Task
    => t.title != null

  -- Soft: recommended for quality
  constraint prefer_description [soft, message: "Add description for clarity"]:
    t: Task
    => t.description != null

  constraint prefer_due_date [soft, message: "Tasks should have due dates"]:
    t: Task WHERE t.status != "done"
    => t.due_date != null

  constraint prefer_assignee [soft, message: "Assign tasks to track ownership"]:
    t: Task WHERE t.status = "in_progress"
    => EXISTS(assigned_to(t, _))
}
```

### Migration Warnings
```
-- During migration, warn about old data patterns
constraint deprecated_status [soft, message: "Status 'pending' is deprecated, use 'todo'"]:
  t: Task WHERE t.status = "pending"
  => false
```

### Informative Hard Constraints
```
-- Better error messages with [message:]
constraint no_self_assignment [message: "Cannot assign task to its creator"]:
  t: Task, p: Person,
  created_by(t, p),
  assigned_to(t, p)
  => false
-- Error: "Constraint 'no_self_assignment' violated: Cannot assign task to its creator"
```

### Warning Aggregation

Multiple soft constraint violations are collected:
```
BEGIN
  SPAWN t: Task { title = "Quick task" }
  -- No description, no due_date, no assignee
COMMIT
-- Warnings:
--   prefer_description: "Add description for clarity"
--   prefer_due_date: "Tasks should have due dates"
-- Transaction succeeds despite warnings
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Hard constraint violation | `"Constraint 'X' violated: <message or default>"` |

(Soft constraint violations produce warnings, not errors)

---

*End of Spec: Soft Constraints*