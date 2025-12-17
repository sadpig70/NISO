//! Statistical testing for TQQC
//!
//! Gantree: L5_TQQC → StatisticalTest
//!
//! Implements z-test for significance testing in delta search.

use crate::config::SigMode;
use niso_core::stats;
use serde::{Deserialize, Serialize};

/// Direction of improvement
/// Gantree: Direction // 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Positive delta is better
    /// Gantree: Plus // +δ
    Plus,

    /// Negative delta is better
    /// Gantree: Minus // -δ
    Minus,

    /// No significant difference
    /// Gantree: Stay // 유지
    Stay,
}

/// Result of statistical test
/// Gantree: TestResult // 검정 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Whether the difference is significant
    /// Gantree: is_significant: bool // 유의 여부
    pub is_significant: bool,

    /// Computed z-score
    /// Gantree: z_score: f64 // z값
    pub z_score: f64,

    /// Critical z value used
    /// Gantree: z_critical: f64 // 임계값
    pub z_critical: f64,

    /// Determined direction (if significant)
    /// Gantree: direction: Option<Direction> // 확정 방향
    pub direction: Option<Direction>,

    /// Is a tie (difference < epsilon)
    pub is_tie: bool,
}

impl TestResult {
    /// Create a significant result
    pub fn significant(z_score: f64, z_critical: f64, direction: Direction) -> Self {
        Self {
            is_significant: true,
            z_score,
            z_critical,
            direction: Some(direction),
            is_tie: false,
        }
    }

    /// Create an insignificant result
    pub fn insignificant(z_score: f64, z_critical: f64) -> Self {
        Self {
            is_significant: false,
            z_score,
            z_critical,
            direction: None,
            is_tie: false,
        }
    }

    /// Create a tie result
    pub fn tie() -> Self {
        Self {
            is_significant: false,
            z_score: 0.0,
            z_critical: 0.0,
            direction: None,
            is_tie: true,
        }
    }
}

/// Statistical test for comparing two measurements
/// Gantree: StatisticalTest // z-test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTest {
    /// Test mode
    /// Gantree: mode: SigMode // 모드
    pub mode: SigMode,

    /// Base significance level
    /// Gantree: level: f64 // 유의 수준
    pub level: f64,
}

impl StatisticalTest {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new statistical test
    /// Gantree: new(mode,level) -> Self // 생성자
    pub fn new(mode: SigMode, level: f64) -> Self {
        Self { mode, level }
    }

    /// Create with default settings
    pub fn default_tqqc() -> Self {
        Self::new(SigMode::Fixed, 0.95)
    }

    /// Create fixed mode test
    pub fn fixed(level: f64) -> Self {
        Self::new(SigMode::Fixed, level)
    }

    /// Create adaptive mode test
    pub fn adaptive(level: f64) -> Self {
        Self::new(SigMode::Adaptive, level)
    }

    // ========================================================================
    // Z-Score Calculation
    // ========================================================================

    /// Compute z-score for two proportions
    /// Gantree: compute_z(p_plus,p_minus,n_plus,n_minus) -> f64 // z값
    ///
    /// Uses pooled proportion method:
    /// p̂ = (p₊N₊ + p₋N₋) / (N₊ + N₋)
    /// SE = sqrt(p̂(1-p̂)(1/N₊ + 1/N₋))
    /// Z = (p₊ - p₋) / SE
    pub fn compute_z(&self, p_plus: f64, p_minus: f64, n_plus: u64, n_minus: u64) -> f64 {
        if n_plus == 0 || n_minus == 0 {
            return 0.0;
        }

        // Gantree: pooled_p(p1,p2,n1,n2) -> f64 // 풀링 확률
        let pooled =
            (p_plus * n_plus as f64 + p_minus * n_minus as f64) / (n_plus + n_minus) as f64;

        // Gantree: standard_error(p,n1,n2) -> f64 // 표준 오차
        let se = (pooled * (1.0 - pooled) * (1.0 / n_plus as f64 + 1.0 / n_minus as f64)).sqrt();

        if se < 1e-10 {
            return 0.0;
        }

        (p_plus - p_minus).abs() / se
    }

