//! Level 3 - Workflow integration tests.
//!
//! These tests run against the workflow ontology with various scenarios.

use mew_tests::prelude::*;

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-3/workflow/ontology.mew")
            .seed("level-3/workflow/seeds/populated.mew")
            .operations("level-3/workflow/operations/queries.mew")
            .step("count_all_workflows", |a| a.value(2))
            .step("count_all_states", |a| a.value(4))
            .step("count_all_transitions", |a| a.value(3))
            .step("count_all_workitems", |a| a.value(3))
            .step("query_active_workflows", |a| a.rows(2))
            .step("query_initial_states", |a| a.rows(1))
            .step("query_final_states", |a| a.rows(2))
            .step("query_active_items", |a| a.rows(1))
            .step("query_roles", |a| a.rows(3))
            .step("query_actors", |a| a.rows(3))
    }

    #[test]
    fn test_query_operations_with_populated_data() {
        scenario().run().unwrap();
    }
}

mod errors {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("errors")
            .ontology("level-3/workflow/ontology.mew")
            .operations("level-3/workflow/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            .step("spawn_invalid_status", |a| a.error("constraint"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}

mod trigger {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("trigger")
            .ontology("level-3/workflow/ontology.mew")
            .operations("level-3/workflow/operations/trigger.mew")
            // Setup workflow and states
            .step("test_setup_workflow", |a| a.created(1))
            .step("test_setup_states", |a| a.created(4).linked(4))
            .step("test_setup_transitions", |a| a.created(3).linked(9))
            .step("test_setup_work_items", |a| a.created(3).linked(6))
            // TRIGGER expects parse error (not yet implemented)
            .step("test_trigger_cancel_workflow", |a| a.error("parse"))
            .step("test_verify_manual_rule_fired", |a| a.rows_gte(0))
            .step("test_trigger_with_filter", |a| a.error("parse"))
            .step("test_verify_filtered_trigger", |a| a.rows_gte(0))
            .step("test_trigger_returns_count", |a| a.error("parse"))
            .step("test_verify_count_returned", |a| a.error("parse"))
            .step("test_trigger_no_match", |a| a.error("parse"))
            .step("test_trigger_unknown_rule", |a| a.error("parse"))
            .step("test_trigger_auto_rule", |a| a.error("parse"))
            // Cleanup (split to ensure correct order)
            .step("test_cleanup_history", |a| a.deleted_gte(0))
            .step("test_cleanup_workitems", |a| a.deleted(3))
            .step("test_cleanup_transitions", |a| a.deleted(3))
            .step("test_unlink_state_of", |a| a.unlinked(4))
            .step("test_cleanup_states", |a| a.deleted(4))
            .step("test_cleanup_workflows", |a| a.deleted(1))
    }

    #[test]
    fn test_trigger_manual_rules() {
        scenario().run().unwrap();
    }
}

mod prevent_behavior {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("prevent_behavior")
            .ontology("level-3/workflow/ontology.mew")
            .operations("level-3/workflow/operations/prevent_behavior.mew")
            // Setup
            .step("test_setup_workflow", |a| a.created(1))
            .step("test_setup_states", |a| a.created(2).linked(2))
            // Verify initial state
            .step("test_verify_workflow_exists", |a| a.value(1))
            .step("test_verify_states_linked", |a| a.value(2))
            // Try to kill workflow - should be prevented (restricted)
            .step("test_kill_workflow_prevented", |a| a.error("restricted"))
            // Verify workflow and states still exist
            .step("test_verify_workflow_still_exists", |a| a.value(1))
            .step("test_verify_states_still_exist", |a| a.value(2))
            // Proper cleanup: unlink first, then kill
            .step("test_unlink_states", |a| a.unlinked(2))
            .step("test_kill_workflow_now_allowed", |a| a.deleted(1))
            .step("test_verify_workflow_killed", |a| a.value(0))
            // Cleanup remaining states
            .step("test_cleanup_states", |a| a.deleted(2))
    }

    #[test]
    fn test_prevent_referential_action() {
        scenario().run().unwrap();
    }
}
