# HOHG Language Specification

## Part II: Ontology DSL (Revised)

**Version:** 1.1
**Status:** Draft
**Scope:** Declarative language for defining graph schemas

---

# 7. Ontology DSL Overview

## 7.1 Purpose

The Ontology DSL defines the structure of a graph:

- **What kinds of entities can exist** (node types)
- **What kinds of relations can exist** (edge types)
- **What must always be true** (constraints)
- **What transformations can occur** (rules)

An ontology is a schema. It does not contain instance data — it describes what instance data can look like.

## 7.2 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Graph-native** | Everything compiles to nodes and edges |
| **Constraints as foundation** | All validation is constraint-based |
| **Sugar for DX** | Inline modifiers compile to constraints |
| **Explicit over implicit** | Behavior is declared, not assumed |

## 7.3 Compilation Model

Ontology DSL source files are **compiled**, not interpreted:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────────────────┐
│  .hog file   │────▶│   Compiler   │────▶│  Layer 0 Graph Structure │
│  (source)    │     │              │     │  + Runtime Registries    │
└──────────────┘     └──────────────┘     └──────────────────────────┘
```

The compiler:
1. Parses source into AST
2. Validates structure and references
3. **Expands sugar into constraints/rules**
4. Generates Layer 0 nodes and edges
5. Builds runtime registries for type checking

## 7.4 File Extension

Ontology files use the `.hog` extension:

```
physics.hog
social.hog
task_management.hog
```

## 7.5 Top-Level Structure

```
OntologyFile = OntologyDecl? Declaration*

Declaration =
    TypeAliasDecl
  | NodeTypeDecl
  | EdgeTypeDecl
  | ConstraintDecl
  | RuleDecl
```

---

# 8. Type Aliases

## 8.1 Purpose

Type aliases define reusable type expressions with optional modifiers. They reduce repetition and establish domain vocabulary.

## 8.2 Syntax

```
TypeAliasDecl = 
    "type" Identifier "=" TypeExpr AttributeModifiers?
  | "type" Identifier "=" UnionTypeExpr

UnionTypeExpr = TypeExpr ("|" TypeExpr)+
```

## 8.3 Examples

### 8.3.1 Scalar Aliases with Modifiers

```
-- Simple alias
type Email = String [match: "^.+@.+\\..+$"]

-- Numeric constraint (can use range syntax)
type Priority = Int [0..10]
type NonNegativeInt = Int [>= 0]

-- Enum alias
type TaskStatus = String [in: ["todo", "in_progress", "done", "blocked"]]

-- Composite: optional with constraint
type OptionalEmail = String? [match: "^.+@.+\\..+$"]
```

### 8.3.2 Union Type Aliases

```
-- Union of node types
type Entity = Person | Organization | Bot
type Assignable = Task | Issue | Story

-- Usage in edge signatures
edge owns(owner: Entity, asset: Asset)
edge assigned_to(item: Assignable, person: Person)
```

### 8.3.3 Alias Chaining

Aliases can reference other aliases:

```
type Email = String [match: "^.+@.+\\..+$"]
type RequiredEmail = Email [required]
type UniqueEmail = RequiredEmail [unique]
```

Expansion is recursive at compile time. `UniqueEmail` becomes `String [match: "...", required, unique]`.

## 8.4 Usage

Aliases can be used anywhere a type is expected:

```
node Person {
  email: Email [required, unique],
  backup_email: Email?
}

node Task {
  status: TaskStatus = "todo",
  priority: Priority = 5
}

-- Union alias in edge
edge created_by(item: Assignable, creator: Person)
```

## 8.5 Modifier Composition

When an alias is used with additional modifiers, they combine:

```
type Email = String [match: "^.+@.+\\..+$"]

node Person {
  email: Email [required, unique, indexed]
}
```

The `email` attribute has: `match`, `required`, `unique`, `indexed`.

**Conflict resolution:** If the same modifier appears in both alias and usage with different values, the usage wins:

```
type Priority = Int [0..10]

node Task {
  priority: Priority [<= 5] = 3   -- max is now 5, not 10
}
```

## 8.6 Restrictions

- Aliases cannot be recursive (directly or indirectly)
- Alias names cannot shadow built-in types or node/edge type names
- Union aliases cannot have modifiers (modifiers only apply to scalar types)

## 8.7 AST

```typescript
interface TypeAliasDecl {
  kind: "TypeAlias"
  name: string
  type: TypeExpr | UnionTypeExpr
  modifiers: AttributeModifiers | null  // null for union aliases
}

interface UnionTypeExpr {
  kind: "UnionType"
  members: TypeExpr[]
}
```

## 8.8 Compilation

Type aliases are expanded at compile time. They do not generate Layer 0 nodes — they are purely syntactic sugar.

```
type Email = String [match: "^.+@.+\\..+$"]

node Person {
  email: Email [required]
}
```

Compiles identically to:

```
node Person {
  email: String [required, match: "^.+@.+\\..+$"]
}
```

Union aliases expand to inline union types:

```
type Entity = Person | Organization
edge owns(owner: Entity, asset: Asset)
```

Compiles to:

```
edge owns(owner: Person | Organization, asset: Asset)
```

---

# 9. Ontology Declaration

## 9.1 Syntax

```
OntologyDecl = "ontology" Identifier InheritanceClause? "{" Declaration* "}"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*
```

## 9.2 Examples

```
-- Simple ontology
ontology TaskManagement {
  -- declarations here
}

-- Ontology with single inheritance
ontology Physics : Layer0 {
  -- inherits from Layer0
}

-- Ontology with multiple inheritance
ontology GameWorld : Physics, Social {
  -- inherits from both
}
```

## 9.3 Implicit Ontology

If no `ontology` declaration is present, the file defines an anonymous ontology that implicitly inherits from Layer0.

## 9.4 Inheritance Semantics

When ontology B inherits from ontology A:

| Inherited | Behavior |
|-----------|----------|
| Node types | Available in B, can be extended |
| Edge types | Available in B |
| Constraints | Active in B |
| Rules | Active in B |

Inheritance is **additive only**. Child ontologies cannot remove or modify inherited definitions.

## 9.5 AST

```typescript
interface OntologyDecl {
  kind: "Ontology"
  name: string | null
  parents: string[]
  declarations: Declaration[]
}
```

## 9.6 Compilation

An `OntologyDecl` compiles to:

```
_Ontology node:
  name: <ontology name>
  
For each parent P:
  _ontology_inherits edge: (this_ontology, P)
```

---

# 10. Node Type Declarations

## 10.1 Syntax

```
NodeTypeDecl = 
  DocComment?
  "node" Identifier InheritanceClause? "{" AttributeDecl* "}"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*
```

## 10.2 Examples

```
-- Simple node type
node Event {
  timestamp: Timestamp
}

-- Node type with inheritance
node Meeting : Event {
  location: String,
  attendee_count: Int
}

-- Node type with full attribute modifiers
node Person {
  name: String [required, length: 1..100],
  email: String [required, unique, match: "^.+@.+$"],
  age: Int [>= 0, <= 150],
  role: String [in: ["admin", "member", "guest"]] = "member",
  active: Bool = true
}

-- Multiple inheritance
node Document : Named, Timestamped {
  content: String
}
```

## 10.3 Type Inheritance

### 10.3.1 Semantics

When type B inherits from type A:

| Inherited | Behavior |
|-----------|----------|
| Attributes | B has all attributes of A |
| Constraints | Constraints on A apply to B |
| Pattern matching | Pattern `a: A` matches instances of B |

### 10.3.2 Single Inheritance

```
node Animal {
  name: String
}

node Dog : Animal {
  breed: String
}
```

`Dog` inherits `name` from `Animal`.

### 10.3.3 Multiple Inheritance

```
node Named {
  name: String [required]
}

node Timestamped {
  created_at: Timestamp [required],
  updated_at: Timestamp
}

node Document : Named, Timestamped {
  content: String
}
```

`Document` has: `name`, `created_at`, `updated_at`, `content`.

### 10.3.4 Diamond Resolution

If the same attribute is inherited through multiple paths from the same origin, it appears once. If different parents define the same attribute name with incompatible types, the ontology is invalid.

## 10.4 AST

```typescript
interface NodeTypeDecl {
  kind: "NodeType"
  name: string
  parents: string[]
  attributes: AttributeDecl[]
  doc: string | null
}
```

## 10.5 Compilation

A `NodeTypeDecl` compiles to:

```
_NodeType node:
  name: <type name>
  doc: <doc comment or null>

