# REVISE PR

Your goal: apply targeted fixes to the implementation based on feedback.

## Input

You receive feedback from one of:
- `/review-pr` — quality issues (complexity, patterns, structure)
- `/compare-spec` — drift issues (impl doesn't match spec)

## Process

1. **List the issues** identified in the feedback. Be specific—quote them.

2. **For each issue**, determine the fix:
   - What file(s) need to change?
   - What's the minimal change that addresses it?
   - Does fixing this affect anything else?

3. **Apply fixes one at a time**:
   - Make the change
   - Verify it compiles
   - Verify tests still pass
   - Move to next issue

4. **Don't over-correct**. Fix what was flagged. Don't refactor adjacent code, don't add improvements you noticed along the way. Stay scoped.

5. **If a fix is unclear or risky**, say so. Better to ask than to guess wrong.

## Constraints

- Fix only what was identified. No drive-by changes.
- If fixing one issue conflicts with another, flag it.
- **Pushback is valid.** If you're confident a requested change is wrong—would make the code worse, contradicts spec, or misunderstands intent—don't apply it. State your case with justification. Reviewers aren't infallible.

## Output

- List of changes made, mapped to the issues they address
- Any issues you couldn't resolve (with explanation)
- Confirmation: compiles, tests pass
