//! Quantum gate definitions for NISO
//!
//! Gantree: L1_Circuit → Gate
//!
//! Comprehensive gate enum supporting all standard gates
//! for NISQ circuit construction and TQQC optimization.

use crate::types::{Angle, QubitId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Quantum gate enumeration
/// Gantree: Gate // 게이트 enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gate {
    // ========================================================================
    // Single-Qubit Gates (Non-Parameterized)
    // ========================================================================
    /// Hadamard gate
    /// Gantree: H(QubitId) // 하다마드
    H(QubitId),

    /// Pauli-X gate (NOT)
    /// Gantree: X(QubitId) // 파울리 X
    X(QubitId),

    /// Pauli-Y gate
    /// Gantree: Y(QubitId) // 파울리 Y
    Y(QubitId),

    /// Pauli-Z gate
    /// Gantree: Z(QubitId) // 파울리 Z
    Z(QubitId),

    /// S gate (sqrt(Z))
    /// Gantree: S(QubitId) // S 게이트
    S(QubitId),

    /// S-dagger gate (inverse of S)
    /// Gantree: Sdg(QubitId) // S†
    Sdg(QubitId),

    /// T gate (fourth root of Z)
    /// Gantree: T(QubitId) // T 게이트
    T(QubitId),

    /// T-dagger gate (inverse of T)
    /// Gantree: Tdg(QubitId) // T†
    Tdg(QubitId),

    /// SX gate (sqrt(X))
    Sx(QubitId),

    /// SX-dagger gate
    Sxdg(QubitId),

    /// Identity gate (for padding/timing)
    Id(QubitId),

    // ========================================================================
    // Single-Qubit Parameterized Rotation Gates
    // ========================================================================
    /// Rotation around X-axis
    /// Gantree: Rx(QubitId, Angle) // X 회전
    Rx(QubitId, Angle),

    /// Rotation around Y-axis
    /// Gantree: Ry(QubitId, Angle) // Y 회전
    Ry(QubitId, Angle),

    /// Rotation around Z-axis
    /// Gantree: Rz(QubitId, Angle) // Z 회전
    Rz(QubitId, Angle),

    /// General single-qubit rotation U(θ, φ, λ)
    U(QubitId, Angle, Angle, Angle),

    /// Phase gate P(λ) = diag(1, e^{iλ})
    P(QubitId, Angle),

    // ========================================================================
    // Two-Qubit Gates
    // ========================================================================
    /// Controlled-NOT (CX)
    /// Gantree: CNOT(QubitId, QubitId) // ctrl, tgt
    Cnot(QubitId, QubitId),

    /// Controlled-Z
    /// Gantree: CZ(QubitId, QubitId) // 제어-Z
    Cz(QubitId, QubitId),

    /// Controlled-Y
    Cy(QubitId, QubitId),

    /// SWAP gate
    /// Gantree: SWAP(QubitId, QubitId) // 스왑
    Swap(QubitId, QubitId),

    /// iSWAP gate
    ISwap(QubitId, QubitId),

    /// Controlled-Phase (CRz)
    Crz(QubitId, QubitId, Angle),

    /// Controlled-Rx
    Crx(QubitId, QubitId, Angle),

    /// Controlled-Ry
    Cry(QubitId, QubitId, Angle),

    /// ECR gate (Echoed Cross-Resonance, IBM native)
    Ecr(QubitId, QubitId),

    // ========================================================================
    // Three-Qubit Gates
    // ========================================================================
    /// Toffoli (CCX)
    Ccx(QubitId, QubitId, QubitId),

    /// Controlled-SWAP (Fredkin)
    Cswap(QubitId, QubitId, QubitId),

    // ========================================================================
    // Measurement and Control
    // ========================================================================
    /// Single qubit measurement
    /// Gantree: Measure(QubitId) // 단일 측정
    Measure(QubitId),

    /// Measure all qubits (convenience)
    /// Gantree: MeasureAll // 전체 측정
    MeasureAll,

    /// Barrier (for timing/visualization)
    /// Gantree: Barrier // 배리어
    Barrier(Vec<QubitId>),

    /// Reset qubit to |0⟩
    Reset(QubitId),
}

impl Gate {
    // ========================================================================
    // Gate Properties
    // ========================================================================

