//! Level 1 - Expressions integration tests.
//!
//! These tests run against the expressions ontology to test:
//! - Data types (Float, Duration, Timestamp)
//! - String functions
//! - Arithmetic operations
//! - Null handling
//! - Advanced aggregations (COLLECT, COUNT DISTINCT)
//! - RETURNING clause variants
//! - Transaction control
//! - Debug statements (EXPLAIN, PROFILE)

use mew_tests::prelude::*;

mod data_types {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("data_types")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/types.mew")
            // Float type tests
            .step("spawn_float_values", |a| a.created(5))
            .step("query_float_values", |a| a.scalar("count", 5i64))
            .step("query_float_comparison_greater", |a| {
                a.scalar("count", 3i64)
            })
            .step("query_float_comparison_less", |a| a.scalar("count", 1i64))
            .step("query_float_equality", |a| a.rows(1))
            .step("query_float_range", |a| a.scalar("count", 4i64))
            // Duration type tests
            .step("spawn_duration_values", |a| a.created(5))
            .step("query_duration_records", |a| a.scalar("count", 5i64))
            // Timestamp literal tests
            .step("spawn_timestamp_literals", |a| a.created(3))
            .step("query_timestamp_comparison", |a| a.scalar("count", 8i64))
            .step("query_timestamp_range", |a| a.scalar("count", 3i64))
            // Mixed type tests
            .step("spawn_mixed_types", |a| a.created(1))
            .step("query_mixed_type_filters", |a| a.rows(1))
    }

    #[test]
    fn test_float_duration_timestamp_types() {
        scenario().run().unwrap();
    }
}

mod string_functions {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("string_functions")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/strings.mew")
            // Seed data
            .step("seed_string_data", |a| a.created(5))
            // length() tests
            .step("query_length_function", |a| a.scalar("len", 11i64))
            .step("query_length_filter", |a| a.scalar("count", 4i64))
            .step("query_length_comparison", |a| a.rows(1))
            // starts_with() tests
            .step("query_starts_with_match", |a| a.scalar("count", 1i64))
            .step("query_starts_with_no_match", |a| a.scalar("count", 0i64))
            .step("query_starts_with_case_sensitive", |a| {
                a.scalar("count", 1i64)
            })
            // ends_with() tests
            .step("query_ends_with_match", |a| a.scalar("count", 1i64))
            .step("query_ends_with_extension", |a| a.rows(1))
            .step("query_ends_with_no_match", |a| a.scalar("count", 0i64))
            // contains() tests
            .step("query_contains_match", |a| a.scalar("count", 1i64))
            .step("query_contains_substring", |a| a.rows(1))
            .step("query_contains_at_symbol", |a| a.scalar("count", 1i64))
            // lower()/upper() tests
            .step("query_lower_function", |a| a.rows(3))
            .step("query_upper_function", |a| a.rows(3))
            .step("query_lower_comparison", |a| a.scalar("count", 1i64))
            // trim() tests
            .step("query_trim_function", |a| a.rows(3))
            .step("query_trim_length", |a| a.scalar("len", 7i64))
            // substring() tests
            .step("query_substring_basic", |a| {
                a.first(row_str! { "sub" => "Lorem" })
            })
            .step("query_substring_middle", |a| {
                a.first(row_str! { "sub" => "World" })
            })
            // String concatenation (++) tests
            .step("query_concat_basic", |a| {
                a.first(row_str! { "concatenated" => "hello world" })
            })
            .step("query_concat_multiple", |a| a.rows(1))
            .step("query_concat_attributes", |a| a.rows(1))
    }

    #[test]
    fn test_string_functions_and_concatenation() {
        scenario().run().unwrap();
    }
}

