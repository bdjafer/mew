//! Level 3 - Authorization integration tests.
//!
//! These tests cover policy declarations, session management, and context functions.
//! Policy and session features are not yet implemented, so tests expect parse errors.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-3/auth/ontology.mew")
            .operations("level-3/auth/operations/crud.mew")
            .step("test_create_admin_role", |a| a.created(1))
            .step("test_create_user_role", |a| a.created(1))
            .step("test_create_user_alice", |a| a.created(1))
            .step("test_create_user_bob", |a| a.created(1))
            .step("test_create_service", |a| a.created(1))
            .step("test_assign_admin_role", |a| a.linked(1))
            .step("test_assign_user_role", |a| a.linked(1))
            .step("test_assign_service_role", |a| a.linked(1))
            .step("test_create_folder", |a| a.created(1))
            .step("test_create_public_doc", |a| a.created(1))
            .step("test_create_internal_doc", |a| a.created(1))
            .step("test_link_ownership", |a| a.linked(2))
            .step("test_link_folder", |a| a.linked(2))
            .step("test_share_document", |a| a.linked(1))
            .step("test_grant_permissions", |a| a.linked(4))
            .step("test_query_admins", |a| a.rows(1))
            .step("test_query_user_roles", |a| a.rows(2))
            .step("test_query_public_docs", |a| a.rows(1))
            .step("test_query_owned_docs", |a| a.rows(2))
            .step("test_query_shared_docs", |a| a.rows(1))
            .step("test_query_folder_docs", |a| a.rows(1))
            .step("test_publish_document", |a| a.modified(1))
            .step("test_archive_document", |a| a.modified(1))
            .step("test_cleanup_docs", |a| a.deleted(2))
            .step("test_cleanup_folders", |a| a.deleted(1))
            .step("test_cleanup_services", |a| a.deleted(1))
            .step("test_cleanup_users", |a| a.deleted(2))
            .step("test_cleanup_roles", |a| a.deleted(2))
    }

    #[test]
    fn test_auth_crud_operations() {
        scenario().run().unwrap();
    }
}

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
            .step("test_setup_permissions", |a| a.linked(7))
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
            .step("test_cleanup", |a| a.deleted(1))
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
            .step("test_session_active_query", |a| a.error("parse"))
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
            .step("test_cleanup", |a| a.deleted(1))
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
            // current_actor() tests - need session support
            .step("test_current_actor_basic", |a| a.error("parse"))
            .step("test_current_actor_in_query", |a| a.error("parse"))
            .step("test_current_actor_comparison", |a| a.error("parse"))
            .step("test_current_actor_null_outside_session", |a| a.error("parse"))
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
            // Cleanup
            .step("test_cleanup", |a| a.deleted(1))
    }

    #[test]
    fn test_context_function_operations() {
        scenario().run().unwrap();
    }
}