    /// Get qubits involved in this gate
    /// Gantree: qubits(&self) -> Vec<QubitId> // 관련 큐비트
    pub fn qubits(&self) -> Vec<QubitId> {
        match self {
            // Single-qubit gates
            Gate::H(q)
            | Gate::X(q)
            | Gate::Y(q)
            | Gate::Z(q)
            | Gate::S(q)
            | Gate::Sdg(q)
            | Gate::T(q)
            | Gate::Tdg(q)
            | Gate::Sx(q)
            | Gate::Sxdg(q)
            | Gate::Id(q)
            | Gate::Rx(q, _)
            | Gate::Ry(q, _)
            | Gate::Rz(q, _)
            | Gate::U(q, _, _, _)
            | Gate::P(q, _)
            | Gate::Measure(q)
            | Gate::Reset(q) => vec![*q],

            // Two-qubit gates
            Gate::Cnot(c, t)
            | Gate::Cz(c, t)
            | Gate::Cy(c, t)
            | Gate::Swap(c, t)
            | Gate::ISwap(c, t)
            | Gate::Ecr(c, t)
            | Gate::Crz(c, t, _)
            | Gate::Crx(c, t, _)
            | Gate::Cry(c, t, _) => vec![*c, *t],

            // Three-qubit gates
            Gate::Ccx(c1, c2, t) | Gate::Cswap(c1, c2, t) => vec![*c1, *c2, *t],

            // Special
            Gate::MeasureAll => vec![], // Applied to all qubits
            Gate::Barrier(qs) => qs.clone(),
        }
    }

    /// Check if gate is single-qubit
    /// Gantree: is_single_qubit(&self) -> bool // 1Q 판별
    pub fn is_single_qubit(&self) -> bool {
        matches!(
            self,
            Gate::H(_)
                | Gate::X(_)
                | Gate::Y(_)
                | Gate::Z(_)
                | Gate::S(_)
                | Gate::Sdg(_)
                | Gate::T(_)
                | Gate::Tdg(_)
                | Gate::Sx(_)
                | Gate::Sxdg(_)
                | Gate::Id(_)
                | Gate::Rx(_, _)
                | Gate::Ry(_, _)
                | Gate::Rz(_, _)
                | Gate::U(_, _, _, _)
                | Gate::P(_, _)
        )
    }

    /// Check if gate is two-qubit
    /// Gantree: is_two_qubit(&self) -> bool // 2Q 판별
    pub fn is_two_qubit(&self) -> bool {
        matches!(
            self,
            Gate::Cnot(_, _)
                | Gate::Cz(_, _)
                | Gate::Cy(_, _)
                | Gate::Swap(_, _)
                | Gate::ISwap(_, _)
                | Gate::Ecr(_, _)
                | Gate::Crz(_, _, _)
                | Gate::Crx(_, _, _)
                | Gate::Cry(_, _, _)
        )
    }

    /// Check if gate is three-qubit
    pub fn is_three_qubit(&self) -> bool {
        matches!(self, Gate::Ccx(_, _, _) | Gate::Cswap(_, _, _))
    }

    /// Check if gate is parameterized
    /// Gantree: is_parameterized(&self) -> bool // 파라미터 여부
    pub fn is_parameterized(&self) -> bool {
        matches!(
            self,
            Gate::Rx(_, _)
                | Gate::Ry(_, _)
                | Gate::Rz(_, _)
                | Gate::U(_, _, _, _)
                | Gate::P(_, _)
                | Gate::Crz(_, _, _)
                | Gate::Crx(_, _, _)
                | Gate::Cry(_, _, _)
        )
    }

    /// Check if gate is measurement
    pub fn is_measurement(&self) -> bool {
        matches!(self, Gate::Measure(_) | Gate::MeasureAll)
    }

    /// Check if gate is a barrier
    pub fn is_barrier(&self) -> bool {
        matches!(self, Gate::Barrier(_))
    }

