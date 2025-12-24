//! # NISO Backend
//!
//! Quantum backend abstraction and execution for NISO.
//!
//! ## Gantree Architecture
//!
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_backend::prelude::*;
//! use niso_core::CircuitBuilder;
//!
//! // Create ideal simulator
//! let backend = SimulatorBackend::ideal(5).with_seed(42);
//!
//! // Create circuit
//! let circuit = CircuitBuilder::new(3)
//!     .h(0)
//!     .cnot(0, 1)
//!     .cnot(1, 2)
//!     .build();
//!
//! // Execute
//! let result = backend.execute(&circuit, 1000).unwrap();
//! println!("Parity expectation: {:.4}", result.parity_expectation());
//! ```
//!
//! ## Noisy Simulation
//!
//! ```rust
//! use niso_backend::prelude::*;
//! use niso_core::CircuitBuilder;
//!
//! // Create noisy simulator (p = 0.02)
//! let backend = SimulatorBackend::from_depol(5, 0.02)
//!     .unwrap()
//!     .with_seed(42);
//!
//! let circuit = CircuitBuilder::new(3)
//!     .h(0)
//!     .cnot(0, 1)
//!     .build();
//!
//! let result = backend.execute(&circuit, 1000).unwrap();
//! println!("P(even): {:.4}, P(odd): {:.4}", result.p_even(), result.p_odd());
//! ```

#![warn(missing_docs)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Execution types and backend trait (Gantree: L6_Backend)
pub mod execution;

/// Simulator backend (Gantree: L6_Backend ??SimulatorBackend)
pub mod simulator;

// ============================================================================
// Re-exports
// ============================================================================

pub use execution::{Backend, ExecutionMetadata, ExecutionResult};
pub use simulator::SimulatorBackend;

// ============================================================================
// Prelude
// ============================================================================

// Convenient imports below
pub mod prelude {
    //! Prelude module for convenient imports
    //!
    //! ```rust
    //! use niso_backend::prelude::*;
    //! ```

    pub use crate::execution::{Backend, ExecutionMetadata, ExecutionResult};
    pub use crate::simulator::SimulatorBackend;
}

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use niso_core::{BasisString, CircuitBuilder, EntanglerType};
    use std::f64::consts::PI;

    #[test]
    fn test_bell_state() {
        let backend = SimulatorBackend::ideal(2).with_seed(42);

        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let result = backend.execute(&circuit, 10000).unwrap();

        // Bell state: (|00??+ |11??/??
        let p00 = result.probability("00");
        let p11 = result.probability("11");

        assert!((p00 - 0.5).abs() < 0.05);
        assert!((p11 - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_ghz_state_parity() {
        // Use 4 qubits for GHZ: |0000??+ |1111??        // Both have even parity (0 ones, 4 ones)
        let backend = SimulatorBackend::ideal(4).with_seed(42);

        let circuit = CircuitBuilder::new(4).h(0).cx_chain().build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // GHZ: (|0000??+ |1111??/??
        // Both states have even parity (0 and 4 ones)
        let parity = result.parity_expectation();
        assert!(parity > 0.9, "GHZ parity should be ~1, got {}", parity);
    }

    #[test]
    fn test_tqqc_parity_circuit() {
        let backend = SimulatorBackend::ideal(7).with_seed(42);

        // TQQC parity circuit
        let basis = BasisString::all_x(7);
        let circuit = CircuitBuilder::new(7)
            .tqqc_parity(0.5, 0.0, EntanglerType::Cx, &basis)
            .build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // Should have valid parity
        let parity = result.parity_expectation();
        assert!(parity.abs() <= 1.0, "Invalid parity: {}", parity);
    }

    #[test]
    fn test_noisy_parity_degradation() {
        // Ideal
        let ideal_backend = SimulatorBackend::ideal(5).with_seed(42);
        // Noisy
        let noisy_backend = SimulatorBackend::from_depol(5, 0.02).unwrap().with_seed(42);

        let circuit = CircuitBuilder::new(5).h(0).cx_chain().build();

        let ideal_result = ideal_backend.execute(&circuit, 1000).unwrap();
        let noisy_result = noisy_backend.execute(&circuit, 1000).unwrap();

        // Noisy should have lower parity
        assert!(
            noisy_result.parity_expectation() < ideal_result.parity_expectation(),
            "Noise should degrade parity: ideal={}, noisy={}",
            ideal_result.parity_expectation(),
            noisy_result.parity_expectation()
        );
    }

    #[test]
    fn test_rotation_gates() {
        let backend = SimulatorBackend::ideal(1).with_seed(42);

        // Rx(PI) should flip |0> to |1>
        let circuit = CircuitBuilder::new(1).rx(0, PI).build();

        let result = backend.execute(&circuit, 1000).unwrap();
        assert!(result.probability("1") > 0.99);

        // Ry(PI) should flip |0> to |1>
        let circuit = CircuitBuilder::new(1).ry(0, PI).build();

        let result = backend.execute(&circuit, 1000).unwrap();
        assert!(result.probability("1") > 0.99);
    }

    #[test]
    fn test_batch_execution() {
        let backend = SimulatorBackend::ideal(3).with_seed(42);

        let circuits: Vec<_> = (0..5)
            .map(|i| CircuitBuilder::new(2).h(0).rz(0, i as f64 * 0.2).build())
            .collect();

        let results = backend.execute_batch(&circuits, 100).unwrap();

        assert_eq!(results.len(), 5);
        for result in &results {
            assert_eq!(result.shots, 100);
        }
    }

    #[test]
    fn test_noise_levels() {
        let circuit = CircuitBuilder::new(3).h(0).cnot(0, 1).cnot(1, 2).build();

        let noise_levels = [0.0, 0.01, 0.02, 0.03];
        let mut parities = Vec::new();

        for &p in &noise_levels {
            let backend = SimulatorBackend::from_depol(3, p).unwrap().with_seed(42);

            let result = backend.execute(&circuit, 1000).unwrap();
            parities.push(result.parity_expectation());
        }

        // Higher noise should give lower parity
        for i in 1..parities.len() {
            assert!(
                parities[i] <= parities[i - 1] + 0.1,
                "Parity should decrease with noise: {:?}",
                parities
            );
        }
    }

    #[test]
    fn test_p_even_p_odd_sum() {
        let backend = SimulatorBackend::from_depol(4, 0.02).unwrap().with_seed(42);

        let circuit = CircuitBuilder::new(4).h_layer().cx_chain().build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // P(even) + P(odd) should always equal 1
        let sum = result.p_even() + result.p_odd();
        assert!((sum - 1.0).abs() < 1e-10);

        // Parity expectation = P(even) - P(odd)
        let parity_from_probs = result.p_even() - result.p_odd();
        assert!((result.parity_expectation() - parity_from_probs).abs() < 1e-10);
    }
}
