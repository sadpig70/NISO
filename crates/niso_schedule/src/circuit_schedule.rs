//! Circuit schedule for NISO
//!
//! Gantree: L4_Scheduling → CircuitSchedule
//!
//! Provides complete circuit timing information including
//! idle time analysis and decoherence estimation.

use crate::scheduled_gate::ScheduledGate;
use niso_core::QubitId;
use niso_noise::NoiseVector;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Complete circuit schedule with timing analysis
/// Gantree: CircuitSchedule // 전체 스케줄
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitSchedule {
    /// Scheduled gates in time order
    /// Gantree: gates: Vec<ScheduledGate> // 스케줄 목록
    gates: Vec<ScheduledGate>,

    /// Total circuit duration in nanoseconds
    /// Gantree: total_duration_ns: f64 // 총 시간
    total_duration_ns: f64,

    /// Number of qubits
    /// Gantree: num_qubits: usize // 큐비트 수
    num_qubits: usize,

    /// Per-qubit end times
    qubit_end_times: Vec<f64>,
}

impl CircuitSchedule {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create a new circuit schedule
    pub fn new(
        gates: Vec<ScheduledGate>,
        total_duration_ns: f64,
        num_qubits: usize,
        qubit_end_times: Vec<f64>,
    ) -> Self {
        Self {
            gates,
            total_duration_ns,
            num_qubits,
            qubit_end_times,
        }
    }

