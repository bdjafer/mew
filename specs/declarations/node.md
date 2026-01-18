---
spec: node-declaration
version: "1.0"
status: draft
category: declaration
capability: schema-definition
requires: [types, attributes, inheritance]
priority: essential
---

# Spec: Node Type Declaration

## Overview

Node type declarations define the kinds of entities that can exist in the graph. Each node type specifies a name, optional parent types for inheritance, and a set of typed attributes with optional modifiers and defaults. Node types form the structural foundation of an ontology, enabling type-safe pattern matching, attribute validation, and polymorphic queries.

## Syntax

### Grammar

```ebnf
NodeTypeDecl     = DocComment? NodeModifiers? "node" Identifier InheritanceClause? "{" AttributeDecl* "}"

NodeModifiers    = "[" NodeModifier ("," NodeModifier)* "]"

NodeModifier     = "abstract" | "sealed"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*

AttributeDecl    = DocComment? Identifier ":" TypeExpr AttrModifiers? DefaultValue? ","?

AttrModifiers    = "[" AttrModifier ("," AttrModifier)* "]"

AttrModifier     = "required" | "unique" | "readonly"
                 | "indexed" (":" ("asc" | "desc"))?
                 | ">=" Literal | "<=" Literal | ">" Literal | "<" Literal
                 | IntLiteral ".." IntLiteral
                 | "in:" "[" Literal ("," Literal)* "]"
                 | "format:" FormatName
                 | "match:" StringLiteral
                 | "length:" IntLiteral ".." IntLiteral

FormatName       = "email" | "url" | "uuid" | "slug" | "phone"
                 | "iso_date" | "iso_datetime" | "ipv4" | "ipv6"

DefaultValue     = "=" (Literal | ConstantExpr)

ConstantExpr     = "now()" | DurationLiteral
                 | ConstantExpr ("+" | "-" | "*" | "/") ConstantExpr

DocComment       = "---" Text Newline
```

### Keywords

| Keyword | Context |
|---------|---------|
| `node` | Declaration - introduces a node type |
| `abstract` | Modifier - node type cannot be instantiated directly |
| `sealed` | Modifier - node type cannot be inherited from |
| `required` | Attribute modifier - value must be non-null |
| `unique` | Attribute modifier - value must be unique across instances |
| `readonly` | Attribute modifier - value cannot be modified after creation |
| `indexed` | Attribute modifier - create index for efficient lookup |

### Examples

```mew
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
  email: String [required, unique, format: email],
  age: Int [>= 0, <= 150],
  role: String [in: ["admin", "member", "guest"]] = "member",
  active: Bool = true
}

-- Multiple inheritance
node Document : Named, Timestamped {
  content: String
}

-- Readonly attribute (cannot be modified after creation)
node Bookmark {
  url: String [required],
  created_at: Timestamp [readonly] = now()
}

-- Abstract node type
[abstract]
node Entity {
  id: String [required, unique],
  created_at: Timestamp [readonly] = now()
}

-- Sealed node type
[sealed]
node SystemConfig {
  key: String [required, unique],
  value: String [required]
}
```

## Semantics

### Node Type Identity

Each node type has a unique name within its ontology. The name is used for:
- Pattern matching: `p: Person` matches nodes of type Person
- SPAWN statements: `SPAWN p: Person { ... }`
- Type checking: attribute access is validated against declared attributes

### Type Inheritance

When type B inherits from type A:

| Inherited | Behavior |
|-----------|----------|
| Attributes | B has all attributes of A |
| Constraints | Constraints on A apply to B |
| Pattern matching | Pattern `a: A` matches instances of B |

**Single Inheritance:**
```mew
node Animal {
  name: String
}

node Dog : Animal {
  breed: String
}
```

`Dog` inherits `name` from `Animal`. A `Dog` instance has both `name` and `breed`.

