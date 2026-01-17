# MEW Interiority

**Version:** 1.0
**Status:** Capability
**Deferred to:** v2
**Scope:** Nested scopes, world boundaries, projections, cross-scope semantics, scoped timing

---

# Part I: Overview

## 1.1 The Core Insight

**Boundaries are epistemic, not just structural.**

A world doesn't have direct edges to external things. It has edges to *its representations* of external things. This mirrors how minds work: I don't have direct access to you — I have a model of you that can be wrong, outdated, incomplete.

This leads to the clean principle: **edges stay inside, external references become projections.**

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   NAVIGATOR'S INTERIOR                                               │
│   ────────────────────                                               │
│                                                                      │
│        ┌──────────┐              ┌───────────────┐                  │
│        │  Route   │─────────────▶│  LandmarkRef  │                  │
│        │ #nav/r1  │    edge      │  #nav/lm1     │                  │
│        └──────────┘              │               │                  │
│                                  │ represents:   │◀── attribute     │
│                                  │ #landmark_42  │    (ID, not edge)│
│                                  └───────────────┘                  │
│                                                                      │
│        All edges stay inside. Boundary is clean.                    │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│                         ╱╱╱ NO EDGE CROSSES ╱╱╱                      │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT                                                               │
│   ────                                                               │
│                                                                      │
│        ┌───────────┐                                                │
│        │ Landmark  │  (the real thing)                              │
│        │#landmark_42│                                               │
│        └───────────┘                                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 1.2 Structural Taxonomy

Two structural primitives, irreducible:

| Component | Definition | Necessity |
|-----------|------------|-----------|
| **Interior** | What's inside the world — local ontology, local data, local rules | Without an inside, no interiority concept exists |
| **Exterior** | How the world appears from outside — a node in the parent scope | Without an exterior, the world is unreachable |

**Sensors and actuators** are not structural primitives. They emerge from:
- Worlds can query/mutate (perception and action)
- Policies gate what succeeds (what you can perceive/affect)

This avoids duplicating policy logic in static type declarations.

## 1.3 What This Enables

| Use Case | How Interiority Helps |
|----------|----------------------|
| **Agent with Knowledge** | Internal knowledge as interior nodes; agent reasons over its own model |
| **Application Sandbox** | Untrusted code runs in isolated interior; cannot affect parent |
| **User Workspace** | Private types and data; selective sharing via exterior |
| **Multi-Tenant System** | Each tenant has interior; shared infrastructure in ROOT |
| **Simulation Within Simulation** | Nested worlds with different physics/rules |

## 1.4 Relationship to Existing Concepts

| MEW Concept | Relationship to Interiority |
|-------------|----------------------------|
| **Ontology** | Interior has its own local ontology (types, edges, constraints, rules) |
| **Policies** | Gate cross-scope access; interior access is policy-controlled |
| **META Mode** | Used to modify interior ontology dynamically |
| **Watches** | Implement perception (sensors) via watch on external patterns |
| **Time Domains** | Interior can have independent tick rate (like existing DOMAIN concept) |
| **Spaces** | Interior can declare local spaces or reference parent spaces |
| **Federation** | Each kernel is a root; federation bridges roots |

---

# Part II: World Declaration

## 2.1 Syntax

```
WorldDecl =
    "node" Identifier "[" "has_interior" "]" "{" 
      ExteriorAttrs
      InteriorDecl?
    "}"

ExteriorAttrs = AttributeDecl*

InteriorDecl = 
    "interior" ":" "ontology" InteriorOptions? "{" OntologyBody "}"

InteriorOptions = "[" InteriorOption ("," InteriorOption)* "]"

InteriorOption =
    "time" ":" TimeConfig
  | "inherit_constraints" ":" Bool
  | "inherit_rules" ":" Bool

TimeConfig =
    "shared"                      -- share parent's tick
  | "independent"                 -- own tick counter
  | "ratio" ":" IntLiteral        -- tick once per N parent ticks (slower)
```

## 2.2 Basic Example

```
node Navigator [has_interior] {
  -- Exterior attributes (visible from outside)
  name: String [required]
  status: String = "idle"
  
  interior: ontology {
    -- Local types (only exist inside this navigator)
    node LandmarkRef {
      represents: ID [required]    -- external ID (attribute, not edge)
      local_name: String?
      cached_position: List<Float>?
      confidence: Float [>= 0, <= 1]
      last_seen: Timestamp?
    }
    
    node Route {
      name: String
      estimated_distance: Float?
    }
    
    -- Hyperedge: connects waypoints in a route
    edge via(LandmarkRef, LandmarkRef, LandmarkRef) {
      order: Int
    }
    
    -- Higher-order edge: confidence about a route
    edge route_confidence(edge<via>, level: Float)
    
    -- Binary edge
    edge visible_from(LandmarkRef, LandmarkRef) {
      distance: Float
    }
    
    -- Local constraints
    constraint confidence_bounds:
      lr: LandmarkRef => lr.confidence >= 0 AND lr.confidence <= 1
    
    -- Local rules
    rule decay_old_sightings [priority: 5]:
      lr: LandmarkRef
      WHERE logical_time() - lr.last_seen_tick > 1000
        AND lr.confidence > 0.1
      =>
      SET lr.confidence = lr.confidence * 0.99
  }
}
```

## 2.3 Exterior vs Interior

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EXTERIOR vs INTERIOR                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   EXTERIOR                             INTERIOR                     │
│   ────────                             ────────                     │
│                                                                      │
│   • Declared in node body              • Declared in interior block │
│   • Visible from parent scope          • Hidden from parent scope   │
│   • Can have edges FROM outside        • Edges only within          │
│   • Standard node semantics            • Separate compilation unit  │
│   • Uses parent's types                • Has own types              │
│                                                                      │
│   Example:                             Example:                     │
│     name: String                         node LandmarkRef { ... }   │
│     status: String                       edge visible_from(...)     │
│     position: List<Float>                rule decay_confidence: ... │
│                                                                      │
│   Queryable as:                        Queryable as:                │
│     MATCH n: Navigator                   MATCH lr: LandmarkRef      │
│     WHERE n.name = "Explorer"            IN #nav WHERE lr.conf > 0.5│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.4 Time Configuration

Interior time can be configured relative to parent:

```
node SlowAgent [has_interior] {
  name: String
  
  interior: ontology [time: ratio(10)] {
    -- This interior ticks once every 10 parent ticks
    -- For slow background processing
    
    node LongTermMemory { content: String }
    
    rule consolidate_memory:
      m: LongTermMemory WHERE m.consolidated = false
      => SET m.consolidated = true
  }
}

node ManualEconomy [has_interior] {
  name: String
  
  interior: ontology [time: independent] {
    -- This interior has completely independent time
    -- Must be explicitly ticked
    
    node Market { price: Float }
  }
}
```

| Time Config | Behavior |
|-------------|----------|
| `shared` (default) | Interior ticks when parent ticks |
| `independent` | Interior only ticks when explicitly advanced |
| `ratio(N)` | Interior ticks once every N parent ticks (slower) |

## 2.5 Constraint and Rule Inheritance

```
node StrictAgent [has_interior] {
  name: String
  
  interior: ontology [inherit_constraints: true] {
    -- Parent constraints also apply here
    -- e.g., if ROOT has constraint on timestamps, it applies
    
    node LocalData { timestamp: Timestamp }
  }
}

node IsolatedAgent [has_interior] {
  name: String
  
  interior: ontology [inherit_constraints: false, inherit_rules: false] {
    -- Clean slate: only local constraints/rules apply
    -- Default behavior
    
    node LocalData { ... }
  }
}
```

Default: `inherit_constraints: false`, `inherit_rules: false`. Interior is isolated unless explicitly inheriting.

---

# Part III: The IN Keyword and Scope Resolution

## 3.1 Scope Specifiers

