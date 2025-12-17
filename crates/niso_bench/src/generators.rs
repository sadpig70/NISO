//! Circuit generators for benchmarking
//!
//! Gantree: L8_Benchmark → Generators
//!
//! Provides various circuit generators for benchmarking NISO.

use niso_core::{BasisString, Circuit, CircuitBuilder, EntanglerType};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::f64::consts::PI;

/// Circuit generator for benchmarks
/// Gantree: CircuitGenerator // 회로 생성기
pub struct CircuitGenerator {
    /// Random seed
    seed: Option<u64>,
}

impl CircuitGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self { seed: None }
    }

    /// Create generator with seed
    pub fn with_seed(seed: u64) -> Self {
        Self { seed: Some(seed) }
    }

    // ========================================================================
    // Standard Circuits
    // ========================================================================

    /// Generate GHZ state preparation circuit
    /// |GHZ⟩ = (|00...0⟩ + |11...1⟩) / √2
    pub fn ghz(&self, num_qubits: usize) -> Circuit {
        CircuitBuilder::new(num_qubits).h(0).cx_chain().build()
    }

    /// Generate W state preparation circuit
    /// |W⟩ = (|100...0⟩ + |010...0⟩ + ... + |00...01⟩) / √n
    pub fn w_state(&self, num_qubits: usize) -> Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);

        // Initialize first qubit
        builder = builder.x(0);

        // Distribute the excitation
        for i in 0..num_qubits - 1 {
            let angle = (1.0 / (num_qubits - i) as f64).sqrt().acos() * 2.0;
            builder = builder.ry(i, angle).cnot(i, i + 1);
        }

        builder.build()
    }

    /// Generate QFT (Quantum Fourier Transform) circuit
    pub fn qft(&self, num_qubits: usize) -> Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);

        for i in 0..num_qubits {
            builder = builder.h(i);

            for j in (i + 1)..num_qubits {
                let k = j - i + 1;
                let angle = PI / (1 << (k - 1)) as f64;
                // Controlled phase - approximate with Rz
                builder = builder
                    .cnot(j, i)
                    .rz(i, angle / 2.0)
                    .cnot(j, i)
                    .rz(i, -angle / 2.0);
            }
        }

        // Swap qubits for standard QFT ordering
        for i in 0..num_qubits / 2 {
            builder = builder.swap(i, num_qubits - 1 - i);
        }

        builder.build()
    }

    /// Generate TQQC parity circuit
    pub fn tqqc_parity(&self, num_qubits: usize, theta: f64, delta: f64) -> Circuit {
        let basis = BasisString::all_x(num_qubits);
        CircuitBuilder::new(num_qubits)
            .tqqc_parity(theta, delta, EntanglerType::Cx, &basis)
            .build()
    }

    /// Generate Bell state circuit
    pub fn bell(&self) -> Circuit {
        CircuitBuilder::new(2).h(0).cnot(0, 1).build()
    }

    // ========================================================================
    // Parameterized Circuits
    // ========================================================================

    /// Generate Hardware Efficient Ansatz (HEA) circuit
    pub fn hea(&self, num_qubits: usize, depth: usize) -> Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);
        let mut rng = self.get_rng();

        for _ in 0..depth {
            // Rotation layer
            for q in 0..num_qubits {
                let rx_angle: f64 = rng.gen::<f64>() * 2.0 * PI;
                let ry_angle: f64 = rng.gen::<f64>() * 2.0 * PI;
                builder = builder.rx(q, rx_angle).ry(q, ry_angle);
            }

            // Entangling layer
            builder = builder.cx_chain();
        }

        builder.build()
    }

    /// Generate random circuit
    pub fn random(&self, num_qubits: usize, depth: usize) -> Circuit {
        let mut builder = CircuitBuilder::new(num_qubits);
        let mut rng = self.get_rng();

        for _ in 0..depth {
            // Random single-qubit gates
            for q in 0..num_qubits {
                let gate_type = rng.gen_range(0..6);
                match gate_type {
                    0 => builder = builder.h(q),
                    1 => builder = builder.x(q),
                    2 => builder = builder.y(q),
                    3 => builder = builder.z(q),
                    4 => {
                        let angle: f64 = rng.gen::<f64>() * 2.0 * PI;
                        builder = builder.rx(q, angle);
                    }
                    _ => {
                        let angle: f64 = rng.gen::<f64>() * 2.0 * PI;
                        builder = builder.ry(q, angle);
                    }
                }
            }

            // Random two-qubit gates
            for q in 0..num_qubits.saturating_sub(1) {
                if rng.gen::<f64>() < 0.5 {
                    builder = builder.cnot(q, q + 1);
                }
            }
        }

        builder.build()
    }

    /// Generate layer of Hadamard gates
    pub fn h_layer(&self, num_qubits: usize) -> Circuit {
        CircuitBuilder::new(num_qubits).h_layer().build()
    }

    /// Generate identity circuit (just barrier)
    pub fn identity(&self, num_qubits: usize) -> Circuit {
        CircuitBuilder::new(num_qubits).barrier().build()
    }

    // ========================================================================
    // Benchmark-Specific Circuits
    // ========================================================================

    /// Generate circuit for parity oscillation test
    /// Varies theta to observe parity oscillation
    pub fn parity_oscillation(&self, num_qubits: usize, num_points: usize) -> Vec<Circuit> {
        (0..num_points)
            .map(|i| {
                let theta = (i as f64 / num_points as f64) * PI;
                self.tqqc_parity(num_qubits, theta, 0.0)
            })
            .collect()
    }

    /// Generate circuit for delta search test
    /// Varies delta around theta
    pub fn delta_search(&self, num_qubits: usize, theta: f64, deltas: &[f64]) -> Vec<Circuit> {
        deltas
            .iter()
            .map(|&delta| self.tqqc_parity(num_qubits, theta, delta))
            .collect()
    }

    /// Generate circuits for depth scaling test
    pub fn depth_scaling(&self, num_qubits: usize, max_depth: usize) -> Vec<Circuit> {
        (1..=max_depth).map(|d| self.hea(num_qubits, d)).collect()
    }

    /// Generate circuits for qubit scaling test
    pub fn qubit_scaling(&self, max_qubits: usize) -> Vec<Circuit> {
        (2..=max_qubits).map(|n| self.ghz(n)).collect()
    }

    // ========================================================================
    // Utility
    // ========================================================================

    /// Get RNG with optional seed
    fn get_rng(&self) -> StdRng {
        match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        }
    }
}