For each parent P:
  _type_inherits edge: (this_type, P)

For each attribute A:
  _AttributeDef node (see Section 11)
  _type_has_attribute edge: (this_type, A)
  
For each attribute modifier:
  Expanded constraint (see Section 11.5)
```

---

# 11. Attribute Declarations

## 11.1 Syntax

```
AttributeDecl = 
  DocComment?
  Identifier ":" TypeExpr AttributeModifiers? DefaultValue? ","?

AttributeModifiers = "[" AttributeModifier ("," AttributeModifier)* "]"

AttributeModifier = 
    "required"
  | "unique"
  | "indexed" (":" ("asc" | "desc"))?
  | ComparisonModifier
  | "in:" "[" Literal ("," Literal)* "]"
  | "match:" StringLiteral
  | "length:" IntLiteral ".." IntLiteral

ComparisonModifier =
    ">=" Literal
  | "<=" Literal
  | ">" Literal
  | "<" Literal

DefaultValue = "=" Literal
```

## 11.2 Basic Examples

```
node Task {
  -- Required attribute
  title: String [required],
  
  -- Optional attribute (nullable)
  description: String?,
  
  -- With default value
  status: String = "pending",
  
  -- Required with default (must be non-null, defaults if omitted)
  priority: Int [required] = 0,
  
  -- Documentation comment
  --- The timestamp when this task was created
  created_at: Timestamp [required]
}
```

## 11.2.1 Attribute Nullability and Requirements

Understanding the relationship between nullable types (`T?`) and the `[required]` modifier:

| Declaration | Nullable? | Must provide at SPAWN? | Omitted behavior |
|-------------|-----------|------------------------|------------------|
| `x: T` | No | No | **Compile warning** (see below) |
| `x: T?` | Yes | No | Value is `null` |
| `x: T = default` | No | No | Value is `default` |
| `x: T [required]` | No | Yes | **Error** if omitted |
| `x: T? [required]` | — | — | **Compile error** (contradictory) |

**Non-nullable without default or required:**

```
node Task {
  priority: Int   -- No ?, no default, no [required]
}
```

This is a **compile-time warning**:

```
WARNING: Attribute 'priority' on 'Task' is non-nullable but has no default 
         and is not [required].
         
         If omitted at SPAWN time, accessing this attribute will error.
         
         Consider one of:
           priority: Int = 0              -- provide default
           priority: Int [required]       -- require at SPAWN
           priority: Int?                 -- make nullable
```

**Best practice:** Every non-nullable attribute should either:
1. Have a default value (`= value`), OR
2. Be marked `[required]`

**Nullable + required is invalid:**

```
node Task {
  description: String? [required]   -- COMPILE ERROR
}
-- Error: Attribute 'description' cannot be both nullable (?) and [required].
--        Use 'String [required]' if a value must be provided.
--        Use 'String?' if the value can be null.
```

## 11.3 Attribute Modifiers

### 11.3.1 Required

The attribute must have a non-null value.

```
name: String [required]
```

**Compiles to:**
```
constraint <type>_<attr>_required:
  x: <Type> WHERE x.<attr> = null
  => false
```

**Behavior:**
- SPAWN without this attribute: Error (unless default exists)
- SET to null: Error

### 11.3.2 Unique

The attribute value must be unique across all instances of this type.

```
email: String [unique]
```

**Compiles to:**
```
constraint <type>_<attr>_unique:
  x1: <Type>, x2: <Type>
  WHERE x1.id != x2.id AND x1.<attr> = x2.<attr> AND x1.<attr> != null
  => false
```

**Notes:**
- Uniqueness applies within the type and its subtypes
- Null values do not violate uniqueness (multiple nulls allowed)
- `unique` implies `indexed` (index required for efficient checking)

### 11.3.3 Indexed

Create an index on this attribute for faster observations.

```
timestamp: Timestamp [indexed]
timestamp: Timestamp [indexed: desc]  -- sorted descending
```

**Compiles to:** Engine index hint (not a constraint).

**Notes:**
- `unique` implies `indexed`
- `indexed: asc` is default
- `indexed: desc` for reverse-order observations (e.g., most recent first)

### 11.3.4 Comparison Modifiers

Constrain numeric values.

| Modifier | Meaning |
|----------|---------|
| `>= N` | Greater than or equal to N |
| `<= N` | Less than or equal to N |
| `> N` | Greater than N |
| `< N` | Less than N |

```
age: Int [>= 0, <= 150]
score: Float [>= 0.0, <= 1.0]
```

**Compiles to:**
```
constraint <type>_<attr>_min:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> >= N

constraint <type>_<attr>_max:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> <= N
```

### 11.3.5 Range Modifier

Shorthand for combined min/max constraints on numeric values.

```
priority: Int [0..10]           -- equivalent to [>= 0, <= 10]
age: Int [0..150]
score: Float [0.0..1.0]
```

**Compiles to:** Same as `[>= N, <= M]` — two constraints.

**Note:** For string length, use `[length: N..M]` instead.

### 11.3.6 Enum Modifier

Restrict to a set of allowed values.

```
status: String [in: ["draft", "active", "archived"]]
priority: Int [in: [1, 2, 3, 4, 5]]
```

**Compiles to:**
```
constraint <type>_<attr>_enum:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> = "draft" OR x.<attr> = "active" OR x.<attr> = "archived"
```

### 11.3.7 Match Modifier

Validate string against regular expression.

```
email: String [match: "^.+@.+\\..+$"]
slug: String [match: "^[a-z0-9-]+$"]
```

**Compiles to:**
```
constraint <type>_<attr>_match:
  x: <Type> WHERE x.<attr> != null
  => matches(x.<attr>, "<pattern>")
```

### 11.3.8 Length Modifier

Constrain string length.

```
name: String [length: 1..100]      -- 1 to 100 characters
code: String [length: 6..6]        -- exactly 6 characters
bio: String [length: 0..1000]      -- up to 1000 characters
```

**Compiles to:**
```
constraint <type>_<attr>_length:
  x: <Type> WHERE x.<attr> != null
  => length(x.<attr>) >= 1 AND length(x.<attr>) <= 100
```

### 11.3.9 NULL Handling in Value Modifiers

Value constraint modifiers (`>=`, `<=`, `>`, `<`, range `N..M`, `in:`, `match:`, `length:`) only apply when the attribute is **non-null**.

**Example:**
```
node Person {
  age: Int? [>= 0]
}
```

**Compiles to:**
```
constraint person_age_min:
  p: Person WHERE p.age != null   -- ← NULL check in WHERE
  => p.age >= 0
```

**Behavior:**

| Value | Constraint Result |
|-------|-------------------|
| `age = 25` | ✅ Valid (25 >= 0) |
| `age = -5` | ❌ Violates (−5 < 0) |
| `age = null` | ✅ Valid (constraint skipped) |

**To require non-null AND validate**, combine with `[required]`:

```
age: Int [required, >= 0]   -- Must be present AND >= 0
```

**Rationale:** This follows SQL semantics where NULL comparisons are neither true nor false. Constraints express "if present, must satisfy X" — use `[required]` to enforce presence.

## 11.4 Modifier Combinations

Modifiers can be combined:

```
node User {
  -- All modifiers together
  email: String [required, unique, match: "^.+@.+\\..+$", length: 5..100],
  
  -- Required with range and default
  priority: Int [required, >= 1, <= 5] = 3,
  
  -- Unique enum
  role_code: String [unique, in: ["ADM", "USR", "GST"]]
}
```

**Order:** Modifiers can appear in any order within brackets.

## 11.5 Default Values

Default values are used when an attribute is not provided at SPAWN time.

```
node Task {
  status: String = "pending",
  priority: Int = 0,
  active: Bool = true
}

SPAWN t: Task { title = "Test" }
-- t.status = "pending" (default)
-- t.priority = 0 (default)
-- t.active = true (default)
```

### 11.5.1 Allowed Default Values

Defaults can be **literals** or **constant expressions**:

```
-- Literal defaults:
count: Int = 0
name: String = "unnamed"
active: Bool = true

