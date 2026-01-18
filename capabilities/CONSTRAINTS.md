# MEW Constraints

**Version:** 1.0
**Status:** Capability
**Scope:** Declarative validation, invariants, structural integrity

---

# Part I: The Core Insight

## 1.1 Validation Without Code

Most systems validate data with imperative code:

```javascript
// The imperative way
function createTask(task) {
  if (!task.title) throw new Error("Title required");
  if (task.priority < 0 || task.priority > 10) throw new Error("Invalid priority");
  if (task.status === "done" && !task.completedAt) throw new Error("Need timestamp");
  // ... scattered across codebase
  // ... easy to forget
  // ... inconsistent enforcement
}
```

MEW inverts this. You declare what must be true. The system enforces it:

```
constraint task_title_required:
  t: Task => t.title != null

constraint task_priority_valid:
  t: Task WHERE t.priority != null
  => t.priority >= 0 AND t.priority <= 10

constraint completed_has_timestamp:
  t: Task WHERE t.status = "done"
  => t.completed_at != null
```

**The shift:** From "check before write" to "declare what's valid, writes that violate fail."

## 1.2 What This Changes

| Imperative Validation | Declarative Constraints |
|-----------------------|------------------------|
| Scattered across codebase | Centralized in schema |
| Easy to bypass | Enforced by kernel |
| Checked when you remember | Checked on every mutation |
| Hard to audit | Queryable, introspectable |
| Different per client | Universal |

## 1.3 The Guarantee

When a constraint exists, **it cannot be violated**. Not by any client. Not by any code path. Not by direct database access. The kernel enforces it at commit time.

```
constraint no_negative_balance:
  a: Account => a.balance >= 0

-- This is not a suggestion. It's a law of this universe.
-- No transaction can commit that would make it false.
```

## 1.4 Constraints as Documentation

Constraints are executable documentation. They state the rules of the domain:

```
-- Business rule: causes must precede effects
constraint temporal_order:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- Organizational rule: managers must be team members
constraint manager_is_member:
  p: Person, t: Team, manages(p, t)
  => EXISTS(member_of(p, t))

-- Data quality rule: high-priority tasks need owners
constraint high_priority_assigned [soft]:
  t: Task WHERE t.priority >= 8
  => EXISTS(p: Person, assigned_to(t, p))
```

Reading the constraints tells you how the system works.

---

# Part II: What Constraints Express

## 2.1 The Universal Form

Every constraint has the same structure:

```
constraint <name>:
  <pattern>
  => <condition>
```

This reads: **"For all matches of pattern, condition must be true."**

The constraint is violated if ANY match exists where condition is false.

## 2.2 Prohibition

"This must not exist."

```
-- No self-loops
constraint no_self_cause:
  e: Event, causes(e, e)
  => false

-- No duplicate assignments
constraint no_double_assign:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1), assigned_to(t, p2)
  WHERE p1.id != p2.id
  => false
```

The pattern describes what's forbidden. The condition is `false`.

## 2.3 Requirement

"This must exist."

```
-- Every task needs a project
constraint task_has_project:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))

-- Every project needs an owner
constraint project_has_owner:
  p: Project
  => EXISTS(o: Person, owns(o, p))
```

The `EXISTS` expression checks that something exists.

## 2.4 Implication

"If X, then Y."

```
-- If completed, must have timestamp
constraint completed_has_timestamp:
  t: Task WHERE t.status = "done"
  => t.completed_at != null

-- If manager, must be senior
constraint managers_are_senior:
  p: Person, t: Team, manages(p, t)
  => p.level >= 3
```

The WHERE clause is the "if". The condition is the "then".

## 2.5 Uniqueness

"No two things can have the same value."

```
-- Unique emails
constraint unique_email:
  p1: Person, p2: Person
  WHERE p1.id != p2.id 
    AND p1.email != null 
    AND p2.email != null
  => p1.email != p2.email

-- Unique names within a project
constraint unique_task_name_per_project:
  t1: Task, t2: Task, p: Project,
  belongs_to(t1, p), belongs_to(t2, p)
  WHERE t1.id != t2.id
  => t1.name != t2.name
```

## 2.6 Range Validation

"Values must be within bounds."

```
constraint valid_priority:
  t: Task WHERE t.priority != null
  => t.priority >= 0 AND t.priority <= 10

constraint valid_percentage:
  p: Progress WHERE p.percent != null
  => p.percent >= 0 AND p.percent <= 100
```

## 2.7 Referential Integrity

"Relationships must be consistent."