    /// Compute z-score from parity expectations
    /// Uses simplified variance estimate
    pub fn compute_z_from_parity(&self, parity_plus: f64, parity_minus: f64, shots: u64) -> f64 {
        if shots == 0 {
            return 0.0;
        }

        // Standard error estimate: sqrt((1 - E²) / N)
        let var_plus = (1.0 - parity_plus.powi(2)) / shots as f64;
        let var_minus = (1.0 - parity_minus.powi(2)) / shots as f64;

        let se = (var_plus + var_minus).sqrt();

        if se < 1e-10 {
            return 0.0;
        }

        (parity_plus - parity_minus).abs() / se
    }

    // ========================================================================
    // Critical Value
    // ========================================================================

    /// Get z critical value
    /// Gantree: z_critical(&self,shots,noise) -> f64 // 임계값
    pub fn z_critical(&self, shots: u64, noise: f64) -> f64 {
        match self.mode {
            SigMode::Fixed => self.fixed_critical(),
            SigMode::Adaptive => self.adaptive_critical(shots, noise),
        }
    }

    /// Get fixed critical value
    /// Gantree: fixed_critical(level) -> f64 // 고정
    fn fixed_critical(&self) -> f64 {
        // Standard normal quantiles
        if self.level >= 0.99 {
            stats::Z_CRIT_99
        } else if self.level >= 0.95 {
            stats::Z_CRIT_95
        } else {
            stats::Z_CRIT_90
        }
    }

    /// Get adaptive critical value
    /// Gantree: adaptive_critical(level,shots,noise) -> f64 // 적응
    ///
    /// v2.2 adaptive rules:
    /// - High noise (>0.02): +0.025 to level
    /// - Low shots (<4096): +0.025 to level
    /// - High shots (>=16384): -0.05 from level
    fn adaptive_critical(&self, shots: u64, noise: f64) -> f64 {
        let mut adjusted_level = self.level;

        // High noise: more conservative
        if noise > 0.02 {
            adjusted_level += 0.025;
        }

        // Low shots: more conservative
        if shots < 4096 {
            adjusted_level += 0.025;
        }

        // High shots: less conservative
        if shots >= 16384 {
            adjusted_level -= 0.05;
        }

        // Clamp to valid range
        adjusted_level = adjusted_level.clamp(0.90, 0.99);

        // Return corresponding z value
        if adjusted_level >= 0.99 {
            stats::Z_CRIT_99
        } else if adjusted_level >= 0.975 {
            2.24 // 97.5%
        } else if adjusted_level >= 0.95 {
            stats::Z_CRIT_95
        } else {
            stats::Z_CRIT_90
        }
    }

    // ========================================================================
    // Significance Testing
    // ========================================================================

    /// Check if z-score is significant
    /// Gantree: is_significant(&self,z,shots,noise) -> bool // 유의성
    pub fn is_significant(&self, z: f64, shots: u64, noise: f64) -> bool {
        z > self.z_critical(shots, noise)
    }

    /// Perform test and return result
    /// Gantree: test(&self,SearchResult,shots,noise) -> TestResult // 검정
    pub fn test(&self, parity_plus: f64, parity_minus: f64, shots: u64, noise: f64) -> TestResult {
        // Check for tie
        const TIE_EPSILON: f64 = 1e-9;
        if (parity_plus - parity_minus).abs() < TIE_EPSILON {
            return TestResult::tie();
        }

        // Compute z-score
        let z_score = self.compute_z_from_parity(parity_plus, parity_minus, shots);
        let z_critical = self.z_critical(shots, noise);

        // Determine significance and direction
        if z_score > z_critical {
            let direction = if parity_plus > parity_minus {
                Direction::Plus
            } else {
                Direction::Minus
            };
            TestResult::significant(z_score, z_critical, direction)
        } else {
            TestResult::insignificant(z_score, z_critical)
        }
    }

