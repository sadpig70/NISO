//! Calibration information for quantum backends
//!
//! Gantree: L3_Calibration → CalibrationInfo
//!
//! Provides structured calibration data extracted from quantum hardware,
//! including T1/T2 times, gate errors, and topology information.

use niso_core::{QubitId, Topology};
use niso_noise::{GateTimes, NoiseModel, NoiseVector, NoiseVectorSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

/// Calibration data from a quantum backend
/// Gantree: CalibrationInfo // 캘리브레이션 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationInfo {
    /// Backend name
    /// Gantree: backend_name: String // 백엔드 이름
    pub backend_name: String,

    /// Extraction timestamp
    /// Gantree: timestamp: DateTime // 추출 시간
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,

    /// T1 relaxation times per qubit (microseconds)
    /// Gantree: t1_times: HashMap<QubitId,f64> // T1 맵
    pub t1_times: HashMap<QubitId, f64>,

    /// T2 dephasing times per qubit (microseconds)
    /// Gantree: t2_times: HashMap<QubitId,f64> // T2 맵
    pub t2_times: HashMap<QubitId, f64>,

    /// Single-qubit gate errors per qubit
    /// Gantree: gate_errors_1q: HashMap<QubitId,f64> // 1Q 에러 맵
    pub gate_errors_1q: HashMap<QubitId, f64>,

    /// Two-qubit gate errors per qubit pair
    /// Gantree: gate_errors_2q: HashMap<(Q,Q),f64> // 2Q 에러 맵
    #[serde(skip)]
    pub gate_errors_2q: HashMap<(QubitId, QubitId), f64>,

    /// Readout errors per qubit
    /// Gantree: readout_errors: HashMap<QubitId,f64> // 측정 에러 맵
    pub readout_errors: HashMap<QubitId, f64>,

    /// Coupling map (connectivity)
    /// Gantree: coupling_map: Vec<(QubitId,QubitId)> // 연결 맵
    pub coupling_map: Vec<(QubitId, QubitId)>,

    /// Single-qubit gate times (optional)
    pub gate_times_1q_ns: Option<f64>,
    /// Two-qubit gate times (optional)
    pub gate_times_2q_ns: Option<f64>,
}

impl CalibrationInfo {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new calibration info
    pub fn new(backend_name: &str) -> Self {
        Self {
            backend_name: backend_name.to_string(),
            timestamp: SystemTime::now(),
            t1_times: HashMap::new(),
            t2_times: HashMap::new(),
            gate_errors_1q: HashMap::new(),
            gate_errors_2q: HashMap::new(),
            readout_errors: HashMap::new(),
            coupling_map: Vec::new(),
            gate_times_1q_ns: None,
            gate_times_2q_ns: None,
        }
    }

    /// Create from uniform values (for testing/simulation)
    pub fn uniform(
        backend_name: &str,
        num_qubits: usize,
        t1_us: f64,
        t2_us: f64,
        error_1q: f64,
        error_2q: f64,
        readout_error: f64,
    ) -> Self {
        let mut info = Self::new(backend_name);

        for q in 0..num_qubits {
            info.t1_times.insert(q, t1_us);
            info.t2_times.insert(q, t2_us);
            info.gate_errors_1q.insert(q, error_1q);
            info.readout_errors.insert(q, readout_error);
        }

        // Linear coupling
        for q in 0..num_qubits.saturating_sub(1) {
            info.coupling_map.push((q, q + 1));
            info.gate_errors_2q.insert((q, q + 1), error_2q);
        }

        info
    }

