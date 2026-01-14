//! Level 3 - ProjectManagement integration tests.
//!
//! These tests run against the projectmanagement ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-3/projectmanagement/ontology.mew")
            .operations("level-3/projectmanagement/operations/crud.mew")
            .step("spawn_project", |a| a.created(1))
            .step("query_count_projects", |a| a.value(1))
            .step("spawn_task", |a| a.created(1))
            .step("link_task_project", |a| a.linked(1))
            .step("spawn_milestone", |a| a.created(1))
            .step("link_milestone_project", |a| a.linked(1))
            .step("link_task_milestone", |a| a.linked(1))
            .step("spawn_member", |a| a.created(1))
            .step("link_assigned", |a| a.linked(1))
            .step("link_member_project", |a| a.linked(1))
            .step("update_task_status", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("update_task_done", |a| a.modified(1))
            // kill_task fails due to edge restriction - deletion blocked by belongs_to edges
            .step("kill_task", |a| a.error("Deletion"))
            // Task still exists because delete was blocked
            .step("query_empty", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_projectmanagement() {
        scenario().run().unwrap();
    }
}

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
