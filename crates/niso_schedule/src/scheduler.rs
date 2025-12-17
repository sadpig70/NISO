//! Circuit scheduler for NISO
//!
//! Gantree: L4_Scheduling → Scheduler
//!
//! Provides ASAP (As Soon As Possible) scheduling for quantum circuits,
//! enabling timing analysis and decoherence estimation.

use crate::circuit_schedule::CircuitSchedule;
use crate::scheduled_gate::ScheduledGate;
use niso_core::{Circuit, Gate, QubitId};
use niso_noise::{GateTimes, NoiseVector};

/// Circuit scheduler
/// Gantree: Scheduler // 스케줄러
pub struct Scheduler;

impl Scheduler {
    // ========================================================================
    // ASAP Scheduling
    // ========================================================================

    /// Compute ASAP (As Soon As Possible) schedule
    /// Gantree: compute_asap(Circuit,GateTimes) -> CircuitSchedule // ASAP
    ///
    /// Each gate is scheduled at the earliest time when all its
    /// qubits become available.
    pub fn compute_asap(circuit: &Circuit, gate_times: &GateTimes) -> CircuitSchedule {
        let num_qubits = circuit.num_qubits();

        if circuit.is_empty() {
            return CircuitSchedule::empty(num_qubits);
        }

        // Gantree: init_qubit_available() // 가용 시간 초기화
        let mut qubit_available = vec![0.0; num_qubits];
        let mut scheduled_gates = Vec::with_capacity(circuit.gate_count());

        for (gate_idx, gate) in circuit.gates().iter().enumerate() {
            // Gantree: schedule_gate(gate,times) // 게이트 배치
            let (start_time, end_time) = Self::schedule_gate(gate, &qubit_available, gate_times);

            scheduled_gates.push(ScheduledGate::new(
                gate_idx,
                gate.clone(),
                start_time,
                end_time,
            ));

            // Gantree: update_availability(gate,end) // 가용 시간 갱신
            Self::update_availability(gate, end_time, &mut qubit_available, num_qubits);
        }

        let total_duration = qubit_available.iter().cloned().fold(0.0, f64::max);

        CircuitSchedule::new(scheduled_gates, total_duration, num_qubits, qubit_available)
    }

    /// Schedule a single gate
    fn schedule_gate(gate: &Gate, qubit_available: &[f64], gate_times: &GateTimes) -> (f64, f64) {
        let qubits = gate.qubits();
        let duration = gate_times.gate_duration(gate);

        // Find earliest start time
        let start_time = if qubits.is_empty() {
            // Global operation: wait for all qubits
            qubit_available.iter().cloned().fold(0.0, f64::max)
        } else {
            // Wait for all involved qubits
            qubits
                .iter()
                .filter_map(|&q| qubit_available.get(q))
                .cloned()
                .fold(0.0, f64::max)
        };

        let end_time = start_time + duration;
        (start_time, end_time)
    }

    /// Update qubit availability after a gate
    fn update_availability(
        gate: &Gate,
        end_time: f64,
        qubit_available: &mut [f64],
        num_qubits: usize,
    ) {
        let qubits = gate.qubits();

        if qubits.is_empty() {
            // Global operation: update all qubits
            for avail in qubit_available.iter_mut() {
                *avail = end_time;
            }
        } else {
            // Update involved qubits
            for &q in &qubits {
                if q < num_qubits {
                    qubit_available[q] = end_time;
                }
            }
        }
    }

    // ========================================================================
    // Decoherence Estimation
    // ========================================================================

    /// Estimate decoherence error from schedule
    /// Gantree: estimate_decoherence(schedule,NoiseVector) -> f64 // T2 에러
    pub fn estimate_decoherence(schedule: &CircuitSchedule, noise_vectors: &[NoiseVector]) -> f64 {
        schedule.estimate_decoherence(noise_vectors)
    }

    /// Compute idle error for a single qubit
    /// Gantree: compute_idle_error(idle,t2) // idle 에러
    pub fn compute_idle_error(idle_ns: f64, t2_us: f64) -> f64 {
        if t2_us <= 0.0 || t2_us.is_infinite() {
            return 0.0;
        }

        let idle_us = idle_ns / 1000.0;
        1.0 - (-idle_us / t2_us).exp()
    }

    // ========================================================================
    // Circuit Scoring
    // ========================================================================

    /// Score a circuit based on noise and timing
    /// Gantree: score_circuit(Circuit,NoiseVector,GateTimes) -> f64 // 점수
    ///
    /// Higher scores indicate better expected fidelity.
    pub fn score_circuit(
        circuit: &Circuit,
        noise_vectors: &[NoiseVector],
        gate_times: &GateTimes,
    ) -> f64 {
        let schedule = Self::compute_asap(circuit, gate_times);

        // Gate fidelity component
        let mut gate_fidelity = 1.0;
        for gate in circuit.gates() {
            let qubits = gate.qubits();
            if qubits.is_empty() {
                continue;
            }

            // Use worst qubit in gate
            let worst_error = qubits
                .iter()
                .filter_map(|&q| noise_vectors.get(q))
                .map(|nv| {
                    if gate.is_two_qubit() {
                        nv.gate_error_2q
                    } else {
                        nv.gate_error_1q
                    }
                })
                .fold(0.0, f64::max);

            gate_fidelity *= 1.0 - worst_error;
        }

        // Decoherence component
        let decoherence = schedule.estimate_decoherence(noise_vectors);
        let coherence_fidelity = 1.0 - decoherence;

        // Readout component
        let mut readout_fidelity = 1.0;
        for (q, nv) in noise_vectors.iter().enumerate() {
            if q < circuit.num_qubits() {
                readout_fidelity *= 1.0 - nv.readout_error;
            }
        }

        // Combined score
        gate_fidelity * coherence_fidelity * readout_fidelity
    }

