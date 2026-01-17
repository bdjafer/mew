---
spec: regex_validation
version: "1.0"
status: draft
category: modifier
requires: []
priority: specialized
---

# Spec: Regex Validation

## Overview

The `[match: "pattern"]` modifier validates string attributes against a custom regular expression. For domain-specific patterns not covered by built-in formats.

**Why needed:** Many domains have custom identifier formats: product codes, license plates, internal IDs. Regex validation ensures data consistency at the schema level.

**Restriction:** Regex validation is compile-time/write-time only. The `matches()` function is not exposed for query-time filtering (performance concern). For query-time validation, use built-in formats or application logic.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | MatchModifier

MatchModifier = "match:" StringLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `match` | Attribute modifier |

### Examples
```
node Product {
  code: String [match: "^[A-Z]{2}[0-9]{4}$"],        -- e.g., "AB1234"
  sku: String [match: "^SKU-[0-9]{8}$"]              -- e.g., "SKU-00012345"
}

node Vehicle {
  plate: String [match: "^[A-Z]{3}-[0-9]{4}$"]      -- e.g., "ABC-1234"
}
```

---

## Semantics

### Regex Flavor

Patterns use a portable regex subset:

| Feature | Supported | Example |
|---------|-----------|---------|
| Literals | ✓ | `abc` |
| Character classes | ✓ | `[a-z]`, `[A-Z0-9]` |
| Negated classes | ✓ | `[^0-9]` |
| Quantifiers | ✓ | `*`, `+`, `?`, `{n}`, `{n,m}` |
| Anchors | ✓ | `^`, `$` |
| Alternation | ✓ | `a|b` |
| Grouping | ✓ | `(abc)+` |
| Escape sequences | ✓ | `\d`, `\w`, `\s`, `\.` |
| Lookahead/behind | ✗ | — |
| Backreferences | ✗ | — |
| Unicode categories | ✗ | — |

### Anchoring

Patterns are implicitly anchored if no anchors present:
```
[match: "[A-Z]+"]
-- Treated as: ^[A-Z]+$
-- Matches "ABC", not "123ABC456"
```

Explicit anchors override:
```
[match: ".*@example\\.com$"]
-- Matches "user@example.com"
-- Matches "admin@example.com"
-- Not "user@other.com"
```

### Validation Timing

Regex validation occurs at SPAWN and SET time only:
```
node Product {
  code: String [match: "^[A-Z]{2}[0-9]{4}$"]
}

SPAWN p: Product { code = "invalid" }
-- ERROR: 'invalid' does not match pattern '^[A-Z]{2}[0-9]{4}$'
```

### Null Handling

Validation is skipped for null values:
```
node Product {
  code: String? [match: "^[A-Z]{2}[0-9]{4}$"]
}

SPAWN p: Product { code = null }  -- OK
```

### No Query-Time Filtering

Unlike `[format:]`, regex patterns are **not** available at query time:
```
-- NOT SUPPORTED:
MATCH p: Product WHERE matches(p.code, "^AB.*")
-- ERROR: 'matches' function not available in queries
```

**Rationale:** Arbitrary regex matching is expensive and not GPU-friendly. Use `[format:]` for patterns that need query-time filtering, or filter in application code.

### Compilation to Constraint
```
code: String [match: "^[A-Z]{2}[0-9]{4}$"]
```

Compiles to internal constraint (not using exposed function):
```
constraint <type>_code_match:
  x: <Type> WHERE x.code != null
  => _internal_matches(x.code, "^[A-Z]{2}[0-9]{4}$")
```

---

## Layer 0

None. Regex validation compiles to constraints using internal matching.

---

## Compilation
```
node Product {
  code: String [required, match: "^[A-Z]{2}[0-9]{4}$"]
}
```

Compiles to:
```
_AttributeDef node:
  name: "code"
  scalar_type: "String"
  required: true

_ConstraintDef node:
  name: "Product_code_match"
  hard: true
  message: "Value does not match pattern '^[A-Z]{2}[0-9]{4}$'"
```

---

## Examples

### Product Codes
```
ontology Inventory {
  node Product {
    -- Format: 2 letters + 4 digits + dash + letter
    code: String [required, unique, match: "^[A-Z]{2}[0-9]{4}-[A-Z]$"],
    name: String [required]
  }
}

-- Valid: "AB1234-X", "ZZ9999-A"
-- Invalid: "AB123-X", "ab1234-x", "AB1234X"
```

### License Plates
```
ontology Vehicles {
  node Vehicle {
    -- US-style plate: 3 letters, dash, 4 digits
    plate: String [required, unique, match: "^[A-Z]{3}-[0-9]{4}$"],
    make: String,
    model: String
  }
}
```

### Internal IDs
```
ontology Integration {
  node Record {
    -- Legacy system ID: PREFIX_YYYYMMDD_SEQUENCE
    legacy_id: String [unique, match: "^[A-Z]+_[0-9]{8}_[0-9]{6}$"],
    data: String
  }
}

-- Valid: "CUST_20240115_000042"
-- Invalid: "CUST_2024-01-15_42"
```

### Version Numbers
```
ontology Packages {
  node Package {
    name: String [required, unique],
    -- Semantic versioning
    version: String [required, match: "^[0-9]+\\.[0-9]+\\.[0-9]+$"]
  }
}

-- Valid: "1.0.0", "12.34.56"
-- Invalid: "1.0", "v1.0.0"
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Value doesn't match | `"'value' does not match pattern 'pattern'"` |
| Invalid regex | `"Invalid regex pattern: <error>"` |
| Unsupported feature | `"Regex feature not supported: <feature>"` |
| Match on non-string | `"[match] only valid for String attributes"` |

---

*End of Spec: Regex Validation*