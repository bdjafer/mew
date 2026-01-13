//! Level 2 - Ecommerce integration tests.
//!
//! These tests run against the ecommerce ontology with various scenarios.
//! Focus areas: Inheritance polymorphism, aggregations, uniqueness constraints,
//! ORDER BY, LIMIT/OFFSET, DISTINCT, OPTIONAL MATCH

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
            .step("test_verify_spawned_subtype_queryable_as_base", |a| a.rows(1))
            // Query as subtype
            .step("test_verify_spawned_subtype_queryable_as_subtype", |a| a.rows(1))
            // Update inherited attribute
            .step("test_set_inherited_attribute_via_subtype", |a| a.modified(1))
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
            .step("test_count_all_products", |a| a.scalar("total_products", 5i64))
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
            .step("test_products_without_reviews_count", |a| a.scalar("products_without_reviews", 3i64))
            // Products with review_count > 0: laptop, phone = 2
            .step("test_aggregation_with_having_like_filter", |a| a.rows(2))
            // Distinct customers with reviews: alice, bob = 2
            .step("test_count_distinct_customers_with_reviews", |a| a.scalar("customers_with_reviews", 2i64))
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
            .step("test_spawn_duplicate_sku_should_fail", |a| a.error("unique"))
            // Duplicate existing SKU should fail
            .step("test_spawn_duplicate_existing_sku_should_fail", |a| a.error("unique"))
            // Duplicate customer email should fail
            .step("test_spawn_duplicate_customer_email_should_fail", |a| a.error("unique"))
            // Duplicate category slug should fail
            .step("test_spawn_duplicate_category_slug_should_fail", |a| a.error("unique"))
            // SET to duplicate SKU should fail
            .step("test_set_to_duplicate_sku_should_fail", |a| a.error("unique"))
            // Verify SKU unchanged
            .step("test_verify_sku_unchanged_after_failed_set", |a| a.rows(1))
            // SET to duplicate email should fail
            .step("test_set_to_duplicate_email_should_fail", |a| a.error("unique"))
            // SET to new unique SKU should succeed
            .step("test_set_to_new_unique_sku_should_succeed", |a| a.modified(1))
            // Verify SKU changed
            .step("test_verify_sku_changed", |a| a.rows(1))
            // Old SKU now available
            .step("test_spawn_with_old_sku_now_succeeds", |a| a.created(1))
            // PhysicalProduct with duplicate SKU fails
            .step("test_physical_product_duplicate_sku_fails", |a| a.error("unique"))
            // DigitalProduct with duplicate SKU fails
            .step("test_digital_product_duplicate_sku_fails", |a| a.error("unique"))
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
            .step("test_optional_match_products_with_or_without_reviews", |a| a.rows(5))
            // OPTIONAL MATCH with aggregation: 4 active products
            .step("test_optional_match_with_aggregation", |a| a.rows(4))
            // OPTIONAL MATCH chained: 5 products
            .step("test_optional_match_chained", |a| a.rows(5))
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