    // ========================================================================
    // Analysis Utilities
    // ========================================================================

    /// Find bottleneck qubit (most idle time)
    pub fn find_bottleneck_qubit(schedule: &CircuitSchedule) -> Option<QubitId> {
        let idle_times = schedule.idle_times();

        idle_times
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(q, _)| q)
    }

    /// Estimate speedup from perfect parallelization
    pub fn potential_speedup(schedule: &CircuitSchedule) -> f64 {
        schedule.parallelism_factor()
    }

    /// Calculate scheduling efficiency
    /// 1.0 = perfect, lower = more idle time
    pub fn scheduling_efficiency(schedule: &CircuitSchedule) -> f64 {
        let total_active: f64 = schedule.gates().iter().map(|g| g.duration()).sum();

        let total_time = schedule.total_duration_ns() * schedule.num_qubits() as f64;

        if total_time > 0.0 {
            total_active / total_time
        } else {
            1.0
        }
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
    fn test_asap_simple() {
        let circuit = CircuitBuilder::new(2).h(0).h(1).cnot(0, 1).build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        assert_eq!(schedule.num_qubits(), 2);
        assert_eq!(schedule.num_gates(), 3);

        // H gates should be parallel
        let gates = schedule.gates();
        assert_eq!(gates[0].start_time_ns, 0.0);
        assert_eq!(gates[1].start_time_ns, 0.0);

        // CNOT should start after H gates
        assert!(gates[2].start_time_ns >= 35.0);
    }

    #[test]
    fn test_asap_chain() {
        let circuit = CircuitBuilder::new(3)
            .h(0)
            .cx_chain() // 0->1, 1->2
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        let gates = schedule.gates();

        // H starts at 0
        assert_eq!(gates[0].start_time_ns, 0.0);

        // First CNOT starts after H
        assert!(gates[1].start_time_ns >= 35.0);

        // Second CNOT starts after first CNOT
        assert!(gates[2].start_time_ns >= gates[1].end_time_ns);
    }

    #[test]
    fn test_parallelism() {
        // Fully parallel circuit
        let circuit = CircuitBuilder::new(5).h_layer().build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        // All H gates in parallel: factor should be ~5
        let factor = schedule.parallelism_factor();
        assert!((factor - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_idle_error() {
        // T2 = 60 μs = 60000 ns
        let error = Scheduler::compute_idle_error(60000.0, 60.0);

        // At t = T2, error ≈ 0.632
        assert!((error - 0.6321205588).abs() < 1e-6);
    }

    #[test]
    fn test_score_circuit() {
        let circuit = CircuitBuilder::new(2).h(0).cnot(0, 1).measure_all().build();

        let noise_vectors = vec![
            NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01),
            NoiseVector::new(1, 100.0, 60.0, 0.001, 0.01, 0.01),
        ];

        let times = GateTimes::default();
        let score = Scheduler::score_circuit(&circuit, &noise_vectors, &times);

        // Score should be positive and less than 1
        assert!(score > 0.0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_bottleneck_qubit() {
        // q1 waits for q0
        let circuit = CircuitBuilder::new(2).h(0).h(0).h(0).cnot(0, 1).build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        let bottleneck = Scheduler::find_bottleneck_qubit(&schedule);

        // q1 should be the bottleneck (more idle time)
        assert_eq!(bottleneck, Some(1));
    }

    #[test]
    fn test_scheduling_efficiency() {
        // Fully sequential
        let sequential = CircuitBuilder::new(2).h(0).cnot(0, 1).h(1).build();

        // Fully parallel
        let parallel = CircuitBuilder::new(2).h(0).h(1).build();

        let times = GateTimes::default();

        let eff_seq =
            Scheduler::scheduling_efficiency(&Scheduler::compute_asap(&sequential, &times));
        let eff_par = Scheduler::scheduling_efficiency(&Scheduler::compute_asap(&parallel, &times));

        // Parallel should be more efficient
        assert!(eff_par > eff_seq);
    }

    #[test]
    fn test_tqqc_circuit_schedule() {
        // TQQC 7-qubit parity circuit
        use niso_core::{BasisString, EntanglerType};

        let basis = BasisString::all_x(7);
        let circuit = CircuitBuilder::new(7)
            .tqqc_parity(0.5, 0.1, EntanglerType::Cx, &basis)
            .build();

        let times = GateTimes::default();
        let schedule = Scheduler::compute_asap(&circuit, &times);

        // Verify structure
        assert_eq!(schedule.num_qubits(), 7);
        assert_eq!(schedule.count_2q(), 6); // 6 CNOTs

        // Duration should be reasonable
        // H (35) + 6*CNOT sequential worst case (1800) + Rz (0) + 7*H (245) + measure (5000)
        // With parallelism, should be less
        assert!(schedule.total_duration_ns() < 10000.0);
    }
}
