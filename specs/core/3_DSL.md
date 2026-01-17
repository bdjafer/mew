```markdown
# MEW Language Specification

## Part III: Ontology DSL

**Version:** 2.1
**Status:** Stable Core
**Scope:** Schema definition syntax

---

# 1. Overview

## 1.1 Purpose

The Ontology DSL defines graph schemas:

- **Node types** — What kinds of entities can exist
- **Edge types** — What kinds of relations can exist
- **Constraints** — What must always be true

An ontology is a schema. It describes structure, not instance data.

## 1.2 Compilation Model

Ontology DSL source files compile to Layer 0 structures:

```
┌─────────────┐     ┌──────────┐     ┌─────────────────────┐
│  .mew file  │────▶│ Compiler │────▶│ Layer 0 Structures  │
└─────────────┘     └──────────┘     └─────────────────────┘
```

The compiler:
1. Parses source into AST
2. Validates structure and references
3. Expands syntactic sugar into constraints
4. Generates Layer 0 nodes and edges

## 1.3 File Structure

```ebnf
OntologyFile = OntologyDecl? Declaration*

Declaration =
    TypeAliasDecl
  | NodeTypeDecl
  | EdgeTypeDecl
  | ConstraintDecl
```

---

# 2. Ontology Declaration

## 2.1 Syntax

```ebnf
OntologyDecl = "ontology" Identifier "{" Declaration* "}"
```

## 2.2 Examples

```
ontology TaskManagement {
  -- declarations
}
```

## 2.3 Implicit Ontology

If no `ontology` declaration is present, the file defines an anonymous ontology.

## 2.4 Compilation

```
_Ontology node:
  name: <ontology name>
```

---

# 3. Type Aliases

## 3.1 Syntax

```ebnf
TypeAliasDecl = 
    "type" Identifier "=" TypeExpr AttributeModifiers?
  | "type" Identifier "=" UnionTypeExpr

UnionTypeExpr = TypeExpr ("|" TypeExpr)+
```

## 3.2 Examples

### 3.2.1 Scalar Aliases

```
-- With value constraint
type Priority = Int [>= 0, <= 10]

-- Enum alias
type TaskStatus = String [in: ["todo", "in_progress", "done"]]

-- Length constraint
type ShortString = String [length: 1..100]
```

### 3.2.2 Union Aliases

```
type Entity = Person | Organization
type Assignable = Task | Issue | Story
```

### 3.2.3 Alias Chaining

```
type NonNegativeInt = Int [>= 0]
type Priority = NonNegativeInt [<= 10]
```

## 3.3 Usage

```
node Task {
  status: TaskStatus = "todo",
  priority: Priority = 5
}

edge assigned_to(item: Assignable, person: Person)
```

## 3.4 Modifier Composition

When alias is used with additional modifiers, they combine:

```
type Priority = Int [>= 0, <= 10]

node Task {
  priority: Priority [required]  -- has: >= 0, <= 10, required
}
```

**Conflict resolution:** Usage wins over alias definition.

## 3.5 Restrictions

- Aliases cannot be recursive
- Alias names cannot shadow built-in types or declared types
- Union aliases cannot have modifiers

## 3.6 Compilation

Type aliases expand at compile time. They do not generate Layer 0 nodes.

---

# 4. Node Type Declarations

## 4.1 Syntax

```ebnf
NodeTypeDecl = 
  DocComment?
  NodeModifier* "node" Identifier InheritanceClause? "{" AttributeDecl* "}"

NodeModifier = "abstract" | "sealed"

InheritanceClause = ":" QualifiedIdentifier ("," QualifiedIdentifier)*
```

## 4.2 Examples

```
--- Represents an event in time
node Event {
  timestamp: Timestamp [required]
}

-- Inheritance
node Meeting : Event {
  location: String,
  attendee_count: Int [>= 0]
}

-- Abstract type
abstract node Entity {
  name: String [required]
}

-- Sealed type
sealed node SystemConfig {
  key: String [required, unique],
  value: String
}

