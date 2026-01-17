---
spec: node_modifiers
version: "1.0"
status: draft
category: declaration
capability: schema
requires: [node_type]
priority: common
---

# Spec: Node Modifiers (abstract, sealed)

## Overview

Node modifiers `abstract` and `sealed` control the instantiation and inheritance behavior of node types. Abstract types cannot be instantiated directly but serve as base types for inheritance hierarchies. Sealed types cannot be inherited by user ontologies, protecting implementation details and ensuring type stability.

## Syntax

### Grammar

```ebnf
NodeTypeDecl =
  DocComment?
  NodeModifiers?
  "node" Identifier InheritanceClause? "{" AttributeDecl* "}"

NodeModifiers = "[" NodeModifier ("," NodeModifier)* "]"

NodeModifier = "abstract" | "sealed"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*
```

### Keywords

| Keyword | Context |
|---------|---------|
| `abstract` | Node modifier |
| `sealed` | Node modifier |

### Examples

```
-- Abstract base type (cannot be instantiated)
node Entity [abstract] {
  id: String [required, unique],
  created_at: Timestamp = now()
}

-- Sealed type (cannot be inherited by user ontologies)
node _SystemConfig [sealed] {
  key: String [required, unique],
  value: String [required]
}

-- Both abstract and sealed (Layer 0 base types use this)
node _MetaType [abstract, sealed] {
  -- Base for all meta-level types
}

-- Concrete type inheriting from abstract
node Person : Entity {
  name: String [required],
  email: String [format: email]
}
```

## Semantics

### Abstract Types

An `abstract` node type:
- **Cannot be instantiated directly** via SPAWN
- **Can be inherited** by other node types
- **Can appear in patterns** to match any subtype
- **Can have attributes** inherited by subtypes
- **Can have constraints** that apply to all subtypes

```
node Entity [abstract] {
  id: String [required, unique]
}

-- Valid: inherit from abstract
node Person : Entity {
  name: String
}

-- Valid: pattern matches Person instances
MATCH e: Entity WHERE e.id = "123"

-- Invalid: cannot instantiate
SPAWN e: Entity { id = "123" }
-- Error: Cannot instantiate abstract type 'Entity'
```

### Sealed Types

A `sealed` node type:
- **Can be instantiated** (unless also abstract)
- **Cannot be inherited** by user-defined ontologies
- **Can be inherited** by types within the same Layer 0 or system ontology

```
node _NodeType [sealed] {
  name: String [required, unique]
}

-- Invalid: user ontology cannot inherit
ontology MyApp {
  node CustomNodeType : _NodeType { }
  -- Error: Cannot inherit from sealed type '_NodeType'
}

-- Valid: Layer 0 can define subtypes of its own sealed types
node _SpecialNodeType : _NodeType [sealed] {
  special: Bool = false
}
```

### Combined Modifiers

When both `abstract` and `sealed` are applied:
- Cannot be instantiated (abstract)
- Cannot be inherited by user ontologies (sealed)
- Used for Layer 0 base types that define interfaces but aren't meant for extension

```
node _TypeExpr [abstract, sealed] {
  -- Base for all type expressions
  -- Only Layer 0 can define subtypes
}
```

### Pattern Matching with Abstract Types

Abstract types enable polymorphic pattern matching:

```
node Entity [abstract] {
  created_at: Timestamp
}

node Person : Entity { name: String }
node Organization : Entity { name: String }

-- Matches both Person and Organization
MATCH e: Entity WHERE e.created_at > @timestamp
RETURN e

-- Constraint applies to all Entity subtypes
constraint entities_have_timestamp:
  e: Entity
  => e.created_at != null
```

### Inheritance Validation

The compiler validates inheritance relationships:

| Parent Modifier | Child Allowed? | Notes |
|-----------------|----------------|-------|
| (none) | Yes | Normal inheritance |
| `abstract` | Yes | Concrete child can instantiate |
| `sealed` | Layer 0 only | User ontologies cannot inherit |
| `abstract, sealed` | Layer 0 only | User ontologies cannot inherit |

### Subtype Checking

