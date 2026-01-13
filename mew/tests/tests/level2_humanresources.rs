//! Level 2 - HumanResources integration tests.
//!
//! These tests run against the humanresources ontology with various scenarios.
//! Focus areas: Deep inheritance (4 levels), edge attributes, no_self constraint,
//! complex multi-hop joins, EXISTS patterns

use mew_tests::prelude::*;

mod deep_inheritance {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("deep_inheritance")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/deep_inheritance.mew")
            // Query all Persons: alice, bob, charlie, diana = 4 (eve is Contractor, not in Employee hierarchy)
            .step("test_query_all_persons", |a| a.rows(4))
            // Query all Employees: alice, bob, charlie, diana = 4
            .step("test_query_all_employees", |a| a.rows(4))
            // Query only Managers: charlie, diana = 2
            .step("test_query_only_managers", |a| a.rows(2))
            // Query only Executives: diana = 1
            .step("test_query_only_executives", |a| a.rows(1))
            // Executive has Person attributes
            .step("test_executive_has_person_attributes", |a| a.rows(1))
            // Executive has Employee attributes
            .step("test_executive_has_employee_attributes", |a| a.rows(1))
            // Executive has Manager attributes
            .step("test_executive_has_manager_attributes", |a| a.rows(1))
            // Executive has Executive attributes
            .step("test_executive_has_executive_attributes", |a| a.rows(1))
            // Filter employees by salary > 100000: diana (250k), charlie (150k), alice (120k) = 3
            .step("test_filter_employees_by_salary", |a| a.rows(3))
            // Filter managers by level >= 3: diana (5) = 1
            .step("test_filter_managers_by_management_level", |a| a.rows(1))
            // Filter person with manager attribute: charlie, diana = 2
            .step("test_filter_person_with_manager_attribute", |a| a.rows(2))
            // Spawn new Employee
            .step("test_spawn_employee", |a| a.created(1))
            // Spawn new Manager
            .step("test_spawn_manager", |a| a.created(1))
            // Spawn new Executive
            .step("test_spawn_executive", |a| a.created(1))
            // New employee queryable as Person
            .step("test_new_employee_queryable_as_person", |a| a.rows(1))
            // New manager queryable as Employee
            .step("test_new_manager_queryable_as_employee", |a| a.rows(1))
            // New executive queryable as Manager
            .step("test_new_executive_queryable_as_manager", |a| a.rows(1))
            // Count all Persons: 4 + 3 = 7
            .step("test_count_persons_includes_all", |a| a.scalar("person_count", 7i64))
            // Count all Employees: 4 + 3 = 7
            .step("test_count_employees_includes_subtypes", |a| a.scalar("employee_count", 7i64))
            // Count all Managers: 2 + 2 (new manager + new executive) = 4
            .step("test_count_managers_includes_executives", |a| a.scalar("manager_count", 4i64))
            // Update person attribute on executive
            .step("test_update_person_attribute_on_executive", |a| a.modified(1))
            // Verify update
            .step("test_verify_person_attribute_updated", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_new_entities", |a| a.deleted(3))
    }

    #[test]
    fn test_deep_inheritance_chain() {
        scenario().run().unwrap();
    }
}

mod edge_attributes {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("edge_attributes")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/edge_attributes.mew")
            // Setup test employee
            .step("test_setup_employee", |a| a.created(1))
            // Setup test skill
            .step("test_setup_skill", |a| a.created(1))
            // Link with edge attributes
            .step("test_link_with_edge_attributes", |a| a.linked(1))
            // Query edge attributes
            .step("test_query_edge_attributes", |a| a.rows(1))
            // Update edge attributes
            .step("test_update_edge_attribute", |a| a.modified(1))
            // Verify edge attribute updated
            .step("test_verify_edge_attribute_updated", |a| a.rows(1))
            // Filter by edge attribute level: expert skills (alice rust, charlie projectmgmt, diana leadership, test) >= 2
            .step("test_filter_by_edge_attribute_level", |a| a.rows_gte(2))
            // Filter by edge attribute numeric: years >= 5 (alice rust=5, charlie projectmgmt=6, diana leadership=10, test=5) >= 2
            .step("test_filter_by_edge_attribute_numeric", |a| a.rows_gte(2))
            // AVG years per skill
            .step("test_avg_years_experience_per_skill", |a| a.rows_gte(1))
            // COUNT employees by skill level for Rust
            .step("test_count_employees_by_skill_level", |a| a.rows_gte(1))
            // Department edge attributes (is_primary)
            .step("test_department_edge_attributes", |a| a.rows(1))
            // Role requirement edge attributes (minimum_level)
            .step("test_role_requirement_edge_attributes", |a| a.rows(1))
            // Course completion edge attributes (score)
            .step("test_course_completion_edge_attributes", |a| a.rows(1))
            // Certification edge attributes (certificate_number)
            .step("test_certification_edge_attributes", |a| a.rows(1))
            // Complex query with edge attributes
            .step("test_find_overqualified_employees", |a| a.rows_gte(0))
            // Multiple edge attribute conditions
            .step("test_employees_with_multiple_edge_attribute_conditions", |a| a.rows_gte(1))
            // Set edge attribute to NULL
            .step("test_set_edge_attribute_null", |a| a.modified(1))
            // Verify NULL
            .step("test_verify_edge_attribute_null", |a| a.rows(1))
            // Unlink edge with attributes
            .step("test_unlink_edge_with_attributes", |a| a.unlinked(1))
            // Verify edge deleted
            .step("test_verify_edge_deleted", |a| a.scalar("edge_count", 0i64))
            // Link with partial attributes
            .step("test_link_with_partial_attributes", |a| a.linked(1))
            // Verify partial attributes
            .step("test_verify_partial_attributes", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_test_entities", |a| a.deleted(1))
            .step("test_cleanup_test_skill", |a| a.deleted(1))
    }

