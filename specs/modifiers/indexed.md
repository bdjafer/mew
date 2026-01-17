---
spec: indexed
version: "1.0"
status: draft
category: modifier
capability: indexed
requires: []
priority: common
---

# Spec: Indexed

## Overview

The `[indexed]` modifier creates an index on an attribute for faster lookups. Indexes speed up queries that filter or sort by the indexed attribute, at the cost of additional storage and write overhead.

**Why needed:** Large graphs require efficient attribute lookups. Without indexes, queries must scan all entities. The indexed modifier allows schema designers to declare which attributes need fast access.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | IndexedModifier

IndexedModifier = "indexed" (":" IndexDirection)?

IndexDirection = "asc" | "desc"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `indexed` | Attribute modifier |
| `asc` | Index direction (ascending, default) |
| `desc` | Index direction (descending) |

### Examples
```
node Event {
  timestamp: Timestamp [indexed],
  priority: Int [indexed: desc]
}

node Person {
  email: String [unique],        -- unique implies indexed
  last_login: Timestamp [indexed: desc]
}
```

---

## Semantics

### Index Types

| Syntax | Behavior |
|--------|----------|
| `[indexed]` | Ascending index (default) |
| `[indexed: asc]` | Ascending index (explicit) |
| `[indexed: desc]` | Descending index |

### Ascending vs Descending

Index direction affects sort order efficiency:
- **Ascending (`asc`)**: Efficient for `ORDER BY attr ASC` and range queries like `attr >= value`
- **Descending (`desc`)**: Efficient for `ORDER BY attr DESC` and "most recent first" patterns

```
node LogEntry {
  created_at: Timestamp [indexed: desc]
}

-- Efficient: uses descending index directly
MATCH l: LogEntry
ORDER BY l.created_at DESC
LIMIT 100
RETURN l
```

### Unique Implies Indexed

The `[unique]` modifier automatically creates an index:
```
email: String [unique]
-- Index is created automatically for uniqueness enforcement
```

### Not a Constraint

Unlike other modifiers, `[indexed]` does not generate a constraint. It is purely an engine optimization hint that affects storage and query planning.

### Multiple Indexes

Multiple attributes can be indexed independently:
```
node Task {
  created_at: Timestamp [indexed: desc],
  priority: Int [indexed: desc],
  status: String [indexed]
}
```

---

## Layer 0

None. The `[indexed]` modifier is an engine hint, not a constraint. It affects the storage layer but does not create Layer 0 graph structures.

---

## Examples

### Event Log with Temporal Index
```
ontology Logging {
  node LogEntry {
    message: String [required],
    level: String [in: ["debug", "info", "warn", "error"]],
    timestamp: Timestamp [required, indexed: desc],
    source: String [indexed]
  }
}

-- Fast: descending index supports this query pattern
MATCH l: LogEntry
WHERE l.level = "error"
ORDER BY l.timestamp DESC
LIMIT 50
RETURN l
```

### User Directory
```
ontology Users {
  node User {
    username: String [required, unique],     -- auto-indexed
    email: String [required, unique],        -- auto-indexed
    created_at: Timestamp [required, indexed],
    last_active: Timestamp [indexed: desc]
  }
}

-- Fast lookup by username (unique = indexed)
MATCH u: User WHERE u.username = "alice"
RETURN u

-- Fast "recently active" query
MATCH u: User
ORDER BY u.last_active DESC
LIMIT 20
RETURN u
```

### Indexed Enums for Filtering
```
ontology Tasks {
  node Task {
    status: String [in: ["todo", "in_progress", "done"], indexed],
    priority: Int [>= 0, <= 10, indexed: desc],
    assignee: String? [indexed]
  }
}

-- Fast: index on status
MATCH t: Task WHERE t.status = "todo"
RETURN t

-- Fast: descending index on priority
MATCH t: Task WHERE t.status = "todo"
ORDER BY t.priority DESC
RETURN t
```

### Edges with Indexed Attributes
```
ontology Relationships {
  edge assigned_to(task: Task, person: Person) {
    assigned_at: Timestamp [required, indexed: desc],
    role: String [indexed]
  }
}

-- Find recent assignments
MATCH t: Task, p: Person, assigned_to(t, p) AS a
ORDER BY a.assigned_at DESC
LIMIT 10
RETURN t, p, a.assigned_at
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Invalid direction | `"Invalid index direction '<value>'. Use 'asc' or 'desc'"` |

---

*End of Spec: Indexed*