mod arithmetic {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("arithmetic")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/arithmetic.mew")
            // Seed data
            .step("seed_arithmetic_data", |a| a.created(5))
            // Addition tests
            .step("query_addition_basic", |a| a.scalar("sum", 13.0))
            .step("query_addition_negative", |a| a.scalar("sum", -3.0))
            .step("query_addition_with_literal", |a| a.scalar("sum", 15.0))
            // Subtraction tests
            .step("query_subtraction_basic", |a| a.scalar("diff", 7.0))
            .step("query_subtraction_negative_result", |a| {
                a.scalar("diff", -7.0)
            })
            // Multiplication tests
            .step("query_multiplication_basic", |a| a.scalar("product", 30.0))
            .step("query_multiplication_decimals", |a| a.rows(1))
            .step("query_multiplication_large", |a| {
                a.scalar("product", 1000.0)
            })
            // Division tests
            .step("query_division_basic", |a| a.rows(1))
            .step("query_division_decimals", |a| a.rows(1))
            // Modulo tests
            .step("query_modulo_basic", |a| a.scalar("remainder", 2i64))
            .step("query_modulo_negative", |a| a.rows(1))
            // Unary minus tests
            .step("query_unary_minus_positive", |a| a.scalar("negated", -10.0))
            .step("query_unary_minus_negative", |a| a.scalar("negated", 5.0))
            .step("query_unary_minus_in_expression", |a| {
                a.scalar("result", 7.0)
            })
            // abs() tests
            .step("query_abs_positive", |a| a.scalar("absolute", 10.0))
            .step("query_abs_negative", |a| a.scalar("absolute", 5.0))
            // min/max function tests
            .step("query_min_function", |a| a.scalar("minimum", 3.0))
            .step("query_max_function", |a| a.scalar("maximum", 10.0))
            .step("query_min_with_negative", |a| a.scalar("minimum", -5.0))
            // floor/ceil/round tests
            .step("query_floor_function", |a| a.scalar("floored", 3.0))
            .step("query_ceil_function", |a| a.scalar("ceiled", 4.0))
            .step("query_round_function", |a| a.scalar("rounded", 3.0))
            // Operator precedence tests
            .step("query_precedence_mul_over_add", |a| {
                a.scalar("result", 16.0)
            })
            .step("query_precedence_parentheses", |a| a.scalar("result", 26.0))
            .step("query_precedence_complex", |a| a.rows(1))
            // Arithmetic in WHERE clause
            .step("query_arithmetic_in_where", |a| a.rows(3))
            .step("query_computed_comparison", |a| a.scalar("count", 3i64))
    }

    #[test]
    fn test_arithmetic_operations_and_precedence() {
        scenario().run().unwrap();
    }
}

mod null_handling {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("null_handling")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/nulls.mew")
            // Seed data
            .step("seed_null_data", |a| a.created(6))
            // Null comparison tests
            .step("query_null_equals_null", |a| a.scalar("count", 4i64))
            .step("query_null_not_equals_null", |a| a.scalar("count", 2i64))
            .step("query_null_check_field", |a| a.rows(1))
            // is_null() tests
            .step("query_is_null_true", |a| a.scalar("count", 4i64))
            .step("query_is_null_false", |a| a.scalar("count", 2i64))
            .step("query_is_null_multiple", |a| a.scalar("count", 4i64))
            // coalesce() tests
            .step("query_coalesce_basic", |a| {
                a.first(row_str! { "result" => "fallback" })
            })
            .step("query_coalesce_not_null", |a| {
                a.first(row_str! { "result" => "has_value" })
            })
            .step("query_coalesce_chain", |a| {
                a.first(row_str! { "result" => "first" })
            })
            .step("query_coalesce_chain_middle", |a| {
                a.first(row_str! { "result" => "second" })
            })
            .step("query_coalesce_chain_last", |a| {
                a.first(row_str! { "result" => "final_fallback" })
            })
            // ?? operator tests
            .step("query_null_coalesce_op_basic", |a| {
                a.first(row_str! { "result" => "default" })
            })
            .step("query_null_coalesce_op_not_null", |a| {
                a.first(row_str! { "result" => "has_value" })
            })
            .step("query_null_coalesce_op_chain", |a| {
                a.first(row_str! { "result" => "final_fallback" })
            })
            // Null propagation tests
            .step("query_null_propagation_arithmetic", |a| a.rows(1))
            .step("query_null_propagation_comparison", |a| a.rows(1))
            .step("query_null_propagation_string", |a| a.rows(1))
            // Null in boolean context
            .step("query_null_and_true", |a| a.rows(1))
            .step("query_null_or_true", |a| a.rows(1))
            .step("query_not_null_bool", |a| a.rows(1))
            // Conditional with null
            .step("query_where_nullable_true", |a| a.scalar("count", 1i64))
            .step("query_where_nullable_false", |a| a.scalar("count", 0i64))
            .step("query_where_nullable_is_null", |a| a.scalar("count", 5i64))
    }

    #[test]
    fn test_null_semantics_and_coalescing() {
        scenario().run().unwrap();
    }
}

