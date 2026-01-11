# 

This is the fixed foundation. It is hardcoded into the engine and cannot be modified by any rewrite rule. Everything else builds on this.

---

## 0. Preliminaries

### 0.1 Notation

In this specification:

- `NodeType` declarations define what node kinds exist at Layer 0
- `EdgeType` declarations define what edge kinds exist at Layer 0
- `Constraint` declarations define invariants that always hold
- `[abstract]` means the type cannot be instantiated directly, only inherited
- `[sealed]` means the type cannot be inherited by user ontologies
- Attributes marked `required` must always have a value
- Attributes marked `unique` must be unique across all instances of that type

### 0.2 Identity

Every node and edge has an immutable identity:

- `id: ID` — unique identifier, assigned at creation, never changes
- IDs are opaque (implementation may use UUID, sequential, or content-hash)
- Two nodes/edges with the same ID are the same node/edge
- Two nodes/edges with different IDs are different, even if all attributes match

### 0.3 Reserved namespace

All identifiers starting with `_` are reserved for Layer 0.
User ontologies cannot declare types, attributes, or variables starting with `_`.

---

## 1. Scalar Types (Built-in)

Primitive value types for attributes. For detailed operations and semantics, see **Part I: Foundations §3**.

```
Scalar Types
============
String      -- UTF-8 text, unbounded length
Int         -- 64-bit signed integer
Float       -- 64-bit IEEE 754 floating point
Bool        -- true or false
Timestamp   -- milliseconds since Unix epoch (Int alias with semantic meaning)
Duration    -- time span in milliseconds (Int alias with semantic meaning)
ID          -- opaque identifier (internal use)
```

### 1.1 Default value serialization

The `_AttributeDef.default_value` field stores defaults as JSON-encoded strings:

| Type | Value | Serialized |
| --- | --- | --- |
| Int | 42 | `"42"` |
| Float | 3.14 | `"3.14"` |
| Bool | true | `"true"` |
| String | "hello" | `"\"hello\""` |
| Timestamp | 1704067200000 | `"1704067200000"` |
| Duration | 86400000 | `"86400000"` |
| (no default) | - | `null` |

### 1.2 Dynamic defaults

For dynamic defaults like `now()`, the `default_value` field uses a special prefix:

| Default Expression | Serialized |
| --- | --- |
| `now()` | `"$now()"` |
| `now() + 24.hours` | `"$now() + 86400000"` |

The `$` prefix indicates the value should be evaluated at entity creation time.

---

## 2. Layer 0 Node Types

### 2.1 Meta-types (describe structure)

```
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
  arity: Int [required],          -- number of targets
  symmetric: Bool = false,        -- order doesn't matter
  reflexive_allowed: Bool = true, -- can connect node to itself
  doc: String?
}

node _AttributeDef [sealed] {
  name: String [required],
  required: Bool = false,
  unique: Bool = false,
  indexed: String = "none",  -- "none", "asc", or "desc"
  default_value: String?,  -- serialized default, null if none (see Section 1.3-1.4)
  doc: String?
}

node _ConstraintDef [sealed] {
  name: String [required],  -- constraint name (required for error messages)
  hard: Bool = true,  -- hard constraints reject; soft constraints warn
  message: String?,  -- custom error/warning message
  doc: String?
}

node _RuleDef [sealed] {
  name: String?,
  priority: Int = 0,  -- higher priority rules fire first
  auto: Bool = true,  -- auto-fire when pattern matches, or manual only
  doc: String?
}

```

### 2.2 Type expressions (describe types in signatures)

```
node _TypeExpr [abstract, sealed] {
  -- Base for all type expressions
}

node _NamedTypeExpr : _TypeExpr [sealed] {
  ref_name: String [required]  -- name of the referenced NodeType
}

node _OptionalTypeExpr : _TypeExpr [sealed] {
  -- Wraps another TypeExpr to make it optional
}

node _UnionTypeExpr : _TypeExpr [sealed] {
  -- Combines two TypeExprs with OR semantics
}

node _EdgeRefTypeExpr : _TypeExpr [sealed] {
  -- Reference to an edge (for higher-order)
  -- If ref_name is null, matches any edge type
  ref_name: String?
}

node _AnyTypeExpr : _TypeExpr [sealed] {
  -- Matches any node type
}

node _ScalarTypeExpr : _TypeExpr [sealed] {
  scalar_type: String [required]  -- "String", "Int", "Float", "Bool", "Timestamp", "Duration"
}

```

### 2.3 Pattern representation (for constraints and rules)

```
node _PatternDef [sealed] {
  -- A pattern is a template hypergraph with variables
}

node _VarDef [sealed] {
  name: String [required],
  is_edge_var: Bool = false  -- true if this binds to an edge, false for node
}

node _EdgePattern [sealed] {
  -- Represents an edge constraint within a pattern
  negated: Bool = false,  -- if true, pattern requires edge NOT to exist
  transitive: String = "none"  -- "none", "+", or "*" for transitive closure
}

```

Note: Conditions in patterns are represented as `_Expr` nodes (specifically, expressions that evaluate to Bool). There is no separate `_ConditionExpr` type.

### 2.4 Production representation (for rules)

