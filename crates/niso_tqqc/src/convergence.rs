//! Convergence and dynamic inner loop for TQQC
//!
//! Gantree: L5_TQQC → Convergence, DynamicInner
//!
//! Implements early stopping and adaptive inner iteration logic.

use niso_core::tqqc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Convergence checker for TQQC
/// Gantree: Convergence // 수렴 판단
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Convergence {
    /// Window size for checking recent improvements
    /// Gantree: window: usize // 윈도우 크기
    pub window: usize,

    /// Absolute threshold for improvement
    /// Gantree: threshold_abs: f64 // 절대 임계
    pub threshold_abs: f64,

    /// Cumulative threshold
    /// Gantree: threshold_cum: f64 // 누적 임계
    pub threshold_cum: f64,

    /// History of recent improvements
    /// Gantree: history: VecDeque<f64> // 이력
    history: VecDeque<f64>,

    /// Cumulative improvement
    cumulative: f64,
}

impl Convergence {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new convergence checker
    /// Gantree: new(qubits,base_threshold) -> Self // 생성자
    pub fn new(qubits: usize, base_threshold: f64) -> Self {
        // Apply depth correction
        // Gantree: depth_correction(qubits,ref_q,ref_th) -> f64 // 깊이 보정
        let threshold_abs = Self::depth_correction(qubits, 5, base_threshold);
        let threshold_cum = threshold_abs * 1.5;

        Self {
            window: 3,
            threshold_abs,
            threshold_cum,
            history: VecDeque::with_capacity(3),
            cumulative: 0.0,
        }
    }

    /// Create with default TQQC parameters
    pub fn default_for_qubits(qubits: usize) -> Self {
        Self::new(qubits, tqqc::THRESHOLD_5Q)
    }

    /// Create from noise level
    pub fn from_noise(qubits: usize, noise: f64) -> Self {
        // Use higher threshold for higher noise
        let base = if noise <= 0.02 { 0.030 } else { 0.040 };
        Self::new(qubits, base)
    }

    // ========================================================================
    // Depth Correction
    // ========================================================================

    /// Apply depth correction for qubit count
    /// Formula: threshold_N = threshold_Nref × (D_Nref / D_N)
    /// where D_N = N - 1 (linear chain depth)
    fn depth_correction(qubits: usize, ref_qubits: usize, ref_threshold: f64) -> f64 {
        let d_n = (qubits.saturating_sub(1)) as f64;
        let d_ref = (ref_qubits.saturating_sub(1)) as f64;

        if d_n > 0.0 && d_ref > 0.0 {
            ref_threshold * (d_ref / d_n)
        } else {
            ref_threshold
        }
    }

    // ========================================================================
    // Convergence Check
    // ========================================================================

    /// Add improvement to history and check convergence
    /// Gantree: push(&mut,improvement) // 이력 추가
    pub fn push(&mut self, improvement: f64) {
        self.history.push_back(improvement);
        self.cumulative += improvement;

        // Maintain window size
        while self.history.len() > self.window {
            self.history.pop_front();
        }
    }

    /// Check if converged
    /// Gantree: check(&self) -> bool // 수렴 체크
    pub fn check(&self) -> bool {
        self.window_condition() && self.cumulative_condition()
    }

    /// Check window condition: all recent improvements below threshold
    /// Gantree: window_condition(&self) -> bool // 윈도우 조건
    pub fn window_condition(&self) -> bool {
        if self.history.len() < self.window {
            return false;
        }

        self.history
            .iter()
            .all(|&imp| imp.abs() < self.threshold_abs)
    }

    /// Check cumulative condition: total improvement below threshold
    /// Gantree: cumulative_condition(&self) -> bool // 누적 조건
    pub fn cumulative_condition(&self) -> bool {
        self.cumulative.abs() < self.threshold_cum
    }

    /// Reset convergence state
    /// Gantree: reset(&mut) // 초기화
    pub fn reset(&mut self) {
        self.history.clear();
        self.cumulative = 0.0;
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get current cumulative improvement
    pub fn cumulative(&self) -> f64 {
        self.cumulative
    }

    /// Get history length
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Get threshold
    pub fn threshold(&self) -> f64 {
        self.threshold_abs
    }
}

/// Dynamic inner loop controller
/// Gantree: DynamicInner // 적응형 내부 반복
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicInner {
    /// Maximum inner iterations
    /// Gantree: inner_max: usize // 최대
    pub inner_max: usize,

    /// Decay rate for step size
    /// Gantree: decay_rate: f64 // 감쇠율
    pub decay_rate: f64,

    /// Safety multiplier cap
    safety_cap: usize,
}

impl DynamicInner {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new dynamic inner controller
    /// Gantree: new(max,decay) -> Self // 생성자
    pub fn new(inner_max: usize, decay_rate: f64) -> Self {
        Self {
            inner_max,
            decay_rate,
            safety_cap: 5, // 5x multiplier cap
        }
    }

    /// Create with TQQC defaults
    pub fn default_tqqc() -> Self {
        Self::new(10, 0.9)
    }

    // ========================================================================
    // Inner Count Calculation
    // ========================================================================