-- Multiple inheritance
node Document : Named, Timestamped {
  content: String
}
```

## 4.3 Node Modifiers

| Modifier | Meaning |
|----------|---------|
| `abstract` | Cannot be instantiated directly |
| `sealed` | Cannot be inherited |

## 4.4 Inheritance

### 4.4.1 Semantics

When type B inherits from type A:

| Inherited | Behavior |
|-----------|----------|
| Attributes | B has all attributes of A |
| Constraints | Constraints on A apply to B |
| Pattern matching | Pattern `a: A` matches instances of B |

### 4.4.2 Diamond Resolution

If the same attribute is inherited through multiple paths from the same origin, it appears once. Conflicting attribute definitions (same name, incompatible types) make the ontology invalid.

## 4.5 Compilation

```
_NodeType node:
  name: <type name>
  abstract: <true/false>
  sealed: <true/false>
  doc: <doc comment or null>

For each parent P:
  _type_inherits edge: (this_type, P)

For each attribute A:
  _AttributeDef node
  _type_has_attribute edge: (this_type, A)
  
For each value constraint:
  _ConstraintDef node (see §5.5)
```

---

# 5. Attribute Declarations

## 5.1 Syntax

```ebnf
AttributeDecl = 
  DocComment?
  Identifier ":" TypeExpr AttributeModifiers? DefaultValue? ","?

AttributeModifiers = "[" AttributeModifier ("," AttributeModifier)* "]"

AttributeModifier =
    "required"
  | "unique"
  | "readonly"
  | "indexed" (":" ("asc" | "desc"))?
  | ValueConstraint

ValueConstraint =
    ComparisonConstraint
  | RangeConstraint
  | EnumConstraint
  | LengthConstraint

ComparisonConstraint = (">=" | "<=" | ">" | "<") Literal

RangeConstraint = IntLiteral ".." IntLiteral

EnumConstraint = "in:" "[" Literal ("," Literal)* "]"

LengthConstraint = "length:" IntLiteral ".." IntLiteral

DefaultValue = "=" Literal
```

## 5.2 Examples

```
node Person {
  -- Required attribute
  name: String [required],
  
  -- Optional (nullable)
  nickname: String?,
  
  -- With default
  active: Bool = true,
  
  -- Required with default
  role: String [required] = "member",
  
  -- Unique and indexed
  email: String [required, unique],
  
  -- Numeric constraints
  age: Int? [>= 0, <= 150],
  
  -- Range shorthand
  priority: Int [0..10] = 5,
  
  -- Enum constraint
  status: String [in: ["active", "inactive", "pending"]] = "pending",
  
  -- String length
  bio: String? [length: 0..2000],
  
  --- Documentation comment
  created_at: Timestamp [required, indexed: desc]
}
```

## 5.3 Core Modifiers

### 5.3.1 Required

Attribute must have a non-null value.

```
name: String [required]
```

**Compiles to constraint:**
```
constraint <type>_<attr>_required:
  x: <Type> WHERE x.<attr> = null
  => false
```

### 5.3.2 Unique

Attribute value must be unique across all instances.

```
email: String [unique]
```

**Compiles to constraint:**
```
constraint <type>_<attr>_unique:
  x1: <Type>, x2: <Type>
  WHERE x1.id != x2.id AND x1.<attr> = x2.<attr> AND x1.<attr> != null
  => false
```

**Notes:**
- Applies within type and subtypes
- Null values don't violate uniqueness
- Implies `indexed`

### 5.3.3 Readonly

Attribute cannot be modified after entity creation. Value can be set during SPAWN but subsequent SET operations on the attribute are rejected.

```
created_at: Timestamp [readonly] = now()
```

**Enforcement:** Runtime check at SET time. Not a constraint (not checked at commit).

**Use cases:**
- Creation timestamps
- Audit fields
- Immutable identifiers

### 5.3.4 Indexed

Create index for faster queries.

```
timestamp: Timestamp [indexed]
timestamp: Timestamp [indexed: desc]
```

**Effect:** Engine index hint (not a constraint).

## 5.4 Value Constraints

### 5.4.1 Comparison Constraints

```
age: Int [>= 0]
age: Int [<= 150]
score: Float [> 0.0]
score: Float [< 1.0]
```

**Compiles to constraint:**
```
constraint <type>_<attr>_min:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> >= N
```

### 5.4.2 Range Constraint

Shorthand for combined min/max:

```
priority: Int [0..10]
-- Equivalent to: [>= 0, <= 10]
```

### 5.4.3 Enum Constraint

Restrict to allowed values:

```
status: String [in: ["draft", "active", "archived"]]
priority: Int [in: [1, 2, 3, 4, 5]]
```

**Compiles to constraint:**
```
constraint <type>_<attr>_enum:
  x: <Type> WHERE x.<attr> != null
  => x.<attr> = "draft" OR x.<attr> = "active" OR x.<attr> = "archived"
