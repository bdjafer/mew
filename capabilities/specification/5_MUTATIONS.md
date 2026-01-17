# MEW Language Specification

## Part V: Mutation Operations

**Version:** 1.0
**Status:** Draft
**Scope:** Write operations and transaction control

---

# 1. Transformation: SPAWN

## 1.1 Purpose

SPAWN creates a new node in the graph.

## 1.2 Syntax

```
SpawnStmt =
  "spawn" Identifier ":" TypeExpr AttrBlock?
  ReturningClause?

AttrBlock = "{" (AttrAssignment ("," AttrAssignment)*)? "}"

AttrAssignment = Identifier "=" Expr

ReturningClause = "returning" ("id" | "*" | Identifier ("," Identifier)*)
```

## 1.3 Examples

```
-- Basic spawn
SPAWN t: Task {
  title = "New task",
  priority = 5
}

-- With all attributes
SPAWN p: Person {
  name = "Alice Smith",
  email = "alice#example.com",
  role = "admin",
  active = true
}

-- Minimal (relies on defaults)
SPAWN t: Task {
  title = "Minimal task"
}

-- Get created ID back
SPAWN t: Task { title = "Test" }
RETURNING id

-- Get full created node
SPAWN p: Person { name = "Bob", email = "bob#example.com" }
RETURNING *
```

## 1.4 Attribute Values

### 23.4.1 Literals

```
SPAWN t: Task {
  title = "Hello",
  priority = 5,
  score = 3.14,
  active = true,
  description = null
}
```

### 23.4.2 Expressions

```
SPAWN t: Task {
  title = "Task " ++ $suffix,
  created_at = now(),
  priority = $basePriority + 1
}
```

### 23.4.3 Defaults

Attributes with defaults in ontology can be omitted:

```
-- Ontology: status: String = "todo"
SPAWN t: Task { title = "Test" }
-- t.status = "todo" (default applied)
```

## 1.5 Validation

Before committing, the engine validates:

1. **Type exists:** The specified type is declared in ontology
2. **Required attributes:** All `[required]` attributes are present
3. **Type checking:** Attribute values match declared types
4. **Constraints:** All attribute modifiers satisfied

```
-- Error: missing required attribute
SPAWN p: Person { name = "Alice" }
-- ERROR: Required attribute 'email' not provided

-- Error: type mismatch
SPAWN t: Task { title = "Test", priority = "high" }
-- ERROR: Type mismatch: priority expects Int, got String

-- Error: constraint violation
SPAWN p: Person { name = "A", email = "invalid" }
-- ERROR: Constraint 'person_email_match' violated
```

## 1.6 RETURNING Clause

Control what's returned after creation:

```
-- Return only ID (default)
SPAWN t: Task { title = "Test" }
RETURNING id
-- { id: "task_abc123" }

-- Return full node
SPAWN t: Task { title = "Test" }
RETURNING *
-- { id: "task_abc123", _type: "Task", title: "Test", status: "todo", ... }

-- Return specific attributes
SPAWN p: Person { name = "Alice", email = "alice#example.com" }
RETURNING id, name, email
-- { id: "person_xyz", name: "Alice", email: "alice#example.com" }
```

## 1.7 Result Format

```typescript
interface SpawnResult {
  success: boolean
  id: string                     // created node ID
  data: Record<string, Value>    // based on RETURNING
  errors?: string[]              // if validation failed
}
```

## 1.8 AST

```typescript
interface SpawnStmt {
  kind: "Spawn"
  variable: string
  type: TypeExpr
  attributes: AttrAssignment[]
  returning: ReturningClause
}

type ReturningClause =
  | { kind: "id" }
  | { kind: "all" }
  | { kind: "fields", fields: string[] }
```

---

# 2. Transformation: KILL

## 2.1 Purpose

KILL removes a node from the graph.

## 2.2 Syntax

```
KillStmt =
  "kill" Target
  CascadeClause?
  ReturningClause?

Target = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern

CascadeClause = "cascade" | "no" "cascade"
```

## 2.3 Examples

```
-- Kill by ID
KILL #task_123

-- Kill by pattern
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }

-- With cascade behavior
KILL #project_456 CASCADE

-- Prevent cascade (explicit)
KILL #person_789 NO CASCADE

-- Return killed ID
KILL #task_123
RETURNING id
```

