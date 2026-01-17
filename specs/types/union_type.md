---
spec: union_type
version: "1.0"
status: draft
category: type
capability: polymorphic_types
requires: []
priority: common
---

# Spec: Union Type

## Overview

The union type (`T | U`) enables polymorphic typing by allowing a value to be one of several possible types. Union types are essential for expressing heterogeneous relationships in graph schemas, such as edges that can connect to multiple node types, and for pattern matching across type variants. They provide static type safety while enabling flexible graph modeling patterns.

## Syntax

### Grammar

```ebnf
UnionType        = OptionalType ("|" OptionalType)*
OptionalType     = PrimaryType "?"?
PrimaryType      = NamedType | EdgeRefType | AnyType | ScalarType | "(" TypeExpr ")"

TypeAliasDecl    = "type" Identifier "=" TypeExpr ("|" TypeExpr)+
```

### Keywords

| Keyword | Context |
|---------|---------|
| `\|` | Type operator (infix) |
| `type` | Type alias declaration |
| `:` | Type annotation / type check expression |

### Examples

```
-- Union of node types
owner: Person | Organization
entity: Task | Project | Milestone

-- Union of scalar types
identifier: String | Int

-- Union with optional (binds looser than ?)
item: Task | Project?           -- means: Task | (Project?)
maybe_item: (Task | Project)?   -- means: (Task | Project) or null

-- Type alias for union
type Entity = Person | Organization | Bot
type Assignable = Task | Issue | Story

-- Edge with union signature
edge owns(owner: Person | Organization, asset: Asset)
edge tagged(entity: any, tag: Tag)
```

## Semantics

### Union Formation Rules

1. **Associativity:**
   - `A | B | C` is equivalent to `(A | B) | C` (left-associative)
   - Internally normalized to flat union: `{A, B, C}`

2. **Commutativity:**
   - `A | B` is equivalent to `B | A` (order does not matter)

3. **Idempotence:**
   - `A | A` is equivalent to `A`

4. **Identity:**
   - There is no identity element (no "bottom" type)

### Precedence with Optional

Union binds looser than the optional modifier:

```
Task | Project?        -- means: Task | (Project?)
                       -- A Task, or an optional Project

(Task | Project)?      -- means: (Task | Project) or null
                       -- Either a Task or Project, or null
```

### Type Compatibility (Subtyping)

```
T <: T | U             -- T is subtype of union containing T
U <: T | U             -- U is subtype of union containing U
A | B <: A | B | C     -- subset union is subtype of superset union

Child <: Parent        -- if Child inherits from Parent
Child <: Parent | X    -- inheritance applies within unions
```

### Assignment Rules

When assigning value of type S to location of type T:

| Source Type | Target Type | Valid? |
|-------------|-------------|--------|
| `Task` | `Task \| Project` | Yes |
| `Project` | `Task \| Project` | Yes |
| `Task \| Project` | `Task` | No (needs type narrowing) |
| `Task \| Project` | `Task \| Project \| Milestone` | Yes |
| `Task \| Project \| Milestone` | `Task \| Project` | No |

### Attribute Access on Union Types

When accessing an attribute on a union type `A | B`:

**Common attribute (same type):** Valid if attribute exists on ALL union members with compatible types:

```
-- Both Task and Project have 'name: String'
x: Task | Project
x.name              -- Valid, type is String
```

**Common attribute (different types):** Result type is union of attribute types:

```
-- Task.meta: String, Project.meta: Int
x: Task | Project
x.meta              -- Valid, type is String | Int
```

**Disjoint attribute:** Compile-time error if attribute missing on any member:

```
-- Task has 'priority', Project does not
x: Task | Project
x.priority          -- ERROR: 'priority' not found on Project
```

### Type Check Expression

Use the `:` operator to check runtime type:

```
x: Task | Project

x:Task              -- true if x is a Task
x:Project           -- true if x is a Project
```

Type checks follow inheritance:

```
-- TeamLead inherits from Employee which inherits from Person
x: Person | Bot

x:Person            -- true for Person, Employee, or TeamLead
x:Employee          -- true only for Employee or TeamLead
x:TeamLead          -- true only for TeamLead
```

### Pattern Matching with Unions

In patterns, union types match any constituent:

