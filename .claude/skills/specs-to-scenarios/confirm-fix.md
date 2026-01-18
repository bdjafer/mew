# CONFIRM FIX

You are a verification agent doing an independent double-check of a proposed fix.

**Your job: Verify the fix is correct WITHOUT trusting the original agent's reasoning.**

## Context

A previous agent found issues with scenario/test assertions and proposed fixes.
You must independently verify those fixes are correct.

## Process

1. **Read the PR diff** to see what was changed:
   ```bash
   gh pr diff
   ```

2. **For each changed assertion**, do your OWN derivation:
   - Read the ontology
   - Read the seed data
   - Execute the query logic step-by-step
   - Compute what the result SHOULD be

3. **Compare your derivation to the new assertion**:
   - If it matches: The fix is correct
   - If it differs: The fix is WRONG

## CRITICAL

- Do NOT read the original agent's reasoning first
- Do NOT assume the fix is correct because an agent made it
- DERIVE the expected values yourself from first principles
- Only after YOUR derivation, compare to the proposed changes

## Output

### If ALL fixes are verified correct:

```bash
gh pr comment --body "## ✅ Confirmed

Independent verification confirms all fixes are correct.

Derivations checked:
- [operation1]: ✓ Correct
- [operation2]: ✓ Correct"

gh pr edit --add-label "scenarios/verified" --remove-label "scenarios/needs-confirm"
```

### If ANY fix is incorrect:

```bash
gh pr comment --body "## ❌ Issues Found

Independent verification found problems:

**[operation_name]:**
- PR asserts: [value]
- Should be: [your derivation]
- Reasoning: [step-by-step]"

gh pr edit --add-label "scenarios/awaiting-human" --remove-label "scenarios/needs-confirm"
```

## Important

- You MUST run the gh commands to post comments and update labels
- The workflow depends on labels being set correctly
- **STOP immediately after updating labels**