## 2.4 Edge Handling

When a node is killed, connected edges are handled according to their `on_kill_*` modifiers:

| Modifier | Behavior |
|----------|----------|
| `unlink` (default) | Edge is removed |
| `cascade` | Connected node is killed |
| `prevent` | Kill operation fails |

### 2.4.1 Default: Unlink

```
-- Edge: assigned_to(task, person) [on_kill_target: unlink]

KILL #person_123
-- All assigned_to edges pointing to person are removed
-- Tasks remain (just unassigned)
```

### 2.4.2 Cascade

```
-- Edge: belongs_to(task, project) [on_kill_target: cascade]

KILL #project_456
-- All tasks belonging to project are also killed
-- (and their edges, recursively)
```

### 2.4.3 Prevent

```
-- Edge: member_of(person, team) [on_kill_target: prevent]

KILL #team_789
-- ERROR: Cannot kill Team: has 5 member_of edges
-- Operation fails, nothing changes
```

## 2.5 CASCADE Override

The CASCADE clause can override ontology-defined behavior:

```
-- Force cascade even if ontology says unlink
KILL #project_123 CASCADE

-- Prevent cascade even if ontology says cascade
KILL #project_123 NO CASCADE
-- Note: unlinks edges but doesn't cascade
```

**Note:** Cannot override `prevent` — if ontology says prevent, the only option is to first remove blocking edges.

## 2.6 Bulk Kill

Pattern-based kill operates on all matches:

```
-- Kill all archived tasks
KILL { MATCH t: Task WHERE t.status = "archived" RETURN t }
-- Returns count of killed nodes

-- Kill with limit (safety)
KILL { 
  MATCH t: Task WHERE t.created_at < $cutoff RETURN t LIMIT 1000 
}
```

## 2.7 Result Format

```typescript
interface KillResult {
  success: boolean
  killedCount: number
  killedIds: string[]
  cascadeCount: number           // nodes killed via cascade
  unlinkedEdges: number          // edges removed
  errors?: string[]
}
```

## 2.8 AST

```typescript
interface KillStmt {
  kind: "Kill"
  target: IdRef | MatchStmt
  cascade: "default" | "cascade" | "no-cascade"
  returning: ReturningClause | null
}
```

---

# 3. Transformation: LINK

## 3.1 Purpose

LINK creates a new edge connecting nodes.

## 3.2 Syntax

```
LinkStmt =
  "link" EdgeType "(" TargetList ")" AliasClause? AttrBlock?
  ReturningClause?

TargetList = TargetRef ("," TargetRef)*

TargetRef = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern (must return single)
  
AliasClause = "as" Identifier
```

## 3.3 Examples

```
-- Basic link by ID
LINK causes(#event_123, #event_456)

-- With attributes
LINK assigned_to(#task_123, #person_456) {
  assigned_at = now(),
  role = "owner"
}

-- With alias (for returning)
LINK causes(#e1, #e2) AS c
RETURNING *

-- Using pattern to find target
LINK belongs_to(
  #task_new,
  { MATCH p: Project WHERE p.name = "Main" RETURN p }
)

-- Higher-order edge
LINK confidence(#causes_edge_123, 0.9) {
  assessed_by = "expert"
}
```

## 3.4 Target Resolution

### 25.4.1 By ID

```
LINK causes(#node_a, #node_b)
```

Direct reference to existing nodes.

### 25.4.2 By Pattern

```
LINK assigned_to(
  #task_123,
  { MATCH p: Person WHERE p.email = "alice#example.com" RETURN p }
)
```

Pattern must return exactly one node. Error if zero or multiple.

### 25.4.3 Mixed

```
LINK belongs_to(
  #new_task,
  { MATCH p: Project WHERE p.id = $projectId RETURN p }
)
```

### 25.4.4 Inline SPAWN

Create a node and link it in a single statement:

```
LINK belongs_to(
  SPAWN Task { title = "New task", priority = 5 },
  #project_123
)
RETURNING id
```

