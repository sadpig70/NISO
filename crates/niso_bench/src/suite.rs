//! Benchmark suite for NISO
//!
//! Gantree: L8_Benchmark → BenchSuite
//!
//! Provides comprehensive benchmarking for NISO optimization.

use niso_backend::SimulatorBackend;
use niso_engine::{NisoConfig, NisoOptimizer, OptimizationResult};
use niso_tqqc::{TqqcConfig, TqqcEngine, TqqcResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,

    /// Number of qubits
    pub qubits: usize,

    /// Noise level
    pub noise: f64,

    /// Baseline parity
    pub baseline: f64,

    /// Final parity
    pub final_parity: f64,

    /// Improvement (final - baseline)
    pub improvement: f64,

    /// Improvement percentage
    pub improvement_percent: f64,

    /// Number of iterations
    pub iterations: usize,

    /// Early stopped
    pub early_stopped: bool,

    /// Execution time (milliseconds)
    pub time_ms: u64,

    /// Total shots used
    pub total_shots: u64,
}

impl BenchmarkResult {
    /// Create from TQQC result
    pub fn from_tqqc(
        name: &str,
        qubits: usize,
        noise: f64,
        result: &TqqcResult,
        time_ms: u64,
        shots_per_run: u64,
    ) -> Self {
        let total_shots = (result.total_inner_iterations * 2 + 1) as u64 * shots_per_run;

        Self {
            name: name.to_string(),
            qubits,
            noise,
            baseline: result.parity_baseline,
            final_parity: result.parity_final,
            improvement: result.improvement,
            improvement_percent: result.improvement_percent(),
            iterations: result.iterations,
            early_stopped: result.early_stopped,
            time_ms,
            total_shots,
        }
    }

    /// Create from optimization result
    pub fn from_optimization(name: &str, config: &NisoConfig, result: &OptimizationResult) -> Self {
        Self {
            name: name.to_string(),
            qubits: config.qubits,
            noise: config.noise,
            baseline: result.baseline_parity(),
            final_parity: result.final_parity(),
            improvement: result.tqqc_result.improvement,
            improvement_percent: result.improvement_percent(),
            iterations: result.tqqc_result.iterations,
            early_stopped: result.tqqc_result.early_stopped,
            time_ms: result.metrics.total_time_ms,
            total_shots: result.metrics.total_shots,
        }
    }
}

/// Benchmark suite
/// Gantree: BenchSuite // 벤치마크 스위트
pub struct BenchSuite {
    /// Base seed for reproducibility
    seed: u64,

    /// Results
    results: Vec<BenchmarkResult>,

    /// Verbose output
    verbose: bool,
}

impl BenchSuite {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new benchmark suite
    pub fn new() -> Self {
        Self {
            seed: 42,
            results: Vec::new(),
            verbose: false,
        }
    }

    /// Create with seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            results: Vec::new(),
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    // ========================================================================
    // Individual Benchmarks
    // ========================================================================

    /// Benchmark single TQQC run
    pub fn bench_tqqc(
        &mut self,
        name: &str,
        qubits: usize,
        noise: f64,
        points: usize,
    ) -> BenchmarkResult {
        if self.verbose {
            println!(
                "Running benchmark: {} ({}Q, noise={:.3})",
                name, qubits, noise
            );
        }

        let config = TqqcConfig::for_qubits(qubits)
            .with_noise(noise)
            .with_points(points)
            .with_dynamic_inner(true)
            .with_seed(self.seed);

        let backend = SimulatorBackend::from_depol(qubits, noise)
            .unwrap()
            .with_seed(self.seed);

        let mut engine = TqqcEngine::new(config, backend);

        let start = Instant::now();
        let result = engine.optimize().unwrap();
        let time_ms = start.elapsed().as_millis() as u64;

        let bench_result = BenchmarkResult::from_tqqc(name, qubits, noise, &result, time_ms, 8192);
        self.results.push(bench_result.clone());

        bench_result
    }

