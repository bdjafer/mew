# 11. RULE

## PURPOSE

Trigger and execute rules reactively on mutations.

## RESPONSIBILITIES

- Find rules triggered by mutations
- Execute rule productions
- Fire rules to quiescence
- Prevent infinite loops

## NON-RESPONSIBILITIES

- Rule definition (that's Compiler)
- Mutation execution (that's Mutation)
- Transaction management (that's Transaction)

## DEPENDS ON

- Pattern: matches rule patterns
- Registry: finds triggered rules
- Mutation: executes rule productions

## DEPENDED ON BY

- Transaction: calls rule processing

## INVARIANTS

- Rules execute in priority order (highest first)
- Same (rule, bindings) pair executes at most once per transaction
- Execution terminates (quiescence or limit)

## ACCEPTANCE CRITERIA

- [ ] Find rules triggered by node creation
- [ ] Find rules triggered by edge creation
- [ ] Find rules triggered by attribute change
- [ ] Execute rules in priority order
- [ ] Execute rule production actions
- [ ] New matches from rule actions trigger more rules
- [ ] Same binding doesn't re-execute
- [ ] Depth limit prevents deep recursion
- [ ] Action limit prevents runaway execution
- [ ] Reach quiescence when no new matches

## NOTES

- Triggered rules found via Registry index
- Production actions: SPAWN, KILL, LINK, UNLINK, SET
- Limits: MAX_DEPTH = 100, MAX_ACTIONS = 10,000
- Cycle detection: hash(rule_id, bindings) in executed set
- Manual rules (auto: false) only fire via FIRE statement
