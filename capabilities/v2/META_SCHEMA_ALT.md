# MEW v2: Core Specification

**Version:** 2.0  
**Status:** Foundation  
**Principle:** Everything that follows is derived from first principles. What is here is forever valid. What is not here can be built on top.

---

# Part I: The Unified Model

## 1.1 The Core Insight

A graph consists of **things** and **relationships between things**. But in a higher-order graph, relationships can themselves be related to. Therefore, relationships are also things.

**Everything is a thing.** The distinction between "node" and "edge" is not ontological — it is about **role**:
- Some things exist independently (arity 0)
- Some things exist as connections between other things (arity > 0)

Both have identity. Both have type. Both have attributes. Both are the same kind of primitive.

We call this primitive a **Glyph**.

## 1.2 The Glyph

```
Glyph {
  id: GlyphId                    // unique within world
  type: GlyphTypeId              // what kind of thing
  targets: [GlyphId; arity]      // what it connects (empty if arity = 0)
  attributes: Map<AttrId, Value> // properties
}
```

**That's it.** This is the only data primitive in MEW.

- When `arity = 0`: the Glyph exists independently. Users call this a "node."
- When `arity > 0`: the Glyph exists as a relationship. Users call this an "edge."
- When `arity > 0` and targets include other edges: users call this "higher-order."

These are all the same thing. The DSL provides friendly syntax; the core has one primitive.

## 1.3 The GlyphType

Every Glyph has a type. Types are also Glyphs (the system is self-describing).

```
GlyphType {
  name: String                   // unique within ontology
  arity: Nat                     // 0 = "node", n > 0 = "edge"
  parents: [GlyphTypeId]         // inheritance (same arity required)
  signature: [GlyphTypeId; arity] // type constraint per position
  attributes: [AttributeDef]     // declared attributes
  abstract: Bool                 // if true, cannot instantiate directly
  sealed: Bool                   // if true, cannot inherit from
  has_interior: Bool             // if true, instances have nested worlds
}
```

**Inheritance rules:**
1. Subtypes must have the **same arity** as supertypes
2. Signature positions are **covariant**: subtype's position type must be subtype of supertype's position type
3. Attributes are **additive**: subtypes inherit all supertype attributes and may add more

```
// Example: edge inheritance
GlyphType { name: "relates", arity: 2, signature: [Entity, Entity] }
GlyphType { name: "causes", arity: 2, signature: [Event, Event], parents: [relates] }
// Valid because Event <: Entity at both positions
```

## 1.4 Values and Scalars

Attributes hold **Values**. The core value types:

| Type | Description | Size |
|------|-------------|------|
| `Null` | Absence of value | 0 |
| `Bool` | true or false | 1 bit |
| `Int` | Signed integer | 64 bits |
| `Float` | IEEE 754 double | 64 bits |
| `String` | UTF-8 text | Variable |
| `Timestamp` | Milliseconds since epoch | 64 bits |
| `GlyphId` | Reference to a Glyph | 64 bits |

**GlyphId as value:** An attribute can hold a GlyphId. This is **data**, not **structure**. The Glyph doesn't "connect to" the referenced Glyph — it "knows about" it. This distinction is critical for world boundaries.

## 1.5 The Three Mutations

All state changes reduce to three primitives:

| Operation | Signature | Effect |
|-----------|-----------|--------|
| `CREATE` | `(type, targets, attrs) → GlyphId` | Bring a new Glyph into existence |
| `DELETE` | `(id) → ()` | Remove a Glyph from existence |
| `SET` | `(id, attr, value) → ()` | Change an attribute value |

**Derived operations (DSL sugar):**
- `SPAWN` = CREATE with arity 0
- `KILL` = DELETE
- `LINK` = CREATE with arity > 0
- `UNLINK` = DELETE on edge

**Idempotence:** `SET x.a = v` when `x.a` is already `v` is a no-op. No mutation occurs. This prevents spurious rule firing.

---

# Part II: Layer Architecture

## 2.1 The Full Stack

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   USERLAND                                                          │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   DATA              Instances of user types                         │
│                     e.g., Task#42, causes#17                        │
│                                                                      │
├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┤
│                                                                      │
│   ONTOLOGY          User-defined types, constraints, rules          │
│                     e.g., Task, Event, causes, triggers             │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   KERNEL (sealed)                                                   │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   STDLIB            Patterns, Expressions, Mutations                │
│                     Constraints, Rules, Policies, Watches           │
│                                                                      │
├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┤
│                                                                      │
│   META-SCHEMA       Types that describe types                       │
│                     _GlyphType, _AttributeDef, _TypeExpr            │
│                                                                      │
├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┤
│                                                                      │
│   BOOTSTRAP         The fixed point: _GlyphType : _GlyphType        │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   RUNTIME           Execution engine (not MEW)                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Layer Definitions

