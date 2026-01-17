# MEW Policy System

**Version:** 1.0
**Status:** Capability
**Scope:** Actor-based access control for graph operations
**Deferred to:** v2

---

# Part I: Context & Motivation

## 1.1 The Problem

MEW graphs can represent worlds with multiple actors — users, agents, services — who should have different capabilities. A task management system needs:

- Users who can only modify their own tasks
- Project leads who can manage project membership
- Admins who can modify system configuration
- Automated agents with carefully scoped access

Without policy, any actor can perform any operation. This is unacceptable for real-world systems.

## 1.2 Why Policy Is Not Constraints

Constraints and policy appear similar but differ fundamentally:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CONSTRAINTS                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Question: "Is the graph state valid?"                              │
│                                                                      │
│  Input:    Graph state                                              │
│  Output:   Valid / Invalid                                          │
│  Timing:   After mutation, before commit                            │
│  On fail:  Rollback mutation                                        │
│                                                                      │
│  Example:  "Tasks must have positive priority"                      │
│            constraint: t: Task => t.priority > 0                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                            POLICY                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Question: "Is this actor allowed to perform this operation?"       │
│                                                                      │
│  Input:    Actor + Operation + Target + Graph state                 │
│  Output:   Allow / Deny                                             │
│  Timing:   Before mutation attempt                                  │
│  On fail:  Reject immediately (no mutation attempted)               │
│                                                                      │
│  Example:  "Only assignees can modify task status"                  │
│            policy: ON SET(t: Task, "status")                        │
│                    ALLOW IF assigned_to(t, current_actor())         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

The critical difference: **policy requires execution context** (who is acting), not just graph state.

## 1.3 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Enforcement in kernel** | Every operation passes through policy. Not optional. Not bypassable. |
| **Definitions in graph** | Policy rules are graph structure. Queryable. Evolvable. Self-describing. |
| **Policy vs grants** | Policy logic compiles rarely. Grant relationships change constantly. |
| **Pre-check semantics** | Deny before mutation, not rollback after. |
| **Explicit actor binding** | Sessions must declare who is acting. No ambient authority. |

---

# Part II: Core Model

## 2.1 Concepts

```
┌─────────────────────────────────────────────────────────────────────┐
│                        POLICY CONCEPTS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ACTOR                                                              │
│  ─────                                                              │
│  An entity performing operations. Typically a Person, Agent, or    │
│  Service node. Bound to a session.                                  │
│                                                                      │
│  OPERATION                                                          │
│  ─────────                                                          │
│  An action on the graph: SPAWN, KILL, LINK, UNLINK, SET, MATCH.    │
│  Includes META variants.                                            │
│                                                                      │
│  TARGET                                                             │
│  ──────                                                             │
│  The resource being operated on. A node, edge, type, or pattern.   │
│                                                                      │
│  POLICY                                                             │
│  ──────                                                             │
│  A rule defining when operations are allowed or denied.            │
│  Compiled. Changes rarely.                                          │
│                                                                      │
│  GRANT                                                              │
│  ─────                                                              │
│  An edge conferring capability to an actor. Runtime data.          │
│  Changes frequently.                                                │
│                                                                      │
│  DECISION                                                           │
│  ────────                                                           │
│  The outcome: ALLOW or DENY. With optional message.                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 The Policy/Grant Separation

This is the central architectural insight:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   POLICY (Schema Layer)                GRANTS (Data Layer)          │
│   ─────────────────────                ───────────────────          │
│                                                                      │
│   "Admins can delete tasks"            "Alice is an admin"          │
│                                                                      │
│   policy admin_delete:                 LINK has_role(alice, admin)  │
│     ON KILL(t: Task)                                                │
│     ALLOW IF has_role(                 ← Policy queries grants      │
│       current_actor(),                                              │
│       r: Role                                                       │
│     ) WHERE r.name = "admin"                                        │
│                                                                      │
│   ┌─────────────────────┐              ┌─────────────────────┐     │
│   │ Compiles to kernel  │              │ Pure graph edges    │     │
│   │ Changes: rarely     │              │ Changes: constantly │     │
│   │ Triggers: recompile │              │ Triggers: nothing   │     │
│   └─────────────────────┘              └─────────────────────┘     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Policies define the rules of access. Grants instantiate access.**

Adding a user to a role: edge mutation, instant, no recompilation.
Adding a new kind of access rule: policy change, rare, triggers recompilation.

---

# Part III: Execution Model

## 3.1 Session Actor Binding

Every session operates on behalf of an actor:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SESSION                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   session_id: "sess_abc123"                                         │
│   actor: #alice          ← All operations attributed to Alice       │
│   current_txn: #txn_456                                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

BEGIN SESSION AS #alice
  SPAWN t: Task { title = "Test" }    ← Policy checked as Alice
  SET t.status = "done"               ← Policy checked as Alice
COMMIT
```

