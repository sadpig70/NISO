//! Simulator backend for NISO
//!
//! Gantree: L6_Backend → SimulatorBackend
//!
//! Provides a noise-aware quantum circuit simulator for TQQC testing.

use crate::execution::{Backend, ExecutionMetadata, ExecutionResult};
use niso_calibration::CalibrationInfo;
use niso_core::{Circuit, Counts, Gate, NisoError, NisoResult};
use niso_noise::NoiseModel;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::f64::consts::PI;

/// Simulator backend with noise model
/// Gantree: SimulatorBackend // 시뮬레이터 구현
pub struct SimulatorBackend {
    /// Backend name
    name: String,

    /// Number of qubits
    num_qubits: usize,

    /// Noise model
    noise_model: NoiseModel,

    /// Calibration info
    calibration: Option<CalibrationInfo>,

    /// Random seed
    seed: Option<u64>,
}

impl SimulatorBackend {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new simulator backend
    pub fn new(num_qubits: usize, noise_model: NoiseModel) -> Self {
        Self {
            name: "niso_simulator".to_string(),
            num_qubits,
            noise_model,
            calibration: None,
            seed: None,
        }
    }

    /// Create ideal (noiseless) simulator
    pub fn ideal(num_qubits: usize) -> Self {
        Self::new(num_qubits, NoiseModel::ideal())
    }

    /// Create IBM-typical simulator
    pub fn ibm_typical(num_qubits: usize) -> Self {
        Self::new(num_qubits, NoiseModel::ibm_typical())
    }

    /// Create from depolarizing error rate
    pub fn from_depol(num_qubits: usize, p_depol: f64) -> NisoResult<Self> {
        let noise_model = NoiseModel::from_depol(p_depol)?;
        Ok(Self::new(num_qubits, noise_model))
    }

    /// Set seed for reproducibility
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set calibration info
    pub fn with_calibration(mut self, calibration: CalibrationInfo) -> Self {
        self.calibration = Some(calibration);
        self
    }

    /// Set backend name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    // ========================================================================
    // Simulation
    // ========================================================================

    /// Simulate circuit and return counts
    fn simulate(&self, circuit: &Circuit, shots: u64, rng: &mut StdRng) -> Counts {
        let mut counts: Counts = HashMap::new();

        for _ in 0..shots {
            let bitstring = self.simulate_single_shot(circuit, rng);
            *counts.entry(bitstring).or_insert(0) += 1;
        }

        counts
    }

    /// Simulate a single shot
    fn simulate_single_shot(&self, circuit: &Circuit, rng: &mut StdRng) -> String {
        // Initialize state vector (simplified: track amplitudes)
        let n = circuit.num_qubits();
        let mut state = vec![Complex::zero(); 1 << n];
        state[0] = Complex::one(); // |00...0⟩

        // Apply gates
        for gate in circuit.gates() {
            self.apply_gate(&mut state, gate, n, rng);
        }

        // Measure with noise
        self.measure_state(&state, n, rng)
    }

    /// Apply a gate to the state
    fn apply_gate(&self, state: &mut [Complex], gate: &Gate, n: usize, rng: &mut StdRng) {
        // Apply depolarizing noise before gate
        let error_rate = if gate.is_two_qubit() {
            self.noise_model.gate_error_2q()
        } else if gate.is_single_qubit() {
            self.noise_model.gate_error_1q()
        } else {
            0.0
        };

        if error_rate > 0.0 && rng.gen::<f64>() < error_rate {
            // Apply random Pauli error (simplified)
            self.apply_random_error(state, gate, n, rng);
            return;
        }

        // Apply ideal gate
        match gate {
            Gate::H(q) => self.apply_h(state, *q, n),
            Gate::X(q) => self.apply_x(state, *q, n),
            Gate::Y(q) => self.apply_y(state, *q, n),
            Gate::Z(q) => self.apply_z(state, *q, n),
            Gate::S(q) => self.apply_s(state, *q, n),
            Gate::Sdg(q) => self.apply_sdg(state, *q, n),
            Gate::T(q) => self.apply_t(state, *q, n),
            Gate::Tdg(q) => self.apply_tdg(state, *q, n),
            Gate::Rx(q, angle) => self.apply_rx(state, *q, *angle, n),
            Gate::Ry(q, angle) => self.apply_ry(state, *q, *angle, n),
            Gate::Rz(q, angle) => self.apply_rz(state, *q, *angle, n),
            Gate::Cnot(c, t) => self.apply_cnot(state, *c, *t, n),
            Gate::Cz(c, t) => self.apply_cz(state, *c, *t, n),
            Gate::Swap(q1, q2) => self.apply_swap(state, *q1, *q2, n),
            _ => {} // Barrier, Measure, etc. - no state change needed here
        }
    }