**Syntax:**
```
TargetRef = 
    IdRef                           -- by ID
  | "{" MatchStmt "}"               -- by pattern (must return single)
  | SpawnExpr                       -- inline spawn

SpawnExpr = "spawn" TypeExpr AttrBlock? ("as" Identifier)?
```

**Examples:**
```
-- Create task and assign in one statement
LINK assigned_to(
  SPAWN Task { title = "Review PR #123" },
  #person_alice
) AS assignment
RETURNING assignment.id

-- Create both endpoints with AS binding
LINK causes(
  SPAWN Event { name = "User clicked", timestamp = now() } AS cause,
  SPAWN Event { name = "Page loaded", timestamp = now() + 100 } AS effect
) AS causation
RETURNING cause.id AS cause_id, effect.id AS effect_id, causation.id AS edge_id

-- Create node, link to existing project
LINK belongs_to(
  SPAWN Task { title = "Implement feature X" },
  { MATCH p: Project WHERE p.name = "Main" RETURN p }
)
```

### 25.4.5 AS Binding in Inline SPAWN

Use `AS` to bind spawned nodes for use in RETURNING:

```
LINK belongs_to(
  SPAWN Task { title = "New task" } AS t,
  #project_123
) AS e
RETURNING t.id AS task_id, e.id AS edge_id
```

**Scope:** Bindings are available only in the RETURNING clause of the same statement.

**Multiple spawns:**
```
LINK collaboration(
  SPAWN Person { name = "Alice" } AS alice,
  SPAWN Person { name = "Bob" } AS bob
) AS collab
RETURNING alice.id, bob.id, collab.id
```

**Without AS:** If no binding needed, RETURNING can still access the edge:
```
LINK assigned_to(SPAWN Task { title = "Test" }, #person)
RETURNING id  -- edge ID
```

**Semantics:**
- SPAWN expressions are evaluated first (left to right)
- Created nodes are available for the LINK
- If LINK fails (constraint violation), all SPAWNs are rolled back
- RETURNING can access both the edge and spawned node IDs via AS bindings

**Benefit:** Eliminates the need for:
```
-- Old way (two statements)
SPAWN t: Task { title = "New task" } RETURNING id
LINK belongs_to(#t, #project_123)

-- New way (one statement)
LINK belongs_to(SPAWN Task { title = "New task" }, #project_123)
```

## 3.5 Edge Attributes

```
LINK assigned_to(#task, #person) {
  assigned_at = now(),
  role = "reviewer",
  notes = "Urgent review needed"
}
```

Same rules as node attributes:
- Required attributes must be provided
- Defaults apply for omitted attributes
- Type checking enforced

## 3.6 Validation

Before committing:

1. **Edge type exists:** Declared in ontology
2. **Target count:** Matches edge arity
3. **Target types:** Each target matches signature type
4. **Target exists:** All referenced nodes exist
5. **Constraints:** Edge modifiers satisfied (no_self, acyclic, cardinality, etc.)

```
-- Error: self-loop on no_self edge
LINK depends_on(#task_123, #task_123)
-- ERROR: Edge 'depends_on' does not allow self-loops

-- Error: cardinality violation
LINK assigned_to(#task_123, #person_789)
-- If task already has assignee and [task -> 0..1]:
-- ERROR: Cardinality constraint violated: task -> 0..1

-- Error: acyclic violation
LINK parent_of(#person_a, #person_b)
-- If this would create cycle:
-- ERROR: Edge 'parent_of' would create cycle
```

## 3.7 Uniqueness and Duplicate Edges

### 25.7.1 Default: Duplicates Allowed

By default, edges are **distinct entities**. Multiple edges of the same type can connect the same nodes:

```
edge tagged(task: Task, tag: Tag) {}  -- no [unique]

LINK tagged(#task_1, #tag_urgent)     -- creates edge_001
LINK tagged(#task_1, #tag_urgent)     -- creates edge_002 (different ID!)

-- Both edges exist. This may be intentional (e.g., multiple tagging events)
-- or accidental. Use [unique] to prevent if undesired.
```

### 25.7.2 With [unique]: Duplicates Prevented

If edge type has `[unique]` modifier:

