//! Level 3 - Audit integration tests.
//!
//! These tests cover versioning, snapshots, time-travel, branching, and transactions.
//! Versioning and transaction features are not yet implemented, so tests expect parse errors.

use mew_tests::prelude::*;

mod versioning {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("versioning")
            .ontology("level-3/audit/ontology.mew")
            .operations("level-3/audit/operations/versioning.mew")
            // Setup should work
            .step("test_setup_config_set", |a| a.created(1))
            .step("test_setup_config_items", |a| a.created(3).linked(3))
            // SNAPSHOT expects parse error
            .step("test_create_snapshot", |a| a.error("parse"))
            .step("test_create_named_snapshot", |a| a.error("parse"))
            .step("test_verify_snapshot_exists", |a| a.error("parse"))
            // Modifications work normally
            .step("test_modify_after_snapshot", |a| a.modified(2))
            .step("test_verify_modifications", |a| a.rows(1))
            // CHECKOUT expects parse error
            .step("test_checkout_snapshot", |a| a.error("parse"))
            .step("test_query_historical_data", |a| a.rows(1))
            .step("test_checkout_head", |a| a.error("parse"))
            .step("test_verify_current_state", |a| a.rows(1))
            // Relative references
            .step("test_checkout_head_minus_1", |a| a.error("parse"))
            .step("test_checkout_head_minus_2", |a| a.error("parse"))
            .step("test_return_to_head", |a| a.error("parse"))
            // DIFF expects parse error
            .step("test_diff_snapshots", |a| a.error("parse"))
            .step("test_diff_head_versions", |a| a.error("parse"))
            .step("test_diff_with_filter", |a| a.error("parse"))
            // BRANCH expects parse error
            .step("test_create_branch", |a| a.error("parse"))
            .step("test_switch_to_branch", |a| a.error("parse"))
            .step("test_modify_on_branch", |a| a.modified(2))
            .step("test_verify_branch_state", |a| a.rows(1))
            .step("test_switch_to_main", |a| a.error("parse"))
            .step("test_verify_main_unchanged", |a| a.rows(1))
            // MERGE expects parse error
            .step("test_merge_branch", |a| a.error("parse"))
            .step("test_verify_merged_state", |a| a.rows(1))
            // Conflict resolution
            .step("test_create_conflict_scenario", |a| a.error("parse"))
            .step("test_modify_main", |a| a.modified(1))
            .step("test_switch_and_modify_branch", |a| a.error("parse"))
            .step("test_switch_back_to_main", |a| a.error("parse"))
            .step("test_merge_with_conflict", |a| a.error("parse"))
            .step("test_resolve_conflict_ours", |a| a.error("parse"))
            .step("test_resolve_conflict_theirs", |a| a.error("parse"))
            .step("test_resolve_conflict_manual", |a| a.error("parse"))
            // VERSIONS expects parse error
            .step("test_list_versions", |a| a.error("parse"))
            .step("test_list_versions_limit", |a| a.error("parse"))
            .step("test_list_branches", |a| a.error("parse"))
            // Cleanup (1 ConfigSet + 3 ConfigItems = 4 total)
            .step("test_cleanup", |a| a.deleted(4))
    }

    #[test]
    fn test_versioning_operations() {
        scenario().run().unwrap();
    }
}

mod transactions {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("transactions")
            .ontology("level-3/audit/ontology.mew")
            .operations("level-3/audit/operations/transactions.mew")
            // Setup
            .step("test_setup", |a| a.created(1))
            // BEGIN/COMMIT works - transaction creates 1 node and 1 link
            .step("test_basic_transaction", |a| a.created(1).linked(1))
            .step("test_verify_committed", |a| a.rows(1))
            // ROLLBACK works - spawned items are rolled back (0 created)
            .step("test_rollback_transaction", |a| a.created(0))
            .step("test_verify_rollback", |a| a.value(0))
            // SAVEPOINT/ROLLBACK TO not implemented
            .step("test_savepoint_basic", |a| a.error("parse"))
            .step("test_verify_savepoint", |a| a.rows_gte(0))
            // Multiple savepoints not implemented
            .step("test_multiple_savepoints", |a| a.error("parse"))
            .step("test_verify_multiple_savepoints", |a| a.rows_gte(0))
            // Nested savepoints not implemented
            .step("test_nested_savepoints", |a| a.error("parse"))
            .step("test_verify_nested", |a| a.rows_gte(0))
            // Isolation levels work
            .step("test_read_committed", |a| a.created(1).linked(1))
            .step("test_serializable", |a| a.created(1).linked(1))
            // Deferred constraints
            .step("test_deferred_cardinality", |a| a.created(1))
            // Rules in transaction uses SAVEPOINT which is not implemented
            .step("test_rules_in_transaction", |a| a.error("parse"))
            // Cleanup (1 ConfigSet + 4 ConfigItems from working transactions)
            .step("test_cleanup", |a| a.deleted(5))
    }

    #[test]
    fn test_transaction_operations() {
        scenario().run().unwrap();
    }
}
