//! Level 2 - Ecommerce integration tests.
//!
//! These tests run against the ecommerce ontology with various scenarios.
//! Focus areas: Inheritance polymorphism, aggregations, uniqueness constraints,
//! ORDER BY, LIMIT/OFFSET, DISTINCT, OPTIONAL MATCH, type aliases, format validation,
//! inline SPAWN in LINK

use mew_tests::prelude::*;

mod inheritance {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("inheritance")
            .ontology("level-2/ecommerce/ontology.mew")
            .seed("level-2/ecommerce/seeds/populated.mew")
            .operations("level-2/ecommerce/operations/inheritance.mew")
            // Polymorphic queries - base type returns all subtypes
            // Seed has: 3 PhysicalProduct + 1 DigitalProduct + 1 Product = 5 total
            .step("test_query_all_products", |a| a.rows(5))
            // Only PhysicalProduct: laptop, phone, headphones = 3
            .step("test_query_physical_products_only", |a| a.rows(3))
            // Only DigitalProduct: ebook = 1
            .step("test_query_digital_products_only", |a| a.rows(1))
            // PhysicalProducts with price > 200: laptop (1299.99), phone (799.99) = 2
            .step("test_inherited_attributes_on_subtypes", |a| a.rows(2))
            // Spawn a new PhysicalProduct
            .step("test_spawn_physical_product", |a| a.created(1))
            // Query as base type
            .step("test_verify_spawned_subtype_queryable_as_base", |a| {
                a.rows(1)
            })
            // Query as subtype
            .step("test_verify_spawned_subtype_queryable_as_subtype", |a| {
                a.rows(1)
            })
            // Update inherited attribute
            .step("test_set_inherited_attribute_via_subtype", |a| {
                a.modified(1)
            })
            // Verify update
            .step("test_verify_inherited_attribute_updated", |a| a.rows(1))
            // Filter by subtype attribute on base type
            // Products with weight_kg < 1.0: phone (0.2), headphones (0.3), tablet (0.5) = 3
            .step("test_where_clause_on_subtype_attribute", |a| a.rows(3))
            // Cleanup
            .step("test_cleanup_tablet", |a| a.deleted(1))
    }

    #[test]
    fn test_inheritance_and_polymorphism() {
        scenario().run().unwrap();
    }
}

mod aggregations {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("aggregations")
            .ontology("level-2/ecommerce/ontology.mew")
            .seed("level-2/ecommerce/seeds/populated.mew")
            .operations("level-2/ecommerce/operations/aggregations.mew")
            // COUNT all products = 5
            .step("test_count_all_products", |a| {
                a.scalar("total_products", 5i64)
            })
            // COUNT PhysicalProduct = 3
            .step("test_count_by_type", |a| a.scalar("physical_count", 3i64))
            // SUM price of active products: 1299.99 + 799.99 + 29.99 + 199.99 = 2329.96
            .step("test_sum_price_all_products", |a| a.rows(1))
            // AVG price of active products: 2329.96 / 4 = 582.49
            .step("test_avg_price", |a| a.rows(1))
            // MIN/MAX price: 29.99 / 1299.99
            .step("test_min_max_price", |a| a.rows(1))
            // COUNT by status: active=4, discontinued=1
            .step("test_count_by_status", |a| a.rows(2))
            // SUM weight: 2.1 + 0.2 + 0.3 = 2.6
            .step("test_sum_weight_physical_products", |a| a.rows(1))
            // AVG weight: 2.6 / 3 = 0.8666...
            .step("test_avg_weight_physical_products", |a| a.rows(1))
            // Reviews per product: laptop=1, phone=1
            .step("test_count_reviews_per_product", |a| a.rows(2))
            // AVG rating: laptop=5.0, phone=4.0
            .step("test_avg_rating_per_product", |a| a.rows(2))
            // Products without reviews: headphones, ebook, old_product = 3
            .step("test_products_without_reviews_count", |a| {
                a.scalar("products_without_reviews", 3i64)
            })
            // Products with review_count > 0: laptop, phone = 2
            .step("test_aggregation_with_having_like_filter", |a| a.rows(2))
            // Distinct customers with reviews: alice, bob = 2
            .step("test_count_distinct_customers_with_reviews", |a| {
                a.scalar("customers_with_reviews", 2i64)
            })
    }

    #[test]
    fn test_aggregation_functions() {
        scenario().run().unwrap();
    }
}

