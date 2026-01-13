//! MEW Core Types
//!
//! This crate provides the foundational types used throughout the MEW system:
//! - Identity types (NodeId, EdgeId, EntityId)
//! - Type identifiers (TypeId, EdgeTypeId, AttrId)
//! - Value types (the Value enum with all scalar and reference types)
//! - Entity structures (Node, Edge)
//! - Common error types

mod entity;
mod error;
mod id;
mod value;

pub use entity::*;
pub use error::*;
pub use id::*;
pub use value::*;
