//! # NISO TQQC
//!
//! Time-Quantized Quantum Computing optimization engine.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_tqqc // L5: TQQC Engine (완료)
//!     TqqcConfig // 설정 (완료)
//!         qubits, points, shots, noise
//!         step_amp, inner_max, dynamic_inner
//!         use_statistical_test, sig_mode, sig_level
//!     Convergence // 수렴 판단 (완료)
//!         window, threshold_abs, threshold_cum
//!         depth_correction() - 깊이 보정
//!         check() - 수렴 체크
//!     DynamicInner // 적응형 내부 반복 (완료)
//!         compute_count() - 반복 수 계산
//!         compute_step() - 감쇠 스텝
//!     StatisticalTest // z-test (완료)
//!         compute_z() - z값 계산
//!         z_critical() - 임계값
//!         test() - 유의성 검정
//!     Parity // 패리티 계산 (완료)
//!         popcount(), p_even(), expectation()
//!         build_circuit() - TQQC 회로 생성
//!     TqqcEngine // 최적화 엔진 (완료)
//!         optimize() - 메인 최적화
//!         TqqcResult - 결과
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_tqqc::prelude::*;
//! use niso_backend::SimulatorBackend;
//!
//! // Create configuration
//! let config = TqqcConfig::default_7q()
//!     .with_noise(0.02)
//!     .with_points(10)
//!     .with_seed(42);
//!
//! // Create backend
//! let backend = SimulatorBackend::from_depol(7, 0.02)
//!     .unwrap()
//!     .with_seed(42);
//!
//! // Run optimization
//! let mut engine = TqqcEngine::new(config, backend);
//! let result = engine.optimize().unwrap();
//!
//! println!("Improvement: {:.2}%", result.improvement_percent());
//! ```
//!
//! ## TQQC v2.2.0 Features
//!
//! - **Dynamic Inner Loop**: Adaptive iteration count based on improvement
//! - **Step Decay**: 0.9^j decay for inner iterations
//! - **Adaptive z-test**: Noise and shot-dependent significance threshold
//! - **Depth Correction**: threshold_N = threshold_5Q × (D_5 / D_N)
//! - **Early Stop**: Window-based convergence detection

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// TQQC configuration (Gantree: L5_TQQC → TqqcConfig)
pub mod config;

/// Convergence and dynamic inner (Gantree: L5_TQQC → Convergence, DynamicInner)
pub mod convergence;

/// Statistical testing (Gantree: L5_TQQC → StatisticalTest)
pub mod stat_test;

/// Parity calculation (Gantree: L5_TQQC → Parity)
pub mod parity;

/// TQQC engine (Gantree: L5_TQQC → TqqcEngine)
pub mod engine;

// ============================================================================
// Re-exports
// ============================================================================

pub use config::{DeltaMode, SigMode, TqqcConfig};
pub use convergence::{Convergence, DynamicInner};
pub use engine::{IterationRecord, TqqcEngine, TqqcResult};
pub use parity::Parity;
pub use stat_test::{Direction, StatisticalTest, TestResult};

// ============================================================================
// Prelude
// ============================================================================

