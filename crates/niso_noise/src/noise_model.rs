//! Noise model for NISO
//!
//! Gantree: L2_Noise → NoiseModel
//!
//! Provides unified noise model representation for NISQ devices,
//! including T1/T2 decoherence, gate errors, and readout errors.

use niso_core::error::{NisoError, NisoResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unified noise model for quantum hardware
/// Gantree: NoiseModel // 통합 노이즈
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseModel {
    /// T1 relaxation time in microseconds
    /// Gantree: t1_us: f64 // T1 (μs)
    t1_us: f64,

    /// T2 dephasing time in microseconds
    /// Gantree: t2_us: f64 // T2 (μs)
    t2_us: f64,

    /// Single-qubit gate error rate
    /// Gantree: gate_error_1q: f64 // 1Q 에러
    gate_error_1q: f64,

    /// Two-qubit gate error rate
    /// Gantree: gate_error_2q: f64 // 2Q 에러
    gate_error_2q: f64,

    /// Readout error rate
    /// Gantree: readout_error: f64 // 측정 에러
    readout_error: f64,

    /// Crosstalk error rate (optional)
    /// Gantree: crosstalk: Option<f64> // 크로스톡 (신규)
    crosstalk: Option<f64>,
}

impl NoiseModel {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create a new noise model with validation
    /// Gantree: new(t1,t2,e1,e2,ro) -> Result<Self> // 생성+검증
    pub fn new(
        t1_us: f64,
        t2_us: f64,
        gate_error_1q: f64,
        gate_error_2q: f64,
        readout_error: f64,
    ) -> NisoResult<Self> {
        let model = Self {
            t1_us,
            t2_us,
            gate_error_1q,
            gate_error_2q,
            readout_error,
            crosstalk: None,
        };
        model.validate()?;
        Ok(model)
    }

    /// Create ideal (noiseless) model
    /// Gantree: ideal() -> Self // 이상적
    pub fn ideal() -> Self {
        Self {
            t1_us: f64::INFINITY,
            t2_us: f64::INFINITY,
            gate_error_1q: 0.0,
            gate_error_2q: 0.0,
            readout_error: 0.0,
            crosstalk: None,
        }
    }

    /// Create typical IBM Quantum noise model
    /// Gantree: ibm_typical() -> Self // IBM 전형
    ///
    /// Based on IBM Falcon/Eagle processors (2024-2025)
    pub fn ibm_typical() -> Self {
        Self {
            t1_us: 100.0,           // ~100 μs
            t2_us: 60.0,            // ~60 μs
            gate_error_1q: 0.0003,  // ~0.03%
            gate_error_2q: 0.01,    // ~1%
            readout_error: 0.01,    // ~1%
            crosstalk: Some(0.001), // ~0.1%
        }
    }

    /// Create high-quality superconducting processor model
    pub fn high_quality() -> Self {
        Self {
            t1_us: 200.0,
            t2_us: 150.0,
            gate_error_1q: 0.0001,
            gate_error_2q: 0.005,
            readout_error: 0.005,
            crosstalk: Some(0.0005),
        }
    }

    /// Create noisy model for testing
    pub fn noisy_test(depol: f64) -> Self {
        Self {
            t1_us: 100.0,
            t2_us: 60.0,
            gate_error_1q: depol,
            gate_error_2q: depol * 10.0,
            readout_error: depol / 4.0,
            crosstalk: None,
        }
    }

    /// Create from effective depolarizing error
    /// Based on TQQC noise model convention
    pub fn from_depol(p_depol: f64) -> NisoResult<Self> {
        if p_depol < 0.0 || p_depol > 0.06 {
            return Err(NisoError::InvalidNoiseLevel(p_depol));
        }

        Ok(Self {
            t1_us: 100.0,
            t2_us: 60.0,
            gate_error_1q: p_depol,
            gate_error_2q: p_depol * 10.0,
            readout_error: p_depol / 4.0,
            crosstalk: None,
        })
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set crosstalk error
    /// Gantree: with_crosstalk(&mut,ct) -> &mut Self // 크로스톡 설정
    pub fn with_crosstalk(mut self, crosstalk: f64) -> Self {
        self.crosstalk = Some(crosstalk);
        self
    }

    /// Set T1 time
    pub fn with_t1(mut self, t1_us: f64) -> Self {
        self.t1_us = t1_us;
        self
    }

    /// Set T2 time
    pub fn with_t2(mut self, t2_us: f64) -> Self {
        self.t2_us = t2_us;
        self
    }

    /// Set single-qubit gate error
    pub fn with_gate_error_1q(mut self, error: f64) -> Self {
        self.gate_error_1q = error;
        self
    }

    /// Set two-qubit gate error
    pub fn with_gate_error_2q(mut self, error: f64) -> Self {
        self.gate_error_2q = error;
        self
    }

    /// Set readout error
    pub fn with_readout_error(mut self, error: f64) -> Self {
        self.readout_error = error;
        self
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get T1 time in microseconds
    pub fn t1_us(&self) -> f64 {
        self.t1_us
    }

    /// Get T2 time in microseconds
    pub fn t2_us(&self) -> f64 {
        self.t2_us
    }

    /// Get T1 time in seconds
    pub fn t1_s(&self) -> f64 {
        self.t1_us * 1e-6
    }

    /// Get T2 time in seconds
    pub fn t2_s(&self) -> f64 {
        self.t2_us * 1e-6
    }

    /// Get single-qubit gate error rate
    pub fn gate_error_1q(&self) -> f64 {
        self.gate_error_1q
    }

    /// Get two-qubit gate error rate
    pub fn gate_error_2q(&self) -> f64 {
        self.gate_error_2q
    }

    /// Get readout error rate
    pub fn readout_error(&self) -> f64 {
        self.readout_error
    }

    /// Get crosstalk error rate
    pub fn crosstalk(&self) -> Option<f64> {
        self.crosstalk
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate noise model constraints
    /// Gantree: validate(&self) -> Result // T2<=2*T1
    pub fn validate(&self) -> NisoResult<()> {
        // T1 must be positive
        if self.t1_us <= 0.0 && self.t1_us.is_finite() {
            return Err(NisoError::CalibrationError(format!(
                "T1 must be positive: {}",
                self.t1_us
            )));
        }

        // T2 must be positive
        if self.t2_us <= 0.0 && self.t2_us.is_finite() {
            return Err(NisoError::CalibrationError(format!(
                "T2 must be positive: {}",
                self.t2_us
            )));
        }

        // Physical constraint: T2 ≤ 2*T1
        if self.t2_us.is_finite() && self.t1_us.is_finite() && self.t2_us > 2.0 * self.t1_us {
            return Err(NisoError::InvalidT2 {
                t2_us: self.t2_us,
                t1_us: self.t1_us,
            });
        }

        // Gate errors in [0, 1]
        if !(0.0..=1.0).contains(&self.gate_error_1q) {
            return Err(NisoError::CalibrationError(format!(
                "1Q gate error must be in [0,1]: {}",
                self.gate_error_1q
            )));
        }

        if !(0.0..=1.0).contains(&self.gate_error_2q) {
            return Err(NisoError::CalibrationError(format!(
                "2Q gate error must be in [0,1]: {}",
                self.gate_error_2q
            )));
        }

        // Readout error in [0, 1]
        if !(0.0..=1.0).contains(&self.readout_error) {
            return Err(NisoError::CalibrationError(format!(
                "Readout error must be in [0,1]: {}",
                self.readout_error
            )));
        }

        // Crosstalk in [0, 1] if present
        if let Some(ct) = self.crosstalk {
            if !(0.0..=1.0).contains(&ct) {
                return Err(NisoError::CalibrationError(format!(
                    "Crosstalk must be in [0,1]: {}",
                    ct
                )));
            }
        }

        Ok(())
    }

    // ========================================================================
    // Derived Quantities
    // ========================================================================

    /// Calculate effective depolarizing error rate
    /// Gantree: effective_depol(&self) -> f64 // 유효 depol
    ///
    /// Returns the single-qubit depolarizing rate (primary noise indicator)
    /// Two-qubit errors are derived from this (typically 10x higher)
    pub fn effective_depol(&self) -> f64 {
        // Primary indicator is 1Q gate error
        // This matches TQQC convention where p_depol = 1Q error rate
        self.gate_error_1q
    }

    /// Check if model is below TQQC critical point
    pub fn is_tqqc_valid(&self, num_qubits: usize) -> bool {
        let critical = niso_core::tqqc::threshold_for_qubits(num_qubits);
        self.effective_depol() <= critical
    }

    /// Check if model is within recommended range
    pub fn is_recommended(&self) -> bool {
        self.effective_depol() <= 0.020
    }

    /// Estimate single-qubit fidelity
    pub fn fidelity_1q(&self) -> f64 {
        1.0 - self.gate_error_1q
    }

    /// Estimate two-qubit fidelity
    pub fn fidelity_2q(&self) -> f64 {
        1.0 - self.gate_error_2q
    }

    /// Estimate readout fidelity
    pub fn fidelity_readout(&self) -> f64 {
        1.0 - self.readout_error
    }

    /// Estimate T1 decay probability for a given time
    pub fn t1_decay_prob(&self, time_us: f64) -> f64 {
        if self.t1_us.is_infinite() {
            0.0
        } else {
            1.0 - (-time_us / self.t1_us).exp()
        }
    }

    /// Estimate T2 dephasing probability for a given time
    pub fn t2_dephasing_prob(&self, time_us: f64) -> f64 {
        if self.t2_us.is_infinite() {
            0.0
        } else {
            1.0 - (-time_us / self.t2_us).exp()
        }
    }

    /// Estimate circuit fidelity
    ///
    /// # Arguments
    /// * `num_1q_gates` - Number of single-qubit gates
    /// * `num_2q_gates` - Number of two-qubit gates
    /// * `num_measurements` - Number of measurements
    /// * `circuit_time_us` - Total circuit execution time
    pub fn estimate_circuit_fidelity(
        &self,
        num_1q_gates: usize,
        num_2q_gates: usize,
        num_measurements: usize,
        circuit_time_us: f64,
    ) -> f64 {
        // Gate fidelity contribution
        let gate_fidelity = self.fidelity_1q().powi(num_1q_gates as i32)
            * self.fidelity_2q().powi(num_2q_gates as i32);

        // Readout fidelity contribution
        let readout_fidelity = self.fidelity_readout().powi(num_measurements as i32);

        // Decoherence contribution (simplified T2 model)
        let decoherence_fidelity = if self.t2_us.is_finite() {
            (-circuit_time_us / self.t2_us).exp()
        } else {
            1.0
        };

        gate_fidelity * readout_fidelity * decoherence_fidelity
    }
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self::ibm_typical()
    }
}

impl fmt::Display for NoiseModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NoiseModel(T1={:.0}μs, T2={:.0}μs, 1Q={:.4}, 2Q={:.4}, RO={:.4})",
            self.t1_us, self.t2_us, self.gate_error_1q, self.gate_error_2q, self.readout_error
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
    fn test_noise_model_new() {
        let model = NoiseModel::new(100.0, 60.0, 0.001, 0.01, 0.01).unwrap();
        assert_eq!(model.t1_us(), 100.0);
        assert_eq!(model.t2_us(), 60.0);
    }

    #[test]
    fn test_noise_model_t2_constraint() {
        // T2 > 2*T1 should fail
        let result = NoiseModel::new(100.0, 250.0, 0.001, 0.01, 0.01);
        assert!(result.is_err());
    }

    #[test]
    fn test_noise_model_ideal() {
        let model = NoiseModel::ideal();
        assert!(model.t1_us().is_infinite());
        assert_eq!(model.gate_error_1q(), 0.0);
        assert_eq!(model.effective_depol(), 0.0);
    }

    #[test]
    fn test_noise_model_ibm_typical() {
        let model = NoiseModel::ibm_typical();
        assert!(model.validate().is_ok());
        assert!(model.t2_us() <= 2.0 * model.t1_us());
    }

    #[test]
    fn test_effective_depol() {
        let model = NoiseModel::new(100.0, 60.0, 0.01, 0.02, 0.01).unwrap();
        let depol = model.effective_depol();
        // effective_depol = gate_error_1q = 0.01
        assert!((depol - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_from_depol() {
        let model = NoiseModel::from_depol(0.02).unwrap();
        assert_eq!(model.gate_error_1q(), 0.02);
        assert_eq!(model.gate_error_2q(), 0.2);
    }

    #[test]
    fn test_from_depol_exceeds_max() {
        let result = NoiseModel::from_depol(0.07);
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_fidelity() {
        let model = NoiseModel::new(100.0, 60.0, 0.001, 0.01, 0.01).unwrap();
        let fidelity = model.estimate_circuit_fidelity(10, 5, 7, 1.0);
        // Should be positive and less than 1
        assert!(fidelity > 0.0 && fidelity < 1.0);
    }

    #[test]
    fn test_with_crosstalk() {
        let model = NoiseModel::ibm_typical().with_crosstalk(0.002);
        assert_eq!(model.crosstalk(), Some(0.002));
    }

    #[test]
    fn test_decay_probabilities() {
        let model = NoiseModel::new(100.0, 60.0, 0.001, 0.01, 0.01).unwrap();

        // At t=0, no decay
        assert!((model.t1_decay_prob(0.0)).abs() < 1e-10);

        // At t=T1, decay is 1-1/e ≈ 0.632
        let decay_at_t1 = model.t1_decay_prob(100.0);
        assert!((decay_at_t1 - 0.6321205588).abs() < 1e-6);
    }
}
