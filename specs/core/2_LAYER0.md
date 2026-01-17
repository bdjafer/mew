# MEW Language Specification

## Part II: Layer 0 Core

**Version:** 2.1
**Status:** Stable Foundation
**Scope:** Minimal kernel meta-schema and extension mechanisms

---

# 0. Overview

## 0.1 Purpose

Layer 0 is the **kernel** of the MEW type system. It defines:

1. **Core meta-types** — The minimal types needed to describe any ontology
2. **Extension points** — Abstract types that feature modules extend
3. **Core constraints** — Invariants that always hold

Layer 0 is **hardcoded** into the engine. It cannot be modified by user ontologies.

## 0.2 Design Principle: Minimal Core

The core contains only what is **necessary for compilation**:

| Construct | Why Core |
|-----------|----------|
| Node types | Entities are fundamental to graphs |
| Edge types | Relationships are fundamental to graphs |
| Attributes | Properties give types meaning |
| Constraints | Validation makes compilation meaningful |
| Patterns | Constraints need patterns to match against |
| Expressions | Constraint conditions need expressions |

**Not in core:**
- Rules (reactive behavior, not validity)
- Higher-order edges (advanced feature)
- Transitive patterns (query optimization)
- Soft constraints (variant of hard constraints)

## 0.3 Document Scope

This document contains **only** the kernel. Feature-specific types are defined in `features/`.

## 0.4 Reserved Namespace

| Prefix | Reserved For |
|--------|--------------|
| `_` | Layer 0 and feature module types |
| `__` | Engine internal (not exposed) |

---

# 1. Scalar Types

Primitive value types for attributes. Built into the engine.

| Type | Description | Size |
|------|-------------|------|
| `String` | UTF-8 text | Variable |
| `Int` | Signed integer | 64 bits |
| `Float` | IEEE 754 double | 64 bits |
| `Bool` | true or false | 1 bit |
| `Timestamp` | Milliseconds since Unix epoch | 64 bits |
| `ID` | Opaque identifier | Implementation-defined |

## 1.1 Default Value Serialization

The `_AttributeDef.default_value` field stores defaults as JSON-encoded strings:

| Type | Example | Serialized |
|------|---------|------------|
| Int | 42 | `"42"` |
| Float | 3.14 | `"3.14"` |
| Bool | true | `"true"` |
| String | "hello" | `"\"hello\""` |
| Timestamp | 1704067200000 | `"1704067200000"` |
| (none) | — | `null` |

## 1.2 Dynamic Defaults

Dynamic defaults use a `$` prefix:

| Expression | Serialized |
|------------|------------|
| `now()` | `"$now()"` |
| `now() + 86400000` | `"$now() + 86400000"` |

Evaluated at entity creation time.

---

# 2. Core Node Types

## 2.1 Meta-Types

The fundamental types for describing ontology structure.
```
node _NodeType [sealed] {
  name: String [required, unique],
  abstract: Bool = false,
  sealed: Bool = false,
  doc: String?
}

node _EdgeType [sealed] {
  name: String [required, unique],
  arity: Int [required],
  doc: String?
}

node _AttributeDef [sealed] {
  name: String [required],
  scalar_type: String?,           -- "String", "Int", etc. (null if node reference)
  required: Bool = false,
  unique: Bool = false,
  indexed: String = "none",       -- "none", "asc", "desc"
  default_value: String?,         -- Serialized (see §1.1-1.2)
  doc: String?
}

node _ConstraintDef [sealed] {
  name: String [required],
  hard: Bool = true,
  message: String?,
  doc: String?
}
```

## 2.2 Type Expressions

Types for representing complex type signatures.
```
node _TypeExpr [abstract, sealed, extension-point] {
  -- Base for all type expressions
}

node _ScalarTypeExpr : _TypeExpr [sealed] {
  scalar_type: String [required]  -- "String", "Int", "Float", "Bool", "Timestamp"
}

node _NamedTypeExpr : _TypeExpr [sealed] {
  ref_name: String [required]     -- Name of referenced NodeType
}

node _OptionalTypeExpr : _TypeExpr [sealed] {
  -- Wraps another TypeExpr to make it optional (T?)
}

node _UnionTypeExpr : _TypeExpr [sealed] {
  -- Combines TypeExprs with OR semantics (A | B)
}
```

## 2.3 Patterns

Patterns match graph structure for constraints.
```
node _PatternDef [sealed] {
  -- A pattern is a template for matching graph structure
}

node _VarDef [sealed] {
  name: String [required],
  is_edge_var: Bool = false       -- true if binds to edge, false for node
}

node _EdgePattern [sealed] {
  negated: Bool = false           -- If true, requires edge NOT to exist
}
```

## 2.4 Expressions