**Multiple Inheritance:**
```mew
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

**Diamond Resolution:**

If the same attribute is inherited through multiple paths from the same origin, it appears once. If different parents define the same attribute name with incompatible types, the ontology is invalid and produces a compile-time error.

**Polymorphic Attribute Access:**

When matching a base type, subtype-specific attributes are accessible with null-safe semantics:

```mew
node Product { name: String }
node PhysicalProduct : Product { weight_kg: Float }
node DigitalProduct : Product { file_size_mb: Float }

-- Query matching base type can access subtype attributes
MATCH p: Product
WHERE p.weight_kg != null AND p.weight_kg < 1.0
RETURN p.name, p.weight_kg
-- Returns only PhysicalProduct instances with weight_kg < 1.0
```

**Semantics:**
- Accessing a subtype attribute on the base type compiles successfully if the attribute exists on any subtype
- At runtime, accessing the attribute returns `null` if the actual instance doesn't have it
- Use null checks (`!= null`) to filter to instances that have the attribute
- Alternative: use type checking (`p:PhysicalProduct`) for explicit type narrowing

This enables flexible polymorphic queries while maintaining type safety through null handling.

### Abstract and Sealed Modifiers

**Abstract:** An abstract node type cannot be instantiated directly. It serves as a base type for inheritance only.

```mew
[abstract]
node Entity {
  id: String [required]
}

node Person : Entity {
  name: String
}

-- Valid: Person is concrete
SPAWN p: Person { id = "123", name = "Alice" }

-- Error: Cannot instantiate abstract type Entity
SPAWN e: Entity { id = "456" }
```

**Sealed:** A sealed node type cannot be inherited from. It is a final type.

```mew
[sealed]
node AuditLog {
  action: String [required],
  timestamp: Timestamp [required]
}

-- Error: Cannot inherit from sealed type AuditLog
node ExtendedLog : AuditLog {
  details: String
}
```

**Combining Abstract and Sealed:** Using both `[abstract, sealed]` is a compile-time error since an abstract type must be inheritable to be useful.

### Attribute Nullability and Requirements

| Declaration | Nullable? | Must provide at SPAWN? | Omitted behavior |
|-------------|-----------|------------------------|------------------|
| `x: T` | No | No | Compile warning |
| `x: T?` | Yes | No | Value is `null` |
| `x: T = default` | No | No | Value is `default` |
| `x: T [required]` | No | Yes | Error if omitted |
| `x: T? [required]` | -- | -- | Compile error (contradictory) |

**Best practice:** Every non-nullable attribute should either:
1. Have a default value (`= value`), OR
2. Be marked `[required]`

### Default Values

Defaults can be literals or constant expressions:

```mew
node Task {
  count: Int = 0,
  name: String = "unnamed",
  active: Bool = true,
  created_at: Timestamp = now(),
  expires_at: Timestamp = now() + 7.days
}
```

Constant expressions are evaluated at entity creation time. Attribute references and non-pure function calls are not allowed in defaults.

## Layer 0

### Nodes

```
_NodeType:
  name: String          -- the node type name
  abstract: Bool        -- whether type is abstract
  sealed: Bool          -- whether type is sealed
  doc: String?          -- documentation comment

_AttributeDef:
  name: String          -- attribute name
  required: Bool        -- whether required
  unique: Bool          -- whether unique
  readonly: Bool        -- whether readonly (immutable after creation)
  indexed: String       -- "none" | "asc" | "desc"
  default_value: Any?   -- serialized default or null
  doc: String?          -- documentation comment
```

### Edges

```
_type_inherits(child: _NodeType, parent: _NodeType)
  -- Inheritance relationship between types

_type_has_attribute(type: _NodeType, attr: _AttributeDef)
  -- Associates attribute definitions with their owning type

_attr_has_type(attr: _AttributeDef, type: _TypeExpr)
  -- Associates attribute with its type expression
```

### Constraints

```
-- For each [required] attribute:
constraint <type>_<attr>_required:
  x: <Type> WHERE x.<attr> = null
  => false

-- For each [unique] attribute:
constraint <type>_<attr>_unique:
  x1: <Type>, x2: <Type>
  WHERE x1.id != x2.id AND x1.<attr> = x2.<attr> AND x1.<attr> != null
  => false

