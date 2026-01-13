# MEW Architecture Extension: Authorization System

**Version:** 1.0
**Status:** Design Specification
**Scope:** Authorization component and integration with existing architecture

---

# Part I: Overview

## 1.1 Summary

This document extends the MEW System Architecture to incorporate authorization. The authorization system adds:

- **1 new component**: Authorization
- **Extensions to 7 existing components**: Session, Parser, Compiler, Registry, Pattern, Query, Transaction
- **Layer 0 extensions**: Authorization rule types
- **2 distinct enforcement models**: Gating (mutations) and Filtering (observations)

The core 13-component architecture remains intact. Authorization integrates as a 14th component with well-defined interaction points.

## 1.2 Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Filtering for observations** | Matches reality — actors see their accessible world, not errors |
| **Gating for mutations** | Binary decision required — can't "partially" create a node |
| **Predicate injection** | Authorization joins query planning, not post-filtering |
| **Policy/Grant separation** | Policies compile rarely; grants are runtime edge data |
| **Default-deny** | Secure by default; explicit ALLOW required |

---

# Part II: Component Architecture

## 2.1 Updated Component Map (14 Components)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      MEW SYSTEM (with Authorization)                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    SESSION (Extended)                          │  │
│  │  • actor: Option<EntityId>                                    │  │
│  │  • BEGIN SESSION AS #actor                                    │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│                                ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    PARSER (Extended)                           │  │
│  │  • Authorization DSL grammar                                  │  │
│  │  • Session binding statements                                 │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│                                ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    ANALYZER (Extended)                         │  │
│  │  • Context function validation                                │  │
│  │  • Authorization scope checking                               │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
│                                │                                     │
│       ┌────────────────────────┼────────────────────────┐           │
│       │                        │                        │           │
│       ▼                        ▼                        ▼           │
│  ┌─────────┐            ┌─────────────┐          ┌───────────┐     │
│  │COMPILER │            │    QUERY    │          │  MUTATION │     │
│  │(Extended)│            │ (Extended)  │          │           │     │
│  │         │            │             │          │     │     │     │
│  │+Auth    │            │+Predicate   │          │     ▼     │     │
│  │ rules   │            │ injection   │          │ ┌───────┐ │     │
│  └────┬────┘            └──────┬──────┘          │ │ AUTH  │ │     │
│       │                        │                 │ │ GATE  │ │     │
│       ▼                        │                 │ └───┬───┘ │     │
│  ┌─────────┐                   │                 │     │     │     │
│  │REGISTRY │◄──────────────────┤                 └─────┼─────┘     │
│  │(Extended)│                   │                       │           │
│  │         │                   │                       │           │
│  │+AuthReg │                   ▼                       ▼           │
│  └─────────┘         ┌─────────────────────────────────────────┐   │
│       │              │            AUTHORIZATION (NEW)           │   │
│       │              │                                          │   │
│       │              │  ┌─────────────┐  ┌──────────────────┐  │   │
│       │              │  │   POLICY    │  │    PREDICATE     │  │   │
│       │              │  │  EVALUATOR  │  │    COMPILER      │  │   │
│       │              │  └─────────────┘  └──────────────────┘  │   │
│       │              │  ┌─────────────┐  ┌──────────────────┐  │   │
│       │              │  │   CACHE     │  │    DECISION      │  │   │
│       │              │  │   MANAGER   │  │    RESOLVER      │  │   │
│       │              │  └─────────────┘  └──────────────────┘  │   │
│       │              │                                          │   │
│       └──────────────┤◄─────────────────────────────────────────┘   │
│                      │                                              │
│                      ▼                                              │
│                   PATTERN (Extended)                                │
│                      │                                              │
│                      │  • EvalContext with actor/operation/target  │
│                      │  • Context functions: current_actor(), etc. │
│                      │                                              │
│       ┌──────────────┼──────────────┐                              │
│       ▼              ▼              ▼                               │
│  CONSTRAINT        RULE       TRANSACTION                           │
│                               (Extended)                            │
│                                  │                                  │
│                   • Pre-mutation auth gate                          │
│                   • System authority for rules                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Component Responsibilities (Updated)

| Component | Original Purpose | Authorization Extension |
|-----------|------------------|------------------------|
| **Session** | External interface | Actor binding, authority context |
| **Parser** | Tokenize, parse | Authorization DSL, session statements |
| **Analyzer** | Name resolution, types | Context function scope validation |
| **Compiler** | Ontology → Registry | Compile authorization rules |
| **Registry** | Schema lookup | Authorization rule lookup |
| **Pattern** | Match, evaluate | Extended EvalContext, context functions |
| **Query** | Plan, execute MATCH | Predicate injection, filtered execution |
| **Mutation** | SPAWN/KILL/LINK/UNLINK/SET | Pre-check hook invocation |
| **Transaction** | ACID orchestration | Authority propagation, rule context |
| **Authorization** | **(NEW)** | Policy evaluation, caching, decisions |

## 2.3 Dependency Graph (Updated)

```
                    Session
                       │
                       │ actor binding
                       ▼
                    Parser
                       │
                       ▼
                   Analyzer ◄──────────┐
                       │               │
          ┌────────────┼────────────┐  │
          │            │            │  │
          ▼            ▼            ▼  │
      Compiler      Query       Mutation
          │            │            │
          │            │ inject     │ gate
          │            ▼            ▼
          │      ┌─────────────────────┐
          │      │    AUTHORIZATION    │◄─────┐
          │      └──────────┬──────────┘      │
          │                 │                 │
          ▼                 │                 │
      Registry ◄────────────┤                 │
          │                 │                 │
          │                 ▼                 │
          │             Pattern               │
          │                 │                 │
          │    ┌────────────┼────────────┐    │
          │    │            │            │    │
          │    ▼            ▼            ▼    │
          │ Constraint    Rule      Transaction
          │                           │
          │                           │ system authority
          └───────────────────────────┘
```

---

# Part III: Authorization Component

## 3.1 Contract