Expressions compute values in constraint conditions.
```
node _Expr [abstract, sealed, extension-point] {
  -- Base for all expressions
}

node _LiteralExpr : _Expr [sealed] {
  value_type: String [required],  -- "String", "Int", "Float", "Bool", "Null"
  value_string: String [required] -- Serialized value
}

node _VarRefExpr : _Expr [sealed] {
  var_name: String [required]
}

node _AttrAccessExpr : _Expr [sealed] {
  attr_name: String [required]
  -- Base expression linked via _attr_access_base edge
}

node _BinaryOpExpr : _Expr [sealed] {
  operator: String [required]
  -- Operators: "=", "!=", "<", ">", "<=", ">=",
  --            "+", "-", "*", "/", "%", "and", "or"
}

node _UnaryOpExpr : _Expr [sealed] {
  operator: String [required]     -- "not", "-"
}

node _CallExpr : _Expr [sealed] {
  function_name: String [required]
  -- Arguments linked via _call_arg edges
}
```

## 2.5 Ontology Structure
```
node _Ontology [sealed] {
  name: String [required],
  version: String?,
  doc: String?
}

node _Import [sealed] {
  ontology_name: String [required],
  alias: String?
}
```

---

# 3. Extension Points

Extension points are abstract types that feature modules extend.

## 3.1 Expression Extension Point
```
node _Expr [abstract, sealed, extension-point]
```

Feature modules may add sealed subtypes of `_Expr`.

**Core expression types:**

| Type | Purpose |
|------|---------|
| `_LiteralExpr` | Constant values |
| `_VarRefExpr` | Variable references |
| `_AttrAccessExpr` | Attribute access |
| `_BinaryOpExpr` | Binary operations |
| `_UnaryOpExpr` | Unary operations |
| `_CallExpr` | Function calls |

**Feature-added expressions:** See Extension Registry (§7).

## 3.2 Type Expression Extension Point
```
node _TypeExpr [abstract, sealed, extension-point]
```

Feature modules may add sealed subtypes of `_TypeExpr`.

**Core type expressions:**

| Type | Purpose |
|------|---------|
| `_ScalarTypeExpr` | Primitive types |
| `_NamedTypeExpr` | Node type references |
| `_OptionalTypeExpr` | Optional wrapper (T?) |
| `_UnionTypeExpr` | Union types (A \| B) |

**Feature-added type expressions:** See Extension Registry (§7).

---

# 4. Core Edge Types

## 4.1 Type Structure Edges
```
edge _type_inherits(
  child: _NodeType,
  parent: _NodeType
) {}

edge _type_has_attribute(
  owner: _NodeType | _EdgeType,
  attr: _AttributeDef
) {}

edge _attr_has_type(
  attr: _AttributeDef,
  type_expr: _TypeExpr
) {}
```

## 4.2 Edge Signature Edges
```
edge _edge_has_position(
  edge_type: _EdgeType,
  var: _VarDef
) {
  position: Int [required]        -- 0-indexed
}

edge _var_has_type(
  var: _VarDef,
  type_expr: _TypeExpr
) {}
```

## 4.3 Type Expression Edges
```
edge _optional_inner(
  optional: _OptionalTypeExpr,
  inner: _TypeExpr
) {}

edge _union_member(
  union: _UnionTypeExpr,
  member: _TypeExpr
) {
  position: Int [required]        -- 0 for left, 1 for right
}
```

## 4.4 Pattern Structure Edges
```
edge _pattern_has_node_var(
  pattern: _PatternDef,
  var: _VarDef
) {}

edge _pattern_has_edge_var(
  pattern: _PatternDef,
  var: _VarDef
) {}

edge _pattern_has_edge_pattern(
  pattern: _PatternDef,
  edge_pattern: _EdgePattern
) {}

edge _edge_pattern_type(
  edge_pattern: _EdgePattern,
  edge_type: _EdgeType
) {}

edge _edge_pattern_target(
  edge_pattern: _EdgePattern,
  var: _VarDef
) {
  position: Int [required]
}

edge _edge_pattern_alias(
  edge_pattern: _EdgePattern,
  var: _VarDef
) {}

edge _pattern_has_condition(
  pattern: _PatternDef,
  condition: _Expr
) {}
```

## 4.5 Constraint Edges
```
edge _constraint_has_pattern(
  constraint: _ConstraintDef,
  pattern: _PatternDef
) {}

edge _constraint_has_condition(
  constraint: _ConstraintDef,
  condition: _Expr
) {}
```

## 4.6 Expression Structure Edges
```
edge _attr_access_base(
  expr: _AttrAccessExpr,
  base: _Expr
) {}

edge _binary_left(
  expr: _BinaryOpExpr,
  left: _Expr
) {}

edge _binary_right(
  expr: _BinaryOpExpr,
  right: _Expr
) {}

edge _unary_operand(
  expr: _UnaryOpExpr,
  operand: _Expr
) {}

edge _call_arg(
  expr: _CallExpr,
  arg: _Expr
) {
  position: Int [required]
}
```

## 4.7 Ontology Structure Edges
```
edge _ontology_declares_type(
  ontology: _Ontology,
  type: _NodeType | _EdgeType
) {}

edge _ontology_declares_constraint(
  ontology: _Ontology,
  constraint: _ConstraintDef
) {}

edge _ontology_imports(
  ontology: _Ontology,
  import: _Import
) {}
```

