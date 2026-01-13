//! Level 2 - Tasks integration tests.
//!
//! These tests run against the tasks ontology with various scenarios.
//! Focus areas: Transitive patterns (+/*), NOT EXISTS, UNLINK operations,
//! bulk KILL, SPAWN RETURNING, SET multiple attributes, blocking semantics

use mew_tests::prelude::*;

mod transitive {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("transitive")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/transitive.mew")
            // Setup chain: A -> B -> C -> D -> E
            .step("test_setup_trans_task_a", |a| a.created(1))
            .step("test_setup_trans_task_b", |a| a.created(1))
            .step("test_setup_trans_task_c", |a| a.created(1))
            .step("test_setup_trans_task_d", |a| a.created(1))
            .step("test_setup_trans_task_e", |a| a.created(1))
            .step("test_setup_chain_links", |a| a.linked(4))
            // Transitive closure (+): A blocks B, C, D, E = 4
            .step("test_transitive_one_hop", |a| a.rows(4))
            // From middle: C blocks D, E = 2
            .step("test_transitive_from_middle", |a| a.rows(2))
            // Reverse: who blocks E? A, B, C, D = 4
            .step("test_transitive_reverse_find_all_blockers", |a| a.rows(4))
            // Reflexive transitive (*): A + B, C, D, E = 5
            .step("test_reflexive_transitive_includes_self", |a| a.rows(5))
            // From leaf: E only = 1
            .step("test_reflexive_from_leaf", |a| a.rows(1))
            // Transitive with status filter: todo tasks blocked by A = 3 (C, D, E)
            .step("test_transitive_with_status_filter", |a| a.rows(3))
            // Transitive with priority filter: done blockers of E = 1 (A)
            .step("test_transitive_with_priority_filter", |a| a.rows(1))
            // Count all blocked by A = 4
            .step("test_count_all_blocked_tasks", |a| a.scalar("blocked_count", 4i64))
            // Count blockers per task
            .step("test_count_blockers_per_task", |a| a.rows_gte(4))
            // Setup branch task
            .step("test_setup_branch_task", |a| a.created(1))
            .step("test_setup_branch_link", |a| a.linked(1))
            // Transitive with branch: A blocks B, C, D, E, Branch = 5
            .step("test_transitive_with_branch", |a| a.rows(5))
            // Find longest chain
            .step("test_find_longest_blocking_chain", |a| a.rows_gte(1))
            // Tasks with no transitive blockers
            .step("test_tasks_with_no_transitive_blockers", |a| a.rows_gte(1))
            // Cleanup
            .step("test_cleanup_transitive_tasks", |a| a.deleted(6))
    }

    #[test]
    fn test_transitive_patterns() {
        scenario().run().unwrap();
    }
}

