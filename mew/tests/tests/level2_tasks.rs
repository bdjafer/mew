//! Level 2 - Tasks integration tests.
//!
//! These tests run against the tasks ontology with various scenarios.
//! Focus areas: UNLINK operations, bulk KILL, SPAWN RETURNING, SET multiple
//! attributes, blocking semantics with [no_self]
//!
//! Test modules:
//! - unlink: UNLINK operations for removing edges without deleting nodes
//! - bulk_mutations: SPAWN RETURNING, SET multiple attributes, bulk KILL
//! - blocking: [no_self] constraint on blocks edge, blocking chains
//! - anonymous_targets: Use of `_` for anonymous pattern matching
//! - link_if_not_exists: Idempotent edge creation with LINK IF NOT EXISTS
//! - parameterized_queries: $param syntax, PREPARE/EXECUTE statements

use mew_tests::prelude::*;

mod unlink {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("unlink")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/unlink.mew")
            // Setup tasks
            .step("test_setup_unlink_task_1", |a| a.created(1))
            .step("test_setup_unlink_task_2", |a| a.created(1))
            .step("test_setup_unlink_task_3", |a| a.created(1))
            .step("test_setup_unlink_tag", |a| a.created(1))
            .step("test_setup_unlink_subtask", |a| a.created(1))
            // Create links: 3 tagged + 2 blocks + 1 subtask_of = 6
            .step("test_setup_create_links", |a| a.linked(6))
            // Verify tagged links = 3
            .step("test_verify_tagged_links_exist", |a| a.scalar("tagged_count", 3i64))
            // Verify blocking links = 2
            .step("test_verify_blocking_links_exist", |a| a.scalar("blocking_count", 2i64))
            // Unlink single tag
            .step("test_unlink_single_tag", |a| a.unlinked(1))
            // Verify single unlink = 0
            .step("test_verify_single_unlink", |a| a.scalar("still_linked", 0i64))
            // Verify other links remain = 2
            .step("test_verify_other_links_remain", |a| a.scalar("remaining_count", 2i64))
            // Unlink all tags from task 2
            .step("test_unlink_all_tags_from_specific_task", |a| a.unlinked(1))
            // Verify all tags unlinked = 0
            .step("test_verify_all_tags_unlinked", |a| a.scalar("tag_count", 0i64))
            // Unlink blocking edge
            .step("test_unlink_blocking_edge", |a| a.unlinked(1))
            // Verify blocking unlinked = 0
            .step("test_verify_blocking_unlinked", |a| a.scalar("block_count", 0i64))
            // Verify other blocking remains = 1
            .step("test_verify_other_blocking_remains", |a| a.scalar("block_count", 1i64))
            // Unlink subtask
            .step("test_unlink_subtask", |a| a.unlinked(1))
            // Verify subtask unlinked = 0
            .step("test_verify_subtask_unlinked", |a| a.scalar("link_count", 0i64))
            // Recreate some tags
            .step("test_recreate_some_tags", |a| a.linked(2))
            // Unlink all tags from tag
            .step("test_unlink_all_tags_from_tag", |a| a.unlinked(2))
            // Verify all unlinked = 0
            .step("test_verify_all_unlinked", |a| a.scalar("remaining", 0i64))
            // Tasks still exist = 3
            .step("test_tasks_still_exist_after_unlink", |a| a.scalar("task_count", 3i64))
            // Tag still exists = 1
            .step("test_tag_still_exists_after_unlink", |a| a.scalar("tag_count", 1i64))
            // Setup tasks with edge attributes
            .step("test_setup_tasks_with_edge_attributes", |a| a.created(1))
            .step("test_setup_tasks_with_edge_attributes_2", |a| a.created(1))
            .step("test_setup_subtasks_with_attributes", |a| a.created(1))
            .step("test_setup_subtasks_with_attributes_2", |a| a.created(1))
            .step("test_setup_link_subtasks_to_task", |a| a.linked(2))
            // Unlink only blocking subtasks
            .step("test_unlink_only_blocking_subtasks", |a| a.unlinked(1))
            // Verify selective unlink
            .step("test_verify_selective_unlink", |a| a.rows(1))
            // Unlink and relink
            .step("test_unlink_and_relink", |a| a.linked(1))
            // Verify relink = 1
            .step("test_verify_relink", |a| a.scalar("new_link_count", 1i64))
            // Unlink non-existent edge (should not error)
            .step("test_unlink_nonexistent_edge", |a| a.rows(2))
            // Cleanup
            .step("test_cleanup_unlink_tasks", |a| a.deleted(5))
            .step("test_cleanup_unlink_subtasks", |a| a.deleted(3))
            .step("test_cleanup_unlink_tag", |a| a.deleted(1))
    }

    #[test]
    fn test_unlink_operations() {
        scenario().run().unwrap();
    }
}

