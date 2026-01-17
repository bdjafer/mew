# MEW Specification: Part 7 — Blockchain Integration

**Version:** 1.0  
**Status:** Design Specification  
**Purpose:** Decentralized ontology governance via blockchain ratification

---

# 1. Motivation

## 1.1 The Problem MEW Solves

MEW provides a substrate where **structure determines possibility**. Types define what can exist. Constraints define what must be true. Rules define automatic consequences.

But who decides the structure?

In a single-operator deployment, the answer is simple: the operator. They control the node, they control the ontology. This is **centralized mode**.

In a multi-party context, this breaks down:

| Scenario | Problem |
|----------|---------|
| Federated communities | No single party should control what concepts exist |
| DAOs | Members must collectively govern their own rules |
| Public knowledge graphs | Schema changes affect all participants |
| Digital jurisdictions | Laws (constraints) require legitimate authority |

**The gap:** MEW has no native mechanism for collective ontology governance.

## 1.2 Why Blockchain

Blockchain provides properties MEW lacks:

| Property | What it enables |
|----------|-----------------|
| **Consensus without trust** | Parties agree on canonical ontology without trusting each other |
| **Immutable history** | Complete audit trail of how rules evolved |
| **Permissionless verification** | Anyone can verify which ontology is legitimate |
| **Credible neutrality** | No party can unilaterally change the constitution |

Blockchain does **not** replace MEW. It governs **which MEW ontology is canonical**.

## 1.3 The Core Insight

In MEW, the ontology is not metadata. It is the **constitution**.

- **Types** → What categories of things can exist (ontological legislation)
- **Constraints** → What must always be true (laws)
- **Rules** → Automatic consequences of states (enforcement mechanisms)

Changing the ontology changes what is **possible**, not just what is **true**.

This makes ontology governance fundamentally different from typical smart contract governance, which operates on state within fixed rules. MEW governance operates on the rules themselves.

---

# 2. Architecture

## 2.1 Separation of Concerns

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MEW LAYER                                    │
│                                                                      │
│   • Rich computation (pattern matching, rule execution)             │
│   • Fast reads and writes                                           │
│   • Complex queries                                                 │
│   • Local state management                                          │
│                                                                      │
│   Does NOT provide: consensus, immutability, decentralized trust    │
└─────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ Ontology changes require
                                 │ external authorization
                                 │
┌────────────────────────────────▼────────────────────────────────────┐
│                      BLOCKCHAIN LAYER                                │
│                                                                      │
│   • Canonical ontology hash (which structure is legitimate)         │
│   • Amendment records (how structure evolved)                       │
│   • Voting/ratification (who approved changes)                      │
│   • Dispute resolution (which nodes are following the rules)        │
│                                                                      │
│   Does NOT provide: storage, computation, query, instance data      │
└─────────────────────────────────────────────────────────────────────┘
```

**Key principle:** The blockchain stores only hashes and votes. The full ontology lives off-chain (IPFS, distributed storage). MEW nodes verify that their local ontology matches the chain's canonical hash.

## 2.2 Operating Modes

MEW instances operate in one of two modes:

### Centralized Mode

```
┌─────────────────────────────────────────┐
│            CENTRALIZED MODE             │
├─────────────────────────────────────────┤
│                                         │
│   Operator has full control             │
│   No external authorization required    │
│   No blockchain dependency              │
│                                         │
│   extendOntology(text) → always allowed │
│                                         │
│   Use cases:                            │
│   • Personal knowledge graphs           │
│   • Single-org deployments              │
│   • Development/testing                 │
│   • Research systems                    │
│                                         │
└─────────────────────────────────────────┘
```

### Decentralized Mode

```
┌─────────────────────────────────────────┐
│            DECENTRALIZED MODE           │
├─────────────────────────────────────────┤
│                                         │
│   Ontology changes require proof        │
│   Proof = on-chain ratification         │
│   Node follows community consensus      │
│                                         │
│   extendOntology(text, proof)           │
│     → verify proof against chain        │
│     → reject if not ratified            │
│                                         │
│   Use cases:                            │
│   • DAOs                                │
│   • Federated communities               │
│   • Public infrastructure               │
│   • Digital jurisdictions               │
│                                         │
└─────────────────────────────────────────┘
```

The mode is set at initialization and defines the **root of trust**:
- Centralized: the operator
- Decentralized: the blockchain contract

## 2.3 What Lives Where

| Component | Location | Rationale |
|-----------|----------|-----------|
| Instance data (nodes, edges) | MEW only | Too large, too dynamic for chain |
| Full ontology text | Distributed storage (IPFS) | Moderate size, needs availability |
| Ontology hash | Blockchain | Small, needs consensus |
| Amendment votes | Blockchain | Needs transparency, verifiability |
| Amendment history | Blockchain | Immutable audit trail |
| Voting eligibility | Both | Defined in ontology, verified on chain |

---

# 3. Governance Model

## 3.1 What Can Be Governed

### Types (Ontological Legislation)

Voting on whether a category of entity can exist:

```
Amendment #12: Recognize Guilds