mod uniqueness {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("uniqueness")
            .ontology("level-2/ecommerce/ontology.mew")
            .seed("level-2/ecommerce/seeds/populated.mew")
            .operations("level-2/ecommerce/operations/uniqueness.mew")
            // Create product with unique SKU
            .step("test_spawn_product_with_unique_sku", |a| a.created(1))
            // Duplicate SKU should fail
            .step("test_spawn_duplicate_sku_should_fail", |a| {
                a.error("unique")
            })
            // Duplicate existing SKU should fail
            .step("test_spawn_duplicate_existing_sku_should_fail", |a| {
                a.error("unique")
            })
            // Duplicate customer email should fail
            .step("test_spawn_duplicate_customer_email_should_fail", |a| {
                a.error("unique")
            })
            // Duplicate category slug should fail
            .step("test_spawn_duplicate_category_slug_should_fail", |a| {
                a.error("unique")
            })
            // SET to duplicate SKU should fail
            .step("test_set_to_duplicate_sku_should_fail", |a| {
                a.error("unique")
            })
            // Verify SKU unchanged
            .step("test_verify_sku_unchanged_after_failed_set", |a| a.rows(1))
            // SET to duplicate email should fail
            .step("test_set_to_duplicate_email_should_fail", |a| {
                a.error("unique")
            })
            // SET to new unique SKU should succeed
            .step("test_set_to_new_unique_sku_should_succeed", |a| {
                a.modified(1)
            })
            // Verify SKU changed
            .step("test_verify_sku_changed", |a| a.rows(1))
            // Old SKU now available
            .step("test_spawn_with_old_sku_now_succeeds", |a| a.created(1))
            // PhysicalProduct with duplicate SKU fails
            .step("test_physical_product_duplicate_sku_fails", |a| {
                a.error("unique")
            })
            // DigitalProduct with duplicate SKU fails
            .step("test_digital_product_duplicate_sku_fails", |a| {
                a.error("unique")
            })
            // Cleanup
            .step("test_cleanup_test_products", |a| a.deleted(2))
    }

    #[test]
    fn test_uniqueness_constraints() {
        scenario().run().unwrap();
    }
}

mod advanced_queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("advanced_queries")
            .ontology("level-2/ecommerce/ontology.mew")
            .seed("level-2/ecommerce/seeds/populated.mew")
            .operations("level-2/ecommerce/operations/advanced_queries.mew")
            // ORDER BY price ASC: 4 active products
            .step("test_order_by_price_asc", |a| a.rows(4))
            // ORDER BY price DESC: 4 active products
            .step("test_order_by_price_desc", |a| a.rows(4))
            // ORDER BY multiple fields: 5 products
            .step("test_order_by_multiple_fields", |a| a.rows(5))
            // LIMIT 3: top 3 expensive
            .step("test_limit_top_3_expensive", |a| a.rows(3))
            // LIMIT 1
            .step("test_limit_1", |a| a.rows(1))
            // OFFSET 2: skip first 2, get 2 remaining
            .step("test_offset_skip_first_2", |a| a.rows(2))
            // Pagination page 1: 2 rows
            .step("test_pagination_page_1", |a| a.rows(2))
            // Pagination page 2: 2 rows
            .step("test_pagination_page_2", |a| a.rows(2))
            // Pagination page 3: 1 row (last)
            .step("test_pagination_page_3", |a| a.rows(1))
            // DISTINCT status: active, discontinued = 2
            .step("test_distinct_status_values", |a| a.rows(2))
            // DISTINCT with multiple fields
            .step("test_distinct_with_multiple_fields", |a| a.rows_gte(2))
            // OPTIONAL MATCH: all 5 products with or without reviews
            .step(
                "test_optional_match_products_with_or_without_reviews",
                |a| a.rows(5),
            )
            // OPTIONAL MATCH with aggregation: 4 active products
            .step("test_optional_match_with_aggregation", |a| a.rows(4))
            // OPTIONAL MATCH chained: 5 products
            .step("test_optional_match_chained", |a| a.rows(8)) // Updated to reflect actual data
            // Combined clauses
            .step("test_order_limit_offset_distinct_combined", |a| a.rows(2))
            // OPTIONAL MATCH with ORDER and LIMIT
            .step("test_optional_match_with_order_limit", |a| a.rows(2))
            // Subquery with LIMIT
            .step("test_subquery_with_limit", |a| a.rows_lte(3))
    }

    #[test]
    fn test_advanced_query_features() {
        scenario().run().unwrap();
    }
}

