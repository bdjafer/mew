# 12. TRANSACTION

## PURPOSE

Provide ACID transactions and orchestrate mutation flow.

## RESPONSIBILITIES

- Track pending changes (transaction buffer)
- Orchestrate: mutation → rules → constraints
- Implement BEGIN/COMMIT/ROLLBACK
- Coordinate with Journal for durability
- Provide read-your-writes isolation

## NON-RESPONSIBILITIES

- Individual mutation logic (that's Mutation)
- Individual constraint logic (that's Constraint)
- Individual rule logic (that's Rule)
- WAL format (that's Journal)

## DEPENDS ON

- Graph: applies committed changes
- Mutation: executes individual mutations
- Constraint: validates constraints
- Rule: executes rules
- Journal: writes WAL

## DEPENDED ON BY

- Session: manages transactions

## INVARIANTS

- Committed transaction is durable
- Aborted transaction has no effect
- Within transaction, reads see own writes
- Constraint violation aborts entire transaction

## ACCEPTANCE CRITERIA

- [ ] BEGIN creates new transaction
- [ ] Operations within transaction go to buffer
- [ ] Query within transaction sees buffered writes
- [ ] COMMIT: run deferred constraints
- [ ] COMMIT: write WAL
- [ ] COMMIT: apply to graph
- [ ] COMMIT failure → rollback
- [ ] ROLLBACK discards buffer
- [ ] Auto-commit mode for single statements
- [ ] Nested transactions via savepoints

## NOTES

- Transaction buffer: created/deleted nodes, created/deleted edges, modified attrs
- Commit sequence: deferred constraints → WAL write → apply to graph
- Read-your-writes: query merges graph state + buffer
- Isolation level: Read Committed for v1 (simplest)
