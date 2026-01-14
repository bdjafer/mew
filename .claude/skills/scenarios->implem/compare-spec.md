# COMPARE SPEC

Your goal: detect drift between implementation and specification.

## Why This Matters

The spec is the source of truth. Everything flows from it:

```
spec → scenarios → implementation → production
```

Drift—unacknowledged divergence—is dangerous. It means either:
- The implementation is wrong and will cause bugs
- The spec is wrong and will mislead future work

Both are expensive. Drift compounds. Catch it now.

## What is Drift

Drift occurs when:
- **Implementation adds behavior not in spec** — code does something the spec never defined
- **Implementation contradicts spec** — code does X, spec says Y
- **Implementation omits specified behavior** — spec says X, code doesn't do it

Drift is not always a mistake. Sometimes implementation discovers something smarter. But drift must be acknowledged and resolved—either the code changes or the spec does. Never leave it ambiguous.

## Process

1. **Identify what was implemented**. Read the diff or PR scope. What behavior was added or changed?

2. **Find the corresponding spec**. Look in `specs/specification/*.md` for the section that should define this behavior. Be precise—section number, exact text.

3. **Compare carefully**:
   - Does the spec define this behavior? If not → **undocumented addition**
   - Does the spec say something different? If so → **contradiction**
   - Does the implementation miss something the spec requires? → **incomplete implementation**
   - Does the spec match the implementation exactly? → **no drift**

4. **If drift exists, classify it**:

   | Type | Meaning | Resolution |
   |------|---------|------------|
   | Spec incomplete | Behavior is correct, spec didn't cover it | Update spec |
   | Spec outdated | Implementation found better approach | Update spec |
   | Spec wrong | Original spec had an error | Update spec |
   | Implementation wrong | Code deviated from correct spec | Fix code |
   | Implementation incomplete | Code doesn't do what spec requires | Fix code |

5. **Be honest about which is wrong**. Don't default to "spec is wrong" because it's easier than fixing code. Don't default to "code is wrong" because specs feel authoritative. Evaluate on merit.

## Output

**No drift:**
> No drift detected. Implementation matches spec [section reference].

**Drift detected:**
> Drift detected.
> 
> **Spec says** (§X.Y): "[exact quote]"
> 
> **Implementation does**: [description]
> 
> **Classification**: [spec wrong | impl wrong | spec incomplete | ...]
> 
> **Recommendation**: [update spec | fix implementation]
> 
> **Rationale**: [why this is the correct resolution]

If you're uncertain which is correct, say so. Escalate rather than guess—wrong resolution here cascades everywhere.