    /// Benchmark using NisoOptimizer
    pub fn bench_niso(&mut self, name: &str, config: NisoConfig) -> BenchmarkResult {
        if self.verbose {
            println!(
                "Running benchmark: {} ({}Q, noise={:.3})",
                name, config.qubits, config.noise
            );
        }

        let config = config.with_seed(self.seed);
        let mut optimizer = NisoOptimizer::new(config.clone());

        let result = optimizer.optimize().unwrap();

        let bench_result = BenchmarkResult::from_optimization(name, &config, &result);
        self.results.push(bench_result.clone());

        bench_result
    }

    // ========================================================================
    // Benchmark Suites
    // ========================================================================

    /// Run qubit scaling benchmark
    pub fn run_qubit_scaling(
        &mut self,
        max_qubits: usize,
        noise: f64,
        points: usize,
    ) -> Vec<BenchmarkResult> {
        if self.verbose {
            println!("=== Qubit Scaling Benchmark ===");
        }

        let mut results = Vec::new();

        for n in 3..=max_qubits {
            let name = format!("qubit_scaling_{}q", n);
            let result = self.bench_tqqc(&name, n, noise, points);
            results.push(result);
        }

        results
    }

    /// Run noise scaling benchmark
    pub fn run_noise_scaling(
        &mut self,
        qubits: usize,
        noise_levels: &[f64],
        points: usize,
    ) -> Vec<BenchmarkResult> {
        if self.verbose {
            println!("=== Noise Scaling Benchmark ===");
        }

        let mut results = Vec::new();

        for &noise in noise_levels {
            let name = format!("noise_scaling_p{:.3}", noise);
            let result = self.bench_tqqc(&name, qubits, noise, points);
            results.push(result);
        }

        results
    }

    /// Run points scaling benchmark
    pub fn run_points_scaling(
        &mut self,
        qubits: usize,
        noise: f64,
        point_counts: &[usize],
    ) -> Vec<BenchmarkResult> {
        if self.verbose {
            println!("=== Points Scaling Benchmark ===");
        }

        let mut results = Vec::new();

        for &points in point_counts {
            let name = format!("points_scaling_{}", points);
            let result = self.bench_tqqc(&name, qubits, noise, points);
            results.push(result);
        }

        results
    }

    /// Run full benchmark suite
    pub fn run_all(&mut self) -> Vec<BenchmarkResult> {
        if self.verbose {
            println!("=== Running Full Benchmark Suite ===");
        }

        let mut all_results = Vec::new();

        // Qubit scaling (5, 7, 9 qubits)
        let qubit_results = self.run_qubit_scaling(7, 0.02, 10);
        all_results.extend(qubit_results);

        // Noise scaling
        let noise_levels = vec![0.01, 0.015, 0.02, 0.025, 0.03];
        let noise_results = self.run_noise_scaling(7, &noise_levels, 10);
        all_results.extend(noise_results);

        // Points scaling
        let point_counts = vec![5, 10, 15, 20];
        let points_results = self.run_points_scaling(7, 0.02, &point_counts);
        all_results.extend(points_results);

        all_results
    }

    /// Run quick benchmark (for testing)
    pub fn run_quick(&mut self) -> Vec<BenchmarkResult> {
        if self.verbose {
            println!("=== Running Quick Benchmark ===");
        }

        let mut results = Vec::new();

        // Just a few quick benchmarks
        results.push(self.bench_tqqc("quick_5q", 5, 0.02, 5));
        results.push(self.bench_tqqc("quick_7q", 7, 0.02, 5));

        results
    }

    // ========================================================================
    // Results
    // ========================================================================

    /// Get all results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }

    /// Get statistics
    pub fn statistics(&self) -> BenchmarkStatistics {
        BenchmarkStatistics::from_results(&self.results)
    }
}

impl Default for BenchSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Benchmark statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStatistics {
    /// Number of benchmarks
    pub count: usize,

