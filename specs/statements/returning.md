---
spec: returning
version: "1.0"
status: draft
category: statement
capability: mutation-result
requires: [spawn, kill, link, unlink, set]
priority: common
---

# Spec: RETURNING Clause

## Overview

The RETURNING clause controls what data is returned after a mutation operation (SPAWN, KILL, LINK, UNLINK, SET). By default, mutations return minimal information, but RETURNING allows retrieving the ID, all attributes, or specific fields of created/modified entities. This enables efficient workflows where the result of a mutation is immediately needed.

## Syntax

### Grammar

```ebnf
ReturningClause = "returning" ReturningTarget

ReturningTarget = "id"
                | "*"
                | "created"
                | Projection ("," Projection)*

Projection      = (Identifier | Identifier "." Identifier) ("as" Identifier)?
```

### Keywords

| Keyword | Context |
|---------|---------|
| `returning` | Clause - specifies what to return after mutation |
| `id` | Target - return only the entity ID |
| `*` | Target - return all attributes |

### Examples

```
-- Return only the created ID
SPAWN t: Task { title = "Test" }
RETURNING id

-- Return full created node
SPAWN p: Person { name = "Alice", email = "alice@example.com" }
RETURNING *

-- Return specific attributes
SPAWN t: Task { title = "Test", priority = 5 }
RETURNING id, title, priority

-- Return from LINK with alias
LINK assigned_to(#task, #person) AS a
RETURNING a.id

-- Return CREATED flag from LINK IF NOT EXISTS
LINK IF NOT EXISTS member_of(#person, #team)
RETURNING CREATED
```

## Semantics

### Default Behavior

Without RETURNING, mutations return a success status and affected count, but not entity data:

```
SPAWN t: Task { title = "Test" }
-- Returns: { success: true }
```

### RETURNING id

Returns only the identifier of the created/affected entity:

```
SPAWN t: Task { title = "Test" }
RETURNING id
-- Returns: { id: "task_abc123" }
```

### RETURNING *

Returns all attributes of the entity including system fields:

```
SPAWN t: Task { title = "Test" }
RETURNING *
-- Returns: { id: "task_abc123", _type: "Task", title: "Test", status: "todo", ... }
```

### RETURNING specific fields

Returns only the specified attributes:

```
SPAWN p: Person { name = "Alice", email = "alice@example.com", role = "admin" }
RETURNING id, name, email
-- Returns: { id: "person_xyz", name: "Alice", email: "alice@example.com" }
```

### RETURNING with Aliases

When using AS to bind entities, RETURNING can reference the alias:

```
LINK causes(#e1, #e2) AS c
RETURNING c.id, c.strength
-- Returns: { id: "edge_123", strength: 0.85 }
```

For LINK with inline SPAWN:

```
LINK belongs_to(
  SPAWN Task { title = "New task" } AS t,
  #project_123
) AS e
RETURNING t.id AS task_id, e.id AS edge_id
-- Returns: { task_id: "task_456", edge_id: "edge_789" }
```

### RETURNING CREATED

For `LINK IF NOT EXISTS`, the special `CREATED` field indicates whether a new edge was created:

```
LINK IF NOT EXISTS assigned_to(#task, #person)
RETURNING id, CREATED
-- Returns: { id: "edge_123", created: true }  -- if new
-- Returns: { id: "edge_456", created: false } -- if existed
```

### RETURNING from Bulk Operations

For operations affecting multiple entities:

```
SET { MATCH t: Task WHERE t.status = "pending" RETURN t }.status = "archived"
RETURNING id, status
-- Returns array: [
--   { id: "task_1", status: "archived" },
--   { id: "task_2", status: "archived" },
--   ...
-- ]
```

### Context by Operation

| Operation | What RETURNING refers to |
|-----------|--------------------------|
| SPAWN | The created node |
| KILL | The killed node(s) (before deletion) |
| LINK | The created edge |
| UNLINK | The removed edge(s) (before removal) |
| SET | The modified node/edge |

## Layer 0

None. RETURNING is a clause that modifies mutation result format, with no graph representation.

## Examples

### Create and Immediately Use ID

```
-- Create task and get its ID for subsequent operations
SPAWN t: Task { title = "New feature", priority = 8 }
RETURNING id

-- Use the returned ID
LINK assigned_to(#returned_id, #person_123)
```

### Create with Full Data Return

```
SPAWN p: Person {
  name = "Bob Smith",
  email = "bob@example.com",
  role = "developer",
  active = true
}
RETURNING *

-- Returns:
{
  id: "person_abc123",
  _type: "Person",
  name: "Bob Smith",
  email: "bob@example.com",
  role: "developer",
  active: true,
  created_at: 1705320000000
}
```

### Inline SPAWN with Multiple Returns

```
LINK collaboration(
  SPAWN Person { name = "Alice" } AS alice,
  SPAWN Person { name = "Bob" } AS bob
) AS collab
RETURNING alice.id, bob.id, collab.id

-- Returns:
{
  alice_id: "person_001",
  bob_id: "person_002",
  collab_id: "edge_003"
}
```

### Idempotent Operation with Status

```
LINK IF NOT EXISTS follows(#user_123, #user_456)
RETURNING id, CREATED AS is_new

-- First call returns:
{ id: "edge_789", is_new: true }

-- Subsequent calls return:
{ id: "edge_789", is_new: false }
```

### Bulk Update with Results

```
SET { MATCH t: Task WHERE t.priority > 8 RETURN t } {
  reviewed = true,
  reviewed_at = now()
}
RETURNING id, title, reviewed_at

-- Returns:
[
  { id: "task_1", title: "Critical bug", reviewed_at: 1705320000000 },
  { id: "task_2", title: "Security fix", reviewed_at: 1705320000000 },
  { id: "task_3", title: "Performance issue", reviewed_at: 1705320000000 }
]
```

## Errors

| Condition | Message |
|-----------|---------|
| Unknown attribute in RETURNING | Attribute 'name' not found on type 'Type' |
| CREATED outside LINK IF NOT EXISTS | CREATED is only valid with LINK IF NOT EXISTS |
| Alias not found | Alias 'name' not defined in this statement |
| RETURNING on failed operation | Operation failed; no data to return |