Sessions without an actor operate in **system context** — typically unrestricted, used for internal operations.

## 3.2 Operation Interception

Policy evaluates **before** any mutation:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      OPERATION FLOW                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Operation received                                                │
│         │                                                            │
│         ▼                                                            │
│   ┌───────────────────────────────────────────────────────────┐    │
│   │                     POLICY CHECK                          │    │
│   │                                                            │    │
│   │   Extract:  actor      = session.actor                    │    │
│   │             operation  = "SET"                            │    │
│   │             target     = #task_123                        │    │
│   │             attribute  = "status"                         │    │
│   │                                                            │    │
│   │   Evaluate policies against context                       │    │
│   │                                                            │    │
│   │   Decision:  ALLOW  →  continue                           │    │
│   │              DENY   →  reject immediately                 │    │
│   │                                                            │    │
│   └───────────────────────────────────────────────────────────┘    │
│         │                                                            │
│         ▼ (only if ALLOW)                                           │
│   ┌───────────────────────────────────────────────────────────┐    │
│   │              NORMAL MUTATION FLOW                          │    │
│   │                                                            │    │
│   │   Type Check → Apply → Constraints → Rules → Commit       │    │
│   │                                                            │    │
│   └───────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Key property: **Denied operations never touch the graph.**

## 3.3 Policy Evaluation

Multiple policies may match an operation. Resolution:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    POLICY RESOLUTION                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. Collect all policies matching (operation, target_type)         │
│                                                                      │
│   2. Evaluate each policy's condition with execution context        │
│                                                                      │
│   3. Gather decisions with priorities:                              │
│                                                                      │
│        Policy A [priority: 100]  →  ALLOW                           │
│        Policy B [priority: 50]   →  DENY                            │
│        Policy C [priority: 50]   →  ALLOW                           │
│                                                                      │
│   4. Resolution:                                                    │
│        • Highest priority wins                                      │
│        • Equal priority: DENY wins (secure default)                 │
│                                                                      │
│   5. Final decision: ALLOW (Policy A, priority 100)                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.4 Execution Context

Policy conditions have access to context beyond the graph:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EXECUTION CONTEXT                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   FUNCTION            RETURNS                 EXAMPLE               │
│   ────────            ───────                 ───────               │
│                                                                      │
│   current_actor()     The session's actor     #alice                │
│                                                                      │
│   operation()         Operation type string   "SET"                 │
│                                                                      │
│   target()            Target entity or null   #task_123             │
│                                                                      │
│   target_type()       Type name of target     "Task"                │
│                                                                      │
│   target_attr()       Attribute (for SET)     "status"              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

These functions are **only valid in policy conditions** — they require execution context that doesn't exist in normal constraints or rules.

## 3.5 Default Behavior

When no policy matches an operation, the system uses **default-deny**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DEFAULT BEHAVIOR                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   No matching policies → DENY                                       │
│                                                                      │
│   This is the secure default. To allow operations, you must have   │
│   at least one ALLOW policy that matches and evaluates to true.    │
│                                                                      │
│   Typical configuration:                                            │
│                                                                      │
│   1. High-priority bypass for superadmins [priority: 1000]         │
│   2. Domain-specific ALLOW policies [priority: 0-100]              │
│   3. Optional explicit default-deny [priority: -1000]              │
│                                                                      │
│   policy default_deny [priority: -1000]:                            │
│     ON *                                                            │
│     DENY IF true                                                    │
│     MESSAGE "Permission denied"                                     │
│                                                                      │
│   The explicit default-deny is optional (implicit behavior is      │
│   already deny) but provides a clear audit trail and custom        │
│   message for denied operations.                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part IV: Policy DSL

## 4.1 Syntax

```
PolicyDecl =
    "policy" Identifier Modifiers? ":"
    "ON" OperationPattern
    Decision "IF" Condition
    Message?

Modifiers = "[" "priority:" Int "]"

OperationPattern =
    "*"                                    -- any operation
  | OperationType                          -- any target
  | OperationType "(" Pattern ")"          -- typed target
  | OperationType "(" Pattern "," Attr ")" -- with attribute (SET)
  | OperationPattern "|" OperationPattern  -- alternatives

OperationType = "SPAWN" | "KILL" | "LINK" | "UNLINK" | "SET" | "MATCH"
              | "META" OperationType

Decision = "ALLOW" | "DENY"

Message = "MESSAGE" StringLiteral
```

