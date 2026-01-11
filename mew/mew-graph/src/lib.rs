//! MEW Graph Storage
//!
//! This crate provides the core graph storage with indexed access:
//! - Node and edge storage
//! - Type index: Find nodes by type
//! - Attribute index: Find nodes by attribute value or range
//! - Adjacency index: Find edges from/to a node
//! - Higher-order index: Find edges about an edge

mod graph;
mod index;

pub use graph::*;
