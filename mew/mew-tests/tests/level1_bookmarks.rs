//! Level 1 - Bookmarks integration tests.
//!
//! These tests run against the bookmarks ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-1/bookmarks/ontology.mew")
            .operations("level-1/bookmarks/operations/crud.mew")
            // No seed - start with empty database
            .step("spawn_bookmark", |a| a.created(1))
            .step("query_count", |a| a.value(1))
            .step("query_all", |a| a.rows(1))
            .step("spawn_folder", |a| a.created(1))
            .step("link_bookmark_folder", |a| a.linked(1))
            .step("query_folders", |a| a.rows(1))
            .step("update_bookmark", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("spawn_tag", |a| a.created(1))
            .step("link_bookmark_tag", |a| a.linked(1))
            .step("query_tags", |a| a.rows(1))
            .step("kill_bookmark", |a| a.deleted(1))
            .step("query_empty", |a| a.empty())
    }

    #[test]
    fn test_crud_operations_on_bookmarks() {
        // GIVEN: an empty database with bookmarks ontology loaded

        // WHEN: crud operations are executed (spawn, query, link, update, delete)

        // THEN: all operations complete successfully
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-1/bookmarks/ontology.mew")
            .seed("level-1/bookmarks/seeds/populated.mew")
            .operations("level-1/bookmarks/operations/queries.mew")
            .step("count_all_bookmarks", |a| a.value(5))
            .step("count_all_folders", |a| a.value(3))
            .step("count_all_tags", |a| a.value(3))
            .step("query_favorites", |a| a.rows(2))
            .step("query_by_title_pattern", |a| a.rows(1))
            .step("query_all_titles", |a| a.rows(5))
            .step("query_folders", |a| a.rows(3))
            .step("query_tags", |a| a.rows(3))
    }

    #[test]
    fn test_query_operations_with_populated_data() {
        // GIVEN: a database with populated bookmarks seed data

        // WHEN: various query operations are executed

        // THEN: all queries return expected row counts
        scenario().run().unwrap();
    }
}

mod errors {
    use super::*;

    /// Test error cases.
    pub fn scenario() -> Scenario {
        Scenario::new("errors")
            .ontology("level-1/bookmarks/ontology.mew")
            .operations("level-1/bookmarks/operations/errors.mew")
            .step("spawn_missing_required", |a| a.error("required"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        // GIVEN: a database with bookmarks ontology loaded

        // WHEN: operations with invalid data are executed

        // THEN: appropriate errors are returned
        scenario().run().unwrap();
    }
}
