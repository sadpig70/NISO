//! Per-qubit noise vector for NISO
//!
//! Gantree: L2_Noise → NoiseVector
//!
//! Provides per-qubit noise characterization for heterogeneous
//! quantum devices where different qubits have different noise levels.

use crate::noise_model::NoiseModel;
use niso_core::QubitId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Per-qubit noise parameters
/// Gantree: NoiseVector // 큐비트별 노이즈
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseVector {
    /// Qubit identifier
    /// Gantree: qubit_id: QubitId // 큐비트 ID
    pub qubit_id: QubitId,

    /// T1 relaxation time in microseconds
    /// Gantree: t1: f64 // 개별 T1
    pub t1: f64,

    /// T2 dephasing time in microseconds
    /// Gantree: t2: f64 // 개별 T2
    pub t2: f64,

    /// Single-qubit gate error rate
    /// Gantree: gate_error_1q: f64 // 개별 1Q 에러
    pub gate_error_1q: f64,

    /// Two-qubit gate error rate (average for this qubit)
    /// Gantree: gate_error_2q: f64 // 개별 2Q 에러
    pub gate_error_2q: f64,

    /// Readout error rate
    /// Gantree: readout_error: f64 // 개별 측정 에러
    pub readout_error: f64,
}

impl NoiseVector {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create a new noise vector
    /// Gantree: new(qid,t1,t2,e1,e2,ro) -> Self // 생성자
    pub fn new(
        qubit_id: QubitId,
        t1: f64,
        t2: f64,
        gate_error_1q: f64,
        gate_error_2q: f64,
        readout_error: f64,
    ) -> Self {
        Self {
            qubit_id,
            t1,
            t2,
            gate_error_1q,
            gate_error_2q,
            readout_error,
        }
    }

    /// Create from a uniform noise model
    /// Gantree: from_noise_model(qid,nm) -> Self // 모델에서 생성
    pub fn from_noise_model(qubit_id: QubitId, model: &NoiseModel) -> Self {
        Self {
            qubit_id,
            t1: model.t1_us(),
            t2: model.t2_us(),
            gate_error_1q: model.gate_error_1q(),
            gate_error_2q: model.gate_error_2q(),
            readout_error: model.readout_error(),
        }
    }

    /// Create ideal (noiseless) vector
    pub fn ideal(qubit_id: QubitId) -> Self {
        Self {
            qubit_id,
            t1: f64::INFINITY,
            t2: f64::INFINITY,
            gate_error_1q: 0.0,
            gate_error_2q: 0.0,
            readout_error: 0.0,
        }
    }

    // ========================================================================
    // Fidelity Estimations
    // ========================================================================

    /// Estimate gate fidelity for a sequence of gates
    /// Gantree: estimate_gate_fidelity(n1q,n2q) -> f64 // 게이트 fidelity
    pub fn estimate_gate_fidelity(&self, num_1q_gates: usize, num_2q_gates: usize) -> f64 {
        let fidelity_1q = (1.0 - self.gate_error_1q).powi(num_1q_gates as i32);
        let fidelity_2q = (1.0 - self.gate_error_2q).powi(num_2q_gates as i32);
        fidelity_1q * fidelity_2q
    }

    /// Estimate decoherence error for a given idle time
    /// Gantree: estimate_decoherence(time_us) -> f64 // 디코히어런스
    ///
    /// Returns probability of error due to decoherence
    pub fn estimate_decoherence(&self, time_us: f64) -> f64 {
        if time_us <= 0.0 {
            return 0.0;
        }

        // T2 dephasing dominates for most circuits
        if self.t2.is_infinite() {
            0.0
        } else {
            1.0 - (-time_us / self.t2).exp()
        }
    }

    /// Estimate T1 relaxation error for a given time
    pub fn estimate_t1_error(&self, time_us: f64) -> f64 {
        if time_us <= 0.0 {
            return 0.0;
        }

        if self.t1.is_infinite() {
            0.0
        } else {
            1.0 - (-time_us / self.t1).exp()
        }
    }

    /// Estimate readout fidelity
    /// Gantree: estimate_readout_fidelity(nmeas) -> f64 // 측정 fidelity
    pub fn estimate_readout_fidelity(&self, num_measurements: usize) -> f64 {
        (1.0 - self.readout_error).powi(num_measurements as i32)
    }

