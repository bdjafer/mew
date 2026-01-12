//! Level 3 - EventChain integration tests.
//!
//! These tests run against the eventchain ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-3/eventchain/ontology.mew")
            .operations("level-3/eventchain/operations/crud.mew")
            .step("spawn_event", |a| a.created(1))
            .step("query_count_events", |a| a.value(1))
            .step("query_all_events", |a| a.rows(1))
            .step("spawn_effect", |a| a.created(1))
            .step("link_causes", |a| a.linked(1))
            .step("query_events", |a| a.rows(2))
            .step("update_event", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("spawn_third", |a| a.created(1))
            .step("link_chain", |a| a.linked(1))
            .step("kill_event", |a| a.deleted(1))
            .step("query_remaining", |a| a.rows(2))
    }

    #[test]
    fn test_crud_operations_on_eventchain() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-3/eventchain/ontology.mew")
            .seed("level-3/eventchain/seeds/populated.mew")
            .operations("level-3/eventchain/operations/queries.mew")
            .step("count_all_events", |a| a.value(9))
            .step("query_triggers", |a| a.rows(2))
            .step("query_effects", |a| a.rows(4))
            .step("query_outcomes", |a| a.rows(1))
            .step("query_all_names", |a| a.rows(9))
            .step("query_with_description", |a| a.rows(5))
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
            .ontology("level-3/eventchain/ontology.mew")
            .operations("level-3/eventchain/operations/errors.mew")
            .step("spawn_missing_name", |a| a.error("required"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