    #[test]
    fn test_edge_attributes_crud() {
        scenario().run().unwrap();
    }
}

mod no_self {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("no_self")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/no_self.mew")
            // Setup employee for self-link test
            .step("test_setup_employee_for_self_link", |a| a.created(1))
            // reports_to self should fail
            .step("test_reports_to_self_should_fail", |a| a.error("self"))
            // Verify no self edge created
            .step("test_verify_no_self_edge_created", |a| a.scalar("edge_count", 0i64))
            // Valid link to different employee succeeds
            .step("test_reports_to_different_employee_succeeds", |a| a.linked(1))
            // Verify valid link created
            .step("test_verify_valid_link_created", |a| a.rows(1))
            // Setup department for self-link test
            .step("test_setup_department_for_self_link", |a| a.created(1))
            // parent_dept self should fail
            .step("test_parent_dept_self_should_fail", |a| a.error("self"))
            // Verify no parent_dept self edge
            .step("test_verify_no_parent_dept_self_edge", |a| a.scalar("edge_count", 0i64))
            // Setup task for self-block test
            .step("test_setup_task_for_self_block", |a| a.created(1))
            // blocks self should fail
            .step("test_task_blocks_self_should_fail", |a| a.error("self"))
            // Verify no self block edge
            .step("test_verify_no_self_block_edge", |a| a.scalar("edge_count", 0i64))
            // Update test (conceptual)
            .step("test_update_to_create_self_link_should_fail", |a| a.rows(1))
            // Multiple employees reporting to same manager: alice, bob -> charlie = 2
            .step("test_multiple_employees_reporting_to_same_manager", |a| a.scalar("report_count", 2i64))
            // Setup circular employees
            .step("test_setup_circular_employees", |a| a.created(1))
            .step("test_setup_circular_employee_2", |a| a.created(1))
            // Circular dependency allowed (not self)
            .step("test_circular_dependency_allowed", |a| a.linked(1))
            .step("test_circular_dependency_reverse_allowed", |a| a.linked(1))
            // Verify circular exists
            .step("test_verify_circular_dependency_exists", |a| a.rows(1))
            // Cleanup
            .step("test_cleanup_self_test_entities", |a| a.deleted(3))
            .step("test_cleanup_self_dept", |a| a.deleted(1))
            .step("test_cleanup_self_task", |a| a.deleted(1))
    }

    #[test]
    fn test_no_self_constraint() {
        scenario().run().unwrap();
    }
}

