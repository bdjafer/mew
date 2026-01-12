//! Level 1 - Contacts integration tests.
//!
//! These tests run against the contacts ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/crud.mew")
            .step("spawn_person", |a| a.created(1))
            .step("query_count_persons", |a| a.value(1))
            .step("query_all_persons", |a| a.rows(1))
            .step("spawn_org", |a| a.created(1))
            .step("link_works_at", |a| a.linked(1))
            .step("query_orgs", |a| a.rows(1))
            .step("spawn_email", |a| a.created(1))
            .step("link_has_email", |a| a.linked(1))
            .step("spawn_phone", |a| a.created(1))
            .step("link_has_phone", |a| a.linked(1))
            .step("spawn_tag", |a| a.created(1))
            .step("link_person_tag", |a| a.linked(1))
            .step("update_person", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("kill_person", |a| a.deleted(1))
            .step("query_empty", |a| a.empty())
    }

    #[test]
    fn test_crud_operations_on_contacts() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-1/contacts/ontology.mew")
            .seed("level-1/contacts/seeds/populated.mew")
            .operations("level-1/contacts/operations/queries.mew")
            .step("count_all_persons", |a| a.value(4))
            .step("count_all_orgs", |a| a.value(2))
            .step("count_all_tags", |a| a.value(3))
            .step("count_all_groups", |a| a.value(2))
            .step("query_favorites", |a| a.rows(2))
            .step("query_by_name", |a| a.rows(1))
            .step("query_all_names", |a| a.rows(4))
            .step("query_orgs", |a| a.rows(2))
            .step("query_tags", |a| a.rows(3))
            .step("query_groups", |a| a.rows(2))
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
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/errors.mew")
            .step("spawn_missing_first_name", |a| a.error("required"))
            .step("spawn_missing_last_name", |a| a.error("required"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