```
-- Assignee must be team member
constraint assignee_in_team:
  t: Task, p: Person, team: Team, proj: Project,
  assigned_to(t, p),
  belongs_to(t, proj),
  owns(team, proj)
  => EXISTS(member_of(p, team))

-- Subtasks must be in same project as parent
constraint subtask_same_project:
  child: Task, parent: Task, p1: Project, p2: Project,
  subtask_of(child, parent),
  belongs_to(child, p1),
  belongs_to(parent, p2)
  => p1.id = p2.id
```

## 2.8 Mutual Exclusion

"These cannot both be true."

```
constraint not_admin_and_guest:
  u: User
  => NOT (u.is_admin = true AND u.is_guest = true)

constraint not_active_and_archived:
  p: Project
  => NOT (p.status = "active" AND p.archived = true)
```

---

# Part III: Hard vs Soft Constraints

## 3.1 The Distinction

| Type | On Violation | Use Case |
|------|--------------|----------|
| **Hard** (default) | Transaction rejected | Data integrity, business rules that must hold |
| **Soft** | Warning logged, operation proceeds | Recommendations, data quality hints |

## 3.2 Hard Constraints

Hard constraints are laws. Violations abort the transaction:

```
constraint valid_priority:
  t: Task => t.priority >= 0 AND t.priority <= 10

BEGIN
  SPAWN t: Task { priority = 15 }
COMMIT
-- ERROR: Constraint 'valid_priority' violated
-- Transaction rolled back. No task created.
```

## 3.3 Soft Constraints

Soft constraints are recommendations. Violations warn but proceed:

```
constraint prefer_description [soft, message: "Tasks should have descriptions"]:
  t: Task => t.description != null

BEGIN
  SPAWN t: Task { title = "Quick fix" }
COMMIT
-- WARNING: Constraint 'prefer_description' violated: Tasks should have descriptions
-- Transaction committed. Task created.
```

## 3.4 When to Use Each

| Use Hard When | Use Soft When |
|---------------|---------------|
| Data would be invalid | Data is valid but suboptimal |
| Business rule must hold | It's a recommendation |
| Downstream code assumes it | Humans should review |
| Security/integrity at stake | It's a quality metric |

## 3.5 Custom Messages

Both hard and soft constraints can have custom messages:

```
constraint temporal_order [message: "Cause must precede effect"]:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp

-- On violation:
-- ERROR: Constraint 'temporal_order' violated: Cause must precede effect
```

---

# Part IV: Attribute Modifiers Compile to Constraints

## 4.1 The Unification

Attribute modifiers like `[required]`, `[unique]`, `[>= N]` are **syntactic sugar** that compile to constraints. This means:

1. There's one validation mechanism (constraints)
2. Modifiers are convenient shortcuts
3. Complex validation uses explicit constraints
4. Everything is inspectable the same way

## 4.2 Modifier Expansions

### 4.2.1 Required

```
node Task {
  title: String [required]
}

-- Compiles to:
constraint _task_title_required:
  t: Task => t.title != null
```

### 4.2.2 Unique

```
node Person {
  email: String [unique]
}

-- Compiles to:
constraint _person_email_unique:
  p1: Person, p2: Person
  WHERE p1.id != p2.id AND p1.email != null AND p2.email != null
  => p1.email != p2.email
```

### 4.2.3 Range

```
node Task {
  priority: Int [>= 0, <= 10]
}

-- Compiles to:
constraint _task_priority_min:
  t: Task WHERE t.priority != null
  => t.priority >= 0

constraint _task_priority_max:
  t: Task WHERE t.priority != null
  => t.priority <= 10
```

### 4.2.4 Enum

```
node Task {
  status: String [in: ["todo", "in_progress", "done"]]
}

-- Compiles to:
constraint _task_status_enum:
  t: Task WHERE t.status != null
  => t.status = "todo" OR t.status = "in_progress" OR t.status = "done"
```

### 4.2.5 Length

```
node Person {
  name: String [length: 1..100]
}

-- Compiles to:
constraint _person_name_length:
  p: Person WHERE p.name != null
  => length(p.name) >= 1 AND length(p.name) <= 100
```

## 4.3 Combining Modifiers and Explicit Constraints

Use modifiers for simple cases, explicit constraints for complex ones:

```
node Task {
  title: String [required, length: 1..200],
  status: String [in: ["todo", "in_progress", "done"]] = "todo",
  priority: Int [0..10] = 5,
  completed_at: Timestamp?
}

-- Explicit constraint for cross-field validation
constraint completed_has_timestamp:
  t: Task WHERE t.status = "done"
  => t.completed_at != null
```

---

# Part V: Constraint-Rule Interaction

## 5.1 Execution Order

