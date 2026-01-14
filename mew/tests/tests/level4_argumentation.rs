//! Level 4 - Argumentation integration tests.
//!
//! These tests run against the argumentation ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-4/argumentation/ontology.mew")
            .operations("level-4/argumentation/operations/crud.mew")
            .step("spawn_claim", |a| a.created(1))
            .step("query_count_claims", |a| a.value(1))
            .step("spawn_argument", |a| a.created(1))
            .step("link_addresses", |a| a.linked(1))
            .step("spawn_participant", |a| a.created(1))
            .step("link_authored", |a| a.linked(1))
            .step("link_stance", |a| a.linked(1))
            .step("spawn_evidence", |a| a.created(1))
            .step("link_backed", |a| a.linked(1))
            .step("spawn_debate", |a| a.created(1))
            .step("link_debates", |a| a.linked(1))
            .step("link_in_debate", |a| a.linked(1))
            .step("update_argument", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("query_all", |a| a.rows(1))
    }

    #[test]
    fn test_crud_operations_on_argumentation() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-4/argumentation/ontology.mew")
            .seed("level-4/argumentation/seeds/populated.mew")
            .operations("level-4/argumentation/operations/queries.mew")
            .step("count_all_claims", |a| a.value(3))
            .step("count_all_arguments", |a| a.value(5))
            .step("count_all_participants", |a| a.value(3))
            .step("count_all_debates", |a| a.value(2))
            .step("query_open_claims", |a| a.rows(1))
            .step("query_contested_claims", |a| a.rows(1))
            .step("query_active_arguments", |a| a.rows(4))
            .step("query_all_claims", |a| a.rows(3))
            .step("query_participants", |a| a.rows(3))
            .step("query_debates", |a| a.rows(2))
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
            .ontology("level-4/argumentation/ontology.mew")
            .operations("level-4/argumentation/operations/errors.mew")
            .step("spawn_missing_statement", |a| a.error("required"))
            .step("spawn_missing_content", |a| a.error("required"))
            .step("spawn_invalid_status", |a| a.error("constraint"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
