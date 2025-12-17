//! # NISO Noise
//!
//! Noise models and gate timing for NISQ systems.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_noise // L2: Noise Model (완료)
//!     NoiseModel // 통합 노이즈 (완료)
//!         t1_us, t2_us, gate_error_1q, gate_error_2q
//!         readout_error, crosstalk
//!         new(), ideal(), ibm_typical(), from_depol()
//!         effective_depol(), estimate_circuit_fidelity()
//!     NoiseVector // 큐비트별 노이즈 (완료)
//!         per-qubit T1, T2, errors
//!         estimate_gate_fidelity(), estimate_decoherence()
//!         quality_score(), is_tqqc_usable()
//!     NoiseVectorSet // 노이즈 벡터 집합 (완료)
//!         avg_t1(), avg_t2(), best_qubits()
//!     GateTimes // 게이트 시간 (완료)
//!         single_qubit_ns, two_qubit_ns, measurement_ns
//!         default_ibm(), trapped_ion(), neutral_atom()
//!         gate_duration(), circuit_duration_asap()
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_noise::prelude::*;
//!
//! // Create noise model from effective depolarizing rate
//! let model = NoiseModel::from_depol(0.02).unwrap();
//! println!("Effective depol: {:.4}", model.effective_depol());
//!
//! // Check TQQC validity
//! assert!(model.is_tqqc_valid(7));
//!
//! // Estimate circuit fidelity
//! let fidelity = model.estimate_circuit_fidelity(10, 6, 7, 1.0);
//! println!("Estimated fidelity: {:.4}", fidelity);
//! ```
//!
//! ## Gate Timing
//!
//! ```rust
//! use niso_noise::prelude::*;
//! use niso_core::CircuitBuilder;
//!
//! let times = GateTimes::default();
//! let circuit = CircuitBuilder::new(7)
//!     .h(0)
//!     .cx_chain()
//!     .measure_all()
//!     .build();
//!
//! let (duration, _) = times.circuit_duration_asap(&circuit);
//! println!("Circuit duration: {:.0} ns", duration);
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Unified noise model (Gantree: L2_Noise → NoiseModel)
pub mod noise_model;

/// Per-qubit noise vectors (Gantree: L2_Noise → NoiseVector)
pub mod noise_vector;

/// Gate timing configuration (Gantree: L2_Noise → GateTimes)
pub mod gate_times;

// ============================================================================
// Re-exports
// ============================================================================

pub use gate_times::GateTimes;
pub use noise_model::NoiseModel;
pub use noise_vector::{NoiseVector, NoiseVectorSet};

// ============================================================================
// Prelude
// ============================================================================

