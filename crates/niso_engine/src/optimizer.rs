//! Integrated optimizer for NISO
//!
//! Gantree: L7_Integration → NisoOptimizer
//!
//! Provides unified optimization interface combining all NISO subsystems.

use crate::config::NisoConfig;
use niso_backend::{Backend, ExecutionResult, SimulatorBackend};
use niso_calibration::{CalibrationCache, CalibrationInfo};
use niso_core::{Circuit, NisoError, NisoResult};
use niso_schedule::Scheduler;
use niso_tqqc::{Parity, TqqcEngine, TqqcResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Optimization result with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// TQQC optimization result
    pub tqqc_result: TqqcResult,

    /// Circuit schedule (if computed)
    pub schedule: Option<ScheduleMetrics>,

    /// Calibration info used
    pub calibration_summary: Option<CalibrationSummary>,

    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

/// Schedule metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetrics {
    /// Total duration in nanoseconds
    pub total_duration_ns: f64,

    /// Critical path depth
    pub critical_depth: usize,

    /// Parallelism factor
    pub parallelism: f64,

    /// Total idle time
    pub idle_time_ns: f64,
}

/// Calibration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSummary {
    /// Backend name
    pub backend: String,

    /// Average T1 (microseconds)
    pub avg_t1: f64,

    /// Average T2 (microseconds)
    pub avg_t2: f64,

    /// Average 1Q gate error
    pub avg_error_1q: f64,

    /// Average 2Q gate error
    pub avg_error_2q: f64,
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution time
    pub total_time_ms: u64,

    /// Number of circuit executions
    pub circuit_executions: usize,

    /// Total shots used
    pub total_shots: u64,

    /// Early stopped
    pub early_stopped: bool,
}

impl OptimizationResult {
    /// Get improvement percentage
    pub fn improvement_percent(&self) -> f64 {
        self.tqqc_result.improvement_percent()
    }

    /// Check if optimization improved
    pub fn improved(&self) -> bool {
        self.tqqc_result.improved()
    }

    /// Get final parity
    pub fn final_parity(&self) -> f64 {
        self.tqqc_result.parity_final
    }

    /// Get baseline parity
    pub fn baseline_parity(&self) -> f64 {
        self.tqqc_result.parity_baseline
    }
}

/// Integrated NISO optimizer
/// Gantree: NisoOptimizer // 통합 최적화
pub struct NisoOptimizer {
    /// Configuration
    config: NisoConfig,

    /// Calibration cache
    calibration_cache: CalibrationCache,

    /// Current calibration info
    calibration: Option<CalibrationInfo>,

    /// Verbose output
    verbose: bool,
}

impl NisoOptimizer {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new optimizer with configuration
    pub fn new(config: NisoConfig) -> Self {
        Self {
            verbose: config.verbose,
            config,
            calibration_cache: CalibrationCache::default_ttl(),
            calibration: None,
        }
    }

    /// Create optimizer with default 7-qubit configuration
    pub fn default_7q() -> Self {
        Self::new(NisoConfig::default_7q())
    }

    /// Create optimizer with default 5-qubit configuration
    pub fn default_5q() -> Self {
        Self::new(NisoConfig::default_5q())
    }

    /// Create quick optimizer
    pub fn quick(qubits: usize) -> Self {
        Self::new(NisoConfig::quick(qubits))
    }

    // ========================================================================
    // Configuration
    // ========================================================================

    /// Set configuration
    pub fn with_config(mut self, config: NisoConfig) -> Self {
        self.verbose = config.verbose;
        self.config = config;
        self
    }

    /// Set calibration info
    pub fn with_calibration(mut self, calibration: CalibrationInfo) -> Self {
        self.calibration = Some(calibration);
        self
    }

    /// Get current configuration
    pub fn config(&self) -> &NisoConfig {
        &self.config
    }

    // ========================================================================
    // Main Optimization
    // ========================================================================

    /// Run full TQQC optimization
    ///
    /// This is the main entry point for optimization.
    pub fn optimize(&mut self) -> NisoResult<OptimizationResult> {
        let start_time = Instant::now();

        // Validate configuration
        self.config
            .validate()
            .map_err(|e| NisoError::InvalidGateParameter(e))?;

        if self.verbose {
            println!("Starting NISO optimization: {}", self.config);
        }

        // Create backend
        let backend = self.create_backend()?;

        // Create TQQC engine
        let tqqc_config = self.config.to_tqqc_config();
        let mut engine = TqqcEngine::new(tqqc_config, backend);

        // Run optimization
        let tqqc_result = engine.optimize()?;

        // Compute schedule metrics if calibration available
        let schedule = self.compute_schedule_metrics();

        // Build calibration summary
        let calibration_summary = self.calibration.as_ref().map(|c| CalibrationSummary {
            backend: c.backend_name.clone(),
            avg_t1: c.avg_t1(),
            avg_t2: c.avg_t2(),
            avg_error_1q: c.avg_error_1q(),
            avg_error_2q: c.avg_error_2q(),
        });

        // Compute metrics
        let total_time_ms = start_time.elapsed().as_millis() as u64;
        let circuit_executions = tqqc_result.total_inner_iterations * 2 + 1; // +1 for baseline
        let total_shots = circuit_executions as u64 * self.config.shots;

        let metrics = ExecutionMetrics {
            total_time_ms,
            circuit_executions,
            total_shots,
            early_stopped: tqqc_result.early_stopped,
        };

        if self.verbose {
            println!(
                "Optimization complete: {:.2}% improvement in {}ms",
                tqqc_result.improvement_percent(),
                total_time_ms
            );
        }

        Ok(OptimizationResult {
            tqqc_result,
            schedule,
            calibration_summary,
            metrics,
        })
    }

