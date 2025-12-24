//! # NISO Bench
//!
//! Benchmarking suite for quantum optimization.
//!
//! ## Gantree Architecture
//!
//! ```text
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_bench::prelude::*;
//!
//! // Run quick benchmark
//! let mut suite = BenchSuite::with_seed(42);
//! let results = suite.run_quick();
//!
//! // Generate report
//! let report = Reporter::to_markdown(&results);
//! println!("{}", report);
//! ```
//!
//! ## Circuit Generation
//!
//! ```rust
//! use niso_bench::prelude::*;
//!
//! let gen = CircuitGenerator::with_seed(42);
//!
//! // Standard circuits
//! let ghz = gen.ghz(5);
//! let bell = gen.bell();
//! let qft = gen.qft(4);
//!
//! // TQQC circuits
//! let parity = gen.tqqc_parity(7, 0.5, 0.0);
//!
//! // Parameterized circuits
//! let hea = gen.hea(5, 3);
//! let random = gen.random(5, 5);
//! ```
//!
//! ## Full Benchmark Suite
//!
//! ```rust
//! use niso_bench::prelude::*;
//!
//! let mut suite = BenchSuite::with_seed(42).verbose();
//!
//! // Run scaling benchmarks
//! let qubit_results = suite.run_qubit_scaling(7, 0.02, 5);
//! let noise_results = suite.run_noise_scaling(5, &[0.01, 0.02], 5);
//!
//! // Get statistics
//! let stats = suite.statistics();
//! println!("Average improvement: {:.2}%", stats.avg_improvement_percent);
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Circuit generators (Gantree: L8_Benchmark ??Generators)
pub mod generators;

/// Benchmark suite (Gantree: L8_Benchmark ??BenchSuite)
pub mod suite;

/// Reporting (Gantree: L8_Benchmark ??Reporter)
pub mod reporter;

// ============================================================================
// Re-exports
// ============================================================================

pub use generators::CircuitGenerator;
pub use reporter::{ReportFormat, Reporter};
pub use suite::{BenchSuite, BenchmarkResult, BenchmarkStatistics};

// ============================================================================
// Prelude
// ============================================================================

// Convenient imports below
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_bench::prelude::*;
    //! ```

    pub use crate::generators::CircuitGenerator;
    pub use crate::reporter::{ReportFormat, Reporter};
    pub use crate::suite::{BenchSuite, BenchmarkResult, BenchmarkStatistics};
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_circuit_generator() {
        let gen = CircuitGenerator::with_seed(42);

        let ghz = gen.ghz(5);
        assert_eq!(ghz.num_qubits(), 5);

        let bell = gen.bell();
        assert_eq!(bell.num_qubits(), 2);

        let parity = gen.tqqc_parity(7, 0.0, 0.0);
        assert_eq!(parity.num_qubits(), 7);
    }

    #[test]
    fn test_bench_suite_quick() {
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_quick();

        assert_eq!(results.len(), 2);

        for r in &results {
            assert!(r.iterations > 0);
            assert!(r.time_ms > 0);
        }
    }

    #[test]
    fn test_reporter_formats() {
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_quick();

        let md = Reporter::report(&results, ReportFormat::Markdown);
        assert!(md.contains("# NISO"));

        let json = Reporter::report(&results, ReportFormat::Json);
        assert!(json.contains("{"));

        let csv = Reporter::report(&results, ReportFormat::Csv);
        assert!(csv.contains("name,qubits"));
    }

    #[test]
    fn test_full_workflow() {
        // Generate circuits
        let gen = CircuitGenerator::with_seed(42);
        let circuits = gen.parity_oscillation(5, 5);
        assert_eq!(circuits.len(), 5);

        // Run benchmarks
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_quick();

        // Get statistics
        let stats = suite.statistics();
        assert_eq!(stats.count, 2);

        // Generate report
        let report = Reporter::to_markdown(&results);
        assert!(!report.is_empty());
    }

    #[test]
    fn test_qubit_scaling_benchmark() {
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_qubit_scaling(5, 0.02, 3);

        // Should have results for 3, 4, 5 qubits
        assert_eq!(results.len(), 3);

        let report = Reporter::qubit_scaling_report(&results);
        assert!(report.contains("Qubit Scaling"));
    }

    #[test]
    fn test_noise_scaling_benchmark() {
        let mut suite = BenchSuite::with_seed(42);
        let noise_levels = vec![0.01, 0.02];
        let results = suite.run_noise_scaling(5, &noise_levels, 3);

        assert_eq!(results.len(), 2);

        let report = Reporter::noise_scaling_report(&results);
        assert!(report.contains("Noise Scaling"));
    }

    #[test]
    fn test_statistics() {
        let mut suite = BenchSuite::with_seed(42);
        suite.run_quick();

        let stats = suite.statistics();

        assert_eq!(stats.count, 2);
        assert!(stats.total_time_ms > 0);
        assert!(stats.avg_time_ms > 0.0);
    }

    #[test]
    fn test_comparison_report() {
        let mut suite1 = BenchSuite::with_seed(42);
        let results1 = suite1.run_quick();

        let mut suite2 = BenchSuite::with_seed(43);
        let results2 = suite2.run_quick();

        let report = Reporter::comparison_report(&results1, &results2);
        assert!(report.contains("Comparison"));
    }

    #[test]
    fn test_random_circuit_reproducibility() {
        let gen1 = CircuitGenerator::with_seed(42);
        let gen2 = CircuitGenerator::with_seed(42);

        let c1 = gen1.random(5, 3);
        let c2 = gen2.random(5, 3);

        assert_eq!(c1.gates().len(), c2.gates().len());
    }

    #[test]
    fn test_depth_scaling_circuits() {
        let gen = CircuitGenerator::with_seed(42);
        let circuits = gen.depth_scaling(5, 5);

        assert_eq!(circuits.len(), 5);

        // Depths should generally increase
        for (i, c) in circuits.iter().enumerate() {
            assert!(c.depth() >= i + 1, "Circuit {} has depth {}", i, c.depth());
        }
    }

    #[test]
    fn test_hea_circuit() {
        let gen = CircuitGenerator::with_seed(42);
        let hea = gen.hea(5, 3);

        assert_eq!(hea.num_qubits(), 5);
        assert!(hea.count_1q() > 0);
        assert!(hea.count_2q() > 0);
    }
}
