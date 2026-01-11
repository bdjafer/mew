# HOHG Language Semantics

---

## Conceptual Categories

| Category | Purpose | Nature |
|----------|---------|--------|
| **Observation** | Reading the graph without changing it | Non-intrusive examination |
| **Transformation** | Changing the graph incrementally | Local, structured change |

These are conceptual frames, not keywords. The actual operations use specific verbs.

---

## Observation Operations

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

**Nodes and edges have distinct operations** (ontologically different things):

| Keyword | Target | Description |
|---------|--------|-------------|
| `SPAWN` | Node | Bring a node into existence |
| `KILL` | Node | Remove a node from existence |
| `LINK` | Edge | Create a connection |
| `UNLINK` | Edge | Remove a connection |
| `SET` | Attribute | Modify an attribute (unified for nodes and edges) |

```
-- Node operations
SPAWN e: Event { timestamp = 100 }
KILL e

-- Edge operations  
LINK causes(e1, e2) { mechanism = "direct" }
UNLINK c

-- Attribute operations (works on both)
SET e.timestamp = 200
SET c.confidence = 0.9
```

---

## Design Principles

| Principle | Decision |
|-----------|----------|
| **No umbrella keywords** | Operations use direct verbs (MATCH, SPAWN, etc.) |
| **Distinct node/edge operations** | SPAWN/KILL for nodes, LINK/UNLINK for edges |
| **Graph-native vocabulary** | Not SQL mutations, but graph transformations |
| **Semantic clarity** | Keyword reveals what you're operating on |
| **Philosophical alignment** | Vocabulary reflects the HOHG worldview |

---

## Vocabulary Etymology

| Term | Why This Word |
|------|---------------|
| **MATCH** | Pattern matching — established, precise |
| **WALK** | Physical movement through structure — approachable |
| **INSPECT** | Direct retrieval — universal |
| **SPAWN** | Entity emerging into existence — evocative |
| **KILL** | Entity disappearing from existence — symmetric with SPAWN |
| **LINK** | Connection forming — graph-native |
| **UNLINK** | Connection breaking — symmetric with LINK |
| **SET** | Assign value — universal, clear |

---

## At a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                      OBSERVATION                             │
│                   (reading the graph)                        │
│                                                              │
│   MATCH pattern WHERE condition RETURN projection           │
│   WALK FROM start FOLLOW edges RETURN reached               │
│   INSPECT id                                                     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     TRANSFORMATION                           │
│                   (changing the graph)                       │
│                                                              │
│   SPAWN x: Type { attr = value }      -- create node        │
│   KILL x                          -- remove node        │
│   LINK edge(a, b) { attr = value }    -- create edge        │
│   UNLINK e                            -- remove edge        │
│   SET x.attr = value                  -- modify attribute   │
└─────────────────────────────────────────────────────────────┘
```

---

*This vocabulary is designed to be graph-native, philosophically aligned with HOHG, and semantically clear while remaining learnable.*