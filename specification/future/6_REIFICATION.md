## Automatic Edge Reification

Every edge `e` automatically has a shadow node `_reified(e)` accessible via:
```
-- Get reified node for edge
MATCH causes(a, b) AS c
LET c_node = reify(c)  -- returns the shadow node

-- Query edges via their reified nodes
MATCH r: _ReifiedEdge
WHERE r.edge_type = "causes" AND r.created_at > $threshold
RETURN r.edge  -- returns the actual edge
```

**Shadow node attributes:**
- `edge`: Reference to actual edge
- `edge_type`: String name
- `created_at`: Timestamp
- `targets`: List of target IDs

**Benefits:**
- Edges become first-class queryable entities
- Higher-order patterns simplified
- Provenance can attach to edge-nodes

**Implementation:** Lazy materialization — shadow nodes created on first access, cached.