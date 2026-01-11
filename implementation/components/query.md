# 9. QUERY

## PURPOSE

Plan and execute read operations (MATCH/WALK/INSPECT).

## RESPONSIBILITIES

- Generate execution plan from analyzed query
- Choose optimal index usage
- Execute plan and stream results
- Handle aggregations and sorting

## NON-RESPONSIBILITIES

- Pattern matching algorithm (that's Pattern)
- Parsing (that's Parser)
- Type checking (that's Analyzer)

## DEPENDS ON

- Graph: source of data
- Registry: schema for planning
- Pattern: pattern matching

## DEPENDED ON BY

- Session: executes user queries
- Transaction: reads within transaction

## INVARIANTS

- Query results are consistent with graph state
- Within transaction, reads see uncommitted writes

## ACCEPTANCE CRITERIA

- [ ] Plan single-type scan
- [ ] Plan indexed attribute lookup
- [ ] Plan multi-variable pattern match
- [ ] Execute and return matching rows
- [ ] Apply ORDER BY sorting
- [ ] Apply LIMIT/OFFSET
- [ ] Compute aggregates (COUNT, SUM, AVG, MIN, MAX)
- [ ] Handle GROUP BY
- [ ] WALK executes transitive traversal
- [ ] INSPECT queries Layer 0 schema

## NOTES

- Execution model: iterator-based (Volcano style)
- Operators: Scan, IndexScan, Filter, Project, Sort, Limit, Aggregate, Join
- Query within transaction merges committed + uncommitted state
- WALK is sugar over MATCH with transitive closure
- INSPECT is sugar over MATCH against Layer 0 types
