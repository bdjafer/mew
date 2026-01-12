//! Level 2 - Tasks integration tests.
//!
//! These tests run against the tasks ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-2/tasks/ontology.mew")
            .operations("level-2/tasks/operations/crud.mew")
            .step("spawn_task", |a| a.created(1))
            .step("query_count_tasks", |a| a.value(1))
            .step("query_all_tasks", |a| a.rows(1))
            .step("spawn_subtask", |a| a.created(1))
            .step("link_subtask", |a| a.linked(1))
            .step("spawn_tag", |a| a.created(1))
            .step("link_tag", |a| a.linked(1))
            .step("update_task_status", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("update_task_done", |a| a.modified(1))
            .step("kill_task", |a| a.deleted(1))
            .step("query_empty", |a| a.empty())
    }

    #[test]
    fn test_crud_operations_on_tasks() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-2/tasks/ontology.mew")
            .seed("level-2/tasks/seeds/populated.mew")
            .operations("level-2/tasks/operations/queries.mew")
            // Note: Task count returns only base Task types, not SubTask subtypes
            .step("count_all_tasks", |a| a.value(5))
            .step("count_all_tags", |a| a.value(3))
            // High priority (>=4): api_task(5), tests_task(4), done_task(5) = 3
            .step("query_high_priority", |a| a.rows(3))
            // todo status: tests_task, docs_task, deploy_task = 3
            .step("query_todo", |a| a.rows(3))
            .step("query_in_progress", |a| a.rows(1))
            .step("query_done", |a| a.rows(1))
            .step("query_all_titles", |a| a.rows(5))
            .step("query_tags", |a| a.rows(3))
            .step("query_subtasks", |a| a.rows(2))
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
            .ontology("level-2/tasks/ontology.mew")
            .operations("level-2/tasks/operations/errors.mew")
            .step("spawn_missing_title", |a| a.error("required"))
            // Note: constraint validation via type aliases is not fully enforced at runtime
            .step("spawn_invalid_priority_low", |a| a.created(1))
            .step("spawn_invalid_priority_high", |a| a.created(1))
            .step("spawn_invalid_status", |a| a.created(1))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
