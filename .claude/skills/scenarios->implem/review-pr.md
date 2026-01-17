# REVIEW PR

You are reviewing the current PR. The PR context is already set.

## STEP 1: GET THE DIFF

```bash
gh pr diff
```

## STEP 2: EVALUATE THE CODE

Read the diff through the eyes of a senior engineer who:
- Writes code that reads like well-structured prose
- Prefers simplicity over abstraction
- Asks "why is this here?" before "how does this work?"
- Sees a 200-line change for a 20-line problem and raises an eyebrow
- Is especially wary of changed test expectations
- Doesn't invent issues to seem thorough

Check for:
- Unnecessary complexity or over-abstraction
- Code that should be deleted (unused, redundant)
- Violations of existing patterns in the codebase
- Missing error handling at boundaries
- Security issues

## STEP 3: POST RESULT AND UPDATE LABELS

### If APPROVED (no issues):

```bash
gh pr comment --body "## Review: Approved

Code is clean, correct, and follows existing patterns. No issues found."

gh pr edit --add-label "gate/quality-passed"
gh pr edit --remove-label "agent/needs-review"
```

### If CHANGES REQUESTED (issues found):

```bash
gh pr comment --body "## Review: Changes Requested

**Issues:**
1. \`file.rs:LINE\` — [specific issue]
2. \`file.rs:LINE\` — [specific issue]

**Action required:** Address these issues before merge."

gh pr edit --add-label "agent/needs-revision"
gh pr edit --remove-label "agent/needs-review"
```

## IMPORTANT

- You MUST run the gh commands to post comments and update labels
- Don't just analyze - execute the commands
- The workflow depends on labels being set correctly