+ node Guild {
+   name: String [required, unique],
+   charter: String,
+   founded: Timestamp
+ }
+ 
+ edge member_of(Citizen, Guild) {
+   role: String,
+   joined: Timestamp
+ }
```

Before ratification: Guilds cannot exist in this world.  
After ratification: Guilds are a valid concept, can be instantiated.

### Constraints (Laws)

Voting on what must be true:

```
Amendment #23: Limit Council Seats

+ constraint max_council_seats:
+   c: Citizen, holds_seat(c, _) AS seats
+   => COUNT(seats) <= 3
+   [message: "Citizens may hold at most 3 council seats"]
```

Once ratified, this is **automatically enforced**. The engine rejects any mutation that would create a 4th seat. No separate enforcement process.

### Rules (Automatic Governance)

Voting on automatic consequences:

```
Amendment #31: Automatic Reputation Decay

+ rule reputation_decay:
+   c: Citizen
+   WHERE c.last_active < now() - 90.days
+     AND c.reputation > 0
+   =>
+   SET c.reputation = c.reputation - 1
```

Once ratified, this fires automatically whenever conditions match. The ontology encodes the policy; execution is mechanical.

### Attribute Modifiers

Voting on data requirements:

```
Amendment #8: Require Proposal Rationale

  node Proposal {
    title: String [required],
    description: String,
-   rationale: String
+   rationale: String [required, min_length: 100]
  }
```

After ratification, proposals without sufficient rationale cannot be created.

## 3.2 What Cannot Be Governed

**Layer 0 is immutable.** The meta-ontology that defines what ontologies are cannot be changed through governance. This is the constitutional foundation:

```
Layer 0 (Fixed)
├── _NodeType, _EdgeType, _ConstraintDef, _RuleDef
├── _PatternDef, _VarDef, _Expr, ...
├── Meta-constraints (no inheritance cycles, etc.)
└── Cannot be modified by any amendment
    
User Ontology (Governed)
├── Types, constraints, rules defined by community
├── Subject to amendment process
└── Changes require ratification
```

**Rationale:** If Layer 0 could be amended, the meaning of "ontology" itself becomes unstable. The system needs a fixed foundation to reason about changes to anything else.

## 3.3 Amendment Semantics

Amendments are **monotonic where possible**:

| Change Type | Semantics | Complexity |
|-------------|-----------|------------|
| Add type | Safe — new things can exist | Low |
| Add constraint | May invalidate existing data | Medium |
| Add rule | May trigger cascading changes | Medium |
| Modify type | Requires migration strategy | High |
| Remove type | Requires orphan handling | High |
| Tighten constraint | May invalidate existing data | High |
| Loosen constraint | Safe — more things allowed | Low |

Non-additive changes require **migration specifications**:

```
Amendment #45: Rename "Member" to "Citizen"

