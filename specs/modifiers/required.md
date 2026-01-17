---
spec: required
version: "1.0"
status: draft
category: modifier
capability: required
requires: []
priority: essential
---

# Spec: Required

## Overview

The `[required]` modifier ensures an attribute must have a non-null value. It prevents creation of entities without the attribute and prevents setting the attribute to null after creation.

**Why needed:** Many domain models have mandatory fields. The required modifier enforces presence at the schema level, preventing incomplete data from entering the graph.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | "required"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `required` | Attribute modifier |

### Examples
```
node Person {
  name: String [required],
  email: String [required, unique]
}

node Task {
  title: String [required],
  priority: Int [required] = 0
}
```

---

## Semantics

### Validation Timing

Required validation occurs at:
1. **SPAWN time**: Attribute must be provided or have a default
2. **SET time**: Cannot set attribute to null

### Behavior Table

| Declaration | Nullable? | Must provide at SPAWN? | Omitted behavior |
|-------------|-----------|------------------------|------------------|
| `x: T` | No | No | Compile warning |
| `x: T?` | Yes | No | Value is `null` |
| `x: T = default` | No | No | Value is `default` |
| `x: T [required]` | No | Yes | Error if omitted |
| `x: T? [required]` | -- | -- | Compile error |

### Required with Default

If an attribute has both `[required]` and a default value, the default satisfies the requirement:
```
node Task {
  status: String [required] = "pending"
}

SPAWN t: Task { title = "Test" }
-- t.status = "pending" (default satisfies required)
```

### Nullable + Required is Invalid

Combining `?` (nullable) with `[required]` is a compile-time error:
```
node Task {
  description: String? [required]  -- COMPILE ERROR
}
-- Error: Attribute 'description' cannot be both nullable (?) and [required].
--        Use 'String [required]' if a value must be provided.
--        Use 'String?' if the value can be null.
```

### Compilation to Constraint
```
name: String [required]
```

Compiles to:
```
constraint <type>_<attr>_required:
  x: <Type> WHERE x.<attr> = null
  => false
```

---

## Layer 0

None. The `[required]` modifier compiles to a constraint.

---

## Examples

### User Registration
```
ontology Users {
  node User {
    username: String [required, unique],
    email: String [required, unique],
    password_hash: String [required],
    display_name: String?
  }
}

-- Valid: all required fields provided
SPAWN u: User {
  username = "alice",
  email = "alice@example.com",
  password_hash = "abc123"
}

-- Error: missing required field
SPAWN u: User {
  username = "bob"
}
-- ERROR: Missing required attribute 'email' on User
```

### Required with Defaults
```
ontology Tasks {
  node Task {
    title: String [required],
    status: String [required] = "todo",
    priority: Int [required] = 5,
    created_at: Timestamp [required] = now()
  }
}

-- Only title must be explicitly provided
SPAWN t: Task { title = "My Task" }
-- t.status = "todo"
-- t.priority = 5
-- t.created_at = <current time>
```

### Preventing Null Assignment
```
node Person {
  name: String [required]
}

SPAWN p: Person { name = "Alice" }
SET p.name = null
-- ERROR: Cannot set required attribute 'name' to null
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Missing at SPAWN | `"Missing required attribute '<attr>' on <Type>"` |
| SET to null | `"Cannot set required attribute '<attr>' to null"` |
| Nullable + required | `"Attribute '<attr>' cannot be both nullable (?) and [required]"` |

---

*End of Spec: Required*
