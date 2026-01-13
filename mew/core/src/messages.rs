//! Common error messages used across MEW components.
//!
//! These constants ensure consistent error messaging between session and REPL.

/// Error: only variable targets are supported.
pub const ERR_ONLY_VAR_TARGETS: &str = "Only variable targets are supported";

/// Error: only variable targets supported in compound statements.
pub const ERR_ONLY_VAR_TARGETS_COMPOUND: &str =
    "Only variable targets are supported in compound statements";

/// Error: SET requires a node target.
pub const ERR_SET_REQUIRES_NODE: &str = "SET requires a node target";

/// Error: KILL requires a node target.
pub const ERR_KILL_REQUIRES_NODE: &str = "KILL requires a node target";

/// Error: UNLINK requires an edge target.
pub const ERR_UNLINK_REQUIRES_EDGE: &str = "UNLINK requires an edge target";

/// Error: edge pattern requires at least 2 targets.
pub const ERR_EDGE_PATTERN_MIN_TARGETS: &str = "Edge pattern requires at least 2 targets";

/// Error: source must be a node.
pub const ERR_SOURCE_MUST_BE_NODE: &str = "Source must be a node";

/// Error: target must be a node.
pub const ERR_TARGET_MUST_BE_NODE: &str = "Target must be a node";