## 4.2 Examples

```
-- Unconditional access for superadmins
policy superadmin_bypass [priority: 1000]:
  ON *
  ALLOW IF EXISTS(has_role(current_actor(), r) WHERE r.name = "superadmin")

-- Type-level: only admins can spawn Projects
policy admin_create_project:
  ON SPAWN(p: Project)
  ALLOW IF has_role(current_actor(), admin_role)

-- Attribute-level: only assignees can change task status
policy assignee_set_status:
  ON SET(t: Task, "status")
  ALLOW IF assigned_to(t, current_actor())

-- Resource-level: project members can read project tasks
policy member_read_tasks:
  ON MATCH(t: Task)
  ALLOW IF EXISTS(
    p: Project,
    belongs_to(t, p),
    project_member(current_actor(), p)
  )

-- Compound: editors can modify, admins can delete
policy editor_modify:
  ON SET(t: Task, _) | LINK(_, t) | UNLINK(_, t)
  ALLOW IF has_project_role(current_actor(), task_project(t), "editor")

policy admin_delete:
  ON KILL(t: Task)
  ALLOW IF has_project_role(current_actor(), task_project(t), "admin")

-- META operations: schema access requires elevated permission
policy meta_read:
  ON META MATCH(_)
  ALLOW IF has_capability(current_actor(), "schema_read")

policy meta_write:
  ON META SPAWN(_) | META SET(_) | META LINK(_) | META UNLINK(_)
  ALLOW IF has_capability(current_actor(), "schema_write")

-- Default deny (lowest priority)
policy default_deny [priority: -1000]:
  ON *
  DENY IF true
  MESSAGE "Permission denied"
```

## 4.3 Operation Patterns

| Pattern | Matches |
|---------|---------|
| `*` | Any operation |
| `SPAWN(_)` | Any node creation |
| `SPAWN(t: Task)` | Task creation |
| `KILL(t: Task)` | Task deletion |
| `SET(t: Task, _)` | Any Task attribute |
| `SET(t: Task, "status")` | Specific attribute |
| `LINK(e: assigned_to)` | Specific edge type |
| `MATCH(t: Task)` | Reading tasks |
| `META SPAWN(_)` | Schema creation |

---

# Part V: Standard Patterns

## 5.1 Role-Based Access Control (RBAC)