-- Constant expression defaults:
created_at: Timestamp = now()
expires_at: Timestamp = now() + 7.days
priority_score: Float = 5.0 * 1.5
```

**Constant expressions** are evaluated at entity creation time:

```
ConstantExpr =
    Literal
  | "now()"
  | DurationLiteral
  | ConstantExpr ("+" | "-" | "*" | "/") ConstantExpr
```

| Expression | Type | Example |
|------------|------|---------|
| Literal | Any scalar | `0`, `"default"`, `true` |
| `now()` | Timestamp | Current time at SPAWN/LINK |
| Duration | Duration | `7.days`, `30.minutes` |
| Arithmetic | Numeric/Timestamp | `now() + 24.hours`, `5 * 10` |

**Examples:**
```
node Token {
  created_at: Timestamp = now(),
  expires_at: Timestamp = now() + 24.hours,
  refresh_interval: Int = 60 * 60 * 1000,  -- 1 hour in ms
  priority: Float = 1.0
}
```

### 11.5.2 Disallowed Defaults

Attribute references and non-pure function calls are not allowed:

```
-- Invalid:
ref: String = other.name          -- ERROR: attribute reference not allowed
random: Int = random()            -- ERROR: non-deterministic function
computed: Int = count(...)        -- ERROR: observe in default
```

For computed defaults, use rules:

```
node Event {
  computed_field: Int?
}

rule compute_field [priority: 100]:
  e: Event WHERE e.computed_field = null
  =>
  SET e.computed_field = e.base_value * 2
```

### 11.5.4 Required with Default

If an attribute has both `required` and a default, the default satisfies the requirement:

```
status: String [required] = "pending"
```

SPAWN without `status` succeeds — it defaults to `"pending"`.

## 11.6 Comprehensive Example

```
node Person {
  --- Unique identifier for external systems
  external_id: String [unique, match: "^[A-Z]{2}[0-9]{6}$"],
  
  --- Full name of the person
  name: String [required, length: 1..200],
  
  --- Email address (required, must be unique and valid)
  email: String [required, unique, match: "^.+@.+\\..+$"],
  
  --- Age in years (optional, but if present must be valid)
  age: Int? [>= 0, <= 150],
  
  --- Role in the system
  role: String [required, in: ["admin", "moderator", "user"]] = "user",
  
  --- Account status
  status: String [in: ["active", "suspended", "deleted"]] = "active",
  
  --- Reputation score (0.0 to 5.0)
  reputation: Float [>= 0.0, <= 5.0] = 0.0,
  
  --- Biography (optional, limited length)
  bio: String? [length: 0..2000],
  
  --- When the account was created (indexed for observations)
  created_at: Timestamp [required, indexed: desc]
}
```

## 11.7 AST

```typescript
interface AttributeDecl {
  kind: "Attribute"
  name: string
  type: TypeExpr
  modifiers: AttributeModifiers
  defaultValue: Literal | null
  doc: string | null
}

interface AttributeModifiers {
  required: boolean
  unique: boolean
  indexed: "none" | "asc" | "desc"
  min: number | null              // >= N
  max: number | null              // <= N
  greaterThan: number | null      // > N
  lessThan: number | null         // < N
  enumValues: Literal[] | null    // in: [...]
  match: string | null            // match: "..."
  lengthMin: number | null        // length: N..M
  lengthMax: number | null
}
```

## 11.8 Compilation

An `AttributeDecl` compiles to:

```
_AttributeDef node:
  name: <attribute name>
  required: <true/false>
  unique: <true/false>
  indexed: <"none"/"asc"/"desc">
  default_value: <serialized default or null>
  doc: <doc comment or null>

_attr_has_type edge:
  (attr_def, type_expr_node)

For each constraint modifier, generate _ConstraintDef:
  (see 10.3.x for expansion patterns)
```

---

# 12. Edge Type Declarations

## 12.1 Syntax

```
EdgeTypeDecl =
  DocComment?
  "edge" Identifier "(" SignatureParams ")" EdgeModifiers? ("{" AttributeDecl* "}")?

SignatureParams = SignatureParam ("," SignatureParam)*

SignatureParam = Identifier ":" TypeExpr

EdgeModifiers = "[" EdgeModifier ("," EdgeModifier)* "]"

EdgeModifier =
    "symmetric"
  | "no_self"
  | "acyclic"
  | "unique"
  | "indexed"
  | CardinalityModifier
  | ReferentialModifier

CardinalityModifier = Identifier "->" Cardinality

Cardinality = 
    IntLiteral                      -- exactly N
  | IntLiteral ".." IntLiteral      -- N to M
  | IntLiteral ".." "*"             -- N or more

ReferentialModifier =
    "on_kill_source:" ReferentialAction
  | "on_kill_target:" ReferentialAction

ReferentialAction = "cascade" | "unlink" | "prevent"
```

**Note:** The attribute block `{ ... }` is optional. Edges without attributes can omit it:

```
edge causes(from: Event, to: Event)                    -- no braces
edge depends_on(a: Task, b: Task) [no_self, acyclic]   -- no braces
edge assigned_to(task: Task, person: Person) {         -- with braces
  assigned_at: Timestamp = now()
}
```

## 12.2 Basic Examples

```
-- Simple binary edge
edge causes(from: Event, to: Event) {}

-- Edge with attributes
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required],
  role: String [in: ["owner", "reviewer", "observer"]] = "owner"
}

-- Polymorphic edge
edge tagged(entity: any, tag: Tag) {}

-- Higher-order edge (targets another edge)
edge confidence(about: edge<causes>, level: Float) {
  assessed_by: String?
}

-- N-ary edge (hyperedge)
edge meeting(organizer: Person, attendee1: Person, attendee2: Person, room: Room) {
  scheduled_at: Timestamp [required]
}
```

## 12.3 Edge Modifiers

### 12.3.1 Symmetric

Edge is order-independent: `edge(a, b)` equals `edge(b, a)`.

```
edge friend_of(a: Person, b: Person) [symmetric] {}
```

**Semantics:**
- Storage: One edge stored with targets in canonical order
- Matching: `friend_of(x, y)` matches regardless of storage order
- Uniqueness: `friend_of(a, b)` and `friend_of(b, a)` are the same edge

**Type requirement:** Symmetric edges require **identical types** on both endpoints:

```
-- VALID: Same type on both endpoints
edge friend_of(a: Person, b: Person) [symmetric] {}
edge similar_to(x: Document, y: Document) [symmetric] {}

-- COMPILE ERROR: Different types
edge collaboration(person: Person, org: Organization) [symmetric] {}
-- Error: Cannot apply [symmetric] to edge with different parameter types.
--        person: Person ≠ org: Organization
--        Symmetric edges require interchangeable endpoints.
```

**Rationale:** Symmetry implies `edge(a, b) = edge(b, a)`. This only makes sense when `a` and `b` can be swapped, which requires the same type.

**Cardinality on symmetric edges:** When a cardinality constraint is specified on one parameter of a symmetric edge, it applies symmetrically to both:

```
edge married_to(a: Person, b: Person) [symmetric, a -> 0..1] {}
-- Equivalent to: [symmetric, a -> 0..1, b -> 0..1]
-- Each person can be married to at most one other person
```

**Cardinality conflict validation:** For symmetric edges with parameters of the same type, conflicting cardinality specifications are a **compile-time error**:

```
-- COMPILE ERROR: Conflicting cardinality on symmetric edge
edge friends(a: Person, b: Person) [symmetric, a -> 0..5, b -> 0..10] {}
-- Error: Symmetric edge 'friends' has conflicting cardinality:
--        a -> 0..5 vs b -> 0..10
--        Symmetric edges require identical cardinality on both parameters.
```

**Valid patterns:**
```
edge friends(a: Person, b: Person) [symmetric, a -> 0..5] {}       -- OK: b inherits
edge friends(a: Person, b: Person) [symmetric, a -> 0..5, b -> 0..5] {} -- OK: explicit match
edge friends(a: Person, b: Person) [symmetric] {}                 -- OK: no cardinality
```

**Note:** This is a structural modifier that affects storage and matching, not just validation.

### 12.3.2 No Self

Prevents an edge from connecting a node to itself.

```
edge depends_on(from: Task, to: Task) [no_self] {}
```

**Compiles to:**
```
constraint depends_on_no_self:
  t: Task, depends_on(t, t)
  => false
