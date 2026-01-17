---
spec: type_alias
version: "1.0"
status: draft
category: declaration
capability: schema
requires: [types, modifiers]
priority: common
---

# Spec: Type Alias

## Overview

Type aliases define reusable type expressions with optional modifiers. They reduce repetition, establish domain vocabulary, and enable union types for polymorphic signatures. Type aliases are purely syntactic sugar expanded at compile time and do not generate Layer 0 nodes.

## Syntax

### Grammar

```ebnf
TypeAliasDecl = "type" Identifier "=" TypeAliasBody

TypeAliasBody =
    ScalarAlias
  | UnionAlias

ScalarAlias = TypeExpr AttributeModifiers?

UnionAlias = TypeExpr ("|" TypeExpr)+

AttributeModifiers = "[" AttributeModifier ("," AttributeModifier)* "]"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `type` | Declaration |

### Examples

```
-- Scalar alias with format modifier
type Email = String [format: email]

-- Numeric range alias
type Priority = Int [0..10]

-- Enum alias
type TaskStatus = String [in: ["todo", "in_progress", "done", "blocked"]]

-- Union type alias (no modifiers allowed)
type Entity = Person | Organization | Bot
```

## Semantics

### Scalar Aliases

Scalar aliases associate a name with a base type plus optional attribute modifiers:

```
type Email = String [format: email]
type RequiredEmail = Email [required]
```

When a scalar alias is used:
1. The alias name is replaced with its underlying type
2. Modifiers from the alias are merged with any modifiers at the usage site

### Union Aliases

Union aliases define a type that can be any of the listed types:

```
type Entity = Person | Organization
```

Union aliases:
- Cannot have modifiers (modifiers only apply to scalar types)
- Expand to inline union types at all usage sites
- Are valid anywhere a type is expected (edge signatures, etc.)

### Alias Chaining

Aliases can reference other aliases:

```
type Email = String [format: email]
type RequiredEmail = Email [required]
type UniqueEmail = RequiredEmail [unique]
```

Expansion is recursive at compile time. `UniqueEmail` expands to `String [format: email, required, unique]`.

### Modifier Composition

When an alias is used with additional modifiers, they combine:

```
type Email = String [format: email]

node Person {
  email: Email [required, unique, indexed]
}
```

The `email` attribute has: `format: email`, `required`, `unique`, `indexed`.

**Conflict resolution:** If the same modifier appears in both alias and usage with different values, the usage wins:

```
type Priority = Int [0..10]

node Task {
  priority: Priority [<= 5] = 3   -- max is now 5, not 10
}
```

### Type Resolution

1. When the compiler encounters a type name, it checks if it's an alias
2. If alias, recursively resolve until reaching a built-in type or node/edge type
3. Collect all modifiers along the resolution chain
4. Apply conflict resolution (usage site wins)

### Restrictions

| Restriction | Reason |
|-------------|--------|
| No recursive aliases | Would cause infinite expansion |
| No shadowing built-ins | `type String = ...` is invalid |
| No shadowing node/edge types | `type Person = ...` when `Person` is a node type is invalid |
| Union aliases cannot have modifiers | Modifiers are scalar-specific |

## Layer 0

None.

Type aliases are purely syntactic sugar. They are fully expanded at compile time and produce no Layer 0 representation. The expanded types and modifiers appear directly in the compiled output.

## Examples

### Domain Vocabulary

```
ontology UserManagement {
  -- Establish domain vocabulary
  type Email = String [format: email]
  type Username = String [length: 3..32, match: "^[a-zA-Z0-9_]+$"]
  type Password = String [length: 8..128]
  type Role = String [in: ["admin", "moderator", "user", "guest"]]

  node User {
    email: Email [required, unique],
    username: Username [required, unique],
    password_hash: String [required],
    role: Role = "user"
  }
}
```

### Polymorphic Edge with Union

```
ontology Collaboration {
  type Participant = Person | Team | Bot
  type Artifact = Document | Codebase | Design

  -- Edge connecting any participant to any artifact
  edge contributes_to(who: Participant, what: Artifact) {
    role: String [in: ["author", "reviewer", "observer"]] = "author",
    since: Timestamp = now()
  }
}
```

### Chained Aliases for Refinement

```
ontology Inventory {
  -- Base type
  type ProductCode = String [match: "^[A-Z]{2}[0-9]{4}$"]

  -- Refinements
  type RequiredProductCode = ProductCode [required]
  type UniqueProductCode = RequiredProductCode [unique]

  node Product {
    sku: UniqueProductCode,               -- required, unique, with pattern
    alternate_sku: ProductCode?           -- optional, with pattern only
  }
}
```

### Numeric Domain Types

```
ontology Metrics {
  type Percentage = Float [>= 0.0, <= 100.0]
  type Probability = Float [>= 0.0, <= 1.0]
  type NonNegativeInt = Int [>= 0]
  type PositiveInt = Int [> 0]
  type Rating = Int [1..5]

  node Review {
    rating: Rating [required],
    confidence: Probability = 1.0,
    helpful_percent: Percentage?
  }
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Recursive alias | `"Type alias 'X' is recursive (references itself directly or indirectly)"` |
| Shadows built-in | `"Type alias 'String' shadows built-in type"` |
| Shadows node type | `"Type alias 'Person' shadows node type 'Person'"` |
| Modifiers on union | `"Union type aliases cannot have modifiers"` |
| Unknown alias reference | `"Unknown type 'X' in alias definition"` |

---

*End of Spec: Type Alias*
