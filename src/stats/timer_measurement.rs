//! Timer measurement implementation for timer resolution benchmarking
//!
//! This module handles timer measurements with robust statistical analysis.

use crate::stats::robust_statistics::RobustStatistics;

/// Timer measurement with all statistical data
#[derive(Debug, Clone)]
pub struct TimerMeasurement {
    pub resolution_ms: f64,
    pub statistics: RobustStatistics,
    pub raw_samples: Vec<f64>,
}