```

**Applies to:** Edges where multiple parameters have compatible types.

### 12.3.3 Acyclic

Prevents cycles through this edge type.

```
edge parent_of(parent: Person, child: Person) [acyclic] {}
```

**Compiles to:**
```
constraint parent_of_acyclic:
  p: Person, parent_of+(p, p)
  => false
```

Where `parent_of+(a, b)` matches transitive closure (one or more hops).

**Note:** Cycle detection is performed at LINK time.

**⚠️ Compile-Time Warning:**

Using `[acyclic]` generates a compile-time warning:

```
WARNING: Edge 'parent_of' uses [acyclic] modifier.
         
         Cycle detection is O(V+E) per LINK operation.
         For graphs > 10,000 reachable nodes, this may cause
         significant performance degradation.
         
         Consider:
         - Application-level cycle detection
         - Depth-limited constraint: [depth: N]
         - Periodic batch validation
         
         To suppress: [acyclic, suppress_warning]
```

**Engine configuration:**
```
-- Maximum nodes to check for acyclic constraint
SET engine.acyclic_check_limit = 10000

-- Behavior when limit exceeded:
-- "error" = fail the LINK (DEFAULT - safe)
-- "skip" = skip check with warning (dangerous - cycles may be introduced)
-- "async" = check asynchronously (eventual consistency)
SET engine.acyclic_check_overflow = "error"  -- DEFAULT
```

**⚠️ Warning about "skip" mode:**

```
-- NOT RECOMMENDED:
SET engine.acyclic_check_overflow = "skip"

-- This allows cycles to be introduced silently when graph exceeds limit!
-- Only use if you have application-level cycle detection.
```

**⚠️ Performance Characteristics:**

The `[acyclic]` modifier requires computing transitive reachability on every LINK operation.

| Graph Size (reachable nodes) | Typical LINK Cost | Recommendation |
|------------------------------|-------------------|----------------|
| < 1,000 | < 1ms | ✅ Use freely |
| 1,000 – 10,000 | 1–10ms | ✅ Generally fine |
| 10,000 – 100,000 | 10–100ms | ⚠️ Consider alternatives |
| > 100,000 | 100ms+ | ❌ Use application-level checks |

**Alternatives for large graphs:**

1. **Application-level detection:** Check for cycles before LINK in application code
2. **Batch validation:** Run periodic cycle detection instead of per-LINK
3. **Depth-limited check:** Use explicit constraint with bounded transitive:
   ```
   constraint shallow_acyclic:
     p: Person, parent_of+(p, p) [depth: 10]
     => false
   ```

### 12.3.4 Unique

Prevents duplicate edges with same targets.

```
edge member_of(person: Person, team: Team) [unique] {}
```

**Semantics:** Only one `member_of(alice, engineering)` can exist.

**Note:** Most edges are unique by default in many graph systems. This modifier makes it explicit.

### 12.3.5 Indexed

Create indexes for efficient edge traversal.

```
edge causes(from: Event, to: Event) [indexed] {}
```

**Creates indexes for:**
- Forward lookup: given `from`, find all `to`
- Reverse lookup: given `to`, find all `from`

## 12.4 Cardinality Modifiers

Control how many edges a node can have.

### 12.4.1 Syntax

```
[param_name -> cardinality]
```

Where `param_name` is the name of a signature parameter.

### 12.4.2 Cardinality Values

| Syntax | Meaning |
|--------|---------|
| `N` | Exactly N |
| `N..M` | Between N and M (inclusive) |
| `N..*` | N or more |
| `0..1` | At most one (optional) |
| `1` | Exactly one (required) |
| `1..*` | At least one |

### 12.4.3 Examples

```
-- Each task belongs to exactly one project
edge belongs_to(task: Task, project: Project) [task -> 1] {}

-- Each task has at most one assignee
edge assigned_to(task: Task, person: Person) [task -> 0..1] {}

-- Each project has at least one task
edge contains(project: Project, task: Task) [project -> 1..*] {}

-- Each person can manage 0-10 people
edge manages(manager: Person, report: Person) [manager -> 0..10] {}
```

### 12.4.4 Bidirectional Cardinality

Multiple cardinality modifiers can be combined:

```
-- One-to-one: each person has at most one spouse, vice versa
edge married_to(a: Person, b: Person) [
  symmetric,
  a -> 0..1,
  b -> 0..1
] {}

-- Many-to-one: many tasks to one project
edge belongs_to(task: Task, project: Project) [
  task -> 1,          -- each task has exactly one project
  project -> 0..*     -- each project has any number of tasks (implicit)
] {}
```

### 12.4.5 Compilation

Cardinality modifiers compile to constraints:

```
edge assigned_to(task: Task, person: Person) [task -> 0..1] {}
```

Compiles to:
```
constraint assigned_to_task_max_1:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1), assigned_to(t, p2)
  WHERE p1.id != p2.id
  => false
```

For `[project -> 1..*]`:
```
constraint contains_project_min_1:
  p: Project
  => exists(t: Task, contains(p, t))
```

### 12.4.6 Cardinality Timing

Cardinality constraints are checked at **transaction commit**, not at each individual operation.

This allows staged creation within a transaction:

```
BEGIN
  SPAWN t: Task { title = "Test" }   -- OK: no project yet
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(t, p)              -- OK: now linked
COMMIT                               -- Cardinality checked here
```

If the transaction commits without satisfying `[task -> 1]`, the commit fails and rolls back.

**Rationale:** Immediate checking would prevent common patterns like creating entities before linking them.

### 12.4.7 Cardinality and Auto-Commit Mode

In **auto-commit mode** (no explicit `BEGIN`/`COMMIT`), each statement is its own transaction. Cardinality constraints are checked after each statement.

**Example with `[task -> 1]` on `belongs_to`:**

```
-- Explicit transaction (works):
BEGIN
  SPAWN t: Task { title = "Test" }
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(t, p)
COMMIT  -- ✅ Cardinality satisfied

-- Auto-commit mode (fails):
SPAWN t: Task { title = "Test" }
-- ❌ Immediate commit fails: task requires exactly 1 belongs_to edge
```

**Recommendation:** When cardinality constraints like `[task -> 1]` exist, use explicit transactions for multi-step entity creation.

### 12.4.8 Minimum vs Maximum Cardinality

Cardinality constraints have two components with different enforcement timing:

| Component | Syntax | When Enforced |
|-----------|--------|---------------|
| **Maximum** | `task -> ..1` | Immediately on LINK |
| **Minimum** | `task -> 1..` | At transaction COMMIT |

**Examples:**

```
-- Maximum only (common): Each task has at most one assignee
edge assigned_to(task: Task, person: Person) [task -> 0..1]
-- LINK checks: "does task already have an assignee?" (immediate)

-- Minimum only: Each task must have at least one tag (eventually)
edge tagged(task: Task, tag: Tag) [task -> 1..*]
-- COMMIT checks: "does task have at least one tag?" (deferred)

-- Both: Each task has exactly one project
edge belongs_to(task: Task, project: Project) [task -> 1..1]
-- LINK checks: max (immediate)
-- COMMIT checks: min (deferred)
```

**Design rationale:**
- Maximum constraints prevent invalid states immediately
- Minimum constraints allow staged construction within transactions
- This split enables natural workflows while maintaining invariants

**Error message example:**
```
Error: Cardinality constraint violated at commit
  Edge: belongs_to
  Constraint: task -> 1 (exactly one)
  Entity: Task (id: abc123, title: "Test")
  Actual: 0 edges
  Expected: 1 edge
  
Hint: Use BEGIN/COMMIT to create task and project together.
```

## 12.5 Referential Actions

Control what happens when a connected node is killed.

**Restriction:** Referential action modifiers (`on_kill_source`, `on_kill_target`) are only valid for **binary edges** (arity = 2). For n-ary edges with more than 2 parameters, use explicit rules to define referential behavior.

### 12.5.1 Syntax

```
[on_kill_source: action]   -- when first parameter's node is killed
[on_kill_target: action]   -- when second parameter's node is killed
```

### 12.5.2 Actions

| Action | Meaning |
|--------|---------|
| `unlink` | Remove the edge (default) |
| `cascade` | Kill the other endpoint(s) |
| `prevent` | Prevent the kill operation |

### 12.5.3 Examples

```
-- When project is killed, kill all its tasks
edge belongs_to(task: Task, project: Project) [
  on_kill_target: cascade
] {}