```

### 5.4.4 Length Constraint

For strings:

```
name: String [length: 1..100]
code: String [length: 6..6]      -- exactly 6
```

**Compiles to constraint:**
```
constraint <type>_<attr>_length:
  x: <Type> WHERE x.<attr> != null
  => length(x.<attr>) >= 1 AND length(x.<attr>) <= 100
```

## 5.5 Null Handling in Value Constraints

Value constraints only apply when attribute is **non-null**:

```
node Person {
  age: Int? [>= 0]
}
```

| Value | Result |
|-------|--------|
| `age = 25` | ✓ Valid |
| `age = -5` | ✗ Violates |
| `age = null` | ✓ Valid (constraint skipped) |

To require non-null AND validate:
```
age: Int [required, >= 0]
```

## 5.6 Nullability and Requirements

| Declaration | Nullable? | Must Provide at SPAWN? |
|-------------|-----------|------------------------|
| `x: T` | No | Warning if omitted |
| `x: T?` | Yes | No (defaults to null) |
| `x: T = default` | No | No (uses default) |
| `x: T [required]` | No | Yes |
| `x: T? [required]` | — | **Compile error** |

## 5.7 Default Values

### 5.7.1 Static Defaults

```
status: String = "pending"
priority: Int = 0
active: Bool = true
```

### 5.7.2 Dynamic Defaults

```
created_at: Timestamp = now()
```

Evaluated at entity creation time.

**Allowed:**
- Literals
- `now()`
- Arithmetic on constants: `now() + 86400000`

**Not allowed:**
- Attribute references
- Non-deterministic functions

## 5.8 Compilation

```
_AttributeDef node:
  name: <attribute name>
  scalar_type: <type if scalar, else null>
  required: <true/false>
  unique: <true/false>
  indexed: <"none"/"asc"/"desc">
  default_value: <serialized default or null>
  doc: <doc comment or null>

_type_has_attribute edge:
  (owner_type, attr_def)

If complex type:
  _attr_has_type edge:
    (attr_def, type_expr_node)

For each value constraint:
  _ConstraintDef node + pattern + condition
```

---

# 6. Edge Type Declarations

## 6.1 Syntax

```ebnf
EdgeTypeDecl =
  DocComment?
  "edge" Identifier "(" SignatureParams ")" ("{" AttributeDecl* "}")?

SignatureParams = SignatureParam ("," SignatureParam)*

SignatureParam = Identifier ":" TypeExpr
```

## 6.2 Examples

```
-- Simple binary edge (no attributes)
edge causes(from: Event, to: Event)

-- With attributes
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required],
  role: String = "owner"
}

-- N-ary edge (hyperedge)
edge meeting(organizer: Person, attendee: Person, room: Room) {
  scheduled_at: Timestamp [required]
}
```

## 6.3 Signature

The signature defines the edge's **arity** and type constraints:

```
edge causes(from: Event, to: Event)
--          └─ position 0   └─ position 1
--          arity = 2
```

## 6.4 Edge Attributes

Edges can have attributes, following the same rules as node attributes:

```
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required] = now(),
  role: String [in: ["owner", "reviewer"]] = "owner"
}
```

Braces are optional if no attributes:
```
edge causes(from: Event, to: Event)
edge causes(from: Event, to: Event) {}  -- equivalent
```

## 6.5 Compilation

```
_EdgeType node:
  name: <edge name>
  arity: <number of parameters>
  doc: <doc comment or null>

For each parameter at position i:
  _VarDef node:
    name: <param name>
    is_edge_var: false
  
  _edge_has_position edge:
    (edge_type, var_def) { position: i }
  
  _var_has_type edge:
    (var_def, type_expr_node)

For each attribute:
  (same as node attributes)
```

---

# 7. Patterns

Patterns match graph structure. They are used in constraints and queries.

## 7.1 Syntax

```ebnf
Pattern = PatternElement ("," PatternElement)* WhereClause?

PatternElement = NodePattern | EdgePattern

NodePattern = Identifier ":" TypeExpr

EdgePattern = Identifier "(" TargetList ")" EdgeAlias?

TargetList = Target ("," Target)*

Target = Identifier | "_"

