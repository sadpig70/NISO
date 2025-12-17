//! Python bindings for benchmarking
//!
//! Gantree: L9_Python → PyBenchSuite, PyCircuitGenerator

use crate::config::PyNisoConfig;
use niso_bench::{BenchSuite, BenchmarkResult, BenchmarkStatistics, CircuitGenerator, Reporter};
use pyo3::prelude::*;
use serde_json;
use std::collections::HashMap;

/// Python wrapper for BenchmarkResult
#[pyclass(name = "BenchmarkResult")]
#[derive(Clone)]
pub struct PyBenchmarkResult {
    inner: BenchmarkResult,
}

#[pymethods]
impl PyBenchmarkResult {
    /// Benchmark name
    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Number of qubits
    #[getter]
    pub fn qubits(&self) -> usize {
        self.inner.qubits
    }

    /// Noise level
    #[getter]
    pub fn noise(&self) -> f64 {
        self.inner.noise
    }

    /// Baseline parity
    #[getter]
    pub fn baseline(&self) -> f64 {
        self.inner.baseline
    }

    /// Final parity
    #[getter]
    pub fn final_parity(&self) -> f64 {
        self.inner.final_parity
    }

    /// Improvement (final - baseline)
    #[getter]
    pub fn improvement(&self) -> f64 {
        self.inner.improvement
    }

    /// Improvement percentage
    #[getter]
    pub fn improvement_percent(&self) -> f64 {
        self.inner.improvement_percent
    }

    /// Number of iterations
    #[getter]
    pub fn iterations(&self) -> usize {
        self.inner.iterations
    }

    /// Early stopped
    #[getter]
    pub fn early_stopped(&self) -> bool {
        self.inner.early_stopped
    }

    /// Execution time (milliseconds)
    #[getter]
    pub fn time_ms(&self) -> u64 {
        self.inner.time_ms
    }

    /// Total shots used
    #[getter]
    pub fn total_shots(&self) -> u64 {
        self.inner.total_shots
    }

    /// Convert to dictionary
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_str = serde_json::to_string(&self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let json_module = py.import("json")?;
        json_module
            .call_method1("loads", (json_str,))
            .map(|o| o.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "BenchmarkResult({}: {:.2}% improvement in {}ms)",
            self.inner.name, self.inner.improvement_percent, self.inner.time_ms
        )
    }
}

impl From<BenchmarkResult> for PyBenchmarkResult {
    fn from(inner: BenchmarkResult) -> Self {
        Self { inner }
    }
}

/// Python wrapper for BenchmarkStatistics
#[pyclass(name = "Statistics")]
#[derive(Clone)]
pub struct PyStatistics {
    inner: BenchmarkStatistics,
}

#[pymethods]
impl PyStatistics {
    /// Number of samples
    #[getter]
    pub fn count(&self) -> usize {
        self.inner.count
    }

    /// Average improvement percentage
    #[getter]
    pub fn avg_improvement_percent(&self) -> f64 {
        self.inner.avg_improvement_percent
    }

    /// Maximum improvement percentage
    #[getter]
    pub fn max_improvement_percent(&self) -> f64 {
        self.inner.max_improvement_percent
    }

    /// Minimum improvement percentage
    #[getter]
    pub fn min_improvement_percent(&self) -> f64 {
        self.inner.min_improvement_percent
    }

    /// Average execution time (ms)
    #[getter]
    pub fn avg_time_ms(&self) -> f64 {
        self.inner.avg_time_ms
    }

    /// Total execution time (ms)
    #[getter]
    pub fn total_time_ms(&self) -> u64 {
        self.inner.total_time_ms
    }

    /// Early stop rate
    #[getter]
    pub fn early_stop_rate(&self) -> f64 {
        self.inner.early_stop_rate
    }

    /// Convert to dictionary
    pub fn to_dict(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        map.insert("count".to_string(), self.inner.count as f64);
        map.insert(
            "avg_improvement_percent".to_string(),
            self.inner.avg_improvement_percent,
        );
        map.insert(
            "max_improvement_percent".to_string(),
            self.inner.max_improvement_percent,
        );
        map.insert(
            "min_improvement_percent".to_string(),
            self.inner.min_improvement_percent,
        );
        map.insert("avg_time_ms".to_string(), self.inner.avg_time_ms);
        map.insert("total_time_ms".to_string(), self.inner.total_time_ms as f64);
        map.insert("early_stop_rate".to_string(), self.inner.early_stop_rate);
        map
    }

