# CLAUDE.md

Project-level context for MEW (Minimum Executable World).

---

## What This Is

A self-describing hypergraph database. Nodes, edges, edges about edges. Schema stored as data. Rules and constraints. ACID transactions.

---

## Project Structure

```
specification/           # Authoritative definitions (read carefully before modifying)
  0_META_ONTOLOGY.md    # Layer 0 self-description
  1_FOUNDATIONS.md      # Philosophical grounding
  2_DSL.md              # Ontology definition language
  3_GQL.md              # Query/mutation language

implementation/          # Implementation guidance
  architecture.md       # 13-component system design
  components/*.md       # Component contracts
  tests/*.md            # Acceptance test suites (158 total)
  meta-roadmap.md       # Navigation methodology

mew/                     # Rust workspace (your code goes here)
  Cargo.toml
  graph/                # Each component is a crate
  parser/
  ...

ontologies/              # Test ontologies by complexity
  level-1/              # Simple (contacts, bookmarks)
  level-5/              # Complex (cognitive agents)

.claude/skills/          # Learned procedures (create more as needed)
```

---

## Implementation Language

**Rust.** Cargo workspace in `mew/`.

Each component is a separate crate: `mew-graph`, `mew-parser`, etc.
Shared types go in `mew-core` (create if needed).

---

## Commands

```bash
# Build
cd mew && cargo build --workspace

# Test everything  
cd mew && cargo test --workspace

# Test single component
cd mew && cargo test -p mew-graph

# Test with output
cd mew && cargo test -p mew-graph -- --nocapture

# Format
cd mew && cargo fmt --all

# Lint
cd mew && cargo clippy --workspace
```

---

## Document Hierarchy

When documents conflict, higher level wins:

```
1. specification/1_FOUNDATIONS.md     # Highest authority
2. specification/2_DSL.md
3. specification/3_GQL.md
4. specification/0_META_ONTOLOGY.md
5. implementation/architecture.md
6. implementation/components/*.md
7. implementation/tests/*.md
8. mew/                               # Code conforms to above
```

---

## Code Style

```rust
// Prefer explicit, simple, readable
pub fn get_node(&self, id: NodeId) -> Option<&Node> { ... }

// Avoid clever, implicit, compact
pub fn node(&self, id: impl Into<NodeId>) -> impl Deref<Target=Node> { ... }
```

- All public APIs return `Result<T, E>` — never panic on user input
- Errors include: what, where, why
- No unsafe without justifying comment
- No dependencies unless necessary

---

## Test Style

```rust
#[test]
fn test_name_matches_acceptance_test() {
    // GIVEN
    let mut graph = Graph::new();
    
    // WHEN
    let id = graph.create_node(TypeId(1), attrs![]);
    
    // THEN
    assert!(graph.get_node(id).is_some());
}
```

Test names correspond to acceptance tests in `implementation/tests/*.md`.

---

## Terminal Condition

**Done when ALL true:**

1. All 158 acceptance tests pass (`cargo test --workspace`)
2. Terminal session in `meta-roadmap.md` section 1 runs without error
3. REPL loads any ontology from `ontologies/`
4. System recovers correctly after kill -9

---

## Modifying Specifications

Specifications can contain errors discovered during implementation.

**To revise a specification, you MUST use the appropriate skill:**

| Document | Required Skill |
|----------|----------------|
| `1_FOUNDATIONS.md`, `2_DSL.md`, `3_GQL.md`, `architecture.md` | `revise-specification` |
| `0_META_ONTOLOGY.md` | `revise-meta-ontology` |

**Never silently diverge from specs.** If reality contradicts a spec, fix the spec explicitly using the skill, then propagate changes downward.

---

## Commit Messages

```
component: what changed

- detail 1
- detail 2

Tests: X/158 passing
```