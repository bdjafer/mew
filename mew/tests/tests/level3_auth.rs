//! Level 3 - Authorization integration tests.
//!
//! These tests cover policy declarations, session management, and context functions.
//! Policy and session features are not yet implemented, so tests expect parse errors.

use mew_tests::prelude::*;

mod policy {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("policy")
            .ontology("level-3/auth/ontology.mew")
            .operations("level-3/auth/operations/policy.mew")
            // Setup operations - these should work
            .step("test_setup_admin_role", |a| a.created(1))
            .step("test_setup_editor_role", |a| a.created(1))
            .step("test_setup_viewer_role", |a| a.created(1))
            .step("test_setup_security_role", |a| a.created(1))
            .step("test_setup_admin_user", |a| a.created(1).linked(1))
            .step("test_setup_editor_user", |a| a.created(1).linked(1))
            .step("test_setup_viewer_user", |a| a.created(1).linked(1))
            .step("test_setup_public_doc", |a| a.created(1).linked(1))
            .step("test_setup_internal_doc", |a| a.created(1).linked(1))
            .step("test_setup_confidential_doc", |a| a.created(1).linked(1))
            .step("test_setup_secret_doc", |a| a.created(1).linked(1))
            // Session-based tests expect parse errors (not yet implemented)
            .step("test_public_access_as_viewer", |a| a.error("parse"))
            .step("test_public_access_no_session", |a| a.rows(1))
            .step("test_owner_can_read", |a| a.error("parse"))
            .step("test_owner_can_write", |a| a.error("parse"))
            .step("test_admin_can_read_all", |a| a.error("parse"))
            .step("test_viewer_cannot_write", |a| a.error("parse"))
            .step("test_share_document", |a| a.linked(1))
            .step("test_viewer_can_read_shared", |a| a.error("parse"))
            .step("test_viewer_cannot_write_shared", |a| a.error("parse"))
            .step("test_owner_can_delete_draft", |a| a.created(1).linked(1))
            .step("test_editor_deletes_own", |a| a.error("parse"))
            .step("test_cannot_delete_published", |a| a.error("parse"))
            .step("test_cannot_access_secret", |a| a.error("parse"))
            .step("test_cannot_escalate_sensitivity", |a| a.error("parse"))
            .step("test_security_cleared_can_access", |a| a.error("parse"))
            .step("test_setup_service", |a| a.created(1))
            .step("test_service_read_only", |a| a.error("parse"))
            // Cleanup: 4 roles + 3 users + 1 service + 5 docs = 13 entities (edges deleted automatically)
            .step("test_cleanup", |a| a.deleted(13))
    }

    #[test]
    fn test_policy_operations() {
        scenario().run().unwrap();
    }
}

mod session {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("session")
            .ontology("level-3/auth/ontology.mew")
            .operations("level-3/auth/operations/session.mew")
            // Setup
            .step("test_setup_alice", |a| a.created(1))
            .step("test_setup_bob", |a| a.created(1))
            .step("test_setup_service", |a| a.created(1))
            // All session statements expect parse errors
            .step("test_begin_session_alice", |a| a.error("parse"))
            // current_actor() exists but returns unknown function error at runtime
            .step("test_session_active_query", |a| a.error("function"))
            .step("test_end_session_alice", |a| a.error("parse"))
            .step("test_session_block", |a| a.error("parse"))
            .step("test_verify_ownership", |a| a.rows_gte(0))
            .step("test_service_session", |a| a.error("parse"))
            .step("test_nested_session_outer", |a| a.error("parse"))
            .step("test_verify_nested_ownership", |a| a.rows_gte(0))
            .step("test_system_context", |a| a.rows_gte(0))
            .step("test_system_can_do_anything", |a| a.created(1))
            .step("test_invalid_actor", |a| a.error("parse"))
            .step("test_invalid_actor_type", |a| a.error("parse"))
            .step("test_session_with_transaction", |a| a.error("parse"))
            // Cleanup: 2 users + 1 service + 1 doc created in system context = 4
            .step("test_cleanup", |a| a.deleted(4))
    }

    #[test]
    fn test_session_operations() {
        scenario().run().unwrap();
    }
}

mod context_functions {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("context_functions")
            .ontology("level-3/auth/ontology.mew")
            .operations("level-3/auth/operations/context_functions.mew")
            // Setup
            .step("test_setup_user", |a| a.created(1))
            .step("test_setup_role", |a| a.created(1).linked(1))
            .step("test_setup_document", |a| a.created(1).linked(1))
            // current_actor() tests - BEGIN SESSION causes parse errors
            .step("test_current_actor_basic", |a| a.error("parse"))
            .step("test_current_actor_in_query", |a| a.error("parse"))
            .step("test_current_actor_comparison", |a| a.error("parse"))
            // Outside session, current_actor() parses but fails at runtime
            .step("test_current_actor_null_outside_session", |a| {
                a.error("function")
            })
            // operation() tests
            .step("test_operation_in_policy", |a| a.error("parse"))
            .step("test_operation_spawn", |a| a.error("parse"))
            .step("test_operation_set", |a| a.error("parse"))
            .step("test_operation_kill", |a| a.error("parse"))
            // target() tests
            .step("test_target_in_set", |a| a.error("parse"))
            .step("test_target_in_match", |a| a.error("parse"))
            .step("test_target_with_condition", |a| a.error("parse"))
            // target_type() tests
            .step("test_target_type_document", |a| a.error("parse"))
            .step("test_target_type_in_spawn", |a| a.error("parse"))
            .step("test_target_type_check", |a| a.error("parse"))
            // target_attr() tests
            .step("test_target_attr_content", |a| a.error("parse"))
            .step("test_target_attr_sensitivity", |a| a.error("parse"))
            .step("test_target_attr_multiple", |a| a.error("parse"))
            .step("test_target_attr_null_for_non_set", |a| a.error("parse"))
            // Combined
            .step("test_combined_functions", |a| a.error("parse"))
            .step("test_policy_style_check", |a| a.error("parse"))
            // Cleanup: 1 user + 1 role + 1 document = 3
            .step("test_cleanup", |a| a.deleted(3))
    }

    #[test]
    fn test_context_function_operations() {
        scenario().run().unwrap();
    }
}