Migration:
  1. Create new type Citizen with same structure
  2. For each existing Member m:
     - Spawn Citizen with m's attributes
     - Relink all edges to new node
     - Kill original Member
  3. Remove Member type

Grace period: 30 days (both types valid)
```

---

# 4. Amendment Lifecycle

## 4.1 States

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  DRAFT  │────▶│ PROPOSED│────▶│ VOTING  │────▶│ RATIFIED│
└─────────┘     └─────────┘     └─────────┘     └────┬────┘
                     │               │               │
                     │               │               ▼
                     │               │          ┌─────────┐
                     │               └─────────▶│ REJECTED│
                     │                          └─────────┘
                     │
                     ▼
                ┌─────────┐
                │WITHDRAWN│
                └─────────┘
```

## 4.2 Process

```
┌─────────────────────────────────────────────────────────────────────┐
│                      AMENDMENT LIFECYCLE                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. DRAFT (off-chain)                                               │
│     │                                                                │
│     │  Author writes ontology change                                │
│     │  Community reviews, discusses                                 │
│     │  Simulations run (see §4.3)                                   │
│     │                                                                │
│     ▼                                                                │
│  2. PROPOSE (on-chain)                                              │
│     │                                                                │
│     │  Author publishes ontology text to distributed storage        │
│     │  Author submits (hash, storage_uri) to governance contract    │
│     │  Proposal registered, voting period begins                    │
│     │                                                                │
│     ▼                                                                │
│  3. VOTE (on-chain)                                                 │
│     │                                                                │
│     │  Eligible participants cast votes                             │
│     │  Voting rules defined by current ontology                     │
│     │  Transparency: all votes public                               │
│     │                                                                │
│     ▼                                                                │
│  4. RESOLVE (on-chain)                                              │
│     │                                                                │
│     ├──▶ RATIFIED: threshold met                                    │
│     │      Contract updates canonical hash                          │
│     │      Event emitted                                            │
│     │                                                                │
│     └──▶ REJECTED: threshold not met                                │
│            No state change                                          │
│            Amendment recorded as rejected                           │
│                                                                      │
│     ▼                                                                │
│  5. APPLY (off-chain, all nodes)                                    │
│     │                                                                │
│     │  Nodes observe ratification event                             │
│     │  Nodes fetch ontology from storage                            │
│     │  Nodes verify hash matches                                    │
│     │  Nodes apply through authorized path                          │
│     │  New ontology active                                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.3 Simulation Before Voting

A key advantage of explicit ontologies: **proposed changes can be simulated**.

Before voting, any participant can:

```
1. Fork current graph state into simulation branch
2. Apply proposed amendment to simulation
3. Analyze impact:
   
   IMPACT REPORT: Amendment #23 (Limit Council Seats)
   ═══════════════════════════════════════════════════
   
   Constraint violations if applied now:
   • 7 citizens currently hold 4+ seats
   • 2 citizens currently hold 5 seats
   
   Affected entities:
   • citizen:alice (4 seats → must release 1)
   • citizen:bob (5 seats → must release 2)
   • ... 
   
   Blocked operations (examples):
   • LINK holds_seat(carol, seat_47) — carol already has 3
   
   Migration required: YES
   Suggested grace period: 14 days
   
4. Share analysis with community
5. Vote based on evidence, not speculation
```

This makes deliberation **evidence-based**. "If we pass this, X happens" is computable.

## 4.4 Voting Rules

The voting mechanism itself is defined in the ontology:

```
node Amendment {
  hash: String [required],
  storage_uri: String [required],
  proposer: Citizen [required],
  proposed_at: Timestamp [required],
  voting_ends: Timestamp [required],
  status: String [required]  // "voting", "ratified", "rejected"
}

edge votes_on(Citizen, Amendment) {
  support: Bool [required],
  voted_at: Timestamp [required]
}