-- When user is killed, just remove edges (default)
edge created_by(entity: any, user: User) [
  on_kill_target: unlink
] {}

-- Cannot kill team if it has members
edge member_of(person: Person, team: Team) [
  on_kill_target: prevent
] {}

-- Killing a parent kills children
edge parent_of(parent: Person, child: Person) [
  on_kill_source: cascade
] {}
```

### 12.5.4 Default Behavior

If no referential action is specified, `unlink` is the default:
- Killing a node silently removes all connected edges
- No cascading, no prevention

### 12.5.5 Compilation

Referential actions compile to rules:

```
edge belongs_to(task: Task, project: Project) [on_kill_target: cascade] {}
```

Compiles to:
```
rule belongs_to_cascade_on_kill_target [priority: 1000]:
  t: Task, p: Project,
  belongs_to(t, p),
  _pending_kill(p)    -- internal marker
  =>
  KILL t
```

For `prevent`:
```
constraint member_of_prevent_kill_target:
  t: Team, p: Person,
  member_of(p, t),
  _pending_kill(t)
  => false
```

### 12.5.6 N-ary Edges: Use Explicit Rules

Referential action modifiers (`on_kill_source`, `on_kill_target`) are **only valid for binary edges** (arity = 2).

For edges with 3+ parameters, the compiler raises an error:

```
-- COMPILE ERROR:
edge meeting(a: Person, b: Person, c: Person) [on_kill_source: cascade] {}
-- Error: Referential actions only supported for binary edges (arity = 2).
--        Edge 'meeting' has arity 3.
--        Use explicit rules instead.
```

**Workaround:** Define explicit rules for the desired behavior:

```
edge meeting(a: Person, b: Person, c: Person) {}

-- Cascade: unlink meeting if any participant is killed
rule meeting_cascade_on_kill:
  p: Person, meeting(p, _, _) AS m, _pending_kill(p)
  =>
  UNLINK m

-- Or more specific: cascade only if organizer (first param) killed
rule meeting_cascade_organizer:
  organizer: Person, meeting(organizer, _, _) AS m, _pending_kill(organizer)
  =>
  UNLINK m
```

**Rationale:** For n-ary edges, "source" and "target" are ambiguous. Explicit rules give full control over which parameter triggers which action.

### 12.5.7 Cascade Depth Accounting

Cascade operations (from `on_kill_*: cascade`) **count toward the depth limit**.

**Default cascade depth limit: 100** (same as rule execution depth limit in §15.5.1)

**Engine configuration:**
```
SET engine.cascade_depth_limit = 100    -- default
SET engine.max_cascade_count = 10000    -- total entities per cascade
```

```
-- If depth limit is 100:
-- A → B → C → D ... (100 levels of cascade)
KILL A  -- Fails: cascade chain exceeds depth limit
```

**Example cascade chain:**
```
edge parent_of(parent: Org, child: Org) [on_kill_source: cascade] {}

-- Graph: RootOrg → Dept1 → Dept2 → ... → Dept100 → Dept101
KILL RootOrg
-- Cascades: RootOrg kills Dept1, Dept1 kills Dept2, ...
-- At Dept101: depth limit (100) exceeded → transaction fails
```

**Error message:**
```
ERROR [E5004]: Cascade depth limit exceeded
  Limit: 100 (engine.cascade_depth_limit)
  Chain: RootOrg → Dept1 → Dept2 → ... → Dept100
  Failed at: Dept101
  
Hint: Increase engine.cascade_depth_limit or restructure cascade relationships.
```

**Count limit (breadth):**
```
-- Wide cascade (many children, not deep)
edge member_of(person: Person, org: Org) [on_kill_target: cascade]

-- Org with 50,000 members
KILL BigOrg  -- Fails: cascade count (50,000) exceeds limit (10,000)

ERROR [E5005]: Cascade count limit exceeded
  Limit: 10,000 (engine.max_cascade_count)
  Affected: 50,000+ entities
  
Hint: Use KILL ... FORCE CASCADE to override, or delete in batches.
```

**Rationale:** Unbounded cascades can cause runaway deletions. The limits provide safety nets.

## 12.6 Combined Example

```
--- Task assignment with full constraints
edge assigned_to(task: Task, person: Person) [
  no_self,                        -- irrelevant here but showing syntax
  unique,                         -- no duplicate assignments
  task -> 0..1,                   -- each task has at most one assignee
  on_kill_target: unlink          -- kill person → remove assignment
] {
  --- When the assignment was made
  assigned_at: Timestamp [required] = now(),
  
  --- Who made the assignment
  assigned_by: String?,
  
  --- Role in this assignment
  role: String [in: ["owner", "reviewer", "observer"]] = "owner"
}
```

### 12.6.1 Edge Attribute Defaults at LINK Time

When LINKing an edge, attribute values can be:
1. **Explicitly provided:** `LINK e(a, b) { x = 1 }`
2. **Omitted (uses default):** `LINK e(a, b) { }` or `LINK e(a, b)`

If all attributes have defaults or are optional, the braces can be omitted:

```
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp = now(),      -- has default
  role: String = "owner"               -- has default
}

-- All valid:
LINK assigned_to(t, p)                           -- both use defaults
LINK assigned_to(t, p) { role = "reviewer" }     -- one explicit
LINK assigned_to(t, p) { assigned_at = now(), role = "reviewer" }
```

**Required attributes without defaults** must be provided:

```
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required],   -- no default!
  role: String = "owner"
}

LINK assigned_to(t, p)                 -- ❌ Error: assigned_at required
LINK assigned_to(t, p) { assigned_at = now() }  -- ✅ OK
```

## 12.7 Higher-Order Edges

Edges can target other edges using `edge<T>` types.

### 12.7.1 Syntax

```
edge confidence(about: edge<causes>, level: Float) {}
edge provenance(about: edge<any>, source: String) {}
```

### 12.7.2 Creating Higher-Order Edges

```
-- First, create the base edge and bind it
LINK causes(event1, event2) AS c

-- Then, create edge about it
LINK confidence(c, 0.85) { assessed_by = "expert" }
```

### 12.7.3 Matching Higher-Order Edges

```
MATCH 
  e1: Event, e2: Event,
  causes(e1, e2) AS c,
  confidence(c, level)
WHERE level > 0.5
RETURN e1, e2, level
```

### 12.7.4 Higher-Order Edge Lifecycle

When a base edge is unlinked, all higher-order edges that reference it are **automatically unlinked** (cascaded). This is implicit and cannot be disabled.

```
edge causes(from: Event, to: Event)
edge confidence(about: edge<causes>, level: Float)

-- Create edges
LINK causes(e1, e2) AS c
LINK confidence(c, 0.8)

-- Unlink base edge
UNLINK c
-- The confidence edge is automatically unlinked
```

**Rationale:** An edge reference (`edge<T>`) that points to a non-existent edge would be invalid state. Cascading deletion maintains referential integrity.

**Note:** This cascade is recursive. If edge A references edge B, and edge B references edge C, unlinking C will unlink B, which will unlink A.

## 12.8 AST

```typescript
interface EdgeTypeDecl {
  kind: "EdgeType"
  name: string
  signature: SignatureParam[]
  modifiers: EdgeModifiers
  attributes: AttributeDecl[]
  doc: string | null
}

interface SignatureParam {
  name: string
  type: TypeExpr
}

interface EdgeModifiers {
  symmetric: boolean
  noSelf: boolean
  acyclic: boolean
  unique: boolean
  indexed: boolean
  cardinality: CardinalityConstraint[]
  onKillSource: ReferentialAction
  onKillTarget: ReferentialAction
}

interface CardinalityConstraint {
  param: string
  min: number
  max: number | "*"
}

type ReferentialAction = "unlink" | "cascade" | "prevent"
```

## 12.9 Compilation

An `EdgeTypeDecl` compiles to:

```
_EdgeType node:
  name: <edge name>
  arity: <number of signature params>
  symmetric: <true/false>
  doc: <doc comment or null>

For each signature parameter at position i:
  _VarDef node:
    name: <param name>
    is_edge_var: <true if edge<T> type>
  
  _edge_has_position edge:
    (edge_type, var_def) { position: i }
  
  _var_has_type edge:
    (var_def, type_expr_node)

For each attribute:
  (same as node attributes)

