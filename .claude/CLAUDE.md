# CLAUDE.md

---

## Commands

```bash
# Build
cd mew && cargo build --workspace
# Format
cd mew && cargo fmt --all
# Lint
cd mew && cargo clippy --workspace
```

---

## Testing

Use the unified test runner `test.sh` at the project root:

```bash
# Run ALL tests (unit + integration + testgen)
./test.sh
```

**Test types:**

| Type | What it tests | Crates |
|------|---------------|--------|
| `unit` | Individual component logic | All 17 crates via cargo test |
| `integration` | Full workflows with ontologies | `mew-tests` scenarios |
| `testgen` | Auto-generated from schemas | `mew-testgen` oracle verification |

**Direct cargo commands (if needed):**

```bash
cd mew && cargo test --workspace           # All unit tests
cd mew && cargo test -p mew-graph          # Single component
cd mew && cargo test -p mew-graph -- --nocapture  # With output
```

---

## Document Hierarchy

When documents conflict, higher level wins:

```
1. specs/specification/1_FOUNDATIONS.md     # Highest authority
2. specs/specification/2_DSL.md
3. specs/specification/3_GQL.md
4. specs/specification/0_META_ONTOLOGY.md
5. specs/architecture.md
6. specs/components/*.md
7. specs/tests/*.md
8. mew/                                     # Code conforms to above
```

---

## Code Style (unspecified)
---

## Test Style (unspecified)

---

## Modifying Specifications

Specifications can contain errors discovered during implementation.

**To revise a specification, you MUST use the appropriate skill:**

| Document | Required Skill |
|----------|----------------|
| `specs/specification/1_FOUNDATIONS.md`, `2_DSL.md`, `3_GQL.md`, `specs/architecture.md` | `revise-specification` |
| `specs/specification/0_META_ONTOLOGY.md` | `revise-meta-ontology` |

**Never silently diverge from specs.** If reality contradicts a spec, fix the spec explicitly using the skill, then propagate changes downward.

---

## Commit Messages (unspecified)


---

NEVER:
- comment out some tests just because they "fail" and features not yet implemented => its the goal of leaving the test to fail until we implement what's needed
- simplify the test to make it pass, because we're lazy, or doesn't feel implementing everything / rewriting the test to make it simpler
example: a test was trying to test multiple chained "spawn" like:
SPAWN ...
SPAWN ...
SPAWN ...
and the AI separate this test into 3 separete tests, testing each spawn individually (like completely cheating)
=> basically should never try to max GAMING