/// Convenient imports for common use cases
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_noise::prelude::*;
    //! ```

    pub use crate::gate_times::GateTimes;
    pub use crate::noise_model::NoiseModel;
    pub use crate::noise_vector::{NoiseVector, NoiseVectorSet};
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use niso_core::{tqqc, CircuitBuilder};

    #[test]
    fn test_tqqc_noise_model() {
        // TQQC v2.2.0: p = 0.02, 7-qubit
        let model = NoiseModel::from_depol(0.02).unwrap();

        // effective_depol = gate_error_1q = 0.02
        assert!((model.effective_depol() - 0.02).abs() < 1e-10);

        // Should be within recommended range
        assert!(model.is_recommended());

        // T2 constraint
        assert!(model.t2_us() <= 2.0 * model.t1_us());
    }

    #[test]
    fn test_noise_model_to_vector() {
        let model = NoiseModel::ibm_typical();
        let vectors = NoiseVectorSet::from_noise_model(7, &model);

        assert_eq!(vectors.num_qubits(), 7);

        // Average should match model
        assert!((vectors.avg_t1() - model.t1_us()).abs() < 1e-10);
        assert!((vectors.avg_t2() - model.t2_us()).abs() < 1e-10);
    }

    #[test]
    fn test_circuit_timing() {
        let times = GateTimes::default();

        // TQQC 7-qubit parity circuit
        let circuit = CircuitBuilder::new(7)
            .h(0)
            .cx_chain() // 6 CNOTs
            .rz(0, 0.5)
            .measure_all()
            .build();

        let (duration, qubit_times) = times.circuit_duration_asap(&circuit);

        // Should have positive duration
        assert!(duration > 0.0);

        // All qubits should have some activity
        assert_eq!(qubit_times.len(), 7);
    }

    #[test]
    fn test_fidelity_estimation() {
        let model = NoiseModel::new(100.0, 60.0, 0.001, 0.01, 0.01).unwrap();

        // Simple circuit: 5 1Q gates, 2 2Q gates, 3 measurements
        let fidelity = model.estimate_circuit_fidelity(5, 2, 3, 0.1);

        // Should be positive and less than 1
        assert!(fidelity > 0.0);
        assert!(fidelity < 1.0);

        // For low noise, expect reasonable fidelity
        assert!(fidelity > 0.9, "Fidelity too low: {}", fidelity);
    }

    #[test]
    fn test_decoherence_impact() {
        let nv = NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01);

        // Short circuit: minimal decoherence
        let short_error = nv.estimate_decoherence(0.1); // 100 ns

        // Long circuit: significant decoherence
        let long_error = nv.estimate_decoherence(60.0); // 60 μs = T2

        assert!(short_error < long_error);
        assert!(short_error < 0.01);
        assert!(long_error > 0.5);
    }

    #[test]
    fn test_best_qubits_selection() {
        // Simulate heterogeneous device
        let vectors = vec![
            NoiseVector::new(0, 80.0, 50.0, 0.002, 0.02, 0.02), // OK
            NoiseVector::new(1, 150.0, 100.0, 0.0005, 0.008, 0.008), // Best
            NoiseVector::new(2, 40.0, 25.0, 0.005, 0.04, 0.04), // Poor
            NoiseVector::new(3, 120.0, 80.0, 0.001, 0.01, 0.01), // Good
            NoiseVector::new(4, 60.0, 35.0, 0.003, 0.03, 0.03), // Medium
        ];
        let set = NoiseVectorSet::new(vectors);

        let best_3 = set.best_qubits(3);

        // Best qubits should include qubit 1 (lowest errors, highest T1/T2)
        assert!(
            best_3.contains(&1),
            "Best qubit 1 not in top 3: {:?}",
            best_3
        );

        // Qubit 3 should also be included (second best)
        assert!(
            best_3.contains(&3),
            "Good qubit 3 not in top 3: {:?}",
            best_3
        );
    }

    #[test]
    fn test_different_hardware_platforms() {
        let ibm = GateTimes::default_ibm();
        let ion = GateTimes::trapped_ion();

        // Ion traps are much slower
        assert!(ion.single_qubit_ns > ibm.single_qubit_ns * 100.0);

        // But similar ratios
        let ibm_ratio = ibm.two_qubit_ns / ibm.single_qubit_ns;
        let ion_ratio = ion.two_qubit_ns / ion.single_qubit_ns;

        // Both should have 2Q gates slower than 1Q
        assert!(ibm_ratio > 1.0);
        assert!(ion_ratio > 1.0);
    }

    #[test]
    fn test_critical_point_validation() {
        // TQQC threshold for 7Q: ~0.020 (0.030 / 1.5)
        let threshold_7q = tqqc::threshold_for_qubits(7);

        // At threshold level
        let at_threshold = NoiseModel::from_depol(threshold_7q).unwrap();
        assert!(at_threshold.is_tqqc_valid(7));

        // Above threshold should fail
        let above_threshold = NoiseModel::from_depol(0.025).unwrap();
        assert!(!above_threshold.is_tqqc_valid(7));

        // Below threshold should pass
        let below_threshold = NoiseModel::from_depol(0.015).unwrap();
        assert!(below_threshold.is_tqqc_valid(7));
    }
}
