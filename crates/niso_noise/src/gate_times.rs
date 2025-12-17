//! Gate timing information for NISO
//!
//! Gantree: L2_Noise → GateTimes
//!
//! Provides gate execution times for different quantum hardware
//! platforms, enabling accurate circuit scheduling and decoherence
//! estimation.

use niso_core::{Circuit, Gate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Gate timing configuration
/// Gantree: GateTimes // 게이트 시간
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateTimes {
    /// Single-qubit gate time in nanoseconds
    /// Gantree: single_qubit_ns: f64 // 1Q 시간
    pub single_qubit_ns: f64,

    /// Two-qubit gate time in nanoseconds
    /// Gantree: two_qubit_ns: f64 // 2Q 시간
    pub two_qubit_ns: f64,

    /// Measurement time in nanoseconds
    /// Gantree: measurement_ns: f64 // 측정 시간
    pub measurement_ns: f64,

    /// Per-gate overrides (optional)
    gate_overrides: HashMap<String, f64>,
}

impl GateTimes {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new gate times
    pub fn new(single_qubit_ns: f64, two_qubit_ns: f64, measurement_ns: f64) -> Self {
        Self {
            single_qubit_ns,
            two_qubit_ns,
            measurement_ns,
            gate_overrides: HashMap::new(),
        }
    }

    /// Default IBM superconducting processor timings
    /// Gantree: default() -> Self // IBM 기본값
    pub fn default_ibm() -> Self {
        Self {
            single_qubit_ns: 35.0,  // ~35 ns for SX gates
            two_qubit_ns: 300.0,    // ~300 ns for CX gates
            measurement_ns: 5000.0, // ~5 μs
            gate_overrides: HashMap::new(),
        }
    }

    /// Superconducting processor (general)
    /// Gantree: superconducting() -> Self // 초전도
    pub fn superconducting() -> Self {
        Self::default_ibm()
    }

    /// Trapped ion processor timings
    /// Gantree: trapped_ion() -> Self // 이온트랩
    pub fn trapped_ion() -> Self {
        Self {
            single_qubit_ns: 10_000.0, // ~10 μs
            two_qubit_ns: 200_000.0,   // ~200 μs
            measurement_ns: 100_000.0, // ~100 μs
            gate_overrides: HashMap::new(),
        }
    }

    /// Neutral atom processor timings
    pub fn neutral_atom() -> Self {
        Self {
            single_qubit_ns: 1_000.0, // ~1 μs
            two_qubit_ns: 1_000.0,    // ~1 μs (Rydberg interaction)
            measurement_ns: 50_000.0, // ~50 μs
            gate_overrides: HashMap::new(),
        }
    }

    /// Photonic processor timings
    pub fn photonic() -> Self {
        Self {
            single_qubit_ns: 10.0,   // ~10 ns (optical components)
            two_qubit_ns: 100.0,     // ~100 ns (fusion gates)
            measurement_ns: 1_000.0, // ~1 μs (SPD detection)
            gate_overrides: HashMap::new(),
        }
    }

    // ========================================================================
    // Gate Time Overrides
    // ========================================================================

    /// Set custom time for a specific gate
    pub fn with_gate_time(mut self, gate_name: &str, time_ns: f64) -> Self {
        self.gate_overrides
            .insert(gate_name.to_lowercase(), time_ns);
        self
    }

    /// Set IBM-specific gate times
    pub fn with_ibm_defaults(mut self) -> Self {
        // Virtual gates (effectively 0)
        self.gate_overrides.insert("rz".to_string(), 0.0);
        self.gate_overrides.insert("z".to_string(), 0.0);
        self.gate_overrides.insert("id".to_string(), 0.0);

        // Standard gates
        self.gate_overrides.insert("h".to_string(), 35.0);
        self.gate_overrides.insert("x".to_string(), 35.0);
        self.gate_overrides.insert("y".to_string(), 35.0);
        self.gate_overrides.insert("sx".to_string(), 35.0);
        self.gate_overrides.insert("s".to_string(), 35.0);
        self.gate_overrides.insert("sdg".to_string(), 35.0);
        self.gate_overrides.insert("t".to_string(), 35.0);
        self.gate_overrides.insert("tdg".to_string(), 35.0);
        self.gate_overrides.insert("rx".to_string(), 35.0);
        self.gate_overrides.insert("ry".to_string(), 35.0);

        // Two-qubit gates
        self.gate_overrides.insert("cx".to_string(), 300.0);
        self.gate_overrides.insert("cz".to_string(), 300.0);
        self.gate_overrides.insert("ecr".to_string(), 300.0);
        self.gate_overrides.insert("swap".to_string(), 900.0); // 3 CNOTs

        // Reset and measurement
        self.gate_overrides.insert("reset".to_string(), 1000.0);
        self.gate_overrides.insert("measure".to_string(), 5000.0);

        self
    }

    // ========================================================================
    // Time Calculations
    // ========================================================================

    /// Get duration for a specific gate
    /// Gantree: gate_duration(&self,Gate) -> f64 // 게이트별 시간
    pub fn gate_duration(&self, gate: &Gate) -> f64 {
        let gate_name = gate.name();

        // Check overrides first
        if let Some(&time) = self.gate_overrides.get(gate_name) {
            return time;
        }

        // Default based on gate type
        if gate.is_measurement() {
            self.measurement_ns
        } else if gate.is_two_qubit() {
            self.two_qubit_ns
        } else if gate.is_three_qubit() {
            // Approximate as 6 CNOTs (Toffoli decomposition)
            self.two_qubit_ns * 6.0
        } else if gate.is_single_qubit() {
            self.single_qubit_ns
        } else if gate.is_barrier() {
            0.0
        } else {
            self.single_qubit_ns // Default
        }
    }

    /// Calculate total circuit duration (sequential, no parallelism)
    /// Gantree: circuit_duration(&self,Circuit) -> f64 // 회로 시간
    pub fn circuit_duration_sequential(&self, circuit: &Circuit) -> f64 {
        circuit.gates().iter().map(|g| self.gate_duration(g)).sum()
    }

    /// Calculate circuit duration with ASAP scheduling
    /// Returns (total_duration, per_qubit_durations)
    pub fn circuit_duration_asap(&self, circuit: &Circuit) -> (f64, Vec<f64>) {
        let num_qubits = circuit.num_qubits();
        let mut qubit_available = vec![0.0; num_qubits];

        for gate in circuit.gates() {
            let qubits = gate.qubits();
            let duration = self.gate_duration(gate);

            if qubits.is_empty() {
                // Global operation (MeasureAll, Barrier)
                let max_time = qubit_available.iter().cloned().fold(0.0, f64::max);
                for t in &mut qubit_available {
                    *t = max_time + duration;
                }
            } else {
                // Find latest available time among gate qubits
                let start_time = qubits
                    .iter()
                    .filter_map(|&q| qubit_available.get(q))
                    .cloned()
                    .fold(0.0, f64::max);

                let end_time = start_time + duration;

                // Update availability
                for &q in &qubits {
                    if q < num_qubits {
                        qubit_available[q] = end_time;
                    }
                }
            }
        }

        let total = qubit_available.iter().cloned().fold(0.0, f64::max);
        (total, qubit_available)
    }

    /// Convert duration to microseconds
    pub fn to_microseconds(ns: f64) -> f64 {
        ns / 1000.0
    }

    /// Convert duration to seconds
    pub fn to_seconds(ns: f64) -> f64 {
        ns * 1e-9
    }

    // ========================================================================
    // Analysis
    // ========================================================================

    /// Estimate idle time per qubit
    pub fn estimate_idle_times(&self, circuit: &Circuit) -> Vec<f64> {
        let (total_duration, qubit_times) = self.circuit_duration_asap(circuit);

        // Calculate active time per qubit
        let mut active_times = vec![0.0; circuit.num_qubits()];

        for gate in circuit.gates() {
            let duration = self.gate_duration(gate);
            for &q in &gate.qubits() {
                if q < active_times.len() {
                    active_times[q] += duration;
                }
            }
        }

        // Idle time = total duration - active time
        active_times
            .iter()
            .enumerate()
            .map(|(i, &active)| {
                let qubit_total = qubit_times.get(i).copied().unwrap_or(total_duration);
                (qubit_total - active).max(0.0)
            })
            .collect()
    }

    /// Calculate parallelism factor
    /// 1.0 = fully sequential, higher = more parallel
    pub fn parallelism_factor(&self, circuit: &Circuit) -> f64 {
        let sequential = self.circuit_duration_sequential(circuit);
        let (parallel, _) = self.circuit_duration_asap(circuit);

        if parallel > 0.0 {
            sequential / parallel
        } else {
            1.0
        }
    }
}

impl Default for GateTimes {
    fn default() -> Self {
        Self::default_ibm().with_ibm_defaults()
    }
}

impl fmt::Display for GateTimes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GateTimes(1Q={:.0}ns, 2Q={:.0}ns, meas={:.0}ns)",
            self.single_qubit_ns, self.two_qubit_ns, self.measurement_ns
        )
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
    fn test_gate_times_default() {
        let times = GateTimes::default();
        assert_eq!(times.single_qubit_ns, 35.0);
        assert_eq!(times.two_qubit_ns, 300.0);
    }

    #[test]
    fn test_gate_duration() {
        let times = GateTimes::default();

        assert_eq!(times.gate_duration(&Gate::H(0)), 35.0);
        assert_eq!(times.gate_duration(&Gate::Cnot(0, 1)), 300.0);
        assert_eq!(times.gate_duration(&Gate::Rz(0, 1.0)), 0.0); // Virtual
    }

    #[test]
    fn test_circuit_duration_sequential() {
        let times = GateTimes::default();
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).build();

        let duration = times.circuit_duration_sequential(&circuit);
        // H (35) + CNOT (300) = 335
        assert!((duration - 335.0).abs() < 1e-6);
    }

    #[test]
    fn test_circuit_duration_asap() {
        let times = GateTimes::default();
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .h(1) // Parallel with h(0)
            .cnot(0, 1)
            .build();

        let (duration, _) = times.circuit_duration_asap(&circuit);
        // H on q0 and q1 in parallel (35), then CNOT (300) = 335
        assert!((duration - 335.0).abs() < 1e-6);
    }

    #[test]
    fn test_parallelism_factor() {
        let times = GateTimes::default();

        // Parallel circuit
        let parallel_circuit = CircuitBuilder::new(3).h(0).h(1).h(2).build();

        let factor = times.parallelism_factor(&parallel_circuit);
        // All H gates in parallel: 3 * 35 / 35 = 3.0
        assert!((factor - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_idle_times() {
        let times = GateTimes::default();
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).h(0).build();

        let idle = times.estimate_idle_times(&circuit);
        assert_eq!(idle.len(), 2);
        // q1 is idle during first H
        assert!(idle[1] > 0.0);
    }

    #[test]
    fn test_trapped_ion_times() {
        let times = GateTimes::trapped_ion();
        assert!(times.single_qubit_ns > 1000.0); // μs scale
        assert!(times.two_qubit_ns > 100_000.0);
    }

    #[test]
    fn test_with_gate_override() {
        let times = GateTimes::default().with_gate_time("h", 50.0);

        assert_eq!(times.gate_duration(&Gate::H(0)), 50.0);
    }
}
