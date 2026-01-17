---
spec: length_constraint
version: "1.0"
status: draft
category: modifier
capability: length_constraint
requires: []
priority: common
---

# Spec: Length Constraint

## Overview

The `[length: N..M]` modifier constrains the character length of a string attribute. It ensures strings are within acceptable bounds, preventing both empty strings and excessively long values.

**Why needed:** String lengths often have domain-specific limits (names, descriptions, codes). The length constraint enforces these limits at the schema level, preventing data that would cause problems in downstream systems.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | LengthModifier

LengthModifier = "length:" IntLiteral ".." IntLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `length` | Attribute modifier |

### Examples
```
node Person {
  name: String [length: 1..100],
  bio: String? [length: 0..2000]
}

node Product {
  sku: String [length: 6..6],      -- exactly 6 characters
  description: String [length: 10..5000]
}
```

---

## Semantics

### Range Interpretation

| Syntax | Meaning |
|--------|---------|
| `[length: 1..100]` | 1 to 100 characters (inclusive) |
| `[length: 6..6]` | Exactly 6 characters |
| `[length: 0..500]` | Up to 500 characters (empty allowed) |

### Validation Timing

Length validation occurs at:
1. **SPAWN time**: String must satisfy length bounds
2. **SET time**: New value must satisfy length bounds

### Null Handling

Length validation is skipped for null values:
```
node Person {
  bio: String? [length: 0..2000]
}

SPAWN p: Person { name = "Alice" }  -- bio = null, OK
SPAWN p: Person { name = "Alice", bio = null }  -- OK
```

**To require a value AND validate length**, combine with `[required]`:
```
name: String [required, length: 1..100]
```

### Character Counting

Length is measured in Unicode scalar values (characters), not bytes:
```
node Message {
  content: String [length: 1..280]
}

-- "Hello" = 5 characters
-- "Caf\u00e9" (Cafe with accent) = 4 characters
-- Emoji "👍" = 1 character (single code point)
```

### Compilation to Constraint
```
name: String [length: 1..100]
```

Compiles to:
```
constraint <type>_<attr>_length:
  x: <Type> WHERE x.<attr> != null
  => length(x.<attr>) >= 1 AND length(x.<attr>) <= 100
```

---

## Layer 0

None. The length constraint compiles to a standard constraint using the built-in `length()` function.

---

## Examples

### User Profile
```
ontology Users {
  node User {
    -- Username: 3-30 characters
    username: String [required, length: 3..30, format: slug],

    -- Display name: 1-100 characters
    display_name: String [required, length: 1..100],

    -- Bio: optional, up to 500 characters
    bio: String? [length: 0..500],

    -- Short bio for cards: exactly what fits
    tagline: String? [length: 1..140]
  }
}

-- Valid
SPAWN u: User {
  username = "alice",
  display_name = "Alice Johnson",
  bio = "Software developer from Seattle."
}

-- Invalid: username too short
SPAWN u: User { username = "ab", display_name = "Test" }
-- ERROR: Attribute 'username' length 2 is below minimum 3

-- Invalid: display_name too long
SPAWN u: User { username = "alice", display_name = "A" * 200 }
-- ERROR: Attribute 'display_name' length 200 exceeds maximum 100
```

### Product Codes
```
ontology Inventory {
  node Product {
    -- SKU: exactly 8 characters
    sku: String [required, unique, length: 8..8],

    -- Barcode: 12-14 digits
    barcode: String? [unique, length: 12..14],

    -- Name: reasonable bounds
    name: String [required, length: 1..200],

    -- Description: allow substantial text
    description: String? [length: 0..10000]
  }
}

-- Valid SKU
SPAWN p: Product { sku = "PROD0001", name = "Widget" }

-- Invalid: SKU too short
SPAWN p: Product { sku = "PROD01", name = "Widget" }
-- ERROR: Attribute 'sku' length 6 is below minimum 8
```

### Comments and Messages
```
ontology Social {
  node Post {
    title: String [required, length: 1..200],
    body: String [required, length: 1..50000]
  }

  node Comment {
    content: String [required, length: 1..2000]
  }

  node Message {
    subject: String? [length: 1..100],
    body: String [required, length: 1..10000]
  }
}
```

### Combining with Other Modifiers
```
node Entity {
  -- Required with length
  name: String [required, length: 1..100],

  -- Unique with length
  code: String [unique, length: 4..10],

  -- Format with length
  email: String [format: email, length: 5..254],

  -- Regex with length (redundant but explicit)
  product_code: String [match: "^[A-Z]{2}[0-9]{4}$", length: 6..6]
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Below minimum | `"Attribute '<attr>' length <len> is below minimum <min>"` |
| Above maximum | `"Attribute '<attr>' length <len> exceeds maximum <max>"` |
| Invalid range | `"Length minimum <min> cannot exceed maximum <max>"` |
| Non-string type | `"[length] constraint only valid for String attributes"` |

---

*End of Spec: Length Constraint*
