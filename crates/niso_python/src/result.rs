//! Python bindings for results
//!
//! Gantree: L9_Python → PyTqqcResult, PyOptimizationResult

use niso_engine::OptimizationResult;
use niso_tqqc::{IterationRecord, TqqcResult};
use pyo3::prelude::*;
use std::collections::HashMap;

/// Python wrapper for IterationRecord
#[pyclass(name = "IterationRecord")]
#[derive(Clone)]
pub struct PyIterationRecord {
    inner: IterationRecord,
}

#[pymethods]
impl PyIterationRecord {
    /// Iteration number
    #[getter]
    pub fn iteration(&self) -> usize {
        self.inner.iteration
    }

    /// Delta value
    #[getter]
    pub fn delta(&self) -> f64 {
        self.inner.delta
    }

    /// Parity for +delta
    #[getter]
    pub fn parity_plus(&self) -> f64 {
        self.inner.parity_plus
    }

    /// Parity for -delta
    #[getter]
    pub fn parity_minus(&self) -> f64 {
        self.inner.parity_minus
    }

    /// Selected parity
    #[getter]
    pub fn parity_selected(&self) -> f64 {
        self.inner.parity_selected
    }

    /// Improvement
    #[getter]
    pub fn improvement(&self) -> f64 {
        self.inner.improvement
    }

    /// Inner iteration count
    #[getter]
    pub fn inner_count(&self) -> usize {
        self.inner.inner_count
    }

    /// Direction (Plus, Minus, Stay, or None)
    #[getter]
    pub fn direction(&self) -> String {
        match &self.inner.direction {
            Some(d) => format!("{:?}", d),
            None => "None".to_string(),
        }
    }

    /// Whether move was statistically significant
    #[getter]
    pub fn is_significant(&self) -> bool {
        self.inner.is_significant
    }

    fn __repr__(&self) -> String {
        format!(
            "IterationRecord(iter={}, delta={:.4}, parity={:.4}, improvement={:.4})",
            self.inner.iteration,
            self.inner.delta,
            self.inner.parity_selected,
            self.inner.improvement
        )
    }
}

impl From<IterationRecord> for PyIterationRecord {
    fn from(inner: IterationRecord) -> Self {
        Self { inner }
    }
}

/// Python wrapper for TqqcResult
/// Gantree: PyTqqcResult // TQQC 결과 바인딩
#[pyclass(name = "TqqcResult")]
#[derive(Clone)]
pub struct PyTqqcResult {
    pub(crate) inner: TqqcResult,
}

#[pymethods]
impl PyTqqcResult {
    /// Optimal delta value
    #[getter]
    pub fn delta_opt(&self) -> f64 {
        self.inner.delta_opt
    }

    /// Baseline parity (before optimization)
    #[getter]
    pub fn parity_baseline(&self) -> f64 {
        self.inner.parity_baseline
    }

    /// Final parity (after optimization)
    #[getter]
    pub fn parity_final(&self) -> f64 {
        self.inner.parity_final
    }

    /// Absolute improvement (final - baseline)
    #[getter]
    pub fn improvement(&self) -> f64 {
        self.inner.improvement
    }

    /// Improvement percentage
    #[getter]
    pub fn improvement_percent(&self) -> f64 {
        self.inner.improvement_percent()
    }

    /// Number of outer iterations
    #[getter]
    pub fn iterations(&self) -> usize {
        self.inner.iterations
    }

    /// Whether optimization stopped early
    #[getter]
    pub fn early_stopped(&self) -> bool {
        self.inner.early_stopped
    }

    /// Number of tie decisions
    #[getter]
    pub fn ties_count(&self) -> usize {
        self.inner.ties_count
    }

    /// Number of statistically significant moves
    #[getter]
    pub fn significant_moves(&self) -> usize {
        self.inner.significant_moves
    }

    /// Total inner iterations
    #[getter]
    pub fn total_inner_iterations(&self) -> usize {
        self.inner.total_inner_iterations
    }

    /// Whether improvement occurred
    pub fn improved(&self) -> bool {
        self.inner.improved()
    }

    /// Estimated k value
    pub fn k_estimated(&self, max_points: usize) -> f64 {
        self.inner.k_estimated(max_points)
    }

    /// Get iteration history
    #[getter]
    pub fn history(&self) -> Vec<PyIterationRecord> {
        self.inner
            .history
            .iter()
            .cloned()
            .map(PyIterationRecord::from)
            .collect()
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
            "TqqcResult(delta_opt={:.4}, improvement={:.2}%, iterations={}, early_stopped={})",
            self.inner.delta_opt,
            self.inner.improvement_percent(),
            self.inner.iterations,
            self.inner.early_stopped
        )
    }
}

impl From<TqqcResult> for PyTqqcResult {
    fn from(inner: TqqcResult) -> Self {
        Self { inner }
    }
}

