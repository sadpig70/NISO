//! Circuit transpilation for IBM backends
//!
//! Gantree: L10_Qiskit → Transpiler

use niso_core::{Circuit, Gate};
use std::collections::HashSet;

/// IBM native gate set
pub const IBM_BASIS_GATES: &[&str] = &["id", "rz", "sx", "x", "cx", "ecr"];

/// Transpiler configuration
#[derive(Debug, Clone)]
pub struct TranspilerConfig {
    /// Target basis gates
    pub basis_gates: Vec<String>,

    /// Coupling map (allowed 2-qubit connections)
    pub coupling_map: Option<Vec<(usize, usize)>>,

    /// Optimization level (0-3)
    pub optimization_level: u8,

    /// Number of qubits on target
    pub num_qubits: usize,
}

impl Default for TranspilerConfig {
    fn default() -> Self {
        Self {
            basis_gates: IBM_BASIS_GATES.iter().map(|s| s.to_string()).collect(),
            coupling_map: None,
            optimization_level: 1,
            num_qubits: 127,
        }
    }
}

impl TranspilerConfig {
    /// Create config for specific backend
    pub fn for_backend(num_qubits: usize, coupling_map: Vec<(usize, usize)>) -> Self {
        Self {
            num_qubits,
            coupling_map: Some(coupling_map),
            ..Default::default()
        }
    }

    /// Set optimization level
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.optimization_level = level.min(3);
        self
    }
}

/// Circuit transpiler for IBM backends
pub struct Transpiler {
    config: TranspilerConfig,
}

impl Transpiler {
    /// Create new transpiler
    pub fn new(config: TranspilerConfig) -> Self {
        Self { config }
    }

