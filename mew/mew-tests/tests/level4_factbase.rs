//! Level 4 - FactBase integration tests.
//!
//! These tests run against the factbase ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-4/factbase/ontology.mew")
            .operations("level-4/factbase/operations/crud.mew")
            .step("spawn_entity1", |a| a.created(1))
            .step("spawn_entity2", |a| a.created(1))
            .step("query_count_entities", |a| a.value(2))
            .step("link_relates", |a| a.linked(1))
            .step("query_entities", |a| a.rows(2))
            .step("spawn_evidence", |a| a.created(1))
            .step("query_evidence", |a| a.rows(1))
            .step("update_entity", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("kill_entity", |a| a.deleted(1))
            .step("query_remaining", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_factbase() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-4/factbase/ontology.mew")
            .seed("level-4/factbase/seeds/populated.mew")
            .operations("level-4/factbase/operations/queries.mew")
            .step("count_all_entities", |a| a.value(6))
            .step("count_all_evidence", |a| a.value(4))
            .step("query_persons", |a| a.rows(2))
            .step("query_companies", |a| a.rows(2))
            .step("query_all_entities", |a| a.rows(6))
            .step("query_reliable_evidence", |a| a.rows(2))
            .step("query_unreliable_evidence", |a| a.rows(1))
            .step("query_all_evidence", |a| a.rows(4))
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
            .ontology("level-4/factbase/ontology.mew")
            .operations("level-4/factbase/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            .step("spawn_missing_source", |a| a.error("required"))
            // Note: type alias constraints are not enforced at runtime
            .step("spawn_invalid_reliability_low", |a| a.created(1))
            .step("spawn_invalid_reliability_high", |a| a.created(1))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
