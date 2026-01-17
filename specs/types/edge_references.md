---
spec: edge_references
version: "1.0"
status: stable
category: type
requires: []
priority: common
---

# Feature: Edge References

## 1. Overview

Edge reference types (`edge<T>`) allow edges to target other edges instead of nodes, enabling higher-order relationships like confidence scores, provenance tracking, and temporal validity on relationships.

**Why needed:** Many domains require statements about relationships, not just entities. "This causation has 80% confidence." "This assignment was approved by X." Without edge references, such meta-information requires reifying edges as nodes, losing the natural relationship semantics.

---

## 2. Syntax

### 2.1 Grammar Additions
```ebnf
PrimaryType = ... | EdgeRefType

EdgeRefType = "edge" "<" (Identifier | "any") ">"
```

### 2.2 Keywords Added

None. `edge` is already a core keyword.

### 2.3 Examples
```
-- Edge targeting a specific edge type
edge confidence(about: edge<causes>) {
  level: Float [>= 0.0, <= 1.0]
}

-- Edge targeting any edge type
edge provenance(about: edge<any>) {
  source: String [required],
  recorded_at: Timestamp = now()
}

-- Creating higher-order edges
LINK causes(event1, event2) AS c
LINK confidence(c) { level = 0.85 }
```

---

## 3. Semantics

### 3.1 Edge Reference Types

`edge<T>` where T is an edge type name:
- Parameter accepts edge IDs of type T only
- Exact type match required (edges have no inheritance)

`edge<any>`:
- Parameter accepts any edge ID regardless of type

### 3.2 Type Checking

When creating edge with `edge<T>` parameter:
```
edge confidence(about: edge<causes>) { ... }

LINK confidence(some_edge) { ... }
```

Validation:
1. `some_edge` must be an edge ID (not node ID)
2. If T specified: edge must be exactly type T
3. If `any`: any edge type accepted

### 3.3 Creating Higher-Order Edges

Use edge binding (`AS`) to capture edge ID, then reference it:
```
LINK causes(e1, e2) AS c          -- c is the edge ID
LINK confidence(c) { level = 0.9 } -- higher-order edge
```

### 3.4 Querying Higher-Order Edges
```
MATCH
  e1: Event, e2: Event,
  causes(e1, e2) AS c,
  confidence(c) AS conf
WHERE conf.level > 0.5
RETURN e1, e2, conf.level
```

### 3.5 Cascade on Unlink

When a base edge is unlinked, all higher-order edges referencing it are **automatically unlinked**:
```
LINK causes(e1, e2) AS c
LINK confidence(c) { level = 0.8 }
LINK provenance(c) { source = "sensor" }

UNLINK c
-- Both confidence and provenance edges automatically unlinked
```

This cascade:
- Is recursive (edges about edges about edges...)
- Cannot be disabled
- Maintains referential integrity

### 3.6 Variable Marking

When a signature parameter has type `edge<T>`, the compiled `_VarDef` has `is_edge_var: true`.

---

## 4. Layer 0 Additions

### 4.1 Node Types
```
node _EdgeRefTypeExpr : _TypeExpr [sealed] {
  ref_name: String?    -- edge type name, null means 'any'
}
```

### 4.2 Edge Types

None.

### 4.3 Constraints
```
constraint _edge_ref_type_valid:
  t: _EdgeRefTypeExpr
  WHERE t.ref_name != null
  => EXISTS(et: _EdgeType WHERE et.name = t.ref_name)
```

---

## 5. Compilation

### 5.1 Edge Type with Edge Reference
```
edge confidence(about: edge<causes>) {
  level: Float
}
```

Compiles to:
```
_EdgeType node:
  name: "confidence"
  arity: 1

_VarDef node:
  name: "about"
  is_edge_var: true       -- marks as edge reference

_edge_has_position edge:
  (confidence_type, about_var) { position: 0 }

_EdgeRefTypeExpr node:
  ref_name: "causes"

_var_has_type edge:
  (about_var, edge_ref_type_expr)

-- Attribute compilation as normal
_AttributeDef node:
  name: "level"
  ...
```

### 5.2 Edge Reference to Any
```
edge provenance(about: edge<any>) { ... }
```

Compiles with:
```
_EdgeRefTypeExpr node:
  ref_name: null          -- null means any edge
```

---

## 6. Examples

### 6.1 Confidence Scoring
```
ontology ConfidentGraph {
  node Event {
    name: String [required],
    timestamp: Timestamp
  }
  
  edge causes(from: Event, to: Event) {
    mechanism: String?
  }
  
  edge confidence(about: edge<causes>) {
    level: Float [>= 0.0, <= 1.0],
    assessor: String?,
    assessed_at: Timestamp = now()
  }
  
  constraint high_confidence_needs_assessor:
    e1: Event, e2: Event,
    causes(e1, e2) AS c,
    confidence(c) AS conf
    WHERE conf.level > 0.9
    => conf.assessor != null
}
```

### 6.2 Universal Provenance
```
ontology Provenance {
  edge provenance(about: edge<any>) {
    source: String [required],
    method: String?,
    recorded_at: Timestamp = now()
  }
}

-- Can attach provenance to any edge
LINK causes(e1, e2) AS c
LINK provenance(c) { source = "sensor_array_1" }

LINK assigned_to(task, person) AS a
LINK provenance(a) { source = "manager_approval" }
```

### 6.3 Temporal Validity
```
ontology TemporalEdges {
  edge valid_during(about: edge<any>) {
    valid_from: Timestamp [required],
    valid_until: Timestamp?
  }
  
  -- Query: find currently valid relationships
  -- MATCH
  --   some_edge(...) AS e,
  --   valid_during(e) AS v
  -- WHERE v.valid_from <= now() 
  --   AND (v.valid_until = null OR v.valid_until > now())
}
```

### 6.4 Stacked Higher-Order
```
ontology MetaMeta {
  edge causes(a: Event, b: Event)
  
  edge confidence(about: edge<causes>) {
    level: Float
  }
  
  edge confidence_source(about: edge<confidence>) {
    method: String [required]
  }
}

-- Three levels:
LINK causes(e1, e2) AS c
LINK confidence(c) { level = 0.8 } AS conf
LINK confidence_source(conf) { method = "bayesian_inference" }
```

---

## 7. Errors

| Code | Condition | Message |
|------|-----------|---------|
| E3501 | `edge<T>` where T is not edge type | `"'T' is not an edge type"` |
| E3502 | Node ID passed to edge reference | `"Expected edge ID, got node ID"` |
| E3503 | Wrong edge type for `edge<T>` | `"Expected edge of type 'T', got 'U'"` |
| E3504 | Reference to unlinked edge | `"Edge ID does not exist"` |

---

*End of Feature: Edge References*