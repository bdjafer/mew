# MEW Spatial Extension

**Version:** 0.1
**Status:** Proposal
**Scope:** Metric embeddings for graph-structured data

---

# Part I: Context

## 1.1 The Gap

MEW models reality as a higher-order hypergraph: entities, relations, relations about relations. This captures *structure* — what connects to what, what claims exist about what.

But reality also has *extension*. Things exist somewhere. Proximity matters. Distance constrains interaction.

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   HYPERGRAPH (Structure)              SPACE (Extension)             │
│   ──────────────────────              ─────────────────             │
│                                                                      │
│   "Alice knows Bob"                   "Alice is 3m from Bob"        │
│   "Event A causes Event B"            "Event A is here, B is there" │
│   "Claim X supports Claim Y"          "Concept X near Concept Y"    │
│                                                                      │
│   Topology                            Geometry                       │
│   Discrete                            Continuous (usually)          │
│   What relates                        Where things are              │
│                                                                      │
│                    ┌─────────────────────┐                          │
│                    │                     │                          │
│                    │   BOTH ARE NEEDED   │                          │
│                    │                     │                          │
│                    └─────────────────────┘                          │
│                                                                      │
│   Structure without extension:        Extension without structure:  │
│   • Cannot express "nearby"           • Cannot express "causes"     │
│   • Cannot scope by proximity         • Cannot express "about"      │
│   • Cannot model embodiment           • Just a point cloud          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 1.2 What Is a Space?

A **space** is a set of positions with a notion of distance.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SPACE ANATOMY                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   COORDINATES                                                       │
│   Points in the space. Where an entity can be.                      │
│                                                                      │
│       (x, y)              -- 2D                                     │
│       (x, y, z)           -- 3D                                     │
│       (v₁, v₂, ..., vₙ)   -- n-dimensional embedding               │
│       (lat, lon)          -- spherical                              │
│       (row, col)          -- discrete grid                          │
│                                                                      │
│   METRIC                                                            │
│   How to measure distance between coordinates.                      │
│                                                                      │
│       Euclidean:   √(Σ(aᵢ - bᵢ)²)                                   │
│       Manhattan:   Σ|aᵢ - bᵢ|                                       │
│       Cosine:      1 - (a·b)/(|a||b|)                               │
│       Discrete:    hop count                                        │
│                                                                      │
│   PROPERTIES                                                        │
│                                                                      │
│       Dimensions:  how many coordinates                             │
│       Bounded:     finite or infinite extent                        │
│       Discrete:    integer or real coordinates                      │
│       Directed:    is there a preferred direction? (time)           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 1.3 Why Multiple Spaces?

An entity can be embedded in several spaces simultaneously:

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│                           ┌─────────┐                               │
│                           │  Alice  │                               │
│                           └────┬────┘                               │
│                                │                                     │
│           ┌────────────────────┼────────────────────┐               │
│           │                    │                    │               │
│           ▼                    ▼                    ▼               │
│   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐        │
│   │   Physical    │   │    Social     │   │   Semantic    │        │
│   │    Space      │   │    Space      │   │    Space      │        │
│   │               │   │               │   │               │        │
│   │  (3.2, 7.1,   │   │  (embedding   │   │  (concept     │        │
│   │   0.0)        │   │   vector)     │   │   embedding)  │        │
│   │               │   │               │   │               │        │
│   │  "where she   │   │  "who she's   │   │  "what she    │        │
│   │   is"         │   │   close to"   │   │   represents" │        │
│   └───────────────┘   └───────────────┘   └───────────────┘        │
│                                                                      │
│   Different spaces, different meanings of "near"                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

| Space | What "Near" Means | Typical Use |
|-------|-------------------|-------------|
| Physical | Spatially close | Simulation, robotics, games |
| Social | Relationally similar | Community detection, influence |
| Semantic | Conceptually similar | Knowledge organization, search |
| Temporal | Close in time | Event correlation, causality |
| Economic | Similar resource position | Market dynamics, inequality |

## 1.4 Relationship to Time

Time is a space — it has coordinates (timestamps) and a metric (duration). But time has special properties:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     TIME vs OTHER SPACES                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                    PHYSICAL SPACE          TIME                     │
│                    ──────────────          ────                     │
│                                                                      │
│   Direction        None (symmetric)        Past → Future            │
│                                                                      │
│   Privileged       None                    "Now" exists             │
│   Point                                                              │
│                                                                      │
│   What Changes     Positions change        Graph itself changes     │
│                    over time               through time             │
│                                                                      │
│   Query            Any region              Past is frozen           │
│   Access           accessible              Future unknown           │
│                                                                      │
│   Causality        None                    Built-in constraint      │
│                                                                      │
│                                                                      │
│   Time is the META-SPACE:                                           │
│   The graph evolves THROUGH time.                                   │
│   The graph is IN physical space.                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Time remains special in MEW (ticks, `now()`, mutation ordering). The spatial extension treats time as a space for query purposes while preserving its special execution semantics.

