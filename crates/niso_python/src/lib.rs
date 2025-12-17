#![allow(non_local_definitions)]
//! # NISO Python Bindings
//!
//! Python bindings for the NISO (NISQ Integrated System Optimizer) library.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_python // L9: Python Bindings
//!     PyNisoConfig // 설정 바인딩
//!         default_5q(), default_7q()
//!         with_noise(), with_points(), with_shots()
//!     PyNisoOptimizer // 최적화기 바인딩
//!         optimize() -> PyTqqcResult
//!         optimize_full() -> PyOptimizationResult
//!         measure_parity(theta, delta)
//!     PyTqqcResult // TQQC 결과 바인딩
//!         delta_opt, improvement_percent
//!         to_json(), to_dict()
//!     PyOptimizationResult // 통합 결과 바인딩
//!         tqqc_result, schedule_metrics
//!         summary()
//!     PyBenchSuite // 벤치마크 바인딩
//!         bench_tqqc(), noise_scaling()
//!         to_json(), to_csv(), to_markdown()
//!     PyCircuitGenerator // 회로 생성기 바인딩
//!         ghz(), qft(), hea(), random()
//! ```
//!
//! ## Quick Start (Python)
//!
//! ```python
//! import niso
//!
//! # Simple optimization
//! result = niso.quick_optimize(qubits=7, noise=0.02, seed=42)
//! print(f"Improvement: {result.improvement_percent:.2f}%")
//!
//! # Full optimization with config
//! config = niso.NisoConfig.default_7q().with_noise(0.02).with_seed(42)
//! optimizer = niso.NisoOptimizer(config)
//! result = optimizer.optimize_full()
//! print(result.summary())
//!
//! # Benchmarking
//! suite = niso.BenchSuite()
//! suite.noise_scaling(7, [0.01, 0.02, 0.03], 10)
//! print(suite.to_markdown())
//! ```
//!
//! ## Installation
//!
//! ```bash
//! pip install maturin
//! cd niso/crates/niso_python
//! maturin develop --release
//! ```

use pyo3::prelude::*;

// ============================================================================
// Module Declarations
// ============================================================================

/// Configuration bindings
pub mod config;

/// Result bindings
pub mod result;

/// Optimizer bindings
pub mod optimizer;

/// Benchmark bindings
pub mod bench;

// ============================================================================
// Re-exports
// ============================================================================

pub use bench::{PyBenchSuite, PyBenchmarkResult, PyCircuitGenerator, PyStatistics};
pub use config::{PyHardwareTarget, PyNisoConfig, PyOptimizationMode};
pub use optimizer::PyNisoOptimizer;
pub use result::{PyIterationRecord, PyOptimizationResult, PyTqqcResult};

// ============================================================================
// Python Module
// ============================================================================