| Layer | Name | Contents | Mutability |
|-------|------|----------|------------|
| — | **Runtime** | Rust code, execution engine | Not MEW |
| 0a | **Bootstrap** | `_GlyphType : _GlyphType` | Hardcoded |
| 0b | **Meta-Schema** | Type system primitives | Sealed |
| 1 | **Stdlib** | Patterns, Expressions, Behaviors | Sealed |
| 2 | **Ontology** | User-defined types | User-defined |
| 3 | **Data** | Instances | User-controlled |

**Runtime:** The interpreter/executor. Rust code implementing storage, pattern matching, tick execution. Cannot be expressed in MEW.

**Bootstrap:** The fixed point that breaks the chicken-egg problem. `_GlyphType` typed as itself.

**Meta-Schema:** Types that describe types. Built using Bootstrap. Defines what a type IS.

**Stdlib:** Built-in capabilities. Built using Meta-Schema. Defines what you can DO.

**Ontology:** User-defined types. Built using Meta-Schema + Stdlib. Defines domain concepts.

**Data:** Instances of user types. The actual graph content.

---

# Part III: Bootstrap

## 3.1 The Fixed Point

To have types, you need a type for types. But what's the type of *that*?

The answer: it types itself.

```
_GlyphType {
  id: #0
  type: #0                               // points to itself
  targets: []                            // arity 0
  attributes: {
    name: "_GlyphType",
    arity: 0,
    abstract: false,
    sealed: true,
    has_interior: false
  }
}
```

This single self-referential Glyph is **hardcoded** in the Runtime. It cannot be "created" in the normal sense — it must exist for anything else to exist.

**The Bootstrap is:**
- One Glyph: `_GlyphType`
- One fact: `_GlyphType.type = _GlyphType`
- The foundation upon which everything else is built

---

# Part IV: Meta-Schema

## 4.1 Purpose

The Meta-Schema defines **what a type is**. It is the type system's type system.

All Meta-Schema types are:
- Sealed (cannot be inherited by user types)
- Prefixed with `_` (reserved namespace)

## 4.2 Core Meta-Types

### 4.2.1 GlyphType

The type of all types.

```
_GlyphType [arity: 0, sealed] {
  name: String [required, unique]
  arity: Int [required, >= 0]
  abstract: Bool = false
  sealed: Bool = false
  has_interior: Bool = false
}
```

### 4.2.2 AttributeDef

Defines an attribute on a type.

```
_AttributeDef [arity: 0, sealed] {
  name: String [required]
  required: Bool = false
  unique: Bool = false
  default_value: String?                 // serialized, see §4.5
}
```

### 4.2.3 VarDef

Defines a variable (for signatures and patterns).

```
_VarDef [arity: 0, sealed] {
  name: String [required]
}
```

### 4.2.4 Ontology

Container for a set of related type definitions.

```
_Ontology [arity: 0, sealed] {
  name: String [required, unique]
  version: String?
}
```

### 4.2.5 Import

Declares a dependency on another ontology.

```
_Import [arity: 0, sealed] {
  source_ontology: String [required]
  alias: String?
}
```

## 4.3 Type Expressions

Type expressions describe complex types (optionals, unions, references).

```
_TypeExpr [arity: 0, abstract, sealed] {
  // Base for all type expressions
}

_ScalarTypeExpr : _TypeExpr [sealed] {
  scalar_type: String [required]         // "Bool", "Int", "Float", "String", 
                                         // "Timestamp", "GlyphId"
}

_NamedTypeExpr : _TypeExpr [sealed] {
  type_name: String [required]           // name of referenced GlyphType
}

_OptionalTypeExpr : _TypeExpr [sealed] {
  // Wraps another TypeExpr to make it optional (T?)
}

_UnionTypeExpr : _TypeExpr [sealed] {
  // Combines TypeExprs with OR semantics (A | B)
}

_ListTypeExpr : _TypeExpr [sealed] {
  // List of elements (List<T>)
}
```

## 4.4 Structure Edges

These edges define relationships between meta-types.

```
// Type inheritance
_type_inherits [arity: 2, sealed] (
  child: _GlyphType,
  parent: _GlyphType
)

// Type has attribute
_type_has_attribute [arity: 2, sealed] (
  owner: _GlyphType,
  attr: _AttributeDef
)

// Attribute's type
_attr_type [arity: 2, sealed] (
  attr: _AttributeDef,
  type_expr: _TypeExpr
)

// Signature position (for arity > 0 types)
_type_has_position [arity: 2, sealed] (
  type: _GlyphType,
  var: _VarDef
) {
  position: Int [required]               // 0-indexed
}

// Variable's type constraint
_var_type [arity: 2, sealed] (
  var: _VarDef,
  type_expr: _TypeExpr
)

// Optional's inner type
_optional_inner [arity: 2, sealed] (
  optional: _OptionalTypeExpr,
  inner: _TypeExpr
)

// Union members
_union_member [arity: 2, sealed] (
  union: _UnionTypeExpr,
  member: _TypeExpr
) {
  position: Int [required]               // 0 = left, 1 = right
}

// List element type
_list_element [arity: 2, sealed] (
  list: _ListTypeExpr,
  element: _TypeExpr
)

// Ontology declares type
_ontology_declares [arity: 2, sealed] (
  ontology: _Ontology,
  type: _GlyphType
)

// Ontology imports another
_ontology_imports [arity: 2, sealed] (
  ontology: _Ontology,
  import: _Import
)
```