```
┌─────────────────────────────────────────────────────────────────────┐
│                           RBAC MODEL                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                    ┌──────────┐                                     │
│                    │   Role   │                                     │
│                    └────┬─────┘                                     │
│                         │                                            │
│          ┌──────────────┼──────────────┐                            │
│          │              │              │                             │
│          ▼              ▼              ▼                             │
│   ┌────────────┐ ┌────────────┐ ┌────────────┐                     │
│   │ Permission │ │ Permission │ │ Permission │                     │
│   │  (delete)  │ │   (edit)   │ │   (view)   │                     │
│   └────────────┘ └────────────┘ └────────────┘                     │
│                                                                      │
│   Schema:                                                           │
│   ────────                                                          │
│   node Role { name: String [required, unique] }                     │
│   node Permission {                                                 │
│     operation: String,                                              │
│     target_type: String?                                            │
│   }                                                                 │
│   edge has_role(actor: Person, role: Role)                         │
│   edge role_has_permission(role: Role, perm: Permission)           │
│                                                                      │
│   Policy:                                                           │
│   ───────                                                           │
│   policy rbac [priority: 100]:                                      │
│     ON *                                                            │
│     ALLOW IF EXISTS(                                                │
│       r: Role, p: Permission,                                       │
│       has_role(current_actor(), r),                                 │
│       role_has_permission(r, p),                                    │
│       WHERE p.operation = operation()                               │
│         AND (p.target_type = null OR p.target_type = target_type()) │
│     )                                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Attribute-Based Access Control (ABAC)

```
┌─────────────────────────────────────────────────────────────────────┐
│                           ABAC MODEL                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Decisions based on attributes of:                                 │
│     • Actor (department, clearance_level, location)                │
│     • Resource (classification, owner, project)                    │
│     • Environment (time, ip_address)                               │
│     • Operation (read, write, delete)                              │
│                                                                      │
│   Example policies:                                                 │
│   ─────────────────                                                 │
│                                                                      │
│   -- Same department access                                         │
│   policy same_department:                                           │
│     ON MATCH(doc: Document)                                         │
│     ALLOW IF current_actor().department = doc.department            │
│                                                                      │
│   -- Clearance-based access                                         │
│   policy clearance_check:                                           │
│     ON MATCH(doc: Document)                                         │
│     ALLOW IF current_actor().clearance >= doc.classification        │
│                                                                      │
│   -- Time-based restriction                                         │
│   policy business_hours:                                            │
│     ON SET(_, _)                                                    │
│     DENY IF hour(now()) < 9 OR hour(now()) > 17                     │
│     MESSAGE "Modifications only during business hours"              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.3 Relationship-Based Access Control

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RELATIONSHIP-BASED MODEL                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Access derived from graph relationships:                          │
│                                                                      │
│       Person ──member_of──▶ Project ◀──belongs_to── Task            │
│                                                                      │
│   "Can access task if member of task's project"                     │
│                                                                      │
│   policy project_member_access:                                     │
│     ON MATCH(t: Task) | SET(t: Task, _)                             │
│     ALLOW IF EXISTS(                                                │
│       p: Project,                                                   │
│       belongs_to(t, p),                                             │
│       member_of(current_actor(), p)                                 │
│     )                                                               │
│                                                                      │
│   Transitive relationships:                                         │
│   ─────────────────────────                                         │
│                                                                      │
│       Person ──manages──▶ Person ──manages──▶ Person                │
│                                                                      │
│   "Can access reports of anyone in your management chain"           │
│                                                                      │
│   policy management_chain:                                          │
│     ON MATCH(r: Report)                                             │
│     ALLOW IF EXISTS(                                                │
│       author: Person,                                               │
│       authored_by(r, author),                                       │
│       manages+(current_actor(), author)    ← transitive             │
│     )                                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.4 Resource Ownership

```
┌─────────────────────────────────────────────────────────────────────┐
│                      OWNERSHIP MODEL                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Simple ownership:                                                 │
│   ─────────────────                                                 │
│                                                                      │
│   edge owned_by(resource: any, owner: Person)                       │
│                                                                      │
│   policy owner_full_access:                                         │
│     ON *                                                            │
│     ALLOW IF target() != null                                       │
│       AND EXISTS(owned_by(target(), current_actor()))               │
│                                                                      │
│   Creator ownership (automatic):                                    │
│   ──────────────────────────────                                    │
│                                                                      │
│   rule auto_ownership:                                              │
│     n: any WHERE NOT EXISTS(owned_by(n, _))                         │
│     =>                                                              │
│     LINK owned_by(n, current_actor())                               │
│                                                                      │
│   Delegated ownership:                                              │
│   ────────────────────                                              │
│                                                                      │
│   edge can_delegate(owner: Person, delegate: Person, resource: any) │
│                                                                      │
│   policy delegated_access:                                          │
│     ON *                                                            │
│     ALLOW IF target() != null                                       │
│       AND EXISTS(can_delegate(_, current_actor(), target()))        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VI: META Mode Integration

## 6.1 Schema Access Control

META operations have their own policy scope:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      META POLICY LEVELS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   META READ                                                         │
│   ─────────                                                         │
│   META MATCH, META WALK, META DESCRIBE                              │
│   Query schema structure, inspect types/rules/constraints           │
│                                                                      │
│   META WRITE                                                        │
│   ──────────                                                        │
│   META CREATE, META SPAWN, META LINK, META SET, META UNLINK        │
│   Create types, modify rules, adjust constraints                    │
│   Cannot delete types with instances                                │
│                                                                      │
│   META ADMIN                                                        │
│   ──────────                                                        │
│   META KILL on types, modify Layer 0 (where permitted)              │
│   Destructive schema operations                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 6.2 Self-Modifying Policy

The system can reason about and modify its own access control:

```
-- Query own permissions
META MATCH p: _PolicyRule
WHERE policy_affects_type(p, "Task")
RETURN p.name, p.decision, p.priority

-- AGI adjusting its own capabilities (with appropriate meta permission)
META CREATE POLICY learned_access_pattern:
  ON MATCH(c: LearnedConcept)
  ALLOW IF confidence_above(c, 0.8)
```

## 6.3 Policy for Policy

Meta-level: who can modify policy rules?

```
policy meta_policy_read:
  ON META MATCH(p: _PolicyRule)
  ALLOW IF has_capability(current_actor(), "policy_admin")

