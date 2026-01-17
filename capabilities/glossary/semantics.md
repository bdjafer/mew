# MEW Language Semantics

---

## Terminology Decisions

### Observation / Transformation (not Query / Mutation)

MEW uses **Observation** and **Transformation** instead of the conventional Query/Mutation terminology.

| Convention | MEW | Rationale |
|------------|-----|-----------|
| Query | **Observation** | Reads don't interfere with state. An observation examines without changing. |
| Mutation | **Transformation** | Writes map one world-state to another. A transformation is structured change. |

This applies throughout: DSL documentation, component names, internal implementation.

The terms are philosophically aligned with MEW's foundations:
- Observers examine systems across Markov blankets
- Transformations are causal interventions that produce new states

Users write specific verbs (MATCH, SPAWN, etc.), not "observe" or "transform" directly. The categories are conceptual frames that organize the operations.

---

### Constraint / Invariant (both, at different levels)

| Term | Level | Meaning |
|------|-------|---------|
| **Constraint** | User-facing | A declared rule that restricts valid states |
| **Invariant** | System-level | A property guaranteed to always hold |

Users *declare* constraints in the ontology:
```
constraint task_has_owner [hard, deferred] {
  MATCH t: Task WHERE NOT EXISTS owner(_, t)
  VIOLATES "Task must have an owner"
}
```

The system *maintains* invariants by enforcing those constraints. If a constraint is declared and enabled, the corresponding invariant holds.

The component is named **Constraint** (it checks declared constraints). The specs describe **invariants** (properties the system guarantees).

---

## Observation Operations

Non-intrusive examination of the graph.

| Keyword | Type | Description |
|---------|------|-------------|
| `MATCH` | Pattern matching | Find all subgraphs matching a pattern |
| `WALK` | Path traversal | Follow edges from a starting point |
| `INSPECT` | Direct access | Retrieve a specific node/edge by ID |

```
-- Pattern matching: declarative, find all instances
MATCH e1: Event, e2: Event, causes(e1, e2)
WHERE e1.timestamp < e2.timestamp
RETURN e1, e2

-- Path traversal: navigational, follow structure
WALK FROM event_123 FOLLOW causes* RETURN REACHED

-- Direct access: lookup by identity
INSPECT node_456
```

---

## Transformation Operations

Structured changes to the graph.

Nodes and edges have distinct operations (ontologically different things):

| Keyword | Target | Description |
|---------|--------|-------------|
| `SPAWN` | Node | Bring a node into existence |
| `KILL` | Node | Remove a node from existence |
| `LINK` | Edge | Create a relation |
| `UNLINK` | Edge | Remove a relation |
| `SET` | Attribute | Modify an attribute (works on nodes and edges) |

```
-- Node operations
SPAWN e: Event { timestamp = 100 }
KILL e

-- Edge operations  
LINK causes(e1, e2) { mechanism = "direct" }
UNLINK c

-- Attribute operations (unified)
SET e.timestamp = 200
SET c.confidence = 0.9
```

---

## Design Principles

| Principle | Decision |
|-----------|----------|
| **Philosophical alignment** | Terminology reflects the worldview (observation, transformation, constraint, invariant) |
| **No umbrella keywords** | Users write specific verbs, not generic "observe" or "transform" |
| **Distinct node/edge operations** | SPAWN/KILL for nodes, LINK/UNLINK for edges |
| **Graph-native vocabulary** | Not SQL, but graph transformations |
| **Semantic clarity** | Keyword reveals what you're operating on |

---

## Vocabulary Etymology

| Term | Why This Word |
|------|---------------|
| **MATCH** | Pattern matching — established, precise |
| **WALK** | Physical movement through structure — approachable |
| **INSPECT** | Direct examination — universal |
| **SPAWN** | Entity emerging into existence — evocative |
| **KILL** | Entity leaving existence — symmetric with SPAWN |
| **LINK** | Relation forming — graph-native |
| **UNLINK** | Relation dissolving — symmetric with LINK |
| **SET** | Assign value — universal, clear |

---

## At a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                      OBSERVATION                            │
│                   (examining the graph)                     │
│                                                             │
│   MATCH pattern WHERE condition RETURN projection           │
│   WALK FROM start FOLLOW edges RETURN reached               │
│   INSPECT id                                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     TRANSFORMATION                          │
│                   (changing the graph)                      │
│                                                             │
│   SPAWN x: Type { attr = value }      -- create node        │
│   KILL x                              -- remove node        │
│   LINK edge(a, b) { attr = value }    -- create relation    │
│   UNLINK e                            -- remove relation    │
│   SET x.attr = value                  -- modify attribute   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      CONSTRAINT                             │
│              (declared rules → system invariants)           │
│                                                             │
│   constraint name [hard|soft, immediate|deferred] {         │
│     MATCH pattern WHERE condition                           │
│     VIOLATES "message"                                      │
│   }                                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Summary

| Concept | User Writes | System Guarantees |
|---------|-------------|-------------------|
| Read | MATCH, WALK, INSPECT | Observation doesn't change state |
| Write | SPAWN, KILL, LINK, UNLINK, SET | Transformation is atomic |
| Restriction | `constraint` declaration | Invariant holds if constraint enabled |