```
edge member_of(person: Person, team: Team) [unique] {}

-- First link succeeds
LINK member_of(#person_123, #team_456)

-- Duplicate link fails
LINK member_of(#person_123, #team_456)
-- ERROR: Duplicate edge: member_of(person_123, team_456) already exists
```

### 25.7.3 When to Use [unique]

| Scenario | Use [unique]? | Reason |
|----------|---------------|--------|
| Membership (person → team) | ✅ Yes | Person is either a member or not |
| Assignment (task → person) | ✅ Yes | Task has at most one assignee |
| Tagging (item → tag) | ✅ Usually | Unless tracking multiple tag events |
| Event causation | ❌ No | Multiple causal links may exist |
| Audit trail edges | ❌ No | Each event is distinct |

## 3.8 LINK IF NOT EXISTS

Create an edge only if it doesn't already exist. Idempotent operation.

### 25.8.1 Syntax

```
LinkIfNotExistsStmt = 
  "link" "if" "not" "exists" EdgeType "(" Targets ")" 
  AttrBlock? 
  AsClause?
  ReturningClause?
```

### 25.8.2 Examples

```
-- Create assignment if not already assigned
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
RETURNING CREATED

-- Result: { created: true, id: "edge_123" } or { created: false, id: "edge_456" }
```

```
-- Idempotent membership
LINK IF NOT EXISTS member_of(#person, #team)

-- Can be called multiple times safely
-- First call: creates edge
-- Subsequent calls: no-op, returns existing edge
```

### 25.8.3 Semantics

| Scenario | Behavior | RETURNING CREATED |
|----------|----------|-------------------|
| Edge doesn't exist | Creates new edge | `true` |
| Edge already exists | No-op, returns existing | `false` |
| Edge type has `[unique]` | Same as without (redundant but allowed) | — |

**Matching criteria:** An edge "exists" if there's an edge of the same type connecting the same targets in the same order (or any order for symmetric edges).

**Attributes on existing edge:** If the edge exists, provided attributes are **ignored** (not updated). Use SET to update.

```
-- Edge exists with assigned_at = yesterday
LINK IF NOT EXISTS assigned_to(#task, #person) { assigned_at = now() }
-- Returns existing edge; assigned_at still = yesterday

-- To update if exists, use separate SET:
LINK IF NOT EXISTS assigned_to(#task, #person) AS a { assigned_at = now() }
SET a.last_checked = now()  -- Always updates
```

### 25.8.4 Use Cases

```
-- Idempotent API endpoint
LINK IF NOT EXISTS follows(#user, #target)
RETURNING id, CREATED AS is_new

-- Ensure relationship (defensive programming)
LINK IF NOT EXISTS belongs_to(#task, #project)

-- Batch import (skip duplicates)
FOR item IN $items:
  LINK IF NOT EXISTS tagged(#item.entity, #item.tag)
```

## 3.9 Result Format

```typescript
interface LinkResult {
  success: boolean
  id: string                     // created edge ID
  data: Record<string, Value>    // based on RETURNING
  errors?: string[]
}
```

## 3.10 AST

```typescript
interface LinkStmt {
  kind: "Link"
  edgeType: string
  targets: TargetRef[]
  alias: string | null
  attributes: AttrAssignment[]
  returning: ReturningClause | null
  ifNotExists: boolean              // true for LINK IF NOT EXISTS
}
```

---

# 4. Transformation: UNLINK

## 4.1 Purpose

UNLINK removes an edge from the graph.

## 4.2 Syntax

```
UnlinkStmt =
  "unlink" Target
  ReturningClause?

Target =
    IdRef                           -- by edge ID
  | EdgePattern                     -- by pattern
  | "{" MatchStmt "}"               -- by match
```

## 4.3 Examples

```
-- Unlink by edge ID
UNLINK #edge_abc123

-- Unlink by pattern (all matching)
UNLINK assigned_to(#task_123, _)
-- Removes all assignments for task_123

-- Unlink specific edge
UNLINK causes(#event_a, #event_b)

-- Unlink via match
UNLINK {
  MATCH t: Task, p: Person, assigned_to(t, p) AS e
  WHERE p.active = false
  RETURN e
}
```

## 4.4 Edge Identification

### 26.4.1 By ID

```
UNLINK #edge_xyz789
```