    /// Create IBM-typical calibration (for testing)
    pub fn ibm_typical(num_qubits: usize) -> Self {
        Self::uniform(
            "ibm_simulator",
            num_qubits,
            100.0,  // T1: 100 μs
            60.0,   // T2: 60 μs
            0.0003, // 1Q error: 0.03%
            0.01,   // 2Q error: 1%
            0.01,   // Readout: 1%
        )
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get number of qubits
    /// Gantree: num_qubits(&self) -> usize // 큐비트 수
    pub fn num_qubits(&self) -> usize {
        self.t1_times
            .len()
            .max(self.t2_times.len())
            .max(self.gate_errors_1q.len())
            .max(self.readout_errors.len())
    }

    /// Get average T1 time (microseconds)
    /// Gantree: avg_t1(&self) -> f64 // 평균 T1
    pub fn avg_t1(&self) -> f64 {
        if self.t1_times.is_empty() {
            return 100.0; // Default
        }
        self.t1_times.values().sum::<f64>() / self.t1_times.len() as f64
    }

    /// Get average T2 time (microseconds)
    /// Gantree: avg_t2(&self) -> f64 // 평균 T2
    pub fn avg_t2(&self) -> f64 {
        if self.t2_times.is_empty() {
            return 60.0; // Default
        }
        self.t2_times.values().sum::<f64>() / self.t2_times.len() as f64
    }

    /// Get average single-qubit gate error
    /// Gantree: avg_error_1q(&self) -> f64 // 평균 1Q 에러
    pub fn avg_error_1q(&self) -> f64 {
        if self.gate_errors_1q.is_empty() {
            return 0.001; // Default
        }
        self.gate_errors_1q.values().sum::<f64>() / self.gate_errors_1q.len() as f64
    }

    /// Get average two-qubit gate error
    /// Gantree: avg_error_2q(&self) -> f64 // 평균 2Q 에러
    pub fn avg_error_2q(&self) -> f64 {
        if self.gate_errors_2q.is_empty() {
            return 0.01; // Default
        }
        self.gate_errors_2q.values().sum::<f64>() / self.gate_errors_2q.len() as f64
    }

    /// Get average readout error
    pub fn avg_readout(&self) -> f64 {
        if self.readout_errors.is_empty() {
            return 0.01; // Default
        }
        self.readout_errors.values().sum::<f64>() / self.readout_errors.len() as f64
    }

    /// Check if calibration is fresh (within TTL)
    pub fn is_fresh(&self, ttl: Duration) -> bool {
        match self.timestamp.elapsed() {
            Ok(elapsed) => elapsed < ttl,
            Err(_) => false,
        }
    }

    /// Get age of calibration
    pub fn age(&self) -> Option<Duration> {
        self.timestamp.elapsed().ok()
    }

    // ========================================================================
    // Conversions
    // ========================================================================

    /// Convert to unified noise model
    /// Gantree: to_noise_model(&self) -> NoiseModel // 모델 변환
    pub fn to_noise_model(&self) -> NoiseModel {
        NoiseModel::new(
            self.avg_t1(),
            self.avg_t2(),
            self.avg_error_1q(),
            self.avg_error_2q(),
            self.avg_readout(),
        )
        .unwrap_or_else(|_| NoiseModel::ibm_typical())
    }

    /// Convert to per-qubit noise vectors
    /// Gantree: to_noise_vectors(&self) -> Vec<NoiseVector> // 벡터 변환
    pub fn to_noise_vectors(&self) -> NoiseVectorSet {
        let num_qubits = self.num_qubits();
        let mut vectors = Vec::with_capacity(num_qubits);

        for q in 0..num_qubits {
            let t1 = self.t1_times.get(&q).copied().unwrap_or(100.0);
            let t2 = self.t2_times.get(&q).copied().unwrap_or(60.0);
            let error_1q = self.gate_errors_1q.get(&q).copied().unwrap_or(0.001);
            let readout = self.readout_errors.get(&q).copied().unwrap_or(0.01);

            // Find worst 2Q error for this qubit
            let error_2q = self
                .gate_errors_2q
                .iter()
                .filter(|((q1, q2), _)| *q1 == q || *q2 == q)
                .map(|(_, &e)| e)
                .fold(0.01, f64::max);

            vectors.push(NoiseVector::new(q, t1, t2, error_1q, error_2q, readout));
        }

        NoiseVectorSet::new(vectors)
    }

    /// Convert to topology
    /// Gantree: to_topology(&self) -> Topology // 토폴로지 변환
    pub fn to_topology(&self) -> Topology {
        Topology::from_coupling_map(self.coupling_map.clone(), true)
            .unwrap_or_else(|_| Topology::linear(self.num_qubits()))
    }

    /// Convert to gate times
    /// Gantree: to_gate_times(&self) -> GateTimes // 시간 변환
    pub fn to_gate_times(&self) -> GateTimes {
        GateTimes::new(
            self.gate_times_1q_ns.unwrap_or(35.0),
            self.gate_times_2q_ns.unwrap_or(300.0),
            5000.0, // Measurement
        )
    }

    // ========================================================================
    // Qubit Selection
    // ========================================================================

    /// Select best qubits by quality
    /// Gantree: best_qubits(&self,n) -> Vec<QubitId> // 최적 큐비트 선택
    pub fn best_qubits(&self, n: usize) -> Vec<QubitId> {
        let vectors = self.to_noise_vectors();
        vectors.best_qubits(n)
    }

    /// Select best linear chain of qubits
    pub fn best_linear_chain(&self, length: usize) -> Option<Vec<QubitId>> {
        let topology = self.to_topology();
        topology.find_linear_chain(length)
    }

    /// Get qubit quality score
    pub fn qubit_quality(&self, qubit: QubitId) -> f64 {
        let t1 = self.t1_times.get(&qubit).copied().unwrap_or(100.0);
        let t2 = self.t2_times.get(&qubit).copied().unwrap_or(60.0);
        let error_1q = self.gate_errors_1q.get(&qubit).copied().unwrap_or(0.001);
        let readout = self.readout_errors.get(&qubit).copied().unwrap_or(0.01);

        // Simple quality score
        let t1_score = (t1 / 200.0).min(1.0);
        let t2_score = (t2 / 120.0).min(1.0);
        let gate_score = 1.0 - error_1q;
        let readout_score = 1.0 - readout;

        (t1_score * t2_score * gate_score * readout_score).powf(0.25)
    }
}

impl fmt::Display for CalibrationInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CalibrationInfo({}, {}Q, T1={:.0}μs, T2={:.0}μs, 1Q={:.4}, 2Q={:.4})",
            self.backend_name,
            self.num_qubits(),
            self.avg_t1(),
            self.avg_t2(),
            self.avg_error_1q(),
            self.avg_error_2q()
        )
    }
}

