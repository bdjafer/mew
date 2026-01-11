# 2. JOURNAL

## 2.1 Basic Operations

### TEST: append_returns_increasing_lsn
```
GIVEN empty journal
WHEN append entry1
AND append entry2
THEN entry1.lsn < entry2.lsn
```

### TEST: sync_makes_durable
```
GIVEN journal with appended entries
WHEN sync()
THEN entries survive process restart
```

## 2.2 Recovery

### TEST: recover_committed_transaction
```
GIVEN journal with:
  - Begin(txn=1)
  - SpawnNode(txn=1, id=100, type=1, attrs={})
  - Commit(txn=1)
WHEN recover into empty graph
THEN graph contains node 100
```

### TEST: recover_skips_uncommitted
```
GIVEN journal with:
  - Begin(txn=1)
  - SpawnNode(txn=1, id=100, ...)
  - Begin(txn=2)
  - SpawnNode(txn=2, id=200, ...)
  - Commit(txn=2)
  (txn=1 never committed)
WHEN recover into empty graph
THEN graph contains node 200
AND graph does NOT contain node 100
```

### TEST: recover_aborted_transaction
```
GIVEN journal with:
  - Begin(txn=1)
  - SpawnNode(txn=1, id=100, ...)
  - Abort(txn=1)
WHEN recover into empty graph
THEN graph does NOT contain node 100
```

### TEST: survive_kill_9
```
GIVEN running system with committed transaction creating node A
WHEN kill -9 the process
AND restart and recover
THEN graph contains node A
```