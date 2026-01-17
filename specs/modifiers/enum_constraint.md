---
spec: enum_constraint
version: "1.0"
status: draft
category: modifier
capability: enum_constraint
requires: []
priority: common
---

# Spec: Enum Constraint

## Overview

The `[in: [...]]` modifier restricts an attribute's value to a fixed set of allowed values. It provides type-safe enumeration without requiring a separate enum type definition.

**Why needed:** Many attributes have a limited set of valid values (status codes, roles, categories). The enum constraint enforces this at the schema level, preventing invalid values from entering the graph.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | EnumModifier

EnumModifier = "in:" "[" Literal ("," Literal)* "]"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `in` | Attribute modifier |

### Examples
```
node Task {
  status: String [in: ["todo", "in_progress", "done", "blocked"]],
  priority: Int [in: [1, 2, 3, 4, 5]]
}

node Person {
  role: String [in: ["admin", "moderator", "user"]] = "user"
}
```

---

## Semantics

### Supported Types

The enum constraint works with:
- `String` - Most common use case
- `Int` - For numeric enums
- `Float` - Less common but supported

```
-- String enum
status: String [in: ["active", "inactive"]]

-- Integer enum
priority: Int [in: [1, 2, 3]]

-- Float enum (rare)
level: Float [in: [0.0, 0.5, 1.0]]
```

### Validation Timing

Enum validation occurs at:
1. **SPAWN time**: Value must be in the allowed set
2. **SET time**: New value must be in the allowed set

### Null Handling

Enum validation is skipped for null values:
```
node Task {
  category: String? [in: ["bug", "feature", "chore"]]
}

SPAWN t: Task { title = "Test" }  -- category = null, OK
SPAWN t: Task { title = "Test", category = null }  -- OK
SPAWN t: Task { title = "Test", category = "other" }  -- ERROR
```

**To require a value AND validate it**, combine with `[required]`:
```
status: String [required, in: ["todo", "done"]]
```

### Type Alias Usage

Enums can be defined as type aliases for reuse:
```
type TaskStatus = String [in: ["todo", "in_progress", "done", "blocked"]]
type Priority = Int [in: [1, 2, 3, 4, 5]]

node Task {
  status: TaskStatus = "todo",
  priority: Priority = 3
}
```

### Compilation to Constraint
```
status: String [in: ["draft", "active", "archived"]]
```

Compiles to:
```
constraint <type>_<attr>_enum:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> = "draft" OR x.<attr> = "active" OR x.<attr> = "archived"
```

---

## Layer 0

None. The enum constraint compiles to a standard constraint with OR conditions.

---

## Examples

### Task Management
```
ontology Tasks {
  type TaskStatus = String [in: ["todo", "in_progress", "done", "blocked"]]
  type Priority = Int [in: [1, 2, 3, 4, 5]]

  node Task {
    title: String [required],
    status: TaskStatus [required] = "todo",
    priority: Priority = 3,
    category: String? [in: ["bug", "feature", "chore", "docs"]]
  }
}

-- Valid
SPAWN t: Task { title = "Fix bug", status = "in_progress", priority = 5 }

-- Invalid status
SPAWN t: Task { title = "Test", status = "pending" }
-- ERROR: Value 'pending' not in allowed values ["todo", "in_progress", "done", "blocked"]

-- Invalid priority
SPAWN t: Task { title = "Test", priority = 10 }
-- ERROR: Value 10 not in allowed values [1, 2, 3, 4, 5]
```

### User Roles
```
ontology Users {
  node User {
    username: String [required, unique],
    role: String [required, in: ["admin", "moderator", "member", "guest"]] = "member",
    status: String [required, in: ["active", "suspended", "deleted"]] = "active"
  }

  edge member_of(person: User, team: Team) {
    role: String [in: ["owner", "admin", "member", "viewer"]] = "member"
  }
}
```

### State Machines
```
ontology Workflow {
  node Order {
    status: String [required, in: [
      "pending",
      "confirmed",
      "processing",
      "shipped",
      "delivered",
      "cancelled",
      "refunded"
    ]] = "pending"
  }

  -- Rule to enforce valid transitions could be added separately
  constraint valid_order_transition [message: "Invalid status transition"]:
    o: Order
    WHERE o.status = "delivered"
    => o.status != "pending"  -- Can't go back to pending from delivered
}
```

### Combining with Other Modifiers
```
node Person {
  -- Required enum with default
  status: String [required, in: ["active", "inactive"]] = "active",

  -- Unique enum (uncommon but valid)
  role_code: String [unique, in: ["ADM", "MOD", "USR"]],

  -- Indexed enum for fast filtering
  department: String [indexed, in: ["engineering", "sales", "support", "hr"]]
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Value not in set | `"Value '<value>' not in allowed values [<values>]"` |
| Empty enum list | `"Enum constraint requires at least one value"` |
| Type mismatch | `"Enum values must match attribute type <type>"` |

---

*End of Spec: Enum Constraint*
