//! TQQC configuration
//!
//! Gantree: L5_TQQC → TqqcConfig
//!
//! Configuration for the TQQC optimization engine.

use niso_core::{tqqc, BasisString, EntanglerType};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Significance test mode
/// Gantree: SigMode // 유의성 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SigMode {
    /// Fixed z-critical threshold
    /// Gantree: Fixed // 고정
    #[default]
    Fixed,

    /// Adaptive threshold based on noise/shots
    /// Gantree: Adaptive // 적응형
    Adaptive,
}

/// Delta accumulation mode
/// Gantree: DeltaMode // 델타 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeltaMode {
    /// Accumulate delta across iterations
    /// Gantree: Track // 누적
    #[default]
    Track,

    /// Reset delta each iteration
    /// Gantree: Reset // 초기화
    Reset,
}

/// TQQC configuration
/// Gantree: TqqcConfig // 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TqqcConfig {
    /// Number of qubits
    /// Gantree: qubits: usize // 큐비트 수 (7)
    pub qubits: usize,

    /// Number of outer loop iterations
    /// Gantree: points: usize // 외부 루프 (20)
    pub points: usize,

    /// Number of measurement shots
    /// Gantree: shots: u64 // 샷 수 (8192)
    pub shots: u64,

    /// Effective depolarizing noise level
    /// Gantree: noise: f64 // 유효 depol
    pub noise: f64,

    /// Step amplitude for delta search
    /// Gantree: step_amp: f64 // 스텝 크기
    pub step_amp: f64,

    /// Maximum inner iterations
    /// Gantree: inner_max: usize // 내부 최대
    pub inner_max: usize,

    /// Enable dynamic inner loop
    /// Gantree: dynamic_inner: bool // 적응형
    pub dynamic_inner: bool,

    /// Enable statistical test
    /// Gantree: use_statistical_test: bool // z-test
    pub use_statistical_test: bool,

    /// Significance test mode
    /// Gantree: sig_mode: SigMode // fixed/adaptive
    pub sig_mode: SigMode,

    /// Significance level
    /// Gantree: sig_level: f64 // 유의 수준
    pub sig_level: f64,

    /// Delta accumulation mode
    /// Gantree: delta_mode: DeltaMode // track/reset
    pub delta_mode: DeltaMode,

    /// Measurement basis
    /// Gantree: basis: String // 기저 문자열
    pub basis: BasisString,

    /// Entangler type
    /// Gantree: entangler: Entangler // cx/cz
    pub entangler: EntanglerType,

    /// Initial theta value
    pub theta_init: f64,

    /// Initial delta value
    pub delta_init: f64,

    /// Random seed
    /// Gantree: seed: Option<u64> // 시드
    pub seed: Option<u64>,
}

impl TqqcConfig {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create default 7-qubit configuration
    /// Gantree: default() -> Self // 기본값
    pub fn default_7q() -> Self {
        Self {
            qubits: 7,
            points: 20,
            shots: 8192,
            noise: 0.02,
            step_amp: 0.12,
            inner_max: 10,
            dynamic_inner: true,
            use_statistical_test: false,
            sig_mode: SigMode::Fixed,
            sig_level: 0.95,
            delta_mode: DeltaMode::Track,
            basis: BasisString::all_x(7),
            entangler: EntanglerType::Cx,
            theta_init: 0.0,
            delta_init: 0.0,
            seed: None,
        }
    }

    /// Create 5-qubit configuration
    pub fn default_5q() -> Self {
        let mut config = Self::default_7q();
        config.qubits = 5;
        config.basis = BasisString::all_x(5);
        config
    }