For type compatibility:
- `T <: T` (reflexive)
- If `B : A`, then `B <: A` (subtype inherits from parent)
- If `B <: A` and `A` is abstract, instances of `B` match patterns typed as `A`

## Layer 0

### Nodes

The `_NodeType` meta-type has attributes for these modifiers:

```
node _NodeType [sealed] {
  name: String [required, unique],
  abstract: Bool = false,
  sealed: Bool = false,
  doc: String?
}
```

### Edges

None specific to modifiers.

### Constraints

```
constraint _abstract_not_instantiated:
  -- Enforced procedurally: SPAWN validates target type is not abstract

constraint _sealed_not_inherited:
  parent: _NodeType, child: _NodeType,
  _type_inherits(child, parent)
  => parent.sealed = false
```

## Examples

### Domain Model Hierarchy

```
ontology Commerce {
  -- Abstract base for all products
  node Product [abstract] {
    sku: String [required, unique],
    name: String [required],
    price: Float [required, >= 0]
  }

  -- Concrete product types
  node PhysicalProduct : Product {
    weight: Float [required, > 0],
    dimensions: String?
  }

  node DigitalProduct : Product {
    download_url: String [required, format: url],
    file_size: Int [required, > 0]
  }

  node Subscription : Product {
    billing_period: String [in: ["monthly", "yearly"]] [required],
    trial_days: Int [>= 0] = 0
  }

  -- Edge works with any Product subtype
  edge purchased(customer: Customer, product: Product) {
    quantity: Int [required, >= 1],
    purchased_at: Timestamp = now()
  }
}
```

### Protected System Types

```
ontology System {
  -- Sealed: users cannot extend system configuration
  node _Config [sealed] {
    key: String [required, unique],
    value: String [required],
    readonly: Bool = false
  }

  -- Sealed: internal audit log
  node _AuditLog [sealed] {
    timestamp: Timestamp = now(),
    action: String [required],
    actor: String?
  }

  -- Users can reference but not extend
  edge configured_by(entity: any, config: _Config)
}
```

### Abstract Trait Pattern

```
ontology Traits {
  -- Abstract "trait" for entities with names
  node Named [abstract] {
    name: String [required, length: 1..200]
  }

  -- Abstract "trait" for entities with timestamps
  node Timestamped [abstract] {
    created_at: Timestamp [required] = now(),
    updated_at: Timestamp?
  }

  -- Abstract "trait" for soft-deletable entities
  node SoftDeletable [abstract] {
    deleted: Bool = false,
    deleted_at: Timestamp?
  }

  -- Constraint on trait
  constraint soft_delete_has_timestamp:
    e: SoftDeletable WHERE e.deleted = true
    => e.deleted_at != null
}

ontology MyApp : Traits {
  -- Combine multiple abstract traits
  node Document : Named, Timestamped, SoftDeletable {
    content: String [required]
  }

  -- Rule for soft delete
  rule set_deleted_timestamp:
    d: Document WHERE d.deleted = true AND d.deleted_at = null
    =>
    SET d.deleted_at = now()
}
```

### Layer 0 Type Hierarchy

```
-- Layer 0 uses abstract+sealed for base meta-types
node _MetaType [abstract, sealed] {
  -- Base for all meta-level types
}

node _NodeType : _MetaType [sealed] {
  name: String [required, unique],
  abstract: Bool = false,
  sealed: Bool = false,
  doc: String?
}

node _EdgeType : _MetaType [sealed] {
  name: String [required, unique],
  arity: Int [required, >= 2],
  symmetric: Bool = false
}

node _TypeExpr [abstract, sealed] {
  -- Base for all type expressions
}

node _NamedTypeExpr : _TypeExpr [sealed] {
  ref_name: String [required]
}

node _OptionalTypeExpr : _TypeExpr [sealed] {
  -- Wraps another TypeExpr
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Instantiate abstract | `"Cannot instantiate abstract type 'Entity'"` |
| Inherit from sealed | `"Cannot inherit from sealed type '_NodeType'"` |
| Invalid modifier combination | (none - `abstract` and `sealed` can be combined) |
| Abstract with no subtypes | Warning: `"Abstract type 'X' has no concrete subtypes"` |

---

*End of Spec: Node Modifiers (abstract, sealed)*
