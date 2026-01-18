# MEW Edges

**Version:** 1.0
**Status:** Capability
**Scope:** Binary edges, hyperedges, higher-order edges, edge attributes

---

# Part I: The Core Insight

## 1.1 Relationships Are First-Class

Most graph systems treat edges as second-class: a pointer from A to B, maybe with a label. MEW treats relationships as **first-class entities** with:

- **Arbitrary arity** — Binary, ternary, or any number of participants
- **Attributes** — Properties on the relationship itself
- **Higher-order structure** — Edges about edges, recursively
- **Identity** — Every edge has an ID, can be referenced, bound in patterns

This enables modeling that other systems cannot express cleanly.

## 1.2 What This Unlocks

| Capability | What It Enables |
|------------|-----------------|
| **Hyperedges** | N-ary relationships without reification |
| **Higher-order edges** | Confidence, provenance, qualification of facts |
| **Edge attributes** | Metadata on relationships |
| **Edge patterns** | Query and constrain relationship structure |

## 1.3 The Progression

```
┌─────────────────────────────────────────────────────────────────────┐
│                      EDGE CAPABILITIES                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   BINARY EDGE                                                       │
│   ───────────                                                       │
│   Two participants. The standard case.                              │
│                                                                      │
│       Person ───follows───▶ Person                                  │
│                                                                      │
│                                                                      │
│   HYPEREDGE (N-ARY)                                                 │
│   ─────────────────                                                 │
│   Three or more participants. No reification needed.                │
│                                                                      │
│       Person ◀───meeting───▶ Person                                 │
│                    │                                                 │
│                    ▼                                                 │
│                  Room                                                │
│                                                                      │
│                                                                      │
│   HIGHER-ORDER EDGE                                                 │
│   ─────────────────                                                 │
│   Edge about an edge. Recursive structure.                          │
│                                                                      │
│       Alice ───claims───▶ (X ───causes───▶ Y)                       │
│                 │                                                    │
│                 └── confidence: 0.8                                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part II: Binary Edges

## 2.1 Declaration

```
edge follows(follower: Person, followed: Person)

edge assigned_to(task: Task, person: Person)

edge causes(from: Event, to: Event)
```

## 2.2 With Attributes

Edges can carry data:

```
edge assigned_to(task: Task, person: Person) {
  assigned_at: Timestamp [required] = now(),
  role: String [in: ["owner", "reviewer"]] = "owner"
}

