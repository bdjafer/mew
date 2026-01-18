//! Level 3 - ProjectManagement integration tests.
//!
//! These tests run against the projectmanagement ontology with various scenarios.

use mew_tests::prelude::*;

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-3/projectmanagement/ontology.mew")
            .seed("level-3/projectmanagement/seeds/populated.mew")
            .operations("level-3/projectmanagement/operations/queries.mew")
            .step("count_all_projects", |a| a.value(2))
            .step("count_all_tasks", |a| a.value(5))
            .step("count_all_milestones", |a| a.value(2))
            .step("count_all_members", |a| a.value(3))
            .step("query_active_projects", |a| a.rows(1))
            .step("query_high_priority_tasks", |a| a.rows(3))
            .step("query_todo_tasks", |a| a.rows(1))
            .step("query_in_progress", |a| a.rows(1))
            .step("query_all_tasks", |a| a.rows(5))
            .step("query_members", |a| a.rows(3))
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
            .ontology("level-3/projectmanagement/ontology.mew")
            .operations("level-3/projectmanagement/operations/errors.mew")
            .step("spawn_missing_title", |a| a.error("required"))
            .step("spawn_invalid_priority_low", |a| a.error("constraint"))
            .step("spawn_invalid_priority_high", |a| a.error("constraint"))
            .step("spawn_invalid_status", |a| a.error("constraint"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}

mod cascade_behavior {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("cascade_behavior")
            .ontology("level-3/projectmanagement/ontology.mew")
            .operations("level-3/projectmanagement/operations/cascade_behavior.mew")
            // Setup
            .step("test_setup_project", |a| a.created(1))
            .step("test_setup_milestone", |a| a.created(1).linked(1))
            .step("test_setup_tasks", |a| a.created(3).linked(3))
            // Verify initial state
            .step("test_verify_tasks_exist", |a| a.value(3))
            .step("test_verify_milestones_exist", |a| a.value(1))
            .step("test_verify_project_links", |a| a.value(3))
            // Kill project - cascade deletes tasks and milestones
            .step("test_kill_project", |a| a.deleted(5)) // 1 project + 3 tasks + 1 milestone
            // Verify cascade
            .step("test_verify_tasks_cascaded", |a| a.value(0))
            .step("test_verify_milestones_cascaded", |a| a.value(0))
            .step("test_verify_project_gone", |a| a.value(0))
    }

    #[test]
    fn test_cascade_referential_action() {
        scenario().run().unwrap();
    }
}

mod cardinality_violation {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("cardinality_violation")
            .ontology("level-3/projectmanagement/ontology.mew")
            .operations("level-3/projectmanagement/operations/cardinality_violation.mew")
            // Setup
            .step("test_setup_projects", |a| a.created(2))
            .step("test_setup_task", |a| a.created(1))
            .step("test_setup_members", |a| a.created(2))
            // Valid link
            .step("test_link_task_to_project1", |a| a.linked(1))
            .step("test_verify_task_belongs", |a| a.rows(1))
            // Cardinality violation: task -> 1 on belongs_to
            .step("test_cardinality_violation_belongs_to", |a| {
                a.error("cardinality")
            })
            .step("test_verify_still_one_project", |a| a.value(1))
            // Valid assignment
            .step("test_assign_to_alice", |a| a.linked(1))
            .step("test_verify_assignment", |a| a.rows(1))
            // Cardinality violation: task -> 0..1 on assigned_to
            .step("test_cardinality_violation_assigned_to", |a| {
                a.error("cardinality")
            })
            .step("test_verify_still_one_assignee", |a| a.value(1))
            // Cleanup
            .step("test_cleanup_tasks", |a| a.deleted(1))
            .step("test_cleanup_members", |a| a.deleted(2))
            .step("test_cleanup_projects", |a| a.deleted(2))
    }

    #[test]
    fn test_cardinality_constraint_enforcement() {
        scenario().run().unwrap();
    }
}

mod acyclic_violation {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("acyclic_violation")
            .ontology("level-3/projectmanagement/ontology.mew")
            .operations("level-3/projectmanagement/operations/acyclic_violation.mew")
            // Setup
            .step("test_setup_tasks", |a| a.created(3))
            // Valid dependency chain
            .step("test_valid_dependency_chain", |a| a.linked(2))
            .step("test_verify_chain", |a| a.rows(2))
            // Self-dependency violation (caught by acyclic constraint)
            .step("test_self_dependency_violation", |a| a.error("Acyclic"))
            .step("test_verify_no_self_loop", |a| a.value(0))
            // Cycle violation (acyclic modifier)
            .step("test_cycle_violation", |a| a.error("Acyclic"))
            .step("test_verify_no_cycle", |a| a.value(0))
            // Valid branching dependency
            .step("test_setup_task_d", |a| a.created(1))
            .step("test_valid_branching_dependency", |a| a.linked(1))
            .step("test_verify_branching", |a| a.value(2)) // A and D both depend on B
            // Cleanup
            .step("test_cleanup", |a| a.deleted(4))
    }

    #[test]
    fn test_acyclic_constraint_enforcement() {
        scenario().run().unwrap();
    }
}
