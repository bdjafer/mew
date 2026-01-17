# MEW Language Specification

This directory contains the complete specification for the Higher-Order Hypergraph (MEW) language family.

## Document Structure

The specification is organized into 6 parts:

### **1_LANGUAGE.md** (~1,545 lines)
**Part I: Shared Language Foundations**

Lexical and semantic foundations shared by all MEW languages.

- Introduction & conformance
- Lexical structure (tokens, identifiers, literals, operators)
- Scalar types (String, Int, Float, Bool, Timestamp, Duration)
- Type modifiers (optional, list, bounded)
- Expressions (arithmetic, logical, comparison, path)
- Built-in functions

### **2_LAYER0.md** (~1,613 lines)
**Part II: Layer 0 Meta-Ontology**

The hardcoded meta-types that define what ontologies are.

- Layer 0 node types (`_NodeType`, `_EdgeType`, `_AttributeDef`, etc.)
- Layer 0 edge types (`_type_has_attribute`, `_extends`, etc.)
- Layer 0 constraints (structural invariants)
- Built-in operations & execution semantics
- Bootstrapping process

**Audience:** Engine implementers, deep internals

### **3_SCHEMA.md** (~2,493 lines)
**Part III: Schema Definition Language**

The Ontology DSL for defining graph schemas (compiled to Layer 0).

- Type aliases
- Ontology declarations & inheritance
- Node type declarations
- Edge type declarations
- Attribute declarations
- Patterns
- Constraint declarations
- Rule declarations
- Complete grammar & examples

**Audience:** Ontology authors, domain modelers

### **4_QUERIES.md** (~1,356 lines)
**Part IV: Query Operations**

Read-only observation operations for querying the graph.

- MATCH (pattern matching with WHERE, RETURN, aggregations)
- WALK (graph traversal)
- INSPECT (schema introspection)
- EXPLAIN/PROFILE (query debugging)

**Audience:** Analysts, query writers

### **5_MUTATIONS.md** (~1,289 lines)
**Part V: Mutation Operations**

Write operations and transaction control.

- SPAWN (create nodes)
- KILL (delete nodes)
- LINK (create edges)
- UNLINK (delete edges)
- SET (update attributes)
- Parameterized queries
- Transaction control (BEGIN, COMMIT, ROLLBACK)

**Audience:** Application developers

### **6_SYSTEM.md** (~1,371 lines)
**Part VI: System Operations & Reference**

Administration, versioning, and complete reference materials.

- Administration (LOAD, EXTEND, SHOW, INDEX)
- Versioning (SNAPSHOT, CHECKOUT, DIFF)
- Query control & safety (LIMIT, TIMEOUT)
- Complete GQL grammar
- Quick reference
- Appendices (keywords, operators, functions, error formats)

**Audience:** DBAs, system administrators

---

## Reading Paths

**Learning MEW from scratch:**
1. LANGUAGE → 2. LAYER0 → 3. SCHEMA → 4. QUERIES → 5. MUTATIONS → 6. SYSTEM

**Writing ontologies:**
1. LANGUAGE (skim) → 3. SCHEMA (deep) → 2. LAYER0 (reference)

**Querying data:**
1. LANGUAGE (expressions) → 4. QUERIES (deep)

**Building applications:**
4. QUERIES + 5. MUTATIONS (deep) → 6. SYSTEM (reference)

**Understanding internals:**
2. LAYER0 → 1. LANGUAGE → 3. SCHEMA

---

## Version

**Specification Version:** 1.0  
**Last Updated:** January 2026  
**Status:** Draft

---

## Total Size

- **Total Lines:** ~9,667
- **Average per file:** ~1,611 lines
- **Largest file:** 3_SCHEMA.md (2,493 lines)
- **Smallest file:** 5_MUTATIONS.md (1,289 lines)
