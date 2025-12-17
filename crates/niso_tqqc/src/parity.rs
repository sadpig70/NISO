//! Parity calculation and circuit building for TQQC
//!
//! Gantree: L5_TQQC → Parity
//!
//! Provides parity expectation calculation and TQQC circuit generation.

use crate::config::TqqcConfig;
use niso_core::{Circuit, CircuitBuilder, Counts};

/// Parity calculation utilities
/// Gantree: Parity // 패리티 계산 (통합)
pub struct Parity;

impl Parity {
    // ========================================================================
    // Parity Calculation
    // ========================================================================

    /// Count number of 1s in a bitstring
    /// Gantree: popcount(bitstring: &str) -> usize // 비트 카운트
    pub fn popcount(bitstring: &str) -> usize {
        bitstring.chars().filter(|&c| c == '1').count()
    }

    /// Check if bitstring has even parity
    pub fn is_even(bitstring: &str) -> bool {
        Self::popcount(bitstring) % 2 == 0
    }

    /// Check if bitstring has odd parity
    pub fn is_odd(bitstring: &str) -> bool {
        !Self::is_even(bitstring)
    }

    /// Calculate probability of even parity
    /// Gantree: p_even(counts: &Counts) -> f64 // 짝수 확률
    pub fn p_even(counts: &Counts) -> f64 {
        let total: u64 = counts.values().sum();
        if total == 0 {
            return 0.5;
        }

        let even_count: u64 = counts
            .iter()
            .filter(|(bs, _)| Self::is_even(bs))
            .map(|(_, &count)| count)
            .sum();

        even_count as f64 / total as f64
    }

    /// Calculate probability of odd parity
    /// Gantree: p_odd(counts: &Counts) -> f64 // 홀수 확률
    pub fn p_odd(counts: &Counts) -> f64 {
        1.0 - Self::p_even(counts)
    }

    /// Calculate parity expectation value
    /// Gantree: expectation(counts: &Counts) -> f64 // 기대값
    ///
    /// E = P_even - P_odd = Σ_b (-1)^popcount(b) * P(b)
    pub fn expectation(counts: &Counts) -> f64 {
        let total: u64 = counts.values().sum();
        if total == 0 {
            return 0.0;
        }

        let mut weighted_sum: i64 = 0;
        for (bitstring, &count) in counts {
            let sign = if Self::is_even(bitstring) { 1 } else { -1 };
            weighted_sum += sign * count as i64;
        }

        weighted_sum as f64 / total as f64
    }

    /// Calculate parity sign: +1 for even, -1 for odd
    pub fn parity_sign(bitstring: &str) -> i32 {
        if Self::is_even(bitstring) {
            1
        } else {
            -1
        }
    }

    // ========================================================================
    // Circuit Building
    // ========================================================================

    /// Build TQQC parity measurement circuit
    /// Gantree: build_circuit(cfg: &TqqcConfig,theta,delta) -> Circuit // 회로 생성
    ///
    /// Circuit structure:
    /// 1. H gate on qubit 0 (create superposition)
    /// 2. Entangler chain (CX or CZ)
    /// 3. Rz(θ+δ) on qubit 0 (parameterized rotation)
    /// 4. Basis transformation (X: H, Y: Sdg+H, Z: identity)
    /// 5. Measurement
    pub fn build_circuit(config: &TqqcConfig, theta: f64, delta: f64) -> Circuit {
        CircuitBuilder::new(config.qubits)
            .tqqc_parity(theta, delta, config.entangler, &config.basis)
            .build()
    }

