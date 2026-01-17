---
spec: unique
version: "1.0"
status: draft
category: modifier
capability: unique
requires: [indexed]
priority: essential
---

# Spec: Unique

## Overview

The `[unique]` modifier ensures an attribute value is unique across all instances of a type. No two entities of the same type (or its subtypes) can have the same non-null value for a unique attribute.

**Why needed:** Identifiers like emails, usernames, and external IDs must be unique to serve as lookup keys. The unique modifier enforces this at the schema level.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | "unique"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `unique` | Attribute modifier |

### Examples
```
node Person {
  email: String [unique],
  username: String [required, unique]
}

node Product {
  sku: String [unique, indexed]
}
```

---

## Semantics

### Uniqueness Scope

Uniqueness applies within the type and its subtypes:
```
node Entity {
  external_id: String [unique]
}

node Person : Entity {}
node Organization : Entity {}

-- These share the same uniqueness constraint
SPAWN p: Person { external_id = "abc" }
SPAWN o: Organization { external_id = "abc" }
-- ERROR: Duplicate value 'abc' for unique attribute 'external_id'
```

### Null Handling

Null values do not violate uniqueness. Multiple entities can have null for a unique attribute:
```
node Person {
  nickname: String? [unique]
}

SPAWN p1: Person { name = "Alice" }           -- nickname = null
SPAWN p2: Person { name = "Bob" }             -- nickname = null
-- OK: multiple nulls allowed

SPAWN p3: Person { name = "Charlie", nickname = "chuck" }
SPAWN p4: Person { name = "David", nickname = "chuck" }
-- ERROR: Duplicate value 'chuck' for unique attribute 'nickname'
```

### Implicit Indexing

The `[unique]` modifier implies `[indexed]`. An index is required for efficient uniqueness checking:
```
email: String [unique]
-- Equivalent to:
email: String [unique, indexed]
```

### Compilation to Constraint
```
email: String [unique]
```

Compiles to:
```
constraint <type>_<attr>_unique:
  x1: <Type>, x2: <Type>
  WHERE x1.id != x2.id AND x1.<attr> = x2.<attr> AND x1.<attr> != null
  => false
```

Plus an index definition for efficient lookup.

---

## Layer 0

### Constraints
```
constraint <Type>_<attr>_unique:
  x1: <Type>, x2: <Type>
  WHERE x1.id != x2.id AND x1.<attr> = x2.<attr> AND x1.<attr> != null
  => false
```

### Engine Hints

An index is created for the attribute to enable efficient duplicate detection.

---

## Examples

### User System
```
ontology Users {
  node User {
    -- Both required and unique
    email: String [required, unique],
    username: String [required, unique],

    -- Optional but unique if provided
    phone: String? [unique]
  }
}

SPAWN u1: User { email = "alice@example.com", username = "alice" }
SPAWN u2: User { email = "alice@example.com", username = "bob" }
-- ERROR: Duplicate value 'alice@example.com' for unique attribute 'email'
```

### External ID Mapping
```
ontology Integration {
  node ExternalEntity {
    external_id: String [required, unique],
    source_system: String [required],
    local_id: String [unique]
  }

  -- Composite uniqueness via constraint
  constraint unique_per_source:
    e1: ExternalEntity, e2: ExternalEntity
    WHERE e1.id != e2.id
      AND e1.source_system = e2.source_system
      AND e1.external_id = e2.external_id
    => false
}
```

### Type Aliases with Unique
```
ontology Types {
  type Email = String [format: email]
  type UniqueEmail = Email [unique]

  node Person {
    -- Expands to String [format: email, unique]
    email: UniqueEmail [required]
  }
}
```

### Multiple Unique Attributes
```
ontology Products {
  node Product {
    sku: String [required, unique],
    barcode: String? [unique],
    internal_code: String? [unique]
  }

  -- Each is independently unique
  SPAWN p1: Product { sku = "ABC-001", barcode = "123456789" }
  SPAWN p2: Product { sku = "ABC-002", barcode = "123456789" }
  -- ERROR: Duplicate value '123456789' for unique attribute 'barcode'
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Duplicate on SPAWN | `"Duplicate value '<value>' for unique attribute '<attr>' on <Type>"` |
| Duplicate on SET | `"Cannot set '<attr>' to '<value>': value already exists on another <Type>"` |

---

*End of Spec: Unique*