impl Default for CircuitGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghz() {
        let gen = CircuitGenerator::new();
        let circuit = gen.ghz(5);

        assert_eq!(circuit.num_qubits(), 5);
        assert_eq!(circuit.count_1q(), 1); // 1 H
        assert_eq!(circuit.count_2q(), 4); // 4 CNOTs
    }

    #[test]
    fn test_bell() {
        let gen = CircuitGenerator::new();
        let circuit = gen.bell();

        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.count_1q(), 1);
        assert_eq!(circuit.count_2q(), 1);
    }

    #[test]
    fn test_qft() {
        let gen = CircuitGenerator::new();
        let circuit = gen.qft(4);

        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.gates().len() > 0);
    }

    #[test]
    fn test_tqqc_parity() {
        let gen = CircuitGenerator::new();
        let circuit = gen.tqqc_parity(7, 0.5, 0.1);

        assert_eq!(circuit.num_qubits(), 7);
        assert_eq!(circuit.count_2q(), 6);
    }

    #[test]
    fn test_hea() {
        let gen = CircuitGenerator::with_seed(42);
        let circuit = gen.hea(5, 3);

        assert_eq!(circuit.num_qubits(), 5);
        assert!(circuit.depth() >= 3);
    }

    #[test]
    fn test_random() {
        let gen = CircuitGenerator::with_seed(42);
        let circuit = gen.random(5, 3);

        assert_eq!(circuit.num_qubits(), 5);
        assert!(circuit.gates().len() > 0);
    }

    #[test]
    fn test_random_reproducibility() {
        let gen1 = CircuitGenerator::with_seed(42);
        let gen2 = CircuitGenerator::with_seed(42);

        let c1 = gen1.random(5, 3);
        let c2 = gen2.random(5, 3);

        assert_eq!(c1.gates().len(), c2.gates().len());
    }

    #[test]
    fn test_parity_oscillation() {
        let gen = CircuitGenerator::new();
        let circuits = gen.parity_oscillation(5, 10);

        assert_eq!(circuits.len(), 10);
        for c in &circuits {
            assert_eq!(c.num_qubits(), 5);
        }
    }

    #[test]
    fn test_depth_scaling() {
        let gen = CircuitGenerator::with_seed(42);
        let circuits = gen.depth_scaling(5, 5);

        assert_eq!(circuits.len(), 5);

        // Deeper circuits should have more gates
        let depths: Vec<usize> = circuits.iter().map(|c| c.depth()).collect();
        for i in 1..depths.len() {
            assert!(depths[i] >= depths[i - 1]);
        }
    }

    #[test]
    fn test_qubit_scaling() {
        let gen = CircuitGenerator::new();
        let circuits = gen.qubit_scaling(7);

        assert_eq!(circuits.len(), 6); // 2 to 7 qubits

        for (i, c) in circuits.iter().enumerate() {
            assert_eq!(c.num_qubits(), i + 2);
        }
    }
}
