---
spec: any_type
version: "1.0"
status: stable
category: type
requires: []
priority: common
---

# Feature: Any Type

## 1. Overview

The `any` type matches any node type, enabling polymorphic edge signatures where an edge can connect to nodes of different types without union enumeration.

**Why needed:** Some relationships apply universally. "Any entity can be tagged." "Any node can have metadata." Without `any`, users must enumerate all types in a union, updating it whenever new types are added.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
PrimaryType = ... | AnyType

AnyType = "any"
```

### 2.2 Keywords Added

| Keyword | Context |
|---------|---------|
| `any` | Type expression |

### 2.3 Examples
```
-- Polymorphic tagging: any node can be tagged
edge tagged(entity: any, tag: Tag)

-- Universal metadata
edge has_metadata(target: any, meta: Metadata)

-- Audit trail for any entity
edge audit_log(subject: any, event: AuditEvent)
```

---

## 3. Semantics

### 3.1 Subtyping

Every node type is a subtype of `any`:
```
T <: any    (for all node types T)
```

This means any node can be passed where `any` is expected.

### 3.2 Usage Restrictions

`any` can only appear in:
- Edge signature parameters

`any` cannot appear in:
- Attribute types
- Node inheritance
- Variable declarations outside edge context
```
-- VALID
edge tagged(entity: any, tag: Tag)

-- INVALID
node Thing {
  ref: any       -- ERROR: 'any' cannot be attribute type
}

-- INVALID
node Universal : any { }  -- ERROR: cannot inherit from 'any'
```

### 3.3 Pattern Matching

When matching an edge with `any` parameter:
```
edge tagged(entity: any, tag: Tag)

MATCH tagged(x, t)
-- x can be any node type
-- No type constraint applied to x
```

To constrain the match, use explicit type:
```
MATCH x: Task, tagged(x, t)
-- x is constrained to Task
```

### 3.4 Type Inference

Variables bound through `any` parameters have type `any`:
```
MATCH tagged(entity, tag)
-- entity has type 'any'
-- entity.id works (id exists on all nodes)
-- entity.title fails (title not on all nodes)
```

To access type-specific attributes, use type checking (separate feature) or explicit binding.

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _AnyTypeExpr : _TypeExpr [sealed] {
  -- Matches any node type
}
```

### 4.2 Edge Types

None.

### 4.3 Constraints

None.

---

## 5. Compilation

### 5.1 Edge with Any Parameter
```
edge tagged(entity: any, tag: Tag)
```

Compiles to:
```
_EdgeType node:
  name: "tagged"
  arity: 2

_VarDef node (position 0):
  name: "entity"
  is_edge_var: false

_AnyTypeExpr node

_var_has_type edge:
  (entity_var_def, any_type_expr)

_VarDef node (position 1):
  name: "tag"
  is_edge_var: false

_NamedTypeExpr node:
  ref_name: "Tag"

_var_has_type edge:
  (tag_var_def, tag_type_expr)
```

---

## 6. Examples

### 6.1 Universal Tagging
```
ontology Taggable {
  node Tag {
    name: String [required, unique]
  }
  
  edge tagged(entity: any, tag: Tag) {
    tagged_at: Timestamp = now(),
    tagged_by: String?
  }
}

-- Usage: tag anything
LINK tagged(some_task, urgent_tag)
LINK tagged(some_person, vip_tag)
LINK tagged(some_project, featured_tag)
```

### 6.2 Audit Trail
```
ontology Auditable {
  node AuditEvent {
    action: String [required],
    timestamp: Timestamp = now(),
    actor: String
  }
  
  edge audited(subject: any, event: AuditEvent)
  
  -- Query: find all audit events for any entity
  -- MATCH audited(entity, event) WHERE entity.id = $id
}
```

### 6.3 Comments on Anything
```
ontology Commentable {
  node Comment {
    text: String [required],
    author: String [required],
    created_at: Timestamp = now()
  }
  
  edge commented_on(target: any, comment: Comment)
}
```

---

## 7. Errors

| Condition | Message |
|-----------|---------|
| `any` as attribute type | `"'any' cannot be used as attribute type"` |
| Inherit from `any` | `"Cannot inherit from 'any'"` |
| Access unknown attribute on `any` | `"Attribute 'x' not guaranteed on type 'any'"` |

---

*End of Feature: Any Type*