mod not_exists {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("not_exists")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/not_exists.mew")
            // Setup untagged tasks
            .step("test_setup_untagged_task", |a| a.created(1))
            .step("test_setup_orphan_task", |a| a.created(1))
            .step("test_setup_isolated_task", |a| a.created(1))
            // Tasks without tags: untagged, orphan, isolated + done_task, review_task >= 3
            .step("test_tasks_without_tags", |a| a.rows_gte(3))
            // Tasks without subtasks >= 5
            .step("test_tasks_without_subtasks", |a| a.rows_gte(5))
            // Tasks not blocking anything >= 5
            .step("test_tasks_not_blocking_others", |a| a.rows_gte(5))
            // Tasks not blocked >= 4
            .step("test_tasks_not_blocked", |a| a.rows_gte(4))
            // Todo tasks without tags >= 3
            .step("test_todo_tasks_without_tags", |a| a.rows_gte(3))
            // High priority without blockers >= 1
            .step("test_high_priority_tasks_without_blockers", |a| a.rows_gte(1))
            // Tasks without urgent tags >= 5
            .step("test_tasks_without_urgent_tags", |a| a.rows_gte(5))
            // Tasks without completed subtasks >= 1
            .step("test_tasks_without_completed_subtasks", |a| a.rows_gte(1))
            // Tasks with no transitive dependencies >= 3
            .step("test_tasks_with_no_transitive_dependencies", |a| a.rows_gte(3))
            // Setup unused tag
            .step("test_setup_unused_tag", |a| a.created(1))
            // Unused tags >= 1
            .step("test_unused_tags", |a| a.rows_gte(1))
            // Tasks without tags or subtasks >= 3
            .step("test_tasks_without_tags_or_subtasks", |a| a.rows_gte(3))
            // Tasks not related to API task >= 3
            .step("test_tasks_not_related_to_api_task", |a| a.rows_gte(3))
            // Setup subtasks with attributes
            .step("test_setup_blocking_subtask", |a| a.created(1))
            .step("test_setup_nonblocking_subtask", |a| a.created(1))
            // Non-blocking subtasks >= 2
            .step("test_subtasks_that_are_not_blocking", |a| a.rows_gte(2))
            // Truly isolated tasks >= 3
            .step("test_truly_isolated_tasks", |a| a.rows_gte(3))
            // Development tasks without dev tag >= 0
            .step("test_development_tasks_without_dev_tag", |a| a.rows_gte(0))
            // Urgent priority without urgent tag >= 1
            .step("test_urgent_priority_without_urgent_tag", |a| a.rows_gte(1))
            // Count untagged tasks >= 3
            .step("test_count_untagged_tasks", |a| a.rows(1))
            // Count orphaned subtasks >= 2
            .step("test_count_orphaned_subtasks", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_test_entities", |a| a.deleted(3))
            .step("test_cleanup_test_subtasks", |a| a.deleted(2))
            .step("test_cleanup_unused_tag", |a| a.deleted(1))
    }

    #[test]
    fn test_not_exists_patterns() {
        scenario().run().unwrap();
    }
}

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
            .step("test_verify_tagged_links_exist", |a| a.scalar("tag_count", 3i64))
            // Verify blocking links = 2
            .step("test_verify_blocking_links_exist", |a| a.scalar("block_count", 2i64))
            // Unlink single tag
            .step("test_unlink_single_tag", |a| a.unlinked(1))
            // Verify single unlink = 0
            .step("test_verify_single_unlink", |a| a.scalar("tag_count", 0i64))
            // Verify other links remain = 2
            .step("test_verify_other_links_remain", |a| a.scalar("tag_count", 2i64))
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
            .step("test_verify_subtask_unlinked", |a| a.scalar("subtask_count", 0i64))
            // Recreate some tags
            .step("test_recreate_some_tags", |a| a.linked(2))
            // Unlink all tags from tag
            .step("test_unlink_all_tags_from_tag", |a| a.unlinked(2))
            // Verify all unlinked = 0
            .step("test_verify_all_unlinked", |a| a.scalar("tag_count", 0i64))
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
            .step("test_verify_relink", |a| a.scalar("subtask_count", 1i64))
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
            // SPAWN multiple with RETURNING
            .step("test_spawn_returning_multiple", |a| a.created(3).rows(3))
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
            // Task cannot block itself
            .step("test_task_cannot_block_itself", |a| a.error("self"))
            // Verify no self block = 0
            .step("test_verify_no_self_block_created", |a| a.scalar("block_count", 0i64))
            // Create blocking chain
            .step("test_create_blocking_chain", |a| a.linked(2))
            // Verify blocking chain
            .step("test_verify_blocking_chain", |a| a.rows(1))
            // Find all blocked by A = 1 (direct only)
            .step("test_find_all_blocked_by_a", |a| a.rows(1))
            // Find tasks that block D = 1 (C)
            .step("test_find_tasks_that_block_d", |a| a.rows(1))
            // Tasks not blocking anything = 1 (D)
            .step("test_find_tasks_not_blocking_anything", |a| a.rows(1))
            // Tasks not blocked = 1 (A)
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
            .step("test_verify_one_blocker_removed", |a| a.scalar("blocker_count", 2i64))
            // Find all transitively blocked >= 3
            .step("test_find_all_transitively_blocked", |a| a.rows_gte(3))
            // Find root blockers = 1 (A)
            .step("test_find_root_blockers", |a| a.rows(1))
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