For each modifier, generate constraints/rules:
  no_self → constraint (see 11.3.2)
  acyclic → constraint (see 11.3.3)
  cardinality → constraints (see 11.4.5)
  on_kill_* → rules (see 11.5.5)
```

---

# 13. Patterns

Patterns are the core construct for matching graph structure. They are used in:
- Constraint conditions
- Rule conditions
- MATCH clauses
- EXISTS expressions

## 13.1 Syntax

```
Pattern = PatternElement ("," PatternElement)* WhereClause?

PatternElement = NodePattern | EdgePattern

NodePattern = Identifier ":" TypeExpr

EdgePattern = EdgeType "(" TargetList ")" EdgeAlias?

TargetList = Target ("," Target)*

Target = Identifier | "_"

EdgeAlias = "as" Identifier

WhereClause = "where" Expr
```

## 13.2 Node Patterns

Bind variables to nodes of a type:

```
e: Event                    -- e is an Event
p: Person                   -- p is a Person
x: Task | Project           -- x is Task or Project
n: any                      -- n is any node
```

## 13.3 Edge Patterns

Match edges between nodes:

```
causes(e1, e2)              -- e1 and e2 connected by 'causes'
assigned_to(t, p)           -- t assigned to p
confidence(c, level)        -- confidence edge (higher-order)
```

### 13.3.1 Anonymous Targets

Use `_` to match without binding:

```
MATCH t: Task, assigned_to(t, _)
-- Matches tasks assigned to anyone
```

### 13.3.2 Edge Binding

Use `AS` to bind the edge itself:

```
MATCH e1: Event, e2: Event, causes(e1, e2) AS c
WHERE c.strength > 0.5
```

## 13.4 Where Clause

Add filtering conditions:

```
MATCH e: Event
WHERE e.timestamp > 1000 AND e.completed = true
```

## 13.5 Negative Patterns

Use `NOT EXISTS` for absence:

```
-- Tasks with no assignment
MATCH t: Task
WHERE NOT EXISTS(assigned_to(t, _))
```

## 13.6 Transitive Patterns

Use `+` for transitive closure and `*` for reflexive transitive closure:

```
-- Direct or indirect cause
causes+(e1, e2)           -- one or more hops

-- Zero or more levels of parenthood
parent_of*(p1, p2)        -- zero or more hops
```

**Used in acyclic constraints:**
```
constraint no_cycle:
  p: Person, parent_of+(p, p)
  => false
```

### 13.6.1 Semantics

| Operator | Meaning | Matches |
|----------|---------|---------|
| `edge+(a, b)` | Transitive closure | Path of 1+ edges from a to b |
| `edge*(a, b)` | Reflexive transitive | Path of 0+ edges (includes a = b) |

### 13.6.2 Depth Limits and Behavior

Transitive patterns have a **default depth limit of 100 hops**.

**Engine configuration:**
```
-- Engine-level default (applies when no explicit depth specified)
SET engine.default_transitive_depth = 100

-- Maximum allowed depth (even with explicit [depth: N])
SET engine.max_transitive_depth = 1000
```

**Behavior at depth limit:**

| Scenario | Behavior |
|----------|----------|
| Path found within limit | ✅ Match returned |
| Path exceeds limit | ⚠️ Warning + matches found so far |
| Cycle detected | ✅ Cycle terminated, other paths continue |
| No path exists | ❌ No match |

**Warning when limit reached:**
```
WARNING [E5010]: Transitive pattern reached depth limit
  Pattern: follows+(a, b)
  Limit: 100 (engine default)
  Paths truncated: 1,234
  
  Hint: Add explicit [depth: N] to increase limit or refine pattern.
```

**Example:**
```
-- Graph: A → B → C → D → ... (1000 nodes deep)
MATCH causes+(a, z)
-- Returns: matches for paths up to 100 hops
-- Warning: if paths were truncated
-- Does NOT return: paths requiring 101+ hops
```

**Customizing depth (in HOHG Language):**
```
MATCH causes+(e1, e2) [depth: 500]   -- Allow up to 500 hops
MATCH causes+(e1, e2) [depth: 10]    -- Limit to 10 hops (faster)
```

**In constraints:** Depth **can** be customized using the same `[depth: N]` syntax:
```
constraint shallow_acyclic:
  p: Person, parent_of+(p, p) [depth: 20]
  => false
```

**Note:** For performance-critical constraints, prefer explicit depth limits over unbounded transitive patterns.

### 13.6.3 Cycle Detection

Cycles are detected and handled gracefully:

```
-- Graph: A → B → C → A (cycle)
MATCH follows+(a, b) WHERE a.name = "A"
-- Returns: (A, B), (A, C), (A, A)
-- The A→B→C→A path terminates when revisiting A
-- No infinite loop
```

### 13.6.4 Restrictions

- Transitive patterns cannot bind intermediate nodes (path binding deferred to future version)
- Transitive patterns in constraints should be used sparingly due to performance cost
- For complex path observations, use the WALK statement in HOHG Language

## 13.7 AST

```typescript
interface Pattern {
  kind: "Pattern"
  elements: PatternElement[]
  where: Expr | null
}

interface NodePattern {
  kind: "NodePattern"
  variable: string
  type: TypeExpr
}

interface EdgePattern {
  kind: "EdgePattern"
  edgeType: string
  targets: (string | "_")[]
  alias: string | null
  transitive: "none" | "+" | "*"
}
```

---

# 14. Constraint Declarations

## 14.1 Syntax

```
ConstraintDecl =
  DocComment?
  "constraint" Identifier ConstraintModifiers? ":" Pattern "=>" Expr

ConstraintModifiers = "[" ConstraintModifier ("," ConstraintModifier)* "]"

ConstraintModifier = 
    "soft"
  | "hard"
  | "message:" StringLiteral
```

**Note:** Constraint names are **required**. Anonymous constraints are not allowed.

### 14.1.1 Constraint Names Required

Every constraint must have a name. This is a compile-time requirement.

```
-- COMPILE ERROR: Missing constraint name
constraint:
  t: Task => t.priority >= 0
-- Error: Constraint name required.
--        Add a name: constraint task_priority_valid: ...
```

**Rationale:** 
- Error messages reference constraint names for debugging
- Named constraints are observable and inspectable
- Forces developers to think about what the constraint means

**Good naming conventions:**
```
constraint task_priority_valid: ...       -- <entity>_<attribute>_<condition>
constraint temporal_order: ...            -- semantic name
constraint no_self_loop: ...              -- describes what's prevented
constraint task_needs_project: ...        -- describes requirement
```

## 14.2 Examples

```
-- Simple constraint
constraint no_self_loop:
  e: Event, causes(e, e)
  => false