    /// Compute inner iteration count based on last improvement
    /// Gantree: compute_count(&self,last_improve,threshold) -> usize // 반복 수
    ///
    /// Formula: inner_count = clamp(1, inner_max, 1 + 2·⌊|g| / τ⌋)
    /// where g = last improvement, τ = threshold
    pub fn compute_count(&self, last_improve: f64, threshold: f64) -> usize {
        // Gantree: raw_count(improve,threshold) -> usize // 원시 계산
        let tau = threshold.max(1e-9);
        let raw = 1 + 2 * (last_improve.abs() / tau).floor() as usize;

        // Gantree: apply_cap(raw,max) -> usize // 상한 적용
        // Apply safety cap (5x) then inner_max
        let capped = raw.min(self.safety_cap);
        capped.max(1).min(self.inner_max)
    }

    /// Compute step size for j-th inner iteration
    /// Gantree: compute_step(&self,j,base_step) -> f64 // j번째 스텝
    ///
    /// Formula: δ_j = step_amp · 0.9^j
    pub fn compute_step(&self, j: usize, base_step: f64) -> f64 {
        base_step * self.decay_rate.powi(j as i32)
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get maximum inner iterations
    pub fn max(&self) -> usize {
        self.inner_max
    }

    /// Get decay rate
    pub fn decay(&self) -> f64 {
        self.decay_rate
    }
}

impl Default for DynamicInner {
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
    fn test_convergence_new() {
        let conv = Convergence::new(7, 0.030);

        // 7Q threshold = 0.030 * (4/6) = 0.020
        assert!((conv.threshold_abs - 0.020).abs() < 1e-6);
        assert!((conv.threshold_cum - 0.030).abs() < 1e-6);
    }

    #[test]
    fn test_convergence_5q() {
        let conv = Convergence::new(5, 0.030);

        // 5Q threshold = 0.030 (reference)
        assert!((conv.threshold_abs - 0.030).abs() < 1e-6);
    }

    #[test]
    fn test_convergence_check() {
        let mut conv = Convergence::new(7, 0.030);

        // Not enough history
        assert!(!conv.check());

        // Add improvements below threshold
        conv.push(0.01);
        conv.push(0.005);
        conv.push(0.003);

        // Window condition satisfied, but cumulative might not be
        // cumulative = 0.018, threshold_cum = 0.030
        assert!(conv.window_condition());
    }

    #[test]
    fn test_convergence_window() {
        let mut conv = Convergence::new(5, 0.030);

        // Add large improvements
        conv.push(0.1);
        conv.push(0.05);
        conv.push(0.02);

        // Should not satisfy window condition
        assert!(!conv.window_condition());

        // Add small improvements
        conv.push(0.01);
        conv.push(0.005);
        conv.push(0.002);

        // Now should satisfy (last 3 are small)
        assert!(conv.window_condition());
    }

    #[test]
    fn test_convergence_reset() {
        let mut conv = Convergence::new(7, 0.030);

        conv.push(0.1);
        conv.push(0.05);

        assert!(conv.cumulative() > 0.0);
        assert!(conv.history_len() > 0);

        conv.reset();

        assert_eq!(conv.cumulative(), 0.0);
        assert_eq!(conv.history_len(), 0);
    }

    #[test]
    fn test_dynamic_inner_count() {
        let di = DynamicInner::default_tqqc();

        // Small improvement -> 1 iteration
        let count = di.compute_count(0.005, 0.020);
        assert_eq!(count, 1);

        // Medium improvement -> more iterations
        let count = di.compute_count(0.040, 0.020);
        // 1 + 2 * floor(0.040 / 0.020) = 1 + 2 * 2 = 5
        assert_eq!(count, 5);

        // Large improvement -> capped at safety_cap
        let count = di.compute_count(0.200, 0.020);
        // Would be 1 + 2 * 10 = 21, but capped at 5
        assert_eq!(count, 5);
    }

    #[test]
    fn test_dynamic_inner_step() {
        let di = DynamicInner::default_tqqc();
        let base_step = 0.12;

        // j=0: 0.12 * 0.9^0 = 0.12
        assert!((di.compute_step(0, base_step) - 0.12).abs() < 1e-10);

        // j=1: 0.12 * 0.9^1 = 0.108
        assert!((di.compute_step(1, base_step) - 0.108).abs() < 1e-10);

        // j=2: 0.12 * 0.9^2 = 0.0972
        assert!((di.compute_step(2, base_step) - 0.0972).abs() < 1e-10);
    }

    #[test]
    fn test_depth_correction() {
        // 5Q -> 5Q (no change)
        let t5 = Convergence::depth_correction(5, 5, 0.030);
        assert!((t5 - 0.030).abs() < 1e-10);

        // 5Q -> 7Q
        let t7 = Convergence::depth_correction(7, 5, 0.030);
        // 0.030 * (4/6) = 0.020
        assert!((t7 - 0.020).abs() < 1e-10);

        // 5Q -> 9Q
        let t9 = Convergence::depth_correction(9, 5, 0.030);
        // 0.030 * (4/8) = 0.015
        assert!((t9 - 0.015).abs() < 1e-10);
    }
}