EdgeAlias = "AS" Identifier

WhereClause = "WHERE" Expr
```

## 7.2 Node Patterns

Bind a variable to nodes of a type:

```
e: Event                    -- e is an Event
p: Person                   -- p is a Person
x: Task | Project           -- x is Task or Project
```

## 7.3 Edge Patterns

Match edges between bound variables:

```
causes(e1, e2)              -- e1 and e2 connected by causes
assigned_to(t, p)           -- t assigned to p
```

### 7.3.1 Anonymous Targets

Use `_` to match without binding:

```
assigned_to(task, _)        -- task assigned to someone
```

Multiple `_` are independent matches.

### 7.3.2 Edge Binding

Use `AS` to bind the edge itself to a variable:

```
assigned_to(t, p) AS a
```

This allows:
- Accessing edge attributes: `a.role`, `a.assigned_at`
- Referencing the edge in conditions: `WHERE a.role = "owner"`
- Using edge ID: `a.id`

## 7.4 Where Clause

Filter with boolean expression:

```
e: Event WHERE e.timestamp > 1000
t: Task WHERE t.status = "done" AND t.priority > 5
```

## 7.5 Combined Patterns

```
-- Task assigned to a person, check assignment role
t: Task, p: Person,
assigned_to(t, p) AS a
WHERE a.role = "owner"

-- Task assigned to person in specific team
t: Task, p: Person, team: Team,
assigned_to(t, p),
member_of(p, team)
WHERE team.name = "Engineering"
```

## 7.6 Variable Scoping

Variables declared in a pattern are available in:
- The WHERE clause
- Subsequent pattern elements
- Constraint conditions

## 7.7 Compilation

```
_PatternDef node

For each node pattern:
  _VarDef node:
    name: <variable name>
    is_edge_var: false
  
  _pattern_has_node_var edge:
    (pattern, var_def)
  
  _var_has_type edge:
    (var_def, type_expr_node)

For each edge pattern:
  _EdgePattern node:
    negated: false
  
  _pattern_has_edge_pattern edge:
    (pattern, edge_pattern)
  
  _edge_pattern_type edge:
    (edge_pattern, edge_type)
  
  For each target at position i:
    _edge_pattern_target edge:
      (edge_pattern, var_def) { position: i }

  If AS alias:
    _VarDef node:
      name: <alias name>
      is_edge_var: true
    
    _pattern_has_edge_var edge:
      (pattern, var_def)
    
    _edge_pattern_alias edge:
      (edge_pattern, var_def)

If WHERE clause:
  _pattern_has_condition edge:
    (pattern, condition_expr)
```

---

# 8. Constraint Declarations

## 8.1 Syntax

```ebnf
ConstraintDecl =
  DocComment?
  "constraint" Identifier ":" Pattern "=>" Expr
```

## 8.2 Examples

```
-- Prohibition: no self-loops
constraint no_self_cause:
  e: Event, causes(e, e)
  => false

-- Requirement: completed tasks have timestamp
constraint completed_has_timestamp:
  t: Task WHERE t.status = "done"
  => t.completed_at != null

-- Implication: temporal ordering
constraint temporal_order:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- Using edge attributes
constraint owner_must_be_team_member:
  t: Task, p: Person, team: Team,
  assigned_to(t, p) AS a,
  belongs_to(t, team)
  WHERE a.role = "owner"
  => EXISTS(member_of(p, team))

-- Uniqueness (manual, for complex cases)
constraint unique_assignment:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1), assigned_to(t, p2)
  WHERE p1.id != p2.id
  => false
```

## 8.3 Semantics

A constraint reads: **"For all matches of pattern, condition must hold."**

```
constraint temporal_order:
  e1: Event, e2: Event, causes(e1, e2)
  => e1.timestamp < e2.timestamp
```

Meaning: "For all (e1, e2) where e1 causes e2, e1.timestamp < e2.timestamp must be true."

## 8.4 Constraint Names

Constraint names are **required**. They appear in:
- Error messages
- Debugging output
- Introspection queries

## 8.5 Enforcement

Constraints are checked at transaction commit. Violations reject the transaction.

```
BEGIN
  SPAWN e1: Event { timestamp = 100 }
  SPAWN e2: Event { timestamp = 50 }
  LINK causes(e1, e2)