    /// Apply random Pauli error
    fn apply_random_error(&self, state: &mut [Complex], gate: &Gate, n: usize, rng: &mut StdRng) {
        let qubits = gate.qubits();
        if qubits.is_empty() {
            return;
        }

        // Apply random Pauli to first qubit
        let q = qubits[0];
        match rng.gen_range(0..3) {
            0 => self.apply_x(state, q, n),
            1 => self.apply_y(state, q, n),
            _ => self.apply_z(state, q, n),
        }
    }

    /// Measure state and return bitstring
    fn measure_state(&self, state: &[Complex], n: usize, rng: &mut StdRng) -> String {
        // Calculate probabilities
        let probs: Vec<f64> = state.iter().map(|c| c.norm_squared()).collect();

        // Sample
        let mut cumsum = 0.0;
        let r: f64 = rng.gen();
        let mut outcome = probs.len() - 1;

        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if r < cumsum {
                outcome = i;
                break;
            }
        }

        // Apply readout error
        let mut result = outcome;
        if self.noise_model.readout_error() > 0.0 {
            for bit in 0..n {
                if rng.gen::<f64>() < self.noise_model.readout_error() {
                    result ^= 1 << bit; // Flip bit
                }
            }
        }

        // Convert to bitstring
        format!("{:0width$b}", result, width = n)
    }

    // ========================================================================
    // Single-Qubit Gates
    // ========================================================================

    fn apply_h(&self, state: &mut [Complex], q: usize, n: usize) {
        let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
        self.apply_single_qubit_gate(state, q, n, |a, b| {
            let new_a = (a + b) * sqrt2_inv;
            let new_b = (a - b) * sqrt2_inv;
            (new_a, new_b)
        });
    }

    fn apply_x(&self, state: &mut [Complex], q: usize, n: usize) {
        self.apply_single_qubit_gate(state, q, n, |a, b| (b, a));
    }

    fn apply_y(&self, state: &mut [Complex], q: usize, n: usize) {
        self.apply_single_qubit_gate(state, q, n, |a, b| {
            (b * Complex::new(0.0, -1.0), a * Complex::new(0.0, 1.0))
        });
    }

    fn apply_z(&self, state: &mut [Complex], q: usize, n: usize) {
        self.apply_single_qubit_gate(state, q, n, |a, b| (a, b * -1.0));
    }

    fn apply_s(&self, state: &mut [Complex], q: usize, n: usize) {
        self.apply_single_qubit_gate(state, q, n, |a, b| (a, b * Complex::new(0.0, 1.0)));
    }

    fn apply_sdg(&self, state: &mut [Complex], q: usize, n: usize) {
        self.apply_single_qubit_gate(state, q, n, |a, b| (a, b * Complex::new(0.0, -1.0)));
    }

    fn apply_t(&self, state: &mut [Complex], q: usize, n: usize) {
        let phase = Complex::from_polar(1.0, PI / 4.0);
        self.apply_single_qubit_gate(state, q, n, |a, b| (a, b * phase));
    }

    fn apply_tdg(&self, state: &mut [Complex], q: usize, n: usize) {
        let phase = Complex::from_polar(1.0, -PI / 4.0);
        self.apply_single_qubit_gate(state, q, n, |a, b| (a, b * phase));
    }

    fn apply_rx(&self, state: &mut [Complex], q: usize, angle: f64, n: usize) {
        let c = (angle / 2.0).cos();
        let s = (angle / 2.0).sin();
        self.apply_single_qubit_gate(state, q, n, |a, b| {
            let new_a = a * c + b * Complex::new(0.0, -s);
            let new_b = a * Complex::new(0.0, -s) + b * c;
            (new_a, new_b)
        });
    }

    fn apply_ry(&self, state: &mut [Complex], q: usize, angle: f64, n: usize) {
        let c = (angle / 2.0).cos();
        let s = (angle / 2.0).sin();
        self.apply_single_qubit_gate(state, q, n, |a, b| {
            let new_a = a * c - b * s;
            let new_b = a * s + b * c;
            (new_a, new_b)
        });
    }

    fn apply_rz(&self, state: &mut [Complex], q: usize, angle: f64, n: usize) {
        let phase_neg = Complex::from_polar(1.0, -angle / 2.0);
        let phase_pos = Complex::from_polar(1.0, angle / 2.0);
        self.apply_single_qubit_gate(state, q, n, |a, b| (a * phase_neg, b * phase_pos));
    }

    fn apply_single_qubit_gate<F>(&self, state: &mut [Complex], q: usize, n: usize, f: F)
    where
        F: Fn(Complex, Complex) -> (Complex, Complex),
    {
        let mask = 1 << q;
        for i in 0..(1 << n) {
            if i & mask == 0 {
                let j = i | mask;
                let (new_i, new_j) = f(state[i], state[j]);
                state[i] = new_i;
                state[j] = new_j;
            }
        }
    }

    // ========================================================================
    // Two-Qubit Gates
    // ========================================================================

    fn apply_cnot(&self, state: &mut [Complex], control: usize, target: usize, n: usize) {
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        for i in 0..(1 << n) {
            if (i & control_mask) != 0 && (i & target_mask) == 0 {
                let j = i | target_mask;
                state.swap(i, j);
            }
        }
    }

    fn apply_cz(&self, state: &mut [Complex], q1: usize, q2: usize, n: usize) {
        let mask1 = 1 << q1;
        let mask2 = 1 << q2;

        for i in 0..(1 << n) {
            if (i & mask1) != 0 && (i & mask2) != 0 {
                state[i] = state[i] * -1.0;
            }
        }
    }

    fn apply_swap(&self, state: &mut [Complex], q1: usize, q2: usize, n: usize) {
        let mask1 = 1 << q1;
        let mask2 = 1 << q2;

        for i in 0..(1 << n) {
            let bit1 = (i & mask1) != 0;
            let bit2 = (i & mask2) != 0;

            if bit1 != bit2 {
                let j = i ^ mask1 ^ mask2;
                if i < j {
                    state.swap(i, j);
                }
            }
        }
    }
}

