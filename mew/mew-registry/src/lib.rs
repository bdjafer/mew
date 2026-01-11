//! MEW Registry
//!
//! Runtime schema lookup. Single source of truth for types, edges, constraints, rules.
//! The registry is immutable after construction via RegistryBuilder.

mod builder;
mod registry;
mod types;

pub use builder::RegistryBuilder;
pub use registry::Registry;
pub use types::*;