    /// Test with even probabilities (proportion test)
    pub fn test_proportions(
        &self,
        p_even_plus: f64,
        p_even_minus: f64,
        shots: u64,
        noise: f64,
    ) -> TestResult {
        // Convert to parity
        let parity_plus = 2.0 * p_even_plus - 1.0;
        let parity_minus = 2.0 * p_even_minus - 1.0;

        self.test(parity_plus, parity_minus, shots, noise)
    }
}

impl Default for StatisticalTest {
    fn default() -> Self {
        Self::default_tqqc()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_test_new() {
        let st = StatisticalTest::new(SigMode::Fixed, 0.95);
        assert_eq!(st.mode, SigMode::Fixed);
        assert_eq!(st.level, 0.95);
    }

    #[test]
    fn test_fixed_critical() {
        let st_90 = StatisticalTest::fixed(0.90);
        let st_95 = StatisticalTest::fixed(0.95);
        let st_99 = StatisticalTest::fixed(0.99);

        assert!((st_90.z_critical(8192, 0.02) - 1.645).abs() < 0.01);
        assert!((st_95.z_critical(8192, 0.02) - 1.96).abs() < 0.01);
        assert!((st_99.z_critical(8192, 0.02) - 2.575).abs() < 0.01);
    }

    #[test]
    fn test_adaptive_critical() {
        let st = StatisticalTest::adaptive(0.95);

        // Normal case
        let z_normal = st.z_critical(8192, 0.02);

        // High noise should increase threshold
        let z_high_noise = st.z_critical(8192, 0.03);
        assert!(z_high_noise >= z_normal);

        // Low shots should increase threshold
        let z_low_shots = st.z_critical(2048, 0.02);
        assert!(z_low_shots >= z_normal);

        // High shots should decrease threshold
        let z_high_shots = st.z_critical(16384, 0.01);
        assert!(z_high_shots <= z_normal);
    }

    #[test]
    fn test_compute_z() {
        let st = StatisticalTest::default_tqqc();

        // Equal proportions -> z = 0
        let z = st.compute_z(0.5, 0.5, 1000, 1000);
        assert!(z < 1e-10);

        // Different proportions -> z > 0
        let z = st.compute_z(0.6, 0.4, 1000, 1000);
        assert!(z > 0.0);
    }

    #[test]
    fn test_compute_z_from_parity() {
        let st = StatisticalTest::default_tqqc();

        // Same parity -> z = 0
        let z = st.compute_z_from_parity(0.3, 0.3, 8192);
        assert!(z < 1e-10);

        // Different parity -> z > 0
        let z = st.compute_z_from_parity(0.5, 0.3, 8192);
        assert!(z > 0.0);
    }

    #[test]
    fn test_test_significant() {
        let st = StatisticalTest::fixed(0.95);

        // Large difference should be significant
        let result = st.test(0.8, 0.2, 8192, 0.02);
        assert!(result.is_significant);
        assert_eq!(result.direction, Some(Direction::Plus));
    }

    #[test]
    fn test_test_insignificant() {
        let st = StatisticalTest::fixed(0.95);

        // Small difference should be insignificant
        let result = st.test(0.51, 0.49, 100, 0.02);
        assert!(!result.is_significant);
        assert!(result.direction.is_none());
    }

    #[test]
    fn test_test_tie() {
        let st = StatisticalTest::default_tqqc();

        // Exactly equal -> tie
        let result = st.test(0.5, 0.5, 8192, 0.02);
        assert!(result.is_tie);
        assert!(!result.is_significant);
    }

    #[test]
    fn test_direction() {
        let st = StatisticalTest::fixed(0.95);

        // Plus better
        let result = st.test(0.8, 0.2, 8192, 0.02);
        assert_eq!(result.direction, Some(Direction::Plus));

        // Minus better
        let result = st.test(0.2, 0.8, 8192, 0.02);
        assert_eq!(result.direction, Some(Direction::Minus));
    }
}