mod timestamps {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("timestamps")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/timestamps.mew")
            // Seed data (t1-t5 with fixed timestamps)
            .step("seed_timestamp_data", |a| a.created(5))
            // year() tests (deterministic: only t1-t5 exist)
            .step("query_year_extraction", |a| a.scalar("yr", 2024i64))
            .step("query_year_filter", |a| a.scalar("count", 4i64))
            .step("query_year_comparison", |a| a.rows(1))
            // month() tests (deterministic: only t1-t5 exist)
            .step("query_month_extraction", |a| a.scalar("mo", 6i64))
            .step("query_month_filter", |a| a.scalar("count", 2i64))
            .step("query_month_range", |a| a.scalar("count", 3i64))
            // day() tests (deterministic: only t1-t5 exist)
            .step("query_day_extraction", |a| a.scalar("dy", 31i64))
            .step("query_day_filter", |a| a.scalar("count", 2i64))
            .step("query_day_mid_month", |a| a.rows(1))
            // hour/minute/second tests (deterministic: only t1-t5 exist)
            .step("query_hour_extraction", |a| a.scalar("hr", 12i64))
            .step("query_minute_extraction", |a| a.scalar("min", 30i64))
            .step("query_second_extraction", |a| a.scalar("sec", 45i64))
            .step("query_time_filter", |a| a.scalar("count", 4i64))
            // now() tests (spawns t6 with runtime timestamp)
            .step("spawn_with_now", |a| a.created(1))
            .step("query_now_comparison", |a| a.scalar("count", 5i64))
            // Timestamp arithmetic tests (uses fixed timestamps by label)
            .step("query_timestamp_plus_duration", |a| a.rows(1))
            .step("query_timestamp_minus_duration", |a| a.rows(1))
            .step("query_timestamp_diff", |a| a.rows(1))
            // Combined tests (t6 now exists)
            .step("query_date_parts_combined", |a| a.rows(1))
            .step("query_timestamp_ordering", |a| a.rows(6))
            .step("query_recent_records", |a| a.scalar("count", 4i64))
    }

    #[test]
    fn test_timestamp_functions_and_arithmetic() {
        scenario().run().unwrap();
    }
}

mod aggregations_advanced {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("aggregations_advanced")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/aggregations.mew")
            // Seed data
            .step("seed_aggregation_data", |a| a.created(8))
            // COLLECT tests
            .step("query_collect_basic", |a| a.rows(1))
            .step("query_collect_by_category", |a| a.rows(1))
            .step("query_collect_with_limit", |a| a.rows(1))
            .step("query_collect_float_values", |a| a.rows(1))
            .step("query_collect_grouped", |a| a.rows(3))
            // COUNT DISTINCT tests
            .step("query_count_distinct_category", |a| {
                a.scalar("unique_categories", 3i64)
            })
            .step("query_count_distinct_int", |a| {
                a.scalar("unique_ints", 4i64)
            })
            .step("query_count_distinct_vs_count", |a| a.rows(1))
            // SUM/AVG with Float tests
            .step("query_sum_float", |a| a.rows(1))
            .step("query_sum_float_by_category", |a| a.rows(3))
            .step("query_avg_float", |a| a.rows(1))
            .step("query_avg_float_by_category", |a| a.rows(3))
            // MIN/MAX with String tests
            .step("query_min_string", |a| {
                a.first(row_str! { "first_label" => "apple" })
            })
            .step("query_max_string", |a| {
                a.first(row_str! { "last_label" => "cherry" })
            })
            .step("query_min_max_string_by_category", |a| a.rows(3))
            // MIN/MAX with Float tests
            .step("query_min_float", |a| a.scalar("minimum", 5.25))
            .step("query_max_float", |a| a.scalar("maximum", 100.0))
            .step("query_min_max_range", |a| a.rows(1))
            // Empty set aggregation tests
            .step("query_count_empty", |a| a.scalar("count", 0i64))
            .step("query_sum_empty", |a| a.rows(1))
            .step("query_avg_empty", |a| a.rows(1))
            .step("query_min_empty", |a| a.rows(1))
            .step("query_max_empty", |a| a.rows(1))
            .step("query_collect_empty", |a| a.rows(1))
            // Combined tests
            .step("query_all_aggregations", |a| a.rows(1))
            .step("query_aggregations_with_grouping", |a| a.rows(3))
    }

    #[test]
    fn test_collect_count_distinct_and_advanced_aggregations() {
        scenario().run().unwrap();
    }
}

