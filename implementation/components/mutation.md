# 8. MUTATION

## PURPOSE

Execute write operations (SPAWN/KILL/LINK/UNLINK/SET).

## RESPONSIBILITIES

- Validate mutations against schema
- Apply mutations to transaction buffer
- Handle cascade deletions
- Return created IDs

## NON-RESPONSIBILITIES

- Constraint checking (that's Constraint, called by Transaction)
- Rule triggering (that's Rule, called by Transaction)
- Transaction management (that's Transaction)
- Durability (that's Journal)

## DEPENDS ON

- Graph: applies mutations
- Registry: validates types
- Pattern: evaluates default expressions

## DEPENDED ON BY

- Transaction: calls mutation operations
- Rule: executes rule productions
- Session: user mutation statements

## INVARIANTS

- Type validation before mutation
- Required attributes present on SPAWN
- Edge targets match signature types
- Cascade deletion is complete

## ACCEPTANCE CRITERIA

- [ ] SPAWN with valid type → creates node
- [ ] SPAWN with abstract type → error
- [ ] SPAWN missing required attribute → error
- [ ] SPAWN with wrong attribute type → error
- [ ] SPAWN applies default values
- [ ] KILL existing node → deletes node
- [ ] KILL deletes incident edges (cascade)
- [ ] KILL non-existent node → error
- [ ] LINK with valid signature → creates edge
- [ ] LINK with wrong target types → error
- [ ] LINK with invalid arity → error
- [ ] UNLINK existing edge → deletes edge
- [ ] UNLINK deletes higher-order edges about it
- [ ] SET valid attribute → updates value
- [ ] SET wrong type → error
- [ ] SET unknown attribute → error

## NOTES

- Mutations go to transaction buffer, not directly to graph
- Cascade: KILL node → UNLINK all edges → KILL triggers recursively
- Higher-order cascade: UNLINK edge → UNLINK all edges about it
- RETURNING clause captures created IDs