policy meta_policy_write:
  ON META CREATE POLICY(_) | META SET(p: _PolicyRule, _)
  ALLOW IF has_capability(current_actor(), "policy_admin")
  
policy meta_policy_delete:
  ON META KILL(p: _PolicyRule)
  ALLOW IF has_capability(current_actor(), "root")
```

---

# Part VII: Caching & Performance

## 7.1 Cache Hierarchy

Policy is on the critical path. Caching is essential:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CACHE HIERARCHY                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   L1: Session Role Cache                                            │
│   ──────────────────────                                            │
│   Key:        actor_id                                              │
│   Value:      Set<Role>                                             │
│   Lifetime:   Session                                               │
│   Invalidate: has_role edge changes for this actor                  │
│                                                                      │
│   L2: Role Permission Cache                                         │
│   ─────────────────────────                                         │
│   Key:        role_id                                               │
│   Value:      Set<Permission>                                       │
│   Lifetime:   Long (roles change rarely)                            │
│   Invalidate: role_has_permission edge changes                      │
│                                                                      │
│   L3: Decision Cache                                                │
│   ──────────────────                                                │
│   Key:        (actor, operation, target_type, target_id?)           │
│   Value:      ALLOW | DENY                                          │
│   Lifetime:   Transaction or short TTL                              │
│   Invalidate: Any policy-relevant edge change                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.2 Cache Invalidation

The kernel tracks which edge types affect policy:

```
Policy-relevant edge types (derived from policy conditions):
  • has_role
  • role_has_permission
  • member_of
  • assigned_to
  • owned_by
  • ...

On LINK/UNLINK of policy-relevant edge:
  → Invalidate affected cache entries
  → Subsequent operations re-evaluate
```

---

# Part VIII: Error Model

## 8.1 Policy Errors

```
┌─────────────────────────────────────────────────────────────────────┐
│                       POLICY ERRORS                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   E7001 - PERMISSION_DENIED                                         │
│   ─────────────────────────                                         │
│   Operation rejected by policy.                                     │
│                                                                      │
│   Fields:                                                           │
│     actor:      The requesting actor                                │
│     operation:  What was attempted                                  │
│     target:     What it was attempted on                            │
│     policy:     The denying policy (if disclosable)                 │
│     message:    Custom message from policy                          │
│                                                                      │
│   E7002 - NO_ACTOR_BOUND                                            │
│   ──────────────────────                                            │
│   Operation requires actor but session has none.                    │
│                                                                      │
│   E7003 - INVALID_ACTOR                                             │
│   ─────────────────────                                             │
│   Bound actor does not exist or is not a valid actor type.         │
│                                                                      │
│   E7004 - POLICY_EVAL_ERROR                                         │
│   ─────────────────────────                                         │
│   Policy condition failed to evaluate.                              │
│   (Graph inconsistency, missing data, etc.)                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.2 Error Information Disclosure

Security consideration: error messages should not leak sensitive information.

```
-- To unauthorized user:
Error: Permission denied

-- To admin/debugging:
Error: Permission denied
  Actor: #alice
  Operation: KILL
  Target: #task_123 (Task)
  Denied by: admin_delete [priority: 50]
  Reason: has_project_role check failed - alice has role "viewer", needs "admin"
```

---

# Part IX: Interaction with Other Systems

## 9.1 Constraints

Policy runs **before** constraints:

```
Operation → POLICY → Mutation → Constraints → Rules → Commit
              ↑
              Deny here = no mutation attempted
```

A denied operation never reaches constraint checking.

## 9.2 Rules

Rules execute with **system authority** by default:

```
rule auto_timestamp:
  t: Task WHERE t.created_at = null
  =>
  SET t.created_at = now()    ← Executes as system, not as session actor
```

Rules can optionally inherit session authority:

```
rule user_triggered_action [inherit_authority]:
  t: Task WHERE t.status = "approved"
  =>
  LINK published(t, ...)      ← Policy checked as session actor
```

## 9.3 Transactions

Policy is checked **per operation**, not per transaction:

```
BEGIN
  SPAWN t: Task { ... }       ← Policy check
  SET t.priority = 5          ← Policy check  
  LINK belongs_to(t, p)       ← Policy check
COMMIT
```

Each operation independently checked. Transaction commits only if all pass.

## 9.4 Invocations