Direct reference to specific edge.

### 26.4.2 By Endpoints

```
UNLINK causes(#node_a, #node_b)
```

Removes the edge connecting these specific nodes.

### 26.4.3 By Partial Pattern

```
-- Remove all causes edges from event_a
UNLINK causes(#event_a, _)

-- Remove all edges to this node
UNLINK _(#_, #person_123)
-- ERROR: Must specify edge type

-- Use match for complex patterns
UNLINK {
  MATCH causes(e1, #person_123) AS edge
  RETURN edge
}
```

### 26.4.4 By Match

```
UNLINK {
  MATCH t: Task, assigned_to(t, _) AS e
  WHERE t.status = "done"
  RETURN e
}
-- Removes all assignments from completed tasks
```

## 4.5 Higher-Order Edge Cascade

When unlinking an edge, all higher-order edges referencing it are automatically unlinked:

```
-- Base edge
LINK causes(#e1, #e2) AS c

-- Higher-order edge
LINK confidence(c, 0.9)

-- Unlink base edge
UNLINK #c
-- The confidence edge is automatically unlinked
```

## 4.6 Bulk Unlink

Pattern-based unlink operates on all matches:

```
-- Remove all assignments older than 30 days
UNLINK {
  MATCH assigned_to(_, _) AS e
  WHERE e.assigned_at < now() - 2592000000
  RETURN e
}
RETURNING COUNT(*)
```

## 4.7 Result Format

```typescript
interface UnlinkResult {
  success: boolean
  unlinkedCount: number
  unlinkedIds: string[]
  cascadeCount: number           // higher-order edges removed
  errors?: string[]
}
```

## 4.8 AST

```typescript
interface UnlinkStmt {
  kind: "Unlink"
  target: IdRef | EdgePattern | MatchStmt
  returning: ReturningClause | null
}
```

---

# 5. Transformation: SET

## 5.1 Purpose

SET modifies attribute values on existing nodes or edges.

## 5.2 Syntax

```
SetStmt =
  "set" Target "." Identifier "=" Expr
  ReturningClause?

Target =
    IdRef                           -- single by ID
  | "{" MatchStmt "}"               -- bulk by pattern
```

## 5.3 Examples

```
-- Set single attribute
SET #task_123.status = "done"

-- Set multiple (separate statements)
SET #task_123.status = "done"
SET #task_123.completed_at = now()

-- Set with expression
SET #task_123.priority = #task_123.priority + 1

-- Bulk set
SET { MATCH t: Task WHERE t.status = "in_progress" RETURN t }.status = "blocked"

-- Set on edge
SET #assignment_edge.role = "reviewer"

-- Set to null (clear optional)
SET #task_123.description = null

-- Return updated value
SET #task_123.status = "done"
RETURNING status
```

## 5.4 Multiple Attributes

### 27.4.1 Block Syntax (Recommended)

Set multiple attributes in a single statement using block syntax:

```
SET #task_123 {
  status = "done",
  completed_at = now(),
  completed_by = "user_456"
}
```

**Grammar:**
```
SetStmt =
    "set" Target "." Identifier "=" Expr ReturningClause?     -- single
  | "set" Target AttrBlock ReturningClause?                   -- multiple

AttrBlock = "{" (AttrAssign ("," AttrAssign)*)? "}"
```

**Examples:**
```
-- Update task completion
SET #task_123 {
  status = "done",
  completed_at = now()
}

-- With RETURNING
SET #person_456 {
  name = "Alice Smith",
  email = "alice.smith#example.com"
}
RETURNING name, email

-- Bulk update with block
SET { MATCH t: Task WHERE t.status = "pending" RETURN t } {
  status = "archived",
  archived_at = now()
}
```

### 27.4.2 Separate Statements

Alternatively, use multiple SET statements in a transaction:

```
BEGIN
  SET #task_123.status = "done"
  SET #task_123.completed_at = now()
  SET #task_123.completed_by = "user_456"
COMMIT
```

**Note:** Block syntax is preferred as it's more concise and clearly shows all changes together.

## 5.5 Validation

Before committing:

1. **Node/edge exists:** Target must exist
2. **Attribute exists:** Declared on the type
3. **Type checking:** Value matches attribute type
4. **Constraints:** All attribute modifiers satisfied

