//! Mutation operation implementations.
//!
//! Each operation (SPAWN, KILL, LINK, UNLINK, SET) is implemented
//! in its own module for better organization and testability.

mod kill;
mod link;
mod set;
mod spawn;
mod unlink;

pub use kill::execute_kill;
pub use link::execute_link;
pub use set::{execute_set, execute_set_edge};
pub use spawn::{execute_spawn, execute_spawn_item};
pub use unlink::execute_unlink;
