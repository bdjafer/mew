---
spec: inspect
version: "1.0"
status: draft
category: statement
capability: observation
requires: []
priority: common
---

# Spec: INSPECT

## Overview

INSPECT retrieves a specific node or edge by its unique identifier. Unlike MATCH which finds patterns, INSPECT provides direct access to a known entity. It returns the complete entity data or a projection of specific attributes, and gracefully handles non-existent IDs by returning a "not found" result rather than an error.

## Syntax

### Grammar

```ebnf
InspectStmt = "inspect" IdRef ReturnClause?

IdRef       = "#" (Identifier | StringLiteral)

ReturnClause = "return" Projection ("," Projection)*

Projection   = Expr ("as" Identifier)? | "*"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `inspect` | Statement - retrieves entity by ID |
| `#` | Modifier - prefix for ID references |
| `return` | Clause - specifies output projection |

### Examples

```
-- Inspect node by ID
INSPECT #node_abc123

-- Inspect edge by ID
INSPECT #edge_xyz789

-- With specific attribute projection
INSPECT #task_123
RETURN title, status, priority

-- With system fields
INSPECT #person_456
RETURN *, _type, _id
```

## Semantics

### ID Reference Syntax

IDs are referenced using the `#` prefix:

```
#simple_id_123          -- Simple: alphanumeric + underscore
#"550e8400-e29b-41d4"   -- Quoted: any characters (UUIDs, hyphens, etc.)
```

**When to use quoted IDs:**
- UUIDs: `#"550e8400-e29b-41d4-a716-446655440000"`
- IDs with hyphens: `#"task-123-abc"`
- IDs with special characters: `#"node:type:123"`

**Note:** The `@` prefix is reserved for timestamp literals (e.g., `@2024-01-15`).

### Default Return

Without a RETURN clause, INSPECT returns the complete entity:

```
INSPECT #task_123
-- Returns all attributes plus system fields
```

### Projection

With RETURN, only specified fields are returned:

```
INSPECT #task_123
RETURN title, status
-- Returns only { title: "...", status: "..." }
```

### System Fields

Special fields available for inspection:

| Field | Description |
|-------|-------------|
| `_id` | Unique identifier |
| `_type` | Entity type name |
| `_targets` | (edges only) Connected node IDs |

```
INSPECT #edge_123
RETURN _id, _type, _targets
-- Returns { _id: "edge_123", _type: "assigned_to", _targets: ["task_1", "person_2"] }
```

### Not Found Handling

If the ID doesn't exist, INSPECT returns a "not found" result rather than raising an error:

```
INSPECT #nonexistent_id
-- Returns: { found: false, type: null, data: null }
```

This allows callers to check existence without exception handling.

### Node vs Edge Inspection

INSPECT works uniformly for both nodes and edges:

```
-- Inspect a node
INSPECT #person_123
-- Returns node data

-- Inspect an edge
INSPECT #assignment_edge_456
-- Returns edge data including _targets
```

## Layer 0

None. INSPECT is a runtime query operation with no graph representation.

## Examples

### Basic Node Inspection

```
INSPECT #task_abc123

-- Result:
{
  found: true,
  type: "node",
  data: {
    _id: "task_abc123",
    _type: "Task",
    title: "Implement feature X",
    status: "in_progress",
    priority: 7,
    created_at: 1705320000000
  }
}
```

### Edge Inspection with Targets

```
INSPECT #causes_edge_789

-- Result:
{
  found: true,
  type: "edge",
  data: {
    _id: "causes_edge_789",
    _type: "causes",
    _targets: ["event_001", "event_002"],
    strength: 0.85,
    created_at: 1705320000000
  }
}
```

### Projected Inspection

```
INSPECT #person_456
RETURN name, email, _type

-- Result:
{
  found: true,
  type: "node",
  data: {
    name: "Alice Smith",
    email: "alice@example.com",
    _type: "Person"
  }
}
```

### Existence Check Pattern

```
-- Check if entity exists before operating
INSPECT #maybe_exists
-- If found: false, handle accordingly
-- If found: true, proceed with operation
```

## Errors

| Condition | Message |
|-----------|---------|
| Invalid ID syntax | Invalid ID reference: expected # prefix |
| Unknown projection field | Attribute 'name' not found on type 'Type' |
| Empty ID reference | Invalid ID reference: ID cannot be empty |