edge follows(follower: Person, followed: Person) {
  since: Timestamp = now(),
  muted: Bool = false
}
```

## 2.3 Creating and Querying

```
-- Create edge
LINK assigned_to(#task_1, #alice) { role = "owner" }

-- Query with edge binding
MATCH t: Task, p: Person, assigned_to(t, p) AS a
WHERE a.role = "owner"
RETURN t.title, p.name, a.assigned_at

-- Reference edge by ID
MATCH e: assigned_to WHERE e.id = #edge_123
RETURN e
```

## 2.4 Type Constraints

Edge signatures constrain what can be connected:

```
edge assigned_to(task: Task, person: Person)

LINK assigned_to(#project_1, #alice)  -- ERROR: Project is not Task
LINK assigned_to(#task_1, #team_1)    -- ERROR: Team is not Person
```

Union types allow flexibility:

```
edge assigned_to(item: Task | Issue | Story, assignee: Person | Team)
```

---

# Part III: Hyperedges

## 3.1 The Problem with Binary-Only

Some relationships are inherently multi-party:

| Relationship | Participants |
|--------------|--------------|
| Meeting | organizer, attendee, room |
| Transaction | buyer, seller, item |
| Route segment | start, waypoint, end |
| Chemical reaction | reactant1, reactant2, catalyst, product |

In binary-only systems, you must **reify** — create an intermediate node:

```
-- Reification (the ugly way)
node Meeting { time: Timestamp }
edge organizes(person: Person, meeting: Meeting)
edge attends(person: Person, meeting: Meeting)
edge held_in(meeting: Meeting, room: Room)

-- Now you have 4 entities for 1 relationship
-- And complex queries to reassemble it
```

## 3.2 Hyperedge Declaration

MEW handles this directly:

```
edge meeting(organizer: Person, attendee: Person, room: Room) {
  scheduled_at: Timestamp [required],
  duration_minutes: Int = 60
}
```

Three participants, one edge. The relationship is atomic.

## 3.3 Arity

The number of participants is the edge's **arity**:

| Arity | Name | Example |
|-------|------|---------|
| 2 | Binary | `follows(a, b)` |
| 3 | Ternary | `meeting(org, att, room)` |
| 4 | Quaternary | `transaction(buyer, seller, item, broker)` |
| N | N-ary | Any number |

## 3.4 Position Semantics

Positions are **ordered and named**:

```
edge via(start: Location, waypoint: Location, destination: Location)

LINK via(#tokyo, #hawaii, #sf)
-- Position 0: start = tokyo
-- Position 1: waypoint = hawaii
-- Position 2: destination = sf
```

Position matters for semantics. `via(A, B, C)` ≠ `via(C, B, A)`.

## 3.5 Querying Hyperedges

```
-- Find all meetings in a specific room
MATCH meeting(org, att, #room_1) AS m
RETURN org.name, att.name, m.scheduled_at

-- Find routes through a waypoint
MATCH via(start, #hawaii, dest) AS v
RETURN start.name, dest.name

-- Bind all positions
MATCH via(s, w, d) AS v
WHERE w.type = "hub"
RETURN s, w, d, v.id
```

## 3.6 Anonymous Positions

Use `_` when you don't need to bind a position:

```
-- Any meeting in room 1
MATCH meeting(_, _, #room_1) AS m
RETURN m.scheduled_at

-- Route ending at destination, don't care about start/waypoint
MATCH via(_, _, #destination) AS v
RETURN v
```

## 3.7 Hyperedge Examples

### 3.7.1 Academic Authorship

```
edge authored(paper: Paper, author: Person, institution: Institution) {
  contribution: String [in: ["primary", "secondary", "advisor"]],
  order: Int [>= 1]
}

-- Query: find all papers where someone from MIT was primary author
MATCH authored(paper, person, #mit) AS a
WHERE a.contribution = "primary"
RETURN paper.title, person.name
```

### 3.7.2 Chemical Reactions

```
edge reaction(
  reactant1: Compound, 
  reactant2: Compound, 
  catalyst: Compound, 
  product: Compound
) {
  temperature_kelvin: Float,
  yield_percent: Float [>= 0, <= 100]
}
```

### 3.7.3 Flight Segments

```
edge flight_segment(
  origin: Airport, 
  destination: Airport, 
  carrier: Airline
) {
  flight_number: String [required],
  departure: Timestamp,
  arrival: Timestamp
}
```

---

# Part IV: Higher-Order Edges

## 4.1 The Insight

Sometimes you need to make statements **about relationships**:

- "Alice **believes** that X causes Y"
- "The claim that A follows B has **confidence** 0.8"
- "Bob **disputes** the assertion that C knows D"
- "The route via A-B-C was **assessed** by method M"

This is **higher-order structure**: edges whose targets include other edges.

## 4.2 Declaration Syntax

Use `edge<EdgeType>` to reference an edge type:

```
-- First-order: basic claim
edge causes(from: Event, to: Event)

-- Second-order: confidence about causes
edge confidence(claim: edge<causes>, level: Float)

-- Third-order: who assessed the confidence
edge assessed_by(rating: edge<confidence>, assessor: Person, method: String)
```

## 4.3 Creating Higher-Order Structure

```
-- Create base edge
LINK causes(#event_1, #event_2) AS c

-- Create second-order edge referencing it
LINK confidence(c) { level = 0.85 } AS conf

-- Create third-order edge
LINK assessed_by(conf, #alice) { method = "experiment" }
```

Or inline:

```
LINK confidence(
  LINK causes(#event_1, #event_2)
) { level = 0.85 }
```

## 4.4 Querying Higher-Order Structure

```
-- Find all causes with confidence > 0.8
MATCH 
  causes(e1, e2) AS c,
  confidence(c) AS conf
WHERE conf.level > 0.8
RETURN e1, e2, conf.level

-- Find who assessed a confidence rating
MATCH
  causes(e1, e2) AS c,
  confidence(c) AS conf,
  assessed_by(conf, assessor) AS a
WHERE a.method = "experiment"
RETURN e1, e2, conf.level, assessor.name

-- Find all claims Alice has made about causes
MATCH
  causes(e1, e2) AS c,
  claims(#alice, c) AS claim
RETURN e1, e2, claim.stated_at
```

## 4.5 Depth Is Unlimited

Higher-order edges can nest arbitrarily:

```
edge causes(from: Event, to: Event)
edge confidence(claim: edge<causes>, level: Float)
edge disputes(disputant: Person, rating: edge<confidence>)
edge supports(supporter: Person, dispute: edge<disputes>)
edge meta_comment(about: edge<supports>, comment: String)
```

Each level adds meaning:
1. X causes Y
2. That claim has confidence 0.8
3. Alice disputes that confidence rating
4. Bob supports Alice's dispute
5. Someone comments on Bob's support

## 4.6 Mixed Signatures

Higher-order edges can mix edge and node targets:

```
-- Person makes a claim about causation
edge claims(claimant: Person, claim: edge<causes>) {
  stated_at: Timestamp = now(),
  retracted: Bool = false
}

-- Evidence supports a claim
edge evidence_for(evidence: Document, claim: edge<causes>) {
  strength: Float [>= 0, <= 1]
}
```

## 4.7 Higher-Order on Hyperedges

Higher-order works with hyperedges too:

```
-- Ternary base edge
edge via(start: Location, waypoint: Location, dest: Location)

-- Confidence about route
edge route_confidence(route: edge<via>, level: Float)

-- Assessment of confidence
edge route_assessment(rating: edge<route_confidence>, method: String)
```

## 4.8 Why This Matters

Without higher-order edges, you must either:

**Option A: Reification** — Create intermediate nodes

```
node CausalClaim {
  from_event: ID,
  to_event: ID,
  confidence: Float
}
-- Loses graph structure, awkward to query
```

**Option B: Duplicate edges** — Embed metadata in base edge

```
edge causes(from: Event, to: Event) {
  confidence: Float,
  assessor: ID,
  assessment_method: String
}
-- Doesn't scale: what if different people have different confidence?
-- What if you want to track multiple assessments?
```

Higher-order edges solve this cleanly: the confidence is a **separate fact** that can be added, disputed, retracted independently.

---

# Part V: Patterns and Use Cases

## 5.1 Provenance Tracking

Track where information came from:

```
edge states(source: Document, fact: edge<any>)
edge derived_from(derived: edge<any>, source: edge<any>)
edge extracted_by(fact: edge<any>, extractor: Agent, method: String)
```

Query: "What's the provenance chain for this fact?"

```
MATCH
  causes(e1, e2) AS fact,
  extracted_by(fact, extractor) AS extraction,
  states(doc, fact)
RETURN e1, e2, extractor.name, doc.title
```

## 5.2 Temporal Qualification

Facts that were true at a time:

```
edge member_of(person: Person, org: Organization)
edge valid_during(membership: edge<member_of>, start: Timestamp, end: Timestamp?)

-- Query: who was in the org during 2023?
MATCH
  member_of(person, #acme) AS m,
  valid_during(m) AS v
WHERE v.start <= @2023-12-31 AND (v.end = null OR v.end >= @2023-01-01)
RETURN person.name, v.start, v.end
```

## 5.3 Multi-Agent Belief Systems

Different agents have different beliefs:

```
edge believes(agent: Agent, claim: edge<any>) {
  confidence: Float [>= 0, <= 1],
  acquired_at: Timestamp
}

edge contradicts(claim1: edge<any>, claim2: edge<any>)

-- Find what Alice and Bob disagree about
MATCH
  believes(#alice, claim1) AS b1,
  believes(#bob, claim2) AS b2,
  contradicts(claim1, claim2)
RETURN claim1, claim2, b1.confidence, b2.confidence
```

## 5.4 Confidence Aggregation

```
edge causes(from: Event, to: Event)
edge confidence(claim: edge<causes>, level: Float)

-- Multiple confidence ratings from different sources
LINK confidence(#causal_claim_1) { level = 0.7 }  -- source A
LINK confidence(#causal_claim_1) { level = 0.9 }  -- source B
LINK confidence(#causal_claim_1) { level = 0.6 }  -- source C

-- Query: average confidence for a claim
MATCH confidence(#causal_claim_1) AS c
RETURN AVG(c.level)
```

## 5.5 Annotation and Commentary

```
edge comment(author: Person, target: edge<any>, text: String)
edge reaction(reactor: Person, target: edge<any>, emoji: String)

-- Comment on any edge
LINK comment(#alice, #some_follows_edge) { text = "Interesting connection" }

-- React to comments (comment is an edge, so this works)
LINK reaction(#bob, #alice_comment_edge) { emoji = "👍" }
```

## 5.6 Knowledge Graph Uncertainty

```
-- Base knowledge
edge is_a(instance: Entity, class: Entity)
edge has_property(entity: Entity, property: Property)

-- Uncertainty
edge probability(fact: edge<is_a> | edge<has_property>, p: Float)
edge source(fact: edge<any>, origin: Source)
edge conflicts_with(fact1: edge<any>, fact2: edge<any>)

-- Query: facts with probability > 0.9 from trusted sources
MATCH
  has_property(entity, prop) AS fact,
  probability(fact) AS prob,
  source(fact, src)
WHERE prob.p > 0.9 AND src.trusted = true
RETURN entity, prop, prob.p
```

---

# Part VI: Edge Type Polymorphism

## 6.1 Union in Signatures

Edge positions can accept multiple types:

```
edge contains(container: Folder | Project, item: File | Folder | Task)
```

## 6.2 `edge<any>` for Higher-Order

When a higher-order edge should accept any edge type:

```
edge confidence(claim: edge<any>, level: Float)
-- Can attach confidence to ANY edge
```

## 6.3 Union of Edge Types

```
edge provenance(
  derived: edge<any>,
  source: edge<causes> | edge<implies> | edge<correlates>
)
```

## 6.4 Querying Polymorphic Edges

```
-- Find all things with confidence > 0.8
MATCH confidence(claim, level) AS c
WHERE c.level > 0.8
RETURN claim, c.level

-- The 'claim' could be any edge type
-- Use type inspection if needed:
MATCH confidence(claim, level) AS c
WHERE c.level > 0.8 AND type_of(claim) = "causes"
RETURN claim
```

---

# Part VII: Constraints on Edges

## 7.1 Edge Attribute Constraints

Same as node attributes:

```
edge assigned_to(task: Task, person: Person) {
  role: String [required, in: ["owner", "reviewer"]],
  assigned_at: Timestamp [required] = now()
}
```

## 7.2 Structural Constraints

```
-- No self-assignment
constraint no_self_assign:
  t: Task, p: Person, assigned_to(t, p)
  WHERE t.creator = p.id
  => false

-- Temporal ordering on causes
constraint causal_order:
  e1: Event, e2: Event, causes(e1, e2)
  WHERE e1.timestamp != null AND e2.timestamp != null
  => e1.timestamp < e2.timestamp
```

## 7.3 Higher-Order Constraints

```
-- Confidence must be between 0 and 1
constraint valid_confidence:
  c: confidence
  => c.level >= 0 AND c.level <= 1

-- Can't assess your own confidence ratings
constraint no_self_assessment:
  confidence(claim) AS c,
  assessed_by(c, assessor) AS a,
  claims(assessor, claim)
  => false
```

## 7.4 Cardinality Constraints

```
-- Task can have at most one owner
edge assigned_to(task: Task, person: Person) [
  unique_on: (task, role) WHERE role = "owner"
]

-- Or as explicit constraint:
constraint single_owner:
  t: Task, p1: Person, p2: Person,
  assigned_to(t, p1) AS a1,
  assigned_to(t, p2) AS a2
  WHERE a1.role = "owner" AND a2.role = "owner" AND p1.id != p2.id
  => false
```

## 7.5 Referential Actions

What happens when edge targets are deleted:

```
edge assigned_to(task: Task, person: Person) [
  on_kill_task: cascade,     -- delete edge if task deleted
  on_kill_person: nullify    -- ERROR: edges can't have null targets
]

-- For edges, deletion of any target typically cascades to the edge
-- This is usually implicit
```

---

# Part VIII: Interaction with Other Systems

## 8.1 Rules on Edges

Rules can match and produce edges:

```
-- Auto-create confidence when claim is made
rule default_confidence:
  causes(e1, e2) AS c
  WHERE NOT EXISTS(confidence(c, _))
  =>
  LINK confidence(c) { level = 0.5 }

-- Cascade confidence updates
rule propagate_confidence:
  causes(e1, e2) AS c1,
  causes(e2, e3) AS c2,
  confidence(c1) AS conf1,
  confidence(c2) AS conf2
  WHERE NOT EXISTS(derived_confidence(c1, c2, _))
  =>
  LINK derived_confidence(c1, c2) { 
    level = conf1.level * conf2.level 
  }
```

## 8.2 Watch on Edges

```
-- Watch for new causal claims
WATCH causes(e1, e2) AS c
[mode: watch]
RETURN e1.name, e2.name, c.id

-- Watch for confidence changes
WATCH confidence(claim) AS c
WHERE c.level > 0.9
[mode: watch]
RETURN claim, c.level
```

## 8.3 Policy on Edges

```
-- Only experts can create causes edges
policy expert_claims_only:
  ON LINK(c: causes)
  ALLOW IF has_role(current_actor(), "expert")

-- Only claim author can modify confidence
policy author_modifies_confidence:
  ON SET(conf: confidence, "level")
  ALLOW IF EXISTS(
    claims(current_actor(), claim),
    conf: confidence WHERE conf.claim = claim
  )
```

## 8.4 Interiority and Edges

Inside a world, edges stay inside:

```
node Agent [has_interior] {
  interior: ontology {
    node BeliefRef { represents: ID }
    
    -- Edges within interior
    edge believes(belief: BeliefRef) { confidence: Float }
    edge derived(conclusion: BeliefRef, premise: BeliefRef)
    
    -- Higher-order within interior
    edge confidence_source(rating: edge<believes>, source: String)
  }
}
```

Cross-boundary references use ID attributes, not edges. See INTERIORITY.md.

---

# Part IX: Layer 0 Representation

## 9.1 Edge Type Definition

```
_EdgeType node:
  name: String [required, unique]
  arity: Int [required]        -- number of positions
  doc: String?

For each position i:
  _VarDef node:
    name: <position name>
    is_edge_var: Bool          -- true if position accepts edges
  
  _edge_has_position edge:
    (edge_type, var_def) { position: i }
  
  _var_has_type edge:
    (var_def, type_expr)       -- including edge<X> types
```

## 9.2 Edge Instances

Edge instances are stored with:
- Unique ID
- Type reference
- Ordered target IDs (nodes or edges)
- Attribute values

## 9.3 Higher-Order Type Expressions

```
_EdgeTypeRefExpr : _TypeExpr [sealed] {
  edge_type_name: String [required]  -- "causes", "any", etc.
}
```

---

# Part XI: Summary

## 11.1 Edge Capabilities

| Capability | Description |
|------------|-------------|
| **Binary** | Standard two-participant relationships |
| **N-ary (Hyperedges)** | Arbitrary number of participants |
| **Higher-Order** | Edges referencing other edges |
| **Attributes** | Data on relationships |
| **Type Constraints** | Signatures restrict what can connect |
| **Polymorphism** | Union types and `edge<any>` |

## 11.2 When to Use Each

| Use Case | Edge Type |
|----------|-----------|
| Simple relationship | Binary |
| Multi-party relationship | Hyperedge |
| Metadata about relationship | Edge attributes |
| Statements about statements | Higher-order |
| Confidence/provenance | Higher-order |
| Temporal qualification | Higher-order |

## 11.3 Key Principles

| Principle | Description |
|-----------|-------------|
| **First-class edges** | Edges have IDs, can be referenced |
| **Arbitrary arity** | Not limited to binary |
| **Recursive structure** | Edges about edges, unlimited depth |
| **Typed signatures** | Compile-time validation |
| **Graph integrity** | Constraints apply to edges too |

## 11.4 The Power

```
┌─────────────────────────────────────────────────────────────────────┐
│                        THE UNLOCK                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Traditional graph:                                                 │
│    "Alice follows Bob"                                              │
│                                                                      │
│  MEW:                                                               │
│    "Alice claims, with confidence 0.8 as assessed by Carol          │
│     using method 'survey', that Event X causes Event Y,             │
│     and Bob disputes this claim, which Dan supports."               │
│                                                                      │
│  All as clean, queryable, constrainable graph structure.            │
│  No reification. No awkward modeling. Just edges.                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Appendix A: Complete Grammar

```ebnf
(* Edge Type Declaration *)
EdgeTypeDecl = DocComment? "edge" Identifier "(" SignatureParams ")"
               ("{" AttributeDecl* "}")?

SignatureParams = SignatureParam ("," SignatureParam)*

SignatureParam = Identifier ":" TypeExpr

(* Type Expressions for Edges *)
TypeExpr = ScalarType
         | Identifier                    -- node type name
         | "edge" "<" EdgeTypeRef ">"    -- higher-order
         | TypeExpr "|" TypeExpr         -- union
         | TypeExpr "?"                  -- optional (for attributes)

EdgeTypeRef = Identifier                 -- specific edge type
            | "any"                      -- any edge type

(* Edge Patterns in Queries *)
EdgePattern = Identifier "(" TargetList ")" EdgeAlias?

TargetList = Target ("," Target)*

Target = Identifier    -- bound variable
       | NodeRef       -- literal node reference
       | "_"           -- anonymous

EdgeAlias = "AS" Identifier

(* Edge Operations *)
LinkStmt = "LINK" Identifier "(" TargetList ")" AttrBlock? ("AS" Identifier)?

UnlinkStmt = "UNLINK" EdgeRef

EdgeRef = "#" Identifier    -- edge ID
        | Identifier        -- bound edge variable
```

---

# Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Arity** | Number of positions in an edge signature |
| **Binary edge** | Edge with exactly 2 positions |
| **Hyperedge** | Edge with 3+ positions (N-ary) |
| **Higher-order edge** | Edge where at least one position accepts edges |
| **Position** | Named, typed slot in an edge signature |
| **Edge binding** | Using `AS` to capture edge reference in a pattern |
| **Reification** | Modeling relationships as nodes (what MEW avoids) |
| **`edge<T>`** | Type expression accepting edges of type T |
| **`edge<any>`** | Type expression accepting any edge |

---

*End of MEW Edges Capability*