mod bulk_mutations {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("bulk_mutations")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/bulk_mutations.mew")
            // SPAWN with RETURNING
            .step("test_spawn_returning_single", |a| a.created(1).rows(1))
            // SPAWN multiple with RETURNING (3 separate statements)
            .step("test_spawn_returning_multiple_1", |a| a.created(1).rows(1))
            .step("test_spawn_returning_multiple_2", |a| a.created(1).rows(1))
            .step("test_spawn_returning_multiple_3", |a| a.created(1).rows(1))
            // SPAWN with specific fields RETURNING
            .step("test_spawn_returning_specific_fields", |a| a.created(1).rows(1))
            // SET multiple attributes
            .step("test_set_multiple_attributes_single_entity", |a| a.modified(1))
            // Verify bulk set
            .step("test_verify_bulk_set", |a| a.rows(1))
            // SET multiple entities
            .step("test_set_multiple_entities", |a| a.modified(3))
            // Verify bulk entity set
            .step("test_verify_bulk_entity_set", |a| a.rows(3))
            // SET with computed values
            .step("test_set_with_computed_values", |a| a.modified(3))
            // Verify computed set
            .step("test_verify_computed_set", |a| a.rows(3))
            // Setup bulk kill tasks
            .step("test_setup_bulk_kill_tasks", |a| a.created(5))
            // Bulk kill by status
            .step("test_bulk_kill_by_status", |a| a.deleted(5))
            // Verify bulk kill = 0
            .step("test_verify_bulk_kill", |a| a.scalar("remaining", 0i64))
            // Setup kill with relationships
            .step("test_setup_kill_with_rels", |a| a.created(1))
            .step("test_setup_child_kills", |a| a.created(3))
            .step("test_setup_kill_rels", |a| a.linked(3))
            // Bulk kill children only
            .step("test_bulk_kill_children_only", |a| a.deleted(3))
            // Verify children killed = 0
            .step("test_verify_children_killed", |a| a.scalar("remaining", 0i64))
            // Verify parent remains = 1
            .step("test_verify_parent_remains", |a| a.scalar("remaining", 1i64))
            // Bulk spawn and link
            .step("test_bulk_spawn_and_link", |a| a.created(3).linked(2))
            // Verify bulk spawn and link = 2
            .step("test_verify_bulk_spawn_and_link", |a| a.scalar("subtask_count", 2i64))
            // Bulk set with RETURNING
            .step("test_bulk_set_returning", |a| a.modified(3).rows(3))
            // Setup conditional tasks
            .step("test_setup_conditional_tasks", |a| a.created(3))
            // Bulk set with condition
            .step("test_bulk_set_with_condition", |a| a.modified(2))
            // Verify conditional bulk set
            .step("test_verify_conditional_bulk_set", |a| a.rows(3))
            // Count affected by bulk set
            .step("test_count_affected_by_bulk_set", |a| a.modified(3).rows(1))
            // Cleanup
            .step("test_cleanup_all_test_tasks", |a| a.deleted(10))
            .step("test_cleanup_all_test_subtasks", |a| a.deleted(2))
    }

    #[test]
    fn test_bulk_mutation_operations() {
        scenario().run().unwrap();
    }
}

mod blocking {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("blocking")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/blocking.mew")
            // Setup blocking tasks
            .step("test_setup_blocking_task_a", |a| a.created(1))
            .step("test_setup_blocking_task_b", |a| a.created(1))
            .step("test_setup_blocking_task_c", |a| a.created(1))
            .step("test_setup_blocking_task_d", |a| a.created(1))
            // Create simple block
            .step("test_create_simple_block", |a| a.linked(1))
            // Verify block created
            .step("test_verify_block_created", |a| a.rows(1))
            // Task cannot block itself (no_self constraint)
            .step("test_task_cannot_block_itself", |a| a.error("self"))
            // Verify no self block = 0
            .step("test_verify_no_self_block_created", |a| a.scalar("self_blocks", 0i64))
            // Create blocking chain
            .step("test_create_blocking_chain", |a| a.linked(2))
            // Verify blocking chain
            .step("test_verify_blocking_chain", |a| a.rows(1))
            // Find all blocked by A = 1 (direct only)
            .step("test_find_all_blocked_by_a", |a| a.rows(1))
            // Find tasks that block D = 1 (C)
            .step("test_find_tasks_that_block_d", |a| a.rows(1))
            // Tasks not blocking anything = 1 (D) - using OPTIONAL MATCH
            .step("test_find_tasks_not_blocking_anything", |a| a.rows(1))
            // Tasks not blocked = 1 (A) - using OPTIONAL MATCH
            .step("test_find_tasks_not_blocked", |a| a.rows(1))
            // Setup circular tasks
            .step("test_setup_circular_tasks", |a| a.created(2))
            // Create circular block
            .step("test_create_circular_block", |a| a.linked(2))
            // Verify circular blocks
            .step("test_verify_circular_blocks", |a| a.rows(1))
            // Setup multi blocker
            .step("test_setup_multi_blocker", |a| a.created(1))
            // Create multiple blockers
            .step("test_create_multiple_blockers", |a| a.linked(3))
            // Verify multiple blockers = 3
            .step("test_verify_multiple_blockers", |a| a.scalar("blocker_count", 3i64))
            // List all blockers = 3
            .step("test_list_all_blockers", |a| a.rows(3))
            // Find in-progress blockers >= 2
            .step("test_find_in_progress_blockers", |a| a.rows_gte(2))
            // Tasks blocked by non-done >= 3
            .step("test_find_tasks_blocked_by_non_done", |a| a.rows_gte(3))
            // Unlink one blocker
            .step("test_unlink_one_blocker", |a| a.unlinked(1))
            // Verify one blocker removed = 2
            .step("test_verify_one_blocker_removed", |a| a.scalar("remaining_blockers", 2i64))
            // High priority blocked by low priority >= 1
            .step("test_high_priority_blocked_by_low_priority", |a| a.rows_gte(1))
            // Count blocking relationships >= 5
            .step("test_count_blocking_relationships", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_blocking_tasks", |a| a.deleted(7))
    }

