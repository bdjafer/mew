# Design Plan: Higher-Order Edge Syntax

## Overview

This plan introduces support for higher-order edges in the MEW parser - edges that target other edges rather than nodes. This enables edges-about-edges functionality for attaching metadata, confidence values, and provenance to existing edges.

**Specification References:**
- §1 FOUNDATIONS.md §4.5 (Edge Reference Types)
- §2 DSL.md §12.7 (Higher-Order Edges)
- §3 GQL.md §20.4.3 (Higher-Order Patterns)
- §0 META_ONTOLOGY.md §9.3, §10.1-10.3

---

## 1. Syntax Specification

### 1.1 Edge Reference Type

```
EdgeRefType = "edge" "<" (Identifier | "any") ">"
```

**Examples:**
```mew
edge<causes>            -- reference to a 'causes' edge
edge<any>               -- reference to any edge type
```

### 1.2 Higher-Order Edge Declaration

```mew
-- Standard edge (for comparison)
edge causes(from: Event, to: Event)

-- Higher-order edge targeting a specific edge type
edge confidence(about: edge<causes>) {
  level: Float [>= 0, <= 1]
}

-- Higher-order edge targeting any edge type
edge provenance(about: edge<any>) {
  source: String
}
```

### 1.3 Higher-Order Edge in Queries

```mew
-- Binding an edge with AS for higher-order observation
MATCH
  e1: Event, e2: Event,
  causes(e1, e2) AS c,           -- bind edge to variable 'c'
  confidence(c, level)            -- higher-order: edge targeting edge 'c'
WHERE level > 0.7
RETURN e1, e2, level
```

---

## 2. AST Changes

### 2.1 New Type: `TypeRef`

Add a new enum to represent type references that can be either simple identifiers or edge references.

**File: `mew/mew-parser/src/ast.rs`**

```rust
/// Type reference in ontology definitions.
/// Can be a simple type name or an edge reference type.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    /// Simple type: String, Int, NodeType, etc.
    Simple(String),
    /// Edge reference: edge<causes> or edge<any>
    EdgeRef {
        /// The edge type being referenced, or None for edge<any>
        edge_type: Option<String>,
        span: Span,
    },
}

impl TypeRef {
    pub fn simple(name: String) -> Self {
        TypeRef::Simple(name)
    }

    pub fn edge_ref(edge_type: Option<String>, span: Span) -> Self {
        TypeRef::EdgeRef { edge_type, span }
    }

    pub fn is_edge_ref(&self) -> bool {
        matches!(self, TypeRef::EdgeRef { .. })
    }

    pub fn is_any_edge_ref(&self) -> bool {
        matches!(self, TypeRef::EdgeRef { edge_type: None, .. })
    }
}
```

### 2.2 Update `EdgeTypeDef`

Change the params field to use `TypeRef` instead of `String` for the type.

**File: `mew/mew-parser/src/ast.rs`**

```rust
/// Edge type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeDef {
    pub name: String,
    /// Parameters: (name, type_ref)
    /// type_ref can be TypeRef::Simple("NodeType") or TypeRef::EdgeRef { edge_type: Some("causes") }
    pub params: Vec<(String, TypeRef)>,
    pub attrs: Vec<AttrDef>,
    pub modifiers: Vec<EdgeModifier>,
    pub span: Span,
}
```

### 2.3 Update `AttrDef` (Optional Enhancement)

Consider whether attribute types should also support edge references. Per spec, `edge<any>` can only appear in edge signatures, not as attribute types. So we may want to leave `AttrDef.type_name` as `String` for now.

```rust
/// Attribute definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AttrDef {
    pub name: String,
    pub type_name: String,  // Keep as simple String - edge<any> not allowed here
    pub nullable: bool,
    pub modifiers: Vec<AttrModifier>,
    pub default_value: Option<Expr>,
    pub span: Span,
}
```

---

## 3. Parser Changes

### 3.1 New Helper: `parse_type_ref`

Add a method to parse either a simple type or an edge reference type.

**File: `mew/mew-parser/src/parser/ontology.rs`**

```rust
/// Parse a type reference: either a simple identifier or edge<Type> / edge<any>
fn parse_type_ref(&mut self) -> ParseResult<TypeRef> {
    let start = self.peek().span;

    // Check for edge<...> syntax
    if self.check(&TokenKind::Edge) {
        self.advance(); // consume 'edge'
        self.expect(&TokenKind::Lt)?; // consume '<'

        // Parse the inner type: either 'any' or an identifier
        let edge_type = if self.check(&TokenKind::Any) {
            self.advance();
            None // edge<any>
        } else {
            Some(self.expect_ident()?) // edge<TypeName>
        };

        self.expect(&TokenKind::Gt)?; // consume '>'

        let span = self.span_from(start);
        Ok(TypeRef::EdgeRef { edge_type, span })
    } else if self.check(&TokenKind::Any) {
        // Allow 'any' as a simple type for backwards compatibility
        self.advance();
        Ok(TypeRef::Simple("any".to_string()))
    } else {
        // Simple type name
        let name = self.expect_ident()?;
        Ok(TypeRef::Simple(name))
    }
}
```