```
node _ProductionDef [sealed] {
  -- A production specifies what to spawn/kill/link/unlink/set
}

node _Action [abstract, sealed] {
  -- Base for all production actions
  order: Int = 0  -- execution order within production
}

node _SpawnAction : _Action [sealed] {
  var_name: String [required],  -- variable to bind the new node to
  -- type is specified via edge to _TypeExpr
}

node _LinkAction : _Action [sealed] {
  var_name: String?,  -- optional variable to bind the new edge to
  -- edge type and targets specified via edges
}

node _KillAction : _Action [sealed] {
  target_var: String [required]  -- variable naming node to kill
}

node _UnlinkAction : _Action [sealed] {
  target_var: String [required]  -- variable naming edge to unlink
}

node _SetAction : _Action [sealed] {
  target_var: String [required],
  attr_name: String [required]
  -- value specified via edge to _Expr
}

```

### 2.5 Expression representation (for conditions and computations)

```
node _Expr [abstract, sealed] {
  -- Base for all expressions
}

node _LiteralExpr : _Expr [sealed] {
  value_type: String [required],  -- "String", "Int", "Float", "Bool", "Null", "Timestamp", "Duration"
  value_string: String [required]  -- serialized value (for Null: "null")
}

node _VarRefExpr : _Expr [sealed] {
  var_name: String [required]
}

node _AttrAccessExpr : _Expr [sealed] {
  attr_name: String [required]
  -- base expression specified via edge
}

node _BinaryOpExpr : _Expr [sealed] {
  operator: String [required]
  -- operators: "=", "!=", "<", ">", "<=", ">=",
  --            "+", "-", "*", "/", "%",
  --            "and", "or"
  -- left and right operands specified via edges
}

node _UnaryOpExpr : _Expr [sealed] {
  operator: String [required]
  -- operators: "not", "-" (negation)
  -- operand specified via edge
}

node _ExistsExpr : _Expr [sealed] {
  negated: Bool = false  -- if true, this is "not exists"
  -- inner pattern specified via edge
}

node _IfExpr : _Expr [sealed] {
  -- IF condition THEN then_branch ELSE else_branch
  -- condition, then_branch, else_branch specified via edges
}

node _CaseExpr : _Expr [sealed] {
  -- CASE subject? WHEN ... THEN ... ELSE ... END
  has_subject: Bool = false  -- true if simple CASE (value matching), false if searched CASE
  -- subject (optional), when_clauses, else_clause specified via edges
}

node _WhenClause [sealed] {
  -- WHEN condition THEN result
  -- condition and result specified via edges
}

node _CoalesceExpr : _Expr [sealed] {
  -- COALESCE(expr1, expr2, ...) or expr1 ?? expr2
  -- arguments specified via ordered _list_element edges
}

node _ListExpr : _Expr [sealed] {
  -- Represents a list of expressions
  -- Elements specified via ordered _list_element edges
  -- v1 usage: Reserved for future use (multi-value returns, set operations)
}

```

### 2.6 Special nodes

```
node _Ontology [sealed] {
  name: String [required],
  version: String?,
  doc: String?
}

node _Import [sealed] {
  -- Represents an import of another ontology
  ontology_name: String [required],
  alias: String?
}

```

---

## 3. Layer 0 Edge Types

**Notation**: Edge declarations follow this structure:
```
edge name(target1: Type1, target2: Type2, ...) {
  attr1: ScalarType [modifiers],
  attr2: ScalarType,
  -- comment
}
```

- **Targets** (in parentheses): Node or edge references that the hyperedge connects
- **Attributes** (in braces): Scalar values attached to the edge itself
- Hyperedges connect **nodes/edges only**; scalar values are always attributes

### 3.1 Type structure edges

```
edge _type_inherits(
  child: _NodeType,
  parent: _NodeType
) {
  -- child inherits from parent
}

edge _type_has_attribute(
  owner: _NodeType | _EdgeType,
  attr: _AttributeDef
) {
  -- owner has this attribute
}

edge _attr_has_type(
  attr: _AttributeDef,
  type_expr: _TypeExpr
) {
  -- attribute has this type (for complex types like node refs, unions)
}

edge _attr_has_scalar_type(
  attr: _AttributeDef
) {
  scalar_type: String [required]  -- "String", "Int", "Float", "Bool", "Timestamp", "Duration"
  -- attribute has this scalar type (unary edge with scalar attribute)
}

```

### 3.2 Edge signature edges

```
edge _edge_has_position(
  edge_type: _EdgeType,
  var: _VarDef
) {
  position: Int [required]  -- 0-indexed position in signature
  -- edge type has this variable at this position
}

edge _var_has_type(
  var: _VarDef,
  type_expr: _TypeExpr
) {
  -- variable has this type constraint
}

```

### 3.3 Type expression structure edges

```
edge _optional_inner(
  optional: _OptionalTypeExpr,
  inner: _TypeExpr
) {
  -- optional wraps this inner type
}

edge _union_member(
  union: _UnionTypeExpr,
  member: _TypeExpr
) {
  position: Int [required]  -- 0 for left, 1 for right (or more for n-ary)
  -- union includes this member type at this position
}

```

### 3.4 Constraint edges

```
edge _constraint_has_pattern(
  constraint: _ConstraintDef,
  pattern: _PatternDef
) {
  -- constraint matches this pattern
}

edge _constraint_has_condition(
  constraint: _ConstraintDef,
  condition: _Expr
) {
  -- constraint requires this condition (pattern => condition)
}

```

### 3.5 Rule edges

```
edge _rule_has_pattern(
  rule: _RuleDef,
  pattern: _PatternDef
) {
  -- rule matches this pattern
}

edge _rule_has_production(
  rule: _RuleDef,
  production: _ProductionDef
) {
  -- rule produces this
}

```