-- For each [>= N] modifier:
constraint <type>_<attr>_min:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> >= N

-- For each [<= M] modifier:
constraint <type>_<attr>_max:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> <= M

-- For each [in: [...]] modifier:
constraint <type>_<attr>_enum:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> IN [values]

-- For each [format: F] modifier:
constraint <type>_<attr>_format:
  x: <Type> WHERE x.<attr> != null
  => is_<format>(x.<attr>)

-- For each [match: "..."] modifier:
constraint <type>_<attr>_match:
  x: <Type> WHERE x.<attr> != null
  => matches(x.<attr>, "<pattern>")

-- For each [length: N..M] modifier:
constraint <type>_<attr>_length:
  x: <Type> WHERE x.<attr> != null
  => length(x.<attr>) >= N AND length(x.<attr>) <= M
```

## Examples

### Domain Model with Inheritance

```mew
-- Abstract base for all entities
[abstract]
node Entity {
  id: String [required, unique, format: uuid],
  created_at: Timestamp [required] = now(),
  updated_at: Timestamp?
}

-- Named mixin
[abstract]
node Named {
  name: String [required, length: 1..200]
}

-- Concrete types
node Person : Entity, Named {
  email: String [required, unique, format: email],
  age: Int? [>= 0, <= 150],
  role: String [in: ["admin", "member", "guest"]] = "member",
  bio: String? [length: 0..2000]
}

node Organization : Entity, Named {
  --- Tax identification number
  tax_id: String? [match: "^[0-9]{2}-[0-9]{7}$"],
  industry: String?,
  employee_count: Int [>= 0] = 0
}

node Project : Entity, Named {
  status: String [in: ["planning", "active", "completed", "archived"]] = "planning",
  deadline: Timestamp?,
  budget: Float? [>= 0]
}

node Task : Entity {
  title: String [required, length: 1..500],
  description: String?,
  status: String [in: ["todo", "in_progress", "done", "blocked"]] = "todo",
  priority: Int [0..10] = 5,
  completed_at: Timestamp?
}
```

### Sealed Configuration Type

```mew
-- System configuration that cannot be extended
[sealed]
node SystemSetting {
  key: String [required, unique, format: slug],
  value: String [required],
  description: String?,
  read_only: Bool = false
}
```

### Complete Person Node with All Modifier Types

```mew
node Person {
  --- Unique identifier for external systems
  external_id: String [unique, match: "^[A-Z]{2}[0-9]{6}$"],

  --- Full name of the person
  name: String [required, length: 1..200],

  --- Email address (required, must be unique and valid)
  email: String [required, unique, format: email],

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

  --- When the account was created (indexed for queries)
  created_at: Timestamp [required, indexed: desc] = now()
}
```

## Errors

| Condition | Message |
|-----------|---------|
| Duplicate node type name | Node type `<name>` already defined in this ontology |
| Unknown parent type | Parent type `<name>` not found |
| Circular inheritance | Circular inheritance detected: `<type1>` -> `<type2>` -> ... -> `<type1>` |
| Conflicting inherited attributes | Attribute `<name>` inherited from multiple parents with incompatible types: `<type1>` vs `<type2>` |
| Instantiate abstract type | Cannot instantiate abstract node type `<name>` |
| Inherit from sealed type | Cannot inherit from sealed node type `<name>` |
| Abstract and sealed combined | Node type `<name>` cannot be both abstract and sealed |
| Nullable with required | Attribute `<name>` cannot be both nullable (?) and [required] |
| Non-nullable without default or required | Attribute `<name>` on `<type>` is non-nullable but has no default and is not [required] |
| Duplicate attribute name | Attribute `<name>` already defined on node type `<type>` |
| Invalid format name | Unknown format `<name>`, expected one of: email, url, uuid, slug, phone, iso_date, iso_datetime, ipv4, ipv6 |
| Invalid range | Range minimum `<min>` is greater than maximum `<max>` |
| Invalid default value type | Default value type `<actual>` does not match attribute type `<expected>` |
| Modify readonly attribute | Cannot modify readonly attribute: `<attr>` on type `<type>` |
