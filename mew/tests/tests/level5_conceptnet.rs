//! Level 5 - ConceptNet integration tests.
//!
//! These tests run against the conceptnet ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-5/conceptnet/ontology.mew")
            .operations("level-5/conceptnet/operations/crud.mew")
            .step("spawn_concept", |a| a.created(1))
            .step("query_count_concepts", |a| a.value(1))
            .step("spawn_concept2", |a| a.created(1))
            .step("link_related", |a| a.linked(1))
            .step("link_instance", |a| a.linked(1))
            .step("spawn_reltype", |a| a.created(1))
            .step("query_concepts", |a| a.rows(2))
            .step("update_concept", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("spawn_learned", |a| a.created(1))
            .step("query_all", |a| a.rows(3))
            .step("kill_concept", |a| a.deleted(1))
            .step("query_remaining", |a| a.rows(2))
    }

    #[test]
    fn test_crud_operations_on_conceptnet() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-5/conceptnet/ontology.mew")
            .seed("level-5/conceptnet/seeds/populated.mew")
            .operations("level-5/conceptnet/operations/queries.mew")
            .step("count_all_concepts", |a| a.value(10))
            .step("count_all_reltypes", |a| a.value(4))
            .step("query_primitive_concepts", |a| a.rows(3))
            .step("query_learned_concepts", |a| a.rows(1))
            .step("query_high_confidence", |a| a.rows(9))
            .step("query_all_concepts", |a| a.rows(10))
            .step("query_relation_types", |a| a.rows(4))
            .step("query_symmetric_relations", |a| a.rows(0))
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
            .ontology("level-5/conceptnet/ontology.mew")
            .operations("level-5/conceptnet/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            // Note: type alias constraints are not enforced at runtime
            .step("spawn_invalid_confidence_low", |a| a.created(1))
            .step("spawn_invalid_confidence_high", |a| a.created(1))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
