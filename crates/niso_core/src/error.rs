//! Error types for NISO
//!
//! Gantree: L0_Foundation → Errors
//!
//! Comprehensive error handling for the NISO system.

// Error variant fields are self-documenting via error messages
#![allow(missing_docs)]

use thiserror::Error;

/// Main error type for NISO
/// Gantree: NisoError // enum
#[derive(Error, Debug, Clone, PartialEq)]
pub enum NisoError {
    // ========================================================================
    // Validation Errors
    // ========================================================================
    /// Probability value out of range [0, 1]
    /// Gantree: InvalidProbability(f64) // 확률 범위
    #[error("Invalid probability {0}: must be in range [0, 1]")]
    InvalidProbability(f64),

    /// Qubit index out of range
    /// Gantree: QubitOutOfRange{{q,max}} // 큐비트 범위
    #[error("Qubit {qubit} out of range: max is {max}")]
    QubitOutOfRange { qubit: usize, max: usize },

    /// Invalid T2 value (must be <= 2*T1)
    /// Gantree: InvalidT2{{t2,t1}} // T2>2*T1
    #[error("Invalid T2 ({t2_us:.2}µs): must be <= 2*T1 ({t1_us:.2}µs)")]
    InvalidT2 { t2_us: f64, t1_us: f64 },

    /// Invalid noise level
    #[error("Invalid noise level {0}: must be in range [0, 0.06]")]
    InvalidNoiseLevel(f64),

    /// Invalid bitstring format
    #[error("Invalid bitstring '{0}': must contain only '0' and '1'")]
    InvalidBitstring(String),

    /// Invalid basis character
    #[error("Invalid basis '{0}': must be X, Y, or Z")]
    InvalidBasis(String),

    /// Invalid angle
    #[error("Invalid angle {0}: must be finite")]
    InvalidAngle(f64),

    // ========================================================================
    // Circuit Errors
    // ========================================================================
    /// Empty circuit
    /// Gantree: EmptyCircuit // 빈 회로
    #[error("Circuit is empty")]
    EmptyCircuit,

    /// Gate on non-existent qubit
    #[error("Gate references qubit {qubit} but circuit has only {num_qubits} qubits")]
    GateQubitMismatch { qubit: usize, num_qubits: usize },

    /// Invalid gate parameter
    #[error("Invalid gate parameter: {0}")]
    InvalidGateParameter(String),

    /// Circuit too deep for TQQC
    #[error("Circuit depth {depth} exceeds maximum {max_depth}")]
    CircuitTooDeep { depth: usize, max_depth: usize },

    /// Topology violation (qubits not connected)
    /// Gantree: TopologyViolation{{q1,q2}} // 연결 위반
    #[error("Topology violation: qubits {q1} and {q2} are not connected")]
    TopologyViolation { q1: usize, q2: usize },

    /// Invalid QASM format
    #[error("Invalid QASM: {0}")]
    InvalidQasm(String),

    // ========================================================================
    // Topology Errors
    // ========================================================================
    /// Empty coupling map
    #[error("Coupling map is empty")]
    EmptyCouplingMap,

    /// Invalid coupling
    #[error("Invalid coupling ({0}, {1}): qubits must be different")]
    InvalidCoupling(usize, usize),

    /// Path not found between qubits
    #[error("No path found between qubits {0} and {1}")]
    PathNotFound(usize, usize),

    // ========================================================================
    // Backend Errors
    // ========================================================================
    /// Backend execution error
    /// Gantree: BackendError(String) // 백엔드
    #[error("Backend error: {0}")]
    BackendError(String),

    /// Backend not available
    #[error("Backend '{0}' not available")]
    BackendNotAvailable(String),

    /// Shots out of range
    #[error("Shots {0} out of range [{1}, {2}]")]
    ShotsOutOfRange(u64, u64, u64),

    // ========================================================================
    // Calibration Errors
    // ========================================================================
    /// Calibration error
    /// Gantree: CalibrationError(String) // 캘리브레이션
    #[error("Calibration error: {0}")]
    CalibrationError(String),

    /// Calibration expired
    #[error("Calibration data expired: last updated {0}")]
    CalibrationExpired(String),

    // ========================================================================
    // TQQC Errors
    // ========================================================================
    /// Convergence failed
    /// Gantree: ConvergenceFailed{{iters}} // 수렴 실패
    #[error("Convergence failed after {iterations} iterations")]
    ConvergenceFailed { iterations: usize },

    /// TQQC configuration error
    #[error("TQQC configuration error: {0}")]
    TqqcConfigError(String),

    /// Statistical test error
    #[error("Statistical test error: {0}")]
    StatisticalTestError(String),

    /// Noise level exceeds critical point
    #[error("Noise level {noise:.4} exceeds critical point {critical:.4} for {qubits} qubits")]
    NoiseExceedsCritical {
        noise: f64,
        critical: f64,
        qubits: usize,
    },

    // ========================================================================
    // I/O Errors
    // ========================================================================
    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// File I/O error
    #[error("File error: {0}")]
    FileError(String),

    // ========================================================================
    // Generic Errors
    // ========================================================================
    /// Internal error (should not happen)
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type alias for NISO operations
/// Gantree: NisoResult<T> // type alias
pub type NisoResult<T> = Result<T, NisoError>;

// ============================================================================
// Error Conversion Helpers
// ============================================================================

impl From<serde_json::Error> for NisoError {
    fn from(err: serde_json::Error) -> Self {
        NisoError::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for NisoError {
    fn from(err: std::io::Error) -> Self {
        NisoError::FileError(err.to_string())
    }
}

// ============================================================================
// Error Helpers
// ============================================================================

impl NisoError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            NisoError::ConvergenceFailed { .. }
                | NisoError::CalibrationExpired(_)
                | NisoError::NoiseExceedsCritical { .. }
        )
    }

    /// Check if error is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            NisoError::InvalidProbability(_)
                | NisoError::QubitOutOfRange { .. }
                | NisoError::InvalidT2 { .. }
                | NisoError::InvalidNoiseLevel(_)
                | NisoError::InvalidBitstring(_)
                | NisoError::InvalidBasis(_)
                | NisoError::InvalidAngle(_)
        )
    }

    /// Check if error is a circuit error
    pub fn is_circuit_error(&self) -> bool {
        matches!(
            self,
            NisoError::EmptyCircuit
                | NisoError::GateQubitMismatch { .. }
                | NisoError::InvalidGateParameter(_)
                | NisoError::CircuitTooDeep { .. }
                | NisoError::TopologyViolation { .. }
                | NisoError::InvalidQasm(_)
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NisoError::InvalidProbability(1.5);
        assert!(err.to_string().contains("1.5"));
    }

    #[test]
    fn test_qubit_out_of_range() {
        let err = NisoError::QubitOutOfRange { qubit: 10, max: 7 };
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("7"));
    }

    #[test]
    fn test_is_recoverable() {
        assert!(NisoError::ConvergenceFailed { iterations: 100 }.is_recoverable());
        assert!(!NisoError::EmptyCircuit.is_recoverable());
    }

    #[test]
    fn test_is_validation_error() {
        assert!(NisoError::InvalidProbability(1.5).is_validation_error());
        assert!(!NisoError::BackendError("test".into()).is_validation_error());
    }
}
