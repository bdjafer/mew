---
spec: negative_patterns
version: "1.0"
status: stable
category: pattern
capability: absence_testing
requires: [node_patterns, edge_patterns, exists]
priority: common
---

# Spec: Negative Patterns

## Overview

Negative patterns test for the absence of graph structure using `NOT EXISTS`. They enable queries and constraints that reason about what does not exist: unassigned tasks, orphan nodes, missing relationships. Without negative patterns, queries can only express what must be present, not what must be absent. This is essential for data quality constraints, cleanup queries, and finding gaps in data.

## Syntax

### Grammar

```ebnf
NegativePattern   = "NOT" "EXISTS" "(" ExistsPattern ")"

ExistsPattern     = PatternElement ("," PatternElement)* WhereClause?

WhereClause       = "WHERE" Expr

PatternElement    = NodePattern | EdgePattern

-- In WHERE clause context
WhereExpr         = ... | NegativePattern | ExistsExpr

-- Full negation with comparison
ConditionExpr     = NegativePattern
                  | "NOT" "(" Expr ")"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `NOT` | Modifier - negates EXISTS expression |
| `EXISTS` | Expression - tests for pattern presence |
| `WHERE` | Clause - filters within EXISTS pattern |

### Examples

```
-- Tasks without any assignment
NOT EXISTS(assigned_to(t, _))

-- Persons not in any team
NOT EXISTS(team: Team, member_of(p, team))

-- Tasks without active assignees
NOT EXISTS(
  person: Person, assigned_to(t, person)
  WHERE person.active = true
)

-- No blocking dependencies
NOT EXISTS(
  upstream: Task, depends_on(t, upstream)
  WHERE upstream.status != "done"
)
```

## Semantics

### Evaluation

`NOT EXISTS(pattern)` evaluates to:
- `true` if **no matches** for `pattern` exist
- `false` if **at least one match** exists

This is the logical negation of `EXISTS(pattern)`:
```
NOT EXISTS(P) === NOT (EXISTS(P))
```

### Variable Scoping

**Outer variables are visible inside NOT EXISTS:**
```
MATCH t: Task
WHERE NOT EXISTS(p: Person, assigned_to(t, p))
--                                       ^ 't' from outer scope
RETURN t
```

**Inner variables are NOT visible outside:**
```
MATCH t: Task
WHERE NOT EXISTS(p: Person, assigned_to(t, p))
RETURN p.name    -- ERROR: 'p' not in scope
```

**Reasoning:** The negative pattern asserts non-existence; there is no `p` to reference.

### Nested WHERE Clauses

The optional WHERE within NOT EXISTS further constrains what "doesn't exist":

```
-- Tasks not assigned to ACTIVE persons
MATCH t: Task
WHERE NOT EXISTS(
  p: Person, assigned_to(t, p)
  WHERE p.active = true
)
RETURN t
```

This returns tasks that:
- Have no assignees at all, OR
- Have only inactive assignees

**Contrast with:**
```
-- Tasks with non-active assignees (different meaning!)
MATCH t: Task, p: Person, assigned_to(t, p)
WHERE p.active = false
RETURN t
```

### Correlation with Outer Query

NOT EXISTS typically correlates with outer pattern variables:

```
MATCH proj: Project
WHERE NOT EXISTS(t: Task, belongs_to(t, proj))
RETURN proj.name
-- Projects with no tasks
```

The inner pattern `belongs_to(t, proj)` references `proj` from the outer MATCH.

### Multiple Negations

Patterns can include multiple NOT EXISTS:

```
MATCH t: Task
WHERE NOT EXISTS(p: Person, assigned_to(t, p))
  AND NOT EXISTS(proj: Project, belongs_to(t, proj))
RETURN t
-- Orphan tasks: no assignee AND no project
```

### Negation of Complex Patterns

NOT EXISTS can negate multi-element patterns:

```
MATCH t: Task
WHERE NOT EXISTS(
  p: Person, proj: Project,
  assigned_to(t, p),
  belongs_to(t, proj),
  owns(p, proj)
)
RETURN t
-- Tasks not assigned to someone who owns the project
```

All pattern elements must match for EXISTS to be true; if any element fails, NOT EXISTS returns true.

### De Morgan's Laws

Logical equivalences apply:

```
-- These are equivalent:
NOT EXISTS(A) AND NOT EXISTS(B)
NOT (EXISTS(A) OR EXISTS(B))

-- These are NOT equivalent:
NOT EXISTS(A, B)        -- neither A nor B
NOT EXISTS(A) AND NOT EXISTS(B)  -- not A and not B (same)
NOT (EXISTS(A) AND EXISTS(B))    -- not both A and B
```

### Performance Considerations

NOT EXISTS patterns use anti-join semantics:
- Efficient when correlated variable has index
- Can be expensive with uncorrelated patterns
- Inner WHERE may enable early termination

## Layer 0

### Nodes

```
node _NotExistsExpr : _Expr [sealed] {
  -- inherits from _Expr
  -- always represents negated existence
}

