---
spec: default_values
version: "1.0"
status: draft
category: modifier
capability: default_values
requires: []
priority: essential
---

# Spec: Default Values

## Overview

Default values provide automatic initialization for attributes when they are not explicitly provided at SPAWN or LINK time. They support both static literal values and dynamic expressions evaluated at entity creation time.

**Why needed:** Many attributes have sensible defaults (status = "pending", count = 0, created_at = now()). Default values reduce boilerplate in SPAWN statements while ensuring entities are always in a valid state.

---

## Syntax

### Grammar
```ebnf
AttributeDecl = Identifier ":" TypeExpr AttributeModifiers? DefaultValue? ","?

DefaultValue = "=" (Literal | ConstantExpr)

ConstantExpr =
    "now()"
  | DurationLiteral
  | ConstantExpr ("+" | "-" | "*" | "/") ConstantExpr

DurationLiteral = IntLiteral "." DurationUnit

DurationUnit = "days" | "hours" | "minutes" | "seconds" | "milliseconds"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `=` | Default value assignment |
| `now()` | Current timestamp |
| `days`, `hours`, `minutes`, `seconds`, `milliseconds` | Duration units |

### Examples
```
node Task {
  status: String = "pending",
  priority: Int = 5,
  created_at: Timestamp = now(),
  due_date: Timestamp = now() + 7.days
}

node Token {
  expires_at: Timestamp = now() + 24.hours,
  refresh_count: Int = 0
}
```

---

## Semantics

### Static Defaults

Static defaults are literal values:
```
node Task {
  status: String = "pending",
  priority: Int = 0,
  active: Bool = true,
  tags: String = ""
}
```

### Dynamic Defaults

Dynamic defaults use `now()` and duration arithmetic:

| Expression | Type | Description |
|------------|------|-------------|
| `now()` | Timestamp | Current time at SPAWN/LINK |
| `7.days` | Duration | 7 days duration |
| `24.hours` | Duration | 24 hours duration |
| `30.minutes` | Duration | 30 minutes duration |
| `60.seconds` | Duration | 60 seconds duration |
| `1000.milliseconds` | Duration | 1000 milliseconds duration |
| `now() + 7.days` | Timestamp | 7 days from now |
| `now() - 24.hours` | Timestamp | 24 hours ago |

```
node Token {
  created_at: Timestamp = now(),
  expires_at: Timestamp = now() + 24.hours,
  refresh_at: Timestamp = now() + 1.hours
}

node Reminder {
  remind_at: Timestamp = now() + 30.minutes
}
```

### Arithmetic in Defaults

Constant expressions support arithmetic:
```
node Config {
  timeout_ms: Int = 60 * 1000,           -- 60000
  max_retries: Int = 3 * 2,              -- 6
  default_score: Float = 10.0 / 2.0      -- 5.0
}
```

### Evaluation Timing

Default values are evaluated at entity creation time:
```
SPAWN t1: Task { title = "First" }
-- t1.created_at = 2024-01-15T10:00:00Z

-- (5 seconds pass)

SPAWN t2: Task { title = "Second" }
-- t2.created_at = 2024-01-15T10:00:05Z (different!)
```

### Within Transactions

Within a single transaction, `now()` returns the same value:
```
BEGIN
  SPAWN t1: Task { title = "First" }
  SPAWN t2: Task { title = "Second" }
COMMIT
-- t1.created_at = t2.created_at (same transaction start time)
```

### Required with Default

An attribute can have both `[required]` and a default. The default satisfies the requirement:
```
node Task {
  status: String [required] = "pending"
}

SPAWN t: Task { title = "Test" }
-- t.status = "pending" (default satisfies required)
```

### Explicit Override

Providing an explicit value overrides the default:
```
node Task {
  status: String = "pending",
  priority: Int = 5
}

SPAWN t: Task { title = "Urgent", status = "in_progress", priority = 10 }
-- t.status = "in_progress" (explicit)
-- t.priority = 10 (explicit)
```

### Disallowed Defaults

The following are not allowed as default values:

```
-- Attribute references: ERROR
ref: String = other.name

-- Non-deterministic functions: ERROR
random_id: Int = random()

-- Queries/aggregations: ERROR
computed: Int = count(...)

-- Other entity's attributes: ERROR
copied: String = parent.name
```

For computed defaults based on other attributes, use rules:
```
node Event {
  base_value: Int,
  computed_field: Int?
}

rule compute_field [priority: 100]:
  e: Event WHERE e.computed_field = null
  =>
  SET e.computed_field = e.base_value * 2
```

---

## Layer 0

None. Default values are applied at SPAWN/LINK time by the engine. They do not generate constraints.

---

## Examples

### Task Management
```
ontology Tasks {
  node Task {
    title: String [required],
    description: String?,
    status: String [in: ["todo", "in_progress", "done"]] = "todo",
    priority: Int [0..10] = 5,
    created_at: Timestamp = now(),
    updated_at: Timestamp = now()
  }

  rule update_timestamp [priority: 100]:
    t: Task
    =>
    SET t.updated_at = now()
}

-- Minimal SPAWN uses all defaults
SPAWN t: Task { title = "Buy groceries" }
-- t.status = "todo"
-- t.priority = 5
-- t.created_at = now()
-- t.updated_at = now()
```

### Authentication Tokens
```
ontology Auth {
  node AccessToken {
    token: String [required, unique],
    user_id: String [required],
    created_at: Timestamp = now(),
    expires_at: Timestamp = now() + 1.hours,
    refresh_count: Int = 0,
    active: Bool = true
  }

  node RefreshToken {
    token: String [required, unique],
    user_id: String [required],
    created_at: Timestamp = now(),
    expires_at: Timestamp = now() + 30.days
  }
}

SPAWN t: AccessToken { token = "abc123", user_id = "user1" }
-- t.created_at = now()
-- t.expires_at = now() + 1 hour
-- t.refresh_count = 0
-- t.active = true
```

### Edge Attributes with Defaults
```
ontology Relationships {
  edge assigned_to(task: Task, person: Person) {
    assigned_at: Timestamp = now(),
    role: String [in: ["owner", "reviewer"]] = "owner",
    notified: Bool = false
  }
}

LINK assigned_to(myTask, alice)
-- assigned_at = now()
-- role = "owner"
-- notified = false

LINK assigned_to(myTask, bob) { role = "reviewer" }
-- assigned_at = now()
-- role = "reviewer" (explicit)
-- notified = false
```

### Scheduled Events
```
ontology Calendar {
  node Event {
    title: String [required],
    start_time: Timestamp [required],
    end_time: Timestamp?,
    created_at: Timestamp = now(),
    reminder_at: Timestamp?
  }

  node Reminder {
    event_id: String [required],
    remind_at: Timestamp = now() + 15.minutes,
    sent: Bool = false
  }
}
```

### Configuration with Computed Defaults
```
ontology Config {
  node Settings {
    -- Static defaults
    theme: String = "light",
    language: String = "en",

    -- Arithmetic defaults
    timeout_ms: Int = 30 * 1000,
    max_connections: Int = 10 * 5,

    -- Timestamp defaults
    created_at: Timestamp = now(),
    last_modified: Timestamp = now()
  }
}
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Invalid expression | `"Default value must be a literal or constant expression"` |
| Attribute reference | `"Attribute references not allowed in default values"` |
| Non-pure function | `"Function '<name>' not allowed in default values"` |
| Type mismatch | `"Default value type '<got>' does not match attribute type '<expected>'"` |
| Invalid duration | `"Invalid duration unit '<unit>'"` |

---

*End of Spec: Default Values*
