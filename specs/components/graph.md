# 1. GRAPH

## PURPOSE

Store and retrieve nodes and edges with indexed access.

## RESPONSIBILITIES

- Allocate unique IDs for nodes and edges
- Store node data (type, attributes, version)
- Store edge data (type, targets, attributes, version)
- Maintain indexes for efficient lookup
- Provide snapshot capability for consistent reads

## NON-RESPONSIBILITIES

- Durability (that's Journal)
- Transaction isolation (that's Transaction)
- Type validation (that's Mutation)
- Schema knowledge (that's Registry)

## DEPENDS ON

- (none — foundation component)

## DEPENDED ON BY

- Journal: persists graph state
- Pattern: matches against graph
- Mutation: writes to graph
- Query: reads from graph
- Compiler: writes Layer 0
- Transaction: manages graph state

## INVARIANTS

- Node/Edge IDs are unique and immutable once assigned
- Deleting a node deletes all edges involving that node
- Deleting an edge deletes all higher-order edges about that edge
- Indexes are always consistent with stored data

## ACCEPTANCE CRITERIA

- [ ] Create node with type and attributes → returns NodeId
- [ ] Get node by ID → returns Node or None
- [ ] Delete node → removes node and all incident edges
- [ ] Create edge with type, targets, attributes → returns EdgeId
- [ ] Get edge by ID → returns Edge or None
- [ ] Delete edge → removes edge and all higher-order edges about it
- [ ] Set attribute on node/edge → updates value
- [ ] Find nodes by type → returns iterator
- [ ] Find nodes by attribute value → returns iterator
- [ ] Find nodes by attribute range → returns iterator
- [ ] Find edges from node → returns iterator
- [ ] Find edges to node → returns iterator
- [ ] Find edges about edge → returns iterator

## NOTES

- v1 starts in-memory; file-backed storage is v1.x
- Indexes are internal implementation detail
- Higher-order edges have EdgeId in targets list
- Snapshot returns consistent view even during concurrent modifications