---

# Part II: Motivation

## 2.1 Use Cases Requiring Space

**Embodied Agents**
```
Agents exist in a physical world.
They perceive nearby entities.
They navigate through space.
They interact with what they can reach.

Without space: "Agent perceives Entity" (which entities? all of them?)
With space:    "Agent perceives Entity WHERE distance < perception_radius"
```

**Simulation**
```
Particles interact by proximity.
Social influence decays with distance.
Information diffuses through space.
Resources are distributed spatially.

Without space: O(n²) interaction checks
With space:    O(n) with locality scoping
```

**Learned Embeddings**
```
Neural networks produce embeddings.
Similar items have nearby embeddings.
Semantic search = nearest neighbor query.
Clustering = spatial density analysis.

Without space: Embeddings are opaque vectors
With space:    Embeddings are queryable positions
```

**Multi-Scale Modeling**
```
Individual → Neighborhood → City → Region → Planet
Each scale has its own spatial granularity.
Zoom in: more detail, smaller distances.
Zoom out: aggregation, larger distances.
```

## 2.2 What MEW Cannot Do Today

| Need | Current MEW | With Spatial Extension |
|------|-------------|------------------------|
| "All entities within 10m of X" | Manual distance edges, O(n²) | Native range query, indexed |
| "5 nearest neighbors of X" | Cannot express directly | `nearest(X, 5)` |
| "Rules only for nearby pairs" | Full cross-product match | Locality-scoped execution |
| "Position changes continuously" | Awkward discrete updates | Optimized position mutation |
| "Distance-weighted influence" | Compute in rule body | `distance()` in expressions |

## 2.3 Design Goals

| Goal | Meaning |
|------|---------|
| **Unified model** | Spaces are graph structure, not separate system |
| **Multiple spaces** | Entities can exist in many spaces simultaneously |
| **Expressive queries** | Range, nearest, distance in WHERE/RETURN |
| **Scoped execution** | Rules can be locality-bounded |
| **Efficient** | Spatial indexes, hardware acceleration |
| **Progressive** | Start simple, add dimensions/complexity as needed |

---

# Part III: Requirements

## 3.1 Space Declaration

Developers must be able to declare spaces with:

| Property | Description | Examples |
|----------|-------------|----------|
| Name | Unique identifier | `Physical`, `Social`, `Semantic` |
| Dimensions | Coordinate count | 2, 3, 128, 768 |
| Metric | Distance function | Euclidean, cosine, Manhattan |
| Bounds | Finite or infinite | `[(0,0), (100,100)]` or unbounded |
| Discrete | Integer or real | Grid vs continuous |

## 3.2 Positioning Entities

Developers must be able to:

- Place an entity in a space at a coordinate
- Move an entity to a new coordinate
- Remove an entity from a space
- Query an entity's position
- Have an entity in multiple spaces

## 3.3 Spatial Queries

The query language must support:

| Query Type | Description |
|------------|-------------|
| **Distance** | Distance between two positioned entities |
| **Range** | All entities within radius of a point |
| **Nearest** | K closest entities to a point |
| **Within** | Test if entity is in a region |
| **Cross-space** | Combine spatial predicates across spaces |

## 3.4 Spatial Constraints

Constraints must be able to reference:

- Distance between entities
- Position bounds
- Spatial relationships (inside, outside, between)
- Cross-space correlations

## 3.5 Spatial Rules

Rules must support:

| Feature | Description |
|---------|-------------|
| **Spatial conditions** | `WHERE distance(a, b) < R` |
| **Locality scoping** | Only consider nearby pairs |
| **Position mutations** | `SET position(e) = ...` in productions |
| **Spatial triggers** | Fire when entity enters/exits region |

## 3.6 Time Integration

The spatial extension must:

- Treat time as a space for query purposes
- Preserve time's special execution semantics
- Allow temporal-spatial combined queries
- Support "where was X at time T" queries (with history)

---

# Part IV: Developer Experience

## 4.1 Declaring Spaces

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SPACE DECLARATION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Simple 2D world                                                 │
│  space GameWorld [dimensions: 2, metric: euclidean] {               │
│    bounds: [(0, 0), (1000, 1000)]                                  │
│  }                                                                   │
│                                                                      │
│  -- 3D physical space, unbounded                                    │
│  space Physical [dimensions: 3, metric: euclidean]                  │
│                                                                      │
│  -- High-dimensional embedding space                                │
│  space SemanticEmbedding [dimensions: 768, metric: cosine]          │
│                                                                      │
│  -- Discrete grid                                                   │
│  space ChessBoard [dimensions: 2, discrete: true] {                 │
│    bounds: [(0, 0), (7, 7)]                                        │
│  }                                                                   │
│                                                                      │
│  -- Spherical (Earth surface)                                       │
│  space Geographic [dimensions: 2, metric: haversine]                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.2 Positioning Entities

