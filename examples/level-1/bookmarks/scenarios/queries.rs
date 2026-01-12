//! Query Operations Scenario
//!
//! Tests various query patterns against a populated database.

use mew_examples::prelude::*;

/// Define the queries scenario.
pub fn scenario() -> Scenario {
    Scenario::new("queries")
        .ontology("level-1/bookmarks/ontology.mew")
        .seed("level-1/bookmarks/seeds/populated.mew")
        .operations("level-1/bookmarks/scenarios/queries.mew")
        .step("count_all_bookmarks", |a| a.value(5))
        .step("count_all_folders", |a| a.value(3))
        .step("count_all_tags", |a| a.value(3))
        .step("query_favorites", |a| {
            a.rows(2)
                .contains(row! { title: "Google" })
                .contains(row! { title: "Hacker News" })
        })
        .step("query_by_title_pattern", |a| {
            a.rows(1).contains(row! { url: "https://google.com" })
        })
        .step("query_all_titles", |a| a.rows(5))
        .step("query_folders", |a| {
            a.rows(3)
                .contains(row! { name: "Work" })
                .contains(row! { name: "Personal" })
                .contains(row! { name: "Reading List" })
        })
        .step("query_tags", |a| {
            a.rows(3)
                .contains(row! { name: "dev", color: "#00ff00" })
                .contains(row! { name: "news", color: "#0000ff" })
                .contains(row! { name: "reference", color: "#ff0000" })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queries() {
        scenario().run().unwrap();
    }
}