    /// Average improvement percentage
    pub avg_improvement_percent: f64,

    /// Maximum improvement percentage
    pub max_improvement_percent: f64,

    /// Minimum improvement percentage
    pub min_improvement_percent: f64,

    /// Average execution time (ms)
    pub avg_time_ms: f64,

    /// Total execution time (ms)
    pub total_time_ms: u64,

    /// Early stop rate
    pub early_stop_rate: f64,
}

impl BenchmarkStatistics {
    /// Compute statistics from results
    pub fn from_results(results: &[BenchmarkResult]) -> Self {
        if results.is_empty() {
            return Self {
                count: 0,
                avg_improvement_percent: 0.0,
                max_improvement_percent: 0.0,
                min_improvement_percent: 0.0,
                avg_time_ms: 0.0,
                total_time_ms: 0,
                early_stop_rate: 0.0,
            };
        }

        let count = results.len();
        let improvements: Vec<f64> = results.iter().map(|r| r.improvement_percent).collect();
        let times: Vec<u64> = results.iter().map(|r| r.time_ms).collect();
        let early_stops: usize = results.iter().filter(|r| r.early_stopped).count();

        Self {
            count,
            avg_improvement_percent: improvements.iter().sum::<f64>() / count as f64,
            max_improvement_percent: improvements
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max),
            min_improvement_percent: improvements.iter().cloned().fold(f64::INFINITY, f64::min),
            avg_time_ms: times.iter().sum::<u64>() as f64 / count as f64,
            total_time_ms: times.iter().sum(),
            early_stop_rate: early_stops as f64 / count as f64,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_suite_new() {
        let suite = BenchSuite::new();
        assert!(suite.results().is_empty());
    }

    #[test]
    fn test_bench_tqqc() {
        let mut suite = BenchSuite::with_seed(42);
        let result = suite.bench_tqqc("test", 5, 0.02, 3);

        assert_eq!(result.qubits, 5);
        assert_eq!(result.noise, 0.02);
        assert!(result.iterations > 0);
    }

    #[test]
    fn test_bench_niso() {
        let mut suite = BenchSuite::with_seed(42);
        let config = NisoConfig::quick(5);
        let result = suite.bench_niso("test_niso", config);

        assert_eq!(result.qubits, 5);
    }

    #[test]
    fn test_run_quick() {
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_quick();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_qubit_scaling() {
        let mut suite = BenchSuite::with_seed(42);
        let results = suite.run_qubit_scaling(5, 0.02, 3);

        assert_eq!(results.len(), 3); // 3, 4, 5 qubits
    }

    #[test]
    fn test_noise_scaling() {
        let mut suite = BenchSuite::with_seed(42);
        let noise_levels = vec![0.01, 0.02];
        let results = suite.run_noise_scaling(5, &noise_levels, 3);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut suite = BenchSuite::with_seed(42);
        suite.bench_tqqc("test1", 5, 0.02, 3);
        suite.bench_tqqc("test2", 5, 0.01, 3);

        let stats = suite.statistics();

        assert_eq!(stats.count, 2);
        assert!(stats.total_time_ms > 0);
    }

    #[test]
    fn test_empty_statistics() {
        let suite = BenchSuite::new();
        let stats = suite.statistics();

        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_benchmark_result_from_tqqc() {
        let result = TqqcResult {
            delta_opt: 0.1,
            parity_baseline: 0.5,
            parity_final: 0.6,
            improvement: 0.1,
            iterations: 10,
            early_stopped: false,
            ties_count: 0,
            significant_moves: 5,
            total_inner_iterations: 15,
            history: vec![],
        };

        let bench = BenchmarkResult::from_tqqc("test", 5, 0.02, &result, 100, 8192);

        assert_eq!(bench.name, "test");
        assert_eq!(bench.qubits, 5);
        assert_eq!(bench.iterations, 10);
        assert_eq!(bench.total_shots, (15 * 2 + 1) * 8192);
    }
}