mod returning_clause {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("returning_clause")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/returning.mew")
            // SPAWN RETURNING tests
            .step("spawn_returning_id", |a| a.created(1))
            .step("spawn_returning_star", |a| a.created(1))
            .step("spawn_returning_specific_attrs", |a| a.created(1))
            .step("spawn_returning_with_default", |a| a.created(1))
            // LINK RETURNING tests
            .step("setup_for_link_returning", |a| a.created(2))
            .step("link_returning_id", |a| a.linked(1))
            .step("setup_for_link_star", |a| a.created(2))
            .step("link_returning_star", |a| a.linked(1))
            // SET RETURNING tests
            .step("setup_for_set_returning", |a| a.created(1))
            .step("set_returning_single", |a| a.modified(1))
            .step("set_returning_multiple", |a| a.modified(1))
            .step("set_returning_star", |a| a.modified(1))
            // KILL RETURNING tests
            .step("setup_for_kill_returning", |a| a.created(3))
            .step("kill_returning_id", |a| a.deleted(1))
            .step("kill_bulk_returning", |a| a.deleted(2))
            // Verification
            // 4 ret_* + 4 link_* + 1 set_* = 9 records
            .step("verify_spawned_records", |a| a.scalar("count", 9i64))
            .step("verify_links_created", |a| a.scalar("count", 2i64))
    }

    #[test]
    fn test_returning_clause_variants() {
        scenario().run().unwrap();
    }
}

mod transactions {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("transactions")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/transactions.mew")
            // Auto-commit tests
            .step("auto_commit_spawn", |a| a.created(2))
            .step("verify_auto_commit", |a| a.scalar("count", 2i64))
            // Explicit commit tests
            .step("explicit_commit_transaction", |a| a.created(2).modified(2))
            .step("verify_commit", |a| a.scalar("count", 2i64))
            // Explicit rollback tests
            .step("explicit_rollback_transaction", |a| a.created(0))
            .step("verify_rollback", |a| a.scalar("count", 0i64))
            // Multi-operation transaction tests
            .step("multi_op_transaction", |a| {
                a.created(4).linked(1).modified(2)
            })
            .step("verify_multi_op", |a| a.rows(1))
            // Failure rollback tests
            .step("setup_for_failure_test", |a| a.created(1))
            .step("transaction_failure_rollback", |a| a.error("not found"))
            .step("verify_failure_rollback", |a| a.rows(1))
            // Nested operations tests
            .step("nested_spawn_link_transaction", |a| a.created(4).linked(1))
            .step("verify_nested_transaction", |a| a.scalar("txn_count", 2i64))
            .step("verify_nested_links", |a| a.scalar("link_count", 1i64))
            // Isolation level tests
            .step("read_committed_transaction", |a| a.created(1))
            .step("verify_read_committed", |a| a.scalar("r.value", 42i64))
            .step("serializable_transaction", |a| a.created(1).modified(1))
            .step("verify_serializable", |a| a.rows(1))
            // Final count: 12 TxnRecord spawns - 1 rolled back (r5) = 11
            .step("final_txn_record_count", |a| a.scalar("total", 11i64))
    }

    #[test]
    fn test_transaction_control() {
        scenario().run().unwrap();
    }
}

mod debug_statements {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("debug_statements")
            .ontology("level-1/expressions/ontology.mew")
            .operations("level-1/expressions/operations/debug.mew")
            // Seed data
            .step("seed_debug_data", |a| a.created(5).linked(2))
            // EXPLAIN tests - should return query plan information
            .step("explain_simple_match", |a| a.rows(1))
            .step("explain_with_where", |a| a.rows(1))
            .step("explain_with_join", |a| a.rows(1))
            .step("explain_with_aggregation", |a| a.rows(1))
            .step("explain_with_order_limit", |a| a.rows(1))
            // PROFILE tests - should return execution statistics
            .step("profile_simple_match", |a| a.rows(3))
            .step("profile_with_where", |a| a.rows(2))
            .step("profile_with_join", |a| a.rows(2))
            .step("profile_with_aggregation", |a| a.rows(1))
            .step("profile_with_order_limit", |a| a.rows(2))
            // Edge cases
            .step("explain_empty_result", |a| a.rows(1))
            .step("profile_empty_result", |a| a.rows(0))
            .step("explain_complex_query", |a| a.rows(1))
    }

    #[test]
    fn test_explain_and_profile() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-1/expressions/ontology.mew")
            .seed("level-1/expressions/seeds/populated.mew")
            .operations("level-1/expressions/operations/queries.mew")
            .step("count_all_measurements", |a| a.scalar("count", 3i64))
            .step("count_all_text_entries", |a| a.scalar("count", 2i64))
            .step("count_all_time_records", |a| a.scalar("count", 2i64))
            .step("count_all_calculations", |a| a.scalar("count", 1i64))
            .step("count_all_nullable_records", |a| a.scalar("count", 2i64))
            .step("count_all_data_points", |a| a.scalar("count", 3i64))
            .step("query_measurements_by_validity", |a| a.rows(3))
            .step("query_data_points_by_category", |a| a.rows(3))
            .step("query_time_records_ordered", |a| a.rows(2))
    }

    #[test]
    fn test_query_with_populated_seed() {
        scenario().run().unwrap();
    }
}
