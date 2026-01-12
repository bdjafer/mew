
# 7. PATTERN

## PURPOSE

Compile patterns, match against graph, evaluate expressions.

## RESPONSIBILITIES

- Compile pattern AST into executable form
- Find all matches of pattern in graph
- Evaluate expressions given variable bindings
- Support transitive closure (edge+, edge*)
- Support negative patterns (NOT EXISTS)

## NON-RESPONSIBILITIES

- Query planning (that's Query)
- Side effects (that's Mutation/Rule)
- Constraint checking logic (that's Constraint)

## DEPENDS ON

- Graph: source of data to match
- Registry: type information for matching

## DEPENDED ON BY

- Query: uses pattern matching
- Constraint: matches constraint patterns
- Rule: matches rule patterns

## INVARIANTS

- Matching is deterministic
- All matches are found (completeness)
- Only valid matches are returned (soundness)
- Expression evaluation is side-effect free

## ACCEPTANCE CRITERIA

- [ ] Compile single-node pattern
- [ ] Compile multi-node pattern with edges
- [ ] Compile pattern with edge alias
- [ ] Match nodes by type
- [ ] Match edges connecting nodes
- [ ] Match with WHERE condition filtering
- [ ] Match transitive closure edge+(a, b)
- [ ] Match reflexive transitive closure edge*(a, b)
- [ ] Match NOT EXISTS subpattern
- [ ] Evaluate arithmetic expressions
- [ ] Evaluate comparison expressions
- [ ] Evaluate attribute access
- [ ] Evaluate function calls (COUNT, SUM, etc.)
- [ ] Handle unbound variables in expressions → error

## NOTES

- Matching algorithm: backtracking search with index hints
- Join order determined by selectivity heuristics
- Transitive closure uses BFS with cycle detection
- Initial bindings can seed the match (for constraint/rule checking)
