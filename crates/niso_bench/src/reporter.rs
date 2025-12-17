//! Benchmark reporting
//!
//! Gantree: L8_Benchmark → Reporter
//!
//! Provides various output formats for benchmark results.

use crate::suite::{BenchmarkResult, BenchmarkStatistics};
use serde_json;
use std::fmt::Write;

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Markdown table
    Markdown,
    /// JSON
    Json,
    /// CSV
    Csv,
    /// Plain text summary
    Text,
}

/// Benchmark reporter
/// Gantree: Reporter // 결과 리포팅
pub struct Reporter;

impl Reporter {
    // ========================================================================
    // Format Converters
    // ========================================================================

    /// Generate report in specified format
    pub fn report(results: &[BenchmarkResult], format: ReportFormat) -> String {
        match format {
            ReportFormat::Markdown => Self::to_markdown(results),
            ReportFormat::Json => Self::to_json(results),
            ReportFormat::Csv => Self::to_csv(results),
            ReportFormat::Text => Self::to_text(results),
        }
    }

    /// Convert results to Markdown table
    pub fn to_markdown(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();

        writeln!(output, "# NISO Benchmark Results\n").unwrap();

        // Statistics
        let stats = BenchmarkStatistics::from_results(results);
        writeln!(output, "## Summary\n").unwrap();
        writeln!(output, "- **Benchmarks**: {}", stats.count).unwrap();
        writeln!(
            output,
            "- **Avg Improvement**: {:.2}%",
            stats.avg_improvement_percent
        )
        .unwrap();
        writeln!(
            output,
            "- **Max Improvement**: {:.2}%",
            stats.max_improvement_percent
        )
        .unwrap();
        writeln!(
            output,
            "- **Early Stop Rate**: {:.1}%",
            stats.early_stop_rate * 100.0
        )
        .unwrap();
        writeln!(
            output,
            "- **Total Time**: {:.2}s\n",
            stats.total_time_ms as f64 / 1000.0
        )
        .unwrap();

        // Results table
        writeln!(output, "## Detailed Results\n").unwrap();
        writeln!(
            output,
            "| Name | Qubits | Noise | Baseline | Final | Improve% | Iters | Early | Time(ms) |"
        )
        .unwrap();
        writeln!(
            output,
            "|------|--------|-------|----------|-------|----------|-------|-------|----------|"
        )
        .unwrap();

        for r in results {
            writeln!(
                output,
                "| {} | {} | {:.3} | {:.4} | {:.4} | {:.2}% | {} | {} | {} |",
                r.name,
                r.qubits,
                r.noise,
                r.baseline,
                r.final_parity,
                r.improvement_percent,
                r.iterations,
                if r.early_stopped { "✓" } else { "-" },
                r.time_ms
            )
            .unwrap();
        }

        output
    }

    /// Convert results to JSON
    pub fn to_json(results: &[BenchmarkResult]) -> String {
        let stats = BenchmarkStatistics::from_results(results);

        let report = serde_json::json!({
            "statistics": stats,
            "results": results,
        });

        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Convert results to CSV
    pub fn to_csv(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "name,qubits,noise,baseline,final_parity,improvement,improvement_percent,iterations,early_stopped,time_ms,total_shots").unwrap();

        // Data
        for r in results {
            writeln!(
                output,
                "{},{},{},{},{},{},{},{},{},{},{}",
                r.name,
                r.qubits,
                r.noise,
                r.baseline,
                r.final_parity,
                r.improvement,
                r.improvement_percent,
                r.iterations,
                r.early_stopped,
                r.time_ms,
                r.total_shots
            )
            .unwrap();
        }

        output
    }