### 3.2 Update `parse_edge_type_def`

Modify the edge parameter parsing to use `parse_type_ref`.

**File: `mew/mew-parser/src/parser/ontology.rs`**

```rust
/// Parse an edge type definition.
/// Syntax: edge EdgeName(param: Type, ...) [modifiers] { attrs }
/// Type can be: NodeType, any, edge<EdgeType>, or edge<any>
fn parse_edge_type_def(&mut self) -> ParseResult<EdgeTypeDef> {
    let start = self.expect(&TokenKind::Edge)?.span;
    let name = self.expect_ident()?;

    // Parse parameters
    self.expect(&TokenKind::LParen)?;
    let mut params = Vec::new();
    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
        let param_name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;

        // Parse type reference (can be edge<...> or simple type)
        let param_type = self.parse_type_ref()?;

        params.push((param_name, param_type));
        if self.check(&TokenKind::Comma) {
            self.advance();
        }
    }
    self.expect(&TokenKind::RParen)?;

    // ... rest of parsing unchanged ...
}
```

---

## 4. Lexer Changes

**No changes required.** The existing lexer already handles:
- `edge` as `TokenKind::Edge`
- `<` as `TokenKind::Lt`
- `>` as `TokenKind::Gt`
- `any` as `TokenKind::Any`
- Identifiers

The parser will combine these tokens to recognize the `edge<...>` syntax.

---

## 5. Compiler/Type-Checker Changes

### 5.1 Validation Rules

The compiler must validate:

1. **Edge type existence**: In `edge<TypeName>`, the `TypeName` must reference a declared edge type.

2. **No edge<any> in attributes**: The `edge<any>` type can only appear in edge signatures (parameters), never as attribute types.

3. **Higher-order edge semantics**: An edge with an `edge<...>` parameter can only target edge instances, not nodes.

**File: `mew/mew-compiler/src/validate.rs` (or similar)**

```rust
fn validate_edge_type(edge_def: &EdgeTypeDef, ontology: &Ontology) -> Result<(), ValidationError> {
    for (param_name, type_ref) in &edge_def.params {
        if let TypeRef::EdgeRef { edge_type, .. } = type_ref {
            if let Some(target_edge) = edge_type {
                // Verify the target edge type exists
                if !ontology.has_edge_type(target_edge) {
                    return Err(ValidationError::UnknownEdgeType {
                        name: target_edge.clone(),
                        // ...
                    });
                }
            }
            // edge<any> is always valid in edge signatures
        }
    }
    Ok(())
}
```

---

## 6. Session/Runtime Changes

### 6.1 Edge ID References

The session must handle edge IDs as targets for higher-order edges.

**LINK command extension:**

```mew
-- Create a base edge, capturing its ID
LINK causes(#event1, #event2) AS c

-- Create a higher-order edge targeting the base edge
LINK confidence(c, 0.85) { assessed_by = "expert" }
```

The `c` variable (or `#edge_id`) must resolve to an edge ID when used as a target for a higher-order edge.

### 6.2 Cascading Deletion

When a base edge is unlinked, all higher-order edges targeting it must be automatically unlinked (cascade). This is implicit per spec §2 DSL.md §12.7.4.

**File: `mew/mew-session/src/ops.rs` (or similar)**

```rust
fn unlink_edge(&mut self, edge_id: EdgeId) -> Result<UnlinkResult, SessionError> {
    // 1. Find all higher-order edges targeting this edge
    let ho_edges = self.find_edges_targeting_edge(edge_id);

    // 2. Recursively unlink them (cascade)
    let mut cascade_count = 0;
    for ho_edge in ho_edges {
        let result = self.unlink_edge(ho_edge)?;
        cascade_count += 1 + result.cascade_count;
    }

    // 3. Unlink the target edge
    self.graph.unlink_edge(edge_id)?;

    Ok(UnlinkResult {
        success: true,
        unlinked_count: 1,
        unlinked_ids: vec![edge_id.to_string()],
        cascade_count,
        errors: None,
    })
}
```

---

## 7. Query Pattern Changes

### 7.1 Binding Edges with AS

The parser already supports `AS alias` for edges. No changes needed for basic binding:

```mew
MATCH causes(e1, e2) AS c
```

### 7.2 Higher-Order Pattern Matching

The pattern parser must support matching higher-order edges. When a pattern like `confidence(c, level)` appears after `causes(e1, e2) AS c`, the `c` should resolve as an edge reference.

