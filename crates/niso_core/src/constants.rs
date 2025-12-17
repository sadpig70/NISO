//! Constants for NISO
//!
//! Gantree: L0_Foundation → Constants
//!
//! Physical constants, TQQC parameters, and statistical thresholds.
//! All values are based on TQQC v2.2.0 specification and IBM hardware defaults.

// ============================================================================
// Physics Constants
// Gantree: physics // 물리 상수
// ============================================================================

pub mod physics {
    //! Physical constants for quantum hardware
    //! Based on IBM Quantum specifications (2024-2025)

    /// Single-qubit gate time in nanoseconds (IBM SX gate)
    /// Gantree: GATE_TIME_1Q_NS: f64 = 35.0
    pub const GATE_TIME_1Q_NS: f64 = 35.0;

    /// Two-qubit gate time in nanoseconds (IBM CX gate)
    /// Gantree: GATE_TIME_2Q_NS: f64 = 300.0
    pub const GATE_TIME_2Q_NS: f64 = 300.0;

    /// Measurement time in nanoseconds
    /// Gantree: MEASUREMENT_NS: f64 = 5000.0
    pub const MEASUREMENT_NS: f64 = 5000.0;

    /// Default T1 relaxation time in microseconds
    /// Gantree: DEFAULT_T1_US: f64 = 100.0
    pub const DEFAULT_T1_US: f64 = 100.0;

    /// Default T2 dephasing time in microseconds
    /// Gantree: DEFAULT_T2_US: f64 = 60.0
    pub const DEFAULT_T2_US: f64 = 60.0;

    /// Minimum recommended T1 for TQQC (microseconds)
    pub const MIN_T1_US: f64 = 50.0;

    /// Minimum recommended T2 for TQQC (microseconds)
    pub const MIN_T2_US: f64 = 30.0;

    /// Gate times in seconds for individual gates
    pub mod gate_times_s {
        /// H gate time (seconds)
        pub const H: f64 = 30e-9;
        /// X gate time (seconds)
        pub const X: f64 = 30e-9;
        /// Y gate time (seconds)
        pub const Y: f64 = 30e-9;
        /// Z gate time (seconds) - virtual, effectively 0
        pub const Z: f64 = 0.0;
        /// Rz gate time (seconds) - virtual
        pub const RZ: f64 = 0.0;
        /// Rx gate time (seconds)
        pub const RX: f64 = 30e-9;
        /// Ry gate time (seconds)
        pub const RY: f64 = 30e-9;
        /// SX gate time (seconds)
        pub const SX: f64 = 30e-9;
        /// S gate time (seconds)
        pub const S: f64 = 30e-9;
        /// Sdg gate time (seconds)
        pub const SDG: f64 = 30e-9;
        /// T gate time (seconds)
        pub const T: f64 = 30e-9;
        /// Tdg gate time (seconds)
        pub const TDG: f64 = 30e-9;
        /// CNOT/CX gate time (seconds)
        pub const CX: f64 = 300e-9;
        /// CZ gate time (seconds)
        pub const CZ: f64 = 300e-9;
        /// SWAP gate time (seconds) - 3 CNOTs
        pub const SWAP: f64 = 900e-9;
    }

    /// Convert microseconds to seconds
    #[inline]
    pub const fn us_to_s(us: f64) -> f64 {
        us * 1e-6
    }

    /// Convert nanoseconds to seconds
    #[inline]
    pub const fn ns_to_s(ns: f64) -> f64 {
        ns * 1e-9
    }
}

// ============================================================================
// TQQC Constants
// Gantree: tqqc // TQQC 상수
// ============================================================================

pub mod tqqc {
    //! TQQC algorithm parameters
    //! Based on TQQC v2.2.0 specification

    /// Default step amplitude for delta search (radians)
    /// Gantree: DEFAULT_STEP_AMP: f64 = 0.12
    pub const DEFAULT_STEP_AMP: f64 = 0.12;

    /// Maximum inner iterations
    /// Gantree: DEFAULT_INNER_MAX: usize = 10
    pub const DEFAULT_INNER_MAX: usize = 10;

    /// Step size decay rate per inner iteration
    /// Gantree: DECAY_RATE: f64 = 0.9
    pub const DECAY_RATE: f64 = 0.9;

    /// Convergence window size
    /// Gantree: CONVERGENCE_WINDOW: usize = 3
    pub const CONVERGENCE_WINDOW: usize = 3;

    /// 5-qubit convergence threshold (p ≤ 0.02)
    /// Gantree: THRESHOLD_5Q: f64 = 0.030
    pub const THRESHOLD_5Q: f64 = 0.030;

    /// 7-qubit convergence threshold
    /// Calculated: THRESHOLD_5Q × (4/6) ≈ 0.020
    /// Empirical: 0.027
    /// Gantree: THRESHOLD_7Q: f64 = 0.027
    pub const THRESHOLD_7Q: f64 = 0.027;

    /// Default number of theta points in main loop
    /// Gantree: DEFAULT_POINTS: usize = 20
    pub const DEFAULT_POINTS: usize = 20;

    /// Default noise level for depolarizing error
    pub const DEFAULT_NOISE: f64 = 0.02;

    /// Recommended maximum noise level
    pub const MAX_RECOMMENDED_NOISE: f64 = 0.030;

    /// Absolute maximum noise level
    pub const ABSOLUTE_MAX_NOISE: f64 = 0.06;

    /// Critical point for 5-qubit
    pub const CRITICAL_5Q: f64 = 0.030;