The `IN` keyword specifies which scope a query/mutation targets:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SCOPE SPECIFIERS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SPECIFIER          MEANING                                        │
│   ─────────          ───────                                        │
│                                                                      │
│   (omitted)          Current scope (default)                        │
│   IN ROOT            Root/shared scope                              │
│   IN SELF            Own interior (from inside a world)             │
│   IN PARENT          Parent scope (one level up)                    │
│   IN #entity_id      Named entity's interior                        │
│   IN $variable       Variable-specified scope                       │
│                                                                      │
│   EXAMPLES                                                          │
│   ────────                                                          │
│                                                                      │
│   -- Query current scope                                            │
│   MATCH t: Task WHERE t.status = "done"                             │
│                                                                      │
│   -- Query root scope                                               │
│   MATCH lm: Landmark IN ROOT WHERE lm.visible = true                │
│                                                                      │
│   -- Query own interior (from within agent code)                    │
│   MATCH lr: LandmarkRef IN SELF WHERE lr.confidence > 0.8           │
│                                                                      │
│   -- Query specific entity's interior                               │
│   MATCH lr: LandmarkRef IN #nav WHERE lr.local_name LIKE "%tower%"  │
│                                                                      │
│   -- Query parent scope (from nested world)                         │
│   MATCH x: SomeType IN PARENT                                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Mutation Scoping

Mutations also use `IN` for targeting:

```
-- Spawn in current scope
SPAWN t: Task { title = "Test" }

-- Spawn in ROOT
SPAWN lm: Landmark { name = "New Tower" } IN ROOT

-- Spawn in own interior
SPAWN lr: LandmarkRef { represents = #lm_1, confidence = 0.7 } IN SELF

-- Spawn in specific entity's interior (requires access)
SPAWN lr: LandmarkRef { represents = #lm_99 } IN #navigator
```

## 3.3 Scope Resolution Order

When a type name is used, resolution follows this order:

```
┌─────────────────────────────────────────────────────────────────────┐
│                   TYPE RESOLUTION ORDER                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. Current scope's local types                                    │
│   2. Inherited parent types (if inherit enabled)                    │
│   3. ROOT types (always available for reference)                    │
│                                                                      │
│   EXPLICIT QUALIFICATION                                            │
│   ──────────────────────                                            │
│                                                                      │
│   When ambiguous, qualify explicitly:                               │
│                                                                      │
│   MATCH t: Task                    -- resolves via order above      │
│   MATCH t: ROOT.Task               -- explicitly ROOT's Task        │
│   MATCH t: SELF.Task               -- explicitly local Task         │
│                                                                      │
│   TYPE SHADOWING                                                    │
│   ──────────────                                                    │
│                                                                      │
│   Local types shadow parent types of same name.                     │
│   This is intentional: allows local redefinition.                   │
│                                                                      │
│   interior: ontology {                                              │
│     node Task {                    -- shadows ROOT.Task            │
│       local_priority: Int          -- different attributes         │
│     }                                                               │
│   }                                                                  │
│                                                                      │
│   MATCH t: Task IN SELF            -- local Task                    │
│   MATCH t: Task IN ROOT            -- ROOT's Task                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.4 Cross-Scope Pattern Matching

Patterns can span scopes when authorized:

```
-- Find beliefs about external landmarks
MATCH 
  lr: LandmarkRef IN SELF,
  r: Route IN SELF,
  starts_at(r, lr),                  -- edge within SELF
  lm: Landmark IN ROOT               -- external landmark
WHERE lr.represents = lm.id          -- correlation via ID
  AND lm.visible = true
RETURN lr.local_name, lm.position

-- This works because:
-- 1. LandmarkRef, Route, starts_at() are all IN SELF (edges stay inside)
-- 2. Landmark is IN ROOT (separate query)
-- 3. Correlation is via ID attribute comparison, not edge
```

---

# Part IV: Projections

## 4.1 What Projections Are

A **projection** is an interior node that represents an external entity. It contains:
- A `represents: ID` attribute pointing to the external entity
- Local attributes capturing the world's understanding/interpretation
- Potentially stale data (that's correct — perception is modeling)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      PROJECTION ANATOMY                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   node LandmarkRef {                                                │
│     represents: ID [required]      -- external entity ID           │
│     local_name: String?            -- my name for it               │
│     cached_position: List<Float>?  -- cached external data         │
│     last_synced: Timestamp?        -- when I last updated          │
│     confidence: Float?             -- how sure am I                │
│     notes: String?                 -- my interpretation            │
│   }                                                                  │
│                                                                      │
│   KEY PROPERTIES                                                    │
│   ──────────────                                                    │
│                                                                      │
│   • It's a regular node in the interior                            │
│   • It can have edges TO other interior nodes                      │
│   • The `represents` field is just an ID (attribute, not edge)     │
│   • It can be stale — external entity may have changed             │
│   • It's the world's MODEL of the external thing                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.2 Creating Projections

Three patterns, from most automated to most manual:

### 4.2.1 WATCH Pattern (Continuous Sync)

```
-- Most common for agents: automatically maintain projections

WATCH Landmark IN ROOT 
WHERE distance(Landmark, owner_of(SELF), Physical) < 100
[mode: watch]
ON_CREATE(lm): 
  SPAWN lr: LandmarkRef IN SELF { 
    represents = lm.id,
    local_name = lm.name,
    cached_position = lm.position,
    last_seen = now()
  }
ON_UPDATE(lm):
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  SET lr.cached_position = lm.position,
      lr.last_seen = now()
ON_DELETE(lm):
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  KILL lr
```

This is the **sensor pattern**: the watch monitors ROOT and maintains projections automatically.

### 4.2.2 INTERNALIZE Pattern (One-Shot Snapshot)

```
-- Sugar for "query external, create projection"

LET landmarks = QUERY Landmark IN ROOT WHERE lm.category = "relevant"

FOR lm IN landmarks:
  SPAWN lr: LandmarkRef IN SELF {
    represents = lm.id,
    local_name = lm.name,
    cached_position = lm.position,
    last_synced = now()
  }
```

Expands to: for each matching entity, SPAWN a projection with `represents = entity.id`.

### 4.2.3 Explicit Pattern (Full Control)

```
-- Manual projection creation
SPAWN lr: LandmarkRef IN SELF {
  represents = #landmark_42,
  local_name = "The tall tower",
  confidence = 0.6,
  notes = "Saw it through fog"
}
```

## 4.3 Projection Staleness

Projections can become stale. This is **correct behavior** — it models real perception:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PROJECTION STALENESS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   TIME T=0                                                          │
│   ────────                                                          │
│   External: Landmark { position = [10, 20, 0] }                     │
│   Projection: LandmarkRef { cached_pos = [10,20,0] } ✓ in sync     │
│                                                                      │
│   TIME T=100 (external changes, no sync)                            │
│   ──────────────────────────────────────                            │
│   External: Landmark { position = [15, 25, 0] }                     │
│   Projection: LandmarkRef { cached_pos = [10,20,0] } ✗ stale       │
│                                                                      │
│   This is CORRECT. The world's model doesn't auto-update.          │
│   To get fresh data, the world must:                                │
│   • Have an active watch (auto-sync)                               │
│   • Explicitly re-query and update                                 │
│   • Accept that its model may be outdated                          │
│                                                                      │
│   DESIGN RATIONALE                                                  │
│   ────────────────                                                  │
│   Real agents don't have perfect information.                      │
│   Modeling staleness is modeling reality.                          │
│   If you want auto-sync, use WATCH.                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.4 Resolving Projections

To get current external data from a projection:

```
-- Manual resolution
MATCH lr: LandmarkRef IN SELF WHERE lr.represents = $target_id
LET current = QUERY Landmark IN ROOT WHERE Landmark.id = lr.represents
RETURN pr.local_name, current.status, pr.cached_status

-- Sugar: RESOLVE function
MATCH lr: LandmarkRef IN SELF
LET current = RESOLVE(pr) IN ROOT  -- fetches current external state
WHERE current != null              -- may not exist anymore
RETURN pr.local_name, current.status
```

`RESOLVE(projection) IN scope` fetches the current state of what the projection represents.

---

# Part V: Access Policies and Cross-Scope Access

## 5.1 Policy Model for Worlds

Access policies for worlds follow these principles:

| Principle | Description |
|-----------|-------------|
| **Self-access** | A world can always read/write its own interior |
| **Parent authority** | Parent can read/write children's interiors |
| **Default deny** | Cross-scope access denied unless explicitly granted |
| **Explicit grants** | Worlds can grant others specific access |

## 5.2 Built-in Access Policies

```
┌─────────────────────────────────────────────────────────────────────┐
│                 DEFAULT WORLD ACCESS POLICIES                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   -- Self always has full access to own interior                    │
│   policy _self_interior_full_access [priority: 1000]:               │
│     ON * IN SELF                                                    │
│     ALLOW IF true                                                   │
│                                                                      │
│   -- Parent can read children's interiors                           │
│   policy _parent_read_child [priority: 900]:                        │
│     ON MATCH(_) IN child                                            │
│     ALLOW IF parent_of(current_scope(), child)                      │
│                                                                      │
│   -- Parent can write children's interiors                          │
│   policy _parent_write_child [priority: 900]:                       │
│     ON SPAWN(_) | KILL(_) | SET(_) | LINK(_) | UNLINK(_) IN child  │
│     ALLOW IF parent_of(current_scope(), child)                      │
│                                                                      │
│   -- ROOT can access all (system authority)                         │
│   policy _root_authority [priority: 800]:                           │
│     ON * IN _                                                       │
│     ALLOW IF current_scope() = ROOT                                 │
│                                                                      │
│   -- Default: deny cross-scope access                               │
│   policy _default_cross_scope_deny [priority: -1000]:               │
│     ON * IN target WHERE target != SELF                             │
│     DENY IF true                                                    │
│     MESSAGE "Cross-scope access requires explicit grant"            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.3 Granting Interior Access