Invocations (external code execution, see COMPUTE.md) have configurable authority:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    INVOCATION AUTHORITY                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Default: Invocation creator's authority                           │
│   ──────────────────────────────────────                            │
│   When code mutates the graph, policy checks use the actor          │
│   who created the Invocation node (typically the session actor).    │
│                                                                      │
│   Restricted grants:                                                │
│   ──────────────────                                                │
│   CodeModules can declare maximum capabilities:                     │
│                                                                      │
│   node CodeModule {                                                 │
│     max_operations: String[]    -- ["SPAWN", "SET"]                 │
│     max_types: String[]         -- ["Task", "Notification"]         │
│   }                                                                 │
│                                                                      │
│   Effective authority = intersection(creator_authority, code_grants)│
│                                                                      │
│   System invocations:                                               │
│   ───────────────────                                               │
│   Invocations without a creator (spawned by reactors/schedulers)   │
│   execute with system authority unless explicitly scoped.          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

This ensures code cannot exceed the permissions of whoever invoked it, while CodeModule grants can further restrict capabilities.

## 9.5 Manual Rule Invocation

The `INVOKE` statement triggers manual rules. Policy applies to INVOKE itself:

```
policy invoke_permission:
  ON INVOKE(r: _RuleDef)
  ALLOW IF has_capability(current_actor(), "invoke_" ++ r.name)
```

Once allowed, the rule's own authority semantics apply (system or inherited).

---

# Part X: Observation Policy

## 10.1 The Core Intuition

In the physical world:
- I can only see what I have access to
- I don't get an "access denied" error when looking at a locked room — I simply don't see inside
- My view of the world is naturally filtered by my access
- Counting things means counting what I can perceive

This suggests: **Observation policy is filtering, not gating.**

The query runs. Results are filtered to what the actor can see. No errors for "unauthorized" rows — they simply don't appear in results.

## 10.2 Filtering Semantics

```
┌─────────────────────────────────────────────────────────────────────┐
│                  OBSERVATION POLICY MODEL                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Observation policy is FILTERING, not GATING.                      │
│                                                                      │
│   User query:                                                       │
│     MATCH t: Task WHERE t.priority > 5 RETURN t                     │
│                                                                      │
│   Policy:                                                           │
│     ON MATCH(t: Task)                                               │
│     ALLOW IF EXISTS(p: Project, belongs_to(t, p),                   │
│                     member_of(current_actor(), p))                  │
│                                                                      │
│   Effective execution:                                              │
│     MATCH t: Task, p: Project,                                      │
│           belongs_to(t, p),                                         │
│           member_of(current_actor(), p)   ← injected                │
│     WHERE t.priority > 5                                            │
│     RETURN t                                                        │
│                                                                      │
│   Result: Only tasks the actor can see, filtered by priority > 5   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Policy predicates **compose into the query**, not wrap it.

## 10.3 Three Levels of Observation Policy

| Level | Question | Enforcement | Example |
|-------|----------|-------------|---------|
| **Type-level** | Can actor query this type at all? | Pre-query gate | "Guests cannot query AuditLogs" |
| **Instance-level** | Which instances can actor see? | Query predicate injection | "Users see tasks in their projects" |
| **Attribute-level** | Which attributes are visible? | Result projection | "Users see task.title but not task.internal_score" |

## 10.4 Syntax Distinction

```
-- Type-level: condition does NOT reference the bound variable
policy no_audit_for_guests:
  ON MATCH(_: AuditLog)
  ALLOW IF has_role(current_actor(), r) WHERE r.name != "guest"

-- Instance-level: condition REFERENCES the bound variable
policy project_member_sees_tasks:
  ON MATCH(t: Task)
  ALLOW IF EXISTS(p: Project, belongs_to(t, p), member_of(current_actor(), p))

-- Attribute-level: new syntax for attribute projection
policy hide_internal_score:
  ON MATCH(t: Task).internal_score
  DENY IF NOT has_role(current_actor(), "analyst")