// ============================================================================
// SystemTime Serde Helper
// ============================================================================

mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_info_new() {
        let info = CalibrationInfo::new("test_backend");
        assert_eq!(info.backend_name, "test_backend");
        assert_eq!(info.num_qubits(), 0);
    }

    #[test]
    fn test_uniform_calibration() {
        let info = CalibrationInfo::uniform("test", 7, 100.0, 60.0, 0.001, 0.01, 0.01);

        assert_eq!(info.num_qubits(), 7);
        assert!((info.avg_t1() - 100.0).abs() < 1e-10);
        assert!((info.avg_t2() - 60.0).abs() < 1e-10);
        assert_eq!(info.coupling_map.len(), 6);
    }

    #[test]
    fn test_to_noise_model() {
        let info = CalibrationInfo::ibm_typical(7);
        let model = info.to_noise_model();

        assert!(model.validate().is_ok());
        assert!(model.t2_us() <= 2.0 * model.t1_us());
    }

    #[test]
    fn test_to_noise_vectors() {
        let info = CalibrationInfo::ibm_typical(5);
        let vectors = info.to_noise_vectors();

        assert_eq!(vectors.num_qubits(), 5);
    }

    #[test]
    fn test_to_topology() {
        let info = CalibrationInfo::uniform("test", 7, 100.0, 60.0, 0.001, 0.01, 0.01);
        let topology = info.to_topology();

        assert_eq!(topology.num_qubits(), 7);
        assert!(topology.is_connected(0, 1));
    }

    #[test]
    fn test_best_qubits() {
        let mut info = CalibrationInfo::new("test");

        // Add qubits with varying quality
        for q in 0..5 {
            let quality_factor = 1.0 + q as f64 * 0.1;
            info.t1_times.insert(q, 100.0 * quality_factor);
            info.t2_times.insert(q, 60.0 * quality_factor);
            info.gate_errors_1q.insert(q, 0.001 / quality_factor);
            info.readout_errors.insert(q, 0.01 / quality_factor);
        }

        let best = info.best_qubits(3);
        assert_eq!(best.len(), 3);
    }

    #[test]
    fn test_freshness() {
        let info = CalibrationInfo::new("test");

        // Just created, should be fresh
        assert!(info.is_fresh(Duration::from_secs(60)));
    }

    #[test]
    fn test_serialization() {
        let info = CalibrationInfo::ibm_typical(5);
        let json = serde_json::to_string(&info).unwrap();
        let restored: CalibrationInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.backend_name, restored.backend_name);
        assert_eq!(info.num_qubits(), restored.num_qubits());
    }
}