/// NISO - NISQ Integrated System Optimizer
///
/// A high-performance quantum optimization library implementing TQQC (Time-Quantized
/// Quantum Computing) algorithms for NISQ devices.
///
/// ## Features
///
/// - **TQQC Optimization**: Dynamic inner loop, adaptive z-test, early stopping
/// - **Multiple Hardware Targets**: IBM, trapped ion, neutral atom
/// - **Comprehensive Benchmarking**: Noise/qubit scaling, statistics
/// - **Circuit Generation**: GHZ, QFT, HEA, random circuits
///
/// ## Example
///
/// ```python
/// import niso
///
/// # Quick optimization
/// result = niso.quick_optimize(qubits=7, noise=0.02)
/// print(f"Improvement: {result.improvement_percent:.2f}%")
///
/// # Benchmarking
/// suite = niso.BenchSuite()
/// results = suite.noise_scaling(7, [0.01, 0.02, 0.03], 10)
/// for r in results:
///     print(f"{r.name}: {r.improvement_percent:.2f}%")
/// ```
#[pymodule]
fn niso(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // ========================================================================
    // Configuration Classes
    // ========================================================================

    m.add_class::<PyNisoConfig>()?;
    m.add_class::<PyOptimizationMode>()?;
    m.add_class::<PyHardwareTarget>()?;

    // ========================================================================
    // Result Classes
    // ========================================================================

    m.add_class::<PyTqqcResult>()?;
    m.add_class::<PyOptimizationResult>()?;
    m.add_class::<PyIterationRecord>()?;

    // ========================================================================
    // Optimizer Classes
    // ========================================================================

    m.add_class::<PyNisoOptimizer>()?;

    // ========================================================================
    // Benchmark Classes
    // ========================================================================

    m.add_class::<PyBenchSuite>()?;
    m.add_class::<PyBenchmarkResult>()?;
    m.add_class::<PyStatistics>()?;
    m.add_class::<PyCircuitGenerator>()?;

    // ========================================================================
    // Convenience Functions
    // ========================================================================

    m.add_function(wrap_pyfunction!(optimizer::quick_optimize, m)?)?;
    m.add_function(wrap_pyfunction!(optimizer::full_optimize, m)?)?;
    m.add_function(wrap_pyfunction!(bench::noise_scaling_benchmark, m)?)?;
    m.add_function(wrap_pyfunction!(bench::qubit_scaling_benchmark, m)?)?;

    // ========================================================================
    // Module Metadata
    // ========================================================================

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "Jung Wook Yang <sadpig70@gmail.com>")?;
    m.add("__doc__", "NISO - NISQ Integrated System Optimizer")?;

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = PyNisoConfig::default_7q();
        assert_eq!(config.qubits(), 7);
    }

    #[test]
    fn test_config_builder() {
        let config = PyNisoConfig::default_7q()
            .with_noise(0.02)
            .with_points(20)
            .with_shots(4096)
            .with_seed(42);

        assert_eq!(config.qubits(), 7);
        assert!((config.noise() - 0.02).abs() < 1e-10);
        assert_eq!(config.points(), 20);
        assert_eq!(config.shots(), 4096);
        assert_eq!(config.seed(), Some(42));
    }

    #[test]
    fn test_optimizer_creation() {
        let config = PyNisoConfig::default_5q().with_noise(0.02);
        let optimizer = PyNisoOptimizer::new(config);
        assert!(optimizer.is_ok());
    }

    #[test]
    fn test_optimization() {
        let config = PyNisoConfig::default_5q()
            .with_noise(0.02)
            .with_points(3)
            .with_seed(42);

        let optimizer = PyNisoOptimizer::new(config).unwrap();
        let result = optimizer.optimize();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.iterations() > 0);
        assert!(result.parity_baseline() >= -1.0 && result.parity_baseline() <= 1.0);
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = PyBenchSuite::new(Some(42));
        let result = suite.bench_tqqc("test", 5, 0.02, 3);

        assert_eq!(result.qubits(), 5);
        assert_eq!(suite.len(), 1);
    }

    #[test]
    fn test_circuit_generator() {
        let gen = PyCircuitGenerator::new(Some(42));

        let ghz = gen.ghz(5);
        assert!(ghz.is_ok());

        let qft = gen.qft(3);
        assert!(qft.is_ok());
    }

    #[test]
    fn test_noise_scaling() {
        let mut suite = PyBenchSuite::new(Some(42));
        let results = suite.noise_scaling(5, vec![0.01, 0.02], 3);

        assert_eq!(results.len(), 2);

        let stats = suite.statistics();
        assert_eq!(stats.count(), 2);
    }

    #[test]
    fn test_result_json() {
        let config = PyNisoConfig::default_5q()
            .with_noise(0.02)
            .with_points(3)
            .with_seed(42);

        let optimizer = PyNisoOptimizer::new(config).unwrap();
        let result = optimizer.optimize().unwrap();

        let json = result.to_json();
        assert!(json.is_ok());
        assert!(json.unwrap().contains("delta_opt"));
    }

    #[test]
    fn test_quick_mode() {
        let config = PyNisoConfig::quick(5).with_seed(42);
        let optimizer = PyNisoOptimizer::new(config).unwrap();
        let result = optimizer.optimize();

        assert!(result.is_ok());
    }

    #[test]
    fn test_benchmark_export() {
        let mut suite = PyBenchSuite::new(Some(42));
        suite.bench_tqqc("test", 5, 0.02, 3);

        let json = suite.to_json();
        assert!(json.is_ok());

        let csv = suite.to_csv();
        assert!(csv.contains("name"));

        let md = suite.to_markdown();
        assert!(md.contains("|"));
    }
}
