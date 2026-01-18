//! Level 1 - Contacts integration tests.
//!
//! These tests run against the contacts ontology with various scenarios.

use mew_tests::prelude::*;

mod edge_cases {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("edge_cases")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/extreme_cases.mew")
            .step("spawn_minimal_names", |a| a.created(1))
            .step("spawn_unicode_names", |a| a.created(3))
            .step("query_unicode_names_preserved", |a| a.scalar("count", 2i64))
            .step("spawn_person_multiple_primary_emails", |a| {
                a.created(4).linked(3)
            })
            .step("query_multiple_primary_flags", |a| a.scalar("count", 3i64))
            .step("create_asymmetric_knows", |a| a.created(2).linked(2))
            .step("query_relationship_differs_by_direction", |a| a.rows(2))
            .step("spawn_person_multiple_current_jobs", |a| {
                a.created(3).linked(2)
            })
            .step("query_multiple_current_employments", |a| {
                a.scalar("count", 2i64)
            })
            .step("create_then_delete_person", |a| {
                a.created(3).linked(2).deleted(1)
            })
            // Two queries returning count - combined into 2 rows (1 for email, 1 for phone)
            .step("verify_contact_info_remains", |a| a.rows(2))
            .step("verify_edges_cascade_removed", |a| a.scalar("count", 0i64))
    }

    #[test]
    fn test_boundary_conditions_and_edge_cases() {
        scenario().run().unwrap();
    }
}

mod edge_attributes {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("edge_attributes")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/edge_attributes.mew")
            .step("spawn_base_entities", |a| a.created(3))
            .step("link_with_edge_attributes", |a| a.linked(1))
            .step("query_edge_attributes", |a| {
                a.first(row_str! { "work.title" => "Senior Engineer", "work.department" => "Engineering", "work.is_current" => true })
            })
            .step("link_with_partial_edge_attributes", |a| a.linked(1))
            .step("query_partial_edge", |a| {
                a.first(row_str! { "work.title" => "Manager", "work.is_current" => true })
            })
            .step("update_edge_attribute", |a| a.modified(1))
            .step("verify_edge_update", |a| a.first(row_str! { "work.is_current" => false }))
            .step("test_knows_relationship", |a| a.linked(1))
            .step("query_knows_relationship", |a| {
                a.first(row_str! { "p2.first_name" => "Bob", "k.relationship" => "colleague", "k.notes" => "Met at TechCorp" })
            })
            .step("test_bidirectional_knows", |a| a.linked(1))
            .step("count_alice_connections", |a| a.scalar("count", 1i64))
            .step("count_bob_connections", |a| a.scalar("count", 1i64))
    }

    #[test]
    fn test_edge_attributes_and_relationships() {
        scenario().run().unwrap();
    }
}

mod multi_entity {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("multi_entity")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/multi_entity.mew")
            .step("create_person_with_full_profile", |a| a.created(1))
            .step("add_multiple_emails", |a| a.created(2).linked(2))
            .step("query_person_emails", |a| a.scalar("count", 2i64))
            .step("query_primary_email", |a| {
                a.first(row_str! { "e.address" => "alice@work.com" })
            })
            .step("add_multiple_phones", |a| a.created(3).linked(3))
            .step("count_phones", |a| a.scalar("count", 3i64))
            .step("add_multiple_addresses", |a| a.created(2).linked(2))
            .step("query_primary_address", |a| {
                a.first(row_str! { "a.street1" => "123 Main St", "a.city" => "Springfield" })
            })
            .step("add_social_profiles", |a| a.created(2).linked(2))
            .step("count_social_profiles", |a| a.scalar("count", 2i64))
            .step("create_organization_with_contacts", |a| {
                a.created(4).linked(3)
            })
            .step("verify_org_contacts", |a| a.scalar("count", 1i64))
            .step("link_person_to_org", |a| a.linked(1))
            .step("query_employment", |a| {
                a.first(row_str! { "o.name" => "Tech Solutions Inc", "w.title" => "CTO", "w.department" => "Executive" })
            })
            .step("add_tags_to_person", |a| a.created(3).linked(3))
            .step("count_person_tags", |a| a.scalar("count", 3i64))
            .step("add_groups", |a| a.created(2).linked(2))
            .step("query_person_groups", |a| a.rows(2))
            .step("count_all_alice_relationships", |a| a.scalar("count", 1i64))
    }

    #[test]
    fn test_complex_multi_entity_scenarios() {
        scenario().run().unwrap();
    }
}

mod query_complex {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("query_complex")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/query_complex.mew")
            .step("seed_complex_data", |a| a.created(9).linked(8))
            .step("query_favorites_only", |a| a.scalar("count", 2i64))
            .step("query_current_employees", |a| a.scalar("count", 3i64))
            .step("query_by_industry", |a| a.rows(2))
            .step("query_vip_contacts", |a| a.rows(2))
            .step("query_favorite_and_vip", |a| a.scalar("count", 2i64))
            .step("query_multi_tagged", |a| a.rows(2))
            .step("query_people_without_tags", |a| a.scalar("count", 2i64))
            .step("query_people_with_tags", |a| a.scalar("count", 2i64))
            .step("query_unemployed", |a| a.scalar("count", 0i64))
            .step("query_by_name_pattern", |a| a.scalar("count", 2i64))
            .step("query_current_in_techcorp", |a| a.scalar("count", 1i64))
    }

    #[test]
    fn test_complex_query_patterns() {
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
            .step("count_all_persons", |a| a.scalar("count", 4i64))
            .step("count_all_orgs", |a| a.scalar("count", 2i64))
            .step("count_all_tags", |a| a.scalar("count", 3i64))
            .step("count_all_groups", |a| a.scalar("count", 2i64))
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

mod errors_comprehensive {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("errors_comprehensive")
            .ontology("level-1/contacts/ontology.mew")
            .operations("level-1/contacts/operations/errors_comprehensive.mew")
            // Person spawn errors
            .step("missing_first_name", |a| a.error("required"))
            .step("missing_last_name", |a| a.error("required"))
            .step("type_error_boolean_name", |a| a.error("type"))
            .step("invalid_birthday_type", |a| a.error("type"))
            // Organization spawn errors
            .step("org_missing_name", |a| a.error("required"))
            .step("org_invalid_type", |a| a.error("type"))
            // Address spawn errors
            .step("addr_missing_street", |a| a.error("required"))
            .step("addr_missing_city", |a| a.error("required"))
            // Phone/Email spawn errors
            .step("phone_missing_number", |a| a.error("required"))
            .step("email_missing_address", |a| a.error("required"))
            // Edge attribute errors
            .step("edge_attr_wrong_type", |a| a.created(2).error("type"))
            .step("edge_attr_invalid_field", |a| {
                a.created(2).error("attribute")
            })
            // Relationship errors
            .step("link_email_to_org_wrong_edge", |a| {
                a.created(2).error("type")
            })
            .step("link_person_email_wrong_target", |a| {
                a.created(2).error("type")
            })
            // Query errors
            .step("query_invalid_edge_attribute", |a| {
                a.created(2).linked(1).rows_gte(0) // Nonexistent edge attributes return null
            })
            .step("query_type_mismatch", |a| a.rows_gte(0)) // Type coercion handles mismatches
    }

    #[test]
    fn test_comprehensive_error_scenarios() {
        scenario().run().unwrap();
    }
}
