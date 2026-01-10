# Test Ontologies

Nine ontologies across three complexity levels for validating the HOHG implementation.

# Summary

| Level | Ontology | Types | Edges | Constraints | Rules | Key Features |
|-------|----------|-------|-------|-------------|-------|--------------|
| **A1** | TaskManagement | 5 | 7 | 6 | 0 | Basic CRUD, dependencies |
| **A2** | SocialGraph | 5 | 11 | 6 | 0 | Social relations, visibility |
| **A3** | Inventory | 6 | 7 | 9 | 0 | Stock tracking, movements |
| **B1** | CausalReasoning | 6 | 10 | 6 | 3 | Higher-order confidence, inference |
| **B2** | DocumentCitations | 7 | 13 | 6 | 4 | Citations, claims, provenance |
| **B3** | StateMachine | 6 | 12 | 7 | 4 | State tracking, transitions |
| **C1** | SelfDescribingKB | 8 | 15 | 5 | 5 | Self-reference, meta-knowledge |
| **C2** | ProofInference | 9 | 16 | 6 | 5 | Formal proofs, validity |
| **C3** | AgentGoals | 10 | 18 | 7 | 7 | BDI agents, planning |

---

## Feature Coverage

| Feature | A1 | A2 | A3 | B1 | B2 | B3 | C1 | C2 | C3 |
|---------|----|----|----|----|----|----|----|----|-----|
| Node types | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Inheritance | | | | ✓ | | | | ✓ | |
| Attributes | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Default values | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Required/unique | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Edge attributes | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Union types in edges | | | | ✓ | | | ✓ | | ✓ |
| Simple constraints | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| EXISTS in constraints | | | ✓ | | ✓ | ✓ | ✓ | ✓ | ✓ |
| NOT EXISTS | | | | | | | | | |
| Higher-order edges | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| edge<any> | | | | | | | ✓ | | |
| Basic rules | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Rule with CREATE | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Rule with SET | | | | | ✓ | ✓ | ✓ | ✓ | ✓ |
| Rule with DELETE | | | | ✓ | | | | | |
| Rule priority | | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Rule auto:false | | | | ✓ | | | | | ✓ |
| Self-reference | | | | | | | ✓ | | |
| Meta-reasoning | | | | | | | ✓ | ✓ | ✓ |
