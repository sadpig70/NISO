//! Python bindings for NisoConfig
//!
//! Gantree: L9_Python → PyNisoConfig

use niso_engine::{HardwareTarget, NisoConfig, OptimizationMode};
use pyo3::prelude::*;

/// Python wrapper for OptimizationMode
#[pyclass(name = "OptimizationMode")]
#[derive(Clone)]
pub struct PyOptimizationMode(pub OptimizationMode);

#[pymethods]
impl PyOptimizationMode {
    /// Full optimization
    #[staticmethod]
    pub fn full() -> Self {
        Self(OptimizationMode::Full)
    }

    /// Quick optimization
    #[staticmethod]
    pub fn quick() -> Self {
        Self(OptimizationMode::Quick)
    }

    /// Benchmark mode
    #[staticmethod]
    pub fn benchmark() -> Self {
        Self(OptimizationMode::Benchmark)
    }

    /// Custom mode
    #[staticmethod]
    pub fn custom() -> Self {
        Self(OptimizationMode::Custom)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

/// Python wrapper for HardwareTarget
#[pyclass(name = "HardwareTarget")]
#[derive(Clone)]
pub struct PyHardwareTarget(pub HardwareTarget);

#[pymethods]
impl PyHardwareTarget {
    /// IBM superconducting qubits
    #[staticmethod]
    pub fn ibm() -> Self {
        Self(HardwareTarget::IbmSuperconducting)
    }

    /// Trapped ion
    #[staticmethod]
    pub fn trapped_ion() -> Self {
        Self(HardwareTarget::TrappedIon)
    }

    /// Neutral atom
    #[staticmethod]
    pub fn neutral_atom() -> Self {
        Self(HardwareTarget::NeutralAtom)
    }

    /// Ideal (no noise)
    #[staticmethod]
    pub fn ideal() -> Self {
        Self(HardwareTarget::Ideal)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

/// Python wrapper for NisoConfig
/// Gantree: PyNisoConfig // Config 바인딩
#[pyclass(name = "NisoConfig")]
#[derive(Clone)]
pub struct PyNisoConfig {
    pub(crate) inner: NisoConfig,
}

#[pymethods]
impl PyNisoConfig {
    /// Create new config with specified qubits
    #[new]
    #[pyo3(signature = (qubits=7))]
    pub fn new(qubits: usize) -> Self {
        Self {
            inner: NisoConfig::quick(qubits),
        }
    }

    /// Default 5-qubit configuration
    #[staticmethod]
    pub fn default_5q() -> Self {
        Self {
            inner: NisoConfig::default_5q(),
        }
    }

    /// Default 7-qubit configuration
    #[staticmethod]
    pub fn default_7q() -> Self {
        Self {
            inner: NisoConfig::default_7q(),
        }
    }

    /// Quick mode configuration
    #[staticmethod]
    pub fn quick(qubits: usize) -> Self {
        Self {
            inner: NisoConfig::quick(qubits),
        }
    }

    /// Benchmark mode configuration
    #[staticmethod]
    pub fn benchmark(qubits: usize) -> Self {
        Self {
            inner: NisoConfig::benchmark(qubits),
        }
    }

    /// Ideal (no noise) configuration
    #[staticmethod]
    pub fn ideal(qubits: usize) -> Self {
        Self {
            inner: NisoConfig::ideal(qubits),
        }
    }

    // ========================================================================
    // Builder methods (only those that exist in NisoConfig)
    // ========================================================================

    /// Set number of qubits
    pub fn with_qubits(&self, qubits: usize) -> Self {
        Self {
            inner: self.inner.clone().with_qubits(qubits),
        }
    }

    /// Set noise level
    pub fn with_noise(&self, noise: f64) -> Self {
        Self {
            inner: self.inner.clone().with_noise(noise),
        }
    }

    /// Set number of optimization points
    pub fn with_points(&self, points: usize) -> Self {
        Self {
            inner: self.inner.clone().with_points(points),
        }
    }

    /// Set number of shots per measurement
    pub fn with_shots(&self, shots: u64) -> Self {
        Self {
            inner: self.inner.clone().with_shots(shots),
        }
    }

    /// Set random seed
    pub fn with_seed(&self, seed: u64) -> Self {
        Self {
            inner: self.inner.clone().with_seed(seed),
        }
    }

    /// Enable/disable dynamic inner loop
    pub fn with_dynamic_inner(&self, enabled: bool) -> Self {
        Self {
            inner: self.inner.clone().with_dynamic_inner(enabled),
        }
    }

    /// Enable/disable statistical testing
    pub fn with_statistical_test(&self, enabled: bool) -> Self {
        Self {
            inner: self.inner.clone().with_statistical_test(enabled),
        }
    }

    /// Enable/disable verbose output
    pub fn with_verbose(&self, verbose: bool) -> Self {
        Self {
            inner: self.inner.clone().with_verbose(verbose),
        }
    }

    /// Set optimization mode
    pub fn with_mode(&self, mode: PyOptimizationMode) -> Self {
        Self {
            inner: self.inner.clone().with_mode(mode.0),
        }
    }

    /// Set hardware target
    pub fn with_hardware(&self, hardware: PyHardwareTarget) -> Self {
        Self {
            inner: self.inner.clone().with_hardware(hardware.0),
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

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

    /// Number of points
    #[getter]
    pub fn points(&self) -> usize {
        self.inner.points
    }

    /// Number of shots
    #[getter]
    pub fn shots(&self) -> u64 {
        self.inner.shots
    }

    /// Random seed
    #[getter]
    pub fn seed(&self) -> Option<u64> {
        self.inner.seed
    }

    /// Dynamic inner enabled
    #[getter]
    pub fn dynamic_inner(&self) -> bool {
        self.inner.dynamic_inner
    }

    /// Statistical test enabled
    #[getter]
    pub fn use_statistical_test(&self) -> bool {
        self.inner.use_statistical_test
    }

    /// Check if configuration is TQQC-recommended
    pub fn is_recommended(&self) -> bool {
        self.inner.is_recommended()
    }

    /// Validate configuration
    pub fn validate(&self) -> PyResult<()> {
        self.inner
            .validate()
            .map_err(pyo3::exceptions::PyValueError::new_err)
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Convert to dictionary
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let json_str = self.to_json()?;
        let json_module = py.import("json")?;
        json_module
            .call_method1("loads", (json_str,))
            .map(|o| o.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "NisoConfig(qubits={}, noise={}, points={}, shots={})",
            self.inner.qubits, self.inner.noise, self.inner.points, self.inner.shots
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_config_new() {
        let config = PyNisoConfig::new(7);
        assert_eq!(config.qubits(), 7);
    }

    #[test]
    fn test_py_config_builder() {
        let config = PyNisoConfig::default_7q()
            .with_noise(0.02)
            .with_points(20)
            .with_seed(42);

        assert_eq!(config.qubits(), 7);
        assert!((config.noise() - 0.02).abs() < 1e-10);
        assert_eq!(config.points(), 20);
        assert_eq!(config.seed(), Some(42));
    }

    #[test]
    fn test_py_config_validate() {
        let config = PyNisoConfig::default_7q();
        assert!(config.validate().is_ok());
    }
}
