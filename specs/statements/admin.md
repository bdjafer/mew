---
spec: admin
version: "1.0"
status: draft
category: statement
capability: administration
requires: []
priority: essential
---

# Spec: Administration Statements

## Overview

Administration statements manage the schema, indexes, and engine state. They include loading and extending ontologies, inspecting schema metadata with SHOW statements, and creating/dropping indexes for query optimization. These operations are essential for system setup, maintenance, and introspection.

## Syntax

### Grammar

```ebnf
AdminStmt        = LoadStmt | ExtendStmt | ShowStmt | CreateIndexStmt | DropIndexStmt

LoadStmt         = "load" "ontology" ("from" StringLiteral | "{" OntologySource "}")

ExtendStmt       = "extend" "ontology" Identifier? "{" Declaration* "}"

ShowStmt         = "show" ShowTarget
ShowTarget       = "types" | "edges" | "constraints" | "rules" | "indexes"
                 | "statistics" | "status" | "branches" | "versions"
                 | "type" Identifier | "edge" Identifier
                 | "constraint" Identifier | "rule" Identifier

CreateIndexStmt  = "create" "index" Identifier? "on" IndexTarget
IndexTarget      = TypeExpr "(" Identifier ("asc" | "desc")? ")" | Identifier

DropIndexStmt    = "drop" "index" Identifier
```

### Keywords

| Keyword | Context |
|---------|---------|
| `load` | Statement - loads an ontology |
| `ontology` | Modifier - specifies ontology context |
| `from` | Clause - specifies file path |
| `extend` | Statement - adds to existing ontology |
| `show` | Statement - displays schema information |
| `create index` | Statement - creates an index |
| `drop index` | Statement - removes an index |
| `on` | Clause - specifies index target |

### Examples

```
-- Load ontology from file
LOAD ONTOLOGY FROM "path/to/ontology.mew"

-- Load inline ontology
LOAD ONTOLOGY {
  node Event { timestamp: Timestamp }
  edge causes(from: Event, to: Event)
}

-- Extend existing ontology
EXTEND ONTOLOGY {
  node Priority { level: Int, name: String }
}

-- Show all types
SHOW TYPES

-- Show specific type details
SHOW TYPE Task

-- Create index
CREATE INDEX task_priority ON Task(priority)

-- Drop index
DROP INDEX task_priority
```

## Semantics

### LOAD ONTOLOGY

Loads an ontology from a file or inline definition:

**From file:**
```
LOAD ONTOLOGY FROM "path/to/ontology.mew"
```

**Inline:**
```
LOAD ONTOLOGY {
  node Event {
    timestamp: Timestamp
  }
  edge causes(from: Event, to: Event)
}
```

**Load behavior:**

| Scenario | Behavior |
|----------|----------|
| First load | Install ontology |
| Same ontology | No-op (idempotent) |
| Different with same name | Error (use EXTEND) |
| Inheritance from unloaded | Error |

### EXTEND ONTOLOGY

Adds new definitions to an existing ontology without replacing it:

```
EXTEND ONTOLOGY {
  node NewType {
    field: String
  }
}

-- Extend named ontology
EXTEND ONTOLOGY TaskManagement {
  node Priority {
    level: Int,
    name: String
  }
}
```

**Extension rules:**
- Can add new types, edges, constraints, rules
- Cannot modify existing definitions
- Cannot remove existing definitions
- New types can inherit from existing types

```
EXTEND ONTOLOGY {
  -- Add new type inheriting from existing
  node SpecialTask : Task {
    special_field: String
  }

  -- Add new edge using existing types
  edge reviewed_by(task: Task, person: Person)
}
```

### SHOW Statements

Inspect schema and engine state:

**SHOW TYPES:**
```
SHOW TYPES

-- Result:
| Name    | Attributes | Parent | Modifiers |
|---------|------------|--------|-----------|
| Task    | 8          | null   |           |
| Person  | 5          | null   |           |
| Project | 4          | null   |           |
```

**SHOW TYPE \<name\>:**
```
SHOW TYPE Task

-- Result:
Type: Task
Parents: []
Attributes:
  title: String [required, length: 1..500]
  description: String?
  status: String [in: ["todo", "in_progress", "done"]] = "todo"
  priority: Int [0..10] = 5
  created_at: Timestamp [required, indexed: desc]
  completed_at: Timestamp?
```

**SHOW EDGES:**
```
SHOW EDGES

-- Result:
| Name        | Signature                  | Modifiers          |
|-------------|----------------------------|--------------------|
| causes      | (Event, Event)             |                    |
| assigned_to | (Task, Person)             | task -> 0..1       |
| belongs_to  | (Task, Project)            | task -> 1, cascade |
```