    /// Create empty schedule
    pub fn empty(num_qubits: usize) -> Self {
        Self {
            gates: Vec::new(),
            total_duration_ns: 0.0,
            num_qubits,
            qubit_end_times: vec![0.0; num_qubits],
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get scheduled gates
    pub fn gates(&self) -> &[ScheduledGate] {
        &self.gates
    }

    /// Get total duration in nanoseconds
    pub fn total_duration_ns(&self) -> f64 {
        self.total_duration_ns
    }

    /// Get total duration in microseconds
    pub fn total_duration_us(&self) -> f64 {
        self.total_duration_ns / 1000.0
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get number of gates
    pub fn num_gates(&self) -> usize {
        self.gates.len()
    }

    /// Get per-qubit end times
    pub fn qubit_end_times(&self) -> &[f64] {
        &self.qubit_end_times
    }

    // ========================================================================
    // Critical Path Analysis
    // ========================================================================

    /// Calculate critical path depth (longest dependency chain)
    /// Gantree: critical_path_depth: usize // 임계 깊이
    pub fn critical_path_depth(&self) -> usize {
        // Count unique time layers
        let mut layer_times: Vec<f64> = self.gates.iter().map(|g| g.start_time_ns).collect();
        layer_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        layer_times.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        layer_times.len()
    }

    /// Find the critical path (longest timing chain)
    pub fn critical_path(&self) -> Vec<usize> {
        // Find qubit with maximum end time
        let critical_qubit = self
            .qubit_end_times
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Collect gates on critical path
        self.gates
            .iter()
            .filter(|g| g.affects_qubit(critical_qubit))
            .map(|g| g.gate_idx)
            .collect()
    }

    // ========================================================================
    // Idle Time Analysis
    // ========================================================================

    /// Calculate idle time per qubit
    /// Gantree: idle_times(&self) -> Vec<f64> // 큐비트별 idle
    pub fn idle_times(&self) -> Vec<f64> {
        let mut active_times = vec![0.0; self.num_qubits];

        for gate in &self.gates {
            let duration = gate.duration();
            for &q in &gate.qubits() {
                if q < self.num_qubits {
                    active_times[q] += duration;
                }
            }
        }

        // Idle = qubit_end_time - active_time
        self.qubit_end_times
            .iter()
            .zip(active_times.iter())
            .map(|(&end, &active)| (end - active).max(0.0))
            .collect()
    }

    /// Calculate total idle time across all qubits
    /// Gantree: total_idle_time(&self) -> f64 // 총 idle
    pub fn total_idle_time(&self) -> f64 {
        self.idle_times().iter().sum()
    }

    /// Calculate weighted idle time (relative to T2)
    /// Gantree: weighted_idle_time(&self) -> f64 // 가중 idle
    pub fn weighted_idle_time(&self, noise_vectors: &[NoiseVector]) -> f64 {
        let idle = self.idle_times();

        idle.iter()
            .enumerate()
            .map(|(q, &idle_ns)| {
                let t2_ns = noise_vectors
                    .get(q)
                    .map(|nv| nv.t2 * 1000.0) // μs to ns
                    .unwrap_or(60_000.0);

                if t2_ns > 0.0 {
                    idle_ns / t2_ns
                } else {
                    0.0
                }
            })
            .sum()
    }

    // ========================================================================
    // Parallelism Analysis
    // ========================================================================

    /// Calculate parallelism factor
    /// Gantree: parallelism_factor(&self) -> f64 // 병렬화율
    pub fn parallelism_factor(&self) -> f64 {
        if self.total_duration_ns <= 0.0 || self.gates.is_empty() {
            return 1.0;
        }

        let sequential_duration: f64 = self.gates.iter().map(|g| g.duration()).sum();

        sequential_duration / self.total_duration_ns
    }

    /// Count gates executing at a given time
    pub fn concurrent_gates_at(&self, time_ns: f64) -> usize {
        self.gates
            .iter()
            .filter(|g| g.start_time_ns <= time_ns && g.end_time_ns > time_ns)
            .count()
    }

    /// Get maximum concurrent gates
    pub fn max_concurrent_gates(&self) -> usize {
        // Sample at gate start times
        let mut max_concurrent = 0;

        for gate in &self.gates {
            let concurrent = self.concurrent_gates_at(gate.start_time_ns);
            max_concurrent = max_concurrent.max(concurrent);
        }

        max_concurrent
    }

    // ========================================================================
    // Decoherence Estimation
    // ========================================================================

    /// Estimate decoherence error from idle times
    pub fn estimate_decoherence(&self, noise_vectors: &[NoiseVector]) -> f64 {
        let idle = self.idle_times();

        let total_error: f64 = idle
            .iter()
            .enumerate()
            .map(|(q, &idle_ns)| {
                let idle_us = idle_ns / 1000.0;
                noise_vectors
                    .get(q)
                    .map(|nv| nv.estimate_decoherence(idle_us))
                    .unwrap_or(0.0)
            })
            .sum();

        // Average per qubit
        if self.num_qubits > 0 {
            total_error / self.num_qubits as f64
        } else {
            0.0
        }
    }

    /// Estimate T1 relaxation error
    pub fn estimate_t1_error(&self, noise_vectors: &[NoiseVector]) -> f64 {
        let idle = self.idle_times();

        let total_error: f64 = idle
            .iter()
            .enumerate()
            .map(|(q, &idle_ns)| {
                let idle_us = idle_ns / 1000.0;
                noise_vectors
                    .get(q)
                    .map(|nv| nv.estimate_t1_error(idle_us))
                    .unwrap_or(0.0)
            })
            .sum();

        if self.num_qubits > 0 {
            total_error / self.num_qubits as f64
        } else {
            0.0
        }
    }

    // ========================================================================
    // Gate Statistics
    // ========================================================================

    /// Count single-qubit gates
    pub fn count_1q(&self) -> usize {
        self.gates.iter().filter(|g| g.is_single_qubit()).count()
    }

    /// Count two-qubit gates
    pub fn count_2q(&self) -> usize {
        self.gates.iter().filter(|g| g.is_two_qubit()).count()
    }

    /// Count measurements
    pub fn count_measurements(&self) -> usize {
        self.gates.iter().filter(|g| g.is_measurement()).count()
    }

    /// Get gates on a specific qubit
    pub fn gates_on_qubit(&self, qubit: QubitId) -> Vec<&ScheduledGate> {
        self.gates
            .iter()
            .filter(|g| g.affects_qubit(qubit))
            .collect()
    }

    /// Get gates in a time range
    pub fn gates_in_range(&self, start_ns: f64, end_ns: f64) -> Vec<&ScheduledGate> {
        self.gates
            .iter()
            .filter(|g| g.overlaps(start_ns, end_ns))
            .collect()
    }
}

impl fmt::Display for CircuitSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CircuitSchedule:")?;
        writeln!(f, "  Qubits: {}", self.num_qubits)?;
        writeln!(f, "  Gates: {}", self.gates.len())?;
        writeln!(f, "  Duration: {:.2} μs", self.total_duration_us())?;
        writeln!(f, "  Parallelism: {:.2}x", self.parallelism_factor())?;
        writeln!(f, "  Critical depth: {}", self.critical_path_depth())?;
        writeln!(f, "  Total idle: {:.2} μs", self.total_idle_time() / 1000.0)?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use niso_core::Gate;

    fn make_test_schedule() -> CircuitSchedule {
        let gates = vec![
            ScheduledGate::new(0, Gate::H(0), 0.0, 35.0),
            ScheduledGate::new(1, Gate::H(1), 0.0, 35.0),
            ScheduledGate::new(2, Gate::Cnot(0, 1), 35.0, 335.0),
            ScheduledGate::new(3, Gate::MeasureAll, 335.0, 5335.0),
        ];

        CircuitSchedule::new(gates, 5335.0, 2, vec![5335.0, 5335.0])
    }

    #[test]
    fn test_schedule_basic() {
        let schedule = make_test_schedule();

        assert_eq!(schedule.num_qubits(), 2);
        assert_eq!(schedule.num_gates(), 4);
        assert_eq!(schedule.total_duration_ns(), 5335.0);
    }

    #[test]
    fn test_idle_times() {
        // Create schedule where q1 waits for q0
        let gates = vec![
            ScheduledGate::new(0, Gate::H(0), 0.0, 35.0),
            ScheduledGate::new(1, Gate::H(0), 35.0, 70.0),
            ScheduledGate::new(2, Gate::Cnot(0, 1), 70.0, 370.0),
        ];

        let schedule = CircuitSchedule::new(gates, 370.0, 2, vec![370.0, 370.0]);
        let idle = schedule.idle_times();

        // q0: active 35+35+300=370, idle=0
        assert!((idle[0]).abs() < 1e-6);
        // q1: active 300, end=370, idle=70
        assert!((idle[1] - 70.0).abs() < 1e-6);
    }

    #[test]
    fn test_parallelism_factor() {
        let schedule = make_test_schedule();
        let factor = schedule.parallelism_factor();

        // Sequential: 35+35+300+5000 = 5370
        // Parallel: 5335
        // Factor: 5370/5335 ≈ 1.006
        assert!(factor > 1.0);
    }

    #[test]
    fn test_critical_path_depth() {
        let schedule = make_test_schedule();
        let depth = schedule.critical_path_depth();

        // Layers: 0 (H,H), 35 (CNOT), 335 (measure) = 3
        assert_eq!(depth, 3);
    }

    #[test]
    fn test_gate_counts() {
        let schedule = make_test_schedule();

        assert_eq!(schedule.count_1q(), 2);
        assert_eq!(schedule.count_2q(), 1);
        assert_eq!(schedule.count_measurements(), 1);
    }

    #[test]
    fn test_gates_on_qubit() {
        let schedule = make_test_schedule();

        let q0_gates = schedule.gates_on_qubit(0);
        assert_eq!(q0_gates.len(), 2); // H and CNOT (MeasureAll has empty qubits)
    }

    #[test]
    fn test_concurrent_gates() {
        let schedule = make_test_schedule();

        // At t=0, two H gates are concurrent
        assert_eq!(schedule.concurrent_gates_at(0.0), 2);

        // At t=100, only CNOT is executing
        assert_eq!(schedule.concurrent_gates_at(100.0), 1);
    }

    #[test]
    fn test_decoherence_estimation() {
        let schedule = make_test_schedule();

        let noise_vectors = vec![
            NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01),
            NoiseVector::new(1, 100.0, 60.0, 0.001, 0.01, 0.01),
        ];

        let decoherence = schedule.estimate_decoherence(&noise_vectors);

        // Should be non-negative
        assert!(decoherence >= 0.0);
    }
}