```rust
trait Authorization {
    // Mutation authorization (gating)
    fn check_mutation(
        &self,
        actor: Option<EntityId>,
        operation: MutationOp,
        target: Option<EntityId>,
        target_type: TypeId,
        target_attr: Option<AttrId>,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
    ) -> AuthDecision;
    
    // Observation authorization (predicate compilation)
    fn compile_observation_predicates(
        &self,
        actor: Option<EntityId>,
        query_types: &[TypeId],
        query_edges: &[EdgeTypeId],
        registry: &Registry,
    ) -> ObservationPredicates;
    
    // Cache management
    fn invalidate_for_edge(&mut self, edge_type: EdgeTypeId, edge: &Edge);
    fn invalidate_for_actor(&mut self, actor: EntityId);
    
    // Policy management
    fn reload_policies(&mut self, registry: &Registry);
}

enum MutationOp {
    Spawn,
    Kill,
    Link(EdgeTypeId),
    Unlink(EdgeTypeId),
    Set(AttrId),
}

enum AuthDecision {
    Allow,
    Deny { policy: String, message: Option<String> },
}

struct ObservationPredicates {
    // Per-type predicates to inject
    node_predicates: Map<TypeId, CompiledPredicate>,
    // Per-edge predicates to inject  
    edge_predicates: Map<EdgeTypeId, CompiledPredicate>,
    // Attribute projections (visible attrs per type)
    projections: Map<TypeId, Set<AttrId>>,
    // Types completely denied (pre-query gate)
    denied_types: Set<TypeId>,
}

struct CompiledPredicate {
    // Pattern elements to add to query
    additional_patterns: Vec<PatternElement>,
    // Conditions to add to WHERE
    additional_conditions: Vec<CompiledExpr>,
    // Variables introduced (for join planning)
    introduced_vars: Vec<VarDef>,
}
```

## 3.2 Internal Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                    AUTHORIZATION COMPONENT                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                     POLICY EVALUATOR                            │ │
│  ├────────────────────────────────────────────────────────────────┤ │
│  │                                                                  │ │
│  │  evaluate_mutation(ctx: &MutationContext) -> Vec<PolicyResult>  │ │
│  │                                                                  │ │
│  │  For each policy matching (operation, target_type):             │ │
│  │    1. Build EvalContext from MutationContext                    │ │
│  │    2. Evaluate condition with Pattern.evaluate()                │ │
│  │    3. Collect (policy_id, priority, decision)                   │ │
│  │                                                                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    PREDICATE COMPILER                           │ │
│  ├────────────────────────────────────────────────────────────────┤ │
│  │                                                                  │ │
│  │  compile_for_type(actor, type_id) -> Option<CompiledPredicate>  │ │
│  │                                                                  │ │
│  │  For each instance-level policy for type:                       │ │
│  │    1. Extract pattern elements referencing bound var            │ │
│  │    2. Substitute current_actor() with concrete actor ID         │ │
│  │    3. Merge ALLOW policies (OR)                                 │ │
│  │    4. Overlay DENY policies                                     │ │
│  │    5. Return combined predicate                                 │ │
│  │                                                                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    DECISION RESOLVER                            │ │
│  ├────────────────────────────────────────────────────────────────┤ │
│  │                                                                  │ │
│  │  resolve(results: Vec<PolicyResult>) -> AuthDecision            │ │
│  │                                                                  │ │
│  │  1. Group by priority (descending)                              │ │
│  │  2. At highest priority level:                                  │ │
│  │     - If any DENY → Deny (with highest-priority deny message)   │ │
│  │     - Else if any ALLOW → Allow                                 │ │
│  │     - Else continue to next priority level                      │ │
│  │  3. If no decisions at any level → Deny (default)               │ │
│  │                                                                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                      CACHE MANAGER                              │ │
│  ├────────────────────────────────────────────────────────────────┤ │
│  │                                                                  │ │
│  │  L1: actor_roles        Map<EntityId, Set<EntityId>>            │ │
│  │      TTL: session lifetime                                      │ │
│  │                                                                  │ │
│  │  L2: role_permissions   Map<EntityId, Set<Permission>>          │ │
│  │      TTL: long (roles change rarely)                            │ │
│  │                                                                  │ │
│  │  L3: mutation_decisions Map<(EntityId, MutationOp, TypeId),     │ │
│  │                             AuthDecision>                       │ │
│  │      TTL: short or transaction-scoped                           │ │
│  │                                                                  │ │
│  │  L4: compiled_predicates Map<(EntityId, TypeId),                │ │
│  │                              CompiledPredicate>                 │ │
│  │      TTL: session lifetime                                      │ │
│  │                                                                  │ │
│  │  Invalidation triggers:                                         │ │
│  │    • L1: has_role edge change for actor                         │ │
│  │    • L2: role_has_permission edge change                        │ │
│  │    • L3: any auth-relevant edge change                          │ │
│  │    • L4: same as L3, plus policy recompilation                  │ │
│  │                                                                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.3 Auth-Relevant Edge Tracking

The Authorization component tracks which edge types affect authorization decisions:

```rust
struct AuthRelevantEdges {
    // Edge types that appear in any authorization policy condition
    edge_types: Set<EdgeTypeId>,
    
    // Computed at policy load time by scanning all policy conditions
    // for edge patterns
}

impl Authorization {
    fn compute_auth_relevant_edges(&mut self, registry: &Registry) {
        self.auth_relevant.clear();
        
        for policy in registry.all_auth_policies() {
            for edge_pattern in policy.condition.edge_patterns() {
                self.auth_relevant.insert(edge_pattern.edge_type);
            }
        }
    }
    
    fn is_auth_relevant(&self, edge_type: EdgeTypeId) -> bool {
        self.auth_relevant.contains(&edge_type)
    }
}
```

---

# Part IV: Mutation Authorization (Gating)