```
-- Error: attribute doesn't exist
SET #task_123.nonexistent = "value"
-- ERROR: Attribute 'nonexistent' not found on type 'Task'

-- Error: type mismatch
SET #task_123.priority = "high"
-- ERROR: Type mismatch: priority expects Int, got String

-- Error: constraint violation
SET #person_123.age = -5
-- ERROR: Constraint 'person_age_min' violated: age must be >= 0

-- Error: required to null
SET #person_123.email = null
-- ERROR: Cannot set required attribute 'email' to null
```

## 5.6 Bulk SET

```
-- Set all matching
SET { MATCH t: Task WHERE t.project_id = #old_project RETURN t }.project_id = #new_project

-- With limit (safety)
SET { 
  MATCH t: Task WHERE t.status = "pending" RETURN t LIMIT 100 
}.status = "archived"
```

## 5.7 Computed Values

SET can use expressions referencing current values:

```
-- Increment
SET #counter_node.value = #counter_node.value + 1

-- Conditional (requires full expression support)
SET #task_123.priority = MIN(#task_123.priority + 1, 10)

-- String manipulation
SET #person_123.name = UPPER(#person_123.name)
```

## 5.8 Result Format

```typescript
interface SetResult {
  success: boolean
  modifiedCount: number
  modifiedIds: string[]
  errors?: string[]
}
```

## 5.9 AST

```typescript
interface SetStmt {
  kind: "Set"
  target: IdRef | MatchStmt
  attribute: string
  value: Expr
  returning: ReturningClause | null
}
```

---

## 5.10 Parameterized Queries

## 5.10.1 Overview

Parameterized queries allow safe, reusable queries with external values. Parameters prevent injection attacks and enable observation plan caching.

## 5.10.2 Syntax

```
Parameter = "$" Identifier

-- In observations:
MATCH t: Task WHERE t.id = $taskId RETURN t
SPAWN t: Task { title = $title, priority = $priority }
LINK assigned_to(#task_123, $personId)
```

## 5.10.3 Parameter Binding

Parameters are bound when executing the observation:

```typescript
// API usage (pseudocode)
engine.execute(
  "MATCH t: Task WHERE t.status = $status RETURN t",
  { status: "done" }
)

engine.execute(
  "SPAWN t: Task { title = $title, priority = $priority } RETURNING t.id",
  { title: "New task", priority: 5 }
)
```

## 5.10.4 Type Checking

Parameters are type-checked against their usage context:

```
-- $priority inferred as Int from Task.priority
MATCH t: Task WHERE t.priority > $priority RETURN t

-- $name inferred as String from Person.name
SPAWN p: Person { name = $name }

-- Type error if wrong type provided at execution:
-- ERROR: Parameter $priority expected Int, got String
```

## 5.10.5 Parameter Scope

Parameters are:
- **Immutable:** Cannot be modified within a query
- **Query-scoped:** Available throughout the entire query
- **Not nullable by default:** Use `$param?` for optional parameters

```
-- Optional parameter with default
MATCH t: Task 
WHERE t.status = COALESCE($status, "pending")
RETURN t

-- Or explicit optional syntax
MATCH t: Task
WHERE $filter? = null OR t.category = $filter
RETURN t
```

## 5.10.6 Parameter in Bulk Operations

```
KILL { MATCH t: Task WHERE t.project_id = $projectId RETURN t }
WITH projectId = "proj_123"

SET { MATCH t: Task WHERE t.assignee = $oldAssignee RETURN t }.assignee = $newAssignee
WITH oldAssignee = "user_old", newAssignee = "user_new"
```

## 5.10.7 Security

Parameters are **always** treated as values, never as identifiers or operators:

```
-- SAFE: $value is always a value
MATCH t: Task WHERE t.title = $value RETURN t
-- Even if $value contains "'; DROP TABLE tasks; --"

-- NOT POSSIBLE: Cannot parameterize type names or attributes
MATCH t: $type RETURN t        -- ERROR: Type cannot be parameterized
MATCH t: Task RETURN t.$attr   -- ERROR: Attribute cannot be parameterized
```

---

# 6. Transaction Control

