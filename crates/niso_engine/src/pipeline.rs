//! Pipeline for staged NISO execution
//!
//! Gantree: L7_Integration → Pipeline
//!
//! Provides staged execution with intermediate results.

use crate::config::NisoConfig;
use crate::optimizer::{CalibrationSummary, OptimizationResult, ScheduleMetrics};
use niso_backend::SimulatorBackend;
use niso_calibration::CalibrationInfo;
use niso_core::{Circuit, NisoError, NisoResult};
use niso_noise::NoiseVectorSet;
use niso_schedule::{CircuitSchedule, Scheduler};
use niso_tqqc::{Parity, TqqcEngine, TqqcResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    /// Initial state
    Initial,
    /// Calibration completed
    Calibrated,
    /// Circuit built
    CircuitBuilt,
    /// Schedule computed
    Scheduled,
    /// Optimization completed
    Optimized,
}

/// Pipeline state holding intermediate results
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Current stage
    pub stage: PipelineStage,

    /// Configuration
    pub config: NisoConfig,

    /// Calibration info
    pub calibration: Option<CalibrationInfo>,

    /// Built circuit
    pub circuit: Option<Circuit>,

    /// Circuit schedule
    pub schedule: Option<CircuitSchedule>,

    /// Noise vectors
    pub noise_vectors: Option<NoiseVectorSet>,

    /// TQQC result
    pub tqqc_result: Option<TqqcResult>,
}

impl PipelineState {
    /// Create new pipeline state
    pub fn new(config: NisoConfig) -> Self {
        Self {
            stage: PipelineStage::Initial,
            config,
            calibration: None,
            circuit: None,
            schedule: None,
            noise_vectors: None,
            tqqc_result: None,
        }
    }

    /// Check if calibrated
    pub fn is_calibrated(&self) -> bool {
        self.calibration.is_some()
    }

    /// Check if circuit built
    pub fn has_circuit(&self) -> bool {
        self.circuit.is_some()
    }

    /// Check if scheduled
    pub fn is_scheduled(&self) -> bool {
        self.schedule.is_some()
    }

    /// Check if optimized
    pub fn is_optimized(&self) -> bool {
        self.tqqc_result.is_some()
    }
}

/// NISO execution pipeline
/// Gantree: Pipeline // 단계별 실행
pub struct Pipeline {
    /// Current state
    state: PipelineState,

    /// Verbose output
    verbose: bool,
}

impl Pipeline {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new pipeline with configuration
    pub fn new(config: NisoConfig) -> Self {
        let verbose = config.verbose;
        Self {
            state: PipelineState::new(config),
            verbose,
        }
    }

    /// Create pipeline with default 7-qubit configuration
    pub fn default_7q() -> Self {
        Self::new(NisoConfig::default_7q())
    }

    /// Create pipeline with default 5-qubit configuration
    pub fn default_5q() -> Self {
        Self::new(NisoConfig::default_5q())
    }

    // ========================================================================
    // Stage Accessors
    // ========================================================================

    /// Get current stage
    pub fn stage(&self) -> PipelineStage {
        self.state.stage
    }

    /// Get current state
    pub fn state(&self) -> &PipelineState {
        &self.state
    }

    /// Get configuration
    pub fn config(&self) -> &NisoConfig {
        &self.state.config
    }

    // ========================================================================
    // Pipeline Stages
    // ========================================================================

    /// Stage 1: Calibration
    ///
    /// Sets up calibration info from backend or defaults.
    pub fn calibrate(&mut self) -> NisoResult<&CalibrationInfo> {
        if self.verbose {
            println!("Pipeline: Calibrating...");
        }

        // Create calibration from config
        let calibration = CalibrationInfo::uniform(
            "niso_simulator",
            self.state.config.qubits,
            self.state.config.t1_us,
            self.state.config.t2_us,
            self.state.config.gate_error_1q,
            self.state.config.gate_error_2q,
            self.state.config.readout_error,
        );

        // Create noise vectors
        let noise_vectors = calibration.to_noise_vectors();

        self.state.calibration = Some(calibration);
        self.state.noise_vectors = Some(noise_vectors);
        self.state.stage = PipelineStage::Calibrated;

        Ok(self.state.calibration.as_ref().unwrap())
    }

