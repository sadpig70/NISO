//! Integrated configuration for NISO
//!
//! Gantree: L7_Integration → NisoConfig
//!
//! Unified configuration combining all NISO subsystems.

use niso_core::{BasisString, EntanglerType};
use niso_noise::{GateTimes, NoiseModel};
use niso_tqqc::{DeltaMode, SigMode, TqqcConfig};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Optimization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OptimizationMode {
    /// Full TQQC optimization
    #[default]
    Full,
    /// Quick optimization (fewer iterations)
    Quick,
    /// Benchmark mode (fixed parameters)
    Benchmark,
    /// Custom parameters
    Custom,
}

/// Hardware target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HardwareTarget {
    /// IBM superconducting qubits
    #[default]
    IbmSuperconducting,
    /// Trapped ion
    TrappedIon,
    /// Neutral atom
    NeutralAtom,
    /// Ideal simulator (no noise)
    Ideal,
    /// Custom hardware
    Custom,
}

/// Unified NISO configuration
/// Gantree: NisoConfig // 통합 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NisoConfig {
    // ========================================================================
    // Core Parameters
    // ========================================================================
    /// Number of qubits
    pub qubits: usize,

    /// Optimization mode
    pub mode: OptimizationMode,

    /// Hardware target
    pub hardware: HardwareTarget,

    // ========================================================================
    // TQQC Parameters
    // ========================================================================
    /// Number of outer iterations
    pub points: usize,

    /// Number of shots per measurement
    pub shots: u64,

    /// Effective depolarizing noise
    pub noise: f64,

    /// Step amplitude
    pub step_amp: f64,

    /// Maximum inner iterations
    pub inner_max: usize,

    /// Enable dynamic inner loop
    pub dynamic_inner: bool,

    /// Enable statistical testing
    pub use_statistical_test: bool,

    /// Significance mode
    pub sig_mode: SigMode,

    /// Significance level
    pub sig_level: f64,

    /// Delta accumulation mode
    pub delta_mode: DeltaMode,

    /// Measurement basis
    pub basis: BasisString,

    /// Entangler type
    pub entangler: EntanglerType,

    // ========================================================================
    // Hardware Parameters
    // ========================================================================
    /// T1 relaxation time (microseconds)
    pub t1_us: f64,

    /// T2 dephasing time (microseconds)
    pub t2_us: f64,

    /// Single-qubit gate error
    pub gate_error_1q: f64,

    /// Two-qubit gate error
    pub gate_error_2q: f64,

    /// Readout error
    pub readout_error: f64,

    /// Single-qubit gate time (nanoseconds)
    pub gate_time_1q_ns: f64,

    /// Two-qubit gate time (nanoseconds)
    pub gate_time_2q_ns: f64,

    // ========================================================================
    // Execution Parameters
    // ========================================================================
    /// Random seed
    pub seed: Option<u64>,

    /// Enable verbose output
    pub verbose: bool,

    /// Enable result caching
    pub cache_results: bool,
}

impl NisoConfig {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create default 7-qubit configuration
    pub fn default_7q() -> Self {
        Self {
            qubits: 7,
            mode: OptimizationMode::Full,
            hardware: HardwareTarget::IbmSuperconducting,
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
            t1_us: 100.0,
            t2_us: 60.0,
            gate_error_1q: 0.0003,
            gate_error_2q: 0.01,
            readout_error: 0.01,
            gate_time_1q_ns: 35.0,
            gate_time_2q_ns: 300.0,
            seed: None,
            verbose: false,
            cache_results: true,
        }
    }

    /// Create default 5-qubit configuration
    pub fn default_5q() -> Self {
        let mut config = Self::default_7q();
        config.qubits = 5;
        config.basis = BasisString::all_x(5);
        config
    }

    /// Create quick optimization configuration
    pub fn quick(qubits: usize) -> Self {
        Self {
            qubits,
            mode: OptimizationMode::Quick,
            points: 10,
            shots: 4096,
            dynamic_inner: true,
            inner_max: 5,
            basis: BasisString::all_x(qubits),
            ..Self::default_7q()
        }
    }

    /// Create benchmark configuration
    pub fn benchmark(qubits: usize) -> Self {
        Self {
            qubits,
            mode: OptimizationMode::Benchmark,
            points: 20,
            shots: 8192,
            dynamic_inner: true,
            use_statistical_test: false,
            basis: BasisString::all_x(qubits),
            seed: Some(42), // Fixed seed for reproducibility
            ..Self::default_7q()
        }
    }

    /// Create ideal (noiseless) configuration
    pub fn ideal(qubits: usize) -> Self {
        Self {
            qubits,
            hardware: HardwareTarget::Ideal,
            noise: 0.0,
            gate_error_1q: 0.0,
            gate_error_2q: 0.0,
            readout_error: 0.0,
            basis: BasisString::all_x(qubits),
            ..Self::default_7q()
        }
    }

    // ========================================================================
    // Builder Methods
    // ========================================================================

    /// Set number of qubits
    pub fn with_qubits(mut self, n: usize) -> Self {
        self.qubits = n;
        self.basis = BasisString::all_x(n);
        self
    }

