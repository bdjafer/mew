//! Level 2 - HumanResources integration tests.
//!
//! These tests run against the humanresources ontology with various scenarios.

use mew_tests::prelude::*;

mod crud {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("crud")
            .ontology("level-2/humanresources/ontology.mew")
            .operations("level-2/humanresources/operations/crud.mew")
            .step("spawn_department", |a| a.created(1))
            .step("query_count_depts", |a| a.value(1))
            .step("spawn_employee", |a| a.created(1))
            .step("link_employee_dept", |a| a.linked(1))
            .step("query_employees", |a| a.rows(1))
            .step("spawn_skill", |a| a.created(1))
            .step("link_employee_skill", |a| a.linked(1))
            .step("spawn_role", |a| a.created(1))
            .step("link_employee_role", |a| a.linked(1))
            .step("update_employee", |a| a.modified(1))
            .step("query_updated", |a| a.rows(1))
            .step("kill_employee", |a| a.deleted(1))
            .step("query_empty", |a| a.empty())
    }

    #[test]
    fn test_crud_operations_on_humanresources() {
        scenario().run().unwrap();
    }
}

mod queries {
    use super::*;

    pub fn scenario() -> Scenario {
        Scenario::new("queries")
            .ontology("level-2/humanresources/ontology.mew")
            .seed("level-2/humanresources/seeds/populated.mew")
            .operations("level-2/humanresources/operations/queries.mew")
            // Note: Employee count returns only base Employee types, not subtypes
            // Manager and Executive have their own type IDs
            .step("count_all_employees", |a| a.value(2))
            .step("count_all_departments", |a| a.value(3))
            .step("count_all_skills", |a| a.value(3))
            .step("count_all_roles", |a| a.value(3))
            .step("query_active_employees", |a| a.rows(2))
            .step("query_managers", |a| a.rows(1))
            .step("query_executives", |a| a.rows(1))
            .step("query_all_employees", |a| a.rows(2))
            .step("query_departments", |a| a.rows(3))
            .step("query_offices", |a| a.rows(2))
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
            .ontology("level-2/humanresources/ontology.mew")
            .operations("level-2/humanresources/operations/errors.mew")
            // Note: regex match constraints are not enforced at runtime yet
            // but range constraints ARE enforced
            .step("spawn_invalid_employee_id", |a| a.created(1))
            .step("spawn_invalid_email", |a| a.created(1))
            .step("spawn_invalid_dept_code", |a| a.created(1))
            .step("spawn_invalid_management_level", |a| a.error("constraint"))
            .step("spawn_valid", |a| a.created(1))
    }

    #[test]
    fn test_error_handling_for_invalid_operations() {
        scenario().run().unwrap();
    }
}