Within a transaction:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TRANSACTION EXECUTION                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. USER MUTATION                                                  │
│      SPAWN, KILL, LINK, UNLINK, SET                                │
│                                                                      │
│   2. RULE EXECUTION (repeat until quiescent)                        │
│      • Find rules whose patterns now match                         │
│      • Execute in priority order                                   │
│      • Rules may trigger other rules                               │
│                                                                      │
│   3. CONSTRAINT CHECKING                                            │
│      • Evaluate all affected constraints                           │
│      • Hard violation → ROLLBACK entire transaction                │
│      • Soft violation → WARN, continue                             │
│                                                                      │
│   4. COMMIT (only if all hard constraints pass)                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Rules Can Fix Violations

Because rules run before constraints are checked, rules can "fix" what would otherwise violate:

```
-- Constraint: tasks must have timestamps
constraint task_has_timestamp:
  t: Task => t.created_at != null

-- Rule: automatically add timestamp
rule auto_timestamp [priority: 100]:
  t: Task WHERE t.created_at = null
  => SET t.created_at = now()
```

**Execution flow:**

```
User: SPAWN t: Task { title = "Test" }

1. Task created without created_at (would violate constraint)
2. auto_timestamp rule fires, sets created_at = now()
3. Constraint checked: t.created_at != null → PASS
4. Commit
```

The user doesn't need to provide `created_at`. The rule ensures the constraint is satisfied.

## 5.3 The Pattern: Constraints Define, Rules Derive

```
┌─────────────────────────────────────────────────────────────────────┐
│                  CONSTRAINTS + RULES                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CONSTRAINTS define WHAT must be true.                             │
│   RULES define HOW to make it true.                                 │
│                                                                      │
│   Constraint: "Every task has a created_at"                         │
│   Rule: "If created_at is null, set it to now()"                   │
│                                                                      │
│   Constraint: "Completed tasks have completed_at"                   │
│   Rule: "When status becomes done, set completed_at"               │
│                                                                      │
│   Constraint: "Every project has an owner"                          │
│   Rule: "When project created, make creator the owner"             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.4 Constraints as Safety Net

Even with rules, constraints catch edge cases:

```
-- Rule handles the common case
rule auto_owner [priority: 100]:
  p: Project WHERE NOT EXISTS(owns(_, p))
  => LINK owns(current_actor(), p)

-- Constraint catches if rule somehow fails
constraint project_has_owner:
  p: Project
  => EXISTS(o: Person, owns(o, p))
```

If `current_actor()` is null or invalid, the rule might not fire correctly. The constraint ensures the invariant holds regardless.

---

# Part VI: Structural Constraints

## 6.1 Graph Structure Validation

Constraints can validate graph structure, not just attribute values:

```
-- No self-loops
constraint no_self_follow:
  p: Person, follows(p, p)
  => false

-- No orphaned tasks
constraint task_has_project:
  t: Task
  => EXISTS(p: Project, belongs_to(t, p))

-- Exactly one owner per task
constraint single_owner:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1) AS a1,
  assigned_to(t, p2) AS a2
  WHERE a1.role = "owner" AND a2.role = "owner" AND p1.id != p2.id
  => false
```

## 6.2 Transitive Constraints

Use `+` for transitive closure to detect cycles:

```
-- No dependency cycles
constraint no_dependency_cycle:
  t: Task, depends_on+(t, t)
  => false

-- No management cycles
constraint no_management_cycle:
  p: Person, manages+(p, p)
  => false

-- No circular subtasks
constraint no_subtask_cycle:
  t: Task, subtask_of+(t, t)
  => false
```

The pattern `depends_on+(t, t)` matches if there's any path from `t` back to itself via `depends_on` edges.

## 6.3 Depth-Limited Transitive

For performance, limit transitive depth:

```
constraint no_deep_cycle:
  p: Person, reports_to+(p, p) [depth: 50]
  => false
```

This checks up to 50 hops. Cycles deeper than that won't be caught (but are also rare/pathological).

## 6.4 Cross-Relationship Constraints

Validate consistency across multiple relationship types:

```
-- Task dependencies must be within same project
constraint deps_same_project:
  t1: Task, t2: Task, p1: Project, p2: Project,
  depends_on(t1, t2),
  belongs_to(t1, p1),
  belongs_to(t2, p2)
  => p1.id = p2.id

-- Can't assign task to non-team-member
constraint assignee_is_member:
  t: Task, person: Person, proj: Project, team: Team,
  assigned_to(t, person),
  belongs_to(t, proj),
  owns(team, proj)
  => EXISTS(member_of(person, team))