/// Python wrapper for OptimizationResult
/// Gantree: PyOptimizationResult // 통합 결과 바인딩
#[pyclass(name = "OptimizationResult")]
#[derive(Clone)]
pub struct PyOptimizationResult {
    pub(crate) inner: OptimizationResult,
}

#[pymethods]
impl PyOptimizationResult {
    /// Get TQQC result
    #[getter]
    pub fn tqqc_result(&self) -> PyTqqcResult {
        PyTqqcResult::from(self.inner.tqqc_result.clone())
    }

    /// Optimal delta value
    #[getter]
    pub fn delta_opt(&self) -> f64 {
        self.inner.tqqc_result.delta_opt
    }

    /// Baseline parity
    #[getter]
    pub fn baseline_parity(&self) -> f64 {
        self.inner.baseline_parity()
    }

    /// Final parity
    #[getter]
    pub fn final_parity(&self) -> f64 {
        self.inner.final_parity()
    }

    /// Improvement
    #[getter]
    pub fn improvement(&self) -> f64 {
        self.inner.tqqc_result.improvement
    }

    /// Improvement percentage
    #[getter]
    pub fn improvement_percent(&self) -> f64 {
        self.inner.improvement_percent()
    }

    /// Number of iterations
    #[getter]
    pub fn iterations(&self) -> usize {
        self.inner.tqqc_result.iterations
    }

    /// Early stopped
    #[getter]
    pub fn early_stopped(&self) -> bool {
        self.inner.tqqc_result.early_stopped
    }

    /// Total execution time (milliseconds)
    #[getter]
    pub fn total_time_ms(&self) -> u64 {
        self.inner.metrics.total_time_ms
    }

    /// Number of circuit executions
    #[getter]
    pub fn circuit_executions(&self) -> usize {
        self.inner.metrics.circuit_executions
    }

    /// Total shots used
    #[getter]
    pub fn total_shots(&self) -> u64 {
        self.inner.metrics.total_shots
    }

    /// Get schedule metrics if available
    pub fn schedule_metrics(&self) -> Option<HashMap<String, f64>> {
        self.inner.schedule.as_ref().map(|s| {
            let mut map = HashMap::new();
            map.insert("total_duration_ns".to_string(), s.total_duration_ns);
            map.insert("critical_depth".to_string(), s.critical_depth as f64);
            map.insert("parallelism".to_string(), s.parallelism);
            map.insert("idle_time_ns".to_string(), s.idle_time_ns);
            map
        })
    }

    /// Get calibration summary if available
    pub fn calibration_summary(&self) -> Option<HashMap<String, f64>> {
        self.inner.calibration_summary.as_ref().map(|c| {
            let mut map = HashMap::new();
            map.insert("avg_t1".to_string(), c.avg_t1);
            map.insert("avg_t2".to_string(), c.avg_t2);
            map.insert("avg_error_1q".to_string(), c.avg_error_1q);
            map.insert("avg_error_2q".to_string(), c.avg_error_2q);
            map
        })
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

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "NISO Optimization Result:\n\
             - Improvement: {:.2}%\n\
             - Iterations: {}\n\
             - Baseline: {:.4}\n\
             - Final: {:.4}\n\
             - Time: {}ms\n\
             - Early stopped: {}",
            self.inner.improvement_percent(),
            self.inner.tqqc_result.iterations,
            self.inner.baseline_parity(),
            self.inner.final_parity(),
            self.inner.metrics.total_time_ms,
            self.inner.tqqc_result.early_stopped
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "OptimizationResult(improvement={:.2}%, iterations={}, time={}ms)",
            self.inner.improvement_percent(),
            self.inner.tqqc_result.iterations,
            self.inner.metrics.total_time_ms
        )
    }
}

impl From<OptimizationResult> for PyOptimizationResult {
    fn from(inner: OptimizationResult) -> Self {
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niso_tqqc::Direction;

    #[test]
    fn test_py_tqqc_result() {
        let result = TqqcResult {
            delta_opt: 0.15,
            parity_baseline: 0.4,
            parity_final: 0.5,
            improvement: 0.1,
            iterations: 10,
            early_stopped: false,
            ties_count: 2,
            significant_moves: 8,
            total_inner_iterations: 25,
            history: vec![],
        };

        let py_result = PyTqqcResult::from(result);

        assert!((py_result.delta_opt() - 0.15).abs() < 1e-10);
        assert!((py_result.improvement_percent() - 25.0).abs() < 1e-6);
        assert_eq!(py_result.iterations(), 10);
    }

    #[test]
    fn test_py_iteration_record() {
        let record = IterationRecord {
            iteration: 5,
            delta: 0.12,
            parity_plus: 0.45,
            parity_minus: 0.42,
            parity_selected: 0.45,
            improvement: 0.03,
            inner_count: 3,
            direction: Some(Direction::Plus),
            is_significant: true,
        };

        let py_record = PyIterationRecord::from(record);

        assert_eq!(py_record.iteration(), 5);
        assert!((py_record.delta() - 0.12).abs() < 1e-10);
        assert!(py_record.is_significant());
    }
}