COMMIT  -- FAILS: e1.timestamp > e2.timestamp violates temporal_order
```

## 8.6 Compilation

```
_ConstraintDef node:
  name: <constraint name>
  hard: true
  doc: <doc comment or null>

_constraint_has_pattern edge:
  (constraint, pattern_def)

_constraint_has_condition edge:
  (constraint, condition_expr)
```

---

# 9. Complete Grammar

```ebnf
(* Top Level *)
OntologyFile     = OntologyDecl? Declaration*
OntologyDecl     = "ontology" Identifier "{" Declaration* "}"
Declaration      = TypeAliasDecl | NodeTypeDecl | EdgeTypeDecl | ConstraintDecl

(* Type Aliases *)
TypeAliasDecl    = "type" Identifier "=" TypeExpr AttrModifiers?
                 | "type" Identifier "=" TypeExpr ("|" TypeExpr)+

(* Node Types *)
NodeTypeDecl     = DocComment? NodeModifier* "node" Identifier 
                   (":" QualIdent ("," QualIdent)*)? 
                   "{" AttributeDecl* "}"
NodeModifier     = "abstract" | "sealed"

(* Attributes *)
AttributeDecl    = DocComment? Identifier ":" TypeExpr AttrModifiers? DefaultValue? ","?
AttrModifiers    = "[" AttrModifier ("," AttrModifier)* "]"
AttrModifier     = "required" | "unique" | "readonly"
                 | "indexed" (":" ("asc" | "desc"))?
                 | ValueConstraint
ValueConstraint  = CompareConstraint | RangeConstraint | EnumConstraint | LengthConstraint
CompareConstraint = (">=" | "<=" | ">" | "<") Literal
RangeConstraint  = IntLiteral ".." IntLiteral
EnumConstraint   = "in:" "[" Literal ("," Literal)* "]"
LengthConstraint = "length:" IntLiteral ".." IntLiteral
DefaultValue     = "=" (Literal | "now()" | ConstantExpr)
ConstantExpr     = "now()" (("+" | "-") IntLiteral)?

(* Edge Types *)
EdgeTypeDecl     = DocComment? "edge" Identifier "(" SigParams ")" 
                   ("{" AttributeDecl* "}")?
SigParams        = SigParam ("," SigParam)*
SigParam         = Identifier ":" TypeExpr

(* Constraints *)
ConstraintDecl   = DocComment? "constraint" Identifier ":" Pattern "=>" Expr

(* Patterns *)
Pattern          = PatternElem ("," PatternElem)* WhereClause?
PatternElem      = NodePattern | EdgePattern
NodePattern      = Identifier ":" TypeExpr
EdgePattern      = Identifier "(" Targets ")" ("AS" Identifier)?
Targets          = Target ("," Target)*
Target           = Identifier | "_"
WhereClause      = "WHERE" Expr
```

---

# 10. Example Ontology

```
ontology TaskManagement {

  -- Type aliases
  type Priority = Int [0..10]
  type TaskStatus = String [in: ["todo", "in_progress", "done"]]

  -- Node types
  node Person {
    name: String [required, length: 1..100],
    email: String [required, unique]
  }

  node Team {
    name: String [required, unique]
  }

  node Project {
    name: String [required],
    deadline: Timestamp?
  }

  node Task {
    title: String [required],
    status: TaskStatus = "todo",
    priority: Priority = 5,
    created_at: Timestamp [required, indexed: desc],
    completed_at: Timestamp?
  }

  -- Edge types
  edge member_of(person: Person, team: Team) {
    role: String = "member"
  }
  
  edge owns(team: Team, project: Project)
  
  edge belongs_to(task: Task, project: Project)
  
  edge assigned_to(task: Task, person: Person) {
    assigned_at: Timestamp = now(),
    role: String [in: ["owner", "reviewer"]] = "owner"
  }
  
  edge depends_on(downstream: Task, upstream: Task)

  -- Constraints
  constraint no_self_dependency:
    t: Task, depends_on(t, t)
    => false

  constraint completed_has_timestamp:
    t: Task WHERE t.status = "done"
    => t.completed_at != null

  constraint single_owner:
    t: Task, p1: Person, p2: Person,
    assigned_to(t, p1) AS a1,
    assigned_to(t, p2) AS a2
    WHERE a1.role = "owner" AND a2.role = "owner" AND p1.id != p2.id
    => false
}
```

---

*End of Part III: Ontology DSL*
```