    fn __repr__(&self) -> String {
        format!(
            "Statistics(n={}, avg={:.2}%, early_stop={:.1}%)",
            self.inner.count,
            self.inner.avg_improvement_percent,
            self.inner.early_stop_rate * 100.0
        )
    }
}

impl From<BenchmarkStatistics> for PyStatistics {
    fn from(inner: BenchmarkStatistics) -> Self {
        Self { inner }
    }
}

/// Python wrapper for BenchSuite
/// Gantree: PyBenchSuite // 벤치마크 스위트 바인딩
#[pyclass(name = "BenchSuite")]
pub struct PyBenchSuite {
    inner: BenchSuite,
}

#[pymethods]
impl PyBenchSuite {
    /// Create new benchmark suite
    #[new]
    #[pyo3(signature = (seed=None))]
    pub fn new(seed: Option<u64>) -> Self {
        let inner = match seed {
            Some(s) => BenchSuite::with_seed(s),
            None => BenchSuite::new(),
        };
        Self { inner }
    }

    /// Run TQQC benchmark
    pub fn bench_tqqc(
        &mut self,
        name: &str,
        qubits: usize,
        noise: f64,
        points: usize,
    ) -> PyBenchmarkResult {
        let result = self.inner.bench_tqqc(name, qubits, noise, points);
        PyBenchmarkResult::from(result)
    }

    /// Run NISO benchmark with config
    pub fn bench_niso(&mut self, name: &str, config: PyNisoConfig) -> PyBenchmarkResult {
        let result = self.inner.bench_niso(name, config.inner.clone());
        PyBenchmarkResult::from(result)
    }

    /// Run noise scaling benchmark
    pub fn noise_scaling(
        &mut self,
        qubits: usize,
        noise_levels: Vec<f64>,
        points: usize,
    ) -> Vec<PyBenchmarkResult> {
        self.inner
            .run_noise_scaling(qubits, &noise_levels, points)
            .into_iter()
            .map(PyBenchmarkResult::from)
            .collect()
    }

    /// Run qubit scaling benchmark
    pub fn qubit_scaling(
        &mut self,
        max_qubits: usize,
        noise: f64,
        points: usize,
    ) -> Vec<PyBenchmarkResult> {
        self.inner
            .run_qubit_scaling(max_qubits, noise, points)
            .into_iter()
            .map(PyBenchmarkResult::from)
            .collect()
    }

    /// Run points scaling benchmark
    pub fn points_scaling(
        &mut self,
        qubits: usize,
        noise: f64,
        point_counts: Vec<usize>,
    ) -> Vec<PyBenchmarkResult> {
        self.inner
            .run_points_scaling(qubits, noise, &point_counts)
            .into_iter()
            .map(PyBenchmarkResult::from)
            .collect()
    }

    /// Run full benchmark suite
    pub fn run_all(&mut self) -> Vec<PyBenchmarkResult> {
        self.inner
            .run_all()
            .into_iter()
            .map(PyBenchmarkResult::from)
            .collect()
    }

    /// Run quick benchmark
    pub fn run_quick(&mut self) -> Vec<PyBenchmarkResult> {
        self.inner
            .run_quick()
            .into_iter()
            .map(PyBenchmarkResult::from)
            .collect()
    }

    /// Get all results
    pub fn results(&self) -> Vec<PyBenchmarkResult> {
        self.inner
            .results()
            .iter()
            .map(|r| PyBenchmarkResult::from(r.clone()))
            .collect()
    }

    /// Compute statistics
    pub fn statistics(&self) -> PyStatistics {
        PyStatistics::from(self.inner.statistics())
    }

    /// Clear all results
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Number of results
    pub fn len(&self) -> usize {
        self.inner.results().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.results().is_empty()
    }

    /// Export to JSON
    pub fn to_json(&self) -> PyResult<String> {
        Ok(Reporter::to_json(self.inner.results()))
    }

    /// Export to CSV
    pub fn to_csv(&self) -> String {
        Reporter::to_csv(self.inner.results())
    }

    /// Export to Markdown
    pub fn to_markdown(&self) -> String {
        Reporter::to_markdown(self.inner.results())
    }

    /// Export to text
    pub fn to_text(&self) -> String {
        Reporter::to_text(self.inner.results())
    }

    fn __repr__(&self) -> String {
        format!("BenchSuite(results={})", self.inner.results().len())
    }

    fn __len__(&self) -> usize {
        self.inner.results().len()
    }
}