```

Type-level policies gate. Instance-level policies filter. Attribute-level policies project.

## 10.5 Policy Composition for Filtering

When multiple instance-level policies apply:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    POLICY COMPOSITION                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ALLOW policies: OR composition (any match permits visibility)     │
│   DENY policies: Override (explicit exclusion, checked after ALLOW) │
│                                                                      │
│   Resolution for instance X:                                        │
│     1. Evaluate all ALLOW policies for X                            │
│     2. If any ALLOW matches → candidate visible                     │
│     3. Evaluate all DENY policies for X                             │
│     4. If any DENY matches → exclude                                │
│     5. Otherwise → visible                                          │
│                                                                      │
│   Example:                                                          │
│     ALLOW IF owns(current_actor(), t)           -- own tasks        │
│     ALLOW IF assigned_to(t, current_actor())    -- assigned tasks   │
│     DENY IF t.confidential = true               -- but not if confidential │
│              AND NOT has_clearance(current_actor())                 │
│                                                                      │
│   Actor sees tasks they own OR are assigned to,                     │
│   EXCEPT confidential tasks without clearance.                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.6 Edge Visibility

Edges require their own policy, not derivation from nodes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      EDGE VISIBILITY                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Question: If I can see node A but not node B, can I see edge(A,B)?│
│                                                                      │
│   Default: Edge visible only if ALL targets visible                 │
│            (prevents information leakage about hidden nodes)        │
│                                                                      │
│   Override: Explicit edge policy can relax this                     │
│                                                                      │
│   policy see_public_connections:                                    │
│     ON MATCH(e: follows)                                            │
│     ALLOW IF follows(current_actor(), source(e))                    │
│     -- Can see who your followees follow, even if you can't see     │
│     -- those people's profiles directly                             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.7 Aggregate Semantics

Aggregates operate on the **filtered** result set:

```
User query:
  MATCH t: Task RETURN COUNT(t)

With policy filtering to 3 of 10 total tasks:
  Result: 3

NOT 10. NOT "access denied". The actor's world contains 3 tasks.
```

This is the only coherent semantics — aggregates reflect what you can see.

## 10.8 Query Planning Integration

Policy predicates inject at planning time, not as post-filters:

```
┌─────────────────────────────────────────────────────────────────────┐
│                  QUERY PLANNING WITH POLICY                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   User query:  MATCH t: Task WHERE t.priority > 5                   │
│                                                                      │
│   Policy:      ALLOW IF EXISTS(belongs_to(t, p),                    │
│                               member_of(current_actor(), p))        │
│                                                                      │
│   Naive (post-filter):                                              │
│     1. Fetch all Tasks where priority > 5                           │
│     2. For each, check policy                                       │
│     3. Filter out hidden                                            │
│     → Wasteful: fetches rows just to discard them                   │
│                                                                      │
│   Optimized (predicate injection):                                  │
│     1. Rewrite query to include policy joins                        │
│     2. Plan combined query with indexes                             │
│     3. Execute once, results are already filtered                   │
│                                                                      │
│   Effective plan:                                                   │
│     IndexScan(member_of, actor=current_actor) → project_ids        │
│     IndexScan(belongs_to, project IN project_ids) → task_ids       │
│     IndexScan(Task, id IN task_ids, priority > 5)                   │
│                                                                      │
│   Policy becomes part of the access path, not a filter.             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.9 The Existence Problem

Subtle case: Can the actor know something exists even if they can't see its content?

```
┌─────────────────────────────────────────────────────────────────────┐
│                  EXISTENCE VS CONTENT VISIBILITY                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Scenario: Private tasks exist. User queries COUNT(Task).          │
│                                                                      │
│   Option A: COUNT returns only visible tasks                        │
│             User doesn't know private tasks exist                   │
│             → Information hiding (default)                          │
│                                                                      │
│   Option B: COUNT returns total, content hidden                     │
│             User knows "there are 10 tasks, I can see 3"            │
│             → Existence exposed, content protected                  │
│                                                                      │
│   MEW default: Option A (full filtering)                            │
│                                                                      │
│   Explicit override possible:                                       │
│     policy existence_visible:                                       │
│       ON MATCH(t: Task).existence    ← special attribute            │
│       ALLOW IF true                                                 │
│                                                                      │
│     MATCH t: Task RETURN COUNT(t)         → 3 (only visible)       │
│     MATCH t: Task RETURN COUNT_EXISTS(t)  → 10 (existence-visible) │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.10 WALK and Transitive Policy

Path traversal has policy at each hop:

```
WALK FROM #start FOLLOW follows [depth: 3] RETURN PATH

At each hop:
  1. Can actor see the edge being traversed?
  2. Can actor see the target node?

If either fails, that branch terminates (not error — just no further traversal).

