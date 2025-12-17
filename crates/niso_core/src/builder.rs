//! Circuit builder for NISO
//!
//! Gantree: L1_Circuit → CircuitBuilder
//!
//! Provides a fluent builder pattern for constructing quantum circuits
//! with convenient methods for common operations.

use crate::circuit::Circuit;
use crate::error::NisoResult;
use crate::gate::{EntanglerType, Gate};
use crate::types::{Angle, Basis, BasisString, QubitId};

/// Fluent circuit builder (consuming self pattern)
/// Gantree: CircuitBuilder // 빌더 패턴
pub struct CircuitBuilder {
    /// Internal circuit being built
    /// Gantree: circuit: Circuit // 내부 회로
    circuit: Circuit,
}

impl CircuitBuilder {
    // ========================================================================
    // Constructor
    // ========================================================================

    /// Create a new circuit builder
    /// Gantree: new(n) -> Self // 생성자
    pub fn new(num_qubits: usize) -> Self {
        Self {
            circuit: Circuit::new(num_qubits),
        }
    }

    /// Create with circuit name
    pub fn with_name(num_qubits: usize, name: impl Into<String>) -> Self {
        Self {
            circuit: Circuit::with_name(num_qubits, name),
        }
    }

    // ========================================================================
    // Single-Qubit Gates (Non-Parameterized)
    // ========================================================================

