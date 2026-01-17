# VERIFY SCENARIO

Your goal: verify that a scenario's expected outputs are actually correct.

This runs after every `/expand-scenarios`. A test with wrong expectations is worse than no test—it creates false confidence or masks real bugs.

## Scope: Operation Files Only

You will be given specific operation file(s) to verify. Each file is at:
`examples/level-N/ontology-name/operations/operation-name.mew`

For each operation file, you need to verify:
1. The corresponding test file's assertions match

The test file is at: `mew/tests/tests/levelN_ontology-name.rs`

**ONLY verify the operations in the files you're given.** Do not verify the entire ontology.

## The Core Question

For every operation in the scenario:

> Given this input, against this schema, with this seed data—is the expected output **truly** what the system should produce?

Not "does it look reasonable." Not "does it compile." **Is it correct per the spec?**

## CRITICAL: Never Trust Existing Assertions

**Both the `.mew` annotations AND the test file assertions can be wrong.** That's literally what you're checking.

DO NOT:
- Look at the test file's `.rows(N)` assertions first
- Assume `.mew` annotations are correct
- Use "matches the test" as verification
- Trust comments like "Updated based on actual data"

DO:
- Derive every expected value independently from first principles
- Only compare your derivation to assertions AFTER you've computed the answer
- If your derivation disagrees with assertions, YOUR derivation wins (re-check it once)

## Verification Process

1. **Read the ontology** (`ontology.mew`). Understand:
   - Type definitions and inheritance
   - Attribute constraints (required, unique, format, range)
   - Edge definitions and cardinalities
   - Default values

2. **Read the seed data** (`seeds/*.mew`). Know exactly what exists before the operation runs.

3. **For each non-trivial operation**, write a FULL DERIVATION:

```
## test_name

**Query:**
[the query]

**Before state:**
- List relevant nodes/edges that exist

**Step-by-step execution:**
- Step 1: MATCH finds X
- Step 2: WALK/FILTER does Y
- Step 3: Each intermediate result

**Final result:**
| column | value |
|--------|-------|
| ...    | ...   |

**Rows returned: N**
```

4. **AFTER deriving**, compare to both `.mew` annotation and test assertion:
   - If both match your derivation: ✓ Verified
   - If either differs: After triple checking your derivation, fix the wrong assertion(s).

5. **For complex operations, spawn a subagent for independent verification:**
   - Give the subagent ONLY: the query, ontology, and seed data
   - Do NOT share your derivation or the existing assertions
   - Ask: "What should this query return? Show full derivation."
   - Compare their derivation to yours — if they differ, investigate before proceeding

## Common Mistakes to Catch

- **Off-by-one in counts**: Forgetting seed data, double-counting, missing edges
- **Wrong default values**: Assuming null when there's a default, or vice versa
- **Inheritance confusion**: Missing inherited attributes in expected output
- **Constraint violations**: Expecting success when a constraint should reject
- **Aggregation errors**: Wrong SUM/AVG math, COUNT including nulls incorrectly
- **Ordering assumptions**: Expected order doesn't match ORDER BY clause
- **ID assumptions**: Hardcoded IDs that depend on execution order
- **UNTIL/termination logic**: Walk stops at FIRST match, not all matches
- **TERMINAL vs NODES**: TERMINAL returns where walk stopped, NODES returns all visited

## Output Format

For trivial operations (simple SPAWN, obvious KILL):
- ✓ Verified — one-line reasoning

For non-trivial operations (WALK, aggregations, filters, joins):
- Show full derivation table
- ✓ Verified OR ✗ Incorrect with fix

## The Standard

You are the last line of defense before a bad test enters the suite.

**Derive. Don't trust. Show your work.**

If you catch yourself writing "matches the test assertion" without a derivation, STOP. You're not verifying—you're rubber-stamping.

## ⚠️ Tests Must Reflect Correct Behavior — Even If Implementation Doesn't Exist

A test expects what the system SHOULD return according to the spec, not what it currently returns.

- If a feature isn't implemented → the test should expect correct behavior → test will FAIL → that's correct
- If a feature is buggy → the test should expect correct behavior → test will FAIL → that's correct
- NEVER set expectations based on "what the implementation currently does"

The only valid source of truth for expected values is the SPEC, not the implementation.