    /// Set optimization mode
    pub fn with_mode(mut self, mode: OptimizationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set hardware target
    pub fn with_hardware(mut self, hardware: HardwareTarget) -> Self {
        self.hardware = hardware;

        // Apply hardware-specific defaults
        match hardware {
            HardwareTarget::IbmSuperconducting => {
                self.t1_us = 100.0;
                self.t2_us = 60.0;
                self.gate_time_1q_ns = 35.0;
                self.gate_time_2q_ns = 300.0;
            }
            HardwareTarget::TrappedIon => {
                self.t1_us = 1000.0;
                self.t2_us = 500.0;
                self.gate_time_1q_ns = 10000.0;
                self.gate_time_2q_ns = 200000.0;
            }
            HardwareTarget::NeutralAtom => {
                self.t1_us = 500.0;
                self.t2_us = 200.0;
                self.gate_time_1q_ns = 1000.0;
                self.gate_time_2q_ns = 1000.0;
            }
            HardwareTarget::Ideal => {
                self.noise = 0.0;
                self.gate_error_1q = 0.0;
                self.gate_error_2q = 0.0;
                self.readout_error = 0.0;
            }
            HardwareTarget::Custom => {}
        }

        self
    }

    /// Set noise level
    pub fn with_noise(mut self, noise: f64) -> Self {
        self.noise = noise;
        self
    }

    /// Set points
    pub fn with_points(mut self, points: usize) -> Self {
        self.points = points;
        self
    }

    /// Set shots
    pub fn with_shots(mut self, shots: u64) -> Self {
        self.shots = shots;
        self
    }

    /// Set seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set dynamic inner
    pub fn with_dynamic_inner(mut self, enabled: bool) -> Self {
        self.dynamic_inner = enabled;
        self
    }

    /// Set statistical test
    pub fn with_statistical_test(mut self, enabled: bool) -> Self {
        self.use_statistical_test = enabled;
        self
    }

    // ========================================================================
    // Conversions
    // ========================================================================

    /// Convert to TqqcConfig
    pub fn to_tqqc_config(&self) -> TqqcConfig {
        TqqcConfig::for_qubits(self.qubits)
            .with_noise(self.noise)
            .with_points(self.points)
            .with_shots(self.shots)
            .with_step_amp(self.step_amp)
            .with_inner_max(self.inner_max)
            .with_dynamic_inner(self.dynamic_inner)
            .with_statistical_test(self.use_statistical_test)
            .with_sig_mode(self.sig_mode)
            .with_sig_level(self.sig_level)
            .with_delta_mode(self.delta_mode)
            .with_basis(self.basis.clone())
            .with_entangler(self.entangler)
    }

    /// Convert to NoiseModel
    pub fn to_noise_model(&self) -> NoiseModel {
        NoiseModel::new(
            self.t1_us,
            self.t2_us,
            self.gate_error_1q,
            self.gate_error_2q,
            self.readout_error,
        )
        .unwrap_or_else(|_| NoiseModel::ibm_typical())
    }

    /// Convert to GateTimes
    pub fn to_gate_times(&self) -> GateTimes {
        GateTimes::new(
            self.gate_time_1q_ns,
            self.gate_time_2q_ns,
            5000.0, // Measurement time
        )
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate configuration
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

        if self.noise < 0.0 || self.noise > 0.1 {
            return Err(format!("noise must be in [0, 0.1], got {}", self.noise));
        }

        if self.t2_us > 2.0 * self.t1_us {
            return Err(format!(
                "T2 ({}) must be <= 2*T1 ({})",
                self.t2_us,
                2.0 * self.t1_us
            ));
        }

        Ok(())
    }

    /// Check if configuration is TQQC-recommended
    pub fn is_recommended(&self) -> bool {
        self.noise <= 0.020 && self.shots >= 4096
    }
}

impl Default for NisoConfig {
    fn default() -> Self {
        Self::default_7q()
    }
}

impl fmt::Display for NisoConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NisoConfig({}Q, {:?}, noise={:.3}, points={}, shots={})",
            self.qubits, self.mode, self.noise, self.points, self.shots
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
        let config = NisoConfig::default_7q();
        assert_eq!(config.qubits, 7);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_quick_mode() {
        let config = NisoConfig::quick(5);
        assert_eq!(config.qubits, 5);
        assert_eq!(config.mode, OptimizationMode::Quick);
        assert_eq!(config.points, 10);
    }

    #[test]
    fn test_benchmark_mode() {
        let config = NisoConfig::benchmark(7);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_ideal_mode() {
        let config = NisoConfig::ideal(5);
        assert_eq!(config.noise, 0.0);
        assert_eq!(config.gate_error_1q, 0.0);
    }

    #[test]
    fn test_hardware_targets() {
        let ibm = NisoConfig::default_7q().with_hardware(HardwareTarget::IbmSuperconducting);
        assert_eq!(ibm.gate_time_1q_ns, 35.0);

        let ion = NisoConfig::default_7q().with_hardware(HardwareTarget::TrappedIon);
        assert!(ion.gate_time_1q_ns > ibm.gate_time_1q_ns);
    }

    #[test]
    fn test_to_tqqc_config() {
        let niso_config = NisoConfig::default_7q().with_noise(0.015);
        let tqqc_config = niso_config.to_tqqc_config();

        assert_eq!(tqqc_config.qubits, 7);
        assert_eq!(tqqc_config.noise, 0.015);
    }

    #[test]
    fn test_to_noise_model() {
        let config = NisoConfig::default_7q();
        let noise_model = config.to_noise_model();

        assert!(noise_model.validate().is_ok());
    }

    #[test]
    fn test_validation() {
        // Valid
        assert!(NisoConfig::default_7q().validate().is_ok());

        // Invalid: qubits < 2
        assert!(NisoConfig::default_7q().with_qubits(1).validate().is_err());
    }

    #[test]
    fn test_is_recommended() {
        let recommended = NisoConfig::default_7q().with_noise(0.015);
        assert!(recommended.is_recommended());

        let not_recommended = NisoConfig::default_7q().with_noise(0.03);
        assert!(!not_recommended.is_recommended());
    }
}
