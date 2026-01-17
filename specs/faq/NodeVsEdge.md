```markdown
# MEW Design FAQ: Nodes, Edges, and Glyphs

**Version:** 1.1  
**Status:** Foundation  
**Scope:** Storage-level monism, language-level dualism, and terminology

---

## Overview

MEW's architecture: **unified storage, distinguished syntax**. At storage, nodes and edges are both "glyphs." At the language layer, `node` and `edge` are distinct keywords with different affordances. The compiler bridges the gap.

---

## Part I: Monism vs Dualism

### Q1: Are nodes and edges fundamentally different?

**Storage level: No.** Both are glyphs:

```
Glyph
├── id: GlobalId
├── type_tag: TypeId
├── alive: Bool
├── attributes: [Values]
└── targets: [GlobalId]?   -- present = edge, absent = node
```

**Language level: Yes.** Different structural roles, modifiers, inheritance rules, pattern syntax.

---

### Q2: Why two keywords if storage is unified?

**Affordances.** `edge` signals: relationship, typed signature, edge-specific modifiers (`symmetric`, cardinality, referential actions). `node` signals: standalone thing, inheritable, no targets. Collapsing to single keyword loses semantic clarity without user-level benefit.

---

### Q3: Is `node`/`edge` syntactic sugar?

Not trivially. Better framing: **two categories compiling to same primitive with different configurations**.

```
node Event { timestamp: Timestamp }     → Glyph, targets = ∅
edge causes(from: Event, to: Event)     → Glyph, targets = [from, to]
```

DSL keywords determine: target schema, valid modifiers, inheritance rules, pattern matching behavior. Storage sees only glyphs.

---

### Q4: Why storage-level monism?

| Reason | Benefit |
|--------|---------|
| Uniform ID space | Higher-order trivial: edge targeting edge is just glyph targeting glyph |
| Uniform operations | SPAWN/KILL/SET identical regardless of targets |
| Simpler model | One primitive easier to reason about than two with complex interactions |

---

### Q5: Why physical layout partitioned if conceptually monist?

**Access patterns differ.**

| Structure | Contents | Optimized For |
|-----------|----------|---------------|
| FamilyTable | Nodes by inheritance root | Type-polymorphic queries, attribute scans |
| EdgeTensor | Edges per-type | Traversal, adjacency, SpMV |

GlobalId encoding routes O(1) to correct structure. Partition is optimization, not conceptual violation.

---

## Part II: Boundaries and Constraints

### Q6: Does Layer 0 need monism?

**No.** Currently dualist (`_NodeType`, `_EdgeType`). Could unify to `_GlyphType` with `arity` field. Current choice: keep separate — aids validation, matches DSL. Storage monism doesn't require meta-schema monism.

---

### Q7: Can a glyph sometimes have targets (hybrid)?

**No.** Type definition determines targets-or-not. Instance-level variance disallowed.

**Why?** Conflates "what X is" with "how X relates." Dualist modeling keeps these separate:

```
node Document { title: String }
edge child_of(child: Document, parent: Document) [child -> 0..1]
```

Clearer, more flexible (query documents independent of hierarchy).

---

### Q8: Cross-category inheritance?

**No.** Nodes inherit from nodes. Edges inherit from nothing (v1). Cross-category (node↔edge) forbidden — positional signatures don't compose with attribute-only inheritance.

---

### Q9: Why can't edges inherit from edges?

**Complexity without demand.** Would require: signature compatibility rules, position covariance/contravariance, modifier inheritance semantics. Solvable but no clear use case yet. V1: edges flat, one type per tensor.

---

## Part III: Higher-Order

### Q10: How does higher-order work?

Storage monism makes it trivial:

```
edge causes(from: Event, to: Event)
edge confidence(about: edge<causes>) { level: Float }
```

`causes(e1, e2)` → glyph with targets `[e1, e2]`, gets GlobalId `c`  
`confidence(c)` → glyph with targets `[c]`

`edge<causes>` is compile-time type constraint, not storage difference. Runtime: `c` is just a GlobalId.

---

### Q11: Target type semantics?

| Type | Accepts |
|------|---------|
| `Event` | Any glyph typed Event or subtype |
| `edge<causes>` | Any glyph typed `causes` |
| `edge<any>` | Any glyph with targets |
| `any` | Any glyph without targets |

Storage identical (GlobalId). Type system prevents linking `confidence` to `Person`.

---

### Q12: Node referencing node via attribute?

**Yes, but not structural.**

```
node Task { creator_id: ID }        -- opaque data, no traversal
edge created_by(task: Task, p: Person)  -- structural, traversable, referential actions
```

Use edges for queryable relationships. Use ID attributes for denormalized refs or cross-world projections.

---

## Part IV: The Complete Distinction

| Aspect | Node | Edge |
|--------|------|------|
| **Targets** | None | Required (arity ≥ 1) |
| **DSL keyword** | `node` | `edge` |
| **Signature** | N/A | Typed positional parameters |
| **Pattern syntax** | `x: Type` | `name(a, b)` / `name(a, b) AS e` |
| **Inheritance** | Yes | No (v1) |
| **Physical storage** | FamilyTable | EdgeTensor |
| **Type expression** | `TypeName` | `edge<TypeName>` / `edge<any>` |
| **Layer 0 meta-type** | `_NodeType` | `_EdgeType` |
| **`symmetric`** | N/A | Yes |
| **`no_self`** | N/A | Yes |
| **`acyclic`** | N/A | Yes |
| **Cardinality** | N/A | Yes (`param -> N..M`) |
| **Referential actions** | N/A | Yes |
| **Can be edge target** | Yes | Yes (higher-order) |
| **Can target others** | No | Yes |
| **Semantic role** | Thing | Relationship |
| **Index structures** | Attribute indexes | CSR/CSC + attribute indexes |
| **Transitive patterns** | N/A | `edge+` / `edge*` |
| **Abstract/sealed** | Yes | No (v1) |

---

## Part V: Why "Glyph"?

### The Problem

Need a term for the unified storage primitive. "Entity" is too node-connotated — it suggests standalone thing, not relationship.

### Alternatives Rejected

| Term | Problem |
|------|---------|
| Entity | Node-flavored |
| Element | Overloaded (sets, HTML, arrays) |
| Object | Overloaded (OOP) |
| Record | Suggests flat/tabular |
| Cell | Biological connotation, unusual |
| Atom | Logic programming baggage |
| Symbol | Overloaded (Lisp, semiotics) |
| Locus | Too spatial |

### Why Glyph Works

**Definition:** A glyph is a deliberately shaped difference whose role is to be recognized.

**Fit:**
- The graph is a surface; glyphs are information carved into it
- Neutral between node/edge — both are "carved marks"
- Unclaimed in database/graph terminology
- Single syllable, smooth to say
- Evokes intentionality: glyphs are placed with meaning

**Philosophical alignment:** MEW models reality as structured information. Glyphs are the atomic units of that structure — recognized patterns, not arbitrary data. Whether a glyph stands alone (node) or connects (edge), it's the same kind of thing: a distinct, identifiable mark in the graph.

### Visibility

Users rarely see "glyph":
- DSL: `node`, `edge`
- Queries: type names, pattern syntax
- Architecture docs: "glyph" when discussing unified model

Users stay in dualist mental model unless examining storage internals.

---

## Part VI: Summary

### One Sentence

Nodes and edges are distinguished in syntax for ergonomics, unified in storage as glyphs for simplicity.

### Layer Model

| Layer | Model | Rationale |
|-------|-------|-----------|
| DSL | Dualist | Clear affordances |
| Query/Pattern | Dualist | Readable syntax |
| Layer 0 | Dualist | Validation clarity |
| Logical Storage | Monist | Uniform operations, clean higher-order |
| Physical Storage | Partitioned | Access pattern optimization |

### Invariants

1. Every glyph has unique GlobalId
2. GlobalId → (type, attributes, targets) resolution is uniform
3. Edge = glyph with targets; Node = glyph without
4. DSL distinction compiles away
5. Higher-order = glyph references glyph

---

## Part VII: Extended Questions

### Q13: Could edges gain inheritance later?

Yes. Would require defining signature compatibility. Likely covariant on target types (child edge accepts same or narrower types). Not v1 priority.

### Q14: Why arity ≥ 1, not ≥ 2?

Unary edges (arity 1) are valid. `edge marked(x: Task)` is a set membership. Binary is common but not required. Hyperedges (arity > 2) native.

### Q15: Can edge target itself?

**No.** Edge targets are set at creation, before edge has ID. Self-reference impossible by construction. (Node self-loops via edges are fine: `edge self_ref(x: Node, y: Node)` where x = y.)

### Q16: How do projections relate?

Projections (cross-world references) use ID attributes, not edges. A projection node holds `represents: GlobalId` pointing to external glyph. No edge crosses world boundary. This is the ID-attribute pattern from Q12.

### Q17: Does `any` include edges?

**No.** `any` = any node type. `edge<any>` = any edge type. Distinction matters for type safety.

### Q18: Why FamilyTable for nodes, not per-type?

Inheritance. Query `MATCH x: Animal` must find Dogs, Cats, etc. Family table groups by inheritance root — single scan covers hierarchy. Edges lack inheritance, so per-type tensors are optimal.

### Q19: Impact on query compilation?

Pattern `x: Event` compiles to FamilyTable scan with type filter. Pattern `causes(a, b)` compiles to EdgeTensor iteration. Compiler knows which structure from type metadata. Uniform GlobalId resolution either way.

### Q20: Is this design locked?

Storage monism: locked (fundamental). DSL dualism: locked (clear ergonomic win). Layer 0 dualism: soft (could unify if compelling reason). Physical partitioning: soft (optimization, could change with access patterns).

---

*Core choice: unified storage, distinguished syntax. Glyphs are the atoms; nodes and edges are the grammar.*
```