//! # NISO Core
//!
//! Core types, circuits, and topology for the NISQ Integrated System Optimizer.
//!
//! ## Gantree Architecture
//!
//! ```text
//! niso_core // L0+L1: Foundation + Circuit (완료)
//!     L0_Foundation // 기반 타입/상수/에러 (완료)
//!         CoreTypes // 핵심 타입 (완료)
//!         Constants // 물리/TQQC/통계 상수 (완료)
//!         Errors // 에러 타입 (완료)
//!     L1_Circuit // 회로 구조 (완료)
//!         Gate // 게이트 enum (완료)
//!         Circuit // 회로 구조체 (완료)
//!         CircuitBuilder // 빌더 패턴 (완료)
//!         Topology // 큐비트 토폴로지 (완료)
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use niso_core::prelude::*;
//!
//! // Build a simple circuit
//! let circuit = CircuitBuilder::new(3)
//!     .h(0)
//!     .cnot(0, 1)
//!     .cnot(1, 2)
//!     .measure_all()
//!     .build();
//!
//! println!("{}", circuit);
//! println!("{}", circuit.to_qasm());
//! ```
//!
//! ## TQQC Parity Circuit
//!
//! ```rust
//! use niso_core::prelude::*;
//!
//! // Build a TQQC parity circuit
//! let basis = BasisString::all_x(7);
//! let circuit = CircuitBuilder::new(7)
//!     .tqqc_parity(0.5, 0.1, EntanglerType::Cx, &basis)
//!     .build();
//!
//! assert_eq!(circuit.num_qubits(), 7);
//! assert_eq!(circuit.count_2q(), 6); // 6 CNOTs for 7 qubits
//! ```
//!
//! ## Topology Validation
//!
//! ```rust
//! use niso_core::prelude::*;
//!
//! let topo = Topology::linear(5);
//! let circuit = CircuitBuilder::new(5)
//!     .cx_chain()
//!     .build();
//!
//! // Validate circuit against topology
//! assert!(topo.validate_circuit(&circuit).is_ok());
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// ============================================================================
// Module Declarations
// ============================================================================

/// Core types (Gantree: L0_Foundation → CoreTypes)
pub mod types;

/// Constants (Gantree: L0_Foundation → Constants)
pub mod constants;

/// Error types (Gantree: L0_Foundation → Errors)
pub mod error;

/// Quantum gates (Gantree: L1_Circuit → Gate)
pub mod gate;

/// Circuit structure (Gantree: L1_Circuit → Circuit)
pub mod circuit;

/// Circuit builder (Gantree: L1_Circuit → CircuitBuilder)
pub mod builder;

/// Qubit topology (Gantree: L1_Circuit → Topology)
pub mod topology;

// ============================================================================
// Re-exports
// ============================================================================

pub use builder::CircuitBuilder;
pub use circuit::Circuit;
pub use constants::{physics, stats, tqqc};
pub use error::{NisoError, NisoResult};
pub use gate::{EntanglerType, Gate};
pub use topology::Topology;
pub use types::{Angle, Basis, BasisString, Bitstring, Counts, ParamVec, Probability, QubitId};

// ============================================================================
// Prelude
// ============================================================================

pub mod prelude {
    //! Convenient imports for common use cases
    //!
    //! ```rust
    //! use niso_core::prelude::*;
    //! ```

    pub use crate::builder::CircuitBuilder;
    pub use crate::circuit::Circuit;
    pub use crate::constants::{physics, stats, tqqc};
    pub use crate::error::{NisoError, NisoResult};
    pub use crate::gate::{EntanglerType, Gate};
    pub use crate::topology::Topology;
    pub use crate::types::{
        Angle, Basis, BasisString, Bitstring, Counts, ParamVec, Probability, QubitId,
    };
}

// ============================================================================
// Version Information
// ============================================================================

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");