    /// Critical point for 7-qubit
    pub const CRITICAL_7Q: f64 = 0.027;

    /// Default readout error rate
    pub const DEFAULT_READOUT_ERROR: f64 = 0.005;

    /// Circuit depth for N qubits (linear chain): N-1 CNOTs
    #[inline]
    pub const fn circuit_depth(num_qubits: usize) -> usize {
        if num_qubits > 0 {
            num_qubits - 1
        } else {
            0
        }
    }

    /// Depth ratio for threshold scaling
    /// Reference: 5-qubit (4 CNOTs)
    #[inline]
    pub fn depth_ratio(num_qubits: usize) -> f64 {
        let depth = circuit_depth(num_qubits) as f64;
        let ref_depth = circuit_depth(5) as f64; // 4 CNOTs
        if ref_depth > 0.0 {
            depth / ref_depth
        } else {
            1.0
        }
    }

    /// Calculate threshold for N qubits
    /// Gantree: threshold_N = threshold_5Q / depth_ratio
    #[inline]
    pub fn threshold_for_qubits(num_qubits: usize) -> f64 {
        let ratio = depth_ratio(num_qubits);
        if ratio > 0.0 {
            THRESHOLD_5Q / ratio
        } else {
            THRESHOLD_5Q
        }
    }

    /// Safety multiplier for inner count calculation
    pub const INNER_SAFETY_MULTIPLIER: usize = 5;

    /// Cumulative threshold multiplier
    pub const CUMULATIVE_THRESHOLD_MULT: f64 = 1.5;
}

// ============================================================================
// Statistics Constants
// Gantree: stats // 통계 상수
// ============================================================================

pub mod stats {
    //! Statistical constants for z-test and significance testing

    /// Z critical value for 90% confidence (one-tailed)
    /// Gantree: Z_CRIT_90: f64 = 1.645
    pub const Z_CRIT_90: f64 = 1.645;

    /// Z critical value for 95% confidence
    /// Gantree: Z_CRIT_95: f64 = 1.960
    pub const Z_CRIT_95: f64 = 1.960;

    /// Z critical value for 97.5% confidence
    pub const Z_CRIT_975: f64 = 2.240;

    /// Z critical value for 99% confidence
    /// Gantree: Z_CRIT_99: f64 = 2.575
    pub const Z_CRIT_99: f64 = 2.575;

    /// Default number of shots per measurement
    /// Gantree: DEFAULT_SHOTS: u64 = 8192
    pub const DEFAULT_SHOTS: u64 = 8192;

    /// Minimum recommended shots
    pub const MIN_SHOTS: u64 = 1024;

    /// Maximum recommended shots
    pub const MAX_SHOTS: u64 = 32768;

    /// High shots threshold for adaptive z-test
    pub const HIGH_SHOTS_THRESHOLD: u64 = 16384;

    /// Low shots threshold for adaptive z-test
    pub const LOW_SHOTS_THRESHOLD: u64 = 4096;

    /// Adaptive level adjustment for high noise
    pub const ADAPTIVE_HIGH_NOISE_ADJ: f64 = 0.025;

    /// Adaptive level adjustment for low shots
    pub const ADAPTIVE_LOW_SHOTS_ADJ: f64 = 0.025;

    /// Adaptive level adjustment for high shots
    pub const ADAPTIVE_HIGH_SHOTS_ADJ: f64 = -0.05;

    /// Minimum confidence level
    pub const MIN_CONFIDENCE_LEVEL: f64 = 0.90;

    /// Maximum confidence level
    pub const MAX_CONFIDENCE_LEVEL: f64 = 0.99;

    /// Default confidence level
    pub const DEFAULT_CONFIDENCE_LEVEL: f64 = 0.95;

    /// Epsilon for tie detection
    pub const TIE_EPSILON: f64 = 1e-9;

    /// Get z-critical value for a given confidence level
    pub fn z_critical(confidence: f64) -> f64 {
        if confidence >= 0.99 {
            Z_CRIT_99
        } else if confidence >= 0.975 {
            Z_CRIT_975
        } else if confidence >= 0.95 {
            Z_CRIT_95
        } else if confidence >= 0.90 {
            Z_CRIT_90
        } else {
            Z_CRIT_95 // fallback
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_depth() {
        assert_eq!(tqqc::circuit_depth(5), 4);
        assert_eq!(tqqc::circuit_depth(7), 6);
        assert_eq!(tqqc::circuit_depth(1), 0);
    }

    #[test]
    fn test_depth_ratio() {
        assert!((tqqc::depth_ratio(5) - 1.0).abs() < 1e-10);
        assert!((tqqc::depth_ratio(7) - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_threshold_for_qubits() {
        let t5 = tqqc::threshold_for_qubits(5);
        let t7 = tqqc::threshold_for_qubits(7);

        assert!((t5 - tqqc::THRESHOLD_5Q).abs() < 1e-10);
        assert!((t7 - tqqc::THRESHOLD_5Q / 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_z_critical() {
        assert!((stats::z_critical(0.90) - stats::Z_CRIT_90).abs() < 1e-10);
        assert!((stats::z_critical(0.95) - stats::Z_CRIT_95).abs() < 1e-10);
        assert!((stats::z_critical(0.99) - stats::Z_CRIT_99).abs() < 1e-10);
    }

    #[test]
    fn test_t2_constraint() {
        // T2 should be <= 2*T1 (physical constraint)
        assert!(physics::DEFAULT_T2_US <= 2.0 * physics::DEFAULT_T1_US);
    }
}
