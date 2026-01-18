# REMOVE SCENARIOS

Your goal: find and remove scenarios that provide zero unique value—complete duplicates of other tests.

## What Counts as Duplicate

A scenario is a duplicate if and only if:
- It tests the **exact same behavior** as another scenario
- Under **equivalent conditions** (same schema constraints, same data shape)
- Such that **removing it loses nothing**—the other scenario already proves the behavior works

This is not about slight overlap. Two tests can share setup, share features, even share expected outcomes—and still both be valuable if they test different aspects or edge cases.

## What Does NOT Count

- Testing the same feature in different ontologies → NOT duplicate (different schema context)
- Testing the same operation with different data → NOT duplicate (different edge cases)
- Testing success vs failure of the same operation → NOT duplicate (different paths)
- Similar-looking tests that verify different spec clauses → NOT duplicate

## The Trap

You might argue "nothing is ever truly duplicate" to avoid removing anything. That's intellectually lazy.

Be honest. If two scenarios exist and you could delete one with zero information loss—delete it. The test suite should be minimal and complete, not exhaustive and redundant.

## Process

1. **Pick a level and ontology** to audit.

2. **List all operation files** in that ontology.

3. **For each pair of scenarios**, ask:
   - What does scenario A prove that B doesn't?
   - What does scenario B prove that A doesn't?
   - If the answer to both is "nothing"—one is redundant.

4. **If you find a duplicate**:
   - State which scenario you're removing
   - State which scenario already covers it
   - Justify: what behavior does the remaining scenario test?
   - Delete the redundant file and its Rust test

5. **If you find nothing to remove**: say so. That's a valid outcome.

## Output

Either:
- Removal of one or more duplicate scenarios with clear justification
- "No duplicates found in [level/ontology]. All scenarios provide unique coverage."

Do not invent removals to look productive. Do not avoid removals to look cautious.
