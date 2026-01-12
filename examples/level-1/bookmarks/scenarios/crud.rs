//! CRUD Operations Scenario
//!
//! Tests basic create, read, update, delete operations for bookmarks.

use mew_examples::prelude::*;

/// Define the CRUD scenario.
pub fn scenario() -> Scenario {
    Scenario::new("crud")
        .ontology("level-1/bookmarks/ontology.mew")
        .operations("level-1/bookmarks/scenarios/crud.mew")
        // No seed - start with empty database
        .step("spawn_bookmark", |a| a.created(1))
        .step("query_count", |a| a.value(1))
        .step("query_all", |a| {
            a.rows(1).contains(row! {
                title: "Example Site",
                url: "https://example.com"
            })
        })
        .step("spawn_folder", |a| a.created(1))
        .step("link_bookmark_folder", |a| a.linked(1))
        .step("query_bookmark_in_folder", |a| {
            a.rows(1).contains(row! {
                title: "Example Site",
                name: "Test Folder"
            })
        })
        .step("update_bookmark", |a| a.modified(1))
        .step("query_updated", |a| {
            a.rows(1).contains(row! { title: "Updated Example" })
        })
        .step("spawn_tag", |a| a.created(1))
        .step("link_bookmark_tag", |a| a.linked(1))
        .step("query_tagged", |a| {
            a.rows(1).contains(row! {
                title: "Updated Example",
                name: "test-tag"
            })
        })
        .step("kill_bookmark", |a| a.deleted(1))
        .step("query_empty", |a| a.empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crud() {
        scenario().run().unwrap();
    }
}