constraint one_vote_per_amendment:
  c: Citizen, a: Amendment,
  votes_on(c, a) AS v1,
  votes_on(c, a) AS v2
  WHERE v1.id != v2.id
  => false

rule ratify_amendment:
  a: Amendment
  WHERE a.status = "voting"
    AND now() > a.voting_ends
    AND count_votes_for(a) > count_votes_against(a)
    AND count_votes_for(a) >= quorum(a)
  =>
  SET a.status = "ratified"
```

**Bootstrap problem:** The initial voting rules must be defined in the genesis ontology. Subsequent amendments can modify voting rules, but this requires passing under the current rules.

---

# 5. Trust Model

## 5.1 What Nodes Trust

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TRUST HIERARCHY                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  FULLY TRUSTED (axiomatic)                                          │
│  ├── Layer 0 specification (hardcoded in engine)                    │
│  ├── Governance contract address (configured at init)               │
│  └── Blockchain finality rules (e.g., 12 confirmations)             │
│                                                                      │
│  VERIFIED (cryptographically)                                       │
│  ├── On-chain state (via RPC + proofs)                              │
│  ├── Ontology text matches hash (content addressing)                │
│  └── Amendment was ratified (proof verification)                    │
│                                                                      │
│  DERIVED (from above)                                               │
│  ├── Current canonical ontology                                     │
│  ├── Which amendments have passed                                   │
│  └── Whether local state is canonical                               │
│                                                                      │
│  NOT TRUSTED                                                        │
│  ├── Other nodes' claims about their state                          │
│  ├── Unverified ontology sources                                    │
│  └── Amendments without valid proofs                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Divergence Detection

A node can verify whether another node is following canonical ontology:

```
Node A claims ontology hash: 0xabc
Canonical hash on chain:     0xabc
→ Node A is canonical ✓