```
edge owns(owner: Person | Organization, asset: Asset)

-- Matches any owns edge, regardless of owner type
MATCH o: Person | Organization, a: Asset, owns(o, a)
RETURN o, a

-- Match specific owner type
MATCH p: Person, a: Asset, owns(p, a)
RETURN p.name, a.name

-- Use type check in WHERE
MATCH o: Person | Organization, a: Asset, owns(o, a)
WHERE o:Organization
RETURN o.company_name, a.name
```

## Layer 0

### Nodes

```
_UnionTypeExpr node:
  -- Represents a union type in the AST
  member_count: Int    -- Number of types in the union
```

### Edges

```
_union_member edge:
  (union_type_expr, member_type_expr) { position: Int }
  -- Links union to each member type with ordering
```

### Constraints

None. Union type behavior is built into the type system.

## Examples

### Type Aliases for Unions

```
-- Define reusable union types
type Entity = Person | Organization | Bot
type Assignable = Task | Issue | Story | Bug
type Temporal = Event | State | Action

-- Use in node definitions
node AuditLog {
  timestamp: Timestamp [required],
  actor: Entity,
  action: String [required]
}

-- Use in edge signatures
edge created_by(item: Assignable, creator: Entity)
edge observed_at(entity: Temporal, location: Location)
```

### Polymorphic Edges

```
-- Edge that can connect different source types
edge owns(
  owner: Person | Organization,
  asset: Asset
) {
  acquired_at: Timestamp = now(),
  ownership_type: String [in: ["full", "partial", "leased"]] = "full"
}

-- Higher-order edge with union
edge reviewed(
  item: Document | Code | Design,
  reviewer: Person
) {
  status: String [in: ["pending", "approved", "rejected"]],
  comments: String?
}
```

### Query with Union Types

```
-- Match any owner type
MATCH o: Person | Organization, a: Asset, owns(o, a)
WHERE a.value > 10000
RETURN o, a.name, a.value

-- Filter by specific type within union
MATCH o: Person | Organization, a: Asset, owns(o, a)
WHERE o:Person AND a.category = "vehicle"
RETURN o.name AS owner_name, a.name AS asset_name

-- Access common attributes
MATCH e: Task | Issue | Story
WHERE e.status = "open"
RETURN e.title, e.priority

-- Handle type-specific attributes via type check
MATCH item: Task | Issue
WHERE item:Task AND item.priority > 5
   OR item:Issue AND item.severity = "critical"
RETURN item.title
```

### Constraint with Union Types

```
-- Constraint applies to all union members
constraint entity_needs_name:
  e: Person | Organization | Bot
  => e.name != null

-- Type-specific constraint within union context
constraint org_needs_registration:
  o: Person | Organization
  WHERE o:Organization
  => o.registration_id != null
```

### Rule with Union Types

```
-- Rule matching union type
rule notify_high_value_transfer:
  from: Person | Organization,
  to: Person | Organization,
  a: Asset,
  owns(from, a) AS old_ownership
  WHERE a.value > 100000
  =>
  SPAWN n: Notification {
    message = "High-value transfer detected",
    value = a.value
  },
  LINK about(n, a)

-- Type-aware rule
rule auto_verify_org_assets:
  o: Person | Organization, a: Asset, owns(o, a)
  WHERE o:Organization AND a.requires_verification = true
  =>
  SET a.verified = true,
  SET a.verified_at = now()
```

### Union Expansion from Aliases

```
-- Type alias definition
type Entity = Person | Organization

-- Usage in edge
edge owns(owner: Entity, asset: Asset)

-- Compiles equivalently to:
edge owns(owner: Person | Organization, asset: Asset)
```

### Nested Unions

```
-- Unions can be nested with aliases
type Individual = Person | Bot
type LegalEntity = Organization | Government
type Actor = Individual | LegalEntity

-- Expands to:
-- Actor = Person | Bot | Organization | Government

-- Use in schema
node Action {
  performed_by: Actor,
  timestamp: Timestamp [required]
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Empty union | `Syntax error: Union type requires at least two member types` |
| Attribute not on all members | `Type error: Attribute 'attr' not found on type 'T' in union 'T \| U'` |
| Assigning supertype to union | `Type error: Cannot assign 'T \| U' to 'T' without type narrowing` |
| Invalid type in union | `Type error: Unknown type 'X' in union 'T \| X'` |
| Recursive union alias | `Compile error: Recursive type alias 'T' not allowed` |
| Union alias with modifiers | `Compile error: Union type aliases cannot have modifiers` |
| Incompatible union member types in edge | `Type error: Edge parameter types must be compatible: 'T' and 'U' have no common operations` |