    /// Get gate name
    pub fn name(&self) -> &'static str {
        match self {
            Gate::H(_) => "h",
            Gate::X(_) => "x",
            Gate::Y(_) => "y",
            Gate::Z(_) => "z",
            Gate::S(_) => "s",
            Gate::Sdg(_) => "sdg",
            Gate::T(_) => "t",
            Gate::Tdg(_) => "tdg",
            Gate::Sx(_) => "sx",
            Gate::Sxdg(_) => "sxdg",
            Gate::Id(_) => "id",
            Gate::Rx(_, _) => "rx",
            Gate::Ry(_, _) => "ry",
            Gate::Rz(_, _) => "rz",
            Gate::U(_, _, _, _) => "u",
            Gate::P(_, _) => "p",
            Gate::Cnot(_, _) => "cx",
            Gate::Cz(_, _) => "cz",
            Gate::Cy(_, _) => "cy",
            Gate::Swap(_, _) => "swap",
            Gate::ISwap(_, _) => "iswap",
            Gate::Ecr(_, _) => "ecr",
            Gate::Crz(_, _, _) => "crz",
            Gate::Crx(_, _, _) => "crx",
            Gate::Cry(_, _, _) => "cry",
            Gate::Ccx(_, _, _) => "ccx",
            Gate::Cswap(_, _, _) => "cswap",
            Gate::Measure(_) => "measure",
            Gate::MeasureAll => "measure",
            Gate::Barrier(_) => "barrier",
            Gate::Reset(_) => "reset",
        }
    }

    /// Convert to OpenQASM 2.0 string
    /// Gantree: to_qasm(&self) -> String // QASM 변환
    pub fn to_qasm(&self) -> String {
        match self {
            // Single-qubit non-parameterized
            Gate::H(q) => format!("h q[{}];", q),
            Gate::X(q) => format!("x q[{}];", q),
            Gate::Y(q) => format!("y q[{}];", q),
            Gate::Z(q) => format!("z q[{}];", q),
            Gate::S(q) => format!("s q[{}];", q),
            Gate::Sdg(q) => format!("sdg q[{}];", q),
            Gate::T(q) => format!("t q[{}];", q),
            Gate::Tdg(q) => format!("tdg q[{}];", q),
            Gate::Sx(q) => format!("sx q[{}];", q),
            Gate::Sxdg(q) => format!("sxdg q[{}];", q),
            Gate::Id(q) => format!("id q[{}];", q),

            // Single-qubit parameterized
            Gate::Rx(q, theta) => format!("rx({}) q[{}];", theta, q),
            Gate::Ry(q, theta) => format!("ry({}) q[{}];", theta, q),
            Gate::Rz(q, theta) => format!("rz({}) q[{}];", theta, q),
            Gate::U(q, theta, phi, lambda) => {
                format!("u({},{},{}) q[{}];", theta, phi, lambda, q)
            }
            Gate::P(q, lambda) => format!("p({}) q[{}];", lambda, q),

            // Two-qubit
            Gate::Cnot(c, t) => format!("cx q[{}],q[{}];", c, t),
            Gate::Cz(c, t) => format!("cz q[{}],q[{}];", c, t),
            Gate::Cy(c, t) => format!("cy q[{}],q[{}];", c, t),
            Gate::Swap(a, b) => format!("swap q[{}],q[{}];", a, b),
            Gate::ISwap(a, b) => format!("iswap q[{}],q[{}];", a, b),
            Gate::Ecr(c, t) => format!("ecr q[{}],q[{}];", c, t),
            Gate::Crz(c, t, theta) => format!("crz({}) q[{}],q[{}];", theta, c, t),
            Gate::Crx(c, t, theta) => format!("crx({}) q[{}],q[{}];", theta, c, t),
            Gate::Cry(c, t, theta) => format!("cry({}) q[{}],q[{}];", theta, c, t),

            // Three-qubit
            Gate::Ccx(c1, c2, t) => format!("ccx q[{}],q[{}],q[{}];", c1, c2, t),
            Gate::Cswap(c, a, b) => format!("cswap q[{}],q[{}],q[{}];", c, a, b),

            // Measurement and control
            Gate::Measure(q) => format!("measure q[{}] -> c[{}];", q, q),
            Gate::MeasureAll => "measure q -> c;".to_string(),
            Gate::Barrier(qs) => {
                if qs.is_empty() {
                    "barrier q;".to_string()
                } else {
                    let qubits: Vec<String> = qs.iter().map(|q| format!("q[{}]", q)).collect();
                    format!("barrier {};", qubits.join(","))
                }
            }
            Gate::Reset(q) => format!("reset q[{}];", q),
        }
    }

    /// Get approximate gate time in nanoseconds
    pub fn gate_time_ns(&self) -> f64 {
        use crate::constants::physics::gate_times_s;

        let time_s = match self {
            Gate::H(_) => gate_times_s::H,
            Gate::X(_) => gate_times_s::X,
            Gate::Y(_) => gate_times_s::Y,
            Gate::Z(_) => gate_times_s::Z,
            Gate::S(_) => gate_times_s::S,
            Gate::Sdg(_) => gate_times_s::SDG,
            Gate::T(_) => gate_times_s::T,
            Gate::Tdg(_) => gate_times_s::TDG,
            Gate::Sx(_) | Gate::Sxdg(_) => gate_times_s::SX,
            Gate::Id(_) => 0.0,
            Gate::Rx(_, _) => gate_times_s::RX,
            Gate::Ry(_, _) => gate_times_s::RY,
            Gate::Rz(_, _) => gate_times_s::RZ,
            Gate::U(_, _, _, _) => gate_times_s::RX * 3.0, // Approximate
            Gate::P(_, _) => gate_times_s::RZ,
            Gate::Cnot(_, _) => gate_times_s::CX,
            Gate::Cz(_, _) => gate_times_s::CZ,
            Gate::Cy(_, _) => gate_times_s::CX, // Similar to CX
            Gate::Swap(_, _) => gate_times_s::SWAP,
            Gate::ISwap(_, _) => gate_times_s::SWAP,
            Gate::Ecr(_, _) => gate_times_s::CX,
            Gate::Crz(_, _, _) | Gate::Crx(_, _, _) | Gate::Cry(_, _, _) => gate_times_s::CX * 2.0,
            Gate::Ccx(_, _, _) => gate_times_s::CX * 6.0, // Toffoli decomposition
            Gate::Cswap(_, _, _) => gate_times_s::CX * 8.0,
            Gate::Measure(_) | Gate::MeasureAll => 5000e-9,
            Gate::Barrier(_) => 0.0,
            Gate::Reset(_) => 1000e-9,
        };

        time_s * 1e9
    }

    // ========================================================================
    // Basis Transformation Helpers
    // ========================================================================

    /// Get basis transformation gates for measuring in given basis
    /// X basis: H
    /// Y basis: Sdg, H
    /// Z basis: (none)
    pub fn basis_transform(qubit: QubitId, basis: crate::types::Basis) -> Vec<Gate> {
        use crate::types::Basis;
        match basis {
            Basis::X => vec![Gate::H(qubit)],
            Basis::Y => vec![Gate::Sdg(qubit), Gate::H(qubit)],
            Basis::Z => vec![],
        }
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_qasm())
    }
}

