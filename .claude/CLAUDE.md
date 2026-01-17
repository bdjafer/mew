# CLAUDE.md

---

## ⛔ TEST INTEGRITY — READ THIS FIRST ⛔

**Tests define correct behavior. Tests are the spec. Tests are sacred.**

A failing test is NOT a problem to solve by changing the test. A failing test means: **implement the feature**.

### The One Rule

> **Tests must always expect the CORRECT result according to the scenario — whether the implementation exists or not.**

If a feature isn't implemented → the test FAILS. That's the point. That's TDD. Leave it failing.

### FORBIDDEN Actions (No Exceptions)

| ❌ NEVER DO THIS | ✅ DO THIS INSTEAD |
|------------------|-------------------|
| Change `.rows(5)` to `.rows(0)` because query isn't implemented | Leave test failing, implement the query |
| Change `.modified(1)` to `.modified(2)` to match buggy output | Leave test failing, fix the implementation |
| Comment out `#[test]` or add `#[ignore]` | Leave test failing |
| Split one complex test into simpler passing tests | Leave test failing, implement the complex behavior |
| Weaken assertions (exact → partial, strict → loose) | Leave test failing |
| Add "TODO" comments and skip the assertion | Leave test failing |
| Change expected values to match actual (wrong) output | Leave test failing |

### Why This Matters

- A test that expects wrong results is **worse than no test** — it creates false confidence
- "Making tests pass" by changing expectations is **cheating**, not progress
- The test suite is the **definition of done** — corrupting it corrupts the project

### How to Know If You're Gaming

Ask yourself: *"Am I changing the test because I discovered the expected value was wrong, or because I don't want to implement the feature?"*

If the answer is the latter → STOP. Leave the test failing.

### The Only Valid Reason to Change a Test

The expected value was **actually incorrect** according to the spec. In this case:
1. Derive the correct value from first principles (see `verify-scenario` skill)
2. Show your derivation
3. Then update the test

"The implementation returns X" is NOT a derivation. The spec defines correct behavior, not the current implementation.

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
