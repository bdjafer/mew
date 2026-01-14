//! Level 5 - BDI integration tests.
//!
//! These tests run against the bdi ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-5/bdi/ontology.mew")
            .operations("level-5/bdi/operations/crud.mew")
            .step("spawn_agent", |a| a.created(1))
            .step("query_count_agents", |a| a.value(1))
            .step("spawn_belief", |a| a.created(1))
            .step("link_believes", |a| a.linked(1))
            .step("spawn_goal", |a| a.created(1))
            .step("link_desires", |a| a.linked(1))
            .step("spawn_plan", |a| a.created(1))
            .step("link_achieves", |a| a.linked(1))
            .step("spawn_intention", |a| a.created(1))
            .step("link_intends", |a| a.linked(1))
            .step("link_intention_goal", |a| a.linked(1))
            .step("link_using_plan", |a| a.linked(1))
            .step("update_goal", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("query_agents", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_bdi() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-5/bdi/ontology.mew")
            .seed("level-5/bdi/seeds/populated.mew")
            .operations("level-5/bdi/operations/queries.mew")
            .step("count_all_agents", |a| a.value(2))
            .step("count_all_beliefs", |a| a.value(4))
            .step("count_all_goals", |a| a.value(5))
            .step("count_all_plans", |a| a.value(2))
            // 4 active goals: reach, avoid + fast, careful default to "active"
            .step("query_active_goals", |a| a.rows(4))
            // priority >= 0.8: reach=0.9, avoid=0.8, achieved=1.0
            .step("query_high_priority_goals", |a| a.rows(3))
            // confidence >= 0.9: loc_belief=1.0, goal_loc_belief=0.9
            .step("query_high_confidence_beliefs", |a| a.rows(2))
            .step("query_all_goals", |a| a.rows(5))
            .step("query_plans", |a| a.rows(2))
            .step("query_intentions", |a| a.rows(2))
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
            .ontology("level-5/bdi/ontology.mew")
            .operations("level-5/bdi/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            .step("spawn_missing_content", |a| a.error("required"))
            .step("spawn_invalid_confidence", |a| a.error("constraint"))
            .step("spawn_invalid_status", |a| a.error("constraint"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
