//! Trust auditing for test results

use crate::types::*;
use std::collections::HashMap;

/// Audits test results for trust violations
pub struct TrustAuditor;

impl TrustAuditor {
    /// Audit test results and report any trust issues
    pub fn audit(results: &[TestResult]) -> TrustReport {
        let mut report = TrustReport::new();

        for result in results {
            Self::audit_result(result, &mut report);
        }

        report.compute_metrics();
        report
    }

    fn audit_result(result: &TestResult, report: &mut TrustReport) {
        // Track by trust level
        report.by_trust_level
            .entry(result.trust_level)
            .or_insert_with(Vec::new)
            .push(result.clone());

        // Flag potential issues
        match result.trust_level {
            TrustLevel::Axiomatic => {
                // Axiomatic tests should NEVER fail
                if !result.passed {
                    report.violations.push(TrustViolation {
                        test_id: result.test_id.clone(),
                        level: result.trust_level,
                        message: "Axiomatic test failed - indicates implementation bug or test generation error".to_string(),
                        severity: Severity::Critical,
                    });
                }
            }
            TrustLevel::Constructive => {
                // Constructive tests should rarely fail
                if !result.passed {
                    report.violations.push(TrustViolation {
                        test_id: result.test_id.clone(),
                        level: result.trust_level,
                        message: "Constructive test failed - expected result was built into generation".to_string(),
                        severity: Severity::High,
                    });
                }
            }
            TrustLevel::Predicted => {
                // Predicted tests may fail due to oracle mismatch
                if !result.passed {
                    report.warnings.push(TrustWarning {
                        test_id: result.test_id.clone(),
                        message: "Predicted test failed - may indicate oracle disagreement".to_string(),
                    });
                }
            }
            TrustLevel::Statistical => {
                // Statistical tests have expected failure rates
                // Track for aggregate analysis
            }
        }
    }
}

/// Report of trust audit findings
#[derive(Debug, Clone)]
pub struct TrustReport {
    pub by_trust_level: HashMap<TrustLevel, Vec<TestResult>>,
    pub violations: Vec<TrustViolation>,
    pub warnings: Vec<TrustWarning>,
    pub metrics: TrustMetrics,
}

impl TrustReport {
    pub fn new() -> Self {
        Self {
            by_trust_level: HashMap::new(),
            violations: Vec::new(),
            warnings: Vec::new(),
            metrics: TrustMetrics::default(),
        }
    }

    fn compute_metrics(&mut self) {
        // Compute pass rates by trust level
        for (level, results) in &self.by_trust_level {
            let total = results.len();
            let passed = results.iter().filter(|r| r.passed).count();
            let rate = if total > 0 { passed as f64 / total as f64 } else { 1.0 };

            match level {
                TrustLevel::Axiomatic => self.metrics.axiomatic_pass_rate = rate,
                TrustLevel::Constructive => self.metrics.constructive_pass_rate = rate,
                TrustLevel::Predicted => self.metrics.predicted_pass_rate = rate,
                TrustLevel::Statistical => self.metrics.statistical_pass_rate = rate,
            }
        }

        // Compute trust score (weighted average)
        self.metrics.overall_trust_score =
            self.metrics.axiomatic_pass_rate * 0.4 +
            self.metrics.constructive_pass_rate * 0.35 +
            self.metrics.predicted_pass_rate * 0.2 +
            self.metrics.statistical_pass_rate * 0.05;
    }

    /// Is the implementation trustworthy based on results?
    pub fn is_trustworthy(&self) -> bool {
        // Critical: All axiomatic tests must pass
        if self.metrics.axiomatic_pass_rate < 1.0 {
            return false;
        }
        // High: Most constructive tests should pass
        if self.metrics.constructive_pass_rate < 0.95 {
            return false;
        }
        // Overall trust score should be high
        self.metrics.overall_trust_score >= 0.9
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== Trust Audit Report ===".to_string());
        lines.push(format!("Overall Trust Score: {:.1}%", self.metrics.overall_trust_score * 100.0));
        lines.push(String::new());

        lines.push("Pass Rates by Trust Level:".to_string());
        lines.push(format!("  Axiomatic:    {:.1}%", self.metrics.axiomatic_pass_rate * 100.0));
        lines.push(format!("  Constructive: {:.1}%", self.metrics.constructive_pass_rate * 100.0));
        lines.push(format!("  Predicted:    {:.1}%", self.metrics.predicted_pass_rate * 100.0));
        lines.push(format!("  Statistical:  {:.1}%", self.metrics.statistical_pass_rate * 100.0));

        if !self.violations.is_empty() {
            lines.push(String::new());
            lines.push(format!("VIOLATIONS ({}):", self.violations.len()));
            for v in &self.violations {
                lines.push(format!("  [{:?}] {} - {}", v.severity, v.test_id, v.message));
            }
        }

        if !self.warnings.is_empty() {
            lines.push(String::new());
            lines.push(format!("Warnings ({}):", self.warnings.len()));
            for w in &self.warnings {
                lines.push(format!("  {} - {}", w.test_id, w.message));
            }
        }

        lines.push(String::new());
        lines.push(format!("Trustworthy: {}", if self.is_trustworthy() { "YES" } else { "NO" }));

        lines.join("\n")
    }
}

impl Default for TrustReport {
    fn default() -> Self {
        Self::new()
    }
}

/// A trust violation (serious issue)
#[derive(Debug, Clone)]
pub struct TrustViolation {
    pub test_id: String,
    pub level: TrustLevel,
    pub message: String,
    pub severity: Severity,
}

/// A trust warning (potential issue)
#[derive(Debug, Clone)]
pub struct TrustWarning {
    pub test_id: String,
    pub message: String,
}

/// Severity of a violation
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// Computed trust metrics
#[derive(Debug, Clone)]
pub struct TrustMetrics {
    pub axiomatic_pass_rate: f64,
    pub constructive_pass_rate: f64,
    pub predicted_pass_rate: f64,
    pub statistical_pass_rate: f64,
    pub overall_trust_score: f64,
}

impl Default for TrustMetrics {
    fn default() -> Self {
        // Default to 1.0 (no failures) for missing trust levels
        Self {
            axiomatic_pass_rate: 1.0,
            constructive_pass_rate: 1.0,
            predicted_pass_rate: 1.0,
            statistical_pass_rate: 1.0,
            overall_trust_score: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_passing() {
        let results = vec![
            TestResult {
                test_id: "t1".to_string(),
                passed: true,
                expected: Expected::Count(1),
                actual: ActualResult::Count(1),
                duration_us: 100,
                trust_level: TrustLevel::Axiomatic,
            },
            TestResult {
                test_id: "t2".to_string(),
                passed: true,
                expected: Expected::Count(2),
                actual: ActualResult::Count(2),
                duration_us: 100,
                trust_level: TrustLevel::Constructive,
            },
        ];

        let report = TrustAuditor::audit(&results);

        assert!(report.violations.is_empty());
        assert!(report.is_trustworthy());
        assert_eq!(report.metrics.axiomatic_pass_rate, 1.0);
    }

    #[test]
    fn test_audit_axiomatic_failure() {
        let results = vec![
            TestResult {
                test_id: "t1".to_string(),
                passed: false,
                expected: Expected::Count(0),
                actual: ActualResult::Count(1),
                duration_us: 100,
                trust_level: TrustLevel::Axiomatic,
            },
        ];

        let report = TrustAuditor::audit(&results);

        assert_eq!(report.violations.len(), 1);
        assert!(!report.is_trustworthy());
    }
}
