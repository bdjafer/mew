//! Level 5 - CognitiveAgent integration tests.
//!
//! These tests run against the cognitiveagent ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-5/cognitiveagent/ontology.mew")
            .operations("level-5/cognitiveagent/operations/crud.mew")
            .step("spawn_agent", |a| a.created(1))
            .step("query_count_agents", |a| a.value(1))
            .step("spawn_belief", |a| a.created(1))
            .step("link_believes", |a| a.linked(1))
            .step("spawn_goal", |a| a.created(1))
            .step("link_has_goal", |a| a.linked(1))
            .step("spawn_plan", |a| a.created(1))
            .step("link_plan_goal", |a| a.linked(1))
            .step("spawn_action", |a| a.created(1))
            .step("link_plan_action", |a| a.linked(1))
            .step("spawn_concept", |a| a.created(1))
            .step("spawn_selfmodel", |a| a.created(1))
            .step("link_selfmodel", |a| a.linked(1))
            .step("update_agent", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("query_agents", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_cognitiveagent() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-5/cognitiveagent/ontology.mew")
            .seed("level-5/cognitiveagent/seeds/populated.mew")
            .operations("level-5/cognitiveagent/operations/queries.mew")
            .step("count_all_agents", |a| a.value(2))
            .step("count_all_beliefs", |a| a.value(4))
            // Note: only base Goal types counted, not SubGoal subtypes
            .step("count_all_goals", |a| a.value(3))
            .step("count_all_concepts", |a| a.value(2))
            .step("query_active_beliefs", |a| a.rows(3))
            // 2 active goals (primary, secondary)
            .step("query_active_goals", |a| a.rows(2))
            // priority >= 0.8: primary=0.95, achieved=1.0
            .step("query_high_priority_goals", |a| a.rows(2))
            // confidence >= 0.9: loc_belief=1.0, goal_belief=0.9
            .step("query_high_confidence_beliefs", |a| a.rows(2))
            .step("query_all_goals", |a| a.rows(3))
            .step("query_concepts", |a| a.rows(2))
            .step("query_plans", |a| a.rows(2))
            .step("query_selfmodels", |a| a.rows(1))
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
            .ontology("level-5/cognitiveagent/ontology.mew")
            .operations("level-5/cognitiveagent/operations/errors.mew")
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