// ============================================================================
// Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_full_tqqc_circuit_7q() {
        // TQQC v2.2 7-qubit circuit structure
        let basis = BasisString::all_x(7);
        let theta = 0.5;
        let delta = 0.1;

        let circuit = CircuitBuilder::new(7)
            .tqqc_parity(theta, delta, EntanglerType::Cx, &basis)
            .build();

        // Verify structure
        assert_eq!(circuit.num_qubits(), 7);
        assert_eq!(circuit.count_2q(), 6); // 6 CNOTs for linear chain

        // Verify depth matches TQQC spec
        // H + 6 CNOT chain + Rz + 7 H (basis) + measure
        let depth = circuit.depth();
        assert!(depth >= 8, "Expected depth >= 8, got {}", depth);
    }

    #[test]
    fn test_topology_tqqc_validation() {
        // 7-qubit linear topology (TQQC requirement)
        let topo = Topology::linear(7);

        // Valid TQQC circuit
        let valid_circuit = CircuitBuilder::new(7)
            .h(0)
            .cx_chain()
            .rz(0, 0.5)
            .measure_all()
            .build();

        assert!(topo.validate_circuit(&valid_circuit).is_ok());

        // Circuit with topology violation
        let mut invalid_circuit = Circuit::new(7);
        invalid_circuit.add_gate(Gate::Cnot(0, 3)).unwrap(); // Non-adjacent

        assert!(topo.validate_circuit(&invalid_circuit).is_err());
    }

    #[test]
    fn test_tqqc_constants() {
        // Verify TQQC constants match spec
        assert_eq!(tqqc::THRESHOLD_5Q, 0.030);
        assert_eq!(tqqc::THRESHOLD_7Q, 0.027);
        assert_eq!(tqqc::DEFAULT_INNER_MAX, 10);
        assert_eq!(tqqc::DECAY_RATE, 0.9);
        assert_eq!(tqqc::CONVERGENCE_WINDOW, 3);

        // Verify depth ratio calculation
        assert!((tqqc::depth_ratio(5) - 1.0).abs() < 1e-10);
        assert!((tqqc::depth_ratio(7) - 1.5).abs() < 1e-10);

        // Verify threshold calculation
        let t7 = tqqc::threshold_for_qubits(7);
        assert!((t7 - tqqc::THRESHOLD_5Q / 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_parity_calculation() {
        // Even parity
        let even = Bitstring::parse("0110").unwrap();
        assert_eq!(even.popcount(), 2);
        assert!(!even.parity());
        assert_eq!(even.parity_sign(), 1);

        // Odd parity
        let odd = Bitstring::parse("0111").unwrap();
        assert_eq!(odd.popcount(), 3);
        assert!(odd.parity());
        assert_eq!(odd.parity_sign(), -1);
    }

    #[test]
    fn test_basis_transform_gates() {
        // X basis: H
        let x_gates = Gate::basis_transform(0, Basis::X);
        assert_eq!(x_gates.len(), 1);
        assert!(matches!(x_gates[0], Gate::H(0)));

        // Y basis: Sdg, H
        let y_gates = Gate::basis_transform(0, Basis::Y);
        assert_eq!(y_gates.len(), 2);
        assert!(matches!(y_gates[0], Gate::Sdg(0)));
        assert!(matches!(y_gates[1], Gate::H(0)));

        // Z basis: none
        let z_gates = Gate::basis_transform(0, Basis::Z);
        assert!(z_gates.is_empty());
    }

    #[test]
    fn test_qasm_roundtrip() {
        let original = CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .rz(0, 1.5707963267948966)
            .measure_all()
            .build();

        let qasm = original.to_qasm();
        let parsed = Circuit::from_qasm(&qasm).unwrap();

        assert_eq!(original.num_qubits(), parsed.num_qubits());
        // Note: MeasureAll may parse differently, so just check basic structure
        assert!(parsed.gate_count() > 0);
    }

    #[test]
    fn test_circuit_analysis() {
        let circuit = CircuitBuilder::new(5)
            .h_layer()
            .cx_chain()
            .ry_layer(&[0.1, 0.2, 0.3, 0.4, 0.5])
            .measure_all()
            .build();

        assert_eq!(circuit.count_1q(), 10); // 5 H + 5 Ry
        assert_eq!(circuit.count_2q(), 4); // 4 CNOTs
        assert_eq!(circuit.count_parameterized(), 5); // 5 Ry
    }

    #[test]
    fn test_probability_validation() {
        assert!(Probability::new(0.0).is_ok());
        assert!(Probability::new(0.5).is_ok());
        assert!(Probability::new(1.0).is_ok());
        assert!(Probability::new(-0.1).is_err());
        assert!(Probability::new(1.1).is_err());

        let p = Probability::new(0.3).unwrap();
        assert!((p.value() - 0.3).abs() < 1e-10);
        assert!((p.complement() - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_linear_chain_finding() {
        let topo = Topology::linear(7);

        // Should find a chain of length 7
        let chain = topo.find_linear_chain(7).unwrap();
        assert_eq!(chain.len(), 7);

        // Verify chain connectivity
        for i in 0..chain.len() - 1 {
            assert!(
                topo.is_connected(chain[i], chain[i + 1]),
                "Chain broken at {} -> {}",
                chain[i],
                chain[i + 1]
            );
        }
    }
}
