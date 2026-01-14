# VERIFY SCENARIO

Your goal: verify that a scenario's expected outputs are actually correct.

This runs after every `/expand-scenarios`. A test with wrong expectations is worse than no test—it creates false confidence or masks real bugs.

## The Core Question

For every operation in the scenario:

> Given this input, against this schema, with this seed data—is the expected output **truly** what the system should produce?

Not "does it look reasonable." Not "does it compile." **Is it correct per the spec?**

## Verification Process

1. **Read the ontology** (`ontology.mew`). Understand:
   - Type definitions and inheritance
   - Attribute constraints (required, unique, format, range)
   - Edge definitions and cardinalities
   - Default values

2. **Read the seed data** (`seeds/*.mew`). Know exactly what exists before the operation runs.

3. **For each operation**, trace through manually:
   - What nodes/edges exist before?
   - What does the operation do step by step?
   - What should exist after?
   - What should the RETURN clause produce?

4. **Compare your trace to the expected output**. Check:
   - Correct node/edge counts in mutation effects
   - Correct attribute values in query results
   - Correct error type and message for failure cases
   - Correct ordering if ORDER BY is used
   - Correct aggregation results (COUNT, SUM, etc.)

## Common Mistakes to Catch

- **Off-by-one in counts**: Forgetting seed data, double-counting, missing edges
- **Wrong default values**: Assuming null when there's a default, or vice versa
- **Inheritance confusion**: Missing inherited attributes in expected output
- **Constraint violations**: Expecting success when a constraint should reject
- **Aggregation errors**: Wrong SUM/AVG math, COUNT including nulls incorrectly
- **Ordering assumptions**: Expected order doesn't match ORDER BY clause
- **ID assumptions**: Hardcoded IDs that depend on execution order

## Output

For each operation in the scenario:
- ✓ Verified correct — with brief reasoning
- ✗ Incorrect — what's wrong, what it should be, fix applied

If you find errors: fix them immediately. A scenario with wrong expectations must not be committed.

## The Standard

You are the last line of defense before a bad test enters the suite. Be meticulous. Trace every value. Trust nothing—verify everything.