**SHOW CONSTRAINTS:**
```
SHOW CONSTRAINTS

-- Result:
| Name                    | Type | Pattern              | Message                    |
|-------------------------|------|----------------------|----------------------------|
| task_title_required     | hard | t: Task              | Title required             |
| temporal_order          | hard | causes(e1, e2)       | Cause must precede effect  |
| prefer_description      | soft | t: Task              | Tasks should have desc     |
```

**SHOW RULES:**
```
SHOW RULES

-- Result:
| Name                   | Priority | Auto | Pattern        |
|------------------------|----------|------|----------------|
| auto_complete_timestamp| 10       | yes  | t: Task        |
| propagate_completion   | 5        | yes  | subtask_of(...)|
```

**SHOW INDEXES:**
```
SHOW INDEXES

-- Result:
| Name               | Type      | Target           | Order |
|--------------------|-----------|------------------|-------|
| task_created_at    | attribute | Task.created_at  | desc  |
| person_email       | attribute | Person.email     | asc   |
| assigned_to_idx    | edge      | assigned_to      | -     |
```

**SHOW STATISTICS:**
```
SHOW STATISTICS

-- Result:
Nodes:
  Total: 15,432
  By type:
    Task: 8,234
    Person: 1,543
    Project: 234
    ...

Edges:
  Total: 45,678
  By type:
    assigned_to: 7,234
    belongs_to: 8,234
    causes: 12,456
    ...

Storage:
  Size: 234 MB
  Index size: 45 MB
```

**SHOW STATUS:**
```
SHOW STATUS

-- Result:
Engine: running
Uptime: 4 days, 3 hours
Ontologies loaded: 3
Active transactions: 2
Pending rules: 0
Last snapshot: 2024-01-15T10:30:00Z
```

### INDEX Management

**CREATE INDEX:**

```
-- Attribute index
CREATE INDEX task_priority ON Task(priority)

-- With sort order
CREATE INDEX task_created ON Task(created_at DESC)

-- Edge index
CREATE INDEX assigned_idx ON assigned_to

-- Auto-named (generates name from target)
CREATE INDEX ON Person(email)
-- Creates index named "person_email_idx"
```

**DROP INDEX:**

```
DROP INDEX task_priority
```

**Index vs [indexed] Modifier:**

Both achieve the same result:

```
-- In ontology (declarative):
node Task {
  created_at: Timestamp [indexed: desc]
}

-- At runtime (imperative):
CREATE INDEX task_created ON Task(created_at DESC)
```

Ontology `[indexed]` is preferred for permanent indexes. Runtime CREATE INDEX is useful for experimentation or temporary indexes.

## Layer 0

None. Administration statements operate on metadata and engine state, not the graph data itself.

## Examples

### Initial System Setup

```
-- Load base ontology
LOAD ONTOLOGY FROM "schemas/core.mew"

-- Extend with application-specific types
EXTEND ONTOLOGY {
  node CustomReport : Report {
    template: String
  }

  edge generated_from(report: CustomReport, data: DataSet)
}

-- Create performance indexes
CREATE INDEX report_created ON CustomReport(created_at DESC)
CREATE INDEX dataset_name ON DataSet(name)
```

### Schema Introspection

```
-- List all types
SHOW TYPES

-- Examine specific type structure
SHOW TYPE Task

-- Check what indexes exist
SHOW INDEXES

-- View current statistics
SHOW STATISTICS
```

### Index Optimization Workflow

```
-- Check query performance
PROFILE MATCH t: Task WHERE t.status = "done" RETURN t
-- Note: Full scan on status

-- Add index
CREATE INDEX task_status ON Task(status)

-- Verify improvement
PROFILE MATCH t: Task WHERE t.status = "done" RETURN t
-- Note: Now uses task_status index

-- Remove if not beneficial
DROP INDEX task_status
```

## Errors

| Condition | Message |
|-----------|---------|
| Ontology file not found | Cannot load ontology: file 'path' not found |
| Duplicate type definition | Type 'Name' already exists (use EXTEND to add fields) |
| Modify existing in EXTEND | Cannot modify existing type 'Name' in EXTEND |
| Unknown type in SHOW | Type 'Name' not found |
| Unknown edge in SHOW | Edge 'Name' not found |
| Duplicate index name | Index 'name' already exists |
| Unknown index in DROP | Index 'name' does not exist |
| Invalid index target | Cannot create index on 'target': type/attribute not found |
