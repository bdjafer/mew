# SPECS COVERAGE

Your goal: maintain an accurate coverage map for a single level.

You produce `examples/level-{N}/SPECS.md`—a truth table showing which spec features are tested and where.

## Process

1. **Read the specs** for this level:
   - `specs/specification/*.md` — authoritative feature definitions
   - `examples/LEVELS.md` — which features belong at this level

2. **Extract every testable feature** from the specs. Be granular:
   - Each mutation type (SPAWN, KILL, LINK, UNLINK, SET)
   - Each query clause (WHERE, ORDER BY, LIMIT, etc.)
   - Each operator and function
   - Each constraint type
   - Each error condition
   - Edge cases called out in specs

3. **Scan existing scenarios** at this level:
   - Every `operations/*.mew` file
   - Note which spec features each scenario exercises

4. **Reconcile with spec changes** (if updating an existing SPECS.md):
   - For each row in the existing matrix, verify the spec reference still exists
   - If a spec section is missing, check `.spec-revisions.log` for what happened:
     - **Removed**: Delete the row from coverage matrix
     - **Renamed/moved**: Update the spec reference to new location
     - **Merged into another section**: Update reference, possibly consolidate rows
   - If revisions log has no entry, the spec section may have been reorganized—search current specs for the feature

5. **Produce the coverage table**:

```markdown
# Level N Coverage

## Mutations
| Feature | Spec | Covered | Scenario |
|---------|------|:-------:|----------|
| SPAWN basic | 5_MUTATIONS.md §1.1 | ✓ | bookmarks/spawn_variants |
| SPAWN with RETURNING | 5_MUTATIONS.md §1.6 | ✗ | — |
| KILL by id | 5_MUTATIONS.md §2.1 | ✓ | contacts/crud |
...

## Queries
| Feature | Spec | Covered | Scenario |
...

## Expressions
...

## Errors
...
```

## Rules

- **One scenario reference is enough**. If 5 scenarios test SPAWN, list one. The point is coverage existence, not exhaustive enumeration.
- **Be honest about gaps**. A feature is covered only if a scenario explicitly tests it—not if it's incidentally used.
- **Spec reference is required**. Every row must cite the spec section defining that feature.
- **Keep it current**. This file reflects the codebase now, not aspirations.
- **Specs evolve**. When a referenced spec section no longer exists:
  1. Check `.spec-revisions.log` for documented changes
  2. If feature was removed from spec, remove from matrix
  3. If feature was moved/renamed, update the reference
  4. If no log entry exists, search specs for the feature—it may have been reorganized

## Output

`examples/level-{N}/SPECS.md` — complete, accurate, ready for `/build-scenario` to consume.