/// Python wrapper for CircuitGenerator
/// Gantree: PyCircuitGenerator // 회로 생성기 바인딩
#[pyclass(name = "CircuitGenerator")]
pub struct PyCircuitGenerator {
    inner: CircuitGenerator,
}

#[pymethods]
impl PyCircuitGenerator {
    /// Create new generator
    #[new]
    #[pyo3(signature = (seed=None))]
    pub fn new(seed: Option<u64>) -> Self {
        let inner = match seed {
            Some(s) => CircuitGenerator::with_seed(s),
            None => CircuitGenerator::new(),
        };
        Self { inner }
    }

    /// Generate GHZ state circuit
    pub fn ghz(&self, num_qubits: usize) -> PyResult<String> {
        let circuit = self.inner.ghz(num_qubits);
        Ok(circuit.to_qasm())
    }

    /// Generate Bell state circuit
    pub fn bell(&self) -> PyResult<String> {
        let circuit = self.inner.bell();
        Ok(circuit.to_qasm())
    }

    /// Generate QFT circuit
    pub fn qft(&self, num_qubits: usize) -> PyResult<String> {
        let circuit = self.inner.qft(num_qubits);
        Ok(circuit.to_qasm())
    }

    /// Generate hardware-efficient ansatz
    pub fn hea(&self, num_qubits: usize, depth: usize) -> PyResult<String> {
        let circuit = self.inner.hea(num_qubits, depth);
        Ok(circuit.to_qasm())
    }

    /// Generate random circuit
    pub fn random(&self, num_qubits: usize, num_gates: usize) -> PyResult<String> {
        let circuit = self.inner.random(num_qubits, num_gates);
        Ok(circuit.to_qasm())
    }

    /// Generate TQQC parity circuit
    pub fn tqqc_parity(&self, num_qubits: usize, theta: f64, delta: f64) -> PyResult<String> {
        let circuit = self.inner.tqqc_parity(num_qubits, theta, delta);
        Ok(circuit.to_qasm())
    }

    /// Generate W state circuit
    pub fn w_state(&self, num_qubits: usize) -> PyResult<String> {
        let circuit = self.inner.w_state(num_qubits);
        Ok(circuit.to_qasm())
    }

    fn __repr__(&self) -> String {
        "CircuitGenerator()".to_string()
    }
}

/// Run quick noise scaling benchmark
#[pyfunction]
#[pyo3(signature = (qubits=7, noise_levels=None, points=10))]
pub fn noise_scaling_benchmark(
    qubits: usize,
    noise_levels: Option<Vec<f64>>,
    points: usize,
) -> Vec<PyBenchmarkResult> {
    let levels = noise_levels.unwrap_or_else(|| vec![0.01, 0.02, 0.03, 0.04, 0.05]);
    let mut suite = BenchSuite::new();
    suite
        .run_noise_scaling(qubits, &levels, points)
        .into_iter()
        .map(PyBenchmarkResult::from)
        .collect()
}

/// Run quick qubit scaling benchmark
#[pyfunction]
#[pyo3(signature = (max_qubits=7, noise=0.02, points=10))]
pub fn qubit_scaling_benchmark(
    max_qubits: usize,
    noise: f64,
    points: usize,
) -> Vec<PyBenchmarkResult> {
    let mut suite = BenchSuite::new();
    suite
        .run_qubit_scaling(max_qubits, noise, points)
        .into_iter()
        .map(PyBenchmarkResult::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_bench_suite() {
        let mut suite = PyBenchSuite::new(Some(42));
        assert!(suite.is_empty());

        let result = suite.bench_tqqc("test", 5, 0.02, 3);
        assert_eq!(result.qubits(), 5);

        assert_eq!(suite.len(), 1);
    }

    #[test]
    fn test_py_noise_scaling() {
        let mut suite = PyBenchSuite::new(Some(42));
        let results = suite.noise_scaling(5, vec![0.01, 0.02], 3);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_py_circuit_generator() {
        let gen = PyCircuitGenerator::new(Some(42));

        let ghz = gen.ghz(5);
        assert!(ghz.is_ok());
        assert!(ghz.unwrap().contains("OPENQASM"));

        let qft = gen.qft(3);
        assert!(qft.is_ok());
    }

    #[test]
    fn test_py_statistics() {
        let mut suite = PyBenchSuite::new(Some(42));
        suite.bench_tqqc("test1", 5, 0.02, 3);
        suite.bench_tqqc("test2", 5, 0.02, 3);

        let stats = suite.statistics();
        assert_eq!(stats.count(), 2);
    }
}
