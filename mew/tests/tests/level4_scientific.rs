//! Level 4 - Scientific integration tests.
//!
//! These tests run against the scientific ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-4/scientific/ontology.mew")
            .operations("level-4/scientific/operations/crud.mew")
            .step("spawn_concept1", |a| a.created(1))
            .step("spawn_concept2", |a| a.created(1))
            .step("query_count_concepts", |a| a.value(2))
            .step("spawn_hypothesis", |a| a.created(1))
            .step("link_causation", |a| a.linked(1))
            .step("spawn_researcher", |a| a.created(1))
            .step("spawn_experiment", |a| a.created(1))
            .step("link_tests", |a| a.linked(1))
            .step("link_conducted", |a| a.linked(1))
            .step("update_hypothesis", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("query_concepts", |a| a.rows(2))
            .step("kill_concept", |a| a.deleted(1))
            .step("query_remaining", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_scientific() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-4/scientific/ontology.mew")
            .seed("level-4/scientific/seeds/populated.mew")
            .operations("level-4/scientific/operations/queries.mew")
            .step("count_all_concepts", |a| a.value(5))
            .step("count_all_hypotheses", |a| a.value(3))
            .step("count_all_experiments", |a| a.value(2))
            .step("count_all_researchers", |a| a.value(2))
            .step("query_health_concepts", |a| a.rows(2))
            .step("query_proposed_hypotheses", |a| a.rows(1))
            .step("query_established_hypotheses", |a| a.rows(1))
            .step("query_all_hypotheses", |a| a.rows(3))
            .step("query_researchers", |a| a.rows(2))
            .step("query_publications", |a| a.rows(2))
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
            .ontology("level-4/scientific/ontology.mew")
            .operations("level-4/scientific/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            .step("spawn_missing_statement", |a| a.error("required"))
            // Note: type alias constraints are not enforced at runtime
            .step("spawn_invalid_status", |a| a.created(1))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
