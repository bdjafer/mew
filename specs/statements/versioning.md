---
spec: versioning
version: "1.0"
status: draft
category: statement
requires: [transactions]
priority: specialized
---

# Spec: Versioning

## Overview

Versioning provides time-travel queries, snapshots, branching, and merging. Enables auditing, experimentation, and recovery without external backup systems.

**Why needed:** Complex applications need history: "What did the graph look like yesterday?", "Let me experiment without affecting production", "Undo that batch of changes".

---

## Syntax

### Grammar
```ebnf
VersionStatement = 
    SnapshotStatement
  | CheckoutStatement
  | DiffStatement
  | BranchStatement
  | SwitchStatement
  | MergeStatement
  | VersionsStatement

SnapshotStatement = "SNAPSHOT" (StringLiteral)?

CheckoutStatement = "CHECKOUT" VersionRef

DiffStatement = "DIFF" VersionRef VersionRef

BranchStatement = "BRANCH" Identifier ("FROM" VersionRef)?

SwitchStatement = "SWITCH" Identifier

MergeStatement = "MERGE" Identifier

VersionsStatement = "VERSIONS" ("LIMIT" IntLiteral)?

VersionRef = Identifier | StringLiteral | "HEAD" | "HEAD~" IntLiteral
```

### Keywords

| Keyword | Context |
|---------|---------|
| `SNAPSHOT` | Statement |
| `CHECKOUT` | Statement |
| `DIFF` | Statement |
| `BRANCH` | Statement |
| `SWITCH` | Statement |
| `MERGE` | Statement |
| `VERSIONS` | Statement |
| `HEAD` | Version reference |

### Examples
```
-- Create snapshot
SNAPSHOT "before-migration"

-- List versions
VERSIONS LIMIT 10

-- Time-travel query
CHECKOUT "before-migration"
MATCH t: Task RETURN t
CHECKOUT HEAD

-- Branching
BRANCH experiment
-- ... make changes ...
SWITCH main
MERGE experiment

-- View differences
DIFF HEAD~1 HEAD
```

---

## Semantics

### Snapshots

`SNAPSHOT` creates an immutable point-in-time copy:
```
SNAPSHOT                    -- auto-generated name (timestamp)
SNAPSHOT "v1.0"            -- named snapshot
SNAPSHOT "pre-migration"   -- descriptive name
```

Snapshots are lightweight (copy-on-write or delta-based).

### Version References

| Syntax | Meaning |
|--------|---------|
| `HEAD` | Current version |
| `HEAD~1` | One version before HEAD |
| `HEAD~N` | N versions before HEAD |
| `"name"` | Named snapshot |
| `branch_name` | Head of branch |

### Checkout

`CHECKOUT` loads a historical version for reading:
```
CHECKOUT "v1.0"
-- Graph is now read-only at v1.0 state
MATCH t: Task RETURN t
-- Returns tasks as they were at v1.0

CHECKOUT HEAD
-- Return to current state, read-write
```

Checkout is session-scoped. Other sessions see current state.

### Read-Only Mode

After checkout to a historical version:
- MATCH, WALK, queries work normally
- SPAWN, KILL, LINK, UNLINK, SET are rejected
- Must CHECKOUT HEAD to mutate
```
CHECKOUT "old-version"
SPAWN t: Task { title = "New" }
-- ERROR: Cannot mutate historical version. CHECKOUT HEAD first.
```

### Diff

`DIFF` shows changes between versions:
```
DIFF HEAD~1 HEAD
-- Returns:
-- + Task#123 created
-- - Task#456 deleted
-- ~ Task#789.status: "todo" -> "done"
-- + causes(Event#1, Event#2) linked
```

Result format:
```
{
  nodes_created: [...],
  nodes_deleted: [...],
  nodes_modified: [{id, attribute, old_value, new_value}, ...],
  edges_created: [...],
  edges_deleted: [...]
}
```

### Branching

Branches enable parallel lines of development:
```
BRANCH experiment           -- Create from current HEAD
BRANCH experiment FROM "v1.0"  -- Create from snapshot

-- Work on branch
SWITCH experiment
SPAWN t: Task { title = "Experimental" }

-- Return to main
SWITCH main
-- experimental task not visible here
```

### Merging

`MERGE` integrates branch changes:
```
SWITCH main
MERGE experiment
```

Merge strategies:
- **Fast-forward:** If main hasn't changed, just move pointer
- **Three-way:** Find common ancestor, apply both change sets
- **Conflict:** If same entity modified differently, report conflict

### Conflict Resolution

Conflicts must be resolved manually:
```
MERGE experiment
-- ERROR: Conflict on Task#123.status
--   main: "done"
--   experiment: "blocked"
-- Use MERGE experiment RESOLVE THEIRS|OURS|MANUAL
```

Resolution options:
```
MERGE experiment RESOLVE OURS    -- keep main's changes
MERGE experiment RESOLVE THEIRS  -- keep experiment's changes
```

### Versions List
```
VERSIONS
-- Returns list of snapshots and branch heads

VERSIONS LIMIT 10
-- Last 10 versions
```

Result:
```
| version_id | name | type | created_at | parent |
|------------|------|------|------------|--------|
| v-001 | HEAD | branch:main | 2024-01-15T10:00:00Z | v-000 |
| v-000 | "initial" | snapshot | 2024-01-15T09:00:00Z | null |
```

---

## Layer 0

None. Versioning is a runtime capability, not an ontology construct.

---

## Examples

### Audit Trail
```
-- Before batch operation
SNAPSHOT "pre-import-2024-01-15"

-- Import data
BEGIN
  SPAWN ...
  SPAWN ...
COMMIT

-- If something went wrong, compare
DIFF "pre-import-2024-01-15" HEAD

-- To see what existed before
CHECKOUT "pre-import-2024-01-15"
MATCH t: Task RETURN t
CHECKOUT HEAD
```

### Feature Development
```
-- Start feature branch
BRANCH feature-new-workflow

-- Develop
SWITCH feature-new-workflow
SPAWN w: Workflow { name = "New Process" }
-- ... more changes ...

-- Test in isolation
MATCH w: Workflow RETURN w

-- Ready to ship
SWITCH main
MERGE feature-new-workflow
```

### Experimentation
```
-- Snapshot current state
SNAPSHOT "baseline"

-- Try something risky
BEGIN
  -- bulk delete
  MATCH t: Task WHERE t.status = "archived"
  -- (hypothetical bulk delete syntax)
COMMIT

-- Check results
MATCH t: Task RETURN count(t)

-- Oops, restore
CHECKOUT "baseline"
-- But this is read-only...

-- Actually need to restore:
-- (Would need RESTORE command, out of scope)
```

### Time-Travel Queries
```
-- What tasks existed last week?
CHECKOUT HEAD~7  -- assuming daily snapshots
MATCH t: Task WHERE t.status = "in_progress"
RETURN t
CHECKOUT HEAD

-- Compare to now
MATCH t: Task WHERE t.status = "in_progress"
RETURN t
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Unknown version | `"Version 'X' not found"` |
| Mutate historical | `"Cannot mutate historical version"` |
| Merge conflict | `"Conflict on <entity>.<attr>: <details>"` |
| Branch exists | `"Branch 'X' already exists"` |
| Cannot delete main | `"Cannot delete main branch"` |

---

*End of Spec: Versioning*