### 3.6 Pattern structure edges

```
edge _pattern_has_node_var(
  pattern: _PatternDef,
  var: _VarDef
) {
  -- pattern contains this node variable
}

edge _pattern_has_edge_var(
  pattern: _PatternDef,
  var: _VarDef
) {
  -- pattern contains this edge variable
}

edge _pattern_has_edge_pattern(
  pattern: _PatternDef,
  edge_pattern: _EdgePattern
) {
  -- pattern contains this edge constraint
}

edge _edge_pattern_type(
  edge_pattern: _EdgePattern,
  edge_type: _EdgeType
) {
  -- edge pattern constrains to this edge type
}

edge _edge_pattern_target(
  edge_pattern: _EdgePattern,
  var: _VarDef
) {
  position: Int [required]  -- 0-indexed position in edge signature
  -- edge pattern expects this variable at this position
}

edge _edge_pattern_alias(
  edge_pattern: _EdgePattern,
  var: _VarDef
) {
  -- edge pattern binds the matched edge to this variable
}

edge _pattern_has_condition(
  pattern: _PatternDef,
  condition: _Expr
) {
  -- pattern includes this condition in its where clause
}

```

### 3.7 Production structure edges

```
edge _production_has_action(
  production: _ProductionDef,
  action: _Action
) {
  -- production includes this action
}

edge _spawn_node_type(
  action: _SpawnAction,
  type_expr: _TypeExpr
) {
  -- spawn action creates a node of this type
}

edge _link_edge_type(
  action: _LinkAction,
  edge_type: _EdgeType
) {
  -- link action creates an edge of this type
}

edge _link_edge_target(
  action: _LinkAction
) {
  position: Int [required],      -- 0-indexed position in edge signature
  var_name: String [required]    -- name of variable to use as target
  -- link action targets this variable at this position
  -- (unary edge: var_name is a string reference, not a node target)
}

edge _set_attr_value(
  action: _SetAction,
  value: _Expr
) {
  -- set action uses this expression as value
}

edge _spawn_node_attribute(
  action: _SpawnAction,
  value: _Expr
) {
  attr_name: String [required]
  -- inline attribute assignment: SPAWN x: T { attr_name = value }
}

edge _link_edge_attribute(
  action: _LinkAction,
  value: _Expr
) {
  attr_name: String [required]
  -- inline attribute assignment: LINK edge(a, b) { attr_name = value }
}

```

### 3.8 Expression structure edges

```
edge _attr_access_base(
  expr: _AttrAccessExpr,
  base: _Expr
) {
  -- attribute access is on this base expression
}

edge _binary_left(
  expr: _BinaryOpExpr,
  left: _Expr
) {
  -- binary operation left operand
}

edge _binary_right(
  expr: _BinaryOpExpr,
  right: _Expr
) {
  -- binary operation right operand
}

edge _unary_operand(
  expr: _UnaryOpExpr,
  operand: _Expr
) {
  -- unary operation operand
}

edge _exists_pattern(
  expr: _ExistsExpr,
  pattern: _PatternDef
) {
  -- exists expression checks this pattern
}

edge _list_element(
  expr: _ListExpr,
  element: _Expr
) {
  position: Int [required]  -- 0-indexed position in list
  -- list contains this element at this position
}

edge _if_condition(
  expr: _IfExpr,
  condition: _Expr
) {
  -- IF expression condition
}

edge _if_then(
  expr: _IfExpr,
  then_branch: _Expr
) {
  -- IF expression THEN branch
}

edge _if_else(
  expr: _IfExpr,
  else_branch: _Expr
) {
  -- IF expression ELSE branch
}

edge _case_subject(
  expr: _CaseExpr,
  subject: _Expr
) {
  -- CASE expression subject (for simple CASE)
}

edge _case_when(
  expr: _CaseExpr,
  when_clause: _WhenClause
) {
  position: Int [required]  -- 0-indexed position in WHEN list
  -- CASE expression WHEN clause
}

edge _case_else(
  expr: _CaseExpr,
  else_branch: _Expr
) {
  -- CASE expression ELSE branch
}

edge _when_condition(
  clause: _WhenClause,
  condition: _Expr
) {
  -- WHEN clause condition
}

edge _when_result(
  clause: _WhenClause,
  result: _Expr
) {
  -- WHEN clause result
}

edge _coalesce_arg(
  expr: _CoalesceExpr,
  arg: _Expr
) {
  position: Int [required]  -- 0-indexed position in argument list
  -- COALESCE argument at this position
}

```

### 3.9 Ontology structure edges

```
edge _ontology_declares_type(
  ontology: _Ontology,
  type: _NodeType | _EdgeType
) {
  -- ontology declares this type
}

edge _ontology_declares_constraint(
  ontology: _Ontology,
  constraint: _ConstraintDef
) {
  -- ontology declares this constraint
}

edge _ontology_declares_rule(
  ontology: _Ontology,
  rule: _RuleDef
) {
  -- ontology declares this rule
}

edge _ontology_imports(
  ontology: _Ontology,
  import: _Import
) {
  -- ontology has this import
}

edge _ontology_inherits(
  child: _Ontology,
  parent: _Ontology
) {
  -- child ontology inherits all types, constraints, rules from parent
  -- child can add new types but cannot remove/modify parent's types
  -- multiple inheritance allowed (child can have multiple parents)
}

```