Node B claims ontology hash: 0xdef
Canonical hash on chain:     0xabc
→ Node B has diverged ✗
```

Divergence is **detectable** and **provable**. Any party can demonstrate that a node is not following community consensus by comparing its claimed hash to the chain.

## 5.3 Failure Modes

| Failure | Detection | Consequence |
|---------|-----------|-------------|
| Node applies unratified amendment | Hash diverges from chain | Node excluded from canonical community |
| Node ignores ratified amendment | Hash diverges from chain | Node excluded from canonical community |
| Node corrupts instance data | Constraint violations or inconsistent queries | Local problem, doesn't affect consensus |
| Governance contract compromised | Depends on chain security | Catastrophic (fork required) |
| Storage layer unavailable | Nodes cannot fetch new ontologies | Stall (cannot apply amendments) |

## 5.4 Honest Minority Guarantee

Even if most nodes are malicious:
- A single honest node can verify it has the canonical ontology
- A single honest node can prove others have diverged
- No coalition can force acceptance of unratified amendments
- History of ratifications is immutable and auditable

---

# 6. Multi-Community Dynamics

## 6.1 Community Identity

A community is defined by its governance contract:

```
Community = (GenesisOntology, GovernanceContract)
```

Two nodes are in the same community iff they trust the same governance contract and have the same genesis hash.

## 6.2 Forking

A community can fork when consensus breaks down:

```
                    Original Community
                    Contract: 0x111
                    Ontology: 0xabc
                           │
                           │ Contentious amendment #50
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
     Community A                 Community B
     Contract: 0x111             Contract: 0x222 (new)
     Ontology: 0xdef             Ontology: 0xabc
     (accepted #50)              (rejected #50, forked)
```

Forking creates a new governance contract with:
- Snapshot of pre-fork ontology as genesis
- New governance rules (possibly different voting thresholds)
- Independent amendment history going forward

## 6.3 Cross-Community Interaction

When communities share Layer 0 but have different ontologies:

```
┌─────────────────┐                    ┌─────────────────┐
│  Community A    │                    │  Community B    │
│                 │                    │                 │
│  node Citizen   │                    │  node Citizen   │
│  node Guild     │ ←── shared ───→    │  node Guild     │
│  node Proposal  │    concepts        │  node Proposal  │
│                 │                    │                 │
│  constraint X   │                    │  constraint Y   │
│  (different)    │ ←── different ──→  │  (different)    │
│                 │      rules         │                 │
└─────────────────┘                    └─────────────────┘
```

**Interoperability questions:**
- Can a Citizen from A participate in B? (Depends on B's ontology)
- Can data migrate between communities? (If types are compatible)
- Can communities recognize each other's credentials? (Requires explicit bridges)

These are not automatically solved by blockchain integration. They require explicit **bridge ontologies** or **mutual recognition agreements**, which are themselves subject to governance.

## 6.4 Ontology Inheritance Across Communities

A community can declare its ontology **extends** another:

```
Community B genesis:
  
  extends community_a.ontology@v2.3
  
  // Additional types specific to B
  node LocalConcept { ... }
```

This creates a dependency:
- B inherits all of A's types, constraints, rules
- B can add but not remove/modify inherited elements
- A's amendments may propagate to B (governance decision)

---

# 7. Synchronization

## 7.1 Node Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                       NODE SYNCHRONIZATION                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  INITIALIZE                                                         │
│  │                                                                   │
│  │  Configure governance contract address                           │
│  │  Load genesis ontology                                           │
│  │  Verify genesis hash matches chain                               │
│  │                                                                   │
│  ▼                                                                   │
│  CATCH UP                                                           │
│  │                                                                   │
│  │  Fetch all ratified amendments since genesis                     │
│  │  Apply each in order with verified proofs                        │
│  │  Reach current canonical state                                   │
│  │                                                                   │
│  ▼                                                                   │
│  OPERATE                                                            │
│  │                                                                   │
│  │  Process queries and mutations normally                          │
│  │  Watch chain for new ratification events                         │
│  │                                                                   │
│  ▼                                                                   │
│  ON RATIFICATION EVENT                                              │
│  │                                                                   │
│  │  Fetch new ontology from storage                                 │
│  │  Verify hash matches ratified hash                               │
│  │  Build proof from chain state                                    │
│  │  Apply through authorized path                                   │
│  │  Resume normal operation                                         │
│  │                                                                   │
│  └──▶ (return to OPERATE)                                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7.2 Consistency Guarantees

| Guarantee | Mechanism |
|-----------|-----------|
| All canonical nodes have same ontology | Same chain, same hash, same content |
| Amendment order is total | Chain provides total ordering |
| No amendment can be "undone" | Chain immutability |
| Nodes cannot silently diverge | Hash comparison detects divergence |

## 7.3 Availability vs. Consistency

When chain is unavailable:
- Nodes continue operating with current ontology
- New amendments cannot be applied
- Nodes cannot verify they are still canonical
- Conservative approach: read-only mode until chain accessible

When storage is unavailable:
- Nodes know an amendment was ratified (from chain)
- Nodes cannot fetch the ontology text
- Amendment application stalls until storage recovers
- Mitigation: multiple storage backends, local caching

---

# 8. Security Considerations

## 8.1 Attack Vectors

| Attack | Mitigation |
|--------|------------|
| Malicious amendment | Requires majority vote; simulation reveals impact |
| Governance contract exploit | Standard smart contract security; upgrade mechanisms |
| Storage poisoning (wrong content for hash) | Content addressing; hash verification |
| Eclipse attack on node | Multiple RPC endpoints; checkpoint verification |
| Bribery/vote buying | Transparent voting; reputation systems; time-locks |
| Rapid-fire amendments | Rate limiting; cooling-off periods |

## 8.2 Governance Attacks

```
ATTACK: Hostile Takeover
════════════════════════

1. Attacker acquires voting majority
2. Attacker proposes amendment removing others' voting rights
3. Amendment passes
4. Attacker has permanent control

MITIGATIONS:
• Supermajority requirements for governance changes
• Time-locks allowing exit before hostile amendments apply
• Veto mechanisms for constitutional changes
• Quadratic voting to limit plutocratic control
```

```
ATTACK: Rushed Amendment
════════════════════════

1. Attacker proposes complex amendment
2. Short voting period
3. Community doesn't have time to analyze
4. Harmful amendment passes

MITIGATIONS:
• Minimum voting periods
• Mandatory simulation reports
• Escalating thresholds for complex changes
• Community review requirements
```

## 8.3 Safety Properties

The system should guarantee:

1. **No unratified ontology changes** — Governed nodes reject amendments without valid proofs

2. **Verifiable canonicality** — Any party can check whether a node follows consensus

3. **Auditable history** — Complete record of all amendments and votes

4. **Deterministic application** — Same amendments applied in same order yield same ontology

5. **Safe exit** — Participants can leave community before unwanted amendments take effect

---

# 9. Bootstrapping

## 9.1 Genesis

Every governed community begins with a **genesis event**:

```
GENESIS
═══════

1. Author writes genesis ontology
   • Core types (Citizen, Proposal, Vote, ...)
   • Initial constraints
   • Voting rules for amendments
   
2. Deploy governance contract
   • Genesis ontology hash as initial state
   • Initial citizen set (or open registration rules)
   
3. Publish genesis ontology to storage
   • Content-addressed, immutable
   
4. First nodes initialize
   • Trust contract address
   • Load genesis ontology
   • Verify hash matches contract
   
5. Community is live
   • Normal amendment process begins
```

## 9.2 The Governance Ontology

The genesis ontology must include governance primitives:

```
// Who can participate
node Citizen {
  address: String [required, unique],  // blockchain address
  joined: Timestamp,
  reputation: Int = 0
}

// What can be proposed
node Amendment {
  hash: String [required],
  storage_uri: String [required],
  proposer: Citizen,
  proposed_at: Timestamp,
  voting_ends: Timestamp,
  status: String  // "voting" | "ratified" | "rejected"
}

// How votes are cast
edge votes_on(Citizen, Amendment) {
  support: Bool,
  weight: Int = 1,
  cast_at: Timestamp
}

// Voting rules (example)
constraint voting_requires_citizenship:
  c: Citizen, a: Amendment, votes_on(c, a)
  => c.joined < a.proposed_at  // must be citizen before proposal

rule tally_and_ratify:
  a: Amendment
  WHERE a.status = "voting" AND now() > a.voting_ends
  =>
  // Tally logic, update status
```

**Note:** The on-chain contract mirrors these rules. The ontology is the specification; the contract is the enforcement for the ratification step itself.

## 9.3 Meta-Governance

How do you amend the amendment process?

```
Amendment #99: Change voting threshold from 50% to 66%

  constraint amendment_passes:
    a: Amendment
    WHERE a.status = "voting" AND now() > a.voting_ends
-   => vote_ratio(a) > 0.50
+   => vote_ratio(a) > 0.66
```

This amendment must pass under **current** rules (50%) to change future rules to 66%.

**Constitutional protections** can make certain rules harder to change:

```
// Requires 75% supermajority to modify
constraint core_rights [protected: supermajority(0.75)]:
  ...

// Requires unanimous consent to modify
constraint fundamental [protected: unanimous]:
  ...
  
// Cannot be modified at all (only by Layer 0 change, i.e., never)
constraint immutable [protected: never]:
  ...
```

---

# 10. Design Rationale

## 10.1 Why Not Full On-Chain?

Running MEW entirely on-chain would mean:
- Every pattern match costs gas
- Rule execution (potentially thousands of operations) is prohibitively expensive
- Storage costs for rich graph structure are extreme
- Latency makes interactive use impossible

The hybrid design captures **what chains are good at** (consensus, immutability, transparency) while leaving **what MEW is good at** (computation, queries, storage) off-chain.

## 10.2 Why Not Pure Off-Chain Consensus?

Alternatives like PBFT among known validators:
- Requires trusted validator set
- No permissionless verification
- History can be rewritten by validator collusion
- Joining requires permission

Blockchain provides **permissionless verification**: anyone can check whether an ontology is canonical without trusting any particular party.

## 10.3 Why Hash-Based Canonicality?

The chain stores only hashes, not full ontologies:

| Approach | Storage Cost | Verification | Privacy |
|----------|--------------|--------------|---------|
| Full ontology on-chain | Very high | Easy | None (fully public) |
| Hash on-chain | Minimal | Requires fetch | Ontology can be access-controlled |

Hash-based approach enables:
- Private communities (ontology only shared with members)
- Efficient chain usage
- Same verification guarantees (content addressing)

## 10.4 Why Not Token-Based Governance?

Token voting has known problems:
- Plutocracy (wealth = power)
- Low participation (rational ignorance)
- Vote buying (liquid markets)

MEW's ontology can express **arbitrary voting rules**:

```
// Quadratic voting
edge votes_on(c, a) { weight = sqrt(c.tokens_committed) }

// Conviction voting  
edge votes_on(c, a) { weight = tokens * time_locked }

// Reputation-weighted
edge votes_on(c, a) { weight = c.reputation }

// One-person-one-vote with identity
edge votes_on(c, a) { weight = 1 }  // requires sybil resistance
```

The governance mechanism is itself governed. Communities can experiment.

---

# 11. Future Directions

## 11.1 Cross-Chain Governance

A community might want governance across multiple chains for resilience:

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  Chain A     │  │  Chain B     │  │  Chain C     │
│  (primary)   │  │  (backup)    │  │  (backup)    │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
                         ▼
               Canonical = 2-of-3 agreement
```

## 11.2 Recursive Governance

Communities governing communities:

```
Meta-Community (governs governance rules)
├── Community A (follows meta-community rules)
├── Community B (follows meta-community rules)
└── Community C (follows meta-community rules)
```

Layer 0 is fixed. But a meta-community could govern the shared base ontology that multiple communities extend.

## 11.3 Dispute Resolution

When participants disagree about constraint interpretation:

```
Dispute: Does action X violate constraint C?

1. Disputant submits claim + evidence to chain
2. Arbitration process (defined in ontology)
3. Ruling recorded on chain
4. Ruling becomes precedent (if ontology supports precedent)
```

This creates **on-chain jurisprudence** grounded in formal ontology.

## 11.4 Formal Verification

Because ontologies are formal:
- Amendments can be **proven** to preserve certain properties
- Type-safety of migrations can be verified
- Constraint satisfiability can be checked

"This amendment provably does not remove any citizen's voting rights" becomes a checkable claim.

---

# 12. Summary

## 12.1 What Blockchain Integration Provides

| Capability | Mechanism |
|------------|-----------|
| Decentralized ontology governance | On-chain voting, ratification |
| Canonical schema consensus | Hash commitment on chain |
| Immutable governance history | Chain immutability |
| Permissionless verification | Any party can check canonicality |
| Credible neutrality | No single party controls rules |
| Evidence-based deliberation | Deterministic simulation |
| Automatic enforcement | Constraints in ontology |

## 12.2 What Remains in MEW

| Function | Location |
|----------|----------|
| Instance data storage | MEW only |
| Query execution | MEW only |
| Constraint checking | MEW only |
| Rule execution | MEW only |
| Rich computation | MEW only |
| Ontology compilation | MEW only |

## 12.3 The Bridge

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  MEW's authorization gate requires cryptographic proof that any     │
│  ontology change has been ratified on-chain before accepting it.    │
│                                                                      │
│  The blockchain is the arbiter of which ontology is canonical.      │
│  MEW nodes are followers that verify their compliance.              │
│                                                                      │
│  This creates decentralized governance of typed, relational,        │
│  constraint-based worlds with automatic enforcement and             │
│  deterministic simulation.                                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

*End of Part 7: Blockchain Integration*