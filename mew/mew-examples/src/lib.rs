//! MEW Integration Test Framework
//!
//! Provides a fluent API for writing integration tests against MEW.
//!
//! # Structure
//!
//! The framework separates concerns into four components:
//!
//! - **ontology.mew** - Type definitions for the domain
//! - **seeds/** - Reusable starting states (MEW mutations)
//! - **operations/** - Reusable operation sequences (MEW statements with step markers)
//! - **scenarios/** - Test orchestration (Rust: seed + operations + assertions)
//!
//! # Example
//!
//! ```ignore
//! use mew_examples::prelude::*;
//!
//! pub fn scenario() -> Scenario {
//!     Scenario::new("crud")
//!         .ontology("level-1/bookmarks/ontology.mew")
//!         .seed("level-1/bookmarks/seeds/minimal.mew")
//!         .operations("level-1/bookmarks/operations/crud.mew")
//!         .step("spawn_bookmark", |a| a.created(1))
//!         .step("query_count", |a| a.value(1))
//! }
//!
//! #[test]
//! fn test() {
//!     scenario().run().unwrap();
//! }
//! ```

mod assertion;
mod error;
mod loader;
mod runner;
mod scenario;

// Must be public for the row! macro to work from external crates
#[doc(hidden)]
pub mod value_ext;

pub use assertion::{Assertion, AssertionBuilder, Row};
pub use error::{ExampleError, ExampleResult};
pub use scenario::Scenario;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::assertion::{Assertion, AssertionBuilder, Row};
    pub use crate::error::{ExampleError, ExampleResult};
    pub use crate::row;
    pub use crate::scenario::Scenario;
}

/// Macro for creating a Row with field values.
///
/// # Example
///
/// ```ignore
/// use mew_examples::prelude::*;
///
/// let row = row!{ title: "Example", count: 42 };
/// ```
#[macro_export]
macro_rules! row {
    { $($key:ident : $value:expr),* $(,)? } => {{
        let mut row = $crate::Row::new();
        $(
            row.insert(stringify!($key).to_string(), $crate::value_ext::IntoRowValue::into_row_value($value));
        )*
        row
    }};
}