```

---

# Part VII: What Constraints Cannot Do

## 7.1 No `now()` in Conditions

Constraints must be **deterministic**. They're checked at commit time and must return the same result for the same graph state.

```
-- INVALID: now() is non-deterministic
constraint recent_only:
  t: Task
  => t.created_at > now() - 86400000
-- ERROR: now() cannot appear in constraint conditions
```

**Why:** If constraints used `now()`, a valid graph could become invalid just by time passing, without any mutation. This breaks the invariant model.

**Workarounds:**

1. Store a reference timestamp:
```
node Task {
  created_at: Timestamp,
  expires_at: Timestamp
}

constraint not_expired:
  t: Task WHERE t.expires_at != null
  => t.completed_at != null OR t.expires_at > t.created_at
-- Compare timestamps to each other, not to now()
```

2. Use rules for time-based actions:
```
rule expire_old_tasks [manual]:
  t: Task WHERE t.status != "done" AND t.created_at < $cutoff
  => SET t.status = "expired"
-- Triggered by scheduler with current time as parameter
```

## 7.2 No Side Effects

Constraint conditions are pure expressions. They cannot:
- Modify the graph
- Call external services
- Generate random values
- Write to logs (except through violation reporting)

## 7.3 No Cross-Transaction State

Constraints see only the current transaction's view of the graph. They cannot:
- Reference previous values ("was X before")
- Track history ("has ever been Y")
- Count changes ("changed more than N times")

For these, use rules that maintain explicit audit data.

## 7.4 Performance Considerations

Constraints are checked on every relevant mutation. Expensive patterns should be used carefully:

```
-- Potentially expensive: transitive closure on large graph
constraint no_cycle:
  n: Node, edge+(n, n)
  => false

-- Better: depth-limited
constraint no_cycle:
  n: Node, edge+(n, n) [depth: 100]
  => false

-- Or: mark edges that could form cycles, check only those
constraint no_cycle:
  n: Node, critical_edge+(n, n)
  => false
```

---

# Part VIII: Integration with Other Systems

## 8.1 Constraints and Policy

Constraints validate **data**. Policy validates **access**. They're orthogonal:

```
-- Constraint: data validity
constraint valid_priority:
  t: Task => t.priority >= 0 AND t.priority <= 10

-- Policy: who can set priority
policy only_lead_sets_priority:
  ON SET(t: Task, "priority")
  ALLOW IF has_role(current_actor(), "lead")
```

A mutation must pass **both**:
1. Policy check (are you allowed to do this?)
2. Constraint check (is the result valid?)

## 8.2 Constraints and Transactions

Constraints are checked at **transaction commit**, not at each statement:

```
BEGIN
  SET t.status = "done"        -- Would violate completed_has_timestamp
  SET t.completed_at = now()   -- Fixes the violation
COMMIT
-- SUCCESS: Constraint checked after both statements
```

This allows multi-step operations that go through invalid intermediate states.

## 8.3 Constraints in Worlds (Interiority)

Each world interior can have its own constraints:

```
node Agent [has_interior] {
  interior: ontology {
    node Belief { confidence: Float }
    
    -- Local constraint
    constraint valid_confidence:
      b: Belief => b.confidence >= 0 AND b.confidence <= 1
  }
}
```

Interior constraints are checked only on interior mutations. They don't affect ROOT or other interiors.

With `inherit_constraints: true`, parent constraints also apply:

```
node StrictAgent [has_interior] {
  interior: ontology [inherit_constraints: true] {
    -- Parent constraints apply here too
  }
}
```

## 8.4 META Constraints

Constraints can apply to the schema itself:

```
-- No more than 50 types per ontology
constraint type_limit:
  o: _Ontology, t: _NodeType, _ontology_declares_type(o, t)
  => COUNT(t2: _NodeType, _ontology_declares_type(o, t2)) <= 50

-- All types must have documentation
constraint types_documented [soft]:
  t: _NodeType
  => t.doc != null