impl Backend for SimulatorBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    fn execute(&self, circuit: &Circuit, shots: u64) -> NisoResult<ExecutionResult> {
        if circuit.num_qubits() > self.num_qubits {
            return Err(NisoError::QubitOutOfRange {
                qubit: circuit.num_qubits(),
                max: self.num_qubits,
            });
        }

        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let counts = self.simulate(circuit, shots, &mut rng);

        Ok(ExecutionResult {
            counts,
            shots,
            metadata: ExecutionMetadata {
                backend: self.name.clone(),
                simulated: true,
                seed: self.seed,
                ..Default::default()
            },
        })
    }

    fn calibration(&self) -> Option<&CalibrationInfo> {
        self.calibration.as_ref()
    }

    fn is_simulator(&self) -> bool {
        true
    }
}

// ============================================================================
// Complex Number Helper
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    fn one() -> Self {
        Self::new(1.0, 0.0)
    }

    fn from_polar(r: f64, theta: f64) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    fn norm_squared(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.re * scalar, self.im * scalar)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use niso_core::CircuitBuilder;

    #[test]
    fn test_simulator_ideal() {
        let backend = SimulatorBackend::ideal(3).with_seed(42);

        // Bell state circuit
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // Should get approximately 50% |00⟩ and 50% |11⟩
        let p00 = result.probability("00");
        let p11 = result.probability("11");

        assert!(p00 > 0.4 && p00 < 0.6, "P(00) = {}", p00);
        assert!(p11 > 0.4 && p11 < 0.6, "P(11) = {}", p11);
    }

    #[test]
    fn test_simulator_noisy() {
        let backend = SimulatorBackend::from_depol(3, 0.01).unwrap().with_seed(42);

        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // With noise, should still see mostly |00⟩ and |11⟩
        let p00 = result.probability("00");
        let p11 = result.probability("11");

        assert!(p00 + p11 > 0.9, "P(00)+P(11) = {}", p00 + p11);
    }

    #[test]
    fn test_parity_circuit() {
        // Use 4 qubits: |0000⟩ + |1111⟩ both have even parity
        let backend = SimulatorBackend::ideal(4).with_seed(42);

        // GHZ-like circuit
        let circuit = CircuitBuilder::new(4)
            .h(0)
            .cnot(0, 1)
            .cnot(1, 2)
            .cnot(2, 3)
            .build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // GHZ state: |0000⟩ + |1111⟩
        // Both have even parity (0 ones, 4 ones)
        let parity = result.parity_expectation();
        assert!(parity > 0.9, "Parity = {}", parity);
    }

    #[test]
    fn test_rz_gate() {
        let backend = SimulatorBackend::ideal(2).with_seed(42);

        // Test Rz gate
        let circuit = CircuitBuilder::new(1).h(0).rz(0, PI).h(0).build();

        let result = backend.execute(&circuit, 1000).unwrap();

        // H-Rz(π)-H should be equivalent to X
        // Starting from |0⟩, should get |1⟩
        let p1 = result.probability("1");
        assert!(p1 > 0.9, "P(1) = {}", p1);
    }

    #[test]
    fn test_qubit_limit() {
        let backend = SimulatorBackend::ideal(3);

        let circuit = CircuitBuilder::new(5).build();

        let result = backend.execute(&circuit, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_seed_reproducibility() {
        let backend1 = SimulatorBackend::from_depol(3, 0.02).unwrap().with_seed(42);
        let backend2 = SimulatorBackend::from_depol(3, 0.02).unwrap().with_seed(42);

        let circuit = CircuitBuilder::new(3).h(0).cnot(0, 1).cnot(1, 2).build();

        let result1 = backend1.execute(&circuit, 100).unwrap();
        let result2 = backend2.execute(&circuit, 100).unwrap();

        assert_eq!(result1.counts, result2.counts);
    }
}