    /// Stage 2: Build circuit
    ///
    /// Constructs the TQQC parity circuit.
    pub fn build_circuit(&mut self, theta: f64, delta: f64) -> NisoResult<&Circuit> {
        if self.verbose {
            println!(
                "Pipeline: Building circuit (θ={:.4}, δ={:.4})...",
                theta, delta
            );
        }

        let tqqc_config = self.state.config.to_tqqc_config();
        let circuit = Parity::build_circuit(&tqqc_config, theta, delta);

        self.state.circuit = Some(circuit);
        self.state.stage = PipelineStage::CircuitBuilt;

        Ok(self.state.circuit.as_ref().unwrap())
    }

    /// Stage 3: Schedule circuit
    ///
    /// Computes ASAP schedule for the circuit.
    pub fn schedule(&mut self) -> NisoResult<&CircuitSchedule> {
        // Ensure circuit is built
        if self.state.circuit.is_none() {
            self.build_circuit(0.0, 0.0)?;
        }

        if self.verbose {
            println!("Pipeline: Scheduling circuit...");
        }

        let circuit = self.state.circuit.as_ref().unwrap();
        let gate_times = self.state.config.to_gate_times();

        let schedule = Scheduler::compute_asap(circuit, &gate_times);

        self.state.schedule = Some(schedule);
        self.state.stage = PipelineStage::Scheduled;

        Ok(self.state.schedule.as_ref().unwrap())
    }

    /// Stage 4: Optimize
    ///
    /// Runs TQQC optimization.
    pub fn optimize(&mut self) -> NisoResult<&TqqcResult> {
        if self.verbose {
            println!("Pipeline: Running TQQC optimization...");
        }

        // Validate config
        self.state
            .config
            .validate()
            .map_err(|e| NisoError::InvalidGateParameter(e))?;

        // Create backend
        let noise_model = self.state.config.to_noise_model();
        let mut backend = SimulatorBackend::new(self.state.config.qubits, noise_model);

        if let Some(seed) = self.state.config.seed {
            backend = backend.with_seed(seed);
        }

        // Create TQQC engine and run
        let tqqc_config = self.state.config.to_tqqc_config();
        let mut engine = TqqcEngine::new(tqqc_config, backend);
        let result = engine.optimize()?;

        self.state.tqqc_result = Some(result);
        self.state.stage = PipelineStage::Optimized;

        Ok(self.state.tqqc_result.as_ref().unwrap())
    }

    /// Run full pipeline
    ///
    /// Executes all stages in sequence.
    pub fn run(&mut self) -> NisoResult<OptimizationResult> {
        let start_time = Instant::now();

        // Run all stages
        self.calibrate()?;
        self.build_circuit(0.0, 0.0)?;
        self.schedule()?;
        self.optimize()?;

        // Build result
        let tqqc_result = self.state.tqqc_result.clone().unwrap();

        let schedule = self.state.schedule.as_ref().map(|s| ScheduleMetrics {
            total_duration_ns: s.total_duration_ns(),
            critical_depth: s.critical_path_depth(),
            parallelism: s.parallelism_factor(),
            idle_time_ns: s.total_idle_time(),
        });

        let calibration_summary = self.state.calibration.as_ref().map(|c| CalibrationSummary {
            backend: c.backend_name.clone(),
            avg_t1: c.avg_t1(),
            avg_t2: c.avg_t2(),
            avg_error_1q: c.avg_error_1q(),
            avg_error_2q: c.avg_error_2q(),
        });

        let total_time_ms = start_time.elapsed().as_millis() as u64;
        let circuit_executions = tqqc_result.total_inner_iterations * 2 + 1;

        Ok(OptimizationResult {
            tqqc_result,
            schedule,
            calibration_summary,
            metrics: crate::optimizer::ExecutionMetrics {
                total_time_ms,
                circuit_executions,
                total_shots: circuit_executions as u64 * self.state.config.shots,
                early_stopped: self
                    .state
                    .tqqc_result
                    .as_ref()
                    .map(|r| r.early_stopped)
                    .unwrap_or(false),
            },
        })
    }