    /// Add Hadamard gate
    /// Gantree: h(self, q) -> Self // H 추가
    pub fn h(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::H(qubit));
        self
    }

    /// Add Pauli-X gate
    /// Gantree: x(self, q) -> Self // X 추가
    pub fn x(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::X(qubit));
        self
    }

    /// Add Pauli-Y gate
    /// Gantree: y(self, q) -> Self // Y 추가
    pub fn y(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Y(qubit));
        self
    }

    /// Add Pauli-Z gate
    /// Gantree: z(self, q) -> Self // Z 추가
    pub fn z(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Z(qubit));
        self
    }

    /// Add S gate
    /// Gantree: s(self, q) -> Self // S 추가
    pub fn s(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::S(qubit));
        self
    }

    /// Add S-dagger gate
    /// Gantree: sdg(self, q) -> Self // Sdg 추가
    pub fn sdg(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Sdg(qubit));
        self
    }

    /// Add T gate
    pub fn t(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::T(qubit));
        self
    }

    /// Add T-dagger gate
    pub fn tdg(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Tdg(qubit));
        self
    }

    /// Add SX gate
    pub fn sx(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Sx(qubit));
        self
    }

    /// Add identity gate
    pub fn id(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Id(qubit));
        self
    }

    // ========================================================================
    // Single-Qubit Parameterized Gates
    // ========================================================================

    /// Add Rx rotation
    /// Gantree: rx(self, q, a) -> Self // Rx 추가
    pub fn rx(mut self, qubit: QubitId, angle: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::Rx(qubit, angle));
        self
    }

    /// Add Ry rotation
    /// Gantree: ry(self, q, a) -> Self // Ry 추가
    pub fn ry(mut self, qubit: QubitId, angle: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::Ry(qubit, angle));
        self
    }

    /// Add Rz rotation
    /// Gantree: rz(self, q, a) -> Self // Rz 추가
    pub fn rz(mut self, qubit: QubitId, angle: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::Rz(qubit, angle));
        self
    }

    /// Add U gate (general single-qubit)
    pub fn u(mut self, qubit: QubitId, theta: Angle, phi: Angle, lambda: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::U(qubit, theta, phi, lambda));
        self
    }

    /// Add phase gate
    pub fn p(mut self, qubit: QubitId, lambda: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::P(qubit, lambda));
        self
    }

    // ========================================================================
    // Two-Qubit Gates
    // ========================================================================

    /// Add CNOT gate
    /// Gantree: cnot(self, c, t) -> Self // CNOT 추가
    pub fn cnot(mut self, control: QubitId, target: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Cnot(control, target));
        self
    }

    /// Alias for cnot
    pub fn cx(self, control: QubitId, target: QubitId) -> Self {
        self.cnot(control, target)
    }

    /// Add CZ gate
    /// Gantree: cz(self, c, t) -> Self // CZ 추가
    pub fn cz(mut self, control: QubitId, target: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Cz(control, target));
        self
    }

    /// Add CY gate
    pub fn cy(mut self, control: QubitId, target: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Cy(control, target));
        self
    }

    /// Add SWAP gate
    pub fn swap(mut self, qubit1: QubitId, qubit2: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Swap(qubit1, qubit2));
        self
    }

    /// Add CRZ gate
    pub fn crz(mut self, control: QubitId, target: QubitId, angle: Angle) -> Self {
        let _ = self.circuit.add_gate(Gate::Crz(control, target, angle));
        self
    }

    /// Add ECR gate (IBM native)
    pub fn ecr(mut self, control: QubitId, target: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Ecr(control, target));
        self
    }

    // ========================================================================
    // Three-Qubit Gates
    // ========================================================================

    /// Add Toffoli (CCX) gate
    pub fn ccx(mut self, c1: QubitId, c2: QubitId, target: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Ccx(c1, c2, target));
        self
    }

    /// Add Fredkin (CSWAP) gate
    pub fn cswap(mut self, control: QubitId, t1: QubitId, t2: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Cswap(control, t1, t2));
        self
    }

    // ========================================================================
    // Measurement and Control
    // ========================================================================

    /// Add measurement on single qubit
    /// Gantree: measure(self, q) -> Self // 측정 추가
    pub fn measure(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Measure(qubit));
        self
    }

    /// Add measurement on all qubits
    /// Gantree: measure_all(self) -> Self // 전체 측정
    pub fn measure_all(mut self) -> Self {
        let _ = self.circuit.add_gate(Gate::MeasureAll);
        self
    }

    /// Add barrier
    /// Gantree: barrier(self) -> Self // 배리어
    pub fn barrier(mut self) -> Self {
        let qubits: Vec<QubitId> = (0..self.circuit.num_qubits()).collect();
        let _ = self.circuit.add_gate(Gate::Barrier(qubits));
        self
    }

    /// Add barrier on specific qubits
    pub fn barrier_on(mut self, qubits: Vec<QubitId>) -> Self {
        let _ = self.circuit.add_gate(Gate::Barrier(qubits));
        self
    }

    /// Add reset
    pub fn reset(mut self, qubit: QubitId) -> Self {
        let _ = self.circuit.add_gate(Gate::Reset(qubit));
        self
    }

    // ========================================================================
    // Layer Operations (TQQC-specific)
    // ========================================================================

    /// Add Ry rotation layer on all qubits
    /// Gantree: ry_layer(self, angles) -> Self // Ry 레이어
    pub fn ry_layer(mut self, angles: &[Angle]) -> Self {
        let n = self.circuit.num_qubits().min(angles.len());
        for i in 0..n {
            let _ = self.circuit.add_gate(Gate::Ry(i, angles[i]));
        }
        self
    }

    /// Add Rz rotation layer on all qubits
    /// Gantree: rz_layer(self, angles) -> Self // Rz 레이어
    pub fn rz_layer(mut self, angles: &[Angle]) -> Self {
        let n = self.circuit.num_qubits().min(angles.len());
        for i in 0..n {
            let _ = self.circuit.add_gate(Gate::Rz(i, angles[i]));
        }
        self
    }

    /// Add Hadamard layer on all qubits
    pub fn h_layer(mut self) -> Self {
        for i in 0..self.circuit.num_qubits() {
            let _ = self.circuit.add_gate(Gate::H(i));
        }
        self
    }

    /// Add entangler chain (linear connectivity)
    /// Gantree: entangler_chain(self, typ) -> Self // 엔탱글러 체인
    pub fn entangler_chain(mut self, entangler: EntanglerType) -> Self {
        let n = self.circuit.num_qubits();
        for i in 0..n.saturating_sub(1) {
            let gate = entangler.gate(i, i + 1);
            let _ = self.circuit.add_gate(gate);
        }
        self
    }

    /// Add CX chain (convenience method)
    pub fn cx_chain(self) -> Self {
        self.entangler_chain(EntanglerType::Cx)
    }

    /// Add CZ chain (convenience method)
    pub fn cz_chain(self) -> Self {
        self.entangler_chain(EntanglerType::Cz)
    }

    /// Apply basis transformation on all qubits
    /// Gantree: apply_basis(self, basis) -> Self // 기저 변환
    pub fn apply_basis(mut self, basis: &BasisString) -> Self {
        for (i, b) in basis.iter().enumerate() {
            if i >= self.circuit.num_qubits() {
                break;
            }
            let gates = Gate::basis_transform(i, *b);
            for gate in gates {
                let _ = self.circuit.add_gate(gate);
            }
        }
        self
    }

    /// Apply single basis to all qubits
    pub fn apply_uniform_basis(mut self, basis: Basis) -> Self {
        for i in 0..self.circuit.num_qubits() {
            let gates = Gate::basis_transform(i, basis);
            for gate in gates {
                let _ = self.circuit.add_gate(gate);
            }
        }
        self
    }

    // ========================================================================
    // TQQC Parity Circuit Helper
    // ========================================================================

    /// Build a TQQC parity circuit
    ///
    /// Structure:
    /// 1. H on qubit 0
    /// 2. Entangler chain (N-1 gates)
    /// 3. Rz(theta + delta) on qubit 0
    /// 4. Basis transformation
    /// 5. Measurement
    pub fn tqqc_parity(
        self,
        theta: Angle,
        delta: Angle,
        entangler: EntanglerType,
        basis: &BasisString,
    ) -> Self {
        self.h(0)
            .entangler_chain(entangler)
            .rz(0, theta + delta)
            .apply_basis(basis)
            .measure_all()
    }

    // ========================================================================
    // VQE/QAOA Helpers
    // ========================================================================

    /// Hardware-efficient ansatz layer
    pub fn hea_layer(mut self, params: &[Angle], layer: usize) -> Self {
        let n = self.circuit.num_qubits();
        let offset = layer * n * 2;

        // Ry layer
        for i in 0..n {
            if offset + i < params.len() {
                let _ = self.circuit.add_gate(Gate::Ry(i, params[offset + i]));
            }
        }

        // Rz layer
        for i in 0..n {
            if offset + n + i < params.len() {
                let _ = self.circuit.add_gate(Gate::Rz(i, params[offset + n + i]));
            }
        }

        // Entangler
        self.cx_chain()
    }

    /// QAOA mixer layer
    pub fn qaoa_mixer(mut self, beta: Angle) -> Self {
        for i in 0..self.circuit.num_qubits() {
            let _ = self.circuit.add_gate(Gate::Rx(i, 2.0 * beta));
        }
        self
    }

    // ========================================================================
    // Build
    // ========================================================================

    /// Build and return the circuit
    /// Gantree: build(self) -> Circuit // 빌드
    pub fn build(self) -> Circuit {
        self.circuit
    }

    /// Build with validation
    pub fn build_validated(self) -> NisoResult<Circuit> {
        if self.circuit.is_empty() {
            return Err(crate::error::NisoError::EmptyCircuit);
        }
        Ok(self.circuit)
    }

    /// Get reference to current circuit state
    pub fn circuit(&self) -> &Circuit {
        &self.circuit
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.circuit.num_qubits()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .measure_all()
            .build();

        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.gate_count(), 4);
    }

    #[test]
    fn test_builder_chain() {
        let circuit = CircuitBuilder::new(5).h(0).cx_chain().measure_all().build();

        // H + 4 CNOTs + MeasureAll
        assert_eq!(circuit.gate_count(), 6);
        assert_eq!(circuit.count_2q(), 4);
    }

    #[test]
    fn test_builder_layers() {
        let angles = vec![0.1, 0.2, 0.3];
        let circuit = CircuitBuilder::new(3).ry_layer(&angles).build();

        assert_eq!(circuit.count_parameterized(), 3);
    }

    #[test]
    fn test_builder_tqqc_parity() {
        let basis = BasisString::all_x(5);
        let circuit = CircuitBuilder::new(5)
            .tqqc_parity(0.5, 0.1, EntanglerType::Cx, &basis)
            .build();

        // H(0) + 4 CNOTs + Rz + 5 H (basis) + MeasureAll
        assert!(circuit.gate_count() > 0);
        assert_eq!(circuit.count_2q(), 4);
    }

    #[test]
    fn test_builder_basis_transform() {
        let basis = BasisString::from_str("XYZ").unwrap();
        let circuit = CircuitBuilder::new(3).apply_basis(&basis).build();

        // X: H (1 gate)
        // Y: Sdg + H (2 gates)
        // Z: none
        // Total: 3 gates
        assert_eq!(circuit.gate_count(), 3);
    }

    #[test]
    fn test_builder_hea() {
        let params = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let circuit = CircuitBuilder::new(3).hea_layer(&params, 0).build();

        // 3 Ry + 3 Rz + 2 CX
        assert_eq!(circuit.count_parameterized(), 6);
        assert_eq!(circuit.count_2q(), 2);
    }
}
