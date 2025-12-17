//! Quantum circuit structure for NISO
//!
//! Gantree: L1_Circuit → Circuit
//!
//! Provides the core Circuit struct for building and manipulating
//! quantum circuits used in TQQC optimization.

use crate::error::{NisoError, NisoResult};
use crate::gate::Gate;
use crate::topology::Topology;
use crate::types::QubitId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Quantum circuit
/// Gantree: Circuit // 회로 구조체
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    /// Number of qubits
    /// Gantree: num_qubits: usize // 큐비트 수
    num_qubits: usize,

    /// Gate sequence
    /// Gantree: gates: Vec<Gate> // 게이트 목록
    gates: Vec<Gate>,

    /// Optional circuit name
    name: Option<String>,
}

impl Circuit {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create a new empty circuit
    /// Gantree: new(n) -> Self // 생성자
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
            name: None,
        }
    }

    /// Create a circuit with a name
    pub fn with_name(num_qubits: usize, name: impl Into<String>) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
            name: Some(name.into()),
        }
    }

    /// Create from a vector of gates
    pub fn from_gates(num_qubits: usize, gates: Vec<Gate>) -> NisoResult<Self> {
        let circuit = Self {
            num_qubits,
            gates,
            name: None,
        };
        circuit.validate_gates()?;
        Ok(circuit)
    }

    // ========================================================================
    // Basic Operations
    // ========================================================================

    /// Add a gate to the circuit
    /// Gantree: add_gate(&mut, Gate) -> Result // 게이트 추가
    pub fn add_gate(&mut self, gate: Gate) -> NisoResult<()> {
        // Validate gate qubits
        for &qubit in &gate.qubits() {
            if qubit >= self.num_qubits {
                return Err(NisoError::GateQubitMismatch {
                    qubit,
                    num_qubits: self.num_qubits,
                });
            }
        }
        self.gates.push(gate);
        Ok(())
    }

    /// Add multiple gates
    pub fn add_gates(&mut self, gates: impl IntoIterator<Item = Gate>) -> NisoResult<()> {
        for gate in gates {
            self.add_gate(gate)?;
        }
        Ok(())
    }

    /// Clear all gates
    pub fn clear(&mut self) {
        self.gates.clear();
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get gates
    pub fn gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Get circuit name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set circuit name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Check if circuit is empty
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    // ========================================================================
    // Circuit Analysis
    // ========================================================================

    /// Calculate circuit depth (longest path)
    /// Gantree: depth(&self) -> usize // 깊이 계산
    pub fn depth(&self) -> usize {
        if self.gates.is_empty() {
            return 0;
        }

        // Track the depth at each qubit
        let mut qubit_depths = vec![0usize; self.num_qubits];

        for gate in &self.gates {
            let qubits = gate.qubits();
            if qubits.is_empty() {
                // MeasureAll or global barrier
                let max_depth = *qubit_depths.iter().max().unwrap_or(&0);
                for d in &mut qubit_depths {
                    *d = max_depth + 1;
                }
            } else {
                // Find maximum depth among gate qubits
                let max_depth = qubits
                    .iter()
                    .filter_map(|&q| qubit_depths.get(q))
                    .max()
                    .copied()
                    .unwrap_or(0);

                // Update all gate qubits to max_depth + 1
                for &q in &qubits {
                    if q < self.num_qubits {
                        qubit_depths[q] = max_depth + 1;
                    }
                }
            }
        }

        qubit_depths.into_iter().max().unwrap_or(0)
    }

    /// Get total gate count
    /// Gantree: gate_count(&self) -> usize // 게이트 수
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Count single-qubit gates
    /// Gantree: count_1q(&self) -> usize // 1Q 수
    pub fn count_1q(&self) -> usize {
        self.gates.iter().filter(|g| g.is_single_qubit()).count()
    }

    /// Count two-qubit gates
    /// Gantree: count_2q(&self) -> usize // 2Q 수
    pub fn count_2q(&self) -> usize {
        self.gates.iter().filter(|g| g.is_two_qubit()).count()
    }

    /// Count three-qubit gates
    pub fn count_3q(&self) -> usize {
        self.gates.iter().filter(|g| g.is_three_qubit()).count()
    }

    /// Count measurement operations
    pub fn count_measurements(&self) -> usize {
        self.gates.iter().filter(|g| g.is_measurement()).count()
    }

    /// Count parameterized gates
    pub fn count_parameterized(&self) -> usize {
        self.gates.iter().filter(|g| g.is_parameterized()).count()
    }

    /// Get qubits used in the circuit
    pub fn used_qubits(&self) -> HashSet<QubitId> {
        let mut used = HashSet::new();
        for gate in &self.gates {
            for qubit in gate.qubits() {
                used.insert(qubit);
            }
        }
        used
    }

    /// Get two-qubit gate pairs (for topology validation)
    pub fn two_qubit_pairs(&self) -> Vec<(QubitId, QubitId)> {
        self.gates
            .iter()
            .filter(|g| g.is_two_qubit())
            .filter_map(|g| {
                let qs = g.qubits();
                if qs.len() >= 2 {
                    Some((qs[0], qs[1]))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Estimate total execution time in nanoseconds
    pub fn total_time_ns(&self) -> f64 {
        self.gates.iter().map(|g| g.gate_time_ns()).sum()
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate all gates in the circuit
    fn validate_gates(&self) -> NisoResult<()> {
        for gate in &self.gates {
            for &qubit in &gate.qubits() {
                if qubit >= self.num_qubits {
                    return Err(NisoError::GateQubitMismatch {
                        qubit,
                        num_qubits: self.num_qubits,
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate circuit against a topology
    /// Gantree: validate(&self, Topology) -> Result // 토폴로지 검증
    pub fn validate(&self, topology: &Topology) -> NisoResult<()> {
        topology.validate_circuit(self)
    }

    // ========================================================================
    // QASM Conversion
    // ========================================================================

    /// Convert to OpenQASM 2.0 string
    /// Gantree: to_qasm(&self) -> String // QASM2 출력
    pub fn to_qasm(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push("OPENQASM 2.0;".to_string());
        lines.push("include \"qelib1.inc\";".to_string());
        lines.push(String::new());

        // Register declarations
        lines.push(format!("qreg q[{}];", self.num_qubits));
        lines.push(format!("creg c[{}];", self.num_qubits));
        lines.push(String::new());

        // Gates
        for gate in &self.gates {
            lines.push(gate.to_qasm());
        }

        lines.join("\n")
    }

    /// Parse from OpenQASM 2.0 string (basic support)
    /// Gantree: from_qasm(s) -> Result<Self> // QASM2 파싱
    pub fn from_qasm(qasm: &str) -> NisoResult<Self> {
        let mut num_qubits = 0;
        let mut gates = Vec::new();

        for line in qasm.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse qreg
            if line.starts_with("qreg") {
                if let Some(n) = parse_register_size(line) {
                    num_qubits = n;
                }
                continue;
            }

            // Skip headers and includes
            if line.starts_with("OPENQASM")
                || line.starts_with("include")
                || line.starts_with("creg")
            {
                continue;
            }

            // Parse gates (simplified)
            if let Some(gate) = parse_gate_line(line)? {
                gates.push(gate);
            }
        }

        if num_qubits == 0 {
            return Err(NisoError::InvalidQasm("No qreg declaration found".into()));
        }

        Circuit::from_gates(num_qubits, gates)
    }
}

// ============================================================================
// QASM Parsing Helpers
// ============================================================================

fn parse_register_size(line: &str) -> Option<usize> {
    // Parse "qreg q[N];" -> N
    let start = line.find('[')?;
    let end = line.find(']')?;
    line[start + 1..end].parse().ok()
}

fn parse_gate_line(line: &str) -> NisoResult<Option<Gate>> {
    let line = line.trim().trim_end_matches(';');

    // Split gate name and arguments
    let (name, args) = if let Some(paren_pos) = line.find('(') {
        let end_paren = line
            .find(')')
            .ok_or_else(|| NisoError::InvalidQasm(format!("Missing closing paren: {}", line)))?;
        let params: Vec<f64> = line[paren_pos + 1..end_paren]
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let rest = &line[end_paren + 1..].trim();
        let name = &line[..paren_pos];
        (name, (params, rest.to_string()))
    } else {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return Ok(None);
        }
        (parts[0], (vec![], parts[1].to_string()))
    };

    let (params, qubits_str) = args;
    let qubits = parse_qubits(&qubits_str)?;

    let gate = match name.to_lowercase().as_str() {
        "h" => qubits.first().map(|&q| Gate::H(q)),
        "x" => qubits.first().map(|&q| Gate::X(q)),
        "y" => qubits.first().map(|&q| Gate::Y(q)),
        "z" => qubits.first().map(|&q| Gate::Z(q)),
        "s" => qubits.first().map(|&q| Gate::S(q)),
        "sdg" => qubits.first().map(|&q| Gate::Sdg(q)),
        "t" => qubits.first().map(|&q| Gate::T(q)),
        "tdg" => qubits.first().map(|&q| Gate::Tdg(q)),
        "rx" => {
            if let (Some(&q), Some(&theta)) = (qubits.first(), params.first()) {
                Some(Gate::Rx(q, theta))
            } else {
                None
            }
        }
        "ry" => {
            if let (Some(&q), Some(&theta)) = (qubits.first(), params.first()) {
                Some(Gate::Ry(q, theta))
            } else {
                None
            }
        }
        "rz" => {
            if let (Some(&q), Some(&theta)) = (qubits.first(), params.first()) {
                Some(Gate::Rz(q, theta))
            } else {
                None
            }
        }
        "cx" | "cnot" => {
            if qubits.len() >= 2 {
                Some(Gate::Cnot(qubits[0], qubits[1]))
            } else {
                None
            }
        }
        "cz" => {
            if qubits.len() >= 2 {
                Some(Gate::Cz(qubits[0], qubits[1]))
            } else {
                None
            }
        }
        "swap" => {
            if qubits.len() >= 2 {
                Some(Gate::Swap(qubits[0], qubits[1]))
            } else {
                None
            }
        }
        "ccx" | "toffoli" => {
            if qubits.len() >= 3 {
                Some(Gate::Ccx(qubits[0], qubits[1], qubits[2]))
            } else {
                None
            }
        }
        "measure" => qubits.first().map(|&q| Gate::Measure(q)),
        "reset" => qubits.first().map(|&q| Gate::Reset(q)),
        "barrier" => Some(Gate::Barrier(qubits)),
        _ => None,
    };

    Ok(gate)
}

fn parse_qubits(s: &str) -> NisoResult<Vec<QubitId>> {
    let mut qubits = Vec::new();

    for part in s.split(',') {
        let part = part.trim();
        // Parse "q[N]" -> N
        if let Some(start) = part.find('[') {
            if let Some(end) = part.find(']') {
                if let Ok(q) = part[start + 1..end].parse() {
                    qubits.push(q);
                }
            }
        }
    }

    Ok(qubits)
}

// ============================================================================
// Display
// ============================================================================

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Circuit({} qubits, {} gates)",
            self.num_qubits,
            self.gates.len()
        )?;
        writeln!(f, "  Depth: {}", self.depth())?;
        writeln!(f, "  1Q gates: {}", self.count_1q())?;
        writeln!(f, "  2Q gates: {}", self.count_2q())?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_new() {
        let circuit = Circuit::new(5);
        assert_eq!(circuit.num_qubits(), 5);
        assert!(circuit.is_empty());
    }

    #[test]
    fn test_add_gate() {
        let mut circuit = Circuit::new(3);
        assert!(circuit.add_gate(Gate::H(0)).is_ok());
        assert!(circuit.add_gate(Gate::Cnot(0, 1)).is_ok());
        assert_eq!(circuit.gate_count(), 2);
    }

    #[test]
    fn test_add_gate_out_of_range() {
        let mut circuit = Circuit::new(3);
        assert!(circuit.add_gate(Gate::H(5)).is_err());
    }

    #[test]
    fn test_circuit_depth() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::Cnot(0, 1)).unwrap();
        circuit.add_gate(Gate::H(2)).unwrap();

        // H(0), H(1) parallel -> depth 1
        // CNOT(0,1) -> depth 2
        // H(2) can be parallel with CNOT -> depth 2
        assert!(circuit.depth() >= 2);
    }

    #[test]
    fn test_gate_counts() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::H(1)).unwrap();
        circuit.add_gate(Gate::Cnot(0, 1)).unwrap();
        circuit.add_gate(Gate::Rx(0, 1.0)).unwrap();

        assert_eq!(circuit.count_1q(), 3);
        assert_eq!(circuit.count_2q(), 1);
        assert_eq!(circuit.count_parameterized(), 1);
    }

    #[test]
    fn test_to_qasm() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(Gate::H(0)).unwrap();
        circuit.add_gate(Gate::Cnot(0, 1)).unwrap();

        let qasm = circuit.to_qasm();
        assert!(qasm.contains("OPENQASM 2.0"));
        assert!(qasm.contains("qreg q[2]"));
        assert!(qasm.contains("h q[0]"));
        assert!(qasm.contains("cx q[0],q[1]"));
    }

    #[test]
    fn test_from_qasm() {
        let qasm = r#"
            OPENQASM 2.0;
            include "qelib1.inc";
            qreg q[2];
            creg c[2];
            h q[0];
            cx q[0],q[1];
        "#;

        let circuit = Circuit::from_qasm(qasm).unwrap();
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.gate_count(), 2);
    }

    #[test]
    fn test_two_qubit_pairs() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(Gate::Cnot(0, 1)).unwrap();
        circuit.add_gate(Gate::Cz(1, 2)).unwrap();

        let pairs = circuit.two_qubit_pairs();
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&(0, 1)));
        assert!(pairs.contains(&(1, 2)));
    }
}
