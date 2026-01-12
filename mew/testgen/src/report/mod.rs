//! Test report generation

use crate::trust::TrustReport;
use crate::types::*;

/// Generates human-readable test reports
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a full report from test results
    pub fn generate(
        suite: &TestSuite,
        summary: &TestSummary,
        trust_report: &TrustReport,
    ) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push("╔══════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║               MEW Generative Test Report                     ║".to_string());
        lines.push("╚══════════════════════════════════════════════════════════════╝".to_string());
        lines.push(String::new());

        // Configuration
        lines.push(format!("Seed: {}", suite.seed));
        lines.push(format!("Test Cases: {}", suite.test_cases.len()));
        lines.push(format!(
            "World State: {} nodes, {} edges",
            suite.world.nodes.len(),
            suite.world.edges.len()
        ));
        lines.push(String::new());

        // Schema Summary
        lines.push("Schema:".to_string());
        for (name, info) in &suite.schema.node_types {
            lines.push(format!("  node {} ({} attrs)", name, info.attrs.len()));
        }
        for (name, info) in &suite.schema.edge_types {
            lines.push(format!(
                "  edge {}({:?})",
                name,
                info.params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect::<Vec<_>>()
            ));
        }
        lines.push(String::new());

        // Results Summary
        lines.push("═══════════════════════════════════════════════════════════════".to_string());
        lines.push("                         RESULTS                               ".to_string());
        lines.push("═══════════════════════════════════════════════════════════════".to_string());
        lines.push(String::new());

        let pass_rate = if summary.total > 0 {
            (summary.passed as f64 / summary.total as f64) * 100.0
        } else {
            0.0
        };

        lines.push(format!("Total:  {} tests", summary.total));
        lines.push(format!("Passed: {} ({:.1}%)", summary.passed, pass_rate));
        lines.push(format!("Failed: {}", summary.failed));
        lines.push(format!(
            "Time:   {:.2}ms",
            summary.total_duration_us as f64 / 1000.0
        ));
        lines.push(String::new());

        // By Trust Level
        lines.push("By Trust Level:".to_string());
        for level in [
            TrustLevel::Axiomatic,
            TrustLevel::Constructive,
            TrustLevel::Predicted,
            TrustLevel::Statistical,
        ] {
            if let Some(&(passed, total)) = summary.by_trust_level.get(&level) {
                let rate = if total > 0 {
                    (passed as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                lines.push(format!(
                    "  {:12} {}/{} ({:.1}%)",
                    level.as_str(),
                    passed,
                    total,
                    rate
                ));
            }
        }
        lines.push(String::new());

        // Trust Report
        lines.push("═══════════════════════════════════════════════════════════════".to_string());
        lines.push("                      TRUST ANALYSIS                           ".to_string());
        lines.push("═══════════════════════════════════════════════════════════════".to_string());
        lines.push(String::new());
        lines.push(trust_report.summary());
        lines.push(String::new());

        // Failed Tests Detail
        if summary.failed > 0 {
            lines.push(
                "═══════════════════════════════════════════════════════════════".to_string(),
            );
            lines.push(
                "                      FAILED TESTS                             ".to_string(),
            );
            lines.push(
                "═══════════════════════════════════════════════════════════════".to_string(),
            );
            lines.push(String::new());

            // Note: We'd need access to actual results here
            // For now, just indicate count
            lines.push(format!(
                "({} failed tests - see detailed log)",
                summary.failed
            ));
            lines.push(String::new());
        }

        // Footer
        lines.push("═══════════════════════════════════════════════════════════════".to_string());
        if trust_report.is_trustworthy() {
            lines.push("✓ Implementation is TRUSTWORTHY".to_string());
        } else {
            lines.push("✗ Implementation has TRUST ISSUES".to_string());
        }
        lines.push("═══════════════════════════════════════════════════════════════".to_string());

        lines.join("\n")
    }

    /// Generate JSON report for programmatic consumption
    pub fn generate_json(
        suite: &TestSuite,
        summary: &TestSummary,
        results: &[TestResult],
    ) -> String {
        use serde_json::json;

        let failures: Vec<_> = results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| {
                json!({
                    "id": r.test_id,
                    "trust_level": r.trust_level.as_str(),
                    "expected": format!("{:?}", r.expected),
                    "actual": format!("{:?}", r.actual),
                    "duration_us": r.duration_us
                })
            })
            .collect();

        let report = json!({
            "seed": suite.seed,
            "total": summary.total,
            "passed": summary.passed,
            "failed": summary.failed,
            "pass_rate": if summary.total > 0 {
                summary.passed as f64 / summary.total as f64
            } else {
                0.0
            },
            "total_duration_us": summary.total_duration_us,
            "failures": failures,
            "schema": {
                "node_types": suite.schema.node_types.keys().collect::<Vec<_>>(),
                "edge_types": suite.schema.edge_types.keys().collect::<Vec<_>>(),
            },
            "world": {
                "nodes": suite.world.nodes.len(),
                "edges": suite.world.edges.len(),
            }
        });

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_report() {
        let suite = TestSuite {
            ontology_source: "node Test {}".to_string(),
            schema: AnalyzedSchema {
                node_types: {
                    let mut m = HashMap::new();
                    m.insert(
                        "Test".to_string(),
                        NodeTypeInfo {
                            name: "Test".to_string(),
                            attrs: Vec::new(),
                            parents: Vec::new(),
                            applicable_constraints: Vec::new(),
                        },
                    );
                    m
                },
                edge_types: HashMap::new(),
                type_aliases: HashMap::new(),
                constraints: Vec::new(),
                rules: Vec::new(),
            },
            world: WorldState::new(),
            test_cases: Vec::new(),
            seed: 42,
        };

        let summary = TestSummary {
            total: 10,
            passed: 8,
            failed: 2,
            by_trust_level: HashMap::new(),
            by_complexity: HashMap::new(),
            by_tag: HashMap::new(),
            total_duration_us: 1000,
        };

        let trust_report = crate::trust::TrustReport::new();

        let report = ReportGenerator::generate(&suite, &summary, &trust_report);
        assert!(report.contains("MEW Generative Test Report"));
        assert!(report.contains("10 tests"));
    }
}
