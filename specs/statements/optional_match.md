---
spec: optional_match
version: "1.0"
status: draft
category: statement
capability: query
requires: [match, pattern]
priority: common
---

# Spec: OPTIONAL MATCH

## Overview

OPTIONAL MATCH attempts to match a pattern, but returns NULL for unmatched variables instead of filtering out the row. This enables left-outer-join semantics in graph queries, allowing retrieval of entities even when optional relationships do not exist.

## Syntax

### Grammar

```ebnf
MatchStmt =
  "match" Pattern
  OptionalMatchClause*
  WhereClause?
  ReturnClause
  OrderClause?
  LimitClause?

OptionalMatchClause = "optional" "match" Pattern ("where" Expr)?
```

### Keywords

| Keyword | Context |
|---------|---------|
| `optional` | Modifier - makes the following MATCH optional |
| `match` | Statement - pattern matching (required after `optional`) |
| `where` | Clause - filters the optional pattern (not the entire result) |

### Examples

```
-- Get all tasks, with assignee if exists (NULL if unassigned)
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
RETURN t.title, p.name

-- Multiple optional matches
MATCH t: Task
OPTIONAL MATCH assigned_to(t, assignee)
OPTIONAL MATCH belongs_to(t, project)
RETURN t.title, assignee.name, project.name

-- Optional match with filter
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p) WHERE p.priority_level > 5
RETURN t.title, p.name

-- Entity with optional metadata
MATCH u: User
OPTIONAL MATCH has_profile(u, profile)
OPTIONAL MATCH has_avatar(u, avatar)
RETURN u.name, profile.bio, avatar.url
```

## Semantics

### Basic Behavior

OPTIONAL MATCH extends the matched bindings with additional optional patterns:

1. The initial MATCH pattern is evaluated normally
2. For each result row, OPTIONAL MATCH attempts to extend with the optional pattern
3. If the optional pattern matches, variables are bound to matched values
4. If the optional pattern does not match, variables are bound to NULL
5. The original row is never filtered out due to OPTIONAL MATCH failure

### Comparison with Regular MATCH

**With OPTIONAL MATCH:**
```
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
RETURN t.title, p.name
```

| t.title | p.name |
|---------|--------|
| "Task A" | "Alice" |
| "Task B" | NULL |
| "Task C" | "Bob" |

**Without OPTIONAL MATCH (inner join):**
```
MATCH t: Task, assigned_to(t, p)
RETURN t.title, p.name
```

| t.title | p.name |
|---------|--------|
| "Task A" | "Alice" |
| "Task C" | "Bob" |

### WHERE on OPTIONAL MATCH

When WHERE is attached to OPTIONAL MATCH, it filters the optional pattern, not the entire result:

```
-- Get tasks with their high-priority assignees (if any)
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p) WHERE p.priority_level > 5
RETURN t.title, p.name
```

- If a task has an assignee with `priority_level <= 5`, `p.name` is NULL (pattern didn't match the filter)
- If a task has an assignee with `priority_level > 5`, `p.name` is the assignee's name
- If a task has no assignee, `p.name` is NULL
- The task row is always returned regardless

### Multiple OPTIONAL MATCH Clauses

Multiple OPTIONAL MATCH clauses are independent of each other:

```
MATCH t: Task
OPTIONAL MATCH assigned_to(t, assignee)
OPTIONAL MATCH belongs_to(t, project)
RETURN t.title, assignee.name, project.name
```

- `assignee` can be NULL while `project` is bound (and vice versa)
- Each OPTIONAL MATCH is evaluated independently against the base pattern
- Order of OPTIONAL MATCH clauses does not affect results

### Aggregations with OPTIONAL MATCH

OPTIONAL MATCH works naturally with aggregations:

```
MATCH p: Project
OPTIONAL MATCH t: Task, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
```

- Projects with no tasks return `task_count = 0`
- NULL values are handled correctly by aggregate functions

### Variable Scoping

Variables from OPTIONAL MATCH are available in subsequent clauses:

```
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
WHERE p.name IS NOT NULL  -- filters to only assigned tasks after the fact
RETURN t, p
```

Note: The WHERE after OPTIONAL MATCH filters the final result, while WHERE attached to OPTIONAL MATCH filters the optional pattern.

## Layer 0

None.

## Examples

```
-- Left outer join equivalent
MATCH t: Task
OPTIONAL MATCH belongs_to(t, p)
RETURN t, p

-- Get entity with all optional metadata
MATCH u: User
OPTIONAL MATCH has_profile(u, profile)
OPTIONAL MATCH has_avatar(u, avatar)
OPTIONAL MATCH has_preferences(u, prefs)
RETURN u.name, profile.bio, avatar.url, prefs.theme

-- Aggregate with optional relationships
MATCH p: Project
OPTIONAL MATCH t: Task, belongs_to(t, p)
RETURN p.name, COUNT(t) AS task_count
-- Projects with 0 tasks return task_count = 0

-- Optional with edge binding
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p) AS assignment
RETURN t.title, p.name, assignment.assigned_at

-- Filter on optional result (post-filter)
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
WHERE p IS NOT NULL
RETURN t.title, p.name
-- Only returns tasks that have assignees

-- Conditional logic on optional
MATCH t: Task
OPTIONAL MATCH assigned_to(t, p)
RETURN t.title,
       CASE WHEN p IS NULL THEN "Unassigned" ELSE p.name END AS assignee

-- Complex query with multiple optionals and ordering
MATCH t: Task
OPTIONAL MATCH assigned_to(t, assignee)
OPTIONAL MATCH belongs_to(t, project)
OPTIONAL MATCH depends_on(t, blocker)
WHERE t.status != "done"
RETURN t.title,
       assignee.name AS assigned_to,
       project.name AS project,
       COUNT(blocker) AS blocking_count
ORDER BY blocking_count DESC
LIMIT 20
```

## Errors

| Condition | Message |
|-----------|---------|
| OPTIONAL without MATCH | Expected `match` after `optional` |
| OPTIONAL MATCH as first clause | OPTIONAL MATCH must follow a regular MATCH |
| Invalid pattern in OPTIONAL MATCH | Expected pattern after OPTIONAL MATCH |
