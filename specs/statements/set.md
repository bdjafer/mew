---
spec: set
version: "1.0"
status: draft
category: statement
capability: mutation
requires: [id_ref, match_stmt, attr_block, expr, returning_clause]
priority: essential
---

# Spec: SET

## Overview

SET modifies attribute values on existing nodes or edges. It supports single attribute updates, block syntax for multiple attributes, computed values using expressions, and bulk updates via pattern matching.

## Syntax

### Grammar

```ebnf
SetStmt         = "set" Target ("." Identifier "=" Expr | AttrBlock) ReturningClause? ;

Target          = IdRef
                | "{" MatchStmt "}" ;

AttrBlock       = "{" (AttrAssignment ("," AttrAssignment)*)? "}" ;

AttrAssignment  = Identifier "=" Expr ;

ReturningClause = "returning" ("id" | "*" | Identifier ("," Identifier)*) ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `set` | Statement - initiates attribute modification |
| `returning` | Clause - specifies what to return after update |

### Examples

```
-- Set single attribute
SET #task_123.status = "done"

-- Set multiple attributes (block syntax)
SET #task_123 {
  status = "done",
  completed_at = now()
}

-- Set with expression
SET #task_123.priority = #task_123.priority + 1

-- Bulk set via pattern
SET { MATCH t: Task WHERE t.status = "in_progress" RETURN t }.status = "blocked"
```

## Semantics

### Single Attribute Update

```
SET #task_123.status = "done"
```

Updates one attribute on the specified node or edge.

### Multiple Attributes (Block Syntax)

Set multiple attributes in a single statement:

```
SET #task_123 {
  status = "done",
  completed_at = now(),
  completed_by = "user_456"
}
```

Block syntax is preferred as it's more concise and clearly shows all changes together.

### Alternative: Separate Statements

Multiple SET statements can be used within a transaction:

```
BEGIN
  SET #task_123.status = "done"
  SET #task_123.completed_at = now()
  SET #task_123.completed_by = "user_456"
COMMIT
```

### Target Resolution

**By ID:**
```
SET #task_123.status = "done"
```
Direct reference to a specific node or edge.

**By Pattern (Bulk):**
```
SET { MATCH t: Task WHERE t.status = "pending" RETURN t }.status = "archived"
```
Updates all matching entities.

**Bulk with Block Syntax:**
```
SET { MATCH t: Task WHERE t.status = "pending" RETURN t } {
  status = "archived",
  archived_at = now()
}
```

### Computed Values

SET can use expressions referencing current values:

```
-- Increment
SET #counter_node.value = #counter_node.value + 1

-- Conditional (with expression support)
SET #task_123.priority = MIN(#task_123.priority + 1, 10)

-- String manipulation
SET #person_123.name = UPPER(#person_123.name)
```

### Validation

Before committing:

1. **Node/edge exists:** Target must exist
2. **Attribute exists:** Declared on the type
3. **Type checking:** Value matches attribute type
4. **Constraints:** All attribute modifiers satisfied

### Setting to Null

Optional attributes can be set to null:

```
SET #task_123.description = null
```

Required attributes cannot be set to null.

### Edge Attributes

SET works on edge attributes the same as node attributes:

```
SET #assignment_edge.role = "reviewer"
```

### Result Format

```typescript
interface SetResult {
  success: boolean
  modifiedCount: number
  modifiedIds: string[]
  errors?: string[]
}
```

## Layer 0

None. SET operates on attributes of user-defined node and edge types declared in the ontology.

## Examples

### Complete Task Update

```
SET #task_123 {
  status = "done",
  completed_at = now(),
  completed_by = $userId
}
RETURNING status, completed_at
```

### Bulk Status Update

```
-- Archive all pending tasks older than 30 days
SET {
  MATCH t: Task
  WHERE t.status = "pending"
    AND t.created_at < now() - 2592000000
  RETURN t
} {
  status = "archived",
  archived_at = now()
}
```

### Bulk with Safety Limit

```
SET {
  MATCH t: Task WHERE t.status = "pending" RETURN t LIMIT 100
}.status = "archived"
```

### Increment Counter

```
SET #counter.value = #counter.value + 1
RETURNING value
```

### Update Edge Attribute

```
SET #assignment_edge {
  role = "lead",
  updated_at = now()
}
```

### Conditional Update in Transaction

```
BEGIN
  SET #task_123.attempts = #task_123.attempts + 1
  SET #task_123.last_attempt = now()
  SET #task_123.status =
    CASE WHEN #task_123.attempts >= 3 THEN "failed" ELSE "retrying" END
COMMIT
```

### Project Migration

```
-- Move tasks from one project to another
SET {
  MATCH t: Task
  WHERE t.project_id = #old_project
  RETURN t
}.project_id = #new_project
```

## Errors

| Condition | Message |
|-----------|---------|
| Node/edge not found | `Node 'X' not found` |
| Attribute not found | `Attribute 'X' not found on type 'Y'` |
| Type mismatch | `Type mismatch: X expects Y, got Z` |
| Constraint violation | `Constraint 'X' violated: Y` |
| Required to null | `Cannot set required attribute 'X' to null` |