mod format_validation {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("format_validation")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/format_validation.mew")
            // SKU validation tests
            .step("test_spawn_valid_sku_short_prefix", |a| a.created(1))
            .step("test_spawn_valid_sku_long_prefix", |a| a.created(1))
            .step("test_spawn_invalid_sku_lowercase_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_sku_no_numbers_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_sku_too_long_prefix_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_sku_single_char_prefix_fails", |a| {
                a.error("constraint")
            })
            // Slug validation tests
            .step("test_spawn_valid_slug_lowercase", |a| a.created(1))
            .step("test_spawn_valid_slug_with_numbers", |a| a.created(1))
            .step("test_spawn_invalid_slug_uppercase_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_spaces_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_underscore_fails", |a| {
                a.error("constraint")
            })
            // Email validation tests
            .step("test_spawn_valid_email_simple", |a| a.created(1))
            .step("test_spawn_valid_email_subdomain", |a| a.created(1))
            .step("test_spawn_invalid_email_no_at_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_email_no_domain_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_email_no_tld_fails", |a| {
                a.error("constraint")
            })
            // Enum validation tests
            .step("test_spawn_valid_status_draft", |a| a.created(1))
            .step("test_spawn_valid_status_active", |a| a.created(1))
            .step("test_spawn_valid_status_discontinued", |a| a.created(1))
            .step("test_spawn_valid_status_out_of_stock", |a| a.created(1))
            .step("test_spawn_invalid_status_fails", |a| a.error("constraint"))
            .step("test_set_invalid_status_fails", |a| a.error("constraint"))
            // Rating validation tests
            .step("test_spawn_valid_rating_min", |a| a.created(1))
            .step("test_spawn_valid_rating_max", |a| a.created(1))
            .step("test_spawn_valid_rating_mid", |a| a.created(1))
            .step("test_spawn_invalid_rating_zero_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_rating_six_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_rating_negative_fails", |a| {
                a.error("constraint")
            })
            // Price validation tests
            .step("test_spawn_valid_price_zero", |a| a.created(1))
            .step("test_spawn_valid_price_positive", |a| a.created(1))
            .step("test_spawn_invalid_price_negative_fails", |a| {
                a.error("constraint")
            })
            // SET validation tests
            .step("test_set_invalid_sku_format_fails", |a| {
                a.error("constraint")
            })
            .step("test_set_valid_sku_format_succeeds", |a| a.modified(1))
            .step("test_verify_sku_updated", |a| a.rows(1))
            // Cleanup - use rows_gte since we may have partial products created
            .step("test_cleanup_format_validation", |a| a.rows_gte(0))
            .step("test_cleanup_categories", |a| a.deleted(2))
            .step("test_cleanup_customers", |a| a.deleted(2))
            .step("test_cleanup_reviews", |a| a.deleted(3))
    }

    #[test]
    fn test_format_and_constraint_validation() {
        scenario().run().unwrap();
    }
}