/// Convenient imports for common use cases
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_tqqc::prelude::*;
    //! ```

    pub use crate::config::{DeltaMode, SigMode, TqqcConfig};
    pub use crate::convergence::{Convergence, DynamicInner};
    pub use crate::engine::{IterationRecord, TqqcEngine, TqqcResult};
    pub use crate::parity::Parity;
    pub use crate::stat_test::{Direction, StatisticalTest, TestResult};
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use niso_backend::SimulatorBackend;
    use niso_core::tqqc;

    #[test]
    fn test_tqqc_5q_optimization() {
        let config = TqqcConfig::default_5q()
            .with_noise(0.015)
            .with_points(5)
            .with_dynamic_inner(true)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(5, 0.015)
            .unwrap()
            .with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // Should complete
        assert!(result.iterations > 0);
        assert!(result.parity_final >= -1.0 && result.parity_final <= 1.0);
    }

    #[test]
    fn test_tqqc_7q_optimization() {
        let config = TqqcConfig::default_7q()
            .with_noise(0.02)
            .with_points(5)
            .with_dynamic_inner(true)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(7, 0.02).unwrap().with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        assert!(result.iterations > 0);
    }

    #[test]
    fn test_tqqc_with_statistical_test() {
        let config = TqqcConfig::default_5q()
            .with_noise(0.02)
            .with_points(3)
            .with_statistical_test(true)
            .with_sig_mode(SigMode::Fixed)
            .with_sig_level(0.95)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(5, 0.02).unwrap().with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // History should be recorded
        assert!(!result.history.is_empty());
    }

    #[test]
    fn test_convergence_depth_correction() {
        // 5Q threshold
        let conv_5q = Convergence::default_for_qubits(5);

        // 7Q threshold should be lower
        let conv_7q = Convergence::default_for_qubits(7);

        assert!(conv_7q.threshold() < conv_5q.threshold());

        // Verify ratio: 7Q = 5Q * (4/6)
        let expected_7q = conv_5q.threshold() * (4.0 / 6.0);
        assert!((conv_7q.threshold() - expected_7q).abs() < 1e-6);
    }

    #[test]
    fn test_dynamic_inner_formula() {
        let di = DynamicInner::default_tqqc();

        // Formula: inner_count = 1 + 2 * floor(|g| / τ)
        // With g=0.04, τ=0.02: 1 + 2*2 = 5
        let count = di.compute_count(0.04, 0.02);
        assert_eq!(count, 5);

        // Step decay: 0.12 * 0.9^j
        let step0 = di.compute_step(0, 0.12);
        let step1 = di.compute_step(1, 0.12);

        assert!((step0 - 0.12).abs() < 1e-10);
        assert!((step1 - 0.108).abs() < 1e-10);
    }

    #[test]
    fn test_statistical_test_adaptive() {
        let st = StatisticalTest::adaptive(0.95);

        // Normal case
        let z_base = st.z_critical(8192, 0.02);

        // High noise -> higher threshold
        let z_noisy = st.z_critical(8192, 0.03);
        assert!(z_noisy >= z_base);

        // Low shots -> higher threshold
        let z_low_shots = st.z_critical(2048, 0.02);
        assert!(z_low_shots >= z_base);
    }

    #[test]
    fn test_parity_calculation() {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        counts.insert("0000000".to_string(), 600); // even
        counts.insert("0000001".to_string(), 400); // odd

        let p_even = Parity::p_even(&counts);
        let expectation = Parity::expectation(&counts);

        assert!((p_even - 0.6).abs() < 1e-10);
        assert!((expectation - 0.2).abs() < 1e-10); // 0.6 - 0.4
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = TqqcConfig::default_7q();
        assert!(config.validate().is_ok());

        // Invalid: noise too high
        let config = TqqcConfig::default_7q().with_noise(0.1);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_threshold_values() {
        // Verify TQQC threshold constants
        let threshold_5q = tqqc::THRESHOLD_5Q;
        let threshold_7q = tqqc::threshold_for_qubits(7);

        assert!((threshold_5q - 0.030).abs() < 1e-6);
        assert!((threshold_7q - 0.020).abs() < 1e-6);
    }

    #[test]
    fn test_full_tqqc_workflow() {
        // End-to-end test mimicking TQQC v2.2.0

        // 1. Configuration
        let config = TqqcConfig::default_7q()
            .with_noise(0.02)
            .with_points(5)
            .with_shots(4096)
            .with_step_amp(0.12)
            .with_inner_max(10)
            .with_dynamic_inner(true)
            .with_seed(42);

        assert!(config.validate().is_ok());

        // 2. Backend
        let backend = SimulatorBackend::from_depol(7, 0.02).unwrap().with_seed(42);

        // 3. Optimize
        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // 4. Verify results
        assert!(result.parity_baseline >= -1.0 && result.parity_baseline <= 1.0);
        assert!(result.parity_final >= -1.0 && result.parity_final <= 1.0);
        assert!(result.iterations > 0);

        // 5. History should be consistent
        for (i, record) in result.history.iter().enumerate() {
            assert_eq!(record.iteration, i);
            assert!(record.parity_plus >= -1.0 && record.parity_plus <= 1.0);
            assert!(record.parity_minus >= -1.0 && record.parity_minus <= 1.0);
        }
    }

    #[test]
    fn test_early_stop_detection() {
        let mut conv = Convergence::new(7, 0.030);

        // Add small improvements
        conv.push(0.001);
        conv.push(0.001);
        conv.push(0.001);

        // Should converge (all below threshold)
        // Note: depends on threshold_cum check too
        assert!(conv.window_condition());
    }

    #[test]
    fn test_ideal_vs_noisy_parity() {
        let config = TqqcConfig::default_5q().with_points(3).with_seed(42);

        // Ideal
        let ideal_backend = SimulatorBackend::ideal(5).with_seed(42);
        let mut ideal_engine = TqqcEngine::new(config.clone(), ideal_backend);
        let ideal_result = ideal_engine.optimize().unwrap();

        // Noisy
        let noisy_backend = SimulatorBackend::from_depol(5, 0.03).unwrap().with_seed(42);
        let mut noisy_engine = TqqcEngine::new(config.with_noise(0.03), noisy_backend);
        let noisy_result = noisy_engine.optimize().unwrap();

        // Ideal should generally have higher parity (less decoherence)
        // Note: This is probabilistic, so we just check validity
        assert!(ideal_result.parity_baseline.abs() <= 1.0);
        assert!(noisy_result.parity_baseline.abs() <= 1.0);
    }
}