    /// Run optimization with custom circuit
    pub fn optimize_circuit(&mut self, circuit: &Circuit) -> NisoResult<ExecutionResult> {
        let backend = self.create_backend()?;
        backend.execute(circuit, self.config.shots)
    }

    /// Measure parity of a circuit
    pub fn measure_parity(&self, theta: f64, delta: f64) -> NisoResult<f64> {
        let backend = self.create_backend()?;
        let tqqc_config = self.config.to_tqqc_config();
        let circuit = Parity::build_circuit(&tqqc_config, theta, delta);
        let result = backend.execute(&circuit, self.config.shots)?;
        Ok(Parity::expectation(&result.counts))
    }

    // ========================================================================
    // Backend Creation
    // ========================================================================

    /// Create backend based on configuration
    fn create_backend(&self) -> NisoResult<SimulatorBackend> {
        let noise_model = self.config.to_noise_model();

        let mut backend = SimulatorBackend::new(self.config.qubits, noise_model);

        if let Some(seed) = self.config.seed {
            backend = backend.with_seed(seed);
        }

        if let Some(ref cal) = self.calibration {
            backend = backend.with_calibration(cal.clone());
        }

        Ok(backend)
    }

    // ========================================================================
    // Schedule Computation
    // ========================================================================

    /// Compute schedule metrics for TQQC circuit
    fn compute_schedule_metrics(&self) -> Option<ScheduleMetrics> {
        let tqqc_config = self.config.to_tqqc_config();
        let circuit = Parity::build_circuit(&tqqc_config, 0.0, 0.0);
        let gate_times = self.config.to_gate_times();

        let schedule = Scheduler::compute_asap(&circuit, &gate_times);

        Some(ScheduleMetrics {
            total_duration_ns: schedule.total_duration_ns(),
            critical_depth: schedule.critical_path_depth(),
            parallelism: schedule.parallelism_factor(),
            idle_time_ns: schedule.total_idle_time(),
        })
    }

    // ========================================================================
    // Calibration Management
    // ========================================================================

    /// Set calibration from backend name (using cache)
    pub fn calibrate(&mut self, backend_name: &str) -> NisoResult<()> {
        // Try cache first
        if let Some(cached) = self.calibration_cache.get(backend_name) {
            self.calibration = Some(cached);
            return Ok(());
        }

        // Create default calibration
        let calibration = CalibrationInfo::ibm_typical(self.config.qubits);
        self.calibration_cache
            .set(backend_name, calibration.clone());
        self.calibration = Some(calibration);

        Ok(())
    }

    /// Get current calibration
    pub fn calibration(&self) -> Option<&CalibrationInfo> {
        self.calibration.as_ref()
    }

    /// Invalidate calibration cache
    pub fn invalidate_calibration(&mut self) {
        self.calibration_cache.invalidate_all();
        self.calibration = None;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_new() {
        let optimizer = NisoOptimizer::default_7q();
        assert_eq!(optimizer.config().qubits, 7);
    }

    #[test]
    fn test_optimizer_quick() {
        let optimizer = NisoOptimizer::quick(5);
        assert_eq!(optimizer.config().qubits, 5);
        assert_eq!(optimizer.config().points, 10);
    }

    #[test]
    fn test_optimize() {
        let config = NisoConfig::default_5q()
            .with_noise(0.01)
            .with_points(3)
            .with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        assert!(result.tqqc_result.iterations > 0);
        assert!(result.metrics.total_time_ms > 0);
    }

    #[test]
    fn test_measure_parity() {
        let config = NisoConfig::ideal(5).with_seed(42);
        let optimizer = NisoOptimizer::new(config);

        let parity = optimizer.measure_parity(0.0, 0.0).unwrap();
        assert!(parity >= -1.0 && parity <= 1.0);
    }

    #[test]
    fn test_calibration() {
        let mut optimizer = NisoOptimizer::default_5q();

        optimizer.calibrate("test_backend").unwrap();

        assert!(optimizer.calibration().is_some());
    }

    #[test]
    fn test_schedule_metrics() {
        let config = NisoConfig::default_7q().with_points(2).with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        assert!(result.schedule.is_some());
        let schedule = result.schedule.unwrap();
        assert!(schedule.total_duration_ns > 0.0);
    }

    #[test]
    fn test_optimization_result_methods() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        // Methods should work
        let _ = result.improvement_percent();
        let _ = result.improved();
        let _ = result.final_parity();
        let _ = result.baseline_parity();
    }
}