    /// Create configuration for a specific number of qubits
    pub fn for_qubits(n: usize) -> Self {
        let mut config = Self::default_7q();
        config.qubits = n;
        config.basis = BasisString::all_x(n);
        config
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set number of qubits
    /// Gantree: with_qubits(&mut,n) -> &mut Self // 큐비트 설정
    pub fn with_qubits(mut self, n: usize) -> Self {
        self.qubits = n;
        self.basis = BasisString::all_x(n);
        self
    }

    /// Set noise level
    /// Gantree: with_noise(&mut,n) -> &mut Self // 노이즈 설정
    pub fn with_noise(mut self, noise: f64) -> Self {
        self.noise = noise;
        self
    }

    /// Set number of points
    pub fn with_points(mut self, points: usize) -> Self {
        self.points = points;
        self
    }

    /// Set number of shots
    pub fn with_shots(mut self, shots: u64) -> Self {
        self.shots = shots;
        self
    }

    /// Set step amplitude
    pub fn with_step_amp(mut self, step_amp: f64) -> Self {
        self.step_amp = step_amp;
        self
    }

    /// Set inner max
    pub fn with_inner_max(mut self, inner_max: usize) -> Self {
        self.inner_max = inner_max;
        self
    }

    /// Enable/disable dynamic inner
    pub fn with_dynamic_inner(mut self, enabled: bool) -> Self {
        self.dynamic_inner = enabled;
        self
    }

    /// Enable/disable statistical test
    pub fn with_statistical_test(mut self, enabled: bool) -> Self {
        self.use_statistical_test = enabled;
        self
    }

    /// Set significance mode
    pub fn with_sig_mode(mut self, mode: SigMode) -> Self {
        self.sig_mode = mode;
        self
    }

    /// Set significance level
    pub fn with_sig_level(mut self, level: f64) -> Self {
        self.sig_level = level;
        self
    }

    /// Set delta mode
    pub fn with_delta_mode(mut self, mode: DeltaMode) -> Self {
        self.delta_mode = mode;
        self
    }

    /// Set basis
    pub fn with_basis(mut self, basis: BasisString) -> Self {
        self.basis = basis;
        self
    }

    /// Set entangler
    pub fn with_entangler(mut self, entangler: EntanglerType) -> Self {
        self.entangler = entangler;
        self
    }

    /// Set initial theta
    pub fn with_theta(mut self, theta: f64) -> Self {
        self.theta_init = theta;
        self
    }

    /// Set initial delta
    pub fn with_delta(mut self, delta: f64) -> Self {
        self.delta_init = delta;
        self
    }

    /// Set seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    // ========================================================================
    // Derived Values
    // ========================================================================

    /// Get convergence threshold for this configuration
    pub fn threshold(&self) -> f64 {
        tqqc::threshold_for_qubits(self.qubits)
    }

    /// Get depth ratio for this configuration
    pub fn depth_ratio(&self) -> f64 {
        tqqc::depth_ratio(self.qubits)
    }

    /// Check if noise is within TQQC recommended range
    pub fn is_recommended_noise(&self) -> bool {
        self.noise <= 0.020
    }

    /// Check if noise is within TQQC valid range
    pub fn is_valid_noise(&self) -> bool {
        self.noise <= self.threshold()
    }

    /// Check if noise exceeds critical point
    pub fn exceeds_critical(&self) -> bool {
        self.noise > self.threshold()
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate configuration
    /// Gantree: validate(&self) -> Result // 검증
    pub fn validate(&self) -> Result<(), String> {
        if self.qubits < 2 {
            return Err("qubits must be >= 2".to_string());
        }

        if self.points == 0 {
            return Err("points must be > 0".to_string());
        }

        if self.shots == 0 {
            return Err("shots must be > 0".to_string());
        }

        if self.noise < 0.0 || self.noise > 0.06 {
            return Err(format!("noise must be in [0, 0.06], got {}", self.noise));
        }

        if self.step_amp <= 0.0 {
            return Err("step_amp must be > 0".to_string());
        }

        if self.inner_max == 0 {
            return Err("inner_max must be > 0".to_string());
        }

        if self.sig_level < 0.8 || self.sig_level > 0.99 {
            return Err(format!(
                "sig_level must be in [0.8, 0.99], got {}",
                self.sig_level
            ));
        }

        if self.basis.len() != self.qubits {
            return Err(format!(
                "basis length {} doesn't match qubits {}",
                self.basis.len(),
                self.qubits
            ));
        }

        Ok(())
    }
}

impl Default for TqqcConfig {
    fn default() -> Self {
        Self::default_7q()
    }
}

impl fmt::Display for TqqcConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TqqcConfig({}Q, points={}, shots={}, noise={:.3}, dynamic={})",
            self.qubits, self.points, self.shots, self.noise, self.dynamic_inner
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
    fn test_default_7q() {
        let config = TqqcConfig::default_7q();

        assert_eq!(config.qubits, 7);
        assert_eq!(config.points, 20);
        assert_eq!(config.shots, 8192);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_builder() {
        let config = TqqcConfig::default_7q()
            .with_qubits(5)
            .with_noise(0.015)
            .with_shots(4096)
            .with_dynamic_inner(true)
            .with_statistical_test(true)
            .with_seed(42);

        assert_eq!(config.qubits, 5);
        assert_eq!(config.noise, 0.015);
        assert_eq!(config.shots, 4096);
        assert!(config.dynamic_inner);
        assert!(config.use_statistical_test);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_threshold() {
        let config_7q = TqqcConfig::default_7q();
        let config_5q = TqqcConfig::default_5q();

        // 7Q threshold should be lower than 5Q
        assert!(config_7q.threshold() < config_5q.threshold());
    }

    #[test]
    fn test_validation() {
        // Valid config
        let config = TqqcConfig::default_7q();
        assert!(config.validate().is_ok());

        // Invalid: qubits < 2
        let config = TqqcConfig::default_7q().with_qubits(1);
        assert!(config.validate().is_err());

        // Invalid: noise > 0.06
        let config = TqqcConfig::default_7q().with_noise(0.1);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_noise_ranges() {
        let config = TqqcConfig::default_7q().with_noise(0.015);
        assert!(config.is_recommended_noise());
        assert!(config.is_valid_noise());

        let config = TqqcConfig::default_7q().with_noise(0.025);
        assert!(!config.is_recommended_noise());
        assert!(!config.is_valid_noise()); // Exceeds 7Q threshold
    }
}