-- With message
constraint temporal_order [message: "Cause must precede effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- Soft constraint (warning only)
constraint prefer_description [soft, message: "Tasks should have descriptions"]:
  t: Task
  => t.description != null

-- Existence requirement
constraint task_needs_project [message: "Every task must belong to a project"]:
  t: Task
  => exists(p: Project, belongs_to(t, p))
```

## 14.3 Constraint Modifiers

| Modifier | Meaning |
|----------|---------|
| `hard` | Violation rejects operation (default) |
| `soft` | Violation logs warning, allows operation |
| `message: "..."` | Custom error/warning message |

## 14.4 Reading Constraints

A constraint reads as: "For all matches of pattern, condition must hold."

```
constraint temporal_order:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp < e2.timestamp

-- "For all (e1, e2) where e1 causes e2,
--  e1.timestamp must be less than e2.timestamp"
```

## 14.5 Common Patterns

### Prohibition
```
constraint no_self_cause:
  e: Event, causes(e, e)
  => false
```

### Requirement
```
constraint task_has_project:
  t: Task
  => exists(p: Project, belongs_to(t, p))
```

### Implication
```
constraint completed_has_timestamp:
  t: Task WHERE t.status = "completed"
  => t.completed_at != null
```

### Uniqueness
```
constraint unique_email:
  p1: Person, p2: Person
  WHERE p1.id != p2.id
  => p1.email != p2.email
```

## 14.6 AST

```typescript
interface ConstraintDecl {
  kind: "Constraint"
  name: string | null
  modifiers: {
    hard: boolean
    message: string | null
  }
  pattern: Pattern
  condition: Expr
  doc: string | null
}
```

---

# 15. Rule Declarations

## 15.1 Syntax

```
RuleDecl =
  DocComment?
  "rule" Identifier? RuleModifiers? ":" Pattern "=>" Production

RuleModifiers = "[" RuleModifier ("," RuleModifier)* "]"

RuleModifier = 
    "priority:" IntLiteral
  | "auto"
  | "manual"

Production = Action ("," Action)*

Action =
    SpawnAction
  | KillAction
  | LinkAction
  | UnlinkAction
  | SetAction

SpawnAction = "spawn" Identifier ":" TypeExpr ("{" AttrAssignments "}")?

KillAction = "kill" Identifier

LinkAction = "link" EdgeType "(" TargetList ")" ("as" Identifier)? ("{" AttrAssignments "}")?

UnlinkAction = "unlink" Identifier

SetAction = "set" AttrAccess "=" Expr

AttrAssignments = (AttrAssignment ","?)*
AttrAssignment = Identifier "=" Expr
```

## 15.2 Examples

```
-- Inference rule
rule transitive_causes [priority: 10]:
  e1: Event, e2: Event, e3: Event,
  causes(e1, e2), causes(e2, e3)
  WHERE NOT EXISTS(causes(e1, e3))
  =>
  LINK causes(e1, e3) { mechanism = "transitive" }

-- Auto-timestamp rule
rule auto_created_at [priority: 100]:
  e: Entity WHERE e.created_at = null
  =>
  SET e.created_at = now()

-- Multi-action rule
rule on_task_complete:
  t: Task
  WHERE t.status = "completed" AND t.completed_at = null
  =>
  SET t.completed_at = now(),
  SPAWN n: Notification { message = "Task done: " ++ t.title },
  LINK notifies(n, t)

-- Manual rule (not auto-fired)
rule archive_old [manual]:
  t: Task
  WHERE t.completed_at < now() - 2592000000  -- 30 days
  =>
  SET t.archived = true
```

## 15.3 Rule Modifiers

| Modifier | Meaning |
|----------|---------|
| `priority: N` | Execution order (higher = first). Default: 0 |
| `auto` | Fire automatically when pattern matches (default) |
| `manual` | Only fire when explicitly invoked |

### 15.3.1 Priority and Execution Order

Rules execute in **priority order** (highest priority first).

**Tie-breaking:** When multiple rules have equal priority:
1. Rules execute in **declaration order** (order in source file)
2. Across ontology inheritance, parent rules execute before child rules at equal priority

```
-- These rules have equal priority (default 0)
rule a: t: Task => SET t.x = 1    -- Executes first (declared first)
rule b: t: Task => SET t.x = 2    -- Executes second

-- This rule executes before both (higher priority)
rule c [priority: 10]: t: Task => SET t.x = 3
```

**Execution order for the above:** `c` (priority 10), then `a` (priority 0, first declared), then `b` (priority 0, second declared).

**Rationale:** Declaration order provides deterministic behavior without requiring explicit priorities on every rule.

## 15.4 Actions

| Action | Syntax | Effect |
|--------|--------|--------|
| SPAWN | `SPAWN x: Type { attrs }` | Create node |
| KILL | `KILL x` | Remove node |
| LINK | `LINK edge(a, b) { attrs }` | Create edge |
| UNLINK | `UNLINK e` | Remove edge |
| SET | `SET x.attr = value` | Modify attribute |

## 15.5 Execution Semantics

### 15.5.0 Rule vs Constraint Execution Order

**Critical:** Rules execute **before** constraint checking within each iteration.

```
Transaction Execution Order:
┌─────────────────────────────────────────────────────────────────┐
│ 1. User transformation (SPAWN, KILL, LINK, UNLINK, SET)        │
│ 2. Find triggered rules                                         │
│ 3. Execute rules in priority order                              │
│ 4. Check constraints (hard constraints fail → rollback ALL)     │
│ 5. If rules modified data, go to step 2 (until quiescence)     │
│ 6. Commit                                                       │
└─────────────────────────────────────────────────────────────────┘
```

**Why this matters:**

```
-- Constraint that would fail without rule:
constraint task_has_timestamp:
  t: Task => t.created_at != null

-- Rule that satisfies the constraint:
rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()

-- User action:
SPAWN t: Task { title = "Test" }  -- No created_at provided

-- Execution:
-- 1. SPAWN creates task without created_at
-- 2. auto_timestamp rule fires, sets created_at = now()
-- 3. task_has_timestamp constraint checked → PASSES
-- 4. Commit
```

**If constraint checked first:**
```
-- WRONG (not how it works):
-- 1. SPAWN creates task without created_at
-- 2. task_has_timestamp checked → FAILS
-- 3. Rollback (rule never gets to run)
```

**Key principle:** Rules can "fix" constraint violations, enabling patterns like:
- Auto-population of required fields
- Derived/computed values
- Cascading updates

**Constraint failure rollback:** If a hard constraint fails **after** rules execute, the **entire transaction** rolls back, including all rule effects.

### 15.5.1 Execution Limits

To prevent infinite loops and runaway rule chains:

| Limit | Default | Description |
|-------|---------|-------------|
| **Same-binding limit** | 1 | A (rule, variable-bindings) pair can execute at most once per transaction |
| **Action limit** | 10,000 | Maximum rule actions per transaction |
| **Depth limit** | 100 | Maximum nested rule triggers |

When any limit is exceeded, the transaction fails and rolls back with a descriptive error.

**Example of same-binding protection:**
```
rule bad_loop:
  t: Task WHERE t.count < 100
  =>
  SET t.count = t.count + 1
```

This rule would fire once for each Task, increment count, then stop (same binding already executed). It will NOT loop 100 times per task.

### 15.5.2 `now()` Evaluation Semantics

The `now()` function returns the current timestamp. Its evaluation timing depends on context:

| Context | When Evaluated | Consistency | Allowed |
|---------|----------------|-------------|---------|
| Attribute default `= now()` | At SPAWN/LINK execution | Per-statement | ✅ Yes |
| Rule SET/SPAWN/LINK | At rule action execution | Same within rule | ✅ Yes |
| Rule WHERE clause | At pattern match time | Same within rule | ✅ Yes |
| Explicit `{ timestamp = now() }` | At SPAWN/LINK execution | Per-statement | ✅ Yes |
| Constraint condition | Would be non-deterministic | — | ❌ No |

**Within a single rule execution**, `now()` returns the same value for all actions:

```
rule set_timestamps:
  t: Task WHERE t.updated_at = null
  =>
  SET t.updated_at = now(),
  SET t.processed_at = now()   -- Same value as above
```

**Across statements in interactive mode**, `now()` is evaluated fresh:

```
SPAWN a: Event { timestamp = now() }
-- (1 second passes)
SPAWN b: Event { timestamp = now() }
-- a.timestamp ≠ b.timestamp (different statements)
```

**Within a single statement with multiple entities:**

```
-- Same statement, same now() value:
BEGIN
  SPAWN a: Event { timestamp = now() }
  SPAWN b: Event { timestamp = now() }
COMMIT
-- a.timestamp = b.timestamp (same transaction's start time)
```

**Constraint restriction:** Using `now()` in constraint conditions is a compile-time error:

```
-- COMPILE ERROR: now() not allowed in constraint conditions
constraint recent_only:
  t: Task
  => t.created_at > now() - 86400000
-- Error: now() cannot appear in constraint conditions.
--        Constraints must be deterministic.
```

**Workaround for time-based validation:**
1. Use manual rules triggered periodically
2. Handle in application code
3. Store a reference timestamp and compare against that

### 15.5.3 `now()` in Rule WHERE: Trigger-Based Evaluation

Rules are evaluated **when their pattern could newly match** due to a transformation, not continuously.

```
rule cleanup_old [manual]:
  t: Task WHERE t.created_at < now() - 86400000  -- 30 days ago
  => SET t.archived = true
```

**Key behavior:**
- This rule does **not** automatically fire when a Task becomes 30 days old
- It fires when explicitly invoked (manual rule) or when a Task is modified (auto rule)
- `now()` is evaluated at the moment of pattern matching

**Implication:** Time-based automation requires external triggers:

```
-- Option 1: Manual rule invoked by scheduler
rule archive_old_tasks [manual]:
  t: Task WHERE t.created_at < now() - 2592000000  -- 30 days
  => SET t.archived = true

-- External scheduler calls: INVOKE archive_old_tasks

-- Option 2: Application-level time check
-- Application periodically observations and updates
```

**Rationale:** Continuous time-based evaluation would require the engine to wake up and re-evaluate rules constantly, which is expensive and complex. Trigger-based evaluation is simpler and more predictable.

## 15.6 AST

```typescript
interface RuleDecl {
  kind: "Rule"
  name: string | null
  modifiers: {
    priority: number
    auto: boolean
  }
  pattern: Pattern
  production: Action[]
  doc: string | null
}

type Action = 
  | { kind: "Spawn", variable: string, type: TypeExpr, attributes: AttrAssignment[] }
  | { kind: "Kill", variable: string }
  | { kind: "Link", edgeType: string, targets: string[], alias: string | null, attributes: AttrAssignment[] }
  | { kind: "Unlink", variable: string }
  | { kind: "Set", target: AttrAccess, value: Expr }
```

---

# 16. Complete Grammar

For shared constructs (Types, Expressions, Literals, Lexical), see **Part I: Foundations §6.3**.

```ebnf
(* Top Level *)
OntologyFile     = OntologyDecl? Declaration*
OntologyDecl     = "ontology" Identifier (":" QualIdent ("," QualIdent)*)? "{" Declaration* "}"
Declaration      = TypeAliasDecl | NodeTypeDecl | EdgeTypeDecl | ConstraintDecl | RuleDecl

(* Type Aliases *)
TypeAliasDecl    = "type" Identifier "=" TypeExpr AttrModifiers?
                 | "type" Identifier "=" TypeExpr ("|" TypeExpr)+

(* Node Types *)
NodeTypeDecl     = DocComment? "node" Identifier (":" QualIdent ("," QualIdent)*)? "{" AttributeDecl* "}"

(* Attributes *)
AttributeDecl    = DocComment? Identifier ":" TypeExpr AttrModifiers? DefaultValue? ","?
AttrModifiers    = "[" AttrModifier ("," AttrModifier)* "]"
AttrModifier     = "required" | "unique" 
                 | "indexed" (":" ("asc" | "desc"))?
                 | ">=" Literal | "<=" Literal | ">" Literal | "<" Literal
                 | IntLiteral ".." IntLiteral
                 | "in:" "[" Literal ("," Literal)* "]"
                 | "match:" StringLiteral
                 | "length:" IntLiteral ".." IntLiteral
DefaultValue     = "=" (Literal | ConstantExpr)
ConstantExpr     = "now()"

(* Edge Types *)
EdgeTypeDecl     = DocComment? "edge" Identifier "(" SigParams ")" EdgeModifiers? ("{" AttributeDecl* "}")?
SigParams        = SigParam ("," SigParam)*
SigParam         = Identifier ":" TypeExpr
EdgeModifiers    = "[" EdgeModifier ("," EdgeModifier)* "]"
EdgeModifier     = "symmetric" | "no_self" | "acyclic" | "unique" | "indexed"
                 | CardinalityMod | ReferentialMod
CardinalityMod   = Identifier "->" Cardinality
Cardinality      = IntLiteral | IntLiteral ".." IntLiteral | IntLiteral ".." "*"
ReferentialMod   = ("on_kill_source:" | "on_kill_target:") RefAction
RefAction        = "cascade" | "unlink" | "prevent"

(* Constraints *)
ConstraintDecl   = DocComment? "constraint" Identifier ConstMods? ":" Pattern "=>" Expr
ConstMods        = "[" ConstMod ("," ConstMod)* "]"
ConstMod         = "soft" | "hard" | "message:" StringLiteral

(* Rules *)
RuleDecl         = DocComment? "rule" Identifier? RuleMods? ":" Pattern "=>" Production
RuleMods         = "[" RuleMod ("," RuleMod)* "]"
RuleMod          = "priority:" IntLiteral | "auto" | "manual"
Production       = Action ("," Action)*
Action           = SpawnAction | KillAction | LinkAction | UnlinkAction | SetAction
SpawnAction      = "spawn" Identifier ":" TypeExpr ("{" AttrAssigns "}")?
KillAction       = "kill" Identifier
LinkAction       = "link" Identifier "(" Targets ")" ("as" Identifier)? ("{" AttrAssigns "}")?
UnlinkAction     = "unlink" Identifier
SetAction        = "set" AttrAccess "=" Expr
AttrAssigns      = (AttrAssign ","?)*
AttrAssign       = Identifier "=" Expr

(* Patterns *)
Pattern          = PatternElem ("," PatternElem)* WhereClause?
PatternElem      = NodePattern | EdgePattern
NodePattern      = Identifier ":" TypeExpr
EdgePattern      = Identifier TransitiveMod? "(" Targets ")" ("as" Identifier)?
TransitiveMod    = "+" | "*"
Targets          = Target ("," Target)*
Target           = Identifier | "_"
WhereClause      = "where" Expr
AttrAccess       = Identifier ("." Identifier)+
```

---

# 17. Complete Example

A compact TaskManagement ontology demonstrating all DSL features:

```
ontology TaskManagement : Layer0 {

  -- Type aliases
  type Email = String [match: "^.+@.+\\..+$"]
  type Priority = Int [>= 0, <= 10]
  type TaskStatus = String [in: ["todo", "in_progress", "done", "blocked"]]

  -- Nodes
  node Person {
    name: String [required, length: 1..100],
    email: Email [required, unique, indexed],
    role: String [in: ["admin", "member", "guest"]] = "member"
  }

  node Team { name: String [required, unique] }
  node Project { name: String [required], status: String = "active", deadline: Timestamp? }
  node Task {
    title: String [required],
    status: TaskStatus = "todo",
    priority: Priority = 5,
    completed_at: Timestamp?
  }
  node Tag { name: String [required, unique, match: "^[a-z0-9-]+$"] }

  -- Edges
  edge member_of(person: Person, team: Team) [unique] { role: String = "member" }
  edge owns(team: Team, project: Project) [project -> 1, on_kill_source: prevent]
  edge belongs_to(task: Task, project: Project) [task -> 1, on_kill_target: cascade]
  edge assigned_to(task: Task, person: Person) [task -> 0..1, on_kill_target: unlink]
  edge depends_on(downstream: Task, upstream: Task) [no_self, acyclic]
  edge subtask_of(child: Task, parent: Task) [no_self, acyclic, child -> 0..1]
  edge tagged(task: Task, tag: Tag) [unique]

  -- Constraints
  constraint completed_has_timestamp [message: "Completed tasks must have completed_at"]:
    t: Task WHERE t.status = "done" => t.completed_at != null

  constraint subtask_same_project [message: "Subtask must be in same project"]:
    child: Task, parent: Task, p1: Project, p2: Project,
    subtask_of(child, parent), belongs_to(child, p1), belongs_to(parent, p2)
    => p1.id = p2.id

  -- Rules
  rule auto_complete_timestamp [priority: 10]:
    t: Task WHERE t.status = "done" AND t.completed_at = null
    => SET t.completed_at = now()

  rule auto_unblock [priority: 8]:
    t: Task WHERE t.status = "blocked"
      AND NOT EXISTS(upstream: Task, depends_on(t, upstream) WHERE upstream.status != "done")
    => SET t.status = "todo"
}
```

---

# 18. Summary

## 18.1 Modifier Quick Reference

| Modifier | Compiles To |
|----------|-------------|
| `[required]` | Constraint |
| `[unique]` | Constraint + Index |
| `[indexed]` | Engine index |
| `[>= N]`, `[<= M]`, `[N..M]` | Constraint |
| `[in: [...]]`, `[match: "..."]`, `[length: N..M]` | Constraint |
| `[no_self]`, `[acyclic]` | Constraint |
| `[a -> 0..1]` | Constraint (at commit) |
| `[symmetric]` | Storage + matching |
| `[on_kill_*: cascade\|unlink\|prevent]` | Rule (binary only) |

## 18.2 Key Semantics

- **Cardinality**: Checked at COMMIT, not per-operation
- **Referential actions**: Binary edges only; use rules for n-ary
- **Rule ordering**: Priority → declaration order → parent before child
- **`now()`**: Allowed in defaults/rules (evaluated once), forbidden in constraints
- **NULL handling**: Value modifiers skip NULL; use `[required]` to enforce

## 18.3 Limits

| Limit | Default |
|-------|---------|
| Same-binding | 1 |
| Action limit | 10,000 |
| Depth limit | 100 |

---

*End of Part II: Ontology DSL*