---

# 5. Core Constraints

These constraints are always enforced by the engine.

## 5.1 Naming Constraints
```
constraint _unique_node_type_names:
  t1: _NodeType, t2: _NodeType
  WHERE t1.id != t2.id
  => t1.name != t2.name

constraint _unique_edge_type_names:
  t1: _EdgeType, t2: _EdgeType
  WHERE t1.id != t2.id
  => t1.name != t2.name

constraint _unique_attribute_per_type:
  owner: _NodeType | _EdgeType,
  a1: _AttributeDef, a2: _AttributeDef,
  _type_has_attribute(owner, a1),
  _type_has_attribute(owner, a2)
  WHERE a1.id != a2.id
  => a1.name != a2.name
```

## 5.2 Inheritance Constraints
```
constraint _no_inheritance_cycle:
  t: _NodeType, _type_inherits+(t, t)
  => false

constraint _sealed_not_inherited:
  parent: _NodeType, child: _NodeType,
  _type_inherits(child, parent)
  => parent.sealed = false
```

## 5.3 Edge Type Constraints
```
constraint _edge_arity_matches_positions:
  e: _EdgeType, v: _VarDef,
  _edge_has_position(e, v) AS pos
  => pos.position >= 0 AND pos.position < e.arity

constraint _edge_positions_unique:
  e: _EdgeType, v1: _VarDef, v2: _VarDef,
  _edge_has_position(e, v1) AS p1,
  _edge_has_position(e, v2) AS p2
  WHERE p1.position = p2.position
  => v1.id = v2.id
```

## 5.4 Type Expression Constraints
```
constraint _optional_has_inner:
  t: _OptionalTypeExpr
  => EXISTS(inner: _TypeExpr, _optional_inner(t, inner))

constraint _union_has_members:
  t: _UnionTypeExpr
  => EXISTS(m: _TypeExpr, _union_member(t, m) AS u WHERE u.position = 0)
     AND EXISTS(m: _TypeExpr, _union_member(t, m) AS u WHERE u.position = 1)
```

## 5.5 Expression Constraints
```
constraint _binary_has_operands:
  e: _BinaryOpExpr
  => EXISTS(l: _Expr, _binary_left(e, l))
     AND EXISTS(r: _Expr, _binary_right(e, r))

constraint _unary_has_operand:
  e: _UnaryOpExpr
  => EXISTS(o: _Expr, _unary_operand(e, o))

constraint _attr_access_has_base:
  e: _AttrAccessExpr
  => EXISTS(b: _Expr, _attr_access_base(e, b))
```

## 5.6 Procedural Constraints

Enforced by engine, not as patterns:

| Constraint | Error |
|------------|-------|
| Abstract types cannot be instantiated | `"Cannot instantiate abstract type 'X'"` |
| Edge positions must be complete (0..arity-1) | `"EdgeType 'X' missing position N"` |
| Pattern variable names must be unique | `"Duplicate variable 'x' in pattern"` |
| Protected types cannot be created directly | `"Cannot create protected type '_X'"` |

---

# 6. Core Operations

## 6.1 Primitive Operations

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `SPAWN` | `(type, attrs) → ID` | Create node |
| `KILL` | `(id) → Bool` | Remove node |
| `LINK` | `(type, targets, attrs) → ID` | Create edge |
| `UNLINK` | `(id) → Bool` | Remove edge |
| `SET` | `(id, attr, value) → Bool` | Modify attribute |

## 6.2 Query Operations

| Operation | Description |
|-----------|-------------|
| `findNodes(type?, filter?)` | Find nodes matching criteria |
| `findEdges(type?, filter?)` | Find edges matching criteria |
| `matchPattern(pattern)` | Find all pattern matches |

## 6.3 Constraint Checking

Constraints are checked at transaction commit:

1. Identify constraints affected by pending mutations
2. For each affected constraint:
   - Execute pattern match
   - Evaluate condition for each match
   - If condition false and hard=true → reject transaction
   - If condition false and hard=false → emit warning
3. If all hard constraints pass → commit


---

# 7. Type Checking

## 7.1 Subtyping Rules
```
T <: T                                    -- reflexive
T <: U, U <: V  ⟹  T <: V                -- transitive
Child <: Parent  (if _type_inherits)      -- inheritance
T <: T | U                                -- union left
U <: T | U                                -- union right
T <: T?                                   -- optional
null <: T?                                -- null in optional
```

## 7.2 Edge Type Checking

When creating edge of type E with targets [t1, t2, ...]:

For each position i:
- Let expected = type of signature variable at position i
- Let actual = type of ti
- Require: actual <: expected

## 7.3 Attribute Type Checking

When setting attribute A on entity of type T:
- Look up AttributeDef for A on T (including inherited)
- Verify value type matches attribute type

---

*End of Part II: Layer 0 Core*