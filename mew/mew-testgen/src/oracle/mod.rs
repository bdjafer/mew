//! Independent oracle for result verification

use crate::types::*;

/// Oracle provides independent verification of results
pub struct Oracle;

impl Oracle {
    /// Verify actual result against expected
    pub fn verify(expected: &Expected, actual: &ActualResult) -> VerifyResult {
        match (expected, actual) {
            (Expected::Rows(expected_rows), ActualResult::Rows(actual_rows)) => {
                Self::verify_rows(expected_rows, actual_rows)
            }
            (Expected::Count(expected_count), ActualResult::Count(actual_count)) => {
                if expected_count == actual_count {
                    VerifyResult::Pass
                } else {
                    VerifyResult::Fail(format!(
                        "Expected count {}, got {}",
                        expected_count, actual_count
                    ))
                }
            }
            (Expected::Count(expected_count), ActualResult::Rows(rows)) => {
                if *expected_count == rows.len() {
                    VerifyResult::Pass
                } else {
                    VerifyResult::Fail(format!(
                        "Expected {} rows, got {}",
                        expected_count, rows.len()
                    ))
                }
            }
            (Expected::Success(_effect), ActualResult::Success) => {
                VerifyResult::Pass
            }
            (Expected::Error(pattern), ActualResult::Error(msg)) => {
                // Check if error message matches pattern
                // Supports: simple contains, or patterns with .* wildcards
                let patterns: Vec<&str> = pattern.split('|').collect();
                let matches = patterns.iter().any(|p| {
                    Self::pattern_matches(p, msg)
                });
                if matches {
                    VerifyResult::Pass
                } else {
                    VerifyResult::Fail(format!(
                        "Error pattern '{}' not found in '{}'",
                        pattern, msg
                    ))
                }
            }
            (Expected::Property(prop), actual) => {
                Self::verify_property(prop, actual)
            }
            (expected, actual) => {
                VerifyResult::Fail(format!(
                    "Type mismatch: expected {:?}, got {:?}",
                    std::mem::discriminant(expected),
                    std::mem::discriminant(actual)
                ))
            }
        }
    }

    /// Check if a pattern matches a message
    /// Supports simple contains and .* wildcard patterns
    fn pattern_matches(pattern: &str, msg: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        let msg_lower = msg.to_lowercase();

        if pattern_lower.contains(".*") {
            // Handle .* as "anything in between"
            // Split by .* and check all parts exist in order
            let parts: Vec<&str> = pattern_lower.split(".*").collect();
            let mut pos = 0;
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if let Some(found) = msg_lower[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            // Simple contains check
            msg_lower.contains(&pattern_lower)
        }
    }

    fn verify_rows(expected: &[Row], actual: &[Row]) -> VerifyResult {
        if expected.len() != actual.len() {
            return VerifyResult::Fail(format!(
                "Row count mismatch: expected {}, got {}",
                expected.len(), actual.len()
            ));
        }

        // For now, just check unordered equality
        let mut expected_sorted: Vec<_> = expected.iter().collect();
        let mut actual_sorted: Vec<_> = actual.iter().collect();

        expected_sorted.sort_by_key(|r| format!("{:?}", r));
        actual_sorted.sort_by_key(|r| format!("{:?}", r));

        for (i, (exp, act)) in expected_sorted.iter().zip(actual_sorted.iter()).enumerate() {
            if exp != act {
                return VerifyResult::Fail(format!(
                    "Row {} mismatch: expected {:?}, got {:?}",
                    i, exp, act
                ));
            }
        }

        VerifyResult::Pass
    }

    fn verify_property(prop: &PropertySpec, actual: &ActualResult) -> VerifyResult {
        match prop {
            PropertySpec::CountInRange { min, max } => {
                let count = match actual {
                    ActualResult::Count(c) => *c,
                    ActualResult::Rows(rows) => rows.len(),
                    _ => return VerifyResult::Fail("Cannot check count on non-row result".to_string()),
                };
                if count >= *min && count <= *max {
                    VerifyResult::Pass
                } else {
                    VerifyResult::Fail(format!(
                        "Count {} not in range {}..{}",
                        count, min, max
                    ))
                }
            }
            PropertySpec::AllMatch { column: _, pattern: _ } => {
                // Would require parsing actual values and matching
                VerifyResult::Skip("AllMatch not fully implemented".to_string())
            }
            PropertySpec::Idempotent => {
                // Idempotence is verified by the test executor, not here
                VerifyResult::Skip("Idempotent verified by executor".to_string())
            }
            PropertySpec::Commutative { other: _ } => {
                // Commutativity is verified by the test executor, not here
                VerifyResult::Skip("Commutative verified by executor".to_string())
            }
        }
    }
}

/// Result of verification
#[derive(Debug, Clone)]
pub enum VerifyResult {
    Pass,
    Fail(String),
    Skip(String),
}

impl VerifyResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, VerifyResult::Pass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_count() {
        let expected = Expected::Count(5);
        let actual = ActualResult::Count(5);
        assert!(Oracle::verify(&expected, &actual).is_pass());

        let actual_wrong = ActualResult::Count(3);
        assert!(!Oracle::verify(&expected, &actual_wrong).is_pass());
    }

    #[test]
    fn test_verify_rows() {
        let expected = Expected::Rows(vec![
            Row { columns: vec![Value::Int(1)] },
            Row { columns: vec![Value::Int(2)] },
        ]);
        let actual = ActualResult::Rows(vec![
            Row { columns: vec![Value::Int(2)] },
            Row { columns: vec![Value::Int(1)] },
        ]);
        // Unordered match
        assert!(Oracle::verify(&expected, &actual).is_pass());
    }

    #[test]
    fn test_verify_error() {
        let expected = Expected::Error("required".to_string());
        let actual = ActualResult::Error("Field 'name' is required".to_string());
        assert!(Oracle::verify(&expected, &actual).is_pass());

        let wrong = ActualResult::Error("Unknown error".to_string());
        assert!(!Oracle::verify(&expected, &wrong).is_pass());
    }
}