## 4.5 Default Value Serialization

The `_AttributeDef.default_value` field stores defaults as JSON-encoded strings:

| Type | Example | Serialized |
|------|---------|------------|
| Int | 42 | `"42"` |
| Float | 3.14 | `"3.14"` |
| Bool | true | `"true"` |
| String | "hello" | `"\"hello\""` |
| Timestamp | 1704067200000 | `"1704067200000"` |
| Null | — | `null` |

**Dynamic defaults** use a `$` prefix:

| Expression | Serialized |
|------------|------------|
| `now()` | `"$now()"` |

Evaluated at Glyph creation time.

## 4.6 Meta-Constraints

These constraints on the Meta-Schema are **hardcoded in Runtime**:

| Constraint | Description |
|------------|-------------|
| Unique type names | No two types in same ontology share a name |
| No inheritance cycles | `_type_inherits+(t, t)` is forbidden |
| Sealed not inherited | Cannot inherit from sealed types |
| Same arity inheritance | Subtypes must have same arity as supertypes |
| Covariant signatures | Position types must be subtypes of parent's position types |
| Complete positions | Arity-n type must have positions 0..n-1 defined |
| Unique attribute names | No duplicate attribute names per type |

---

# Part V: Stdlib — Pattern Language

## 5.1 Purpose

The Pattern Language defines **how to match graph structure**. Patterns are the foundation for queries, constraints, rules, policies, and watches.

## 5.2 Pattern Definition

```
_PatternDef [arity: 0, sealed] {
  // A template graph for matching
}

_EdgePattern [arity: 0, sealed] {
  negated: Bool = false                  // if true, edge must NOT exist
  closure: String = "none"               // "none", "transitive", "reflexive_transitive"
}
```

**Closure modes:**
- `none`: direct edge only — `edge(a, b)`
- `transitive`: one or more hops — `edge+(a, b)`
- `reflexive_transitive`: zero or more hops — `edge*(a, b)`

## 5.3 Pattern Structure Edges

```
// Pattern declares a variable
_pattern_has_var [arity: 2, sealed] (
  pattern: _PatternDef,
  var: _VarDef
)

// Pattern has edge pattern
_pattern_has_edge_pattern [arity: 2, sealed] (
  pattern: _PatternDef,
  edge_pattern: _EdgePattern
)

// Pattern has condition
_pattern_has_condition [arity: 2, sealed] (
  pattern: _PatternDef,
  condition: _Expr
)

// Edge pattern's type
_edge_pattern_type [arity: 2, sealed] (
  edge_pattern: _EdgePattern,
  type: _GlyphType
)

// Edge pattern's target at position
_edge_pattern_target [arity: 2, sealed] (
  edge_pattern: _EdgePattern,
  var: _VarDef
) {
  position: Int [required]
}

// Edge pattern's alias (binds the edge itself)
_edge_pattern_alias [arity: 2, sealed] (
  edge_pattern: _EdgePattern,
  var: _VarDef
)
```

## 5.4 Pattern Semantics

A **pattern** is a template graph. Pattern matching finds all **homomorphisms** from the template to the data.