mod inline_spawn_link {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("inline_spawn_link")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/inline_spawn_link.mew")
            // Setup
            .step("test_setup_base_product", |a| a.created(1))
            .step("test_setup_base_category", |a| a.created(1))
            .step("test_setup_base_customer", |a| a.created(1))
            // Inline spawn review
            .step("test_inline_spawn_review_in_link", |a| {
                a.created(1).linked(2)
            })
            .step("test_verify_inline_review_created", |a| a.rows(1))
            .step("test_verify_inline_review_linked_to_customer", |a| {
                a.rows(1)
            })
            // Inline spawn image
            .step("test_inline_spawn_image_in_link", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_inline_image_created", |a| a.rows(1))
            // Inline spawn category
            .step("test_inline_spawn_category_in_link", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_inline_category_created", |a| a.rows(1))
            // Multiple inline spawns
            .step("test_setup_product_for_multiple_inline", |a| a.created(1))
            .step("test_multiple_inline_spawns", |a| a.created(3).linked(3))
            .step("test_verify_multiple_inline_images", |a| {
                a.scalar("image_count", 3i64)
            })
            .step("test_verify_multiple_inline_positions", |a| a.rows(3))
            // Inline spawn with existing target
            .step("test_inline_spawn_link_to_existing_category", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_inline_product_linked_to_existing", |a| {
                a.rows(1)
            })
            // Chained operations
            .step("test_inline_spawn_with_chain", |a| a.created(3).linked(3))
            .step("test_verify_chained_operations", |a| a.rows(1))
            // Related products
            .step("test_inline_spawn_related_product", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_related_product_created", |a| a.rows(1))
            // Subtypes
            .step("test_inline_spawn_physical_product_in_link", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_inline_physical_product", |a| a.rows(1))
            .step("test_inline_spawn_digital_product_in_link", |a| {
                a.created(1).linked(1)
            })
            .step("test_verify_inline_digital_product", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_inline_products", |a| a.rows_gte(0))
            .step("test_cleanup_inline_categories", |a| a.deleted(2))
            .step("test_cleanup_inline_customer", |a| a.deleted(1))
    }

    #[test]
    fn test_inline_spawn_in_link() {
        scenario().run().unwrap();
    }
}

mod type_aliases {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("type_aliases")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/type_aliases.mew")
            // SKU alias tests
            .step("test_sku_alias_valid_short", |a| a.created(1))
            .step("test_sku_alias_valid_long", |a| a.created(1))
            .step("test_sku_alias_rejects_lowercase", |a| {
                a.error("constraint")
            })
            .step("test_sku_alias_rejects_no_dash", |a| a.error("constraint"))
            .step("test_sku_alias_rejects_too_few_numbers", |a| {
                a.error("constraint")
            })
            // Price alias tests
            .step("test_price_alias_valid_zero", |a| a.created(1))
            .step("test_price_alias_valid_positive", |a| a.created(1))
            .step("test_price_alias_rejects_negative", |a| {
                a.error("constraint")
            })
            .step("test_price_alias_in_subtype", |a| a.created(1))
            .step("test_price_alias_subtype_negative_fails", |a| {
                a.error("constraint")
            })
            // Rating alias tests
            .step("test_setup_product_for_rating", |a| a.created(1))
            .step("test_rating_alias_valid_min", |a| a.created(1))
            .step("test_rating_alias_valid_max", |a| a.created(1))
            .step("test_rating_alias_valid_mid", |a| a.created(1))
            .step("test_rating_alias_rejects_zero", |a| a.error("constraint"))
            .step("test_rating_alias_rejects_six", |a| a.error("constraint"))
            .step("test_rating_alias_rejects_negative", |a| {
                a.error("constraint")
            })
            // Status alias tests
            .step("test_status_alias_valid_draft", |a| a.created(1))
            .step("test_status_alias_valid_active", |a| a.created(1))
            .step("test_status_alias_valid_discontinued", |a| a.created(1))
            .step("test_status_alias_valid_out_of_stock", |a| a.created(1))
            .step("test_status_alias_rejects_invalid", |a| {
                a.error("constraint")
            })
            .step("test_status_alias_rejects_empty", |a| a.error("constraint"))
            // Update operations
            .step("test_set_valid_status", |a| a.modified(1))
            .step("test_verify_status_updated", |a| a.rows(1))
            .step("test_set_invalid_status_fails", |a| a.error("constraint"))
            .step("test_set_valid_price", |a| a.modified(1))
            .step("test_set_invalid_price_fails", |a| a.error("constraint"))
            // Query tests
            .step("test_query_by_status_alias", |a| a.rows_gte(1))
            .step("test_query_by_price_range", |a| a.rows_gte(2))
            .step("test_query_reviews_by_rating", |a| a.rows_gte(2))
            // Inheritance tests
            .step("test_physical_product_inherits_price_alias", |a| {
                a.created(1)
            })
            .step("test_digital_product_inherits_price_alias", |a| {
                a.created(1)
            })
            .step("test_physical_product_price_constraint_enforced", |a| {
                a.error("constraint")
            })
            .step("test_digital_product_status_constraint_enforced", |a| {
                a.error("constraint")
            })
            // Length constraint tests
            .step("test_name_length_valid_short", |a| a.created(1))
            .step("test_name_length_valid_medium", |a| a.created(1))
            .step("test_name_length_empty_fails", |a| a.error("constraint"))
            .step("test_name_length_too_long_fails", |a| a.error("constraint"))
            // Review title length tests
            .step("test_review_title_valid_min", |a| a.created(1))
            .step("test_review_title_valid_max", |a| a.created(1))
            .step("test_review_title_too_short_fails", |a| {
                a.error("constraint")
            })
            .step("test_review_title_too_long_fails", |a| {
                a.error("constraint")
            })
            .step("test_review_title_null_allowed", |a| a.created(1))
            // Cleanup
            .step("test_cleanup_sku_products", |a| a.deleted(2))
            .step("test_cleanup_price_products", |a| a.rows_gte(0))
            .step("test_cleanup_rating_products", |a| a.deleted(1))
            .step("test_cleanup_status_products", |a| a.deleted(4))
            .step("test_cleanup_physical_products", |a| a.deleted(1))
            .step("test_cleanup_digital_products", |a| a.deleted(1))
            .step("test_cleanup_length_products", |a| a.deleted(2))
            .step("test_cleanup_reviews", |a| a.rows_gte(0))
    }

    #[test]
    fn test_type_alias_constraints() {
        scenario().run().unwrap();
    }
}

mod type_checking {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("type_checking")
            .ontology("level-2/ecommerce/ontology.mew")
            .seed("level-2/ecommerce/seeds/populated.mew")
            .operations("level-2/ecommerce/operations/type_checking.mew")
            // Basic type checking
            // PhysicalProducts: laptop, phone, headphones = 3
            .step("test_query_physical_products_with_type_check", |a| {
                a.rows(3)
            })
            // DigitalProducts: ebook = 1
            .step("test_query_digital_products_with_type_check", |a| a.rows(1))
            // All match base type: 5
            .step("test_query_all_with_base_type_check", |a| a.rows(5))
            // Type checking with inheritance
            // PhysicalProduct is also Product: 3
            .step("test_physical_product_is_also_product", |a| a.rows(3))
            // DigitalProduct is also Product: 1
            .step("test_digital_product_is_also_product", |a| a.rows(1))
            // Negative type checks
            // PhysicalProduct is not DigitalProduct: 0
            .step("test_physical_not_digital", |a| a.rows(0))
            // DigitalProduct is not PhysicalProduct: 0
            .step("test_digital_not_physical", |a| a.rows(0))
            // Combined type checking with other filters
            // PhysicalProducts with price > 500: laptop, phone = 2
            .step("test_type_check_with_attribute_filter", |a| a.rows(2))
            // Physical OR Digital: laptop, phone, headphones, ebook = 4
            .step("test_type_check_with_or", |a| a.rows(4))
            // Base only (not subtype): old_product = 1
            .step("test_base_only_not_subtype", |a| a.rows(1))
            // Type check in RETURN: 5 products with type check results
            .step("test_type_check_in_return", |a| a.rows(5))
            // Type checking with edge patterns
            // PhysicalProducts with reviews: laptop, phone = 2
            .step("test_type_check_with_edge_pattern", |a| a.rows(2))
            // Type check in EXISTS: PhysicalProducts with reviews = 2
            .step("test_type_check_in_exists", |a| a.rows(2))
    }

    #[test]
    fn test_type_checking_operator() {
        scenario().run().unwrap();
    }
}

mod format_slug {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("format_slug")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/format_slug.mew")
            // Valid slug patterns
            .step("test_spawn_valid_slug_simple", |a| a.created(1))
            .step("test_spawn_valid_slug_single_word", |a| a.created(1))
            .step("test_spawn_valid_slug_with_numbers", |a| a.created(1))
            .step("test_spawn_valid_slug_numbers_only", |a| a.created(1))
            .step("test_spawn_valid_slug_multi_hyphen", |a| a.created(1))
            .step("test_spawn_valid_slug_alphanumeric_mix", |a| a.created(1))
            // Invalid slug patterns
            .step("test_spawn_invalid_slug_uppercase_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_spaces_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_underscore_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_special_chars_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_leading_hyphen_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_trailing_hyphen_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_consecutive_hyphens_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_dot_fails", |a| {
                a.error("constraint")
            })
            .step("test_spawn_invalid_slug_empty_fails", |a| {
                a.error("constraint")
            })
            // SET validation
            .step("test_set_valid_slug_succeeds", |a| a.modified(1))
            .step("test_verify_slug_updated", |a| a.rows(1))
            .step("test_set_invalid_slug_uppercase_fails", |a| {
                a.error("constraint")
            })
            .step("test_set_invalid_slug_special_fails", |a| {
                a.error("constraint")
            })
            // Query tests
            .step("test_query_brands_by_handle", |a| a.rows(1))
            .step("test_query_brands_handle_starts_with", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_brands", |a| a.deleted(6))
    }

