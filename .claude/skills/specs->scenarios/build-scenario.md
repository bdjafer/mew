# BUILD SCENARIO

Your goal: write one scenario that covers an untested spec feature.

## Input

You receive a level number. Read `examples/level-{N}/SPECS.md` to see what's missing.

## Process

1. **Pick one uncovered feature** from the coverage table. Prioritize:
   - Core features over edge cases
   - Features blocking other work
   - Features with clear spec definitions

2. **Check if existing ontologies can express it**. Scan ontologies at this level—can any of them exercise this feature without modification?

3. **If no ontology fits**, create a new one. But only if the feature genuinely requires schema constructs not present elsewhere. New ontologies are expensive—justify them.

4. **Write the scenario**:
   - Operation file: `operations/{feature_name}.mew`
   - Clear setup (use seeds or inline SPAWNs)
   - The operation being tested
   - Expected result (success with effects, or specific error)

5. **Write the Rust test** with correct expected outputs.

6. **Run `/verify-scenario`** before considering it done.

7. **Update `examples/level-{N}/SPECS.md`** — mark the feature as covered with your new scenario name.

## Constraints

- One feature per scenario. Don't bundle.
- Name the file after the feature, not the ontology.
- If the feature is already covered (coverage table is stale), say so and stop.

## Output

Either:
- One new scenario (`.mew` + Rust test) covering a previously untested feature
- "Feature X is already covered by Y" if coverage table was outdated
- "No suitable ontology exists and creating one isn't justified for this feature" (rare)