    /// Estimate total circuit fidelity
    /// Gantree: estimate_circuit_fidelity(n1q,n2q,nm,t) -> f64 // 회로 fidelity
    pub fn estimate_circuit_fidelity(
        &self,
        num_1q_gates: usize,
        num_2q_gates: usize,
        num_measurements: usize,
        circuit_time_us: f64,
    ) -> f64 {
        let gate_fidelity = self.estimate_gate_fidelity(num_1q_gates, num_2q_gates);
        let readout_fidelity = self.estimate_readout_fidelity(num_measurements);
        let coherence_fidelity = 1.0 - self.estimate_decoherence(circuit_time_us);

        gate_fidelity * readout_fidelity * coherence_fidelity
    }

    // ========================================================================
    // Quality Metrics
    // ========================================================================

    /// Calculate quality score (higher is better)
    /// Combines all error sources into a single metric [0, 1]
    pub fn quality_score(&self) -> f64 {
        // Fidelity-based scoring
        let t1_fidelity = if self.t1.is_finite() && self.t1 > 0.0 {
            (self.t1 / 200.0).min(1.0) // Normalize to ~200μs
        } else {
            1.0
        };

        let t2_fidelity = if self.t2.is_finite() && self.t2 > 0.0 {
            (self.t2 / 120.0).min(1.0) // Normalize to ~120μs
        } else {
            1.0
        };

        let gate_1q_fidelity = 1.0 - self.gate_error_1q;
        let gate_2q_fidelity = 1.0 - self.gate_error_2q;
        let readout_fidelity = 1.0 - self.readout_error;

        // Geometric mean of fidelities
        (t1_fidelity * t2_fidelity * gate_1q_fidelity * gate_2q_fidelity * readout_fidelity)
            .max(0.0)
            .powf(0.2) // 5th root
    }

    /// Check if this qubit is usable for TQQC
    pub fn is_tqqc_usable(&self) -> bool {
        // Minimum requirements
        self.t1 >= 50.0
            && self.t2 >= 30.0
            && self.gate_error_1q <= 0.01
            && self.gate_error_2q <= 0.05
            && self.readout_error <= 0.05
    }
}

impl fmt::Display for NoiseVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Q{}: T1={:.0}μs T2={:.0}μs 1Q={:.4} 2Q={:.4} RO={:.4}",
            self.qubit_id,
            self.t1,
            self.t2,
            self.gate_error_1q,
            self.gate_error_2q,
            self.readout_error
        )
    }
}

// ============================================================================
// NoiseVectorSet - Collection of noise vectors
// ============================================================================

/// Collection of noise vectors for a device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoiseVectorSet {
    vectors: Vec<NoiseVector>,
}

impl NoiseVectorSet {
    /// Create from a vector of noise vectors
    pub fn new(vectors: Vec<NoiseVector>) -> Self {
        Self { vectors }
    }

    /// Create from uniform noise model
    pub fn from_noise_model(num_qubits: usize, model: &NoiseModel) -> Self {
        let vectors = (0..num_qubits)
            .map(|q| NoiseVector::from_noise_model(q, model))
            .collect();
        Self { vectors }
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.vectors.len()
    }

    /// Get noise vector for a specific qubit
    pub fn get(&self, qubit: QubitId) -> Option<&NoiseVector> {
        self.vectors.get(qubit)
    }

    /// Get all vectors
    pub fn vectors(&self) -> &[NoiseVector] {
        &self.vectors
    }

    /// Calculate average T1
    pub fn avg_t1(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .vectors
            .iter()
            .filter(|v| v.t1.is_finite())
            .map(|v| v.t1)
            .sum();
        let count = self.vectors.iter().filter(|v| v.t1.is_finite()).count();
        if count > 0 {
            sum / count as f64
        } else {
            f64::INFINITY
        }
    }

    /// Calculate average T2
    pub fn avg_t2(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .vectors
            .iter()
            .filter(|v| v.t2.is_finite())
            .map(|v| v.t2)
            .sum();
        let count = self.vectors.iter().filter(|v| v.t2.is_finite()).count();
        if count > 0 {
            sum / count as f64
        } else {
            f64::INFINITY
        }
    }