A **match** (or **binding**) is an assignment of Glyphs to variables such that:
1. Each variable's Glyph satisfies its type constraint
2. Each edge pattern is satisfied (edge exists, or doesn't if negated)
3. All conditions evaluate to true

**Key semantics:**

| Rule | Description |
|------|-------------|
| Variables not distinct | Two variables can bind to same Glyph unless `WHERE a != b` |
| Edge patterns existential | `knows(a, b)` means "there exists a knows edge" |
| Anonymous positions | `_` binds but isn't returned |
| Edge aliases | `knows(a, b) AS k` binds `k` to the edge Glyph |
| Negation | `NOT knows(a, b)` requires edge to not exist |
| Type hierarchy | `MATCH e: Event` matches Event and all subtypes |

## 5.5 Match Identity

A match is identified by the **complete binding** — all pattern elements (including anonymous) mapped to Glyphs.

Two matches with same named variables but different anonymous bindings are **distinct matches**.

---

# Part VI: Stdlib — Expression Language

## 6.1 Purpose

The Expression Language defines **how to compute values**. Expressions are used in pattern conditions, constraint conditions, rule actions, and more.

## 6.2 Expression Types

```
_Expr [arity: 0, abstract, sealed] {
  // Base for all expressions
}

_LiteralExpr : _Expr [sealed] {
  value_type: String [required]          // "Bool", "Int", "Float", "String", "Null"
  value_string: String [required]        // serialized value
}

_VarRefExpr : _Expr [sealed] {
  var_name: String [required]
}

_AttrAccessExpr : _Expr [sealed] {
  attr_name: String [required]
}

_BinaryOpExpr : _Expr [sealed] {
  operator: String [required]            // see §6.3
}

_UnaryOpExpr : _Expr [sealed] {
  operator: String [required]            // "NOT", "-"
}

_CallExpr : _Expr [sealed] {
  function_name: String [required]       // see §6.4
}

_ExistsExpr : _Expr [sealed] {
  // Tests if a sub-pattern has matches
}
```

## 6.3 Operators

**Comparison:** `=`, `!=`, `<`, `>`, `<=`, `>=`

**Arithmetic:** `+`, `-`, `*`, `/`, `%`

**Boolean:** `AND`, `OR`

**Unary:** `NOT`, `-` (negation)

**Null handling:** `x = null` tests for null. Operations on null yield null (null propagation).

## 6.4 Context Functions

Built-in functions available in expressions:

| Function | Returns |
|----------|---------|
| `logical_time()` | Current world's tick count |
| `logical_time(scope)` | Specified world's tick count |
| `wall_time()` | Real-world timestamp (milliseconds) |
| `now()` | Alias for `wall_time()` by default |
| `current_actor()` | GlyphId of identity performing operation |
| `current_scope()` | WorldId where operation executes |
| `target_scope()` | WorldId being accessed (cross-world ops) |
| `parent_of(scope)` | Parent WorldId |
| `owner_of(scope)` | Exterior GlyphId of a world |

## 6.5 Expression Structure Edges

```
// Attribute access base
_attr_access_base [arity: 2, sealed] (
  expr: _AttrAccessExpr,
  base: _Expr
)

// Binary operands
_binary_left [arity: 2, sealed] (
  expr: _BinaryOpExpr,
  left: _Expr
)

_binary_right [arity: 2, sealed] (
  expr: _BinaryOpExpr,
  right: _Expr
)

// Unary operand
_unary_operand [arity: 2, sealed] (
  expr: _UnaryOpExpr,
  operand: _Expr
)

// Function arguments
_call_arg [arity: 2, sealed] (
  expr: _CallExpr,
  arg: _Expr
) {
  position: Int [required]
}

// Exists sub-pattern
_exists_pattern [arity: 2, sealed] (
  expr: _ExistsExpr,
  pattern: _PatternDef
)
```

## 6.6 Expression Properties

Expressions are:
- **Pure:** No side effects
- **Total:** Always produce a value (or null)
- **Terminating:** No recursion or unbounded loops

---

# Part VII: Stdlib — Mutation Language

## 7.1 Purpose

The Mutation Language defines **how to represent state changes** as Glyphs. This enables rules, watches, and other behaviors to declare their actions.

## 7.2 Mutation Types

```
_MutationDef [arity: 0, abstract, sealed] {
  // Base for all mutations
}

_CreateMutation : _MutationDef [sealed] {
  target_type: String [required]         // name of GlyphType to create
}

_DeleteMutation : _MutationDef [sealed] {
  // Deletes a Glyph
}

_SetMutation : _MutationDef [sealed] {
  attr_name: String [required]           // attribute to modify
}
```

## 7.3 Mutation Structure Edges

```
// Target Glyph for delete/set (expression evaluating to GlyphId)
_mutation_target [arity: 2, sealed] (
  mutation: _DeleteMutation | _SetMutation,
  target_expr: _Expr
)

// Create target values (for arity > 0)
_create_target [arity: 2, sealed] (
  mutation: _CreateMutation,
  target_expr: _Expr
) {
  position: Int [required]
}

// Create attribute values
_create_attr [arity: 2, sealed] (
  mutation: _CreateMutation,
  value_expr: _Expr
) {
  attr_name: String [required]
}

// Set new value
_set_value [arity: 2, sealed] (
  mutation: _SetMutation,
  value_expr: _Expr
)
```

---

# Part VIII: Stdlib — Behavioral Features

## 8.1 Constraints

### 8.1.1 Definition

A constraint declares: **for all matches of pattern, condition must hold.**

```
_ConstraintDef [arity: 0, sealed] {
  name: String [required]
  hard: Bool = true                      // hard = reject, soft = warn
  message: String?                       // error message on violation
}
```

### 8.1.2 Structure Edges

```
_constraint_pattern [arity: 2, sealed] (
  constraint: _ConstraintDef,
  pattern: _PatternDef
)

_constraint_condition [arity: 2, sealed] (
  constraint: _ConstraintDef,
  condition: _Expr
)

_ontology_declares_constraint [arity: 2, sealed] (
  ontology: _Ontology,
  constraint: _ConstraintDef
)
```

### 8.1.3 Semantics

Constraints are universally quantified:

```
∀ binding ∈ matches(pattern) : condition(binding) = true
```

- **Hard constraint violation:** Transaction rejected
- **Soft constraint violation:** Warning emitted, transaction proceeds

Constraints are checked **at commit**, after rules have reached quiescence.

---

## 8.2 Rules

### 8.2.1 Definition

A rule declares: **when pattern matches, apply mutations.**

```
_RuleDef [arity: 0, sealed] {
  name: String [required]
  priority: Int = 0                      // higher fires first
}
```

### 8.2.2 Structure Edges

```
_rule_pattern [arity: 2, sealed] (
  rule: _RuleDef,
  pattern: _PatternDef
)

_rule_action [arity: 2, sealed] (
  rule: _RuleDef,
  action: _MutationDef
) {
  position: Int [required]               // execution order
}

_ontology_declares_rule [arity: 2, sealed] (
  ontology: _Ontology,
  rule: _RuleDef
)
```

### 8.2.3 Semantics

Rules execute during the **REACT** phase:

```
react(world):
  fired = {}
  
  loop:
    matches = find_all_rule_matches(world)
    new_matches = [m for m in matches if (m.rule, identity(m.binding)) not in fired]
    
    if empty(new_matches):
      return                             // quiescent
    
    best = max_by_priority(new_matches)
    apply(best.rule.actions, best.binding)
    fired.add((best.rule.id, identity(best.binding)))
```

**Identity-based deduplication:** A binding's identity is the tuple of GlyphIds for all bound variables. A rule fires at most once per identity per tick.

**Quiescence:** Rules fire until no new matches exist. Guaranteed to terminate with identity-based dedup.

---

## 8.3 Policies

### 8.3.1 Definition

A policy declares: **access control for operations.**

```
_PolicyDef [arity: 0, sealed] {
  name: String [required]
  operations: String [required]          // comma-separated: "MATCH,CREATE,DELETE,SET"
  action: String [required]              // "ALLOW" or "DENY"
  priority: Int = 0                      // higher evaluated first
}
```

### 8.3.2 Structure Edges

```
_policy_pattern [arity: 2, sealed] (
  policy: _PolicyDef,
  pattern: _PatternDef
)

_policy_condition [arity: 2, sealed] (
  policy: _PolicyDef,
  condition: _Expr
)

_ontology_declares_policy [arity: 2, sealed] (
  ontology: _Ontology,
  policy: _PolicyDef
)
```

### 8.3.3 Semantics

**Observation policies (MATCH):** Filter results. Unauthorized matches return empty, not error.

**Mutation policies (CREATE, DELETE, SET):** Gate operations. First matching DENY rejects; first matching ALLOW permits; default deny.

Policies use context functions (`current_actor()`, `current_scope()`, etc.) to make decisions.

---

## 8.4 Watches

### 8.4.1 Definition

A watch declares: **reactive subscription to pattern changes.**

```
_WatchDef [arity: 0, sealed] {
  name: String [required]
}
```

### 8.4.2 Structure Edges

```
_watch_pattern [arity: 2, sealed] (
  watch: _WatchDef,
  pattern: _PatternDef
)

_watch_target_scope [arity: 2, sealed] (
  watch: _WatchDef,
  scope_expr: _Expr                      // evaluates to WorldId
)

_watch_on_create [arity: 2, sealed] (
  watch: _WatchDef,
  action: _MutationDef
) {
  position: Int [required]
}

_watch_on_update [arity: 2, sealed] (
  watch: _WatchDef,
  action: _MutationDef
) {
  position: Int [required]
}

_watch_on_delete [arity: 2, sealed] (
  watch: _WatchDef,
  action: _MutationDef
) {
  position: Int [required]
}

_ontology_declares_watch [arity: 2, sealed] (
  ontology: _Ontology,
  watch: _WatchDef
)
```

### 8.4.3 Semantics

At each tick of the target scope:
1. Evaluate pattern → current matches
2. Diff against previous matches
3. Fire `on_create` for new matches
4. Fire `on_update` for changed matches (attribute changes)
5. Fire `on_delete` for removed matches
6. Store current as previous

**This is the sensor pattern** — watches enable perception across world boundaries.

---

# Part IX: Worlds

## 9.1 The Isolation Principle

**Edges never cross world boundaries.**

This single rule has profound consequences:
- Each world is a self-contained database
- Worlds can be distributed without distributed edge transactions
- GlyphIds can be world-local (64-bit sufficient)
- Schema compilation is isolated

## 9.2 World Structure

```
World {
  id: WorldId
  parent: WorldId?                       // null for ROOT
  exterior_id: GlyphId?                  // this world's Glyph in parent (null for ROOT)
  
  ontology: CompiledOntology             // types, constraints, rules, policies
  glyphs: Storage                        // all Glyphs in this world
  logical_time: Int                      // current tick count
  time_mode: TimeMode                    // shared | independent | ratio(N)
  
  children: Map<GlyphId, World>          // exteriors → interiors
}
```

## 9.3 Exterior and Interior

A type with `has_interior: true` means its instances have nested worlds.

```
// DSL
node Navigator [has_interior] {
  name: String                           // exterior attributes
  
  interior: ontology {
    node LandmarkRef { ... }             // interior types
    edge visible_from(...) { ... }
  }
}
```

- **Exterior:** The `Navigator` Glyph in the parent world. Has attributes, can have edges to other parent Glyphs.
- **Interior:** A separate world with its own types, Glyphs, rules, constraints.

## 9.4 Projections

Cross-world references use **projections** — interior Glyphs holding exterior GlyphIds as attributes.

```
node LandmarkRef {
  represents: GlyphId [required]         // ID of Landmark in ROOT (attribute!)
  local_name: String?
  cached_position: List<Float>?
  confidence: Float
}
```

The `represents` field is **data**, not **structure**. The Glyph "knows about" the external entity but doesn't "connect to" it.

**Consequences:**
- Projections can be stale (exterior changed, projection didn't)
- Models real perception: incomplete, possibly outdated knowledge
- No distributed transaction needed

## 9.5 Scope Resolution

The `IN` keyword specifies target world:

```
MATCH t: Task                            // current scope
MATCH t: Task IN ROOT                    // root world
MATCH t: Task IN SELF                    // own interior
MATCH t: Task IN PARENT                  // parent world
MATCH t: Task IN #navigator              // specific world by exterior
```

Type resolution order:
1. Current scope's local types
2. Inherited types (if enabled)
3. ROOT types (always available for reference)

---

# Part X: Time

## 10.1 Logical Time

Each world has a **logical clock** — an integer that increments on tick.

```
logical_time: Int                        // starts at 0
```

Logical time is deterministic. Same inputs + same initial state → same sequence of states.

## 10.2 The Tick

A **tick** is the unit of time advancement:

```
tick(world):
  1. PROCESS   — Apply pending mutations
  2. REACT     — Fire rules to quiescence
  3. VALIDATE  — Check constraints
  4. ADVANCE   — Increment logical_time
  5. PROPAGATE — Tick children based on time_mode
  6. NOTIFY    — Process watches
```

## 10.3 Time Modes

Child worlds can have different time relationships:

| Mode | Behavior |
|------|----------|
| `shared` | Ticks when parent ticks (default) |
| `independent` | Only ticks when explicitly requested |
| `ratio(N)` | Ticks once every N parent ticks |

```
// Propagation logic
for (exterior_id, child) in world.children:
  match child.time_mode:
    shared      → tick(child)
    independent → skip
    ratio(n)    → if world.logical_time % n == 0: tick(child)
```

## 10.4 Commit vs Tick

**Commit** and **Tick** are related but distinct:

- **Commit:** Persist mutations, fire rules, check constraints
- **Tick:** Advance logical time, propagate to children

**Auto mode (default):** Commit includes implicit tick.

**Manual mode:** Commit and tick separate. Enables multiple commits at "same instant."

```
// Manual mode example
BEGIN
SPAWN a: Agent { turn = logical_time() }    // turn = 0
COMMIT
// logical_time still 0

SPAWN b: Agent { turn = logical_time() }    // turn = 0
COMMIT

TICK                                         // now logical_time = 1
```

---

# Part XI: DSL Compilation

The DSL provides friendly syntax. It compiles to the core model.

## 11.1 Node Declaration

```
// DSL
node Event {
  timestamp: Timestamp [required]
  name: String
}

// Compiles to
CREATE _GlyphType { name: "Event", arity: 0 }
CREATE _AttributeDef { name: "timestamp", required: true }
CREATE _AttributeDef { name: "name", required: false }
CREATE _ScalarTypeExpr { scalar_type: "Timestamp" }
CREATE _ScalarTypeExpr { scalar_type: "String" }
LINK _type_has_attribute(Event_type, timestamp_attr)
LINK _type_has_attribute(Event_type, name_attr)
LINK _attr_type(timestamp_attr, timestamp_type_expr)
LINK _attr_type(name_attr, string_type_expr)
```

## 11.2 Edge Declaration

```
// DSL
edge causes(a: Event, b: Event) {
  strength: Float
}

// Compiles to
CREATE _GlyphType { name: "causes", arity: 2 }
CREATE _VarDef { name: "a" }
CREATE _VarDef { name: "b" }
CREATE _NamedTypeExpr { type_name: "Event" }
LINK _type_has_position(causes_type, var_a) { position: 0 }
LINK _type_has_position(causes_type, var_b) { position: 1 }
LINK _var_type(var_a, event_type_expr)
LINK _var_type(var_b, event_type_expr)
CREATE _AttributeDef { name: "strength" }
CREATE _ScalarTypeExpr { scalar_type: "Float" }
LINK _type_has_attribute(causes_type, strength_attr)
LINK _attr_type(strength_attr, float_type_expr)
```

## 11.3 Inheritance

```
// DSL
node BusinessEvent : Event {
  contract_id: String
}

edge triggers(a: Event, b: Event) : causes {
  delay: Int
}

// Compiles to
CREATE _GlyphType { name: "BusinessEvent", arity: 0 }
LINK _type_inherits(BusinessEvent_type, Event_type)
// ... attributes ...

CREATE _GlyphType { name: "triggers", arity: 2 }
LINK _type_inherits(triggers_type, causes_type)
// signature inherited, attributes added
```

## 11.4 Constraint Declaration

```
// DSL
constraint task_priority_bounds:
  t: Task
  => t.priority >= 0 AND t.priority <= 100

// Compiles to
CREATE _ConstraintDef { name: "task_priority_bounds", hard: true }
CREATE _PatternDef {}
CREATE _VarDef { name: "t" }
LINK _pattern_has_var(pattern, var_t)
LINK _var_type(var_t, task_type_expr)
// ... build condition expression tree ...
LINK _constraint_pattern(constraint, pattern)
LINK _constraint_condition(constraint, condition_expr)
```

## 11.5 Rule Declaration

```
// DSL
rule auto_archive [priority: 10]:
  t: Task WHERE t.status = "done" AND t.archived = false
  => SET t.archived = true

// Compiles to
CREATE _RuleDef { name: "auto_archive", priority: 10 }
CREATE _PatternDef {}
// ... build pattern ...
CREATE _SetMutation { attr_name: "archived" }
// ... build mutation target and value ...
LINK _rule_pattern(rule, pattern)
LINK _rule_action(rule, set_mutation) { position: 0 }
```

---

# Part XII: Execution Model

## 12.1 The Tick Cycle

```
tick(world):
  
  // 1. PROCESS: Apply pending mutations
  for delta in world.pending_mutations:
    apply(delta)
  world.pending_mutations.clear()
  
  // 2. REACT: Fire rules to quiescence
  fired = {}
  loop:
    matches = evaluate_all_rule_patterns(world)
    new_matches = [m for m in matches if (m.rule.id, binding_id(m)) not in fired]
    
    if empty(new_matches):
      break
    
    best = max_by_priority(new_matches)
    execute(best.rule.actions, best.binding)
    fired.add((best.rule.id, binding_id(best.binding)))
  
  // 3. VALIDATE: Check constraints
  for constraint in world.ontology.constraints:
    for binding in evaluate_pattern(constraint.pattern):
      if not evaluate(constraint.condition, binding):
        if constraint.hard:
          rollback()
          raise ConstraintViolation(constraint.message)
        else:
          warn(constraint.message)
  
  // 4. ADVANCE: Increment time
  world.logical_time += 1
  
  // 5. PROPAGATE: Tick children
  for (exterior_id, child) in world.children:
    match child.time_mode:
      shared      → tick(child)
      independent → pass
      ratio(n)    → if world.logical_time % n == 0: tick(child)
  
  // 6. NOTIFY: Process watches
  for watch in watches_targeting(world):
    current = evaluate_pattern(watch.pattern, world)
    created = current - watch.previous_matches
    deleted = watch.previous_matches - current
    updated = detect_changes(current ∩ watch.previous_matches)
    
    for binding in created: execute(watch.on_create, binding)
    for binding in updated: execute(watch.on_update, binding)
    for binding in deleted: execute(watch.on_delete, binding)
    
    watch.previous_matches = current
```

## 12.2 Storage Model (Guidance)

The core model doesn't mandate storage format. Recommended approaches:

**Columnar layout:** Each attribute as dense array. Enables SIMD, GPU coalescing.

**Family tables:** Subtypes of common root in single table. Type tag discriminates.

**Edge tensors:** Each edge type as sparse matrix (COO, CSR, CSC).

**GlyphId encoding:**
```
GlyphId (64 bits):
┌────────────────┬────────────────────────────────────────────┐
│   Table ID     │              Local Index                    │
│   (16 bits)    │              (48 bits)                      │
└────────────────┴────────────────────────────────────────────┘

• 65K types per world
• 281 trillion Glyphs per type
• Sufficient for trillion-scale worlds
```

## 12.3 Parallelization (Guidance)

**Within a world:**
- Pattern matching: embarrassingly parallel
- Rule firing: serialize conflicts, parallelize non-conflicting
- Constraint checking: parallel per constraint

**Across worlds:**
- Worlds are isolated → tick in parallel
- No cross-world edges → no cross-world conflicts

**GPU acceleration:**
- Patterns → sparse tensor operations
- Traversal → SpMV
- Filtering → parallel predicate evaluation

## 12.4 Distribution (Guidance)

**World as distribution unit:**
- Each world can live on separate node
- No distributed transactions for edges
- Projections enable eventual consistency

**Large world partitioning:**
- Partition by GlyphId range or type
- Cross-partition patterns need shuffle/gather

---

# Part XIII: Formal Properties

## 13.1 Determinism

Given:
- Same initial state
- Same sequence of external mutations
- Same rule priorities

The system produces identical final state.

**Guarantee:** Logical time, priority ordering, and identity-based dedup ensure determinism.

## 13.2 Termination

Rule firing terminates if:
- Identity-based dedup enforced (default), or
- Rules proven monotonic, or
- Explicit iteration bounds set

**Guarantee:** Each (rule, binding-identity) pair fires at most once per tick.

## 13.3 Isolation

Worlds are isolated:
- Edges never cross boundaries
- Type namespaces independent
- Time can be independent
- Rules/constraints scoped

**Guarantee:** Changes in one world cannot directly affect another.

## 13.4 Type Safety

Operations are type-checked:
- CREATE validates targets against signature
- SET validates value against attribute type
- Pattern matching respects type hierarchy

**Guarantee:** Well-typed operations never produce runtime type errors.

---

# Part XIV: Extensions

These features build on the core. They do not change the foundation.

## 14.1 Space

Spatial positioning and queries.

```
_SpaceDef [arity: 0] {
  name: String [required]
  dimensions: Int [required]
  metric: String = "euclidean"           // "euclidean", "cosine", "manhattan"
}

_attr_space [arity: 2] (
  attr: _AttributeDef,
  space: _SpaceDef
)
```

**Functions:** `distance(a, b, space)`, `nearest(target, k, space)`

## 14.2 Branching

Alternate timelines / versioning.

```
branch(world) → new_world
snapshot(world) → frozen_state
merge(source, target) → conflicts
```

New world initialized from snapshot. Independent evolution. Merge with conflict detection.

## 14.3 Federation

Cross-kernel connectivity.

```
ExternalRef {
  kernel: KernelId
  world: WorldId
  glyph: GlyphId
}
```

Stored as attribute value. Async replication. Eventually consistent.

## 14.4 Blockchain / Event Sourcing

Immutable history.

```
MutationLog = [
  { tick, operation, glyph_id, data, prev_hash, hash }
]
```

All mutations logged. Cryptographic chaining optional. Enables audit, replay, consensus.

---

# Part XV: Summary

## 15.1 The Core (Forever Valid)

| Concept | Definition |
|---------|------------|
| **Glyph** | The universal primitive: id, type, targets, attributes |
| **GlyphType** | Describes Glyphs: arity, signature, attributes, inheritance |
| **Arity** | 0 = "node", n > 0 = "edge" |
| **Pattern** | Template graph; matching = homomorphism |
| **Expression** | Pure value computation |
| **Mutation** | CREATE, DELETE, SET |
| **Rule** | Pattern → Mutations, fires to quiescence |
| **Constraint** | Pattern + Condition, universally quantified |
| **Policy** | Pattern + Condition → ALLOW/DENY |
| **Watch** | Pattern subscription, diff-based notification |
| **World** | Isolated database; edges never cross |
| **Tick** | Process → React → Validate → Advance → Propagate → Notify |
| **Projection** | Interior Glyph with exterior GlyphId as attribute |

## 15.2 Layer Summary

| Layer | Name | Purpose |
|-------|------|---------|
| — | Runtime | Execution engine (not MEW) |
| 0a | Bootstrap | Fixed point: `_GlyphType : _GlyphType` |
| 0b | Meta-Schema | Type system primitives |
| 1 | Stdlib | Patterns, Expressions, Mutations, Behaviors |
| 2 | Ontology | User-defined domain types |
| 3 | Data | Instances |

## 15.3 Design Principles

1. **One primitive (Glyph)** — nodes and edges unified
2. **Self-describing** — types are Glyphs
3. **Edges stay inside** — world isolation enables distribution
4. **Patterns everywhere** — queries, rules, constraints, policies, watches
5. **Deterministic** — same inputs → same outputs
6. **Layered** — semantics separate from execution

## 15.4 What Builds On Top

| Feature | Foundation |
|---------|-----------|
| Rules | Patterns + Mutations |
| Constraints | Patterns + Expressions |
| Policies | Patterns + Context |
| Watches | Patterns + Diff |
| Time modes | World + Tick propagation |
| Space | Attributes + Functions |
| Branching | Worlds + Snapshot |
| Federation | Worlds + ExternalRef |
| GPU | Execution strategy |
| Distribution | World isolation |

---

*This is MEW v2. The foundation is complete.*