    // ========================================================================
    // Reset
    // ========================================================================

    /// Reset pipeline to initial state
    pub fn reset(&mut self) {
        let config = self.state.config.clone();
        self.state = PipelineState::new(config);
    }

    /// Reset and reconfigure
    pub fn reconfigure(&mut self, config: NisoConfig) {
        self.verbose = config.verbose;
        self.state = PipelineState::new(config);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_new() {
        let pipeline = Pipeline::default_7q();
        assert_eq!(pipeline.stage(), PipelineStage::Initial);
    }

    #[test]
    fn test_calibrate() {
        let mut pipeline = Pipeline::default_5q();

        pipeline.calibrate().unwrap();

        assert_eq!(pipeline.stage(), PipelineStage::Calibrated);
        assert!(pipeline.state().is_calibrated());
    }

    #[test]
    fn test_build_circuit() {
        let mut pipeline = Pipeline::default_5q();

        pipeline.build_circuit(0.5, 0.1).unwrap();

        assert_eq!(pipeline.stage(), PipelineStage::CircuitBuilt);
        assert!(pipeline.state().has_circuit());
    }

    #[test]
    fn test_schedule() {
        let mut pipeline = Pipeline::default_5q();

        pipeline.schedule().unwrap();

        assert_eq!(pipeline.stage(), PipelineStage::Scheduled);
        assert!(pipeline.state().is_scheduled());
    }

    #[test]
    fn test_optimize() {
        let config = NisoConfig::default_5q().with_points(3).with_seed(42);

        let mut pipeline = Pipeline::new(config);

        pipeline.optimize().unwrap();

        assert_eq!(pipeline.stage(), PipelineStage::Optimized);
        assert!(pipeline.state().is_optimized());
    }

    #[test]
    fn test_full_run() {
        let config = NisoConfig::default_5q().with_points(3).with_seed(42);

        let mut pipeline = Pipeline::new(config);
        let result = pipeline.run().unwrap();

        assert!(result.tqqc_result.iterations > 0);
        assert!(result.schedule.is_some());
        assert!(result.calibration_summary.is_some());
    }

    #[test]
    fn test_reset() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut pipeline = Pipeline::new(config);

        pipeline.calibrate().unwrap();
        assert!(pipeline.state().is_calibrated());

        pipeline.reset();
        assert_eq!(pipeline.stage(), PipelineStage::Initial);
        assert!(!pipeline.state().is_calibrated());
    }

    #[test]
    fn test_reconfigure() {
        let mut pipeline = Pipeline::default_7q();

        pipeline.calibrate().unwrap();

        let new_config = NisoConfig::default_5q();
        pipeline.reconfigure(new_config);

        assert_eq!(pipeline.stage(), PipelineStage::Initial);
        assert_eq!(pipeline.config().qubits, 5);
    }

    #[test]
    fn test_staged_execution() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut pipeline = Pipeline::new(config);

        // Stage 1
        let cal = pipeline.calibrate().unwrap();
        assert!(cal.num_qubits() >= 5);

        // Stage 2
        let circuit = pipeline.build_circuit(0.0, 0.0).unwrap();
        assert_eq!(circuit.num_qubits(), 5);

        // Stage 3
        let schedule = pipeline.schedule().unwrap();
        assert!(schedule.total_duration_ns() > 0.0);

        // Stage 4
        let result = pipeline.optimize().unwrap();
        assert!(result.iterations > 0);
    }
}
