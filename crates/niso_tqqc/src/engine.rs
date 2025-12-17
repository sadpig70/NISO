//! TQQC optimization engine
//!
//! Gantree: L5_TQQC → TqqcEngine
//!
//! Main TQQC optimization engine implementing delta search
//! with dynamic inner loop and statistical testing.

use crate::config::{DeltaMode, TqqcConfig};
use crate::convergence::{Convergence, DynamicInner};
use crate::parity::Parity;
use crate::stat_test::{Direction, StatisticalTest, TestResult};
use niso_backend::Backend;
use niso_core::NisoResult;
use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// TQQC optimization result
/// Gantree: TqqcResult // 최적화 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TqqcResult {
    /// Optimized delta value
    pub delta_opt: f64,

    /// Baseline parity (delta=0)
    pub parity_baseline: f64,

    /// Final parity
    pub parity_final: f64,

    /// Improvement (final - baseline)
    pub improvement: f64,

    /// Number of outer iterations
    pub iterations: usize,

    /// Whether early stop was triggered
    pub early_stopped: bool,

    /// Number of ties
    pub ties_count: usize,

    /// Number of significant moves
    pub significant_moves: usize,

    /// Total inner iterations
    pub total_inner_iterations: usize,

    /// Iteration history
    pub history: Vec<IterationRecord>,
}

impl TqqcResult {
    /// Get improvement percentage
    pub fn improvement_percent(&self) -> f64 {
        if self.parity_baseline.abs() < 1e-9 {
            return 0.0;
        }
        (self.improvement / self.parity_baseline.abs()) * 100.0
    }

    /// Check if optimization improved
    pub fn improved(&self) -> bool {
        self.improvement > 0.0
    }

    /// Calculate k_estimated (early stop efficiency)
    pub fn k_estimated(&self, max_points: usize) -> f64 {
        if !self.early_stopped || self.iterations >= max_points {
            return 1.0;
        }

        let early_stop_frac = 1.0 - (self.iterations as f64 / max_points as f64);
        let compute_reduction =
            1.0 - ((2 * self.iterations + 1) as f64 / (2 * max_points + 1) as f64);

        if early_stop_frac > 1e-9 {
            compute_reduction / early_stop_frac
        } else {
            1.0
        }
    }
}

/// Record of a single iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    /// Iteration number
    pub iteration: usize,

    /// Delta value at this iteration
    pub delta: f64,

    /// Parity at +delta
    pub parity_plus: f64,

    /// Parity at -delta
    pub parity_minus: f64,

    /// Selected parity
    pub parity_selected: f64,

    /// Improvement from previous
    pub improvement: f64,

    /// Inner count used
    pub inner_count: usize,

    /// Direction chosen
    pub direction: Option<Direction>,

    /// Was significant
    pub is_significant: bool,
}

/// TQQC optimization engine
/// Gantree: TqqcEngine // 통합 엔진
pub struct TqqcEngine<B: Backend> {
    /// Configuration
    config: TqqcConfig,

    /// Backend for circuit execution
    backend: B,

    /// Convergence checker
    convergence: Convergence,

    /// Dynamic inner controller
    dynamic_inner: DynamicInner,

    /// Statistical test
    stat_test: StatisticalTest,

    /// Random generator
    rng: StdRng,
}

impl<B: Backend> TqqcEngine<B> {
    // ========================================================================
    // Constructor
    // ========================================================================

    /// Create new TQQC engine
    pub fn new(config: TqqcConfig, backend: B) -> Self {
        let convergence = Convergence::from_noise(config.qubits, config.noise);
        let dynamic_inner = DynamicInner::new(config.inner_max, 0.9);
        let stat_test = StatisticalTest::new(config.sig_mode, config.sig_level);

        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        Self {
            config,
            backend,
            convergence,
            dynamic_inner,
            stat_test,
            rng,
        }
    }

    // ========================================================================
    // Main Optimization
    // ========================================================================