## 4.1 Execution Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                   MUTATION AUTHORIZATION FLOW                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   SPAWN t: Task { title = "Test" }                                  │
│                │                                                     │
│                ▼                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  1. EXTRACT CONTEXT                                         │   │
│   │     actor      = session.actor       // Option<EntityId>    │   │
│   │     operation  = MutationOp::Spawn                          │   │
│   │     target     = None                // not yet created     │   │
│   │     type_id    = TypeId::Task                               │   │
│   │     attr       = None                                       │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│                ▼                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  2. CHECK CACHE                                             │   │
│   │     key = (actor, Spawn, Task)                              │   │
│   │     if cached → return cached decision                      │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│                ▼ (cache miss)                                        │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  3. FIND MATCHING POLICIES                                  │   │
│   │     registry.auth_policies_for(Spawn, Task)                 │   │
│   │     → [admin_create, member_create, default_deny]           │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│                ▼                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  4. EVALUATE EACH POLICY                                    │   │
│   │     Build EvalContext { actor, operation, target, ... }     │   │
│   │     For each policy:                                        │   │
│   │       result = pattern.evaluate(policy.condition, ctx)      │   │
│   │       if result == true → collect (policy, decision)        │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│                ▼                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  5. RESOLVE DECISION                                        │   │
│   │     Sort by priority, apply DENY-wins-at-same-priority      │   │
│   │     → Allow or Deny                                         │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│                ▼                                                     │
│   ┌────────────────────────────────────────────────────────────┐   │
│   │  6. CACHE & RETURN                                          │   │
│   │     cache.insert(key, decision)                             │   │
│   │     return decision                                         │   │
│   └────────────────────────────────────────────────────────────┘   │
│                │                                                     │
│        ┌───────┴───────┐                                            │
│        │               │                                             │
│        ▼               ▼                                             │
│      Allow           Deny                                            │
│        │               │                                             │
│        ▼               ▼                                             │
│   [Continue to    [Return Error]                                    │
│    Type Check]     E7001 PERMISSION_DENIED                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.2 Per-Operation Semantics

| Operation | Target Known? | Context Available |
|-----------|---------------|-------------------|
| `SPAWN(Type)` | No (not yet created) | type only |
| `KILL(node)` | Yes | node + type + all attrs |
| `LINK(edge_type)` | Yes (targets exist) | edge type + target nodes |
| `UNLINK(edge)` | Yes | edge + targets + attrs |
| `SET(node, attr)` | Yes | node + type + attr + old value + new value |

```rust
struct MutationContext {
    actor: Option<EntityId>,
    operation: MutationOp,
    
    // For SPAWN: None. For others: the target entity.
    target: Option<EntityId>,
    
    // Always known
    target_type: TypeId,
    
    // For SET: the attribute being modified
    target_attr: Option<AttrId>,
    
    // For SET: old and new values
    old_value: Option<Value>,
    new_value: Option<Value>,
    
    // For LINK: the target entities
    edge_targets: Option<Vec<EntityId>>,
}
```

## 4.3 Transaction Component Integration

```rust
impl Transaction {
    fn execute_mutation(
        &mut self,
        stmt: MutationStmt,
        session: &Session,
        auth: &Authorization,
        // ... other components
    ) -> Result<MutationResult, TxnError> {
        
        // 1. Build mutation context
        let ctx = self.build_mutation_context(&stmt, session)?;
        
        // 2. Authorization gate
        match auth.check_mutation(ctx, graph, registry, pattern) {
            AuthDecision::Allow => { /* continue */ }
            AuthDecision::Deny { policy, message } => {
                return Err(TxnError::PermissionDenied {
                    actor: session.actor,
                    operation: ctx.operation,
                    target: ctx.target,
                    policy,
                    message,
                });
            }
        }
        
        // 3. Normal mutation flow (type check, apply, constraints, rules)
        self.execute_mutation_inner(stmt, /* ... */)
    }
}
```

---

# Part V: Observation Authorization (Filtering)