node _ExistsExpr : _Expr [sealed] {
  negated: Bool = false    -- true for NOT EXISTS
}
```

### Edges

```
edge _exists_pattern(
  expr: _ExistsExpr | _NotExistsExpr,
  pattern: _PatternDef
) {}

edge _exists_where(
  expr: _ExistsExpr | _NotExistsExpr,
  condition: _Expr
) {}
```

### Constraints

```
constraint _not_exists_has_pattern:
  ne: _NotExistsExpr
  => EXISTS(p: _PatternDef, _exists_pattern(ne, p))
```

## Examples

### Find Unassigned Tasks

```
-- Tasks without any assignment
MATCH t: Task
WHERE NOT EXISTS(assigned_to(t, _))
RETURN t.title, t.created_at
ORDER BY t.created_at DESC
```

### Find Orphan Nodes

```
-- Persons not in any team
MATCH p: Person
WHERE NOT EXISTS(t: Team, member_of(p, t))
RETURN p.name, p.email
```

### Find Tasks Without Active Dependencies

```
-- Tasks with no blocking (non-done) dependencies
MATCH t: Task
WHERE NOT EXISTS(
  upstream: Task, depends_on(t, upstream)
  WHERE upstream.status != "done"
)
RETURN t.title AS ready_task
```

### Constraint: Every Task Must Have Owner

```
constraint task_has_owner:
  t: Task
  WHERE NOT EXISTS(
    p: Person, assigned_to(t, p) AS a
    WHERE a.role = "owner"
  )
  => false   -- violation if no owner exists
```

Or equivalently using positive EXISTS:
```
constraint task_has_owner:
  t: Task
  => EXISTS(
    p: Person, assigned_to(t, p) AS a
    WHERE a.role = "owner"
  )
```

### Find Leaf Nodes

```
-- Tasks that nothing depends on (leaf tasks)
MATCH t: Task
WHERE NOT EXISTS(
  other: Task, depends_on(other, t)
)
RETURN t.title
```

### Find Root Nodes

```
-- Tasks that don't depend on anything (root tasks)
MATCH t: Task
WHERE NOT EXISTS(depends_on(t, _))
RETURN t.title
```

### Complex Negation in Business Logic

```
-- Find projects that have no tasks assigned to any admin
MATCH proj: Project
WHERE NOT EXISTS(
  t: Task, p: Person,
  belongs_to(t, proj),
  assigned_to(t, p)
  WHERE p.role = "admin"
)
RETURN proj.name
```

### Multiple NOT EXISTS Conditions

```
-- Find tasks needing attention:
-- No assignee AND no comments AND no updates in 7 days
MATCH t: Task
WHERE NOT EXISTS(assigned_to(t, _))
  AND NOT EXISTS(c: Comment, comment_on(c, t))
  AND t.updated_at < now() - 7d
RETURN t.title, t.created_at
```

### NOT EXISTS with Aggregation in Outer Query

```
-- Count unassigned tasks per project
MATCH t: Task, proj: Project, belongs_to(t, proj)
WHERE NOT EXISTS(assigned_to(t, _))
RETURN proj.name, COUNT(t) AS unassigned_count
ORDER BY unassigned_count DESC
```

### Negation in Rules

```
-- Auto-assign orphan tasks to default pool
rule auto_assign_orphan_tasks:
  ON SPAWN t: Task
  WHERE NOT EXISTS(assigned_to(t, _))
  DO LINK assigned_to(t, $default_assignee)
```

### Finding Gaps in Relationships

```
-- Teams without any projects
MATCH team: Team
WHERE NOT EXISTS(proj: Project, owns(team, proj))
RETURN team.name AS idle_team

-- People who follow but aren't followed back
MATCH p: Person, other: Person, follows(p, other)
WHERE NOT EXISTS(follows(other, p))
RETURN p.name AS follower, other.name AS not_following_back
```

### Constraint: No Circular Assignment

```
-- A person cannot be assigned to a task they created
constraint no_self_assignment:
  t: Task, p: Person,
  created_by(t, p),
  assigned_to(t, p)
  => false

-- Alternative using NOT EXISTS in rule form
constraint no_self_assignment:
  t: Task, p: Person, created_by(t, p)
  => NOT EXISTS(assigned_to(t, p))
```

## Errors

| Condition | Message |
|-----------|---------|
| Variable shadowing | `Variable 'x' already declared in outer scope` |
| Inner variable used outside | `Variable 'x' not in scope (declared in NOT EXISTS)` |
| Invalid pattern in NOT EXISTS | `Invalid pattern in NOT EXISTS expression` |
| Empty pattern | `NOT EXISTS requires at least one pattern element` |
| Mismatched parentheses | `Unmatched '(' in NOT EXISTS expression` |
| Invalid WHERE in NOT EXISTS | `Invalid expression in NOT EXISTS WHERE clause` |
| NOT without EXISTS | `NOT keyword requires EXISTS, found 'X'` |
| Double negation warning | `Double negation 'NOT NOT EXISTS' simplifies to 'EXISTS'` |
