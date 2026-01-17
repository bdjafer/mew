# 10. CONSTRAINT

## PURPOSE

Validate graph state against declared constraints.

## RESPONSIBILITIES

- Check immediate constraints after mutations
- Check deferred constraints at commit
- Distinguish hard (abort) vs soft (warn) constraints
- Produce meaningful violation messages

## NON-RESPONSIBILITIES

- Constraint definition (that's Compiler)
- Transaction abort (that's Transaction)
- Pattern matching (that's Pattern)

## DEPENDS ON

- Pattern: matches constraint patterns
- Registry: finds applicable constraints

## DEPENDED ON BY

- Transaction: calls constraint checking

## INVARIANTS

- Hard constraint violation always prevents commit
- All affected constraints are checked
- Constraint checking is deterministic

## ACCEPTANCE CRITERIA

- [ ] Find constraints affected by node mutation
- [ ] Find constraints affected by edge mutation
- [ ] Check immediate constraint after mutation
- [ ] Check deferred constraint at commit
- [ ] Hard constraint violation → return error
- [ ] Soft constraint violation → return warning
- [ ] Error message includes constraint name and context
- [ ] Constraint with pattern finds relevant matches

## NOTES

- Affected constraints found via Registry index
- Immediate: value constraints, required, no_self, acyclic
- Deferred: cardinality (→ 1), existence (EXISTS)
- Seeding: pattern match starts from mutated entity for efficiency