```

---

# Part IX: Common Patterns

## 9.1 Complete Example

```
ontology TaskManagement {

  -- Node types with modifier-based constraints
  node Person {
    name: String [required, length: 1..100],
    email: String [required, unique, format: email]
  }

  node Team {
    name: String [required, unique]
  }

  node Project {
    name: String [required],
    status: String [in: ["planning", "active", "completed"]] = "planning",
    deadline: Timestamp?
  }

  node Task {
    title: String [required, length: 1..200],
    status: String [in: ["todo", "in_progress", "blocked", "done"]] = "todo",
    priority: Int [0..10] = 5,
    created_at: Timestamp [required],
    completed_at: Timestamp?
  }

  -- Edge types
  edge member_of(person: Person, team: Team) {
    role: String [in: ["member", "lead"]] = "member",
    joined_at: Timestamp = now()
  }
  
  edge owns(team: Team, project: Project)
  edge belongs_to(task: Task, project: Project)
  edge assigned_to(task: Task, person: Person)
  edge depends_on(downstream: Task, upstream: Task)
  edge subtask_of(child: Task, parent: Task)

  -- Structural constraints
  constraint no_self_dependency:
    t: Task, depends_on(t, t)
    => false

  constraint no_dependency_cycle:
    t: Task, depends_on+(t, t)
    => false

  constraint no_subtask_cycle:
    t: Task, subtask_of+(t, t)
    => false

  -- Referential integrity
  constraint task_has_project [message: "Every task must belong to a project"]:
    t: Task
    => EXISTS(p: Project, belongs_to(t, p))

  constraint single_project [message: "Task cannot belong to multiple projects"]:
    t: Task, p1: Project, p2: Project,
    belongs_to(t, p1), belongs_to(t, p2)
    WHERE p1.id != p2.id
    => false

  constraint subtask_same_project [message: "Subtask must be in same project as parent"]:
    child: Task, parent: Task, p1: Project, p2: Project,
    subtask_of(child, parent),
    belongs_to(child, p1),
    belongs_to(parent, p2)
    => p1.id = p2.id

  constraint assignee_is_team_member [message: "Can only assign to team members"]:
    task: Task, person: Person, proj: Project, team: Team,
    assigned_to(task, person),
    belongs_to(task, proj),
    owns(team, proj)
    => EXISTS(member_of(person, team))

  -- State machine constraints
  constraint completed_has_timestamp [message: "Completed tasks must have completion time"]:
    t: Task WHERE t.status = "done"
    => t.completed_at != null

  constraint blocked_has_blocker [message: "Blocked tasks must have pending dependency"]:
    t: Task WHERE t.status = "blocked"
    => EXISTS(
      upstream: Task, 
      depends_on(t, upstream) 
      WHERE upstream.status != "done"
    )

  -- Soft constraints (recommendations)
  constraint high_priority_assigned [soft, message: "High priority tasks should be assigned"]:
    t: Task WHERE t.priority >= 8
    => EXISTS(p: Person, assigned_to(t, p))

  constraint tasks_have_descriptions [soft, message: "Tasks should have descriptions"]:
    t: Task
    => t.description != null AND length(t.description) >= 10

  constraint deadline_reasonable [soft, message: "Deadline should be in the future"]:
    p: Project WHERE p.deadline != null AND p.status = "planning"
    => p.deadline > p.created_at
}
```

## 9.2 Pattern Catalog

| Pattern | Structure | Use Case |
|---------|-----------|----------|
| Prohibition | `pattern => false` | Prevent invalid structure |
| Requirement | `entity => EXISTS(...)` | Ensure relationships exist |
| Implication | `entity WHERE cond => result` | Conditional requirements |
| Uniqueness | `e1, e2 WHERE e1 != e2 => e1.x != e2.x` | Unique values |
| Range | `entity WHERE x != null => x >= a AND x <= b` | Value bounds |
| Mutual exclusion | `entity => NOT (a AND b)` | Prevent combinations |
| Referential | `multiple edges => consistency check` | Cross-relationship rules |
| Acyclicity | `edge+(e, e) => false` | Prevent cycles |

---

# Part X: Summary

## 10.1 What Constraints Provide

| Capability | Benefit |
|------------|---------|
| Declarative validation | State what's valid, not how to check |
| Kernel enforcement | Cannot be bypassed |
| Unified mechanism | Modifiers compile to constraints |
| Introspectable | Query and analyze constraints |
| Hard + soft modes | Mandatory vs advisory |
| Structural validation | Graph topology, not just values |

## 10.2 Key Principles

| Principle | Meaning |
|-----------|---------|
| **Universal form** | pattern => condition, for all matches |
| **Checked at commit** | Intermediate states can be invalid |
| **Rules run first** | Rules can fix potential violations |
| **Deterministic** | No `now()`, no side effects |
| **Additive** | More constraints = more restricted |

## 10.3 The Mental Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CONSTRAINTS                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Constraints are LAWS of the graph universe.                       │
│                                                                      │
│   They don't check data — they define what data CAN exist.          │
│                                                                      │
│   A graph state either satisfies all constraints or it              │
│   cannot exist. There is no "invalid but present" data.             │
│                                                                      │
│   Rules can derive. Constraints validate.                           │
│   Together they make the graph self-maintaining AND self-valid.     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of MEW Constraints Capability*