    /// Build circuit with explicit basis string
    pub fn build_circuit_with_basis(
        num_qubits: usize,
        theta: f64,
        delta: f64,
        entangler: niso_core::EntanglerType,
        basis: &niso_core::BasisString,
    ) -> Circuit {
        CircuitBuilder::new(num_qubits)
            .tqqc_parity(theta, delta, entangler, basis)
            .build()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_counts() -> Counts {
        let mut counts = HashMap::new();
        counts.insert("000".to_string(), 400); // even
        counts.insert("001".to_string(), 100); // odd
        counts.insert("010".to_string(), 100); // odd
        counts.insert("011".to_string(), 100); // even
        counts.insert("100".to_string(), 100); // odd
        counts.insert("101".to_string(), 50); // even
        counts.insert("110".to_string(), 100); // even
        counts.insert("111".to_string(), 50); // odd
        counts
    }

    #[test]
    fn test_popcount() {
        assert_eq!(Parity::popcount("000"), 0);
        assert_eq!(Parity::popcount("001"), 1);
        assert_eq!(Parity::popcount("011"), 2);
        assert_eq!(Parity::popcount("111"), 3);
        assert_eq!(Parity::popcount("1111111"), 7);
    }

    #[test]
    fn test_is_even_odd() {
        assert!(Parity::is_even("000"));
        assert!(Parity::is_even("011"));
        assert!(Parity::is_even("110"));

        assert!(Parity::is_odd("001"));
        assert!(Parity::is_odd("010"));
        assert!(Parity::is_odd("111"));
    }

    #[test]
    fn test_p_even() {
        let counts = make_test_counts();
        let p_even = Parity::p_even(&counts);

        // Even: 400 + 100 + 50 + 100 = 650
        // Total: 1000
        assert!((p_even - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_p_odd() {
        let counts = make_test_counts();
        let p_odd = Parity::p_odd(&counts);

        // Odd: 100 + 100 + 100 + 50 = 350
        // Total: 1000
        assert!((p_odd - 0.35).abs() < 1e-10);
    }

    #[test]
    fn test_expectation() {
        let counts = make_test_counts();
        let exp = Parity::expectation(&counts);

        // E = P_even - P_odd = 0.65 - 0.35 = 0.30
        assert!((exp - 0.30).abs() < 1e-10);
    }

    #[test]
    fn test_p_even_p_odd_sum() {
        let counts = make_test_counts();
        let sum = Parity::p_even(&counts) + Parity::p_odd(&counts);
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_expectation_relation() {
        let counts = make_test_counts();

        // E = P_even - P_odd = 2*P_even - 1
        let exp = Parity::expectation(&counts);
        let p_even = Parity::p_even(&counts);

        assert!((exp - (2.0 * p_even - 1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_parity_sign() {
        assert_eq!(Parity::parity_sign("000"), 1);
        assert_eq!(Parity::parity_sign("001"), -1);
        assert_eq!(Parity::parity_sign("011"), 1);
        assert_eq!(Parity::parity_sign("111"), -1);
    }

    #[test]
    fn test_build_circuit() {
        let config = TqqcConfig::default_7q();
        let circuit = Parity::build_circuit(&config, 0.5, 0.1);

        assert_eq!(circuit.num_qubits(), 7);
        assert_eq!(circuit.count_2q(), 6); // 6 CNOTs in linear chain
    }

    #[test]
    fn test_build_circuit_5q() {
        let config = TqqcConfig::default_5q();
        let circuit = Parity::build_circuit(&config, 0.0, 0.0);

        assert_eq!(circuit.num_qubits(), 5);
        assert_eq!(circuit.count_2q(), 4); // 4 CNOTs
    }

    #[test]
    fn test_empty_counts() {
        let counts: Counts = HashMap::new();

        assert_eq!(Parity::p_even(&counts), 0.5);
        assert_eq!(Parity::expectation(&counts), 0.0);
    }

    #[test]
    fn test_all_even_parity() {
        let mut counts = HashMap::new();
        counts.insert("000".to_string(), 500);
        counts.insert("011".to_string(), 500);

        assert!((Parity::p_even(&counts) - 1.0).abs() < 1e-10);
        assert!((Parity::expectation(&counts) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_all_odd_parity() {
        let mut counts = HashMap::new();
        counts.insert("001".to_string(), 500);
        counts.insert("010".to_string(), 500);

        assert!((Parity::p_odd(&counts) - 1.0).abs() < 1e-10);
        assert!((Parity::expectation(&counts) - -1.0).abs() < 1e-10);
    }
}
