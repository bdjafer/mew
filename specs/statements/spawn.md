---
spec: spawn
version: "1.0"
status: draft
category: statement
capability: mutation
requires: [type_expr, attr_block, returning_clause]
priority: essential
---

# Spec: SPAWN

## Overview

SPAWN creates a new node in the graph. It is the fundamental operation for introducing entities into the graph, supporting attribute initialization, default values, and returning clause to control what data is returned after creation.

## Syntax

### Grammar

```ebnf
SpawnStmt       = "spawn" Identifier ":" TypeExpr AttrBlock? ReturningClause? ;

AttrBlock       = "{" (AttrAssignment ("," AttrAssignment)*)? "}" ;

AttrAssignment  = Identifier "=" Expr ;

ReturningClause = "returning" ("id" | "*" | Identifier ("," Identifier)*) ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `spawn` | Statement - initiates node creation |
| `returning` | Clause - specifies what to return after creation |

### Examples

```
-- Basic spawn with attributes
SPAWN t: Task {
  title = "New task",
  priority = 5
}

-- Minimal spawn relying on defaults
SPAWN t: Task { title = "Minimal task" }

-- Return created ID
SPAWN t: Task { title = "Test" }
RETURNING id

-- Return full created node
SPAWN p: Person { name = "Bob", email = "bob#example.com" }
RETURNING *
```

## Semantics

### Node Creation

When SPAWN executes:
1. A new node ID is generated
2. The node type is set to the specified TypeExpr
3. Provided attributes are set to their specified values
4. Missing attributes with defaults in the ontology receive their default values
5. Validation occurs before the node is committed

### Attribute Values

Attributes can be specified using:

**Literals:**
```
SPAWN t: Task {
  title = "Hello",
  priority = 5,
  score = 3.14,
  active = true,
  description = null
}
```

**Expressions:**
```
SPAWN t: Task {
  title = "Task " ++ $suffix,
  created_at = now(),
  priority = $basePriority + 1
}
```

**Defaults:**
Attributes with defaults in the ontology can be omitted:
```
-- Ontology: status: String = "todo"
SPAWN t: Task { title = "Test" }
-- t.status = "todo" (default applied)
```

### Validation

Before committing, the engine validates:

1. **Type exists:** The specified type is declared in the ontology
2. **Required attributes:** All `[required]` attributes are present
3. **Type checking:** Attribute values match declared types
4. **Constraints:** All attribute modifiers are satisfied

### RETURNING Clause

Controls what data is returned after creation:

| Form | Returns |
|------|---------|
| `RETURNING id` | Only the created node ID |
| `RETURNING *` | Full node with all attributes |
| `RETURNING attr1, attr2, ...` | Specific attributes only |

```
-- Return only ID (default)
SPAWN t: Task { title = "Test" }
RETURNING id
-- { id: "task_abc123" }

-- Return full node
SPAWN t: Task { title = "Test" }
RETURNING *
-- { id: "task_abc123", _type: "Task", title: "Test", status: "todo", ... }

-- Return specific attributes
SPAWN p: Person { name = "Alice", email = "alice#example.com" }
RETURNING id, name, email
-- { id: "person_xyz", name: "Alice", email: "alice#example.com" }
```

### Result Format

```typescript
interface SpawnResult {
  success: boolean
  id: string                     // created node ID
  data: Record<string, Value>    // based on RETURNING
  errors?: string[]              // if validation failed
}
```

## Layer 0

None. SPAWN operates on user-defined node types declared in the ontology.

## Examples

### Complete Person Creation

```
SPAWN p: Person {
  name = "Alice Smith",
  email = "alice#example.com",
  role = "admin",
  active = true
}
RETURNING *
```

### Task with Computed Values

```
SPAWN t: Task {
  title = "Review PR #" ++ $prNumber,
  created_at = now(),
  priority = COALESCE($priority, 5),
  status = "pending"
}
RETURNING id, title, created_at
```

### Chained SPAWNs in Transaction

```
BEGIN
  SPAWN p: Project { name = "New Initiative" }
  SPAWN t: Task { title = "Initial planning", priority = 1 }
  SPAWN t2: Task { title = "Requirements gathering", priority = 2 }
COMMIT
```

## Errors

| Condition | Message |
|-----------|---------|
| Unknown type | `Type 'X' not declared in ontology` |
| Missing required attribute | `Required attribute 'X' not provided` |
| Type mismatch | `Type mismatch: X expects Y, got Z` |
| Constraint violation | `Constraint 'X' violated` |