A world can grant others access to its interior:

```
-- Grant read access to specific entity
LINK interior_read_access(#navigator, #trusted_observer)

-- Grant write access
LINK interior_write_access(#navigator, #trusted_modifier)

-- Grant with limitations
LINK interior_read_access(#navigator, #limited_observer) {
  allowed_types: ["PublicLandmarkRef"],  -- only certain types
  expires_at: now() + 1.hour             -- time-limited
}
```

Policy that respects grants:

```
policy granted_interior_read:
  ON MATCH(x: any) IN target_world
  ALLOW IF EXISTS(interior_read_access(target_world, current_actor()))
       OR EXISTS(interior_read_access(target_world, current_scope()))

policy granted_interior_write:
  ON SPAWN(_) | KILL(_) | SET(_) | LINK(_) | UNLINK(_) IN target_world
  ALLOW IF EXISTS(interior_write_access(target_world, current_actor()))
       OR EXISTS(interior_write_access(target_world, current_scope()))
```

## 5.4 Observation Policies in Worlds

Following the general MEW policy model, observation policies for interiors are **filtering, not gating**:

```
┌─────────────────────────────────────────────────────────────────────┐
│              INTERIOR OBSERVATION POLICIES                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Query from ROOT:                                                  │
│     MATCH lr: LandmarkRef IN #navigator                             │
│                                                                      │
│   If not authorized to see #navigator's interior:                   │
│     Result: empty (not error)                                       │
│                                                                      │
│   If authorized to see some but not all:                            │
│     Result: filtered to what's visible                              │
│                                                                      │
│   EXAMPLE                                                           │
│   ───────                                                           │
│                                                                      │
│   Navigator's interior has:                                         │
│     PublicLandmarkRef (10 instances)                                │
│     PrivateLandmarkRef (5 instances)                                │
│                                                                      │
│   Observer has:                                                     │
│     interior_read_access(#navigator, #observer) {                   │
│       allowed_types: ["Public*"]                                    │
│     }                                                               │
│                                                                      │
│   Observer queries:                                                 │
│     MATCH lr: LandmarkRef IN #navigator                             │
│                                                                      │
│   Result: 10 PublicLandmarkRef instances                            │
│   (PrivateLandmarkRef filtered out, no error)                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.5 Mutation Policies

External mutation of interior requires explicit grant:

```
-- Attempting to modify navigator's interior from observer's context
SPAWN lr: LandmarkRef IN #navigator { represents = #lm_99 }