**File: `mew/mew-parser/src/parser/pattern.rs`**

This may need updates to track which variables are bound to edges vs nodes, but the basic parsing should work unchanged - the semantic analysis phase determines whether a variable reference is a node or edge.

---

## 8. Implementation Phases

### Phase 1: AST & Parser (This PR)

1. Add `TypeRef` enum to AST
2. Update `EdgeTypeDef.params` type
3. Add `parse_type_ref()` method
4. Update `parse_edge_type_def()` to use `parse_type_ref()`
5. Add parser tests for `edge<...>` syntax

**Files to modify:**
- `mew/mew-parser/src/ast.rs`
- `mew/mew-parser/src/parser/ontology.rs`

### Phase 2: Compiler Validation

1. Validate edge type references exist
2. Validate `edge<any>` not used in attribute types
3. Semantic checks for higher-order edge declarations

**Files to modify:**
- `mew/mew-compiler/src/validate.rs` (or equivalent)

### Phase 3: Session/Runtime

1. Support edge IDs as LINK targets
2. Implement cascading deletion for higher-order edges
3. Update UNLINK result format with cascade_count

**Files to modify:**
- `mew/mew-session/src/ops.rs` (or equivalent)
- `mew/mew-graph/src/graph.rs` (or equivalent)

### Phase 4: Query Execution

1. Support higher-order edge patterns in MATCH
2. Type-checking for edge variable usage
3. Query optimization for higher-order traversals

---

## 9. Test Cases

### 9.1 Parser Tests

```rust
#[test]
fn test_parse_higher_order_edge_specific_type() {
    let input = r#"
        ontology Test {
            edge causes(from: Event, to: Event)
            edge confidence(about: edge<causes>) {
                level: Float
            }
        }
    "#;
    let defs = parse_ontology(input).unwrap();
    // Verify confidence edge has edge<causes> parameter
}

#[test]
fn test_parse_higher_order_edge_any() {
    let input = r#"
        ontology Test {
            edge provenance(about: edge<any>) {
                source: String
            }
        }
    "#;
    let defs = parse_ontology(input).unwrap();
    // Verify provenance edge has edge<any> parameter
}

#[test]
fn test_parse_mixed_params() {
    let input = r#"
        ontology Test {
            edge annotates(subject: Node, about: edge<any>) {
                note: String
            }
        }
    "#;
    let defs = parse_ontology(input).unwrap();
    // Verify mixed node and edge<any> parameters
}
```

### 9.2 Validation Tests

```rust
#[test]
fn test_reject_edge_any_in_attribute() {
    let input = r#"
        ontology Test {
            edge foo(a: Node) {
                ref: edge<any>  -- Should fail: edge<any> not allowed in attrs
            }
        }
    "#;
    assert!(parse_and_validate(input).is_err());
}

#[test]
fn test_reject_unknown_edge_type() {
    let input = r#"
        ontology Test {
            edge foo(about: edge<nonexistent>)  -- Should fail: nonexistent edge type
        }
    "#;
    assert!(parse_and_validate(input).is_err());
}
```

### 9.3 Runtime Tests

```rust
#[test]
fn test_cascade_delete_higher_order() {
    // Setup: Create base edge and higher-order edge
    // Unlink base edge
    // Verify higher-order edge was also unlinked
}

#[test]
fn test_query_higher_order_edge() {
    // Setup: Create edges with higher-order annotations
    // Query for edges with specific annotation values
    // Verify correct results
}
```

---

## 10. Backwards Compatibility

The changes are **backwards compatible**:

1. Existing ontologies using simple type names continue to work
2. The `any` keyword already exists and is handled
3. New syntax `edge<...>` uses existing tokens in a new combination

No migration is required for existing ontologies.

---

## 11. Error Messages

Provide clear error messages for common mistakes:

```
Error: Unknown edge type 'foo' in edge reference
  --> ontology.mew:5:20
   |
 5 |     edge confidence(about: edge<foo>)
   |                            ^^^^^^^^^
   |
   = help: did you mean 'causes'?
```

```
Error: edge<any> is not allowed as an attribute type
  --> ontology.mew:7:12
   |
 7 |         ref: edge<any>
   |              ^^^^^^^^^
   |
   = note: edge<any> can only be used in edge parameter signatures
```

---

## 12. Summary

| Component | Changes Required |
|-----------|-----------------|
| Lexer | None |
| AST | Add `TypeRef` enum, update `EdgeTypeDef.params` |
| Parser | Add `parse_type_ref()`, update `parse_edge_type_def()` |
| Compiler | Add validation for edge type references |
| Session | Support edge IDs as targets, implement cascade deletion |
| Query | Support higher-order patterns (minimal changes) |

**Estimated effort:** Phase 1 (Parser) is straightforward and can be completed quickly. Phases 2-4 require more work in the compiler and runtime.
