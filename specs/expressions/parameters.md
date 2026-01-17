---
spec: parameters
version: "1.0"
status: draft
category: expression
capability: parameterization
requires: [literals, identifiers]
priority: essential
---

# Spec: Parameters

## Overview

Parameters enable safe, reusable queries with external values. They prevent injection attacks by ensuring values are always treated as data (never as identifiers or operators), and enable query plan caching for improved performance. Parameters are referenced using the `$` prefix followed by an identifier.

## Syntax

### Grammar

```ebnf
Parameter       = "$" Identifier
OptionalParam   = "$" Identifier "?"

Identifier      = Letter (Letter | Digit | "_")*
```

### Keywords

| Keyword | Context |
|---------|---------|
| `$` | Parameter prefix in expressions |
| `?` | Optional parameter suffix |

### Examples

```
-- In WHERE clauses
MATCH t: Task WHERE t.id = $taskId RETURN t

-- In SPAWN statements
SPAWN t: Task { title = $title, priority = $priority }

-- In LINK statements
LINK assigned_to(#task_123, $personId)

-- Optional parameters with COALESCE
MATCH t: Task WHERE t.status = COALESCE($status, "pending") RETURN t
```

## Semantics

### Parameter Binding

Parameters are bound when executing the query:

```typescript
// API usage (pseudocode)
engine.execute(
  "MATCH t: Task WHERE t.status = $status RETURN t",
  { status: "done" }
)

engine.execute(
  "SPAWN t: Task { title = $title, priority = $priority } RETURNING t.id",
  { title: "New task", priority: 5 }
)
```

### Type Checking

Parameters are type-checked against their usage context:

```
-- $priority inferred as Int from Task.priority
MATCH t: Task WHERE t.priority > $priority RETURN t

-- $name inferred as String from Person.name
SPAWN p: Person { name = $name }

-- Type error if wrong type provided at execution:
-- ERROR: Parameter $priority expected Int, got String
```

### Parameter Scope

Parameters have the following characteristics:

- **Immutable:** Cannot be modified within a query
- **Query-scoped:** Available throughout the entire query
- **Not nullable by default:** Use `$param?` for optional parameters

### Optional Parameters

```
-- Optional parameter with default via COALESCE
MATCH t: Task
WHERE t.status = COALESCE($status, "pending")
RETURN t

-- Explicit optional syntax
MATCH t: Task
WHERE $filter? = null OR t.category = $filter
RETURN t
```

### Parameters in Bulk Operations

```
KILL { MATCH t: Task WHERE t.project_id = $projectId RETURN t }

SET { MATCH t: Task WHERE t.assignee = $oldAssignee RETURN t }.assignee = $newAssignee
```

### Security

Parameters are **always** treated as values, never as identifiers or operators:

```
-- SAFE: $value is always a value
MATCH t: Task WHERE t.title = $value RETURN t
-- Even if $value contains "'; DROP TABLE tasks; --"

-- NOT POSSIBLE: Cannot parameterize type names or attributes
MATCH t: $type RETURN t        -- ERROR: Type cannot be parameterized
MATCH t: Task RETURN t.$attr   -- ERROR: Attribute cannot be parameterized
```

## Layer 0

### Nodes

None.

### Edges

None.

### Constraints

None.

Parameters are a syntactic feature resolved at query compilation time, not stored in the graph.

## Examples

### Basic Query with Parameter

```
-- Find tasks by status
MATCH t: Task WHERE t.status = $status RETURN t

-- Execution:
engine.execute(query, { status: "in_progress" })
```

### Multiple Parameters

```
-- Find tasks within priority range
MATCH t: Task
WHERE t.priority >= $minPriority AND t.priority <= $maxPriority
RETURN t

-- Execution:
engine.execute(query, { minPriority: 5, maxPriority: 10 })
```

### Parameters in Mutations

```
-- Create task with parameterized attributes
SPAWN t: Task {
  title = $title,
  priority = $priority,
  created_at = now()
}
RETURNING id

-- Execution:
engine.execute(query, { title: "Review PR", priority: 8 })
```

### Optional Parameter Pattern

```
-- Search with optional filters
MATCH t: Task
WHERE ($status? = null OR t.status = $status)
  AND ($minPriority? = null OR t.priority >= $minPriority)
RETURN t

-- Execution with partial filters:
engine.execute(query, { status: "done" })  -- minPriority not provided
```

### Bulk Update with Parameters

```
-- Reassign tasks from one person to another
SET { MATCH t: Task WHERE t.assignee = $from RETURN t }.assignee = $to

-- Execution:
engine.execute(query, { from: "user_old", to: "user_new" })
```

## Errors

| Condition | Message |
|-----------|---------|
| Missing required parameter | `Parameter '$name' not provided` |
| Type mismatch | `Parameter $name expected Type, got ActualType` |
| Parameterized identifier | `Type cannot be parameterized` |
| Parameterized attribute | `Attribute cannot be parameterized` |