-- Policy check:
-- 1. Is current_actor() authorized to SPAWN in #navigator?
-- 2. Check: interior_write_access(#navigator, current_actor())
-- 3. If not found: DENIED

-- If observer has write access:
LINK interior_write_access(#navigator, #observer)

-- Then the mutation succeeds (subject to navigator's interior constraints)
```

## 5.6 Policy Context Functions

New context functions for world policies:

```
┌─────────────────────────────────────────────────────────────────────┐
│                  CONTEXT FUNCTIONS FOR WORLDS                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   FUNCTION                  RETURNS                                 │
│   ────────                  ───────                                 │
│                                                                      │
│   current_scope()           The scope executing the operation       │
│   current_actor()           The actor (unchanged from base)         │
│   target_scope()            The scope being accessed                │
│   parent_of(scope)          The parent scope, or null if ROOT       │
│   is_interior(scope)        True if scope is a world interior       │
│   owner_of(scope)           The exterior node owning this interior  │
│                                                                      │
│   EXAMPLES                                                          │
│   ────────                                                          │
│                                                                      │
│   -- Policy based on scope relationship                             │
│   policy sibling_read:                                              │
│     ON MATCH(_) IN target                                           │
│     ALLOW IF parent_of(current_scope()) = parent_of(target)        │
│     -- Siblings can read each other                                │
│                                                                      │
│   -- Policy based on owner                                          │
│   policy owner_delegates:                                           │
│     ON * IN target                                                  │
│     ALLOW IF delegates_to(owner_of(target), current_actor())       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VI: META Operations in Worlds

## 6.1 Querying Own Ontology

A world can inspect its own structure using META:

```
-- From inside a world, query own types
META MATCH t: _NodeType IN SELF
RETURN t.name, t.abstract

-- Query own constraints
META MATCH c: _ConstraintDef IN SELF
WHERE c.hard = true
RETURN c.name

-- Query own rules
META MATCH r: _RuleDef IN SELF
WHERE r.auto = true
RETURN r.name, r.priority, r._invocation_count
```

## 6.2 Modifying Own Ontology

Worlds can evolve their own structure dynamically:

```
-- Create new type in own interior
META CREATE NODE LearnedConcept {
  name: String [required],
  confidence: Float,
  discovered_at: Timestamp = now()
} IN SELF

-- Create new edge type
META CREATE EDGE relates(LearnedConcept, LearnedConcept) {
  strength: Float
} IN SELF

-- Create new rule
META CREATE RULE decay_confidence [priority: 5] IN SELF:
  c: LearnedConcept
  WHERE logical_time() - c.discovered_tick > 100
    AND c.confidence > 0.1
  =>
  SET c.confidence = c.confidence * 0.95

-- Create new constraint
META CREATE CONSTRAINT concept_name_required IN SELF:
  c: LearnedConcept => c.name != ""
```

## 6.3 META Permissions in Worlds

| Operation | SELF | Child | Other |
|-----------|------|-------|-------|
| META MATCH | Always | If parent | Requires grant |
| META CREATE | Always | If parent | Requires grant |
| META SET | Always | If parent | Requires grant |
| META KILL | Always | If parent | Requires grant |

```
-- META permission grant
LINK interior_meta_access(#target_world, #trusted_entity) {
  level: "write"  -- "read", "write", or "admin"
}

-- Access policy
policy granted_meta_access:
  ON META * IN target_world
  ALLOW IF EXISTS(interior_meta_access(target_world, current_actor()) AS g)
       AND (g.level = "write" OR g.level = "admin")
```

## 6.4 Recompilation Scope

When interior ontology changes, only that interior recompiles:

```
┌─────────────────────────────────────────────────────────────────────┐
│                  RECOMPILATION SCOPE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   META CREATE NODE NewType { ... } IN #navigator                    │
│                                                                      │
│   Recompiles: #navigator's interior only                            │
│   Does NOT recompile: ROOT, other worlds                            │
│                                                                      │
│   META CREATE NODE NewType { ... } IN ROOT                          │
│                                                                      │
│   Recompiles: ROOT and all worlds that inherit ROOT types          │
│   (Most interiors don't inherit, so usually just ROOT)             │
│                                                                      │
│   ISOLATION BENEFIT                                                 │
│   ─────────────────                                                 │
│   Worlds evolving independently don't affect each other.           │
│   An agent learning new concepts doesn't slow down ROOT.           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VII: Rules and Constraints

## 7.1 Rule Scope

Rules fire only within their declared scope:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      RULE SCOPE                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT RULE                                                         │
│   ─────────                                                         │
│   rule in ROOT:                                                     │
│     lm: Landmark WHERE lm.decayed = true                            │
│     => SET lm.status = "needs_update"                               │
│                                                                      │
│   Fires on: ROOT's Landmark instances                               │
│   Does NOT fire on: Any interior's instances                        │
│                                                                      │
│                                                                      │
│   INTERIOR RULE                                                     │
│   ─────────────                                                     │
│   rule in #navigator's interior:                                    │
│     lr: LandmarkRef WHERE lr.confidence < 0.1                       │
│     => KILL lr                                                      │
│                                                                      │
│   Fires on: #navigator's LandmarkRef instances                      │
│   Does NOT fire on: ROOT, other interiors                           │
│                                                                      │
│                                                                      │
│   RULE CANNOT CROSS BOUNDARIES                                      │
│   ────────────────────────────                                      │
│   A rule's pattern and production are always in the same scope.    │
│   Cannot match in one scope and produce in another.                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.2 Rules That Affect External (Actuators)

A world can affect ROOT through explicit mutations (its "actuators"):

```
-- Interior rule that affects external (via policy)
rule publish_high_confidence_discovery [priority: 10]:
  c: LearnedConcept IN SELF
  WHERE c.confidence > 0.9
    AND c.published = false
  =>
  -- Create in ROOT (requires access)
  SPAWN d: Discovery { 
    content = c.name, 
    source = owner_of(SELF)  -- the agent's exterior node
  } IN ROOT,
  -- Update local state
  SET c.published = true

-- This rule:
-- 1. Matches pattern in SELF (interior)
-- 2. Produces in both SELF and ROOT
-- 3. The ROOT mutation is subject to policy
-- 4. If not authorized, rule fails (transaction rolls back)
```

Policy for actuators:

```
-- Grant agent permission to create Discoveries in ROOT
policy agent_can_publish:
  ON SPAWN(d: Discovery) IN ROOT
  ALLOW IF d.source = current_scope()  -- agent publishing its own discoveries
       AND is_interior(current_scope())
```

## 7.3 Constraint Scope

Constraints are scoped similarly to rules:

```
┌─────────────────────────────────────────────────────────────────────┐
│                   CONSTRAINT SCOPE                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT CONSTRAINT                                                   │
│   ───────────────                                                   │
│   constraint unique_landmark_name IN ROOT:                          │
│     lm1: Landmark, lm2: Landmark                                    │
│     WHERE lm1.id != lm2.id                                          │
│     => lm1.name != lm2.name                                         │
│                                                                      │
│   Enforced on: ROOT's Landmark instances                            │
│   NOT enforced on: Any interior (even if they have Landmark type)  │
│                                                                      │
│                                                                      │
│   INTERIOR CONSTRAINT                                               │
│   ───────────────────                                               │
│   constraint landmark_confidence_valid IN #navigator:               │
│     lr: LandmarkRef                                                 │
│     => lr.confidence >= 0 AND lr.confidence <= 1                    │
│                                                                      │
│   Enforced on: #navigator's LandmarkRef instances                   │
│   NOT enforced on: ROOT, other interiors                            │
│                                                                      │
│                                                                      │
│   INHERITED CONSTRAINTS                                             │
│   ─────────────────────                                             │
│   If interior is declared with [inherit_constraints: true]:         │
│   • Parent constraints also apply to interior                      │
│   • Interior can add additional constraints (more restrictive)     │
│   • Interior CANNOT relax parent constraints                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.4 Cross-Scope Constraint (Meta-Constraints)

A parent can define constraints that span multiple children:

```
-- ROOT constraint about world exteriors (not interiors)
constraint navigator_landmark_limit:
  n: Navigator
  WHERE n.has_interior = true
  => landmark_count(n) < 10000  -- can query interior stats on exterior

-- This queries the exterior's derived attribute
-- The interior itself doesn't need to know about this limit
```

The exterior can expose derived attributes from interior:

```
node Navigator [has_interior] {
  name: String
  
  -- Derived from interior
  landmark_count: Int [computed: COUNT(LandmarkRef IN SELF)]
  max_confidence: Float [computed: MAX(lr.confidence FOR lr: LandmarkRef IN SELF)]
  
  interior: ontology { ... }
}
```

---

# Part VIII: Time in Worlds

## 8.1 Interiors as Time Scopes

Each world interior has its own **logical time** that can be configured relative to its parent. See [TIME_CLOCK.md](./TIME_CLOCK.md) for time concepts (wall_time, logical_time, now, presets).

```
┌─────────────────────────────────────────────────────────────────────┐
│                  INTERIOR TIME CONFIGURATION                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   TIME MODE              BEHAVIOR                                   │
│   ─────────              ────────                                   │
│                                                                      │
│   [time: shared]         Ticks when parent ticks (default)          │
│                                                                      │
│   [time: ratio(N)]       Ticks every Nth parent tick (slower)       │
│                          e.g., ratio(10) = 1/10th the rate          │
│                                                                      │
│   [time: independent]    Only ticks when explicitly advanced        │
│                          via TICK IN #interior                       │
│                                                                      │
│   KEY PRINCIPLE                                                     │
│   ─────────────                                                     │
│   Parent is the clock source. Children can be:                      │
│   • Same rate (shared)                                              │
│   • Slower (ratio)                                                  │
│   • Decoupled (independent)                                         │
│                                                                      │
│   Children cannot tick faster than parent — that would violate      │
│   causality (child seeing stale parent state across ticks).         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.2 Time Functions in Worlds

```
-- From inside a world:
logical_time()              -- this interior's tick count
logical_time(PARENT)        -- parent's tick count
logical_time(ROOT)          -- root's tick count

wall_time()                 -- always global wall clock

now()                       -- configured per interior:
                           -- if time.now_source = "logical": logical_time()
                           -- if time.now_source = "wall": wall_time()
```

## 8.3 Tick Propagation

```
┌─────────────────────────────────────────────────────────────────────┐
│                   TICK PROPAGATION                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT ticks                                                        │
│     │                                                                │
│     ├── Agent A [time: shared]        → ticks with ROOT            │
│     │     │                                                          │
│     │     └── SubAgent [time: ratio(2)] → ticks every 2nd A tick   │
│     │                                                                │
│     ├── Agent B [time: independent]   → does NOT tick              │
│     │                                                                │
│     └── Agent C [time: ratio(10)]     → ticks every 10th ROOT tick │
│                                                                      │
│                                                                      │
│   EXPLICIT TICK                                                     │
│   ─────────────                                                     │
│                                                                      │
│   TICK                    -- current scope only                     │
│   TICK IN #agent_b        -- tick specific interior                 │
│   TICK ALL                -- tick all scopes (expensive)            │
│   TICK CHILDREN           -- tick all direct children               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.4 Temporal Queries Across Scopes

```
-- Find landmarks seen recently (interior logical time)
MATCH lr: LandmarkRef IN SELF
WHERE logical_time() - lr.last_seen_tick < 10
RETURN lr

-- Correlate interior time with external events
MATCH 
  lr: LandmarkRef IN SELF,
  e: Event IN ROOT
WHERE lr.represents = e.landmark
  AND lr.last_seen > e.timestamp  -- wall time comparison
RETURN lr, e
```

---

# Part IX: Space in Worlds

## 9.1 Space Declaration in Interiors

Interiors can declare their own spaces:

```
node Navigator [has_interior] {
  name: String
  
  interior: ontology {
    -- Local space (only meaningful inside this navigator)
    space SemanticSpace [dimensions: 128, metric: cosine]
    
    node LandmarkRef {
      represents: ID
      -- can be positioned in SemanticSpace
    }
    
    -- Landmarks can be positioned by semantic similarity
  }
}
```

## 9.2 Referencing Parent Spaces

Interiors can reference and use parent spaces:

```
node Navigator [has_interior] {
  name: String
  
  interior: ontology {
    -- Reference ROOT's Physical space
    use space Physical FROM ROOT
    
    node LandmarkRef {
      represents: ID
      -- Can track believed positions in ROOT's space
    }
    
    node PositionEstimate {
      landmark: LandmarkRef
      believed_position: Position IN Physical  -- in ROOT's space
    }
  }
}
```

## 9.3 Position Relationships

```
┌─────────────────────────────────────────────────────────────────────┐
│                SPACE AND WORLD INTERACTION                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ENTITY POSITIONING                                                │
│   ──────────────────                                                │
│                                                                      │
│   The EXTERIOR node can be positioned in ROOT's spaces:             │
│                                                                      │
│   PLACE #navigator IN Physical AT (10, 20, 0)                       │
│   -- Navigator's exterior has a position in ROOT's Physical space  │
│                                                                      │
│   Interior nodes can be positioned in interior spaces:              │
│                                                                      │
│   PLACE #navigator/lm_ref_1 IN SemanticSpace AT $embedding          │
│   -- Navigator's landmark ref has position in its semantic space   │
│                                                                      │
│                                                                      │
│   CROSS-SPACE QUERIES                                               │
│   ───────────────────                                               │
│                                                                      │
│   -- From inside navigator, query what's near it in ROOT            │
│   MATCH e: Entity IN ROOT                                           │
│   WHERE distance(owner_of(SELF), e, Physical) < 50                  │
│   RETURN e                                                          │
│                                                                      │
│   -- Query local semantic space                                     │
│   MATCH lr: LandmarkRef IN SELF                                     │
│   WHERE lr IN nearest(#target_landmark, 5, SemanticSpace)           │
│   RETURN lr                                                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 9.4 Spatial Access Policies

Spatial queries across scopes follow policy rules:

```
policy perception_range:
  ON MATCH(e: Entity) IN ROOT
  ALLOW IF distance(e, owner_of(current_scope()), Physical) < 100
       OR NOT is_interior(current_scope())  -- non-interior has no range limit
  MESSAGE "Entity outside perception range"
```

---

# Part X: Watches as Sensors

## 10.1 The Sensor Pattern

**Sensors are watches on external scopes.** This unifies concepts:

```
┌─────────────────────────────────────────────────────────────────────┐
│                   WATCHES AS SENSORS                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   TRADITIONAL VIEW                   MEW VIEW                       │
│   ────────────────                   ────────                       │
│                                                                      │
│   "Agent has sensors for            "Agent watches                  │
│    perceiving Landmarks"             Landmarks in ROOT"             │
│                                                                      │
│   sensor: Landmark                   WATCH Landmark IN ROOT         │
│                                        [mode: watch]                │
│                                        ON_CREATE: ...               │
│                                        ON_UPDATE: ...               │
│                                                                      │
│                                                                      │
│   THE UNIFICATION                                                   │
│   ───────────────                                                   │
│                                                                      │
│   • Perception IS watching                                          │
│   • What you can perceive = what you're authorized to query        │
│   • Updates propagate via watch events                             │
│   • Projection maintenance is watch handler logic                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.2 Sensor Watch Patterns

### 10.2.1 Simple Observation Sensor

```
-- Perceive all visible landmarks
WATCH Landmark IN ROOT
WHERE Landmark.visible = true
[mode: watch, initial: full]
ON_CREATE(lm):
  SPAWN lr: LandmarkRef IN SELF { represents = lm.id, local_name = lm.name }
ON_UPDATE(lm):
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  SET lr.local_name = lm.name, lr.last_seen = now()
ON_DELETE(lm):
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  KILL lr

-- This watch IS the sensor
-- It defines what the world perceives and how it updates its model
```

### 10.2.2 Proximity Sensor (Spatial + Watch)

```
-- Perceive nearby landmarks
WATCH Landmark IN ROOT
WHERE distance(Landmark, owner_of(SELF), Physical) < perception_radius()
[mode: watch]
ON_CREATE(lm):
  SPAWN lr: LandmarkRef IN SELF { 
    represents = lm.id, 
    perceived_at = now(),
    cached_position = position(lm, Physical)
  }
ON_UPDATE(lm):
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  SET lr.cached_position = position(lm, Physical),
      lr.last_seen = now()
ON_DELETE(lm):
  -- Landmark left perception range or was deleted
  MATCH lr: LandmarkRef IN SELF WHERE lr.represents = lm.id
  SET lr.last_seen = now()  -- keep stale, don't delete

-- Spatial filtering happens server-side
-- Only nearby landmarks trigger watch events
```

### 10.2.3 Selective Sensor (Type-Filtered)

```
-- Only perceive specific types
WATCH (Landmark | Obstacle | Waypoint) IN ROOT
WHERE relevance_score(current_scope(), $) > 0.5
[mode: watch]
ON_CREATE(entity):
  -- Create appropriate projection based on type
  CASE type_of(entity)
    WHEN "Landmark": SPAWN LandmarkRef IN SELF { represents = entity.id }
    WHEN "Obstacle": SPAWN ObstacleRef IN SELF { represents = entity.id }
    WHEN "Waypoint": SPAWN WaypointRef IN SELF { represents = entity.id }
```

## 10.3 Sensor Policy Integration

Sensors (watches) are subject to observation policies:

```
┌─────────────────────────────────────────────────────────────────────┐
│              SENSOR POLICIES                                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   WATCH CREATION                                                    │
│   ──────────────                                                    │
│                                                                      │
│   When creating a watch, policy is checked:                         │
│   • Can current_scope() create watches?                             │
│   • Can current_scope() MATCH the pattern in target scope?         │
│                                                                      │
│   If either fails, watch creation is denied.                        │
│                                                                      │
│                                                                      │
│   EVENT FILTERING                                                   │
│   ───────────────                                                   │
│                                                                      │
│   Even with a valid watch, events are filtered:                     │
│   • Each event checked against observation policies                │
│   • Events for unauthorized entities are not delivered             │
│   • Policies can change over time                                  │
│                                                                      │
│                                                                      │
│   EXAMPLE                                                           │
│   ───────                                                           │
│                                                                      │
│   Agent watches: Landmark IN ROOT                                   │
│   Agent can see: public Landmarks only                             │
│   Agent receives: CREATE events only for public Landmarks          │
│                                                                      │
│   If a Landmark becomes public → Agent receives CREATE event       │
│   If a Landmark becomes private → Agent receives DELETE event      │
│   (from agent's perspective, it disappeared)                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 10.4 Actuators via Mutation

Symmetrically, actuators are mutations to external scopes:

```
-- Interior rule that acts on ROOT (actuator pattern)
rule report_discovery:
  d: Discovery IN SELF
  WHERE d.confidence > 0.9 AND d.reported = false
  =>
  -- Actuator: create in ROOT
  SPAWN r: Report {
    content = d.summary,
    source = owner_of(SELF)
  } IN ROOT,
  -- Update local state
  SET d.reported = true

-- This mutation to ROOT is subject to policy
-- Policies determine what the world can affect
```

---

# Part XI: Higher-Order Edges in Worlds

## 11.1 Unchanged Semantics

Higher-order edges work the same within worlds. The key insight: **edges about edges are still edges within the same scope.**

```
┌─────────────────────────────────────────────────────────────────────┐
│              HIGHER-ORDER IN INTERIORS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   node Navigator [has_interior] {                                   │
│     interior: ontology {                                            │
│       node LandmarkRef { represents: ID }                           │
│       edge via(LandmarkRef, LandmarkRef, LandmarkRef)               │
│       edge route_confidence(edge<via>, level: Float)                │
│       edge assessed_by(edge<route_confidence>, method: String)      │
│     }                                                                │
│   }                                                                  │
│                                                                      │
│   USAGE                                                             │
│   ─────                                                             │
│                                                                      │
│   SPAWN lr1: LandmarkRef IN SELF { represents = #lm_1 }             │
│   SPAWN lr2: LandmarkRef IN SELF { represents = #lm_2 }             │
│   SPAWN lr3: LandmarkRef IN SELF { represents = #lm_3 }             │
│   LINK via(lr1, lr2, lr3) AS v IN SELF                              │
│   LINK route_confidence(v) { level = 0.8 } AS rc IN SELF            │
│   LINK assessed_by(rc) { method = "traversal" } IN SELF             │
│                                                                      │
│   QUERY                                                             │
│   ─────                                                             │
│                                                                      │
│   MATCH                                                             │
│     lr1: LandmarkRef, lr2: LandmarkRef, lr3: LandmarkRef,          │
│     via(lr1, lr2, lr3) AS v,                                       │
│     route_confidence(v) AS rc                                       │
│   IN SELF                                                           │
│   WHERE rc.level > 0.7                                              │
│   RETURN lr1.represents, lr3.represents, rc.level                   │
│                                                                      │
│   All edges are within SELF. No cross-boundary edges.              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 11.2 Higher-Order on Projections

Higher-order edges can reference projections:

```
interior: ontology {
  node LandmarkRef { represents: ID }
  node Route { name: String }
  
  edge passes_through(Route, LandmarkRef)
  edge certainty(edge<passes_through>, level: Float)
}

-- "I believe this route passes through landmark with confidence 0.8"
SPAWN r: Route IN SELF { name = "Northern Path" }
LINK passes_through(r, #lr_ref) AS pt IN SELF
LINK certainty(pt) { level = 0.8 } IN SELF

-- The higher-order structure is entirely within SELF
-- It's about the projection, not the external entity
```

## 11.3 No Cross-Scope Higher-Order

Higher-order edges cannot span scopes:

```
-- INVALID: Cannot have edge in SELF targeting edge in ROOT
LINK confidence(#edge_in_root) { level = 0.8 } IN SELF
-- ERROR: Target edge is not in current scope

-- VALID: Create projection of the edge concept
SPAWN edge_ref: EdgeRef IN SELF { represents = #edge_in_root }
LINK my_confidence(edge_ref) { level = 0.8 } IN SELF
-- This is metadata about my projection, not the real edge
```

---

# Part XII: Sandbox and Isolation

## 12.1 Temporary Isolated Scope

The `WITH ISOLATED` construct creates a temporary world:

```
WITH ISOLATED {
  -- Temporary interior, no access to outside
  
  SPAWN t: Task { title = "Test" }      -- OK: local
  MATCH lm: Landmark IN ROOT            -- FAILS: not authorized
  
  -- Whatever is created here is discarded after block
}
```

## 12.2 Scoped Access

More granular control via `WITH SCOPE`:

```
-- Read-only access to ROOT
WITH SCOPE(can_query: ROOT, can_mutate: SELF) {
  LET landmarks = QUERY Landmark IN ROOT WHERE lm.visible = true
  
  FOR lm IN landmarks:
    SPAWN lr: LandmarkRef { represents = lm.id }  -- OK: local
  
  SPAWN x: Landmark IN ROOT  -- FAILS: mutation not authorized
}

-- Full isolation
WITH SCOPE(can_query: SELF, can_mutate: SELF) {
  -- Complete sandbox
}

-- Specific type access
WITH SCOPE(can_query: [Landmark, Waypoint] IN ROOT, can_mutate: SELF) {
  -- Can only see Landmarks and Waypoints from ROOT
}
```

## 12.3 Persistent Sandbox (World as Sandbox)

A world can serve as a persistent sandbox:

```
-- Create sandboxed execution environment
SPAWN sandbox: SandboxWorld [has_interior] {
  owner = current_actor()
  
  interior: ontology [inherit_constraints: false] {
    -- Clean environment
    node Experiment { data: String }
    rule auto_process: e: Experiment => SET e.processed = true
  }
}

-- Run untrusted code in sandbox
EXECUTE $user_code IN #sandbox
-- Code runs with #sandbox as its scope
-- Cannot affect ROOT or other worlds
```

## 12.4 Sandbox Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                   SANDBOX LIFECYCLE                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CREATE                                                            │
│   ──────                                                            │
│   SPAWN s: Sandbox [has_interior] { ... }                          │
│   -- Interior starts empty (or with declared types)                │
│                                                                      │
│   EXECUTE                                                           │
│   ───────                                                           │
│   EXECUTE code IN #s                                                │
│   -- Code runs with #s as scope                                    │
│   -- Subject to #s's access policies (usually isolated)            │
│                                                                      │
│   INSPECT                                                           │
│   ───────                                                           │
│   MATCH x: any IN #s                                                │
│   -- Owner can inspect sandbox contents                            │
│                                                                      │
│   RESET                                                             │
│   ─────                                                             │
│   RESET INTERIOR #s                                                 │
│   -- Clear all data, keep types                                    │
│                                                                      │
│   DESTROY                                                           │
│   ───────                                                           │
│   KILL #s                                                           │
│   -- Destroy sandbox and all contents                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part XIII: Nesting

## 13.1 Worlds Within Worlds

Worlds can contain sub-worlds:

```
node Navigator [has_interior] {
  name: String
  
  interior: ontology {
    node LandmarkRef { represents: ID }
    
    -- Sub-world for isolated route planning
    node PlanningContext [has_interior] {
      goal: String
      
      interior: ontology {
        node Hypothesis { route: String }
        node Evaluation { score: Float }
      }
    }
  }
}

-- Create nested structure
SPAWN nav: Navigator { name = "Explorer" }
SPAWN ctx: PlanningContext IN #nav { goal = "reach summit" }
SPAWN h: Hypothesis IN #ctx { route = "via north ridge" }
```

## 13.2 Nesting Depth

```
┌─────────────────────────────────────────────────────────────────────┐
│                    NESTING STRUCTURE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ROOT                                                              │
│     │                                                                │
│     ├── Navigator A (depth 1)                                       │
│     │     │                                                          │
│     │     ├── PlanningContext 1 (depth 2)                          │
│     │     │     │                                                    │
│     │     │     └── SubPlan (depth 3)                              │
│     │     │                                                          │
│     │     └── PlanningContext 2 (depth 2)                          │
│     │                                                                │
│     └── Navigator B (depth 1)                                       │
│           │                                                          │
│           └── Simulation (depth 2)                                  │
│                 │                                                    │
│                 └── SimNavigator (depth 3)                          │
│                       │                                              │
│                       └── SimPlanningContext (depth 4)              │
│                                                                      │
│   DEPTH LIMITS                                                      │
│   ────────────                                                      │
│   Default max depth: 10                                             │
│   Configurable: SET world.max_nesting_depth = N                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 13.3 Scope References in Nested Contexts

```
-- From depth 3 (SubPlan):
MATCH x: Type IN SELF              -- SubPlan's interior
MATCH x: Type IN PARENT            -- PlanningContext 1's interior
MATCH x: Type IN PARENT.PARENT     -- Navigator A's interior
MATCH x: Type IN ROOT              -- always the root

-- Relative references
MATCH x: Type IN ancestor(2)       -- 2 levels up
MATCH x: Type IN ancestor("Navigator") -- nearest ancestor of type Navigator
```

## 13.4 Policy Inheritance

```
┌─────────────────────────────────────────────────────────────────────┐
│              NESTED ACCESS POLICIES                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   DEFAULT RULES                                                     │
│   ─────────────                                                     │
│                                                                      │
│   • Parent can access all descendants                              │
│   • Child cannot access parent (unless granted)                    │
│   • Siblings cannot access each other (unless granted)             │
│   • ROOT can access everything                                      │
│                                                                      │
│                                                                      │
│   EXPLICIT GRANTS PROPAGATE                                         │
│   ─────────────────────────                                         │
│                                                                      │
│   If Navigator grants Observer access:                              │
│   • Observer can access Navigator's interior                       │
│   • Observer can access children's interiors (cascade option)      │
│                                                                      │
│   LINK interior_access(#nav, #observer) { cascade: true }          │
│   -- Observer can access nav and all of nav's sub-worlds          │
│                                                                      │
│   LINK interior_access(#nav, #observer) { cascade: false }         │
│   -- Observer can access nav but not nav's sub-worlds             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part XIV: Federation Integration

## 14.1 Kernels as Roots

In federation, each kernel is a root:

```
┌─────────────────────────────────────────────────────────────────────┐
│                FEDERATION AND WORLDS                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   KERNEL A                          KERNEL B                        │
│   ────────                          ────────                        │
│                                                                      │
│   ROOT_A                            ROOT_B                          │
│     │                                 │                              │
│     ├── World 1                       ├── World 3                   │
│     │                                 │                              │
│     └── World 2                       └── World 4                   │
│                                                                      │
│                                                                      │
│   Each kernel is independent.                                       │
│   Federation bridges roots, not worlds.                             │
│   Worlds exist WITHIN a kernel.                                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 14.2 Cross-Kernel World References

A world can have projections of entities from other kernels:

```
interior: ontology {
  node RemoteLandmarkRef {
    kernel: String [required]       -- which kernel
    represents: ID [required]       -- entity ID in that kernel
    sync_id: String?                -- federation sync ID
    last_synced: Timestamp?
  }
}

-- Reference to entity in another kernel
SPAWN ref: RemoteLandmarkRef IN SELF {
  kernel = "kernel_b",
  represents = #landmark_in_b,
  sync_id = "sync_123"
}
```

## 14.3 Federation Watches

Watching across kernels for cross-kernel perception:

```
-- Watch remote kernel (via federation bridge)
WATCH Landmark IN REMOTE("kernel_b")
WHERE Landmark.shared = true
[mode: watch]
ON_CREATE(lm):
  SPAWN lr: RemoteLandmarkRef IN SELF {
    kernel = "kernel_b",
    represents = lm.id,
    sync_id = lm.sync_id
  }
```

---

# Part XV: Layer 0 Extensions

## 15.1 New Node Types

```
node _Interior [sealed] {
  -- Represents a world's interior
  owner: ID [required]            -- the exterior node
  time_mode: String = "shared"    -- "shared", "independent", "ratio"
  time_ratio: Int?                -- if mode = "ratio", ticks once per N parent ticks
  inherit_constraints: Bool = false
  inherit_rules: Bool = false
}

node _InteriorType [sealed] {
  -- A type declared in an interior
  interior: _Interior [required]
  -- inherits from _NodeType
}

node _InteriorEdgeType [sealed] {
  -- An edge type declared in an interior
  interior: _Interior [required]
  -- inherits from _EdgeType
}
```

## 15.2 New Edge Types

```
edge _has_interior(
  exterior: any,
  interior: _Interior
) {
  -- exterior node has this interior
}

edge _interior_declares_type(
  interior: _Interior,
  type: _InteriorType | _InteriorEdgeType
) {
  -- interior declares this type
}

edge _interior_declares_constraint(
  interior: _Interior,
  constraint: _ConstraintDef
) {
  -- interior-local constraint
}

edge _interior_declares_rule(
  interior: _Interior,
  rule: _RuleDef
) {
  -- interior-local rule
}

edge _interior_parent(
  child: _Interior,
  parent: _Interior
) {
  -- nesting relationship
}
```

## 15.3 Policy Extensions

```
edge interior_read_access(
  interior: _Interior | any,  -- the world (or its exterior)
  grantee: any                -- who gets access
) {
  allowed_types: List<String>?  -- optional type filter
  expires_at: Timestamp?        -- optional expiration
  cascade: Bool = false         -- apply to sub-worlds?
}

edge interior_write_access(
  interior: _Interior | any,
  grantee: any
) {
  allowed_types: List<String>?
  allowed_operations: List<String>?  -- ["SPAWN", "SET", ...]
  expires_at: Timestamp?
  cascade: Bool = false
}

edge interior_meta_access(
  interior: _Interior | any,
  grantee: any
) {
  level: String [required]  -- "read", "write", "admin"
  expires_at: Timestamp?
}
```

---

# Part XVI: Complete Grammar Extensions

```ebnf
(* World Declaration *)
WorldDecl = "node" Identifier "[" "has_interior" "]" "{" 
            ExteriorAttrs InteriorDecl? "}"

InteriorDecl = "interior" ":" "ontology" InteriorOptions? "{" OntologyBody "}"

InteriorOptions = "[" InteriorOption ("," InteriorOption)* "]"

InteriorOption = 
    "time" ":" TimeConfig
  | "inherit_constraints" ":" Bool
  | "inherit_rules" ":" Bool

TimeConfig = "shared" | "independent" | "ratio" "(" IntLiteral ")"  -- ratio(N): tick once per N parent ticks

(* Scope Specifiers *)
ScopeSpec = 
    "IN" "ROOT"
  | "IN" "SELF"
  | "IN" "PARENT"
  | "IN" NodeRef
  | "IN" Variable
  | "IN" "REMOTE" "(" StringLiteral ")"
  | "IN" "ancestor" "(" IntLiteral ")"
  | "IN" "ancestor" "(" StringLiteral ")"

(* Extended Statements *)
MatchStmt = "MATCH" Pattern ScopeSpec? WhereClause? ReturnClause

SpawnStmt = "SPAWN" Variable ":" TypeRef AttrBlock? ScopeSpec?

LinkStmt = "LINK" EdgePattern AttrBlock? ScopeSpec?

(* Projection Operations *)
InternalizeStmt = "INTERNALIZE" TypeRef ScopeSpec? WhereClause? "AS" TypeRef ScopeSpec?

ResolveExpr = "RESOLVE" "(" Expr ")" ScopeSpec?

(* Scope Context Functions *)
ScopeContextFunc =
    "current_scope" "(" ")"
  | "target_scope" "(" ")"
  | "parent_of" "(" Expr ")"
  | "is_interior" "(" Expr ")"
  | "owner_of" "(" Expr ")"

(* Watch Extensions *)
WatchStmt = "WATCH" Pattern ScopeSpec? 
            WatchOptions? WatchHandlers?

WatchHandlers = 
    OnCreateHandler? OnUpdateHandler? OnDeleteHandler?

OnCreateHandler = "ON_CREATE" "(" Variable ")" ":" ActionBlock
OnUpdateHandler = "ON_UPDATE" "(" Variable ")" ":" ActionBlock
OnDeleteHandler = "ON_DELETE" "(" Variable ")" ":" ActionBlock

(* Sandbox Constructs *)
IsolatedBlock = "WITH" "ISOLATED" Block

ScopedBlock = "WITH" "SCOPE" "(" ScopePermissions ")" Block

ScopePermissions = ScopePermission ("," ScopePermission)*

ScopePermission =
    "can_query" ":" ScopeTarget
  | "can_mutate" ":" ScopeTarget

ScopeTarget = "SELF" | "ROOT" | "NONE" | "[" TypeList "]" ScopeSpec?

(* Interior Management *)
ResetInteriorStmt = "RESET" "INTERIOR" NodeRef

TickStmt = 
    "TICK"
  | "TICK" IntLiteral
  | "TICK" ScopeSpec
  | "TICK" "ALL"
  | "TICK" "CHILDREN"
```

---

# Part XVII: Examples

## 17.1 Navigator Agent

```
-- Define agent type with hyperedge and higher-order edge
node Navigator [has_interior] {
  name: String [required]
  status: String = "idle"
  
  interior: ontology [time: shared] {
    -- Core knowledge types
    node LandmarkRef {
      represents: ID [required]
      local_name: String?
      cached_position: List<Float>?
      confidence: Float [>= 0, <= 1] = 0.5
      last_seen: Timestamp?
    }
    
    node Route {
      name: String
      estimated_distance: Float?
    }
    
    -- Hyperedge: connects multiple landmarks via a route
    -- (ternary edge: start, waypoint, end)
    edge via(LandmarkRef, LandmarkRef, LandmarkRef) {
      order: Int
    }
    
    -- Higher-order edge: confidence about a route connection
    edge route_confidence(edge<via>, level: Float, assessed_at: Timestamp)
    
    -- Binary edges
    edge visible_from(LandmarkRef, LandmarkRef) { distance: Float }
    
    -- Local rules
    rule decay_landmark_confidence [priority: 5]:
      lr: LandmarkRef
      WHERE logical_time() - lr.last_seen_tick > 100
        AND lr.confidence > 0.1
      =>
      SET lr.confidence = lr.confidence * 0.99
    
    rule forget_low_confidence [priority: 1]:
      lr: LandmarkRef WHERE lr.confidence < 0.05
      =>
      KILL lr
    
    -- Constraint
    constraint confidence_bounds:
      lr: LandmarkRef => lr.confidence >= 0 AND lr.confidence <= 1
  }
}

-- Create navigator
SPAWN nav: Navigator { name = "Explorer" }

-- Navigator's sensor (watch on ROOT)
WATCH Landmark IN ROOT
WHERE distance(Landmark, #nav, Physical) < 100
[mode: watch]
ON_CREATE(lm):
  SPAWN lr: LandmarkRef IN #nav { 
    represents = lm.id, 
    cached_position = lm.position,
    last_seen = now(),
    confidence = 0.9
  }
ON_UPDATE(lm):
  MATCH lr: LandmarkRef IN #nav WHERE lr.represents = lm.id
  SET lr.cached_position = lm.position, lr.last_seen = now()

-- Navigator builds route knowledge (from within agent's context)
MATCH lr1: LandmarkRef, lr2: LandmarkRef, lr3: LandmarkRef IN SELF
WHERE lr1.local_name = "Tower" AND lr3.local_name = "Bridge"
LINK via(lr1, lr2, lr3) { order = 1 } AS v
LINK route_confidence(v) { level = 0.8, assessed_at = now() }
```

## 17.2 Multi-Tenant Application

```
-- Tenant type
node Tenant [has_interior] {
  name: String [required]
  plan: String = "basic"
  
  interior: ontology {
    -- Each tenant has isolated data
    node Document {
      title: String [required]
      content: String
      created_at: Timestamp = now()
    }
    
    node Folder {
      name: String [required]
    }
    
    edge contains(Folder, Document | Folder)
    
    -- Tenant-specific rules
    rule auto_organize:
      d: Document WHERE d.folder = null
      =>
      LINK contains(#inbox, d)  -- #inbox is tenant's inbox folder
  }
}

-- Create tenants
SPAWN acme: Tenant { name = "Acme Corp", plan = "enterprise" }
SPAWN startup: Tenant { name = "StartupXYZ", plan = "basic" }

-- Each tenant's data is isolated
SPAWN doc1: Document IN #acme { title = "Acme Secret" }
SPAWN doc2: Document IN #startup { title = "Startup Plan" }

-- Query isolation
MATCH d: Document IN #acme         -- returns: [doc1]
MATCH d: Document IN #startup      -- returns: [doc2]
MATCH d: Document                  -- from ROOT: returns nothing (no Document type in ROOT)
```

## 17.3 Simulation Sandbox

```
-- Sandbox for running untrusted simulations
node Sandbox [has_interior] {
  name: String
  owner: ID [required]
  
  interior: ontology [time: independent, inherit_constraints: false] {
    -- Clean environment for simulation
    node Particle {
      position: List<Float>
      velocity: List<Float>
      mass: Float
    }
    
    space SimSpace [dimensions: 3, metric: euclidean]
    
    rule physics_step:
      p: Particle
      =>
      SET p.position = vec_add(p.position, p.velocity)
  }
}

-- Create sandbox
SPAWN sim: Sandbox { name = "Physics Test", owner = current_actor() }

-- Run simulation (isolated)
SPAWN p1: Particle IN #sim { position = [0,0,0], velocity = [1,0,0], mass = 1 }
SPAWN p2: Particle IN #sim { position = [10,0,0], velocity = [-1,0,0], mass = 1 }

-- Advance simulation time independently
TICK IN #sim
TICK IN #sim
TICK IN #sim

-- Query results
MATCH p: Particle IN #sim
RETURN p.position
-- Returns updated positions after 3 ticks

-- Cannot affect ROOT from sandbox
SPAWN x: SomeRootType IN ROOT  -- from sandbox context: DENIED
```

---

# Part XVIII: Error Model

## 18.1 Error Categories

Interior and scope errors are grouped into categories:

| Category | Code Range | Description |
|----------|------------|-------------|
| Scope | E7001-E7010 | Scope resolution and access errors |
| Interior | E7011-E7020 | Interior declaration and configuration errors |
| Time | E7021-E7030 | Scoped timing errors |
| Policy | E7031-E7040 | Cross-scope access policy errors |

## 18.2 Scope Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E7001 | SCOPE_NOT_FOUND | Referenced scope doesn't exist | `SCOPE_NOT_FOUND: Scope '{ref}' does not exist` |
| E7002 | INVALID_SCOPE_REF | Malformed scope reference | `INVALID_SCOPE_REF: '{ref}' is not a valid scope reference` |
| E7003 | NOT_AN_INTERIOR | Node doesn't have an interior | `NOT_AN_INTERIOR: Node '{ref}' does not have an interior` |
| E7004 | TYPE_NOT_IN_SCOPE | Type not declared in target scope | `TYPE_NOT_IN_SCOPE: Type '{type}' is not declared in scope '{scope}'` |

## 18.3 Interior Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E7011 | INVALID_TIME_CONFIG | Invalid time configuration | `INVALID_TIME_CONFIG: '{value}' is not a valid time configuration` |
| E7012 | INVALID_RATIO | Ratio must be positive integer | `INVALID_RATIO: Time ratio must be positive integer, got {value}` |
| E7013 | NESTING_DEPTH_EXCEEDED | Too many nested interiors | `NESTING_DEPTH_EXCEEDED: Maximum nesting depth {limit} exceeded` |
| E7014 | INTERIOR_ALREADY_EXISTS | Node already has an interior | `INTERIOR_ALREADY_EXISTS: Node '{ref}' already has an interior` |

## 18.4 Scoped Time Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E7021 | TICK_SCOPE_NOT_FOUND | TICK target scope doesn't exist | `TICK_SCOPE_NOT_FOUND: Cannot tick scope '{ref}' - not found` |
| E7022 | TICK_NOT_INTERIOR | TICK target is not an interior | `TICK_NOT_INTERIOR: Cannot tick '{ref}' - not an interior` |
| E7023 | TICK_INDEPENDENT_ONLY | Can only manually tick independent interiors | `TICK_INDEPENDENT_ONLY: Cannot manually tick '{ref}' - not configured as independent` |
| E7024 | LOGICAL_TIME_SCOPE_ERROR | Invalid scope for logical_time() | `LOGICAL_TIME_SCOPE_ERROR: Cannot query logical_time for '{ref}'` |

## 18.5 Cross-Scope Policy Errors

| Code | Name | Condition | Message |
|------|------|-----------|---------|
| E7031 | CROSS_SCOPE_ACCESS_DENIED | No permission for cross-scope access | `CROSS_SCOPE_ACCESS_DENIED: Access to scope '{target}' denied for '{actor}'` |
| E7032 | INTERIOR_READ_DENIED | No read access to interior | `INTERIOR_READ_DENIED: Read access to '{interior}' denied` |
| E7033 | INTERIOR_WRITE_DENIED | No write access to interior | `INTERIOR_WRITE_DENIED: Write access to '{interior}' denied` |
| E7034 | INTERIOR_META_DENIED | No META access to interior | `INTERIOR_META_DENIED: META access to '{interior}' denied` |

---

# Part XIX: Versioning Considerations

## 19.1 v1 Anticipation

v1 implementations can prepare for interiority by:

**Schema design:**
- Design node types that might need interiors (agents, tenants, sandboxes)
- Use ID attributes for cross-entity references (projection pattern)
- Avoid edges that would need to cross future scope boundaries

**Query patterns:**
- Structure queries to be scope-local where possible
- Parameterize scope-dependent logic

## 19.2 v2 Implementation

v2 introduces the full interiority system:

**Core features:**
- `[has_interior]` node modifier
- `interior: ontology { ... }` declaration
- `IN` keyword for scope targeting
- Time configuration: `shared`, `ratio(N)`, `independent`
- Access policies for cross-scope operations

**Syntax additions:**
- `MATCH ... IN #scope`
- `SPAWN ... IN #scope`
- `TICK IN #scope`
- `logical_time(SCOPE)`

## 19.3 v2+ Extensions

Future versions may extend with:

**Advanced nesting:**
- Deeper nesting limits
- Cross-branch interior references

**Performance optimizations:**
- Interior-level caching
- Lazy interior loading

**Federation (v3+):**
- Cross-kernel interior references
- Remote interior watches

---

# Part XX: Summary

## 20.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **World** | A node with an interior — its own ontology, data, rules, constraints |
| **Interior** | The inside of a world — isolated namespace with local types |
| **Exterior** | The outside of a world — visible as a node in parent scope |
| **Projection** | Interior node representing an external entity via `represents: ID` |
| **Scope** | The context in which queries/mutations execute |
| **IN keyword** | Specifies target scope for operations |
| **Sensor** | Watch on external scope for perception |
| **Actuator** | Mutation to external scope for action |

## 20.2 Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Edges stay inside** | Cross-boundary references use ID attributes, not edges |
| **Perception is modeling** | Projections can be stale; that's correct |
| **Action through ROOT** | Worlds affect each other via shared medium |
| **Policies gate access** | No pre-declared sensors/actuators |
| **IN makes scope explicit** | No ambiguity about query/mutation target |
| **Self-access always allowed** | Worlds can always access own interior |
| **Parent has authority** | Parents can access children's interiors |

## 20.3 Integration Points

| MEW Feature | Integration |
|-------------|-------------|
| **Ontology** | Interior has local ontology; types can shadow |
| **Policies** | Scope-aware policies; cross-scope grants |
| **META** | Modify own ontology dynamically |
| **Watches** | Sensors for perception; projection maintenance |
| **Time** | Interior as time domain; configurable tick relationship (ratio = slower) |
| **Space** | Local spaces; reference parent spaces |
| **Rules** | Scoped execution; actuator pattern for external effects |
| **Constraints** | Scoped enforcement; optional inheritance |
| **Higher-Order** | Works within scope; no cross-scope edges |
| **Federation** | Kernels as roots; cross-kernel projections |

## 20.4 Remaining Open Questions

| Question | Status |
|----------|--------|
| **Performance of nested worlds** | Needs benchmarking; may need optimization hints |
| **Garbage collection of stale projections** | Policy decision per use case |
| **Cross-scope transactions** | Atomicity guarantees when affecting multiple scopes |
| **Debugging nested execution** | Tooling for tracing cross-scope operations |
| **Migration of existing data into worlds** | API for "wrapping" existing nodes |

---

*End of MEW Interiority Capability*