    /// Run TQQC optimization
    pub fn optimize(&mut self) -> NisoResult<TqqcResult> {
        let theta = self.config.theta_init;
        let mut delta = self.config.delta_init;
        let mut last_improve = 0.0;
        let mut total_inner = 0;

        // Baseline measurement (delta=0)
        let parity_baseline = self.measure_parity(theta, 0.0)?;
        let mut parity_current = parity_baseline;

        // Tracking
        let mut early_stopped = false;
        let mut ties_count = 0;
        let mut significant_moves = 0;
        let mut history = Vec::with_capacity(self.config.points);

        // Main optimization loop
        for iteration in 0..self.config.points {
            // Dynamic inner count
            let inner_count = if self.config.dynamic_inner {
                self.dynamic_inner
                    .compute_count(last_improve, self.convergence.threshold())
            } else {
                1
            };

            let mut best_delta = delta;
            let mut best_parity = parity_current;
            let mut record_parity_plus = 0.0;
            let mut record_parity_minus = 0.0;
            let mut record_direction = None;
            let mut record_significant = false;

            // Inner loop
            for j in 0..inner_count {
                let step_j = self.dynamic_inner.compute_step(j, self.config.step_amp);

                // Evaluate +delta and -delta
                let parity_plus = self.measure_parity(theta, delta + step_j)?;
                let parity_minus = self.measure_parity(theta, delta - step_j)?;

                if j == 0 {
                    record_parity_plus = parity_plus;
                    record_parity_minus = parity_minus;
                }

                // Statistical test
                let (candidate_delta, candidate_parity, direction, _is_significant) =
                    if self.config.use_statistical_test {
                        let test_result = self.stat_test.test(
                            parity_plus,
                            parity_minus,
                            self.config.shots,
                            self.config.noise,
                        );

                        if test_result.is_significant {
                            significant_moves += 1;
                            record_significant = true;
                        }

                        self.select_direction(
                            delta,
                            step_j,
                            parity_plus,
                            parity_minus,
                            parity_current,
                            &test_result,
                        )
                    } else {
                        // No statistical test: always select better
                        if parity_plus > parity_minus {
                            (delta + step_j, parity_plus, Some(Direction::Plus), true)
                        } else {
                            (delta - step_j, parity_minus, Some(Direction::Minus), true)
                        }
                    };

                if j == 0 {
                    record_direction = direction;
                }

                // Update best
                if candidate_parity > best_parity {
                    best_delta = candidate_delta;
                    best_parity = candidate_parity;
                }
            }

            // Apply best from inner loop
            let improvement = best_parity - parity_current;

            // Update delta based on mode
            delta = match self.config.delta_mode {
                DeltaMode::Track => best_delta,
                DeltaMode::Reset => best_delta - delta, // Relative change
            };

            parity_current = best_parity;
            last_improve = improvement;
            total_inner += inner_count;

            // Check for ties
            if (record_parity_plus - record_parity_minus).abs() < 1e-9 {
                ties_count += 1;
            }

            // Record iteration
            history.push(IterationRecord {
                iteration,
                delta,
                parity_plus: record_parity_plus,
                parity_minus: record_parity_minus,
                parity_selected: parity_current,
                improvement,
                inner_count,
                direction: record_direction,
                is_significant: record_significant,
            });

            // Convergence check
            self.convergence.push(improvement);
            if self.config.dynamic_inner && self.convergence.check() {
                early_stopped = true;
                break;
            }
        }

        Ok(TqqcResult {
            delta_opt: delta,
            parity_baseline,
            parity_final: parity_current,
            improvement: parity_current - parity_baseline,
            iterations: history.len(),
            early_stopped,
            ties_count,
            significant_moves,
            total_inner_iterations: total_inner,
            history,
        })
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Measure parity at given theta+delta
    fn measure_parity(&self, theta: f64, delta: f64) -> NisoResult<f64> {
        let circuit = Parity::build_circuit(&self.config, theta, delta);
        let result = self.backend.execute(&circuit, self.config.shots)?;
        Ok(Parity::expectation(&result.counts))
    }

    /// Select direction based on statistical test
    fn select_direction(
        &mut self,
        delta: f64,
        step: f64,
        parity_plus: f64,
        parity_minus: f64,
        parity_current: f64,
        test_result: &TestResult,
    ) -> (f64, f64, Option<Direction>, bool) {
        if test_result.is_tie {
            // Tie: random selection
            if self.rng.gen::<f64>() > 0.5 {
                (delta + step, parity_plus, Some(Direction::Plus), false)
            } else {
                (delta - step, parity_minus, Some(Direction::Minus), false)
            }
        } else if test_result.is_significant {
            // Significant: follow direction
            match test_result.direction {
                Some(Direction::Plus) => (delta + step, parity_plus, Some(Direction::Plus), true),
                Some(Direction::Minus) => {
                    (delta - step, parity_minus, Some(Direction::Minus), true)
                }
                _ => (delta, parity_current, Some(Direction::Stay), true),
            }
        } else {
            // Not significant: conservative (stay)
            (delta, parity_current, Some(Direction::Stay), false)
        }
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Get configuration
    pub fn config(&self) -> &TqqcConfig {
        &self.config
    }

    /// Get backend
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use niso_backend::SimulatorBackend;

    fn make_test_engine() -> TqqcEngine<SimulatorBackend> {
        let config = TqqcConfig::default_5q()
            .with_noise(0.01)
            .with_points(5)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(5, 0.01).unwrap().with_seed(42);

        TqqcEngine::new(config, backend)
    }

    #[test]
    fn test_engine_creation() {
        let engine = make_test_engine();
        assert_eq!(engine.config().qubits, 5);
    }

    #[test]
    fn test_measure_parity() {
        let engine = make_test_engine();
        let parity = engine.measure_parity(0.0, 0.0).unwrap();

        // Parity should be in [-1, 1]
        assert!(parity >= -1.0 && parity <= 1.0);
    }

    #[test]
    fn test_optimize_runs() {
        let mut engine = make_test_engine();
        let result = engine.optimize().unwrap();

        // Should have run iterations
        assert!(result.iterations > 0);

        // Parity should be valid
        assert!(result.parity_baseline >= -1.0 && result.parity_baseline <= 1.0);
        assert!(result.parity_final >= -1.0 && result.parity_final <= 1.0);
    }

    #[test]
    fn test_optimize_with_dynamic_inner() {
        let config = TqqcConfig::default_5q()
            .with_noise(0.01)
            .with_points(5)
            .with_dynamic_inner(true)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(5, 0.01).unwrap().with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // Should have tracked inner iterations
        assert!(result.total_inner_iterations >= result.iterations);
    }

    #[test]
    fn test_optimize_with_statistical_test() {
        let config = TqqcConfig::default_5q()
            .with_noise(0.01)
            .with_points(5)
            .with_statistical_test(true)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(5, 0.01).unwrap().with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // Should have history
        assert!(!result.history.is_empty());
    }

    #[test]
    fn test_result_improvement_percent() {
        let result = TqqcResult {
            delta_opt: 0.1,
            parity_baseline: 0.5,
            parity_final: 0.6,
            improvement: 0.1,
            iterations: 10,
            early_stopped: false,
            ties_count: 0,
            significant_moves: 5,
            total_inner_iterations: 10,
            history: vec![],
        };

        // 0.1 / 0.5 * 100 = 20%
        assert!((result.improvement_percent() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_result_k_estimated() {
        let result = TqqcResult {
            delta_opt: 0.1,
            parity_baseline: 0.5,
            parity_final: 0.6,
            improvement: 0.1,
            iterations: 10,
            early_stopped: true,
            ties_count: 0,
            significant_moves: 5,
            total_inner_iterations: 10,
            history: vec![],
        };

        let k = result.k_estimated(20);
        assert!(k > 0.0 && k <= 1.0);
    }

    #[test]
    fn test_7q_optimization() {
        let config = TqqcConfig::default_7q()
            .with_noise(0.02)
            .with_points(3)
            .with_seed(42);

        let backend = SimulatorBackend::from_depol(7, 0.02).unwrap().with_seed(42);

        let mut engine = TqqcEngine::new(config, backend);
        let result = engine.optimize().unwrap();

        // Should complete
        assert!(result.iterations > 0);
    }
}