### 3.10 Instance-level edges (for user data)

```
edge _instance_of(
  instance: any,
  type: _NodeType
) {
  -- this node is an instance of this type
  -- (implicit for all nodes, made explicit for observing)
}

edge _edge_instance_of(
  instance: edge<any>,
  type: _EdgeType
) {
  -- this edge is an instance of this edge type
  -- (implicit, made explicit for higher-order observing)
}

```

---

## 4. Layer 0 Constraints

These are invariants that the engine enforces. They cannot be violated.

### 4.1 Naming constraints

```
constraint _unique_node_type_names:
  t1: _NodeType, t2: _NodeType
  where t1.id != t2.id
  => t1.name != t2.name

constraint _unique_edge_type_names:
  t1: _EdgeType, t2: _EdgeType
  where t1.id != t2.id
  => t1.name != t2.name

constraint _unique_attribute_per_type:
  owner: _NodeType | _EdgeType,
  a1: _AttributeDef, a2: _AttributeDef,
  _type_has_attribute(owner, a1),
  _type_has_attribute(owner, a2)
  where a1.id != a2.id
  => a1.name != a2.name

```

### 4.2 Inheritance constraints

```
constraint _no_inheritance_cycle:
  t: _NodeType, _type_inherits(t, t)
  => false

constraint _no_inheritance_cycle_2:
  t1: _NodeType, t2: _NodeType,
  _type_inherits(t1, t2), _type_inherits(t2, t1)
  => false

constraint _abstract_not_instantiated:
  -- (enforced procedurally: cannot create instance of abstract type)

constraint _sealed_not_inherited:
  parent: _NodeType, child: _NodeType,
  _type_inherits(child, parent)
  => parent.sealed = false

constraint _no_ontology_inheritance_cycle:
  o: _Ontology, _ontology_inherits(o, o)
  => false

constraint _no_ontology_inheritance_cycle_2:
  o1: _Ontology, o2: _Ontology,
  _ontology_inherits(o1, o2), _ontology_inherits(o2, o1)
  => false

```

### 4.3 Edge type constraints

```
constraint _edge_arity_matches_positions:
  e: _EdgeType, v: _VarDef,
  _edge_has_position(e, v) AS pos_edge
  => pos_edge.position >= 0 and pos_edge.position < e.arity

constraint _edge_positions_complete:
  -- (enforced procedurally: all positions 0..arity-1 must be filled)

constraint _edge_positions_unique:
  e: _EdgeType, v1: _VarDef, v2: _VarDef,
  _edge_has_position(e, v1) AS p1,
  _edge_has_position(e, v2) AS p2
  WHERE p1.position = p2.position
  => v1.id = v2.id

```

### 4.4 Type expression constraints

```
constraint _named_type_exists:
  t: _NamedTypeExpr
  => exists(nt: _NodeType where nt.name = t.ref_name)
     or exists(et: _EdgeType where et.name = t.ref_name)

constraint _optional_has_inner:
  t: _OptionalTypeExpr
  => exists(inner: _TypeExpr, _optional_inner(t, inner))

constraint _union_has_members:
  t: _UnionTypeExpr
  => exists(m1: _TypeExpr, _union_member(t, m1) AS u1 WHERE u1.position = 0)
     and exists(m2: _TypeExpr, _union_member(t, m2) AS u2 WHERE u2.position = 1)

```

### 4.5 Pattern constraints

```
constraint _pattern_edge_var_has_type:
  p: _PatternDef, v: _VarDef,
  _pattern_has_edge_var(p, v)
  => v.is_edge_var = true

constraint _pattern_node_var_has_type:
  p: _PatternDef, v: _VarDef,
  _pattern_has_node_var(p, v)
  => v.is_edge_var = false

constraint _edge_pattern_has_type:
  ep: _EdgePattern
  => exists(et: _EdgeType, _edge_pattern_type(ep, et))

constraint _edge_pattern_targets_complete:
  -- (enforced procedurally: all positions must have targets)

```

### 4.7 Procedural constraints

These invariants are enforced by the engine rather than as pattern-based constraints:

**_abstract_not_instantiated**:
Attempting to create an instance of a NodeType where `abstract=true` results in:
`Error("Cannot instantiate abstract type 'TypeName'")`

**_edge_positions_complete**:
When creating an EdgeType, all positions 0 through arity-1 must have exactly one `_edge_has_position` edge. Incomplete signatures result in:
`Error("EdgeType 'Name' missing signature position N")`

**_edge_pattern_targets_complete**:
When creating an EdgePattern referencing EdgeType E, all positions of E must have a target variable via `_edge_pattern_target`. Incomplete patterns result in:
`Error("EdgePattern missing target for position N")`

**_pattern_var_names_unique**:
When compiling a pattern, all variable names (node vars and edge vars) must be distinct. Duplicate names result in:
`Error("Duplicate variable name 'x' in pattern")`

### 4.6 Expression constraints

```
constraint _binary_has_operands:
  e: _BinaryOpExpr
  => exists(l: _Expr, _binary_left(e, l))
     and exists(r: _Expr, _binary_right(e, r))

constraint _unary_has_operand:
  e: _UnaryOpExpr
  => exists(o: _Expr, _unary_operand(e, o))

constraint _attr_access_has_base:
  e: _AttrAccessExpr
  => exists(b: _Expr, _attr_access_base(e, b))

```

---

## 5. Built-in Operations