    #[test]
    fn test_blocking_edge_semantics() {
        scenario().run().unwrap();
    }
}

mod anonymous_targets {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("anonymous_targets")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/anonymous_targets.mew")
            // Setup tasks and tags
            .step("test_setup_task_alpha", |a| a.created(1))
            .step("test_setup_task_beta", |a| a.created(1))
            .step("test_setup_task_gamma", |a| a.created(1))
            .step("test_setup_task_orphan", |a| a.created(1))
            .step("test_setup_tags", |a| a.created(2))
            .step("test_setup_links", |a| a.linked(4))
            // Anonymous target: Find tasks with ANY tag
            .step("test_tasks_with_any_tag", |a| a.rows(2))
            .step("test_tasks_with_any_tag_distinct", |a| a.rows(2))
            // Anonymous target: Find tasks that block ANYTHING
            .step("test_tasks_that_block_anything", |a| a.rows(1))
            // Anonymous target: Find tasks blocked by ANYTHING
            .step("test_tasks_blocked_by_anything", |a| a.rows(1))
            // Anonymous target: Find tags used by ANY task
            .step("test_tags_used_by_any_task", |a| a.rows(2))
            // NOT EXISTS with anonymous: Find tasks WITHOUT any tag
            .step("test_tasks_without_any_tag", |a| a.rows(2))
            // NOT EXISTS with anonymous: Find tasks that DON'T block anything
            .step("test_tasks_that_dont_block_anything", |a| a.rows(3))
            // NOT EXISTS with anonymous: Find tasks not blocked by anything
            .step("test_tasks_not_blocked_by_anything", |a| a.rows(3))
            // Combined patterns with anonymous
            .step("test_tasks_with_tag_but_not_blocking", |a| a.rows(1))
            .step("test_tasks_blocking_but_not_tagged_with_review", |a| a.rows(1))
            // Count with anonymous target
            .step("test_count_tasks_with_any_relationship", |a| a.rows(1))
            // Multiple anonymous targets
            .step("test_multiple_anonymous_in_path", |a| a.rows_gte(1))
            // Cleanup
            .step("test_cleanup_anonymous_tasks", |a| a.deleted(4))
            .step("test_cleanup_anonymous_tags", |a| a.deleted(2))
    }

    #[test]
    fn test_anonymous_target_patterns() {
        scenario().run().unwrap();
    }
}

