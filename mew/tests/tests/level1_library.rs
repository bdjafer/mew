//! Level 1 - Library integration tests.
//!
//! These tests run against the library ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-1/library/ontology.mew")
            .operations("level-1/library/operations/crud.mew")
            .step("spawn_book", |a| a.created(1))
            .step("query_count_books", |a| a.value(1))
            .step("query_all_books", |a| a.rows(1))
            .step("spawn_author", |a| a.created(1))
            .step("link_book_author", |a| a.linked(1))
            .step("query_authors", |a| a.rows(1))
            .step("spawn_genre", |a| a.created(1))
            .step("link_book_genre", |a| a.linked(1))
            .step("spawn_copy", |a| a.created(1))
            .step("link_copy_book", |a| a.linked(1))
            .step("update_book", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("spawn_member", |a| a.created(1))
            .step("query_members", |a| a.rows(1))
            .step("kill_book", |a| a.deleted(1))
            .step("query_empty", |a| a.empty())
    }

    #[test]
    fn test_crud_operations_on_library() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-1/library/ontology.mew")
            .seed("level-1/library/seeds/populated.mew")
            .operations("level-1/library/operations/queries.mew")
            .step("count_all_books", |a| a.value(3))
            .step("count_all_authors", |a| a.value(3))
            .step("count_all_members", |a| a.value(3))
            .step("count_all_copies", |a| a.value(3))
            .step("query_active_members", |a| a.rows(2))
            .step("query_available_copies", |a| a.rows(2))
            .step("query_by_title", |a| a.rows(1))
            .step("query_all_titles", |a| a.rows(3))
            .step("query_genres", |a| a.rows(3))
            .step("query_authors", |a| a.rows(3))
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
            .ontology("level-1/library/ontology.mew")
            .operations("level-1/library/operations/errors.mew")
            .step("spawn_missing_required_isbn", |a| a.error("required"))
            .step("spawn_missing_required_title", |a| a.error("required"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