These are primitive operations implemented by the engine. The HOHG language uses specific keywords for these operations:

- **SPAWN**: Create a node
- **KILL**: Remove a node
- **LINK**: Create an edge
- **UNLINK**: Remove an edge
- **SET**: Modify an attribute

### 5.1 Node operations

```
SPAWN(type: _NodeType, attributes: Map<String, Any>) -> ID
  -- Creates a node of the given type with given attributes
  -- Validates: type is not abstract, required attributes present, types match
  -- Returns: the new node's ID

KILL(id: ID) -> Bool
  -- Removes the node with given ID
  -- Validates: no edges reference this node (or cascade delete)
  -- Returns: true if removed, false if not found

getNode(id: ID) -> Node?
  -- Returns the node or null if not found

SET(id: ID, attr: String, value: Any) -> Bool
  -- Sets an attribute on a node
  -- Validates: attribute exists on type, value type matches
  -- Returns: true if set, false if invalid

```

### 5.2 Edge operations

```
LINK(type: _EdgeType, targets: List<ID>, attributes: Map<String, Any>) -> ID
  -- Creates an edge of the given type connecting the targets
  -- Validates: targets.length = type.arity, target types match signature
  -- Returns: the new edge's ID

UNLINK(id: ID) -> Bool
  -- Removes the edge with given ID
  -- Validates: no higher-order edges reference this edge (or cascade)
  -- Returns: true if removed, false if not found

getEdge(id: ID) -> Edge?
  -- Returns the edge or null if not found

SET(id: ID, attr: String, value: Any) -> Bool
  -- Sets an attribute on an edge
  -- Validates: attribute exists on type, value type matches
  -- Returns: true if set, false if invalid

```

### 5.3 Observation operations

```
findNodes(type: _NodeType?, filter: Expr?) -> Iterator<Node>
  -- Finds nodes matching criteria
  -- type: filter by type (and subtypes), null for any
  -- filter: boolean expression over attributes

findEdges(type: _EdgeType?, source: ID?, target: ID?, filter: Expr?) -> Iterator<Edge>
  -- Finds edges matching criteria
  -- type: filter by type, null for any
  -- source: filter by target at position 0
  -- target: filter by target at position 1 (for binary edges)
  -- filter: boolean expression over attributes

matchPattern(pattern: _PatternDef) -> Iterator<Match>
  -- Finds all matches of the pattern in the graph
  -- Returns bindings for each match

```

### 5.4 Transaction operations

```
beginTransaction() -> TransactionID
commit(txn: TransactionID) -> Bool
rollback(txn: TransactionID) -> Bool

```

### 5.5 Version operations

```
snapshot() -> VersionID
  -- Creates an immutable snapshot of current state

checkout(version: VersionID) -> Bool
  -- Loads a historical version (read-only)

diff(v1: VersionID, v2: VersionID) -> Diff
  -- Returns the differences between two versions

branch(name: String) -> BranchID
  -- Creates a new branch from current state

merge(branch: BranchID) -> MergeResult
  -- Merges branch into current state

```

---

## 6. Execution Semantics

### 6.1 Constraint checking

When any mutation occurs (SPAWN, KILL, LINK, UNLINK, SET):

1. Identify all constraints whose pattern could be affected
2. For each affected constraint:
    - Find all matches of the pattern
    - For each match, evaluate the condition
    - If condition is false and constraint is hard: reject mutation
    - If condition is false and constraint is soft: log warning
3. If all hard constraints pass: commit mutation
4. If any hard constraint fails: rollback mutation, return error

### 6.2 Rule execution

When a mutation commits and the graph changes:

1. Identify all rules whose pattern could now match
2. For each rule with `auto = true`, in priority order:
    - Find all new matches of the pattern (not matched before)
    - For each new match:
        - Execute the production actions in order
        - This may trigger further rule execution (recursive)
3. Continue until no new matches (quiescence) or cycle detected

### 6.3 Cycle detection

To prevent infinite loops:

- Track (rule, match) pairs executed in current chain
- If same pair appears twice: cycle detected
- On cycle: stop execution, log warning, optionally rollback

### 6.4 Transaction semantics

- All mutations within a transaction are atomic
- Constraint checking happens at commit time
- Rule execution happens after commit, before returning to caller
- If rule execution fails, entire transaction is rolled back

---

## 7. Reserved Identifiers

### 7.1 Reserved prefixes

| Prefix | Usage |
| --- | --- |
| `_` | Layer 0 internal names |
| `__` | Engine internal (not exposed) |

### 7.2 Reserved type names

All Layer 0 type names (prefixed with `_`) are reserved.

### 7.3 Reserved attribute names

| Name | Type | Meaning |
| --- | --- | --- |
| `id` | ID | Node/edge identity (implicit, always present) |
| `_type` | String | Type name (implicit, always present) |
| `_created` | Timestamp | Creation time (optional, if tracking enabled) |
| `_modified` | Timestamp | Last modification time (optional) |
| `_version` | Int | Version counter (optional) |

---

## 8. Bootstrapping

### 8.1 Initial state

When a new HOHG database is created:

1. Layer 0 types are registered (but not stored as nodes—they're built-in)
2. Layer 0 edges are registered
3. Layer 0 constraints are registered
4. The graph is empty (no user nodes or edges)

### 8.2 Ontology loading

When an ontology is loaded:

1. Parse the ontology source
2. Create `_Ontology` node
3. For each type declaration: create `_NodeType` or `_EdgeType` nodes with structure
4. For each constraint: create `_ConstraintDef` node with pattern and condition
5. For each rule: create `_RuleDef` node with pattern and production
6. Validate all Layer 0 constraints on the ontology structure
7. If valid: ontology is active
8. If invalid: reject with error

### 8.3 Instance creation

When user creates a node of a declared type:

1. Look up `_NodeType` by name
2. Validate type is not abstract
3. Validate required attributes are provided
4. Validate attribute types match
5. Create node with type tag (stored in `_type` field)
6. Check constraints
7. If valid: commit
8. Execute triggered rules

### 8.4 `_instance_of` edge policy

The `_instance_of` and `_edge_instance_of` edges are **not created by default**.

**Rationale**: Every node already has its type in the `_type` field. Creating explicit edges would double storage.

**When created**:
- On explicit API request: `SPAWN(..., { trackInstance: true })`
- Lazily materialized when a meta-observation requires them
- Can be bulk-created via utility function for existing data

**Usage**: These edges enable meta-observations that treat type relationships as graph structure:
```
-- Find all instances of a type (using _instance_of)
MATCH n: any, t: _NodeType, _instance_of(n, t)
WHERE t.name = "Event"
RETURN n

-- Equivalent without _instance_of (preferred)
MATCH n: Event
RETURN n
```

### 8.5 Type aliases

Type aliases defined in the Ontology DSL (e.g., `type Email = String [match: "..."]`) are **expanded at compile time** and do not generate Layer 0 nodes.

**Example:**
```
-- Ontology DSL:
type Email = String [match: "^.+@.+\\..+$"]
node Person { email: Email [required] }

-- Compiles to (no _TypeAlias node):
node Person { email: String [required, match: "^.+@.+\\..+$"] }
```

**Rationale**: Type aliases are syntactic sugar for developer convenience. Expanding them at compile time:
- Keeps Layer 0 simpler (no additional node type needed)
- Ensures all type information is directly on `_AttributeDef` nodes
- Avoids indirection when observing ontology structure

---

## 9. Type Checking Rules

For general subtyping rules, see **Part I: Foundations §4.8**.

### 9.1 Edge type checking

When creating edge of type E with targets [t1, t2, ...]:

For each position i:

- Let V = signature variable at position i
- Let T = type of V
- Let actual = type of ti
- Require: actual <: T

### 9.2 Attribute type checking

When setting attribute A on node/edge of type T:

- Look up AttributeDef for A on T (or inherited types)
- If not found: reject
- Let expected = type of A
- Let actual = type of value
- For scalar types: exact match required
- For node types: subtyping allowed

### 9.3 Edge reference type checking

When a signature position has type `edge<E>`:

- Target must be an **edge ID** (not node ID)
- If E is specified (`ref_name != null`): edge must have type E exactly
- If E is "any" (`ref_name = null`): any edge type accepted

**Note**: Edge types do not support inheritance. Exact type match is required when E is specified.

### 9.4 Edge immutability

Edge targets are immutable after creation:

- Once created, an edge's targets cannot be changed
- To "move" an edge, delete and recreate it
- This prevents self-referential edges (an edge cannot target itself)

---

## 10. Higher-Order Observation Syntax

### 10.1 Edge binding with AS

To observe higher-order edges (edges about edges), bind an edge to a variable using `AS`:

```
MATCH 
  e1: Event, e2: Event, 
  causes(e1, e2) AS c,           -- bind edge to variable 'c'
  confidence(c) AS conf          -- higher-order: edge targeting edge 'c'
WHERE conf.level > 0.5
RETURN e1.name, e2.name, conf.level
```

**Edge attribute access**: When an edge is bound to a variable, its attributes are accessible:
- `c.mechanism` — accesses the 'mechanism' attribute on the causes edge
- `conf.level` — accesses the 'level' attribute on the confidence edge
- `c.id` — accesses the edge's ID (implicit on all edges)

### 10.2 Higher-order pattern syntax

In patterns (constraints and rules), edge binding uses the same syntax:

```
constraint high_confidence_requires_evidence:
  e1: Event, e2: Event,
  causes(e1, e2) AS c,
  confidence(c) AS conf
  WHERE conf.level > 0.8
  => EXISTS(ev: Evidence, supports(ev, c))
```

### 10.3 Observing edge<any>

The `edge<any>` type matches any edge. Use it for meta-level edges:

```
MATCH
  e: edge<any>,                  -- any edge in the graph
  confidence(e) AS conf          -- confidence about that edge
WHERE conf.level < 0.3
RETURN e, conf.level
```

---

## 11. Negative Patterns

### 11.1 NOT EXISTS syntax

Use `NOT EXISTS(pattern)` in WHERE clauses to require absence:

```
constraint no_orphan_task:
  t: Task 
  WHERE NOT EXISTS(assigned_to(t, _))
  => false

rule create_missing_inverse:
  c1: Concept, c2: Concept,
  related_to(c1, c2) AS r
  WHERE r.relation_type = "similar_to"
    AND NOT EXISTS(related_to(c2, c1))
  =>
  LINK related_to(c2, c1) { relation_type = "similar_to" }
```

### 11.2 Anonymous variable

The underscore `_` matches any node/edge of compatible type without binding:

```
-- Matches if ANY assignment exists, doesn't care which person
assigned_to(task, _)

-- Multiple underscores are independent
depends_on(_, _)  -- any dependency edge exists
```

### 11.3 Negated edge patterns

In _EdgePattern, `negated: Bool = true` compiles to NOT EXISTS:

```
-- These are equivalent:
WHERE NOT EXISTS(assigned_to(t, _))
-- compiles to EdgePattern with negated=true
```

---

## 12. Deletion Semantics

### 12.1 Default behavior: reject if referenced

```
KILL(id) 
  → Error("Cannot kill: node referenced by edges") 
    if any edge targets this node

UNLINK(id)
  → Error("Cannot unlink: edge referenced by higher-order edges")
    if any edge targets this edge
```

### 12.2 Cascade deletion

Optional cascade flag removes referencing edges recursively:

```
KILL(id, { cascade: true })
  1. Find all edges E where id ∈ E.targets
  2. For each E: UNLINK(E.id, { cascade: true })
  3. Kill the node

UNLINK(id, { cascade: true })
  1. Find all edges E where id ∈ E.targets (higher-order)
  2. For each E: UNLINK(E.id, { cascade: true })
  3. Unlink the edge
```

Cascade is recursive and respects constraints—if any deletion would violate a constraint, the entire cascade fails.

---

## 13. Attribute Limitations

### 13.1 Scalar-only attributes (v1)

Attributes hold scalar values only: String, Int, Float, Bool, Timestamp, Duration.

**Not supported in v1**:
- List attributes: `tags: List<String>`
- Set attributes: `categories: Set<String>`
- Node references as attributes: `parent: Node`

### 13.2 Collections via edges

Model collections as edges instead:

```
-- Instead of: tags: List<String>
node Tag { value: String }
edge has_tag(Task, Tag)

-- Observe all tags for a task:
MATCH t: Task, tag: Tag, has_tag(t, tag)
WHERE t.id = $taskId
RETURN tag.value
```

This keeps the model uniform and observable.

---

## 14. Variable Scoping

### 14.1 Pattern variables

Variables declared in a pattern are available in:
- The WHERE clause of that pattern
- The condition of a constraint using that pattern
- The production of a rule using that pattern

### 14.2 Production variables

In rule productions, variables are bound sequentially:

```
rule example:
  a: TypeA
  =>
  SPAWN b: TypeB { name = a.name },  -- 'b' bound here
  LINK connects(a, b),               -- 'b' available
  SET b.processed = true             -- 'b' available
```

**Scope rules**:
- Pattern variables: available throughout production
- Spawned variables: available after their SPAWN action
- Cannot reference a variable before it's created

### 14.3 Condition scope

In constraint conditions, only pattern variables are in scope:

```
constraint example:
  x: TypeX, y: TypeY, connects(x, y)
  => x.value < y.value  -- x and y in scope
```

### 14.4 EXISTS scope

Variables in EXISTS patterns:

- **CAN** reference variables from enclosing pattern
- **CANNOT** be referenced outside the EXISTS
- **CANNOT** shadow (reuse names of) outer variables

Variable names must be unique across the entire pattern including nested EXISTS:

```
-- Valid: inner references outer
constraint example:
  x: TypeX
  WHERE EXISTS(y: TypeY, connects(x, y))  -- x visible inside EXISTS
  => ...

-- Invalid: shadowing
constraint bad:
  x: TypeX
  WHERE EXISTS(x: TypeY, ...)  -- ERROR: x already declared
  => ...
```

### 14.5 Inline attribute evaluation

Attributes in a SPAWN statement are evaluated independently (not sequentially):

- Cannot reference other attributes in the same SPAWN
- Cannot reference the variable being bound

```
-- Valid: reference pattern variable
SPAWN x: TypeX { a = y.value }

-- Invalid: cross-reference within SPAWN
SPAWN x: TypeX { a = 1, b = a + 1 }   -- ERROR: a not in scope

-- Invalid: self-reference
SPAWN x: TypeX { a = x.id }           -- ERROR: x not yet bound
```

---

## 15. Transaction Boundaries

### 15.1 Atomic execution

A transaction includes:
1. User mutation(s)
2. Constraint checking on mutations
3. All triggered rule executions
4. Constraint checking on rule results

If any step fails, the entire transaction rolls back.

### 15.2 Execution flow

```
BEGIN TRANSACTION
  1. Apply user mutation
  2. Check constraints on mutation
     → If hard constraint fails: ROLLBACK
  3. Find triggered rules (auto=true)
  4. For each rule match, in priority order:
     a. Execute production actions
     b. Check constraints on results
        → If hard constraint fails: ROLLBACK
     c. Find newly triggered rules
  5. Repeat step 4 until quiescence or cycle
     → If cycle detected: ROLLBACK (or warn, based on config)
COMMIT
```

### 15.3 Rule failure semantics

If a rule's production:
- Violates a constraint: entire transaction rolls back
- Creates a duplicate (already exists): skip that action, continue
- References invalid variable: rule definition error (caught at compile time)

---

## 16. Constraint Indexing

### 16.1 Constraint-to-type index

At ontology compile time, build:

```
constraintsByType: Map<TypeName, Set<ConstraintDef>>
```

For each constraint:
1. Extract all type names from its pattern
2. Add constraint to each type's set

### 16.2 Affected constraint detection

On mutation of entity with type T:

```
affectedConstraints = constraintsByType[T]
for each ancestor A of T:
  affectedConstraints ∪= constraintsByType[A]
```

### 16.3 edge<any> constraints

Constraints mentioning `edge<any>` are added to a special set checked on ALL edge mutations. Use sparingly.

---

## 17. Reserved Words

See **Part I: Foundations §2.5** for the complete list of reserved keywords.

### 17.1 Reserved prefixes

| Prefix | Usage |
| --- | --- |
| `_` | Layer 0 internal names |
| `__` | Engine internal (not exposed) |

---

## 18. Layer 0 Bootstrapping Clarification

### 18.1 Two levels of existence

Layer 0 types exist in two senses:

**1. As engine primitives (hardcoded)**:
- The engine knows _NodeType, _EdgeType, etc. as built-in concepts
- These enable validation and type checking
- They are NOT stored as nodes in the graph
- There is no _NodeType node representing _NodeType itself

**2. As instances (for user ontologies)**:
- When you define `node Event {}`, the compiler creates a _NodeType node with name="Event"
- This IS stored in the graph
- This enables observing the ontology structure

### 18.2 The recursion stops

Layer 0 is the axiomatic foundation. Asking "what is the type of _NodeType?" is like asking "what set contains the set of all sets"—the question doesn't apply at this level.

### 18.3 Self-observing capability

User-defined types can be observed because they're stored as nodes:

```
-- List all types in the loaded ontology
MATCH t: _NodeType
RETURN t.name

-- Get attributes of a specific type
MATCH 
  t: _NodeType,
  a: _AttributeDef,
  _type_has_attribute(t, a)
WHERE t.name = "Event"
RETURN a.name, a.required
```

---

## 19. Edge Type Semantics

### 19.1 Edge inheritance

Edge types do **not** support inheritance in v1.

Unlike node types (which can use `_type_inherits`), edge types cannot extend other edge types. Each edge type is standalone.

**Rationale**: Edge inheritance would require complex signature compatibility rules. Keeping edge types flat simplifies the type system.

### 19.2 Symmetric edges

When `_EdgeType.symmetric = true`:

**Creation**: Creating a symmetric edge that already exists (in either order) returns the existing edge's ID:
```
-- If similar(a, b) already exists:
LINK similar(b, a)  -- Returns existing edge ID, no duplicate created
```

**Storage**: Only one edge is stored, with targets canonicalized by ID order:
```
-- For symmetric edge similar(a, b):
-- Stored as similar(min(a.id, b.id), max(a.id, b.id))
```

**Matching**: Observations match regardless of argument order:
```
-- Both observations find the same edge:
MATCH similar(x, y) WHERE x.name = "A"
MATCH similar(y, x) WHERE x.name = "A"
```

**Attributes**: Belong to the single stored edge.

### 19.3 Reflexive edges

When `_EdgeType.reflexive_allowed = false`:

Creating an edge where any two targets are the same node is rejected:
```
-- If reflexive_allowed = false for 'depends_on':
depends_on(task, task)  -- ERROR: reflexive edge not allowed
```

---

## 20. Protected Types

### 20.1 Definition

Layer 0 types are "protected"—they cannot be instantiated directly through normal graph operations.

Protected types include all types whose names begin with underscore:
```
_NodeType, _EdgeType, _AttributeDef, _ConstraintDef, _RuleDef,
_PatternDef, _VarDef, _EdgePattern, _ProductionDef, _Action,
_SpawnAction, _LinkAction, _KillAction, _UnlinkAction, _SetAction,
_Expr, _LiteralExpr, _VarRefExpr, _AttrAccessExpr, _BinaryOpExpr,
_UnaryOpExpr, _ExistsExpr, _IfExpr, _CaseExpr, _WhenClause, _CoalesceExpr,
_ListExpr, _TypeExpr, _NamedTypeExpr, _OptionalTypeExpr, _UnionTypeExpr,
_EdgeRefTypeExpr, _AnyTypeExpr, _ScalarTypeExpr, _Ontology, _Import
```

### 20.2 Enforcement

Attempting to create a protected type directly results in an error:

```
SPAWN("_NodeType", { name: "Foo" })
→ Error: "Cannot create protected type '_NodeType' directly. 
          Use engine.loadOntology() or engine.extendOntology() instead."
```

### 20.3 Allowed Operations

Protected types can be:
- **READ**: Match, traverse, inspect (always allowed)
- **CREATED**: Only through compiler (ontology loading, extension API)
- **MODIFIED**: Only through compiler (attribute updates on existing)
- **DELETED**: Only through compiler (ontology unloading, if supported)

### 20.4 Rationale

Protected types define the structure of ontologies. Unrestricted creation could produce invalid ontologies, corrupt registries, or create inconsistent state. The compiler validates all meta-structure before creation.

### 20.5 Extension API

To create new types at runtime, use the engine's extension API:

```typescript
engine.extendOntology(`
  node NewType {
    attr: String
  }
`)
```

This goes through full compiler validation before creating Layer 0 structure.

---

## 21. Summary: What Layer 0 Provides

| Component | Count | Purpose |
| --- | --- | --- |
| Node types | 31 | Represent ontology structure |
| Edge types | 49 | Connect ontology components |
| Constraints | 19 | Ensure ontology validity |
| Operations | 15 | Primitive manipulations |
| Scalars | 7 | Primitive value types |

This is complete and self-contained. An implementation that correctly handles all of the above can:

- Parse and store any valid ontology
- Validate ontologies against Layer 0 constraints
- Store user instances conforming to ontologies
- Enforce user constraints
- Execute user rewrite rules
- Support versioning and transactions
- Observe its own ontology structure (self-model)

---