## 5.1 Predicate Injection Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                  OBSERVATION AUTHORIZATION MODEL                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   User Query:                                                       │
│     MATCH t: Task WHERE t.priority > 5 RETURN t.title               │
│                                                                      │
│   Authorization Policy:                                             │
│     authorization project_member_sees_tasks:                        │
│       ON MATCH(t: Task)                                             │
│       ALLOW IF EXISTS(p: Project, belongs_to(t, p),                 │
│                       member_of(current_actor(), p))                │
│                                                                      │
│   Predicate Compilation (for actor #alice):                         │
│     additional_patterns: [                                          │
│       p: Project,                                                   │
│       belongs_to(t, p),                                             │
│       member_of(#alice, p)                                          │
│     ]                                                               │
│     additional_conditions: []                                       │
│                                                                      │
│   Effective Query (after injection):                                │
│     MATCH t: Task, p: Project,                                      │
│           belongs_to(t, p),                                         │
│           member_of(#alice, p)                                      │
│     WHERE t.priority > 5                                            │
│     RETURN t.title                                                  │
│                                                                      │
│   Result: Only tasks in projects where Alice is a member            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.2 Query Component Integration

```rust
impl Query {
    fn execute(
        &self,
        stmt: AnalyzedMatchStmt,
        session: &Session,
        auth: &Authorization,
        graph: &Graph,
        registry: &Registry,
        pattern: &Pattern,
    ) -> Result<ResultSet, QueryError> {
        
        // 1. Collect types/edges referenced in query
        let query_types = stmt.referenced_types();
        let query_edges = stmt.referenced_edges();
        
        // 2. Compile authorization predicates
        let predicates = auth.compile_observation_predicates(
            session.actor,
            &query_types,
            &query_edges,
            registry,
        );
        
        // 3. Check for type-level denials
        for type_id in &query_types {
            if predicates.denied_types.contains(type_id) {
                return Err(QueryError::TypeAccessDenied {
                    type_id: *type_id,
                    actor: session.actor,
                });
            }
        }
        
        // 4. Inject predicates into query
        let augmented = self.inject_predicates(stmt, &predicates);
        
        // 5. Apply attribute projections
        let projected = self.apply_projections(augmented, &predicates.projections);
        
        // 6. Plan and execute augmented query
        let plan = self.plan(projected, graph, registry);
        self.execute_plan(plan, graph, pattern)
    }
    
    fn inject_predicates(
        &self,
        stmt: AnalyzedMatchStmt,
        predicates: &ObservationPredicates,
    ) -> AnalyzedMatchStmt {
        let mut augmented = stmt.clone();
        
        // For each node variable in the query
        for (var, type_id) in stmt.node_variables() {
            if let Some(pred) = predicates.node_predicates.get(&type_id) {
                // Add pattern elements, substituting the variable
                for elem in &pred.additional_patterns {
                    augmented.pattern.push(elem.substitute(var));
                }
                // Add conditions
                for cond in &pred.additional_conditions {
                    augmented.where_clause.push(cond.substitute(var));
                }
            }
        }
        
        // For each edge pattern in the query
        for (edge_var, edge_type) in stmt.edge_variables() {
            if let Some(pred) = predicates.edge_predicates.get(&edge_type) {
                for elem in &pred.additional_patterns {
                    augmented.pattern.push(elem.substitute(edge_var));
                }
                for cond in &pred.additional_conditions {
                    augmented.where_clause.push(cond.substitute(edge_var));
                }
            }
        }
        
        augmented
    }
}
```

## 5.3 Policy Composition for Filtering

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PREDICATE COMPOSITION                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Multiple ALLOW policies → OR (disjunction)                        │
│   DENY policies → AND NOT (exclusion)                               │
│                                                                      │
│   Example policies:                                                 │
│     ALLOW IF owns(current_actor(), t)                               │
│     ALLOW IF assigned_to(t, current_actor())                        │
│     DENY IF t.confidential = true AND NOT has_clearance(actor)      │
│                                                                      │
│   Compilation:                                                      │
│                                                                      │
│     allow_predicate =                                               │
│       EXISTS(owns(#alice, t))                                       │
│       OR EXISTS(assigned_to(t, #alice))                             │
│                                                                      │
│     deny_predicate =                                                │
│       t.confidential = true AND NOT EXISTS(                         │
│         c: Clearance, has_clearance(#alice, c)                      │
│       )                                                             │
│                                                                      │
│     combined =                                                      │
│       allow_predicate AND NOT deny_predicate                        │
│                                                                      │
│   Injected as WHERE clause addition                                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

```rust
impl Authorization {
    fn compile_observation_predicates(
        &self,
        actor: Option<EntityId>,
        query_types: &[TypeId],
        query_edges: &[EdgeTypeId],
        registry: &Registry,
    ) -> ObservationPredicates {
        let mut result = ObservationPredicates::default();
        
        for type_id in query_types {
            let policies = registry.auth_policies_for_match(*type_id);
            
            // Separate type-level from instance-level
            let (type_level, instance_level): (Vec<_>, Vec<_>) = 
                policies.partition(|p| !p.references_bound_var());
            
            // Type-level: evaluate immediately
            for policy in type_level {
                let ctx = EvalContext::for_type_check(actor, *type_id);
                if !self.evaluate_policy(policy, &ctx) && policy.is_deny() {
                    result.denied_types.insert(*type_id);
                }
            }
            
            // Instance-level: compile to predicate
            if !instance_level.is_empty() {
                let predicate = self.compile_instance_policies(
                    actor, 
                    &instance_level
                );
                result.node_predicates.insert(*type_id, predicate);
            }
            
            // Attribute-level: compute visible projection
            let visible_attrs = self.compute_visible_attrs(actor, *type_id, registry);
            result.projections.insert(*type_id, visible_attrs);
        }
        
        // Similar for edge types...
        
        result
    }
    
    fn compile_instance_policies(
        &self,
        actor: Option<EntityId>,
        policies: &[&AuthPolicy],
    ) -> CompiledPredicate {
        let mut allow_disjuncts = Vec::new();
        let mut deny_conjuncts = Vec::new();
        
        for policy in policies {
            // Substitute current_actor() with concrete ID
            let substituted = policy.condition.substitute_actor(actor);
            
            match policy.decision {
                Decision::Allow => allow_disjuncts.push(substituted),
                Decision::Deny => deny_conjuncts.push(substituted),
            }
        }
        
        // Build: (allow1 OR allow2 OR ...) AND NOT (deny1) AND NOT (deny2) ...
        CompiledPredicate::compose_or_and_not(allow_disjuncts, deny_conjuncts)
    }
}
```

## 5.4 Edge Visibility

```
┌─────────────────────────────────────────────────────────────────────┐
│                      EDGE VISIBILITY RULES                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Default rule: Edge visible IFF all targets visible                │
│                                                                      │
│   Implementation:                                                   │
│     When query includes edge pattern e(a, b, ...):                  │
│       1. Inject visibility predicates for a, b, ...                 │
│       2. Edge implicitly filtered by target visibility              │
│                                                                      │
│   Override with explicit edge policy:                               │
│     authorization see_follows_of_followees:                         │
│       ON MATCH(e: follows)                                          │
│       ALLOW IF follows(current_actor(), source(e))                  │
│                                                                      │
│   With override:                                                    │
│     Edge predicate evaluated independently                          │
│     May see edge even if target node content hidden                 │
│                                                                      │
│   Higher-order edges:                                               │
│     Same rules apply recursively                                    │
│     meta_edge visible IFF target_edge visible (by default)          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.5 Attribute Visibility

```rust
struct AttributeProjection {
    // Attributes visible to this actor for this type
    visible: Set<AttrId>,
    // Attributes explicitly hidden (even if would be visible by default)
    hidden: Set<AttrId>,
}

impl Query {
    fn apply_projections(
        &self,
        stmt: AnalyzedMatchStmt,
        projections: &Map<TypeId, Set<AttrId>>,
    ) -> AnalyzedMatchStmt {
        let mut result = stmt.clone();
        
        // Filter RETURN clause to visible attributes
        result.return_clause = result.return_clause.filter(|proj| {
            match proj {
                Projection::Attr(var, attr) => {
                    let type_id = result.type_of(var);
                    projections.get(&type_id)
                        .map(|visible| visible.contains(attr))
                        .unwrap_or(true)
                }
                _ => true
            }
        });
        
        // Filter WHERE clause to not reference hidden attributes
        // (prevents probing hidden values)
        result.where_clause = result.where_clause.filter(|cond| {
            !cond.references_hidden_attrs(projections)
        });
        
        result
    }
}
```

## 5.6 Aggregate Semantics

```
┌─────────────────────────────────────────────────────────────────────┐
│                     AGGREGATE AUTHORIZATION                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Aggregates operate on filtered result set.                        │
│                                                                      │
│   Query:   MATCH t: Task RETURN COUNT(t)                            │
│   Graph:   10 tasks total, 3 visible to actor                       │
│   Result:  3                                                        │
│                                                                      │
│   This is the only coherent semantics:                              │
│     • Actor's world contains 3 tasks                                │
│     • COUNT counts what exists in actor's world                     │
│     • No information leakage about hidden entities                  │
│                                                                      │
│   Implementation:                                                   │
│     Aggregation happens AFTER predicate injection                   │
│     No special handling needed                                      │
│                                                                      │
│   ┌──────────────────────────────────────────────────────────────┐ │
│   │  Query Plan:                                                  │ │
│   │    Aggregate(COUNT)                                           │ │
│   │      ↑                                                        │ │
│   │    Filter(auth_predicate)   ← injected                        │ │
│   │      ↑                                                        │ │
│   │    Scan(Task)                                                 │ │
│   └──────────────────────────────────────────────────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 5.7 WALK Authorization

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WALK AUTHORIZATION                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   WALK FROM #start FOLLOW follows [depth: 3] RETURN PATH            │
│                                                                      │
│   At each traversal step:                                           │
│     1. Can actor see the edge being traversed?                      │
│     2. Can actor see the target node?                               │
│                                                                      │
│   If either fails → branch terminates (no error, just stops)        │
│                                                                      │
│   Result: The subgraph reachable within actor's visibility          │
│                                                                      │
│   Implementation:                                                   │
│                                                                      │
│   fn walk_step(current: EntityId, edge_type: EdgeTypeId) -> Vec<EntityId> {│
│       let edges = graph.edges_from(current, edge_type);            │
│                                                                      │
│       edges.filter(|edge| {                                         │
│           // Check edge visibility                                  │
│           auth.can_see_edge(actor, edge) &&                         │
│           // Check target visibility                                │
│           auth.can_see_node(actor, edge.target())                   │
│       })                                                            │
│       .map(|edge| edge.target())                                    │
│       .collect()                                                    │
│   }                                                                 │
│                                                                      │
│   Note: Predicate injection still applies for efficiency            │
│         Walk becomes filtered BFS/DFS                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part VI: Extended Components

## 6.1 Session (Extended)

```rust
struct Session {
    id: SessionId,
    
    // NEW: Actor performing operations in this session
    actor: Option<EntityId>,
    
    // NEW: Authority mode for rule execution
    authority_mode: AuthorityMode,
    
    current_txn: Option<TxnId>,
    auto_commit: bool,
}

enum AuthorityMode {
    // Operations authorized as session actor
    Actor,
    // Operations authorized as system (unrestricted)
    System,
}

impl Session {
    // NEW: Bind actor to session
    fn bind_actor(&mut self, actor: EntityId, graph: &Graph) -> Result<(), SessionError> {
        // Validate actor exists and is valid actor type
        if !graph.node_exists(actor) {
            return Err(SessionError::InvalidActor(actor));
        }
        self.actor = Some(actor);
        Ok(())
    }
    
    // NEW: Unbind actor
    fn unbind_actor(&mut self) {
        self.actor = None;
    }
}
```

**New Statements:**

```sql
BEGIN SESSION AS #alice     -- Bind actor
END SESSION                 -- Unbind actor (or close session)
```

## 6.2 Registry (Extended)

```rust
struct Registry {
    // ... existing fields ...
    
    // NEW: Authorization registry
    auth: AuthorizationRegistry,
}

struct AuthorizationRegistry {
    // All authorization policies
    policies: Vec<AuthPolicy>,
    
    // Index: operation → policies
    by_operation: Map<MutationOp, Vec<PolicyId>>,
    
    // Index: (operation, type) → policies
    by_op_and_type: Map<(MutationOp, TypeId), Vec<PolicyId>>,
    
    // Index: type → instance-level MATCH policies
    match_policies_by_type: Map<TypeId, Vec<PolicyId>>,
    
    // Index: edge_type → MATCH policies
    match_policies_by_edge: Map<EdgeTypeId, Vec<PolicyId>>,
    
    // Edge types that appear in any policy condition
    auth_relevant_edges: Set<EdgeTypeId>,
}

impl AuthorizationRegistry {
    fn policies_for_mutation(&self, op: MutationOp, type_id: TypeId) -> &[AuthPolicy] {
        // Return policies matching this operation and type
    }
    
    fn policies_for_match(&self, type_id: TypeId) -> &[AuthPolicy] {
        // Return MATCH policies for this type
    }
    
    fn is_auth_relevant(&self, edge_type: EdgeTypeId) -> bool {
        self.auth_relevant_edges.contains(&edge_type)
    }
}

struct AuthPolicy {
    id: PolicyId,
    name: String,
    priority: i32,
    operation_pattern: OperationPattern,
    decision: Decision,
    condition: CompiledExpr,
    message: Option<String>,
    
    // Precomputed
    references_bound_var: bool,  // True if condition references target
    affected_types: Set<TypeId>,
    affected_edges: Set<EdgeTypeId>,
    
    l0_node_id: NodeId,
}
```

## 6.3 Pattern (Extended)

```rust
// Extended evaluation context
struct EvalContext {
    // Standard bindings (pattern variables → entities)
    bindings: Bindings,
    
    // NEW: Authorization context (optional, for auth conditions only)
    auth_context: Option<AuthContext>,
}

struct AuthContext {
    actor: Option<EntityId>,
    operation: Option<String>,
    target: Option<EntityId>,
    target_type: Option<TypeId>,
    target_attr: Option<AttrId>,
}

impl Pattern {
    fn evaluate(
        &self, 
        expr: &CompiledExpr, 
        ctx: &EvalContext,
        graph: &Graph,
    ) -> Result<Value, EvalError> {
        match expr {
            // NEW: Context functions
            CompiledExpr::CurrentActor => {
                ctx.auth_context
                    .as_ref()
                    .and_then(|ac| ac.actor)
                    .map(Value::NodeRef)
                    .ok_or(EvalError::NoAuthContext("current_actor()"))
            }
            
            CompiledExpr::Operation => {
                ctx.auth_context
                    .as_ref()
                    .and_then(|ac| ac.operation.clone())
                    .map(Value::String)
                    .ok_or(EvalError::NoAuthContext("operation()"))
            }
            
            CompiledExpr::Target => {
                ctx.auth_context
                    .as_ref()
                    .and_then(|ac| ac.target)
                    .map(Value::NodeRef)
                    .ok_or(EvalError::NoAuthContext("target()"))
            }
            
            CompiledExpr::TargetType => {
                ctx.auth_context
                    .as_ref()
                    .and_then(|ac| ac.target_type)
                    .map(|t| Value::String(registry.type_name(t)))
                    .ok_or(EvalError::NoAuthContext("target_type()"))
            }
            
            CompiledExpr::TargetAttr => {
                ctx.auth_context
                    .as_ref()
                    .and_then(|ac| ac.target_attr)
                    .map(|a| Value::String(registry.attr_name(a)))
                    .ok_or(EvalError::NoAuthContext("target_attr()"))
            }
            
            // ... existing expression evaluation ...
        }
    }
}
```

## 6.4 Compiler (Extended)

```rust
impl Compiler {
    fn compile(&self, ontology: OntologyAST, graph: &mut Graph) -> Result<Registry, CompileError> {
        // ... existing compilation ...
        
        // NEW: Compile authorization rules
        for auth_decl in ontology.authorization_rules {
            let policy = self.compile_auth_policy(auth_decl, &registry)?;
            self.generate_auth_l0(&policy, graph)?;
            registry.auth.register(policy);
        }
        
        // NEW: Compute auth-relevant edges
        registry.auth.compute_relevant_edges();
        
        Ok(registry)
    }
    
    fn compile_auth_policy(
        &self, 
        decl: AuthorizationDecl,
        registry: &Registry,
    ) -> Result<AuthPolicy, CompileError> {
        // Parse operation pattern
        let op_pattern = self.compile_operation_pattern(&decl.on_clause)?;
        
        // Compile condition expression
        let condition = self.compile_auth_condition(&decl.condition, registry)?;
        
        // Validate context function usage
        self.validate_context_functions(&condition)?;
        
        // Determine if condition references bound variable
        let refs_bound = condition.references_pattern_var();
        
        Ok(AuthPolicy {
            id: self.next_policy_id(),
            name: decl.name,
            priority: decl.priority.unwrap_or(0),
            operation_pattern: op_pattern,
            decision: decl.decision,
            condition,
            message: decl.message,
            references_bound_var: refs_bound,
            affected_types: self.extract_affected_types(&op_pattern),
            affected_edges: self.extract_affected_edges(&condition),
            l0_node_id: NodeId::PLACEHOLDER, // Set during L0 generation
        })
    }
    
    fn validate_context_functions(&self, expr: &CompiledExpr) -> Result<(), CompileError> {
        // Ensure context functions only appear in authorization conditions
        // This is called only for auth policies, so all uses are valid
        // But verify no nested constraint/rule conditions
        Ok(())
    }
}
```

## 6.5 Analyzer (Extended)

```rust
impl Analyzer {
    fn analyze(&self, stmt: Statement, registry: &Registry) -> Result<AnalyzedStmt, AnalysisError> {
        match stmt {
            // NEW: Validate context functions are only in auth conditions
            Statement::Constraint(c) => {
                if c.condition.contains_context_function() {
                    return Err(AnalysisError::ContextFunctionInConstraint {
                        constraint: c.name,
                        function: c.condition.first_context_function(),
                    });
                }
                // ... normal analysis
            }
            
            Statement::Rule(r) => {
                if r.condition.contains_context_function() {
                    return Err(AnalysisError::ContextFunctionInRule {
                        rule: r.name,
                        function: r.condition.first_context_function(),
                    });
                }
                // ... normal analysis
            }
            
            // ... other statements
        }
    }
}
```

---

# Part VII: Layer 0 Extensions

## 7.1 Authorization Types

```
-- Authorization policy definition
node _AuthorizationRule [sealed] {
  name: String [required, unique],
  priority: Int = 0,
  decision: String [required, in: ["allow", "deny"]],
  message: String?,
  references_bound_var: Bool = false,
  doc: String?
}

-- Operation pattern for matching
node _OperationPattern [sealed] {
  -- "spawn", "kill", "link", "unlink", "set", "match", "*"
  -- Prefixed with "meta_" for META operations
  operation: String [required],
  -- null = any type
  target_type_name: String?,
  -- For SET: specific attribute, null = any
  target_attr_name: String?
}

-- Compound operation pattern (for alternatives)
node _CompoundOpPattern [sealed] {
  -- "or" for alternatives
  combinator: String [required]
}
```

## 7.2 Authorization Edges

```
-- Policy has operation pattern
edge _auth_has_pattern(
  rule: _AuthorizationRule,
  pattern: _OperationPattern | _CompoundOpPattern
)

-- Compound pattern has members
edge _compound_pattern_member(
  compound: _CompoundOpPattern,
  member: _OperationPattern
) {
  position: Int [required]
}

-- Policy has condition expression
edge _auth_has_condition(
  rule: _AuthorizationRule,
  condition: _Expr
)

-- Ontology declares authorization rule
edge _ontology_declares_auth(
  ontology: _Ontology,
  rule: _AuthorizationRule
)
```

## 7.3 Context Function Expressions

```
-- Context function expressions (only valid in auth conditions)
node _CurrentActorExpr : _Expr [sealed] {
  -- Returns the session's bound actor
}

node _OperationExpr : _Expr [sealed] {
  -- Returns the operation type as string
}

node _TargetExpr : _Expr [sealed] {
  -- Returns the target entity (for mutations on existing entities)
}

node _TargetTypeExpr : _Expr [sealed] {
  -- Returns the target type name as string
}

node _TargetAttrExpr : _Expr [sealed] {
  -- Returns the target attribute name (for SET)
}
```

---

# Part VIII: Cache Invalidation

## 8.1 Invalidation Strategy

```
┌─────────────────────────────────────────────────────────────────────┐
│                     CACHE INVALIDATION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Auth-relevant edge mutation triggers cache invalidation:          │
│                                                                      │
│   LINK has_role(#alice, #admin_role)                                │
│     │                                                                │
│     ▼                                                                │
│   Graph.notify_edge_change(has_role, edge)                          │
│     │                                                                │
│     ▼                                                                │
│   if auth.is_auth_relevant(has_role):                               │
│     │                                                                │
│     ├─► Invalidate L1 for actor #alice                              │
│     │   (actor's role cache)                                        │
│     │                                                                │
│     └─► Invalidate L3 for actor #alice                              │
│         (mutation decision cache)                                   │
│         (compiled predicate cache)                                  │
│                                                                      │
│   Subsequent operations re-evaluate policies                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 8.2 Graph Integration

```rust
impl Graph {
    fn create_edge(
        &mut self, 
        type_id: EdgeTypeId, 
        targets: Vec<EntityId>,
        attrs: Attributes,
        auth: &mut Authorization,  // NEW: for invalidation
    ) -> EdgeId {
        let edge_id = self.create_edge_inner(type_id, targets, attrs);
        
        // NEW: Notify authorization for cache invalidation
        if auth.is_auth_relevant(type_id) {
            let edge = self.get_edge(edge_id).unwrap();
            auth.invalidate_for_edge(type_id, &edge);
        }
        
        edge_id
    }
    
    fn delete_edge(
        &mut self,
        edge_id: EdgeId,
        auth: &mut Authorization,
    ) -> Result<(), GraphError> {
        let edge = self.get_edge(edge_id)?;
        let type_id = edge.type_id;
        
        // NEW: Notify before deletion
        if auth.is_auth_relevant(type_id) {
            auth.invalidate_for_edge(type_id, &edge);
        }
        
        self.delete_edge_inner(edge_id)
    }
}
```

---

# Part IX: Rule Authority

## 9.1 System vs Actor Authority

```
┌─────────────────────────────────────────────────────────────────────┐
│                      RULE AUTHORITY MODES                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   DEFAULT: Rules execute with SYSTEM authority                      │
│                                                                      │
│   rule auto_timestamp:                                              │
│     t: Task WHERE t.created_at = null                               │
│     =>                                                              │
│     SET t.created_at = now()    ← No auth check, system authority   │
│                                                                      │
│   Rationale: Rules are schema-level logic, not user actions.        │
│              They've already been authorized by inclusion in        │
│              the ontology.                                          │
│                                                                      │
│   ─────────────────────────────────────────────────────────────────│
│                                                                      │
│   OPTIONAL: Rules with inherited authority                          │
│                                                                      │
│   rule user_triggered [inherit_authority]:                          │
│     t: Task WHERE t.approved = true                                 │
│     =>                                                              │
│     LINK published(t, ...)      ← Auth checked as session actor     │
│                                                                      │
│   Use case: Rules that should respect user permissions.             │
│             If user can't publish, rule fails silently or errors.   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 9.2 Implementation

```rust
struct RuleDef {
    // ... existing fields ...
    
    // NEW: Authority mode
    inherit_authority: bool,
}

impl Rule {
    fn execute_production(
        &self,
        rule: &RuleDef,
        bindings: &Bindings,
        session: &Session,
        auth: &Authorization,
        txn: &mut Transaction,
        // ...
    ) -> Result<(), RuleError> {
        
        // Determine authority for this rule execution
        let authority = if rule.inherit_authority {
            session.actor  // Use session's actor
        } else {
            None  // System authority (no actor = no auth check)
        };
        
        for action in &rule.production.actions {
            match self.execute_action(action, bindings, authority, auth, txn) {
                Ok(()) => continue,
                Err(e) if rule.inherit_authority => {
                    // With inherited authority, auth failures are possible
                    // Option 1: Fail the rule (and transaction)
                    // Option 2: Skip the action silently
                    // Configurable per-rule?
                    return Err(e);
                }
                Err(e) => {
                    // System authority should never fail auth
                    // This is a bug
                    panic!("System authority auth failure: {:?}", e);
                }
            }
        }
        
        Ok(())
    }
}
```

---

# Part X: Error Codes

## 10.1 Authorization Errors

| Code | Name | Description |
|------|------|-------------|
| E7001 | PERMISSION_DENIED | Mutation rejected by authorization policy |
| E7002 | NO_ACTOR_BOUND | Operation requires actor but session has none |
| E7003 | INVALID_ACTOR | Bound actor does not exist or is invalid type |
| E7004 | AUTH_EVAL_ERROR | Policy condition failed to evaluate |
| E7005 | TYPE_ACCESS_DENIED | Entire type denied for observation (type-level policy) |
| E7006 | CONTEXT_FUNCTION_INVALID | Context function used outside authorization |

## 10.2 Error Structure

```rust
enum AuthError {
    PermissionDenied {
        actor: Option<EntityId>,
        operation: MutationOp,
        target: Option<EntityId>,
        target_type: TypeId,
        policy: String,
        message: Option<String>,
    },
    
    NoActorBound {
        operation: MutationOp,
        required_by: String,  // Policy name
    },
    
    InvalidActor {
        actor: EntityId,
        reason: String,
    },
    
    AuthEvalError {
        policy: String,
        expr: String,
        cause: EvalError,
    },
    
    TypeAccessDenied {
        actor: Option<EntityId>,
        type_id: TypeId,
        policy: String,
    },
}
```

---

# Part XI: Implementation Plan

## 11.1 Phases

```
Phase A: Foundation (v2.0)
══════════════════════════
  □ EvalContext with auth fields
  □ Context functions in Pattern
  □ Analyzer validation (no context funcs in constraints/rules)
  □ Session actor binding
  □ Layer 0 auth types

Phase B: Mutation Authorization (v2.0)
══════════════════════════════════════
  □ Authorization component (policy evaluator, decision resolver)
  □ Registry auth index
  □ Transaction pre-check hook
  □ Compiler auth policy support
  □ Basic cache (L3 mutation decisions)

Phase C: Observation Authorization (v2.0)
═════════════════════════════════════════
  □ Predicate compiler
  □ Query predicate injection
  □ Type-level gating
  □ Instance-level filtering (simple cases)
  □ Attribute projection

Phase D: Caching (v2.1)
═══════════════════════
  □ L1 actor role cache
  □ L2 role permission cache
  □ L4 compiled predicate cache
  □ Invalidation on auth-relevant edges
  □ Graph integration for notifications

Phase E: Advanced (v2.1+)
═════════════════════════
  □ Edge visibility rules
  □ WALK authorization
  □ Rule inherit_authority
  □ Query planner optimization (auth-aware planning)
  □ META authorization
```

## 11.2 Testing Strategy

| Category | Tests |
|----------|-------|
| **Unit** | Policy evaluation, predicate compilation, decision resolution |
| **Integration** | Mutation gating, query filtering, cache invalidation |
| **Security** | Cannot bypass auth, no information leakage, default deny |
| **Performance** | Cache hit rates, predicate injection overhead, large policy sets |

---

# Part XII: Performance Considerations

## 12.1 Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Mutation auth check (cached) | < 1μs | Hash lookup |
| Mutation auth check (uncached) | < 100μs | Policy evaluation |
| Predicate compilation (cached) | < 1μs | Hash lookup |
| Predicate compilation (uncached) | < 1ms | Policy analysis |
| Query overhead (simple filter) | < 5% | Additional join |
| Query overhead (complex filter) | < 20% | Multiple joins |

## 12.2 Optimization Strategies

```
┌─────────────────────────────────────────────────────────────────────┐
│                   OPTIMIZATION STRATEGIES                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. POLICY INDEXING                                                │
│      Policies indexed by (operation, type) for O(1) lookup          │
│      Avoids scanning all policies per operation                     │
│                                                                      │
│   2. PREDICATE CACHING                                              │
│      Compiled predicates cached per (actor, type)                   │
│      Reused across queries in same session                          │
│                                                                      │
│   3. EARLY TYPE-LEVEL REJECTION                                     │
│      Type-level denials checked before query planning               │
│      Avoids work for completely denied types                        │
│                                                                      │
│   4. PREDICATE PUSH-DOWN                                            │
│      Auth predicates pushed into query plan                         │
│      Indexes used for auth joins                                    │
│      Not post-filtered                                              │
│                                                                      │
│   5. MATERIALIZED ROLE GRAPH                                        │
│      For RBAC: precompute transitive role memberships               │
│      roles(actor) = direct ∪ inherited                              │
│      Updated on role edge changes                                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Appendix A: Grammar Extensions

```ebnf
(* Session Statements *)
SessionStmt      = "BEGIN" "SESSION" "AS" NodeRef
                 | "END" "SESSION"

(* Authorization Declarations *)
AuthDecl         = "authorization" Identifier AuthMods? ":"
                   "ON" OpPattern
                   Decision "IF" Expr
                   Message?

AuthMods         = "[" AuthMod ("," AuthMod)* "]"
AuthMod          = "priority:" IntLiteral

OpPattern        = "*"
                 | OpType
                 | OpType "(" Pattern ")"
                 | OpType "(" Pattern "," AttrName ")"
                 | OpPattern "|" OpPattern

OpType           = "SPAWN" | "KILL" | "LINK" | "UNLINK" | "SET" | "MATCH"
                 | "META" OpType

Decision         = "ALLOW" | "DENY"

Message          = "MESSAGE" StringLiteral

(* Context Functions - only valid in authorization conditions *)
ContextFunc      = "current_actor" "(" ")"
                 | "operation" "(" ")"
                 | "target" "(" ")"
                 | "target_type" "(" ")"
                 | "target_attr" "(" ")"

(* Rule Extension *)
RuleMods         = "[" RuleMod ("," RuleMod)* "]"
RuleMod          = "priority:" IntLiteral 
                 | "auto" | "manual"
                 | "inherit_authority"       (* NEW *)
```

---

# Appendix B: Complete Example

```
ontology TaskManagement {

  -- Types
  node Person { name: String [required], clearance: Int = 0 }
  node Role { name: String [required, unique] }
  node Project { name: String [required], confidential: Bool = false }
  node Task { 
    title: String [required],
    status: String = "todo",
    internal_score: Int?
  }
  
  -- Relationships
  edge has_role(person: Person, role: Role)
  edge member_of(person: Person, project: Project) { role: String = "member" }
  edge belongs_to(task: Task, project: Project)
  edge assigned_to(task: Task, person: Person)
  
  -- AUTHORIZATION POLICIES --
  
  -- Superadmin bypass (highest priority)
  authorization superadmin [priority: 1000]:
    ON *
    ALLOW IF EXISTS(r: Role, has_role(current_actor(), r) WHERE r.name = "superadmin")
  
  -- Project creation requires manager role
  authorization manager_creates_project:
    ON SPAWN(p: Project)
    ALLOW IF EXISTS(r: Role, has_role(current_actor(), r) WHERE r.name = "manager")
  
  -- Task creation in project requires membership
  authorization member_creates_task:
    ON SPAWN(t: Task)
    ALLOW IF true  -- Tasks can be created, but must be linked to authorized project
  
  authorization link_task_to_project:
    ON LINK(e: belongs_to)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(target(), p),
      member_of(current_actor(), p)
    )
  
  -- Task modification by assignee or project admin
  authorization assignee_modifies_task:
    ON SET(t: Task, _)
    ALLOW IF assigned_to(t, current_actor())
  
  authorization project_admin_modifies_task:
    ON SET(t: Task, _)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      member_of(current_actor(), p) AS m
      WHERE m.role = "admin"
    )
  
  -- Observation: members see project tasks
  authorization member_sees_tasks:
    ON MATCH(t: Task)
    ALLOW IF EXISTS(
      p: Project,
      belongs_to(t, p),
      member_of(current_actor(), p)
    )
  
  -- Observation: hide internal_score from non-analysts
  authorization hide_internal_score:
    ON MATCH(t: Task).internal_score
    DENY IF NOT EXISTS(r: Role, has_role(current_actor(), r) WHERE r.name = "analyst")
  
  -- Confidential projects require clearance
  authorization clearance_for_confidential:
    ON MATCH(t: Task)
    DENY IF EXISTS(
      p: Project,
      belongs_to(t, p)
      WHERE p.confidential = true
    ) AND current_actor().clearance < 3
    MESSAGE "Insufficient clearance for confidential project"
  
  -- Default deny (lowest priority)
  authorization default_deny [priority: -1000]:
    ON *
    DENY IF true
    MESSAGE "Permission denied"
}
```

---

*End of MEW Architecture Extension: Authorization System*