mod link_if_not_exists {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("link_if_not_exists")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/link_if_not_exists.mew")
            // Setup
            .step("test_setup_task_parent", |a| a.created(1))
            .step("test_setup_task_child", |a| a.created(1))
            .step("test_setup_link_tag", |a| a.created(1))
            .step("test_setup_link_subtask", |a| a.created(1))
            // Basic LINK IF NOT EXISTS: Creates edge when none exists
            .step("test_link_if_not_exists_creates_edge", |a| a.linked(1))
            .step("test_verify_edge_created", |a| a.scalar("edge_count", 1i64))
            // Idempotency: Second LINK IF NOT EXISTS is no-op
            .step("test_link_if_not_exists_idempotent", |a| a.linked(0))
            .step("test_verify_still_one_edge", |a| a.scalar("edge_count", 1i64))
            // LINK IF NOT EXISTS: Multiple different targets
            .step("test_setup_second_tag", |a| a.created(1))
            .step("test_link_if_not_exists_different_target", |a| a.linked(1))
            .step("test_verify_two_different_edges", |a| a.scalar("tag_count", 2i64))
            // LINK IF NOT EXISTS: On blocks edge
            .step("test_link_blocks_if_not_exists", |a| a.linked(1))
            .step("test_verify_blocks_created", |a| a.scalar("block_count", 1i64))
            .step("test_blocks_idempotent", |a| a.linked(0))
            .step("test_verify_still_one_block", |a| a.scalar("block_count", 1i64))
            // LINK IF NOT EXISTS: With subtask_of edge
            .step("test_link_subtask_if_not_exists", |a| a.linked(1))
            .step("test_verify_subtask_linked", |a| a.scalar("subtask_count", 1i64))
            .step("test_subtask_idempotent", |a| a.linked(0))
            .step("test_verify_still_one_subtask_link", |a| a.scalar("subtask_count", 1i64))
            // LINK IF NOT EXISTS: Multiple sources to same target
            .step("test_link_child_to_same_tag", |a| a.linked(1))
            .step("test_verify_multiple_sources_to_tag", |a| a.scalar("task_count", 2i64))
            // LINK IF NOT EXISTS vs regular LINK
            .step("test_setup_new_tag_for_comparison", |a| a.created(1))
            .step("test_regular_link_first", |a| a.linked(1))
            .step("test_link_if_not_exists_after_regular_link", |a| a.linked(0))
            .step("test_verify_still_one_comparison_edge", |a| a.scalar("edge_count", 1i64))
            // LINK IF NOT EXISTS: In a loop (multiple invocations)
            .step("test_setup_loop_task", |a| a.created(1))
            .step("test_link_loop_iteration_1", |a| a.linked(1))
            .step("test_link_loop_iteration_2", |a| a.linked(0))
            .step("test_link_loop_iteration_3", |a| a.linked(0))
            .step("test_verify_only_one_edge_after_loop", |a| a.scalar("edge_count", 1i64))
            // Cleanup
            .step("test_cleanup_tasks", |a| a.deleted(3))
            .step("test_cleanup_subtask", |a| a.deleted(1))
            .step("test_cleanup_tags", |a| a.deleted(3))
    }

    #[test]
    fn test_link_if_not_exists_operations() {
        scenario().run().unwrap();
    }
}

mod parameterized_queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("parameterized_queries")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/parameterized_queries.mew")
            // Setup
            .step("test_setup_param_tasks", |a| a.created(5))
            .step("test_setup_param_tags", |a| a.created(2))
            .step("test_setup_param_links", |a| a.linked(4))
            // Basic $param in WHERE clause
            .step("test_param_in_where_string", |a| a.rows(2))
            .step("test_param_in_where_int", |a| a.rows(3))
            .step("test_param_in_where_multiple", |a| a.rows(2))
            // $param in pattern matching
            .step("test_param_in_tag_filter", |a| a.rows(2))
            // $param in SPAWN
            .step("test_param_in_spawn", |a| a.created(1))
            .step("test_verify_param_spawn", |a| a.rows(1))
            // $param in SET
            .step("test_param_in_set", |a| a.modified(1))
            .step("test_verify_param_set", |a| a.rows(1))
            // $param in list operations
            .step("test_param_in_list", |a| a.rows(4))
            .step("test_param_in_list_titles", |a| a.rows(3))
            // $param type inference
            .step("test_param_string_type", |a| a.rows(1))
            .step("test_param_bool_type", |a| a.created(1))
            .step("test_verify_param_bool", |a| a.rows(1))
            // PREPARE / EXECUTE
            .step("test_prepare_basic_query", |a| a.rows(0))
            .step("test_execute_prepared_todo", |a| a.rows(2))
            .step("test_execute_prepared_done", |a| a.rows(1))
            .step("test_execute_prepared_in_progress", |a| a.rows(2))
            // PREPARE with multiple parameters
            .step("test_prepare_multi_param", |a| a.rows(0))
            .step("test_execute_multi_param_1", |a| a.rows(2))
            .step("test_execute_multi_param_2", |a| a.rows(1))
            // PREPARE for mutations
            .step("test_prepare_update_status", |a| a.rows(0))
            .step("test_execute_update_status", |a| a.modified(1))
            .step("test_verify_executed_update", |a| a.rows(1))
            // DROP PREPARED STATEMENT
            .step("test_drop_prepared", |a| a.rows(0))
            .step("test_execute_dropped_should_fail", |a| a.error("not_found"))
            // Missing parameter error
            .step("test_missing_param_error", |a| a.error("missing_parameter"))
            // Cleanup
            .step("test_cleanup_param_tasks", |a| a.deleted(6))
            .step("test_cleanup_param_subtask", |a| a.deleted(1))
            .step("test_cleanup_param_tags", |a| a.deleted(2))
            .step("test_drop_remaining_prepared", |a| a.rows(0))
    }

    #[test]
    fn test_parameterized_query_operations() {
        scenario().run().unwrap();
    }
}
