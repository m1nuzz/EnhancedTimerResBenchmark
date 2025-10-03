//! Robust statistics implementation for timer resolution benchmarking
//!
//! This module provides robust statistical methods for accurate timer resolution measurements.

/// Robust statistics struct for reliable measurements
#[derive(Debug, Clone)]
pub struct RobustStatistics {
    pub mean: f64,
    pub median: f64,
    pub stdev: f64,
    pub mad: f64,              // Median Absolute Deviation - robust stdev
    pub p95: f64,              // 95th percentile
    pub p99: f64,              // 99th percentile
    pub outliers_removed: usize,
    pub confidence_interval_95: (f64, f64),
}

impl RobustStatistics {
    /// Create robust statistics from a vector of samples
    pub fn from_samples(samples: Vec<f64>) -> Self {
        if samples.is_empty() {
            panic!("Cannot compute statistics from empty samples");
        }
        let mut sorted = samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Median - robust central tendency
        let median = Self::percentile(&sorted, 50.0);

        // MAD (Median Absolute Deviation) - robust measure of spread
        let deviations: Vec<f64> = sorted.iter()
            .map(|&x| (x - median).abs())
            .collect();
        let mut dev_sorted = deviations.clone();
        dev_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mad = Self::percentile(&dev_sorted, 50.0);

        // Outlier removal using MAD method (more robust than Z-score)
        // Rule: |x - median| > k * MAD, where k = 3.5 (corresponds to ~3σ)
        let k = 3.5;
        let threshold = k * mad;
        let clean_samples: Vec<f64> = sorted.iter()
            .filter(|&&x| (x - median).abs() <= threshold)
            .copied()
            .collect();
        let outliers_removed = samples.len() - clean_samples.len();

        // Recalculate on cleaned data
        let clean_mean = clean_samples.iter().sum::<f64>() / clean_samples.len() as f64;
        let variance = clean_samples.iter()
            .map(|x| (x - clean_mean).powi(2))
            .sum::<f64>() / clean_samples.len() as f64;
        let stdev = variance.sqrt();

        // Percentiles - critical for understanding distribution
        let p95 = Self::percentile(&clean_samples, 95.0);
        let p99 = Self::percentile(&clean_samples, 99.0);

        // 95% confidence interval for mean
        let se = stdev / (clean_samples.len() as f64).sqrt();
        let ci_margin = 1.96 * se; // z-score for 95% CI
        let confidence_interval_95 = (clean_mean - ci_margin, clean_mean + ci_margin);

        Self {
            mean: clean_mean,
            median,
            stdev,
            mad,
            p95,
            p99,
            outliers_removed,
            confidence_interval_95,
        }
    }

    /// Calculate percentile of sorted data
    fn percentile(sorted_data: &[f64], p: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }
        let idx = (p / 100.0 * (sorted_data.len() - 1) as f64).round() as usize;
        sorted_data[idx.min(sorted_data.len() - 1)]
    }

    /// Composite performance score for optimization (uses p95 instead of mean!)
    pub fn performance_score(&self, weights: &PerformanceWeights) -> f64 {
        // P95 instead of mean - ignores worst 5% cases
        weights.accuracy * self.p95 + 
        // MAD instead of stdev - more robust consistency measure
        weights.consistency * self.mad +  
        // Consider worst case scenario
        weights.worst_case * self.p99     
    }
}

/// Performance weights for multi-criteria optimization
#[derive(Debug, Clone)]
pub struct PerformanceWeights {
    pub accuracy: f64,      // Weight for accuracy (p95)
    pub consistency: f64,   // Weight for consistency (MAD)
    pub worst_case: f64,    // Weight for worst case (p99)
}

impl Default for PerformanceWeights {
    fn default() -> Self {
        Self {
            accuracy: 0.60,    // 60% - main criterion
            consistency: 0.30, // 30% - stability importance
            worst_case: 0.10,  // 10% - protection from outliers
        }
    }
}