    /// Transpile circuit to OpenQASM 3.0
    pub fn to_qasm3(&self, circuit: &Circuit) -> String {
        let mut qasm = String::new();

        // Header
        qasm.push_str("OPENQASM 3.0;\n");
        qasm.push_str("include \"stdgates.inc\";\n\n");

        // Declarations
        qasm.push_str(&format!("qubit[{}] q;\n", circuit.num_qubits()));
        qasm.push_str(&format!("bit[{}] c;\n\n", circuit.num_qubits()));

        // Gates
        let mut measure_idx = 0;
        for gate in circuit.gates() {
            match gate {
                Gate::H(q) => {
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q));
                }
                Gate::X(q) => qasm.push_str(&format!("x q[{}];\n", q)),
                Gate::Y(q) => {
                    qasm.push_str(&format!("rz(pi) q[{}];\n", q));
                    qasm.push_str(&format!("x q[{}];\n", q));
                }
                Gate::Z(q) => qasm.push_str(&format!("rz(pi) q[{}];\n", q)),
                Gate::S(q) => qasm.push_str(&format!("rz(pi/2) q[{}];\n", q)),
                Gate::Sdg(q) => qasm.push_str(&format!("rz(-pi/2) q[{}];\n", q)),
                Gate::T(q) => qasm.push_str(&format!("rz(pi/4) q[{}];\n", q)),
                Gate::Tdg(q) => qasm.push_str(&format!("rz(-pi/4) q[{}];\n", q)),
                Gate::Sx(q) => qasm.push_str(&format!("sx q[{}];\n", q)),
                Gate::Sxdg(q) => qasm.push_str(&format!("sxdg q[{}];\n", q)),
                Gate::Id(q) => qasm.push_str(&format!("id q[{}];\n", q)),
                Gate::Rx(q, theta) => {
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz({} + pi) q[{}];\n", theta, q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q));
                }
                Gate::Ry(q, theta) => {
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz({} + pi) q[{}];\n", theta, q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                }
                Gate::Rz(q, theta) => qasm.push_str(&format!("rz({}) q[{}];\n", theta, q)),
                Gate::P(q, lambda) => qasm.push_str(&format!("rz({}) q[{}];\n", lambda, q)),
                Gate::U(q, theta, phi, lambda) => {
                    qasm.push_str(&format!("rz({}) q[{}];\n", lambda, q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz({} + pi) q[{}];\n", theta, q));
                    qasm.push_str(&format!("sx q[{}];\n", q));
                    qasm.push_str(&format!("rz({} + pi) q[{}];\n", phi, q));
                }
                Gate::Cnot(ctrl, tgt) => qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt)),
                Gate::Cz(ctrl, tgt) => {
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                }
                Gate::Cy(ctrl, tgt) => {
                    qasm.push_str(&format!("rz(-pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                }
                Gate::Swap(q0, q1) => {
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", q0, q1));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", q1, q0));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", q0, q1));
                }
                Gate::ISwap(q0, q1) => {
                    // iSWAP = S(q0) S(q1) H(q0) CX(q0,q1) CX(q1,q0) H(q1)
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q0));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q1));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q0));
                    qasm.push_str(&format!("sx q[{}];\n", q0));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q0));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", q0, q1));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", q1, q0));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q1));
                    qasm.push_str(&format!("sx q[{}];\n", q1));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", q1));
                }
                Gate::Ecr(q0, q1) => qasm.push_str(&format!("ecr q[{}], q[{}];\n", q0, q1)),
                Gate::Crz(ctrl, tgt, theta) => {
                    qasm.push_str(&format!("rz({}/2) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("rz(-{}/2) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                }
                Gate::Crx(ctrl, tgt, theta) => {
                    // CRX = RZ(pi/2) CX RY(theta/2) CX RY(-theta/2) RZ(-pi/2)
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz({}/2 + pi) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(-{}/2 + pi) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("rz(-pi/2) q[{}];\n", tgt));
                }
                Gate::Cry(ctrl, tgt, theta) => {
                    // CRY = RY(theta/2) CX RY(-theta/2) CX
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz({}/2 + pi) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(-{}/2 + pi) q[{}];\n", theta, tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                }
                Gate::Ccx(c1, c2, tgt) => {
                    // Toffoli decomposition using 6 CNOT gates
                    // Based on Nielsen & Chuang decomposition
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c2, tgt));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c1, tgt));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c2, tgt));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c1, tgt));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", c2));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("sx q[{}];\n", tgt));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", tgt));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c1, c2));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", c1));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", c2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c1, c2));
                }
                Gate::Cswap(c, t1, t2) => {
                    // CSWAP = CX(t2,t1) CCX(c,t1,t2) CX(t2,t1)
                    // Decompose CCX inline
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", t2, t1));
                    // CCX(c, t1, t2) decomposition
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", t2));
                    qasm.push_str(&format!("sx q[{}];\n", t2));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", t2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", t1, t2));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", t2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c, t2));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", t2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", t1, t2));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", t2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c, t2));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", t1));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", t2));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", t2));
                    qasm.push_str(&format!("sx q[{}];\n", t2));
                    qasm.push_str(&format!("rz(pi/2) q[{}];\n", t2));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c, t1));
                    qasm.push_str(&format!("rz(pi/4) q[{}];\n", c));
                    qasm.push_str(&format!("rz(-pi/4) q[{}];\n", t1));
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", c, t1));
                    // Final CX
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", t2, t1));
                }
                Gate::Measure(q) => {
                    qasm.push_str(&format!("c[{}] = measure q[{}];\n", measure_idx, q));
                    measure_idx += 1;
                }
                Gate::MeasureAll => {
                    for q in 0..circuit.num_qubits() {
                        qasm.push_str(&format!("c[{}] = measure q[{}];\n", q, q));
                    }
                }
                Gate::Barrier(qubits) => {
                    if qubits.is_empty() {
                        qasm.push_str("barrier;\n");
                    } else {
                        let qs: Vec<_> = qubits.iter().map(|q| format!("q[{}]", q)).collect();
                        qasm.push_str(&format!("barrier {};\n", qs.join(", ")));
                    }
                }
                Gate::Reset(q) => qasm.push_str(&format!("reset q[{}];\n", q)),
            }
        }

        qasm
    }

    /// Transpile circuit to OpenQASM 2.0 (legacy)
    pub fn to_qasm2(&self, circuit: &Circuit) -> String {
        let mut qasm = String::new();

        qasm.push_str("OPENQASM 2.0;\n");
        qasm.push_str("include \"qelib1.inc\";\n\n");
        qasm.push_str(&format!("qreg q[{}];\n", circuit.num_qubits()));
        qasm.push_str(&format!("creg c[{}];\n\n", circuit.num_qubits()));

        let mut measure_idx = 0;
        for gate in circuit.gates() {
            match gate {
                Gate::H(q) => qasm.push_str(&format!("h q[{}];\n", q)),
                Gate::X(q) => qasm.push_str(&format!("x q[{}];\n", q)),
                Gate::Y(q) => qasm.push_str(&format!("y q[{}];\n", q)),
                Gate::Z(q) => qasm.push_str(&format!("z q[{}];\n", q)),
                Gate::S(q) => qasm.push_str(&format!("s q[{}];\n", q)),
                Gate::Sdg(q) => qasm.push_str(&format!("sdg q[{}];\n", q)),
                Gate::T(q) => qasm.push_str(&format!("t q[{}];\n", q)),
                Gate::Tdg(q) => qasm.push_str(&format!("tdg q[{}];\n", q)),
                Gate::Sx(q) => qasm.push_str(&format!("sx q[{}];\n", q)),
                Gate::Sxdg(q) => qasm.push_str(&format!("sxdg q[{}];\n", q)),
                Gate::Id(q) => qasm.push_str(&format!("id q[{}];\n", q)),
                Gate::Rx(q, theta) => qasm.push_str(&format!("rx({}) q[{}];\n", theta, q)),
                Gate::Ry(q, theta) => qasm.push_str(&format!("ry({}) q[{}];\n", theta, q)),
                Gate::Rz(q, theta) => qasm.push_str(&format!("rz({}) q[{}];\n", theta, q)),
                Gate::P(q, lambda) => qasm.push_str(&format!("p({}) q[{}];\n", lambda, q)),
                Gate::U(q, theta, phi, lambda) => {
                    qasm.push_str(&format!("u3({},{},{}) q[{}];\n", theta, phi, lambda, q))
                }
                Gate::Cnot(c, t) => qasm.push_str(&format!("cx q[{}],q[{}];\n", c, t)),
                Gate::Cz(c, t) => qasm.push_str(&format!("cz q[{}],q[{}];\n", c, t)),
                Gate::Cy(c, t) => qasm.push_str(&format!("cy q[{}],q[{}];\n", c, t)),
                Gate::Swap(a, b) => qasm.push_str(&format!("swap q[{}],q[{}];\n", a, b)),
                Gate::ISwap(a, b) => qasm.push_str(&format!("// iswap q[{}],q[{}];\n", a, b)),
                Gate::Ecr(a, b) => qasm.push_str(&format!("// ecr q[{}],q[{}];\n", a, b)),
                Gate::Crz(c, t, theta) => {
                    qasm.push_str(&format!("crz({}) q[{}],q[{}];\n", theta, c, t))
                }
                Gate::Crx(c, t, theta) => {
                    qasm.push_str(&format!("crx({}) q[{}],q[{}];\n", theta, c, t))
                }
                Gate::Cry(c, t, theta) => {
                    qasm.push_str(&format!("cry({}) q[{}],q[{}];\n", theta, c, t))
                }
                Gate::Ccx(c1, c2, t) => {
                    qasm.push_str(&format!("ccx q[{}],q[{}],q[{}];\n", c1, c2, t))
                }
                Gate::Cswap(c, t1, t2) => {
                    qasm.push_str(&format!("cswap q[{}],q[{}],q[{}];\n", c, t1, t2))
                }
                Gate::Measure(q) => {
                    qasm.push_str(&format!("measure q[{}] -> c[{}];\n", q, measure_idx));
                    measure_idx += 1;
                }
                Gate::MeasureAll => {
                    for q in 0..circuit.num_qubits() {
                        qasm.push_str(&format!("measure q[{}] -> c[{}];\n", q, q));
                    }
                }
                Gate::Barrier(qubits) => {
                    if qubits.is_empty() {
                        qasm.push_str("barrier q;\n");
                    } else {
                        let qs: Vec<_> = qubits.iter().map(|q| format!("q[{}]", q)).collect();
                        qasm.push_str(&format!("barrier {};\n", qs.join(",")));
                    }
                }
                Gate::Reset(q) => qasm.push_str(&format!("reset q[{}];\n", q)),
            }
        }

        qasm
    }

    /// Check if circuit uses only basis gates
    pub fn uses_basis_gates(&self, circuit: &Circuit) -> bool {
        let basis_set: HashSet<_> = self.config.basis_gates.iter().collect();
        circuit.gates().iter().all(|gate| {
            let name = gate.name().to_string();
            basis_set.contains(&name)
        })
    }

    /// Validate circuit for backend
    pub fn validate(&self, circuit: &Circuit) -> Result<(), String> {
        if circuit.num_qubits() > self.config.num_qubits {
            return Err(format!(
                "Circuit requires {} qubits, backend has {}",
                circuit.num_qubits(),
                self.config.num_qubits
            ));
        }

        if let Some(ref coupling) = self.config.coupling_map {
            let coupling_set: HashSet<_> = coupling.iter().collect();

            for gate in circuit.gates() {
                let qubits = gate.qubits();
                if qubits.len() == 2 {
                    let (q0, q1) = (qubits[0], qubits[1]);
                    if !coupling_set.contains(&(q0, q1)) && !coupling_set.contains(&(q1, q0)) {
                        return Err(format!("Qubits ({}, {}) not connected", q0, q1));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niso_core::CircuitBuilder;

    #[test]
    fn test_qasm3_output() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();
        let transpiler = Transpiler::new(TranspilerConfig::default());
        let qasm = transpiler.to_qasm3(&circuit);

        assert!(qasm.contains("OPENQASM 3.0"));
        assert!(qasm.contains("qubit[2]"));
        assert!(qasm.contains("cx"));
    }

    #[test]
    fn test_qasm2_output() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();
        let transpiler = Transpiler::new(TranspilerConfig::default());
        let qasm = transpiler.to_qasm2(&circuit);

        assert!(qasm.contains("OPENQASM 2.0"));
        assert!(qasm.contains("qreg q[2]"));
    }

    #[test]
    fn test_validation() {
        let circuit = CircuitBuilder::new(10).h(0).build();
        let config = TranspilerConfig {
            num_qubits: 5,
            ..Default::default()
        };
        let transpiler = Transpiler::new(config);

        assert!(transpiler.validate(&circuit).is_err());
    }
}