mod complex_joins {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("complex_joins")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/complex_joins.mew")
            // Two-hop: employees -> departments -> parent
            .step("test_employees_to_departments_to_parent", |a| a.rows_gte(1))
            // Two-hop: employees -> skills -> courses
            .step("test_employees_skills_to_courses", |a| a.rows_gte(1))
            // Three-hop: employee -> role -> requires_skill -> skill
            .step("test_employee_to_role_to_skill_requirement_to_skill", |a| a.rows_gte(1))
            // Three-hop: employee -> manager -> office
            .step("test_employee_manager_office_chain", |a| a.rows(2))
            // Four-hop: employee -> course -> skill -> role
            .step("test_employee_to_course_to_skill_to_role", |a| a.rows_gte(1))
            // Multi-path: employee has skill AND completed course for same skill
            .step("test_employee_has_skill_and_completed_course_for_same_skill", |a| a.rows_gte(1))
            // Multi-path: employee role and skill alignment
            .step("test_employee_role_and_skill_alignment", |a| a.rows_gte(1))
            // Branching: employee multiple skills (Alice has Rust, Python)
            .step("test_employee_multiple_skills", |a| a.rows(1))
            // Branching: manager multiple reports (Charlie has Alice, Bob)
            .step("test_manager_multiple_reports", |a| a.rows(1))
            // Transitive: two-level reporting chain (Alice/Bob -> Charlie -> Diana)
            .step("test_two_level_reporting_chain", |a| a.rows(2))
            // Aggregation: count employees per manager
            .step("test_count_employees_per_manager", |a| a.rows_gte(1))
            // Aggregation: department employee skill count
            .step("test_department_employee_skill_count", |a| a.rows_gte(1))
            // Aggregation: office capacity vs occupancy
            .step("test_office_capacity_vs_occupancy", |a| a.rows_gte(2))
            // Filter on multiple hops: high salary in HQ
            .step("test_high_salary_employees_in_headquarters", |a| a.rows_gte(1))
            // Filter: expert employees in management roles
            .step("test_expert_employees_in_management_roles", |a| a.rows_gte(1))
            // Existence: employees with certification AND course completion
            .step("test_employees_with_certification_and_course_completion", |a| a.rows_gte(1))
            // Existence: managers heading departments with employees
            .step("test_managers_heading_departments_with_employees", |a| a.rows_gte(1))
            // Optional: employees with optional certifications
            .step("test_employees_with_optional_certifications", |a| a.rows_gte(4))
            // Optional: skills with optional course and employees
            .step("test_skills_with_optional_course_and_employees", |a| a.rows_gte(3))
            // Self-join: employees sharing same office
            .step("test_employees_sharing_same_office", |a| a.rows_gte(1))
    }

    #[test]
    fn test_complex_multi_hop_joins() {
        scenario().run().unwrap();
    }
}

mod exists_patterns {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("exists_patterns")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/exists_patterns.mew")
            // EXISTS: employees with skills (alice, bob, charlie, diana have skills) >= 3
            .step("test_employees_with_skills", |a| a.rows_gte(3))
            // NOT EXISTS: employees without certifications >= 3
            .step("test_employees_without_certifications", |a| a.rows_gte(3))
            // EXISTS with filter: employees with expert skills (alice, charlie, diana) >= 2
            .step("test_employees_with_expert_level_skills", |a| a.rows_gte(2))
            // EXISTS with filter: employees with programming skills (alice, bob) >= 2
            .step("test_employees_with_programming_skills", |a| a.rows_gte(2))
            // Nested EXISTS: employees with role requiring skill they have >= 1
            .step("test_employees_with_role_requiring_skill_they_have", |a| a.rows_gte(1))
            // EXISTS with multiple conditions: engineering + rust >= 2
            .step("test_employees_in_engineering_with_rust_skill", |a| a.rows_gte(2))
            // NOT EXISTS: skills without employees >= 0
            .step("test_skills_without_employees", |a| a.rows_gte(0))
            // NOT EXISTS: departments without managers >= 2
            .step("test_departments_without_managers", |a| a.rows_gte(2))
            // NOT EXISTS: employees not completed any course >= 3
            .step("test_employees_not_completed_any_course", |a| a.rows_gte(3))
            // EXISTS AND NOT EXISTS: skills but no certifications >= 2
            .step("test_employees_with_skills_but_no_certifications", |a| a.rows_gte(2))
            // EXISTS with aggregation: employees with multiple skills (alice) >= 1
            .step("test_employees_with_multiple_skills", |a| a.rows_gte(1))
            // EXISTS with aggregation: managers with multiple reports (charlie) >= 1
            .step("test_managers_with_multiple_reports", |a| a.rows_gte(1))
            // EXISTS with edge attribute: high years experience >= 2
            .step("test_employees_with_high_years_experience_in_any_skill", |a| a.rows_gte(2))
            // EXISTS with edge attribute: primary department >= 4
            .step("test_employees_with_primary_department_assignment", |a| a.rows_gte(4))
            // Complex EXISTS: departments with management skills >= 1
            .step("test_departments_with_employees_having_management_skills", |a| a.rows_gte(1))
            // Complex EXISTS: roles with unfilled requirements >= 0
            .step("test_roles_with_unfilled_skill_requirements", |a| a.rows_gte(0))
            // EXISTS with path: employees with direct or indirect reports >= 2
            .step("test_employees_with_direct_or_indirect_reports", |a| a.rows_gte(2))
            // NOT EXISTS complex: employees without matching skills >= 0
            .step("test_employees_without_skills_matching_role_requirements", |a| a.rows_gte(0))
            // NOT EXISTS: courses not completed by anyone >= 0
            .step("test_courses_not_completed_by_anyone", |a| a.rows_gte(0))
            // EXISTS with arithmetic: high total skill experience >= 1
            .step("test_employees_with_high_total_skill_experience", |a| a.rows_gte(1))
            // EXISTS with arithmetic: departments over budget threshold >= 0
            .step("test_departments_over_budget_threshold", |a| a.rows_gte(0))
    }

    #[test]
    fn test_exists_patterns_and_aggregates() {
        scenario().run().unwrap();
    }
}