    /// Convert results to plain text summary
    pub fn to_text(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();
        let stats = BenchmarkStatistics::from_results(results);

        writeln!(output, "NISO Benchmark Results").unwrap();
        writeln!(output, "======================\n").unwrap();

        writeln!(output, "Summary:").unwrap();
        writeln!(output, "  Benchmarks run: {}", stats.count).unwrap();
        writeln!(
            output,
            "  Average improvement: {:.2}%",
            stats.avg_improvement_percent
        )
        .unwrap();
        writeln!(
            output,
            "  Best improvement: {:.2}%",
            stats.max_improvement_percent
        )
        .unwrap();
        writeln!(
            output,
            "  Worst improvement: {:.2}%",
            stats.min_improvement_percent
        )
        .unwrap();
        writeln!(
            output,
            "  Early stop rate: {:.1}%",
            stats.early_stop_rate * 100.0
        )
        .unwrap();
        writeln!(
            output,
            "  Total time: {:.2}s\n",
            stats.total_time_ms as f64 / 1000.0
        )
        .unwrap();

        writeln!(output, "Individual Results:").unwrap();
        for r in results {
            writeln!(
                output,
                "  {} ({}Q, p={:.3}): {:.2}% improvement, {} iters, {}ms{}",
                r.name,
                r.qubits,
                r.noise,
                r.improvement_percent,
                r.iterations,
                r.time_ms,
                if r.early_stopped { " [early stop]" } else { "" }
            )
            .unwrap();
        }

        output
    }

    // ========================================================================
    // Specialized Reports
    // ========================================================================

    /// Generate comparison report between two result sets
    pub fn comparison_report(
        baseline: &[BenchmarkResult],
        optimized: &[BenchmarkResult],
    ) -> String {
        let mut output = String::new();

        writeln!(output, "# NISO Comparison Report\n").unwrap();

        let baseline_stats = BenchmarkStatistics::from_results(baseline);
        let optimized_stats = BenchmarkStatistics::from_results(optimized);

        writeln!(output, "## Statistics Comparison\n").unwrap();
        writeln!(output, "| Metric | Baseline | Optimized | Change |").unwrap();
        writeln!(output, "|--------|----------|-----------|--------|").unwrap();

        writeln!(
            output,
            "| Avg Improvement | {:.2}% | {:.2}% | {:.2}% |",
            baseline_stats.avg_improvement_percent,
            optimized_stats.avg_improvement_percent,
            optimized_stats.avg_improvement_percent - baseline_stats.avg_improvement_percent
        )
        .unwrap();

        writeln!(
            output,
            "| Avg Time (ms) | {:.0} | {:.0} | {:.0} |",
            baseline_stats.avg_time_ms,
            optimized_stats.avg_time_ms,
            optimized_stats.avg_time_ms - baseline_stats.avg_time_ms
        )
        .unwrap();

        writeln!(
            output,
            "| Early Stop Rate | {:.1}% | {:.1}% | {:.1}% |",
            baseline_stats.early_stop_rate * 100.0,
            optimized_stats.early_stop_rate * 100.0,
            (optimized_stats.early_stop_rate - baseline_stats.early_stop_rate) * 100.0
        )
        .unwrap();

        output
    }

    /// Generate qubit scaling report
    pub fn qubit_scaling_report(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();

        writeln!(output, "# Qubit Scaling Analysis\n").unwrap();
        writeln!(
            output,
            "| Qubits | Improvement% | Time(ms) | Time/Qubit(ms) |"
        )
        .unwrap();
        writeln!(
            output,
            "|--------|--------------|----------|----------------|"
        )
        .unwrap();

        for r in results {
            let time_per_qubit = r.time_ms as f64 / r.qubits as f64;
            writeln!(
                output,
                "| {} | {:.2}% | {} | {:.1} |",
                r.qubits, r.improvement_percent, r.time_ms, time_per_qubit
            )
            .unwrap();
        }

        output
    }