```
┌─────────────────────────────────────────────────────────────────────┐
│                      POSITIONING                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Place entity in space                                           │
│  PLACE #alice IN Physical AT (10.0, 20.0, 0.0)                     │
│  PLACE #alice IN SemanticEmbedding AT $embedding_vector             │
│                                                                      │
│  -- Move entity                                                     │
│  MOVE #alice IN Physical TO (15.0, 20.0, 0.0)                      │
│                                                                      │
│  -- Remove from space                                               │
│  DISPLACE #alice FROM Physical                                      │
│                                                                      │
│  -- Query position                                                  │
│  MATCH e: Entity                                                    │
│  RETURN e, position(e, Physical)                                    │
│                                                                      │
│  -- Alternative: position as explicit edge (more MEW-native)        │
│  LINK positioned(#alice, #Physical) { at: [10.0, 20.0, 0.0] }      │
│  SET positioned(#alice, #Physical).at = [15.0, 20.0, 0.0]          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.3 Spatial Queries

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SPATIAL QUERIES                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Distance                                                        │
│  MATCH a: Agent, b: Agent                                           │
│  RETURN a, b, distance(a, b, Physical)                              │
│                                                                      │
│  -- Range query: all within radius                                  │
│  MATCH e: Entity                                                    │
│  WHERE within(e, center: (50, 50, 0), radius: 10, space: Physical) │
│  RETURN e                                                           │
│                                                                      │
│  -- Range from another entity                                       │
│  MATCH a: Agent, e: Entity                                          │
│  WHERE a.id = #player                                               │
│    AND distance(a, e, Physical) < 20                                │
│  RETURN e AS nearby_entities                                        │
│                                                                      │
│  -- K nearest neighbors                                             │
│  MATCH e: Entity                                                    │
│  WHERE e.type = "Resource"                                          │
│  ORDER BY distance(#player, e, Physical)                            │
│  LIMIT 5                                                            │
│  RETURN e AS closest_resources                                      │
│                                                                      │
│  -- Nearest with dedicated syntax                                   │
│  MATCH e: Entity                                                    │
│  WHERE e IN nearest(#player, 5, Physical)                           │
│  RETURN e                                                           │
│                                                                      │
│  -- Cross-space query                                               │
│  MATCH a: Person, b: Person                                         │
│  WHERE distance(a, b, Physical) < 10                                │
│    AND distance(a, b, Social) > 0.8                                 │
│  RETURN a, b  -- physically close, socially distant                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.4 Spatial Constraints

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SPATIAL CONSTRAINTS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Entities must stay in bounds                                    │
│  constraint in_bounds:                                              │
│    e: Entity, positioned(e, #GameWorld)                             │
│    => within_bounds(e, GameWorld)                                   │
│                                                                      │
│  -- Minimum separation                                              │
│  constraint no_overlap:                                             │
│    a: Physical, b: Physical                                         │
│    WHERE a.id != b.id                                               │
│    => distance(a, b, Physical) >= a.radius + b.radius               │
│                                                                      │
│  -- Spatial-structural correlation                                  │
│  constraint communication_range:                                    │
│    a: Agent, b: Agent, communicates(a, b)                          │
│    => distance(a, b, Physical) < 100                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.5 Spatial Rules

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SPATIAL RULES                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Locality-scoped rule (efficient)                                │
│  rule interaction [locality: 50, space: Physical]:                  │
│    a: Particle, b: Particle                                         │
│    -- implicitly: WHERE distance(a, b, Physical) < 50               │
│    =>                                                               │
│    LINK interacts(a, b) { strength = 1 / distance(a, b, Physical) } │
│                                                                      │
│  -- Proximity creates relationship                                  │
│  rule proximity_acquaintance [locality: 5, space: Physical]:        │
│    a: Person, b: Person                                             │
│    WHERE NOT EXISTS(knows(a, b))                                    │
│    =>                                                               │
│    LINK knows(a, b) { strength = 0.1 }                              │
│                                                                      │
│  -- Movement rule                                                   │
│  rule apply_velocity:                                               │
│    e: Entity                                                        │
│    WHERE e.velocity != null                                         │
│    =>                                                               │
│    MOVE e IN Physical BY e.velocity * tick_duration()               │
│                                                                      │
│  -- Region entry trigger                                            │
│  rule entered_zone [trigger: enter, region: #danger_zone]:          │
│    e: Agent                                                         │
│    =>                                                               │
│    SPAWN a: Alert { message = e.name ++ " entered danger zone" }    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.6 Spatial + Graph Combined

```
┌─────────────────────────────────────────────────────────────────────┐
│                  COMBINED SPATIAL + STRUCTURAL                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  -- Find nearby entities of specific relation                       │
│  MATCH                                                              │
│    me: Player,                                                      │
│    target: NPC,                                                     │
│    quest: Quest,                                                    │
│    gives_quest(target, quest)                                       │
│  WHERE me.id = #current_player                                      │
│    AND distance(me, target, Physical) < 50                          │
│    AND NOT EXISTS(completed(me, quest))                             │
│  RETURN target, quest                                               │
│                                                                      │
│  -- Spatial traversal                                               │
│  WALK FROM #start_location                                          │
│  FOLLOW connected_to                                                │
│  WHERE distance(node, #goal, Physical) < previous_distance          │
│  UNTIL node = #goal OR depth > 100                                  │
│  RETURN PATH                                                        │
│                                                                      │
│  -- Higher-order + spatial                                          │
│  MATCH                                                              │
│    e1: Event, e2: Event,                                            │
│    causes(e1, e2) AS c,                                             │
│    confidence(c, level)                                             │
│  WHERE distance(e1, e2, Physical) < 100                             │
│    AND level > 0.8                                                  │
│  RETURN e1, e2, distance(e1, e2, Physical) AS spatial_gap           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part V: Integration with MEW

## 5.1 Spaces as Graph Structure

Spaces and positions are represented as graph elements, not a separate system:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GRAPH REPRESENTATION                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                     ┌──────────────────┐                            │
│                     │     _Space       │                            │
│                     │  ────────────    │                            │
│                     │  name: "Physical"│                            │
│                     │  dimensions: 3   │                            │
│                     │  metric: "euclid"│                            │
│                     └────────▲─────────┘                            │
│                              │                                       │
│                              │ _positioned                          │
│                              │ { at: [10, 20, 0] }                  │
│                              │                                       │
│                     ┌────────┴─────────┐                            │
│                     │     Entity       │                            │
│                     │  ────────────    │                            │
│                     │  name: "Alice"   │                            │
│                     └────────┬─────────┘                            │
│                              │                                       │
│                              │ _positioned                          │
│                              │ { at: [0.3, -0.2, ..., 0.7] }       │
│                              │                                       │
│                     ┌────────▼─────────┐                            │
│                     │     _Space       │                            │
│                     │  ────────────    │                            │
│                     │  name: "Semantic"│                            │
│                     │  dimensions: 768 │                            │
│                     │  metric: "cosine"│                            │
│                     └──────────────────┘                            │
│                                                                      │
│   Same entity, two spaces, two positions                            │
│   All queryable as graph structure                                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Layer 0 Additions

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 0 EXTENSIONS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   NEW NODE TYPES                                                    │
│   ──────────────                                                    │
│                                                                      │
│   _Space [sealed]                                                   │
│     name: String [required, unique]                                 │
│     dimensions: Int [required, >= 1]                                │
│     metric: String [required]                                       │
│     directed: Bool = false                                          │
│     bounded: Bool = false                                           │
│     discrete: Bool = false                                          │
│     bounds_min: List<Float>?                                        │
│     bounds_max: List<Float>?                                        │
│                                                                      │
│   _Region [sealed]                                                  │
│     name: String?                                                   │
│     shape: String [required]  -- "sphere", "box", "polygon"        │
│     parameters: List<Float> [required]                              │
│                                                                      │
│                                                                      │
│   NEW EDGE TYPES                                                    │
│   ──────────────                                                    │
│                                                                      │
│   _positioned(entity: any, space: _Space)                          │
│     at: List<Float> [required]                                      │
│     -- coordinates in the space                                     │
│                                                                      │
│   _region_in(region: _Region, space: _Space)                       │
│     -- region belongs to space                                      │
│                                                                      │
│                                                                      │
│   The engine recognizes these specially and maintains               │
│   spatial indexes for efficient queries.                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.3 Expression Language Extensions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  EXPRESSION EXTENSIONS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SPATIAL FUNCTIONS                                                 │
│   ─────────────────                                                 │
│                                                                      │
│   distance(a, b)                 -- default/only space              │
│   distance(a, b, space)          -- specific space                  │
│   distance(a, point, space)      -- entity to coordinate            │
│                                                                      │
│   position(entity)               -- default space, returns coord    │
│   position(entity, space)        -- specific space                  │
│                                                                      │
│   within(entity, region)         -- is entity in region?            │
│   within(entity, center, radius, space)  -- sphere test             │
│                                                                      │
│   nearest(entity, k)             -- k nearest in default space      │
│   nearest(entity, k, space)      -- k nearest in specific space     │
│   nearest(point, k, space)       -- k nearest to coordinate         │
│                                                                      │
│   within_bounds(entity, space)   -- is entity in space bounds?      │
│                                                                      │
│                                                                      │
│   VECTOR OPERATIONS (for coordinate manipulation)                   │
│   ─────────────────                                                 │
│                                                                      │
│   vec_add(a, b)                  -- coordinate addition             │
│   vec_sub(a, b)                  -- coordinate subtraction          │
│   vec_scale(v, scalar)           -- scalar multiplication           │
│   vec_norm(v)                    -- magnitude                       │
│   vec_normalize(v)               -- unit vector                     │
│   vec_dot(a, b)                  -- dot product                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.4 Rule Modifiers

```
┌─────────────────────────────────────────────────────────────────────┐
│                    RULE MODIFIERS                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   [locality: <radius>]                                              │
│   [locality: <radius>, space: <space>]                              │
│                                                                      │
│     Scope rule to entity pairs within radius.                       │
│     Semantically equivalent to WHERE distance < radius.             │
│     Execution optimized via spatial index.                          │
│                                                                      │
│                                                                      │
│   [trigger: enter, region: <region>]                                │
│   [trigger: exit, region: <region>]                                 │
│                                                                      │
│     Fire when entity enters/exits region.                           │
│     Only meaningful for rules with single entity pattern.           │
│                                                                      │
│                                                                      │
│   EXAMPLE                                                           │
│   ───────                                                           │
│                                                                      │
│   -- Without locality: O(n²) pairs checked                          │
│   rule gravity:                                                     │
│     a: Massive, b: Massive                                          │
│     WHERE distance(a, b, Physical) < 1000                           │
│     => ...                                                          │
│                                                                      │
│   -- With locality: O(n) via spatial index                          │
│   rule gravity [locality: 1000, space: Physical]:                   │
│     a: Massive, b: Massive                                          │
│     => ...                                                          │
│                                                                      │
│   Same semantics. Different execution cost.                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.5 Time as Space

```
┌─────────────────────────────────────────────────────────────────────┐
│                      TIME AS SPACE                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Time is a built-in space with special properties:                 │
│                                                                      │
│   _Space {                                                          │
│     name: "Time",                                                   │
│     dimensions: 1,                                                  │
│     metric: "duration",                                             │
│     directed: true        ← special: past to future                │
│   }                                                                  │
│                                                                      │
│   TEMPORAL QUERIES (using spatial primitives)                       │
│                                                                      │
│   -- Events within time window                                      │
│   MATCH e: Event                                                    │
│   WHERE within(e, center: now(), radius: 1.hour, space: Time)      │
│   RETURN e                                                          │
│                                                                      │
│   -- Temporal distance                                              │
│   MATCH e1: Event, e2: Event, causes(e1, e2)                       │
│   RETURN e1, e2, distance(e1, e2, Time) AS time_gap                │
│                                                                      │
│   PRESERVED SPECIAL SEMANTICS                                       │
│                                                                      │
│   • now() returns privileged current position in Time               │
│   • logical_time() returns tick position                            │
│   • Mutations happen AT time positions                              │
│   • Causality constraints reference temporal order                  │
│   • TICK advances the "now" position                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.6 Subscription Integration

```
┌─────────────────────────────────────────────────────────────────────┐
│                  SPATIAL SUBSCRIPTIONS                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   -- Subscribe to entities entering a region                        │
│   SUBSCRIBE                                                         │
│     MATCH e: Entity                                                 │
│     WHERE within(e, #danger_zone)                                   │
│     [trigger: enter]                                                │
│     RETURN e, position(e, Physical)                                 │
│                                                                      │
│   -- Subscribe to nearby changes                                    │
│   SUBSCRIBE                                                         │
│     MATCH e: Entity                                                 │
│     WHERE distance(e, #my_agent, Physical) < 100                    │
│     [mode: watch]                                                   │
│     RETURN e                                                        │
│                                                                      │
│   -- Spatial window aggregation                                     │
│   SUBSCRIBE                                                         │
│     MATCH e: Entity                                                 │
│     WHERE within(e, center: (0, 0, 0), radius: 50, space: Physical)│
│     [window: tumbling(1s)]                                          │
│     RETURN COUNT(e), AVG(position(e, Physical))                     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.7 Authorization Integration

```
┌─────────────────────────────────────────────────────────────────────┐
│                  SPATIAL AUTHORIZATION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   -- Can only observe entities in perception range                  │
│   authorization perception_range:                                   │
│     ON MATCH(e: Entity)                                             │
│     ALLOW IF distance(e, current_actor(), Physical) < 100           │
│             OR owns(current_actor(), e)                             │
│                                                                      │
│   -- Can only modify nearby entities                                │
│   authorization interaction_range:                                  │
│     ON SET(e: Entity, _)                                            │
│     ALLOW IF distance(e, current_actor(), Physical) < 10            │
│             OR has_role(current_actor(), "admin")                   │
│                                                                      │
│   -- Region-based access                                            │
│   authorization zone_access:                                        │
│     ON MATCH(e: Entity)                                             │
│     ALLOW IF within(e, #public_zone)                                │
│             OR has_clearance(current_actor(), zone_of(e))           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VI: Scopes & Phases

## 6.1 Phase Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION PHASES                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   PHASE 1: Foundation                                               │
│   ───────────────────                                               │
│   • Space declaration (2D, 3D Euclidean)                           │
│   • Position edges                                                  │
│   • distance(), position() functions                                │
│   • Range queries (within radius)                                   │
│   • Basic spatial indexing                                          │
│                                                                      │
│   PHASE 2: Rich Queries                                             │
│   ─────────────────────                                             │
│   • K-nearest neighbor queries                                      │
│   • Region types (sphere, box, polygon)                            │
│   • Locality-scoped rules                                           │
│   • Enter/exit triggers                                             │
│   • Multiple metrics (Manhattan, Chebyshev)                        │
│                                                                      │
│   PHASE 3: High Dimensions                                          │
│   ────────────────────────                                          │
│   • High-dimensional spaces (embeddings)                            │
│   • Approximate nearest neighbor                                    │
│   • Cosine and other embedding metrics                              │
│   • Vector operations in expressions                                │
│                                                                      │
│   PHASE 4: Advanced                                                 │
│   ───────────────────                                               │
│   • Non-Euclidean spaces (spherical, hyperbolic)                   │
│   • Time as queryable space                                         │
│   • Spatial history (where was X at time T)                        │
│   • Cross-space projections                                         │
│                                                                      │
│   PHASE 5: Optimization                                             │
│   ─────────────────────                                             │
│   • GPU-accelerated spatial operations                              │
│   • Continuous position update optimization                         │
│   • Spatial partitioning for distributed execution                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 6.2 Phase 1 Scope (MVP)

What's in:
- `space` declaration with dimensions, metric (Euclidean only)
- `_positioned` edge type
- `distance(a, b)` and `distance(a, b, space)` functions
- `within(entity, center, radius, space)` function
- `position(entity)` and `position(entity, space)` functions
- `PLACE`, `MOVE`, `DISPLACE` statements
- Spatial predicates in WHERE clauses
- Basic spatial index (engine chooses structure)

What's out (deferred):
- K-nearest queries
- Locality-scoped rules
- Non-Euclidean metrics
- High-dimensional spaces
- Regions
- Enter/exit triggers
- GPU acceleration

## 6.3 Feature Dependencies

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FEATURE DEPENDENCY GRAPH                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                          ┌───────────────┐                          │
│                          │ GPU Spatial   │                          │
│                          │ Acceleration  │                          │
│                          └───────┬───────┘                          │
│                                  │                                   │
│                    ┌─────────────┴─────────────┐                    │
│                    │                           │                    │
│                    ▼                           ▼                    │
│           ┌───────────────┐          ┌───────────────┐             │
│           │  Locality     │          │    High-Dim   │             │
│           │  Scoped Rules │          │    Spaces     │             │
│           └───────┬───────┘          └───────┬───────┘             │
│                   │                          │                      │
│                   │         ┌────────────────┤                      │
│                   │         │                │                      │
│                   ▼         ▼                ▼                      │
│           ┌───────────────────┐     ┌───────────────┐              │
│           │    K-Nearest      │     │ Cosine Metric │              │
│           │    Queries        │     │               │              │
│           └─────────┬─────────┘     └───────┬───────┘              │
│                     │                       │                       │
│                     │         ┌─────────────┘                       │
│                     │         │                                     │
│                     ▼         ▼                                     │
│           ┌─────────────────────────────────────────┐              │
│           │            Spatial Index                 │              │
│           └─────────────────────┬───────────────────┘              │
│                                 │                                   │
│                                 ▼                                   │
│           ┌─────────────────────────────────────────┐              │
│           │     Range Queries (within radius)        │              │
│           └─────────────────────┬───────────────────┘              │
│                                 │                                   │
│                                 ▼                                   │
│           ┌─────────────────────────────────────────┐              │
│           │          distance() function             │              │
│           └─────────────────────┬───────────────────┘              │
│                                 │                                   │
│                                 ▼                                   │
│           ┌─────────────────────────────────────────┐              │
│           │    Space Declaration + Position Edge     │              │
│           └─────────────────────────────────────────┘              │
│                                                                      │
│   FOUNDATION                                                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VII: Hardware Acceleration

## 7.1 Computational Profiles

```
┌─────────────────────────────────────────────────────────────────────┐
│              OPERATION COMPUTATIONAL PROFILES                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   OPERATION              ACCESS PATTERN        BEST HARDWARE        │
│   ─────────              ──────────────        ─────────────        │
│                                                                      │
│   Graph traversal        Irregular             CPU                  │
│   (WALK, pattern match)  Pointer-chasing       (cache-sensitive)    │
│                                                                      │
│   Distance computation   Regular               GPU                  │
│   (many pairs)           Data-parallel         (SIMD/tensor cores)  │
│                                                                      │
│   Range query            Semi-regular          CPU or GPU           │
│   (spatial index)        Tree traversal        (depends on index)   │
│                                                                      │
│   K-NN (exact)           Irregular             CPU                  │
│                          Priority queue        (sequential)         │
│                                                                      │
│   K-NN (approximate)     Regular               GPU                  │
│   (LSH, HNSW)            Hash/vector ops       (highly parallel)    │
│                                                                      │
│   Position update        Regular               GPU                  │
│   (bulk)                 Embarrassingly        (trivially parallel) │
│                          parallel                                    │
│                                                                      │
│   Locality-scoped rule   Regular per cell      GPU                  │
│   (with spatial hash)    Parallel cells        (cell-parallel)      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.2 Acceleration Alignment

```
┌─────────────────────────────────────────────────────────────────────┐
│                 GPU ACCELERATION OPPORTUNITIES                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SPATIAL HASH / GRID PARTITIONING                                  │
│   ────────────────────────────────                                  │
│                                                                      │
│   Space divided into cells. Each cell processed in parallel.        │
│                                                                      │
│        ┌───┬───┬───┬───┐                                           │
│        │ A │   │ B │   │       A and B in different cells          │
│        ├───┼───┼───┼───┤       → Can process in parallel           │
│        │   │ C │ D │   │       C and D in same/adjacent cells      │
│        ├───┼───┼───┼───┤       → Process together (may interact)   │
│        │   │   │   │ E │                                            │
│        └───┴───┴───┴───┘                                           │
│                                                                      │
│   Locality-scoped rules become cell-parallel:                       │
│   • Partition entities by cell                                      │
│   • Process each cell on separate GPU thread                        │
│   • Handle cell boundaries (ghost zones)                            │
│                                                                      │
│                                                                      │
│   EMBEDDING OPERATIONS                                              │
│   ────────────────────                                              │
│                                                                      │
│   High-dimensional operations are tensor operations:                │
│   • Distance = vector subtraction + norm                            │
│   • Cosine = dot product + norms                                    │
│   • K-NN = batched distance + top-k selection                       │
│                                                                      │
│   These map directly to GPU tensor cores / TPU matrix units.        │
│                                                                      │
│                                                                      │
│   BULK POSITION UPDATES                                             │
│   ─────────────────────                                             │
│                                                                      │
│   position[i] += velocity[i] * dt                                   │
│                                                                      │
│   Pure SIMD. Millions of entities per millisecond on GPU.           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.3 Hybrid Execution Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                    HYBRID CPU/GPU EXECUTION                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                         QUERY                                │  │
│   └─────────────────────────┬───────────────────────────────────┘  │
│                             │                                       │
│                             ▼                                       │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                      QUERY PLANNER                           │  │
│   │                                                              │  │
│   │   Analyze query structure:                                   │  │
│   │   • Graph patterns → CPU                                    │  │
│   │   • Spatial predicates → GPU (if available)                 │  │
│   │   • Hybrid → Split and coordinate                           │  │
│   │                                                              │  │
│   └─────────────┬────────────────────────────┬──────────────────┘  │
│                 │                            │                      │
│                 ▼                            ▼                      │
│   ┌─────────────────────────┐  ┌─────────────────────────────┐    │
│   │         CPU             │  │           GPU                │    │
│   │                         │  │                              │    │
│   │  • Pattern matching     │  │  • Distance calculations    │    │
│   │  • Graph traversal      │  │  • Range filtering          │    │
│   │  • Constraint checking  │  │  • K-NN search              │    │
│   │  • Rule orchestration   │  │  • Position updates         │    │
│   │                         │  │  • Locality partitioning    │    │
│   └───────────┬─────────────┘  └──────────────┬──────────────┘    │
│               │                               │                     │
│               └───────────────┬───────────────┘                     │
│                               │                                     │
│                               ▼                                     │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │                    RESULT MERGE                              │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.4 Future: Quantum Considerations

```
┌─────────────────────────────────────────────────────────────────────┐
│                  QUANTUM ACCELERATION (SPECULATIVE)                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   POTENTIAL APPLICATIONS                                            │
│   ──────────────────────                                            │
│                                                                      │
│   Grover's Algorithm                                                │
│   • Unstructured spatial search: O(√n) vs O(n)                     │
│   • "Find any entity within region" without index                  │
│                                                                      │
│   Quantum Annealing                                                 │
│   • Spatial optimization (facility location, clustering)           │
│   • Constraint satisfaction with spatial components                │
│                                                                      │
│   Quantum Simulation                                                │
│   • Physical systems with quantum dynamics                         │
│   • Particle interactions, molecular simulation                    │
│                                                                      │
│                                                                      │
│   DESIGN IMPLICATIONS                                               │
│   ───────────────────                                               │
│                                                                      │
│   • Keep spatial operations as pure functions (no side effects)    │
│   • Support batch queries (quantum works on superpositions)        │
│   • Abstract backend selection (swap classical for quantum)        │
│   • Identify operations that are "quantum-ready"                   │
│                                                                      │
│   NOT BLOCKING: Design for classical. Quantum is bonus.            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VIII: Summary

## 8.1 What This Extension Provides

| Capability | Description |
|------------|-------------|
| **Space declaration** | Named coordinate systems with metrics |
| **Multi-embedding** | Entities in multiple spaces simultaneously |
| **Spatial queries** | Distance, range, nearest in expressions |
| **Locality scoping** | Rules bounded by spatial proximity |
| **Spatial constraints** | Distance and region predicates in constraints |
| **Time unification** | Time as queryable space (preserving special semantics) |
| **Hardware alignment** | Operations map to GPU acceleration patterns |

## 8.2 What This Extension Does Not Provide

| Explicitly Out of Scope | Rationale |
|-------------------------|-----------|
| Physics engine | MEW is structure, not simulation. Interface with physics engines instead. |
| Continuous integration | Discrete ticks approximate continuous. External solvers for precision. |
| Rendering | Spatial data can feed renderers but MEW doesn't render. |
| Collision detection | Can be built on spatial primitives but not built-in. |

## 8.3 Integration Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SPATIAL EXTENSION IN CONTEXT                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│                    ┌─────────────────────────────┐                  │
│                    │        APPLICATION          │                  │
│                    │    (Games, Agents, etc.)    │                  │
│                    └──────────────┬──────────────┘                  │
│                                   │                                  │
│                                   ▼                                  │
│   ┌───────────────────────────────────────────────────────────────┐│
│   │                         MEW ENGINE                             ││
│   │ ┌─────────────────────────────────────────────────────────┐   ││
│   │ │                    QUERY / MUTATION                      │   ││
│   │ │                                                          │   ││
│   │ │   MATCH, SPAWN, LINK, SET, WALK, SUBSCRIBE, ...         │   ││
│   │ │   + distance(), within(), nearest(), PLACE, MOVE        │   ││
│   │ │                                                          │   ││
│   │ └──────────────────────────┬──────────────────────────────┘   ││
│   │                            │                                   ││
│   │ ┌──────────────────────────┴──────────────────────────────┐   ││
│   │ │                     EXECUTION                            │   ││
│   │ │                                                          │   ││
│   │ │   Pattern Matching │ Spatial Index │ Constraint Check   │   ││
│   │ │   Rule Engine      │ Locality Scope│ Subscription       │   ││
│   │ │         │                  │               │             │   ││
│   │ │         └────────┬─────────┴───────────────┘             │   ││
│   │ │                  │                                       │   ││
│   │ └──────────────────┼───────────────────────────────────────┘   ││
│   │                    │                                           ││
│   │ ┌──────────────────┴───────────────────────────────────────┐   ││
│   │ │                      STORAGE                              │   ││
│   │ │                                                           │   ││
│   │ │   Graph Store │ Spatial Index │ Position Data │ History  │   ││
│   │ │                                                           │   ││
│   │ └───────────────────────────────────────────────────────────┘   ││
│   └───────────────────────────────────────────────────────────────┘│
│                                                                      │
│   Layer 0: _Space, _positioned, _Region (new)                       │
│   + existing: _NodeType, _EdgeType, _ConstraintDef, ...             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.4 Success Criteria

The spatial extension is successful if:

1. **Simple cases are simple**: 2D game with proximity interactions is straightforward to model
2. **Complex cases are possible**: High-dimensional embeddings, multiple spaces, cross-space queries work
3. **Performance scales**: Locality-scoped rules are O(n), not O(n²)
4. **Integration is seamless**: Spatial predicates compose with graph patterns naturally
5. **Hardware accelerates**: GPU backend provides measurable speedup for spatial operations

---

*End of MEW Spatial Extension Specification*