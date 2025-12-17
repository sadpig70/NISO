//! Python bindings for NisoOptimizer
//!
//! Gantree: L9_Python → PyNisoOptimizer

use crate::config::PyNisoConfig;
use crate::result::{PyOptimizationResult, PyTqqcResult};
use niso_backend::{Backend, SimulatorBackend};
use niso_engine::NisoOptimizer;
use niso_tqqc::{Parity, TqqcEngine};
use pyo3::prelude::*;
use std::collections::HashMap;

/// Python wrapper for NisoOptimizer
/// Gantree: PyNisoOptimizer // Optimizer 바인딩
#[pyclass(name = "NisoOptimizer")]
pub struct PyNisoOptimizer {
    config: PyNisoConfig,
}

#[pymethods]
impl PyNisoOptimizer {
    /// Create new optimizer with configuration
    #[new]
    pub fn new(config: PyNisoConfig) -> PyResult<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Get current configuration
    #[getter]
    pub fn config(&self) -> PyNisoConfig {
        self.config.clone()
    }

    /// Run TQQC optimization
    ///
    /// Returns TqqcResult with optimization details
    pub fn optimize(&self) -> PyResult<PyTqqcResult> {
        let tqqc_config = self.config.inner.to_tqqc_config();

        let backend =
            SimulatorBackend::from_depol(self.config.inner.qubits, self.config.inner.noise)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .with_seed(self.config.inner.seed.unwrap_or(42));

        let mut engine = TqqcEngine::new(tqqc_config, backend);

        let result = engine
            .optimize()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(PyTqqcResult::from(result))
    }

    /// Run full NISO optimization pipeline
    ///
    /// Returns OptimizationResult with comprehensive metrics
    pub fn optimize_full(&self) -> PyResult<PyOptimizationResult> {
        // NisoOptimizer::new returns Self directly, not Result
        let mut optimizer = NisoOptimizer::new(self.config.inner.clone());

        let result = optimizer
            .optimize()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(PyOptimizationResult::from(result))
    }

    /// Measure parity at specific theta and delta
    ///
    /// Returns parity expectation value in [-1, 1]
    pub fn measure_parity(&self, theta: f64, delta: f64) -> PyResult<f64> {
        let tqqc_config = self.config.inner.to_tqqc_config();

        let backend =
            SimulatorBackend::from_depol(self.config.inner.qubits, self.config.inner.noise)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .with_seed(self.config.inner.seed.unwrap_or(42));

        let circuit = Parity::build_circuit(&tqqc_config, theta, delta);
        // Use Backend trait method
        let result = backend
            .execute(&circuit, self.config.inner.shots)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Parity::expectation(&result.counts))
    }

    /// Measure parity for multiple delta values
    ///
    /// Returns dict mapping delta -> parity
    pub fn scan_delta(&self, theta: f64, deltas: Vec<f64>) -> PyResult<HashMap<String, f64>> {
        let mut results = HashMap::new();

        for delta in deltas {
            let parity = self.measure_parity(theta, delta)?;
            results.insert(format!("{:.4}", delta), parity);
        }

        Ok(results)
    }

    fn __repr__(&self) -> String {
        format!(
            "NisoOptimizer(qubits={}, noise={}, points={})",
            self.config.qubits(),
            self.config.noise(),
            self.config.points()
        )
    }
}

/// Quick optimization function
///
/// Runs TQQC optimization with default settings
#[pyfunction]
#[pyo3(signature = (qubits=7, noise=0.02, points=20, shots=4096, seed=None))]
pub fn quick_optimize(
    qubits: usize,
    noise: f64,
    points: usize,
    shots: u64,
    seed: Option<u64>,
) -> PyResult<PyTqqcResult> {
    let mut config = PyNisoConfig::quick(qubits)
        .with_noise(noise)
        .with_points(points)
        .with_shots(shots);

    if let Some(s) = seed {
        config = config.with_seed(s);
    }

    let optimizer = PyNisoOptimizer::new(config)?;
    optimizer.optimize()
}

/// Full optimization function
///
/// Runs complete NISO pipeline with all metrics
#[pyfunction]
#[pyo3(signature = (qubits=7, noise=0.02, points=20, shots=4096, seed=None))]
pub fn full_optimize(
    qubits: usize,
    noise: f64,
    points: usize,
    shots: u64,
    seed: Option<u64>,
) -> PyResult<PyOptimizationResult> {
    let mut config = PyNisoConfig::default_7q()
        .with_qubits(qubits)
        .with_noise(noise)
        .with_points(points)
        .with_shots(shots);

    if let Some(s) = seed {
        config = config.with_seed(s);
    }

    let optimizer = PyNisoOptimizer::new(config)?;
    optimizer.optimize_full()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_optimizer_new() {
        let config = PyNisoConfig::default_5q().with_noise(0.02);
        let optimizer = PyNisoOptimizer::new(config);
        assert!(optimizer.is_ok());
    }

    #[test]
    fn test_py_optimizer_optimize() {
        let config = PyNisoConfig::default_5q()
            .with_noise(0.02)
            .with_points(3)
            .with_seed(42);

        let optimizer = PyNisoOptimizer::new(config).unwrap();
        let result = optimizer.optimize();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.iterations() > 0);
    }

    #[test]
    fn test_py_optimizer_measure_parity() {
        let config = PyNisoConfig::default_5q().with_noise(0.01).with_seed(42);

        let optimizer = PyNisoOptimizer::new(config).unwrap();
        let parity = optimizer.measure_parity(0.0, 0.0);

        assert!(parity.is_ok());
        let p = parity.unwrap();
        assert!(p >= -1.0 && p <= 1.0);
    }
}
