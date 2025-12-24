//! # NISO Schedule
//!
//! Circuit scheduling and timing analysis for NISQ systems.
//!
//! ## Gantree Architecture
//!
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_schedule::prelude::*;
//! use niso_core::CircuitBuilder;
//! use niso_noise::GateTimes;
//!
//! let circuit = CircuitBuilder::new(3)
//!     .h(0)
//!     .h(1)
//!     .cnot(0, 1)
//!     .cnot(1, 2)
//!     .measure_all()
//!     .build();
//!
//! let times = GateTimes::default();
//! let schedule = Scheduler::compute_asap(&circuit, &times);
//!
//! println!("{}", schedule);
//! println!("Parallelism: {:.2}x", schedule.parallelism_factor());
//! ```
//!
//! ## Decoherence Analysis
//!
//! ```rust
//! use niso_schedule::prelude::*;
//! use niso_core::CircuitBuilder;
//! use niso_noise::{GateTimes, NoiseVector};
//!
//! let circuit = CircuitBuilder::new(2)
//!     .h(0)
//!     .cnot(0, 1)
//!     .build();
//!
//! let times = GateTimes::default();
//! let schedule = Scheduler::compute_asap(&circuit, &times);
//!
//! let noise_vectors = vec![
//!     NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01),
//!     NoiseVector::new(1, 100.0, 60.0, 0.001, 0.01, 0.01),
//! ];
//!
//! let decoherence = schedule.estimate_decoherence(&noise_vectors);
//! println!("Estimated decoherence error: {:.4}", decoherence);
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Scheduled gate with timing (Gantree: L4_Scheduling ??ScheduledGate)
pub mod scheduled_gate;

/// Circuit schedule (Gantree: L4_Scheduling ??CircuitSchedule)
pub mod circuit_schedule;

/// Scheduler algorithms (Gantree: L4_Scheduling ??Scheduler)
pub mod scheduler;

// ============================================================================
// Re-exports
// ============================================================================

pub use circuit_schedule::CircuitSchedule;
pub use scheduled_gate::{ScheduledGate, TimeSlot};
pub use scheduler::Scheduler;

// ============================================================================
// Prelude
// ============================================================================

// Convenient imports below
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_schedule::prelude::*;
    //! ```

    pub use crate::circuit_schedule::CircuitSchedule;
    pub use crate::scheduled_gate::{ScheduledGate, TimeSlot};
    pub use crate::scheduler::Scheduler;
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use niso_core::{BasisString, CircuitBuilder, EntanglerType};
    use niso_noise::{GateTimes, NoiseModel, NoiseVector, NoiseVectorSet};

    #[test]
    fn test_tqqc_7q_schedule() {
        // TQQC v2.2.0 7-qubit parity circuit
        let basis = BasisString::all_x(7);
        let circuit = CircuitBuilder::new(7)
            .tqqc_parity(0.5, 0.1, EntanglerType::Cx, &basis)
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        // Structure verification
        assert_eq!(schedule.num_qubits(), 7);
        assert_eq!(schedule.count_2q(), 6);

        // Timing verification
        assert!(schedule.total_duration_ns() > 0.0);

        // Parallelism - CNOTs are sequential, but H gates can be parallel
        let factor = schedule.parallelism_factor();
        assert!(factor >= 1.0);
    }

    #[test]
    fn test_schedule_with_noise() {
        let circuit = CircuitBuilder::new(5)
            .h_layer()
            .cx_chain()
            .measure_all()
            .build();

        let times = GateTimes::default();
        let model = NoiseModel::from_depol(0.02).unwrap();
        let noise_set = NoiseVectorSet::from_noise_model(5, &model);

        let schedule = Scheduler::compute_asap(&circuit, &times);
        let decoherence = schedule.estimate_decoherence(noise_set.vectors());

        // Should have some decoherence due to idle time
        assert!(decoherence >= 0.0);
    }

    #[test]
    fn test_circuit_scoring() {
        let good_circuit = CircuitBuilder::new(3).h(0).h(1).h(2).build();

        let bad_circuit = CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .cnot(0, 1)
            .cnot(1, 2)
            .build();

        let times = GateTimes::default();
        let noise_vectors = vec![
            NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01),
            NoiseVector::new(1, 100.0, 60.0, 0.001, 0.01, 0.01),
            NoiseVector::new(2, 100.0, 60.0, 0.001, 0.01, 0.01),
        ];

        let good_score = Scheduler::score_circuit(&good_circuit, &noise_vectors, &times);
        let bad_score = Scheduler::score_circuit(&bad_circuit, &noise_vectors, &times);

        // Good circuit should score higher (less noise)
        assert!(good_score > bad_score);
    }

    #[test]
    fn test_idle_time_analysis() {
        // Circuit where q1 and q2 wait for q0
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .h(0)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        let idle = schedule.idle_times();

        // q0 should have minimal idle time (always active)
        // q1 and q2 should have more idle time
        assert!(idle[1] > idle[0] || idle[2] > idle[0]);
    }

    #[test]
    fn test_critical_path() {
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .h(1)
            .h(2)
            .cnot(0, 1)
            .cnot(1, 2)
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        let critical = schedule.critical_path();

        // Critical path should include gates on the longest chain
        assert!(!critical.is_empty());
    }

    #[test]
    fn test_scheduling_efficiency() {
        // Perfect parallel circuit
        let parallel = CircuitBuilder::new(4).h(0).h(1).h(2).h(3).build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&parallel, &times);

        let efficiency = Scheduler::scheduling_efficiency(&schedule);

        // Perfect parallelism = 100% efficiency
        assert!((efficiency - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bottleneck_detection() {
        // q2 is a bottleneck (waits longest)
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        let bottleneck = Scheduler::find_bottleneck_qubit(&schedule);

        // Should identify a bottleneck
        assert!(bottleneck.is_some());
    }

    #[test]
    fn test_different_hardware_timing() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let ibm_times = GateTimes::default_ibm();
        let ion_times = GateTimes::trapped_ion();

        let ibm_schedule = Scheduler::compute_asap(&circuit, &ibm_times);
        let ion_schedule = Scheduler::compute_asap(&circuit, &ion_times);

        // Ion trap should be much slower
        assert!(ion_schedule.total_duration_ns() > ibm_schedule.total_duration_ns() * 100.0);
    }
}