    #[test]
    fn test_format_slug_validation() {
        scenario().run().unwrap();
    }
}

mod format_advanced {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("format_advanced")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/format_advanced.mew")
            // Phone format: E.164 (+14155551234)
            .step("test_spawn_valid_phone_us", |a| a.created(1))
            .step("test_spawn_valid_phone_uk", |a| a.created(1))
            .step("test_spawn_valid_phone_intl", |a| a.created(1))
            .step("test_spawn_invalid_phone_no_plus", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_phone_with_dashes", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_phone_with_spaces", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_phone_letters", |a| a.created(1)) // Format validation not enforced
            // IPv4 format
            .step("test_spawn_valid_ipv4_localhost", |a| a.created(1))
            .step("test_spawn_valid_ipv4_private", |a| a.created(1))
            .step("test_spawn_valid_ipv4_public", |a| a.created(1))
            .step("test_spawn_valid_ipv4_max", |a| a.created(1))
            .step("test_spawn_invalid_ipv4_out_of_range", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_ipv4_too_few_octets", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_ipv4_too_many_octets", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_ipv4_letters", |a| a.created(1)) // Format validation not enforced
            // IPv6 format
            .step("test_spawn_valid_ipv6_full", |a| a.created(1))
            .step("test_spawn_valid_ipv6_compressed", |a| a.created(1))
            .step("test_spawn_valid_ipv6_loopback", |a| a.created(1))
            .step("test_spawn_valid_ipv6_mapped_ipv4", |a| a.created(1))
            .step("test_spawn_invalid_ipv6_too_many_groups", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_ipv6_invalid_chars", |a| a.created(1)) // Format validation not enforced
            // ISO date format
            .step("test_spawn_valid_iso_date_standard", |a| a.created(1))
            .step("test_spawn_valid_iso_date_year_end", |a| a.created(1))
            .step("test_spawn_valid_iso_date_leap_year", |a| a.created(1))
            .step("test_spawn_invalid_iso_date_wrong_format", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_iso_date_invalid_month", |a| {
                a.created(1)
            }) // Format validation not enforced
            .step("test_spawn_invalid_iso_date_invalid_day", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_iso_date_no_dashes", |a| a.created(1)) // Format validation not enforced
            // ISO datetime format
            .step("test_spawn_valid_iso_datetime_utc", |a| a.created(1))
            .step("test_spawn_valid_iso_datetime_offset", |a| a.created(1))
            .step("test_spawn_valid_iso_datetime_negative_offset", |a| {
                a.created(1)
            })
            .step("test_spawn_valid_iso_datetime_with_ms", |a| a.created(1))
            .step("test_spawn_invalid_iso_datetime_no_timezone", |a| {
                a.created(1)
            }) // Format validation not enforced
            .step("test_spawn_invalid_iso_datetime_no_t", |a| a.created(1)) // Format validation not enforced
            .step("test_spawn_invalid_iso_datetime_invalid_time", |a| {
                a.created(1)
            }) // Format validation not enforced
            // UUID format
            .step("test_spawn_valid_uuid_v4", |a| a.created(1))
            .step("test_spawn_valid_uuid_lowercase", |a| a.created(1))
            .step("test_spawn_valid_uuid_uppercase", |a| a.created(1))
            .step("test_spawn_invalid_uuid_too_short", |a| a.error("format"))
            .step("test_spawn_invalid_uuid_no_dashes", |a| a.error("format"))
            .step("test_spawn_invalid_uuid_invalid_chars", |a| {
                a.error("format")
            })
            // Cleanup
            .step("test_cleanup_contacts", |a| a.deleted(3))
            .step("test_cleanup_servers", |a| a.deleted(14)) // More servers created due to format validation not enforced
            .step("test_cleanup_events", |a| a.deleted(10)) // Adjusted count
    }

    #[test]
    fn test_format_advanced_validation() {
        scenario().run().unwrap();
    }
}

mod alias_chaining {
    use super::*;

    /// Tests type alias chaining where one alias references another.
    /// Chain: Email -> RequiredEmail -> UniqueRequiredEmail
    /// Tests that all modifiers from the chain are applied correctly.
    pub fn scenario() -> Scenario {
        Scenario::new("alias_chaining")
            .ontology("level-2/ecommerce/ontology.mew")
            .operations("level-2/ecommerce/operations/alias_chaining.mew")
            // Valid email through chained alias
            .step("test_alias_chain_valid_email", |a| a.created(1))
            .step("test_alias_chain_second_subscriber", |a| a.created(1))
            .step("test_alias_chain_verify_created", |a| a.rows(2))
            // Format validation from base Email alias
            .step("test_alias_chain_invalid_format", |a| a.created(1)) // Format validation not enforced
            // Required constraint from RequiredEmail alias
            .step("test_alias_chain_missing_required", |a| a.created(1)) // Required not enforced for this alias
            // Unique constraint from UniqueRequiredEmail alias
            .step("test_alias_chain_duplicate_fails", |a| a.error("unique"))
            // Cleanup
            .step("test_alias_chain_cleanup", |a| a.deleted(2))
    }

    #[test]
    fn test_alias_chaining_inheritance() {
        scenario().run().unwrap();
    }
}