## 6.1 Purpose

Transactions group multiple operations into an atomic unit. Either all succeed or all fail.

## 6.2 Syntax

```
TransactionStmt =
    BeginStmt
  | CommitStmt
  | RollbackStmt

BeginStmt = "begin" IsolationLevel?

IsolationLevel = "read" "committed" | "serializable"

CommitStmt = "commit"

RollbackStmt = "rollback"
```

## 6.3 Basic Usage

```
BEGIN
  SPAWN p: Project { name = "New Project" }
  SPAWN t: Task { title = "First task" }
  LINK belongs_to(#t, #p)
COMMIT
```

If any statement fails, everything rolls back.

## 6.4 Auto-Commit Mode

Without explicit BEGIN, each statement is its own transaction:

```
-- Auto-commit: each is separate transaction
SPAWN t: Task { title = "Task 1" }  -- commits immediately
SPAWN t: Task { title = "Task 2" }  -- commits immediately
```

## 6.5 Explicit Transactions

```
BEGIN
  -- Multiple operations
  SPAWN t: Task { title = "Test" }
  SET #existing_task.status = "done"
  LINK depends_on(#t, #other_task)
  -- Nothing committed yet
COMMIT  -- All committed atomically
```

## 6.6 Rollback

### 28.6.1 Explicit Rollback

```
BEGIN
  SPAWN t: Task { title = "Test" }
  -- Changed my mind
ROLLBACK
-- Node not created
```

### 28.6.2 Automatic Rollback

If any operation fails, automatic rollback:

```
BEGIN
  SPAWN t: Task { title = "Test" }       -- succeeds tentatively
  SET #nonexistent.status = "done"       -- fails
COMMIT
-- Automatic rollback: Task not created
-- ERROR: Node #nonexistent not found
```

## 6.7 Isolation Levels

### 28.7.1 READ COMMITTED (default)

```
BEGIN READ COMMITTED
  -- Sees committed data from other transactions
  -- No dirty reads
COMMIT
```

### 28.7.2 SERIALIZABLE

```
BEGIN SERIALIZABLE
  -- Full isolation
  -- Transactions appear to execute sequentially
COMMIT
```

## 6.8 Constraint Checking

Constraints are checked at COMMIT time (for cardinality) or operation time (for others):

| Constraint Type | When Checked |
|-----------------|--------------|
| Type validation | At operation |
| Required | At operation |
| Value constraints | At operation |
| Unique | At operation |
| no_self, acyclic | At operation |
| Cardinality | At COMMIT |

```
BEGIN
  SPAWN t: Task { title = "Test" }
  -- Cardinality [task -> 1] on belongs_to not yet checked
  SPAWN p: Project { name = "Proj" }
  LINK belongs_to(#t, #p)
  -- Now cardinality satisfied
COMMIT  -- Cardinality verified here
```

## 6.9 Rule Execution

Rules execute within the transaction:

```
BEGIN
  SPAWN t: Task { title = "Test", status = "done" }
  -- Rule 'auto_complete_timestamp' fires
  -- SET t.completed_at = now() happens within transaction
COMMIT
-- t has completed_at set
```

## 6.10 Nested Transactions (Savepoints)

```
BEGIN
  SPAWN p: Project { name = "Main" }
  
  SAVEPOINT sp1
    SPAWN t: Task { title = "Subtask" }
    -- Something goes wrong
  ROLLBACK TO sp1
  -- Task not created, Project still pending
  
  SPAWN t: Task { title = "Different task" }
COMMIT
-- Only Project and "Different task" committed
```

## 6.11 Result Format

```typescript
interface TransactionResult {
  action: "begin" | "commit" | "rollback"
  success: boolean
  transactionId: string | null
  errors?: string[]
}
```

## 6.12 AST

```typescript
interface BeginStmt {
  kind: "Begin"
  isolation: "read-committed" | "serializable"
}

interface CommitStmt {
  kind: "Commit"
}

interface RollbackStmt {
  kind: "Rollback"
  savepoint: string | null
}

interface SavepointStmt {
  kind: "Savepoint"
  name: string
}

interface RollbackToStmt {
  kind: "RollbackTo"
  savepoint: string
}
```

---

