//! Scheduled gate representation for NISO
//!
//! Gantree: L4_Scheduling → ScheduledGate
//!
//! Provides time-tagged gate representation for circuit scheduling
//! and decoherence analysis.

use niso_core::{Gate, QubitId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A gate with timing information
/// Gantree: ScheduledGate // 스케줄된 게이트
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduledGate {
    /// Index in the original circuit
    /// Gantree: gate_idx: usize // 원본 인덱스
    pub gate_idx: usize,

    /// The gate operation
    pub gate: Gate,

    /// Start time in nanoseconds
    /// Gantree: start_time_ns: f64 // 시작 시간
    pub start_time_ns: f64,

    /// End time in nanoseconds
    /// Gantree: end_time_ns: f64 // 종료 시간
    pub end_time_ns: f64,
}

impl ScheduledGate {
    /// Create a new scheduled gate
    pub fn new(gate_idx: usize, gate: Gate, start_time_ns: f64, end_time_ns: f64) -> Self {
        Self {
            gate_idx,
            gate,
            start_time_ns,
            end_time_ns,
        }
    }

    /// Get gate duration in nanoseconds
    /// Gantree: duration(&self) -> f64 // 실행 시간
    pub fn duration(&self) -> f64 {
        self.end_time_ns - self.start_time_ns
    }

    /// Get affected qubits
    /// Gantree: qubits: Vec<QubitId> // 관련 큐비트
    pub fn qubits(&self) -> Vec<QubitId> {
        self.gate.qubits()
    }

    /// Check if this gate overlaps with a time interval
    pub fn overlaps(&self, start: f64, end: f64) -> bool {
        self.start_time_ns < end && self.end_time_ns > start
    }

    /// Check if this gate affects a specific qubit
    pub fn affects_qubit(&self, qubit: QubitId) -> bool {
        self.qubits().contains(&qubit)
    }

    /// Get duration in microseconds
    pub fn duration_us(&self) -> f64 {
        self.duration() / 1000.0
    }

    /// Check if gate is single-qubit
    pub fn is_single_qubit(&self) -> bool {
        self.gate.is_single_qubit()
    }

    /// Check if gate is two-qubit
    pub fn is_two_qubit(&self) -> bool {
        self.gate.is_two_qubit()
    }

    /// Check if gate is measurement
    pub fn is_measurement(&self) -> bool {
        self.gate.is_measurement()
    }
}

impl fmt::Display for ScheduledGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.1}-{:.1}ns] {} on {:?}",
            self.start_time_ns,
            self.end_time_ns,
            self.gate.name(),
            self.qubits()
        )
    }
}

// ============================================================================
// TimeSlot - For collision detection
// ============================================================================

/// Time slot for a qubit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSlot {
    /// Qubit ID
    pub qubit: QubitId,
    /// Start time
    pub start_ns: f64,
    /// End time
    pub end_ns: f64,
}

impl TimeSlot {
    /// Create a new time slot
    pub fn new(qubit: QubitId, start_ns: f64, end_ns: f64) -> Self {
        Self {
            qubit,
            start_ns,
            end_ns,
        }
    }

    /// Check overlap with another slot
    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.qubit == other.qubit && self.start_ns < other.end_ns && self.end_ns > other.start_ns
    }

    /// Duration
    pub fn duration(&self) -> f64 {
        self.end_ns - self.start_ns
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_gate_new() {
        let sg = ScheduledGate::new(0, Gate::H(0), 0.0, 35.0);
        assert_eq!(sg.gate_idx, 0);
        assert_eq!(sg.start_time_ns, 0.0);
        assert_eq!(sg.end_time_ns, 35.0);
    }

    #[test]
    fn test_duration() {
        let sg = ScheduledGate::new(0, Gate::Cnot(0, 1), 100.0, 400.0);
        assert_eq!(sg.duration(), 300.0);
        assert_eq!(sg.duration_us(), 0.3);
    }

    #[test]
    fn test_qubits() {
        let sg = ScheduledGate::new(0, Gate::Cnot(2, 3), 0.0, 300.0);
        assert_eq!(sg.qubits(), vec![2, 3]);
    }

    #[test]
    fn test_overlaps() {
        let sg = ScheduledGate::new(0, Gate::H(0), 100.0, 200.0);

        assert!(sg.overlaps(150.0, 250.0)); // Partial overlap
        assert!(sg.overlaps(50.0, 150.0)); // Partial overlap
        assert!(sg.overlaps(100.0, 200.0)); // Exact match
        assert!(sg.overlaps(110.0, 190.0)); // Contained
        assert!(!sg.overlaps(0.0, 100.0)); // Before (touching)
        assert!(!sg.overlaps(200.0, 300.0)); // After (touching)
    }

    #[test]
    fn test_affects_qubit() {
        let sg = ScheduledGate::new(0, Gate::Cnot(1, 2), 0.0, 300.0);

        assert!(sg.affects_qubit(1));
        assert!(sg.affects_qubit(2));
        assert!(!sg.affects_qubit(0));
        assert!(!sg.affects_qubit(3));
    }

    #[test]
    fn test_time_slot_overlap() {
        let s1 = TimeSlot::new(0, 0.0, 100.0);
        let s2 = TimeSlot::new(0, 50.0, 150.0);
        let s3 = TimeSlot::new(1, 50.0, 150.0);
        let s4 = TimeSlot::new(0, 100.0, 200.0);

        assert!(s1.overlaps(&s2)); // Same qubit, time overlap
        assert!(!s1.overlaps(&s3)); // Different qubit
        assert!(!s1.overlaps(&s4)); // Same qubit, touching but no overlap
    }
}
