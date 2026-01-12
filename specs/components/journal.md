# 2. JOURNAL

## PURPOSE

Ensure durability through write-ahead logging and crash recovery.

## RESPONSIBILITIES

- Append mutation records to durable log before commit
- Sync log to disk on transaction commit
- Replay log on startup to recover state
- Manage log segments (rotation, cleanup)

## NON-RESPONSIBILITIES

- Graph operations (that's Graph)
- Transaction semantics (that's Transaction)
- Deciding what to log (that's Transaction)

## DEPENDS ON

- Graph: replays mutations into graph during recovery

## DEPENDED ON BY

- Transaction: writes WAL records on commit

## INVARIANTS

- Committed transactions have durable WAL records
- Recovery produces same state as before crash
- WAL records are ordered by LSN

## ACCEPTANCE CRITERIA

- [ ] Append entry → returns LSN
- [ ] Sync → ensures all appended entries are durable
- [ ] Recover → replays committed transactions into graph
- [ ] Uncommitted transactions are not replayed
- [ ] Process survives kill -9 and recovers correctly

## NOTES

- WAL entry types: Begin, Commit, Abort, SpawnNode, KillNode, LinkEdge, UnlinkEdge, SetAttr
- Each entry includes txn_id for grouping
- Recovery: replay all entries, apply only committed transactions
- v1 can use simple append-only file; segments are optimization