// ============================================================================
// Entangler Type
// ============================================================================

/// Entangler gate type for TQQC circuits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EntanglerType {
    /// CNOT (CX) gates
    #[default]
    Cx,
    /// CZ gates
    Cz,
}

impl EntanglerType {
    /// Create gate for given qubit pair
    pub fn gate(&self, control: QubitId, target: QubitId) -> Gate {
        match self {
            EntanglerType::Cx => Gate::Cnot(control, target),
            EntanglerType::Cz => Gate::Cz(control, target),
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cx" | "cnot" => Some(EntanglerType::Cx),
            "cz" => Some(EntanglerType::Cz),
            _ => None,
        }
    }
}

impl fmt::Display for EntanglerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntanglerType::Cx => write!(f, "cx"),
            EntanglerType::Cz => write!(f, "cz"),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Basis;

    #[test]
    fn test_gate_qubits() {
        assert_eq!(Gate::H(0).qubits(), vec![0]);
        assert_eq!(Gate::Cnot(0, 1).qubits(), vec![0, 1]);
        assert_eq!(Gate::Ccx(0, 1, 2).qubits(), vec![0, 1, 2]);
    }

    #[test]
    fn test_gate_classification() {
        assert!(Gate::H(0).is_single_qubit());
        assert!(!Gate::H(0).is_two_qubit());

        assert!(Gate::Cnot(0, 1).is_two_qubit());
        assert!(!Gate::Cnot(0, 1).is_single_qubit());

        assert!(Gate::Rx(0, 1.0).is_parameterized());
        assert!(!Gate::H(0).is_parameterized());
    }

    #[test]
    fn test_gate_to_qasm() {
        assert_eq!(Gate::H(0).to_qasm(), "h q[0];");
        assert_eq!(Gate::Cnot(0, 1).to_qasm(), "cx q[0],q[1];");
        assert_eq!(
            Gate::Rx(0, 1.5707963267948966).to_qasm(),
            "rx(1.5707963267948966) q[0];"
        );
    }

    #[test]
    fn test_basis_transform() {
        let x_gates = Gate::basis_transform(0, Basis::X);
        assert_eq!(x_gates.len(), 1);
        assert!(matches!(x_gates[0], Gate::H(0)));

        let y_gates = Gate::basis_transform(0, Basis::Y);
        assert_eq!(y_gates.len(), 2);
        assert!(matches!(y_gates[0], Gate::Sdg(0)));
        assert!(matches!(y_gates[1], Gate::H(0)));

        let z_gates = Gate::basis_transform(0, Basis::Z);
        assert!(z_gates.is_empty());
    }

    #[test]
    fn test_entangler_type() {
        assert_eq!(EntanglerType::Cx.gate(0, 1), Gate::Cnot(0, 1));
        assert_eq!(EntanglerType::Cz.gate(0, 1), Gate::Cz(0, 1));
    }

    #[test]
    fn test_gate_time() {
        let h_time = Gate::H(0).gate_time_ns();
        let cx_time = Gate::Cnot(0, 1).gate_time_ns();

        // CX should be slower than H
        assert!(cx_time > h_time);
    }
}