    /// Generate noise scaling report
    pub fn noise_scaling_report(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();

        writeln!(output, "# Noise Scaling Analysis\n").unwrap();
        writeln!(
            output,
            "| Noise | Baseline | Final | Improvement% | Early Stop |"
        )
        .unwrap();
        writeln!(
            output,
            "|-------|----------|-------|--------------|------------|"
        )
        .unwrap();

        for r in results {
            writeln!(
                output,
                "| {:.3} | {:.4} | {:.4} | {:.2}% | {} |",
                r.noise,
                r.baseline,
                r.final_parity,
                r.improvement_percent,
                if r.early_stopped { "Yes" } else { "No" }
            )
            .unwrap();
        }

        output
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_results() -> Vec<BenchmarkResult> {
        vec![
            BenchmarkResult {
                name: "test1".to_string(),
                qubits: 5,
                noise: 0.02,
                baseline: 0.5,
                final_parity: 0.6,
                improvement: 0.1,
                improvement_percent: 20.0,
                iterations: 10,
                early_stopped: false,
                time_ms: 100,
                total_shots: 100000,
            },
            BenchmarkResult {
                name: "test2".to_string(),
                qubits: 7,
                noise: 0.015,
                baseline: 0.4,
                final_parity: 0.55,
                improvement: 0.15,
                improvement_percent: 37.5,
                iterations: 15,
                early_stopped: true,
                time_ms: 200,
                total_shots: 150000,
            },
        ]
    }

    #[test]
    fn test_to_markdown() {
        let results = make_test_results();
        let md = Reporter::to_markdown(&results);

        assert!(md.contains("# NISO Benchmark Results"));
        assert!(md.contains("| Name |"));
        assert!(md.contains("test1"));
        assert!(md.contains("test2"));
    }

    #[test]
    fn test_to_json() {
        let results = make_test_results();
        let json = Reporter::to_json(&results);

        assert!(json.contains("\"statistics\""));
        assert!(json.contains("\"results\""));
        assert!(json.contains("test1"));
    }

    #[test]
    fn test_to_csv() {
        let results = make_test_results();
        let csv = Reporter::to_csv(&results);

        assert!(csv.contains("name,qubits,noise"));
        assert!(csv.contains("test1,5,0.02"));
        assert!(csv.contains("test2,7,0.015"));
    }

    #[test]
    fn test_to_text() {
        let results = make_test_results();
        let text = Reporter::to_text(&results);

        assert!(text.contains("NISO Benchmark Results"));
        assert!(text.contains("Summary:"));
        assert!(text.contains("test1"));
    }

    #[test]
    fn test_report_format() {
        let results = make_test_results();

        let md = Reporter::report(&results, ReportFormat::Markdown);
        assert!(md.contains("# NISO"));

        let json = Reporter::report(&results, ReportFormat::Json);
        assert!(json.contains("{"));

        let csv = Reporter::report(&results, ReportFormat::Csv);
        assert!(csv.contains(","));
    }

    #[test]
    fn test_comparison_report() {
        let baseline = vec![make_test_results()[0].clone()];
        let optimized = vec![make_test_results()[1].clone()];

        let report = Reporter::comparison_report(&baseline, &optimized);

        assert!(report.contains("Comparison Report"));
        assert!(report.contains("Baseline"));
        assert!(report.contains("Optimized"));
    }

    #[test]
    fn test_qubit_scaling_report() {
        let results = make_test_results();
        let report = Reporter::qubit_scaling_report(&results);

        assert!(report.contains("Qubit Scaling"));
        assert!(report.contains("| 5 |"));
        assert!(report.contains("| 7 |"));
    }

    #[test]
    fn test_noise_scaling_report() {
        let results = make_test_results();
        let report = Reporter::noise_scaling_report(&results);

        assert!(report.contains("Noise Scaling"));
        assert!(report.contains("0.02"));
        assert!(report.contains("0.015"));
    }

    #[test]
    fn test_empty_results() {
        let results: Vec<BenchmarkResult> = vec![];

        let md = Reporter::to_markdown(&results);
        assert!(md.contains("Benchmarks**: 0"));

        let json = Reporter::to_json(&results);
        assert!(json.contains("\"count\": 0"));
    }
}
