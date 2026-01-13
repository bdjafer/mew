//! MEW Query
//!
//! Plan and execute read operations (MATCH/WALK/INSPECT).
//!
//! Responsibilities:
//! - Generate execution plan from analyzed query
//! - Choose optimal index usage
//! - Execute plan and stream results
//! - Handle aggregations and sorting

mod aggregates;
mod error;
mod executor;
mod operators;
mod plan;
mod result;

pub use error::{QueryError, QueryResult};
pub use executor::QueryExecutor;
pub use plan::{QueryPlan, QueryPlanner};
pub use result::{QueryResults, QueryRow};
