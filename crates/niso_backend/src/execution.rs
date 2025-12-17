//! Backend execution types and traits
//!
//! Gantree: L6_Backend → BackendTrait
//!
//! Defines the interface for quantum backend execution.

use niso_calibration::CalibrationInfo;
use niso_core::{Circuit, Counts, NisoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Result of circuit execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Measurement counts (bitstring -> count)
    pub counts: Counts,

    /// Number of shots executed
    pub shots: u64,

    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Execution metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Backend name
    pub backend: String,

    /// Job ID (if applicable)
    pub job_id: Option<String>,

    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,

    /// Whether simulation was used
    pub simulated: bool,

    /// Seed used (if any)
    pub seed: Option<u64>,

    /// Additional info
    pub extra: HashMap<String, String>,
}

impl ExecutionResult {
    /// Create new execution result
    pub fn new(counts: Counts, shots: u64, backend: &str) -> Self {
        Self {
            counts,
            shots,
            metadata: ExecutionMetadata {
                backend: backend.to_string(),
                simulated: true,
                ..Default::default()
            },
        }
    }

    /// Get total count (should equal shots)
    pub fn total_counts(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Get probability of a specific bitstring
    pub fn probability(&self, bitstring: &str) -> f64 {
        let count = self.counts.get(bitstring).copied().unwrap_or(0);
        count as f64 / self.shots as f64
    }

    /// Get most frequent bitstring
    pub fn most_frequent(&self) -> Option<(&String, u64)> {
        self.counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(bs, &count)| (bs, count))
    }

    /// Calculate parity expectation value
    /// E = P_even - P_odd = Σ_b (-1)^popcount(b) * P(b)
    pub fn parity_expectation(&self) -> f64 {
        let mut expectation = 0.0;

        for (bitstring, &count) in &self.counts {
            let popcount = bitstring.chars().filter(|&c| c == '1').count();
            let sign = if popcount % 2 == 0 { 1.0 } else { -1.0 };
            expectation += sign * (count as f64);
        }

        expectation / self.shots as f64
    }

    /// Calculate probability of even parity
    pub fn p_even(&self) -> f64 {
        let mut even_count = 0u64;

        for (bitstring, &count) in &self.counts {
            let popcount = bitstring.chars().filter(|&c| c == '1').count();
            if popcount % 2 == 0 {
                even_count += count;
            }
        }

        even_count as f64 / self.shots as f64
    }

    /// Calculate probability of odd parity
    pub fn p_odd(&self) -> f64 {
        1.0 - self.p_even()
    }
}

impl fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ExecutionResult(shots={}, unique={}, parity={:.4})",
            self.shots,
            self.counts.len(),
            self.parity_expectation()
        )
    }
}

/// Quantum backend trait
/// Gantree: BackendTrait // 백엔드 인터페이스
pub trait Backend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;

    /// Get number of qubits
    fn num_qubits(&self) -> usize;

    /// Execute a circuit
    /// Gantree: execute(circuit, shots) -> Result<ExecutionResult>
    fn execute(&self, circuit: &Circuit, shots: u64) -> NisoResult<ExecutionResult>;

    /// Execute multiple circuits (batch)
    fn execute_batch(&self, circuits: &[Circuit], shots: u64) -> NisoResult<Vec<ExecutionResult>> {
        circuits.iter().map(|c| self.execute(c, shots)).collect()
    }

    /// Get calibration info (if available)
    fn calibration(&self) -> Option<&CalibrationInfo> {
        None
    }

    /// Check if backend is simulator
    fn is_simulator(&self) -> bool {
        true
    }

    /// Get maximum shots per execution
    fn max_shots(&self) -> u64 {
        100_000
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_counts() -> Counts {
        let mut counts = HashMap::new();
        counts.insert("000".to_string(), 400);
        counts.insert("001".to_string(), 100);
        counts.insert("010".to_string(), 100);
        counts.insert("011".to_string(), 100);
        counts.insert("100".to_string(), 100);
        counts.insert("101".to_string(), 50);
        counts.insert("110".to_string(), 100);
        counts.insert("111".to_string(), 50);
        counts
    }

    #[test]
    fn test_execution_result_new() {
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        assert_eq!(result.shots, 1000);
        assert_eq!(result.metadata.backend, "test");
    }

    #[test]
    fn test_total_counts() {
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        assert_eq!(result.total_counts(), 1000);
    }

    #[test]
    fn test_probability() {
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        assert!((result.probability("000") - 0.4).abs() < 1e-10);
        assert!((result.probability("111") - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_parity_expectation() {
        // Even parity: 000, 011, 101, 110
        // Odd parity: 001, 010, 100, 111
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        // Even: 400 + 100 + 50 + 100 = 650
        // Odd: 100 + 100 + 100 + 50 = 350
        // Expectation = (650 - 350) / 1000 = 0.3
        let expectation = result.parity_expectation();
        assert!((expectation - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_p_even_p_odd() {
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        let p_even = result.p_even();
        let p_odd = result.p_odd();

        assert!((p_even - 0.65).abs() < 1e-10);
        assert!((p_odd - 0.35).abs() < 1e-10);
        assert!((p_even + p_odd - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_most_frequent() {
        let counts = make_test_counts();
        let result = ExecutionResult::new(counts, 1000, "test");

        let (bs, count) = result.most_frequent().unwrap();
        assert_eq!(bs, "000");
        assert_eq!(count, 400);
    }
}
