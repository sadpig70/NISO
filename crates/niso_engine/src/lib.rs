//! # NISO Engine
//!
//! Integrated quantum optimization pipeline.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_engine // L7: Integration (완료)
//!     NisoConfig // 통합 설정 (완료)
//!         qubits, mode, hardware
//!         TQQC params, hardware params
//!         to_tqqc_config(), to_noise_model()
//!     NisoOptimizer // 통합 최적화 (완료)
//!         optimize() - 원클릭 최적화
//!         measure_parity() - 패리티 측정
//!         calibrate() - 캘리브레이션
//!     Pipeline // 단계별 실행 (완료)
//!         calibrate() → build_circuit() → schedule() → optimize()
//!         run() - 전체 파이프라인
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_engine::prelude::*;
//!
//! // Simple one-liner optimization
//! let mut optimizer = NisoOptimizer::quick(5);
//! let result = optimizer.optimize().unwrap();
//! println!("Improvement: {:.2}%", result.improvement_percent());
//! ```
//!
//! ## Using Pipeline
//!
//! ```rust
//! use niso_engine::prelude::*;
//!
//! let config = NisoConfig::default_7q()
//!     .with_noise(0.02)
//!     .with_seed(42);
//!
//! let mut pipeline = Pipeline::new(config);
//!
//! // Run stages individually
//! pipeline.calibrate().unwrap();
//! pipeline.build_circuit(0.0, 0.0).unwrap();
//! pipeline.schedule().unwrap();
//! pipeline.optimize().unwrap();
//!
//! // Or run all at once
//! // let result = pipeline.run().unwrap();
//! ```
//!
//! ## Configuration Modes
//!
//! ```rust
//! use niso_engine::prelude::*;
//!
//! // Full optimization (default)
//! let full = NisoConfig::default_7q();
//!
//! // Quick optimization (fewer iterations)
//! let quick = NisoConfig::quick(5);
//!
//! // Benchmark mode (fixed seed)
//! let bench = NisoConfig::benchmark(7);
//!
//! // Ideal (no noise)
//! let ideal = NisoConfig::ideal(5);
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Configuration (Gantree: L7_Integration → NisoConfig)
pub mod config;

/// Optimizer (Gantree: L7_Integration → NisoOptimizer)
pub mod optimizer;

/// Pipeline (Gantree: L7_Integration → Pipeline)
pub mod pipeline;

// ============================================================================
// Re-exports
// ============================================================================

pub use config::{HardwareTarget, NisoConfig, OptimizationMode};
pub use optimizer::{
    CalibrationSummary, ExecutionMetrics, NisoOptimizer, OptimizationResult, ScheduleMetrics,
};
pub use pipeline::{Pipeline, PipelineStage, PipelineState};

// ============================================================================
// Prelude
// ============================================================================

/// Convenient imports for common use cases
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_engine::prelude::*;
    //! ```

    pub use crate::config::{HardwareTarget, NisoConfig, OptimizationMode};
    pub use crate::optimizer::{ExecutionMetrics, NisoOptimizer, OptimizationResult};
    pub use crate::pipeline::{Pipeline, PipelineStage};
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_quick_optimization() {
        let config = NisoConfig::quick(5).with_seed(42);
        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        assert!(result.tqqc_result.iterations > 0);
        assert!(result.metrics.total_time_ms > 0);
    }

    #[test]
    fn test_full_optimization() {
        let config = NisoConfig::default_5q()
            .with_noise(0.015)
            .with_points(5)
            .with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        assert!(result.tqqc_result.parity_final.abs() <= 1.0);
    }

    #[test]
    fn test_pipeline_full_run() {
        let config = NisoConfig::default_5q().with_points(3).with_seed(42);

        let mut pipeline = Pipeline::new(config);
        let result = pipeline.run().unwrap();

        assert!(result.schedule.is_some());
        assert!(result.calibration_summary.is_some());
    }

    #[test]
    fn test_pipeline_staged() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut pipeline = Pipeline::new(config);

        // Check stages
        assert_eq!(pipeline.stage(), PipelineStage::Initial);

        pipeline.calibrate().unwrap();
        assert_eq!(pipeline.stage(), PipelineStage::Calibrated);

        pipeline.build_circuit(0.0, 0.0).unwrap();
        assert_eq!(pipeline.stage(), PipelineStage::CircuitBuilt);

        pipeline.schedule().unwrap();
        assert_eq!(pipeline.stage(), PipelineStage::Scheduled);

        pipeline.optimize().unwrap();
        assert_eq!(pipeline.stage(), PipelineStage::Optimized);
    }

    #[test]
    fn test_benchmark_mode() {
        let config = NisoConfig::benchmark(5);

        assert_eq!(config.seed, Some(42));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ideal_mode() {
        let config = NisoConfig::ideal(5);

        assert_eq!(config.noise, 0.0);
        assert_eq!(config.hardware, HardwareTarget::Ideal);
    }

    #[test]
    fn test_hardware_targets() {
        let ibm = NisoConfig::default_5q().with_hardware(HardwareTarget::IbmSuperconducting);
        assert_eq!(ibm.gate_time_1q_ns, 35.0);

        let ion = NisoConfig::default_5q().with_hardware(HardwareTarget::TrappedIon);
        assert!(ion.gate_time_1q_ns > ibm.gate_time_1q_ns);
    }

    #[test]
    fn test_7q_optimization() {
        let config = NisoConfig::default_7q().with_points(3).with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        assert!(result.tqqc_result.iterations > 0);
    }

    #[test]
    fn test_optimization_result_methods() {
        let config = NisoConfig::quick(5).with_seed(42);
        let mut optimizer = NisoOptimizer::new(config);
        let result = optimizer.optimize().unwrap();

        // Test all result methods
        let _ = result.improvement_percent();
        let _ = result.improved();
        let _ = result.final_parity();
        let _ = result.baseline_parity();
    }

    #[test]
    fn test_config_conversions() {
        let config = NisoConfig::default_7q().with_noise(0.015);

        // To TqqcConfig
        let tqqc = config.to_tqqc_config();
        assert_eq!(tqqc.qubits, 7);
        assert_eq!(tqqc.noise, 0.015);

        // To NoiseModel
        let noise = config.to_noise_model();
        assert!(noise.validate().is_ok());

        // To GateTimes
        let times = config.to_gate_times();
        assert!(times.single_qubit_ns > 0.0);
    }

    #[test]
    fn test_calibration_workflow() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut optimizer = NisoOptimizer::new(config);

        // Calibrate
        optimizer.calibrate("test_backend").unwrap();

        // Should have calibration
        assert!(optimizer.calibration().is_some());

        // Optimize with calibration
        let result = optimizer.optimize().unwrap();
        assert!(result.calibration_summary.is_some());
    }

    #[test]
    fn test_schedule_metrics() {
        let config = NisoConfig::default_5q().with_points(2).with_seed(42);

        let mut pipeline = Pipeline::new(config);
        let result = pipeline.run().unwrap();

        let schedule = result.schedule.unwrap();
        assert!(schedule.total_duration_ns > 0.0);
        assert!(schedule.critical_depth > 0);
        assert!(schedule.parallelism > 0.0);
    }
}
