---
spec: ontology
version: "1.0"
status: draft
category: declaration
capability: schema
requires: []
priority: essential
---

# Spec: Ontology Declaration

## Overview

The `ontology` declaration defines a named schema container that groups related type, edge, constraint, and rule declarations. Ontologies support inheritance, enabling schema composition and extension. When no explicit ontology declaration is present, the file defines an anonymous ontology that implicitly inherits from Layer0.

## Syntax

### Grammar

```ebnf
OntologyDecl = "ontology" Identifier InheritanceClause? "{" Declaration* "}"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*

QualifiedIdentifier = Identifier ("." Identifier)*

Declaration =
    TypeAliasDecl
  | NodeTypeDecl
  | EdgeTypeDecl
  | ConstraintDecl
  | RuleDecl
```

### Keywords

| Keyword | Context |
|---------|---------|
| `ontology` | Declaration |

### Examples

```
-- Simple ontology (implicit Layer0 inheritance)
ontology TaskManagement {
  node Task { title: String [required] }
}

-- Explicit single inheritance
ontology Physics : Layer0 {
  node Particle { mass: Float [required] }
}

-- Multiple inheritance
ontology GameWorld : Physics, Social {
  node Character : Person {
    health: Int [>= 0] = 100
  }
}

-- File without ontology declaration (anonymous, inherits Layer0)
node SimpleTask { name: String }
```

## Semantics

### Implicit Ontology

If no `ontology` declaration is present, the file defines an **anonymous ontology** that implicitly inherits from Layer0:

```
-- This file:
node Task { title: String }
edge depends_on(a: Task, b: Task)

-- Is equivalent to:
ontology Anonymous : Layer0 {
  node Task { title: String }
  edge depends_on(a: Task, b: Task)
}
```

### Inheritance Semantics

When ontology B inherits from ontology A (`ontology B : A`):

| Inherited Element | Behavior |
|-------------------|----------|
| Node types | Available in B, can be extended via node inheritance |
| Edge types | Available in B, can be used in patterns and declarations |
| Type aliases | Available in B, can be referenced |
| Constraints | Active in B, enforced on B's instances |
| Rules | Active in B, fire on B's pattern matches |

### Additive-Only Inheritance

Inheritance is **additive only**. Child ontologies:
- **Can** add new types, edges, constraints, and rules
- **Can** extend inherited node types (add attributes via node inheritance)
- **Cannot** remove inherited declarations
- **Cannot** modify inherited declarations
- **Cannot** weaken inherited constraints

### Multiple Inheritance

An ontology can inherit from multiple parents:

```
ontology GameWorld : Physics, Social {
  -- Has access to all types from Physics and Social
}
```

**Name conflict resolution:**
- If two parents define the same name, it's a compile error
- Exception: if both inherit the same definition from a common ancestor (diamond), it appears once

### Diamond Inheritance

```
ontology A { node Base { x: Int } }
ontology B : A { }
ontology C : A { }
ontology D : B, C { }  -- D has one copy of Base, not two
```

### Qualified References

Types from inherited ontologies can be referenced by their simple name (if unambiguous) or qualified name:

```
ontology MyApp : external.UserSystem, external.Billing {
  node Order {
    customer: external.UserSystem.User,  -- qualified reference
    total: Float
  }
}
```

### Inheritance Order

When multiple parents define rules or constraints at the same priority level:
1. Parents are processed in declaration order (left to right)
2. Earlier parents' rules/constraints take precedence

## Layer 0

### Nodes

```
node _Ontology [sealed] {
  name: String [required],
  version: String?,
  doc: String?
}
```

### Edges

```
edge _ontology_inherits(child: _Ontology, parent: _Ontology) [
  acyclic,
  on_kill_target: prevent
]

edge _ontology_declares(ontology: _Ontology, declaration: _MetaType)
```

### Constraints

```
constraint _no_ontology_inheritance_cycle:
  o: _Ontology, _ontology_inherits+(o, o)
  => false
```

## Examples

### Schema Composition

```
-- Base ontology for common patterns
ontology Common {
  node Named {
    name: String [required, length: 1..200]
  }

  node Timestamped {
    created_at: Timestamp [required] = now(),
    updated_at: Timestamp?
  }
}

-- Domain-specific ontology
ontology TaskManagement : Common {
  node Task : Named, Timestamped {
    status: String [in: ["todo", "in_progress", "done"]] = "todo",
    priority: Int [0..10] = 5
  }

  node Project : Named, Timestamped {
    deadline: Timestamp?
  }

  edge belongs_to(task: Task, project: Project) [task -> 1]
}
```

### Layered Architecture

```
-- Core domain types
ontology Domain {
  node User { email: String [required, unique, format: email] }
  node Document { content: String [required] }
}

-- Add access control
ontology SecureDomain : Domain {
  node Permission {
    level: String [in: ["read", "write", "admin"]]
  }

  edge has_permission(user: User, doc: Document) {
    permission: Permission [required]
  }

  constraint users_need_permission:
    u: User, d: Document
    WHERE NOT EXISTS(has_permission(u, d))
    => false
}

-- Add audit logging
ontology AuditedDomain : SecureDomain {
  node AuditEvent {
    action: String [required],
    timestamp: Timestamp = now()
  }

  edge audit_log(subject: User, event: AuditEvent)

  rule log_document_access:
    u: User, d: Document, has_permission(u, d)
    WHERE u.last_access_logged = null OR u.last_access_logged < now() - 60000
    =>
    SPAWN e: AuditEvent { action = "access" },
    LINK audit_log(u, e),
    SET u.last_access_logged = now()
}
```

### Extending External Ontologies

```
-- Import and extend a third-party ontology
ontology MyApp : vendor.CRM {
  -- Add custom attributes to vendor type
  node Customer : vendor.CRM.Customer {
    loyalty_tier: String [in: ["bronze", "silver", "gold"]] = "bronze",
    internal_notes: String?
  }

  -- Add custom edge type
  edge managed_by(customer: Customer, rep: vendor.CRM.SalesRep)
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Circular inheritance | `"Ontology 'A' cannot inherit from itself (directly or indirectly)"` |
| Unknown parent | `"Unknown ontology 'X' in inheritance clause"` |
| Name conflict | `"Name 'Y' is defined in multiple parent ontologies: A, B"` |
| Modify inherited | `"Cannot modify inherited declaration 'Z' from ontology 'A'"` |
| Remove inherited | `"Cannot remove inherited declaration 'Z' from ontology 'A'"` |

---

*End of Spec: Ontology Declaration*