Result: The subgraph reachable within actor's visibility.
```

## 10.11 Visibility Metadata

MEW defaults to silent filtering — actors see only what they can access, with no indication of hidden content. This mirrors physical reality: you don't see a sign saying "7 rooms hidden from you."

For debugging and UX purposes, an optional query modifier can expose visibility metadata:

```
MATCH t: Task RETURN t WITH VISIBILITY_INFO
-- Returns results + { visible_count: 3 }
```

Note: Exposing `total_count` would leak information about hidden resources and is intentionally not provided by default. Type-level existence indicators may be added in future versions with appropriate policy controls.

## 10.12 Implementation Phases

| Phase | Scope | Complexity |
|-------|-------|------------|
| **v2.0** | Type-level gating | Low — pre-query check |
| **v2.0** | Instance-level filtering (simple) | Medium — predicate injection |
| **v2.1** | Edge visibility rules | Medium — default + override |
| **v2.1** | Attribute-level projection | Medium — result transformation |
| **v2.2** | Optimized query planning | High — planner integration |

---

# Part XI: Versioning Considerations

## 11.1 v1 Anticipation

Policy is deferred to v2, but v1 must anticipate:

| Element | v1 Requirement |
|---------|----------------|
| Session.actor | Field exists, optional, unused |
| Pre-check hook | Method exists, returns Ok(()) |
| EvalContext | Struct exists with optional fields |
| Registry | Extensible for future policy rules |

## 11.2 v2 Implementation

Full policy system:

| Element | v2 Delivery |
|---------|-------------|
| Policy DSL | Parser, compiler support |
| _PolicyRule | Layer 0 type |
| Policy evaluation | Policy component |
| Context functions | current_actor(), etc. |
| Caching | Multi-level cache system |
| META integration | Policy for schema operations |

## 11.3 Future Extensions

| Extension | Description |
|-----------|-------------|
| Row-level security | Automatic MATCH filtering by policy |
| Capability delegation | Actors granting subsets of their access |
| Temporal policies | Time-bounded access grants |
| Audit integration | Policy decisions in audit log |
| Policy simulation | "What if" analysis for policy changes |

---

# Appendix A: Complete Grammar

```ebnf
(* Policy Declarations *)
PolicyDecl       = "policy" Identifier PolicyMods? ":"
                   "ON" OpPattern
                   Decision "IF" Expr
                   Message?

PolicyMods       = "[" "priority:" IntLiteral "]"

OpPattern        = "*"
                 | OpType
                 | OpType "(" Pattern ")"
                 | OpType "(" Pattern "," AttrName ")"
                 | OpPattern "|" OpPattern

OpType           = "SPAWN" | "KILL" | "LINK" | "UNLINK" | "SET" | "MATCH"
                 | "META" OpType

Decision         = "ALLOW" | "DENY"

Message          = "MESSAGE" StringLiteral

(* Context Functions - valid only in policy conditions *)
ContextFunc      = "current_actor" "(" ")"
                 | "operation" "(" ")"
                 | "target" "(" ")"
                 | "target_type" "(" ")"
                 | "target_attr" "(" ")"

(* Session Binding *)
SessionStmt      = "BEGIN" "SESSION" "AS" NodeRef
                 | "END" "SESSION"
```

---

# Appendix B: Layer 0 Extensions

```
node _PolicyRule {
  name: String [required, unique],
  priority: Int = 0,
  decision: String [in: ["allow", "deny"]],
  message: String?,
  doc: String?
}

node _OperationPattern {
  operation: String,         -- "SPAWN", "SET", "*", etc.
  target_type: String?,      -- null = any
  target_attr: String?       -- for SET
}

edge _policy_has_pattern(
  rule: _PolicyRule,
  pattern: _OperationPattern
)

edge _policy_has_condition(
  rule: _PolicyRule,
  condition: _Expr
)

edge _ontology_declares_policy(
  ontology: _Ontology,
  rule: _PolicyRule
)
```

---

# Appendix C: Glossary

| Term | Definition |
|------|------------|
| **Actor** | Entity performing operations; bound to session |
| **ABAC** | Attribute-Based Access Control; decisions based on entity attributes |
| **Decision** | ALLOW or DENY outcome |
| **Default-deny** | Security model where no matching policy means denied |
| **Execution context** | Actor + operation + target; available in policy conditions |
| **Filtering** | Observation policy that removes invisible results without error |
| **Gating** | Mutation policy that rejects operations before execution |
| **Grant** | Edge conferring capability; runtime data |
| **Policy** | Rule defining when operations are allowed or denied |
| **Pre-check** | Policy evaluation before mutation |
| **RBAC** | Role-Based Access Control; permissions derived from roles |
| **ReBAC** | Relationship-Based Access Control; permissions derived from graph relationships |
| **System authority** | Operations not attributed to any actor; bypasses policy |

---

*End of MEW Policy System Capability*