    /// Calculate average 1Q gate error
    pub fn avg_error_1q(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }
        self.vectors.iter().map(|v| v.gate_error_1q).sum::<f64>() / self.vectors.len() as f64
    }

    /// Calculate average 2Q gate error
    pub fn avg_error_2q(&self) -> f64 {
        if self.vectors.is_empty() {
            return 0.0;
        }
        self.vectors.iter().map(|v| v.gate_error_2q).sum::<f64>() / self.vectors.len() as f64
    }

    /// Find best qubits by quality score
    pub fn best_qubits(&self, n: usize) -> Vec<QubitId> {
        let mut scored: Vec<_> = self
            .vectors
            .iter()
            .map(|v| (v.qubit_id, v.quality_score()))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.iter().take(n).map(|(q, _)| *q).collect()
    }

    /// Find qubits usable for TQQC
    pub fn tqqc_usable_qubits(&self) -> Vec<QubitId> {
        self.vectors
            .iter()
            .filter(|v| v.is_tqqc_usable())
            .map(|v| v.qubit_id)
            .collect()
    }

    /// Convert to unified noise model (average values)
    pub fn to_noise_model(&self) -> NoiseModel {
        NoiseModel::new(
            self.avg_t1(),
            self.avg_t2(),
            self.avg_error_1q(),
            self.avg_error_2q(),
            self.vectors.iter().map(|v| v.readout_error).sum::<f64>()
                / self.vectors.len().max(1) as f64,
        )
        .unwrap_or_else(|_| NoiseModel::ibm_typical())
    }
}

impl Default for NoiseVectorSet {
    fn default() -> Self {
        Self::from_noise_model(7, &NoiseModel::ibm_typical())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_vector_new() {
        let nv = NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01);
        assert_eq!(nv.qubit_id, 0);
        assert_eq!(nv.t1, 100.0);
    }

    #[test]
    fn test_noise_vector_from_model() {
        let model = NoiseModel::ibm_typical();
        let nv = NoiseVector::from_noise_model(3, &model);
        assert_eq!(nv.qubit_id, 3);
        assert_eq!(nv.t1, model.t1_us());
    }

    #[test]
    fn test_gate_fidelity() {
        let nv = NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01);
        let fidelity = nv.estimate_gate_fidelity(10, 5);
        // (1-0.001)^10 * (1-0.01)^5 ≈ 0.990 * 0.951 ≈ 0.941
        assert!(fidelity > 0.9 && fidelity < 1.0);
    }

    #[test]
    fn test_decoherence() {
        let nv = NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01);

        // No decoherence at t=0
        assert!((nv.estimate_decoherence(0.0)).abs() < 1e-10);

        // Some decoherence at t=T2
        let decoherence = nv.estimate_decoherence(60.0);
        assert!((decoherence - 0.6321205588).abs() < 1e-6);
    }

    #[test]
    fn test_circuit_fidelity() {
        let nv = NoiseVector::new(0, 100.0, 60.0, 0.001, 0.01, 0.01);
        let fidelity = nv.estimate_circuit_fidelity(10, 5, 1, 1.0);
        assert!(fidelity > 0.0 && fidelity < 1.0);
    }

    #[test]
    fn test_noise_vector_set() {
        let model = NoiseModel::ibm_typical();
        let set = NoiseVectorSet::from_noise_model(7, &model);

        assert_eq!(set.num_qubits(), 7);
        assert!((set.avg_t1() - model.t1_us()).abs() < 1e-10);
    }

    #[test]
    fn test_best_qubits() {
        let vectors = vec![
            NoiseVector::new(0, 100.0, 60.0, 0.002, 0.02, 0.02), // Worst
            NoiseVector::new(1, 150.0, 90.0, 0.001, 0.01, 0.01), // Best
            NoiseVector::new(2, 120.0, 70.0, 0.0015, 0.015, 0.015), // Middle
        ];
        let set = NoiseVectorSet::new(vectors);

        let best = set.best_qubits(2);

        // Best qubit (1) should be in top 2
        assert!(
            best.contains(&1),
            "Best qubit 1 not in selection: {:?}",
            best
        );
    }

    #[test]
    fn test_quality_score() {
        let good = NoiseVector::new(0, 200.0, 120.0, 0.0001, 0.005, 0.005);
        let bad = NoiseVector::new(1, 50.0, 30.0, 0.005, 0.05, 0.05);

        assert!(good.quality_score() > bad.quality_score());
    }
}
