//! Bayesian optimization implementation for timer resolution tuning
//!
//! This module implements Bayesian optimization with Gaussian processes
//! for intelligent exploration of timer resolution parameter space.

use crate::stats::timer_measurement::TimerMeasurement;
use crate::stats::robust_statistics::PerformanceWeights;
use std::f64;

/// Bayesian optimizer for intelligent parameter search
pub struct BayesianOptimizer {
    pub observations: Vec<TimerMeasurement>,
    kernel_width: f64,
    weights: PerformanceWeights,
}

impl BayesianOptimizer {
    /// Create a new Bayesian optimizer with specified kernel width
    pub fn new(kernel_width: f64, weights: PerformanceWeights) -> Self {
        Self {
            observations: Vec::new(),
            kernel_width,
            weights,
        }
    }

    /// Add observation to the optimizer's knowledge base
    pub fn add_observation(&mut self, measurement: TimerMeasurement) {
        self.observations.push(measurement);
    }

    /// Gaussian process with robust metrics
    fn predict(&self, x: f64) -> (f64, f64) {
        if self.observations.is_empty() {
            return (1.0, 1.0);
        }
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        for obs in &self.observations {
            let dist_sq = (x - obs.resolution_ms).powi(2);
            let weight = (-dist_sq / (2.0 * self.kernel_width.powi(2))).exp();
            let score = obs.statistics.performance_score(&self.weights);
            weighted_sum += weight * score;
            weight_total += weight;
        }
        let mu = if weight_total > 1e-10 { weighted_sum / weight_total } else { 1.0 };
        // Uncertainty accounts for observation density
        let sigma = 0.3 / (1.0 + weight_total * 0.1);
        (mu, sigma)
    }

    /// Upper Confidence Bound (UCB) instead of Expected Improvement
    fn acquisition_ucb(&self, x: f64, kappa: f64) -> f64 {
        let (mu, sigma) = self.predict(x);
        mu - kappa * sigma  // Minus because we minimize score
    }

    /// Suggest next point to evaluate using acquisition function
    pub fn suggest_next(&self, bounds: (f64, f64), n_samples: usize, kappa: f64) -> f64 {
        let (low, high) = bounds;
        let step = (high - low) / (n_samples as f64);
        let mut best_x = low;
        let mut best_ucb = f64::MAX; // Seek minimum UCB
        for i in 0..n_samples {
            let x = low + (i as f64) * step;
            let ucb = self.acquisition_ucb(x, kappa);
            if ucb < best_ucb {  // Smaller is better
                best_ucb = ucb;
                best_x = x;
            }
        }
        best_x
    }


}