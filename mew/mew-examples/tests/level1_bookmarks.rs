//! Level 1 - Bookmarks integration tests.
//!
//! These tests run against the bookmarks ontology with various scenarios.

use mew_examples::prelude::*;

/// Base path helper for this test file.
fn examples_path() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
}

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .base_path(examples_path())
            .ontology("level-1/bookmarks/ontology.mew")
            .operations("level-1/bookmarks/scenarios/crud.mew")
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
    fn test() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .base_path(examples_path())
            .ontology("level-1/bookmarks/ontology.mew")
            .seed("level-1/bookmarks/seeds/populated.mew")
            .operations("level-1/bookmarks/scenarios/queries.mew")
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
    fn test() {
        scenario().run().unwrap();
    }
}

mod errors {
    use super::*;

    /// Test error cases.
    pub fn scenario() -> Scenario {
        Scenario::new("errors")
            .base_path(examples_path())
            .ontology("level-1/bookmarks/ontology.mew")
            .operations("level-1/bookmarks/scenarios/errors.mew")
            .step("spawn_missing_required", |a| a.error("required"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test() {
        scenario().run().unwrap();
    }
}
