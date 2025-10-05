//! TOPSIS (Technique for Order Preference by Similarity to Ideal Solution)
//! implementation for multi-criteria decision making
//!
//! This module provides TOPSIS ranking for selecting optimal timer resolution values
//! based on multiple criteria.

use crate::stats::timer_measurement::TimerMeasurement;

/// TOPSIS score for ranking solutions
#[derive(Debug, Clone)]
pub struct TopsisScore {
    pub resolution_ms: f64,
    pub closeness_coefficient: f64,
    pub rank: usize,
    pub criteria_scores: CriteriaScores,
}

/// Individual criteria scores for TOPSIS analysis
#[derive(Debug, Clone)]
pub struct CriteriaScores {
    pub p95_delta: f64,      // Lower is better
    pub mad: f64,            // Lower is better
    pub p99_delta: f64,      // Lower is better
    pub confidence_width: f64, // Lower is better (narrow CI = more reliable)
}

/// Perform TOPSIS ranking on measurements
pub fn topsis_ranking(measurements: &[TimerMeasurement]) -> Vec<TopsisScore> {
    if measurements.is_empty() {
        return Vec::new();
    }
    
    // Step 1: Build decision matrix
    let n = measurements.len();
    let mut matrix: Vec<Vec<f64>> = Vec::new();
    for m in measurements {
        let ci_width = m.statistics.confidence_interval_95.1 - m.statistics.confidence_interval_95.0;
        matrix.push(vec![
            m.statistics.p95,
            m.statistics.mad,
            m.statistics.p99,
            ci_width,
        ]);
    }

    // Step 2: Normalization (vector normalization) ✅ С ЗАЩИТОЙ!
    let num_criteria = 4;
    let mut normalized: Vec<Vec<f64>> = vec![vec![0.0; num_criteria]; n];
    for j in 0..num_criteria {
        let sum_sq: f64 = matrix.iter().map(|row| row[j].powi(2)).sum();
        let norm = sum_sq.sqrt();
        
        // ✅ ЗАЩИТА ОТ ДЕЛЕНИЯ НА 0
        if norm < 1e-10 {
            // Если все значения ≈ 0, используем равномерное распределение
            for i in 0..n {
                normalized[i][j] = 1.0 / (n as f64).sqrt();
            }
        } else {
            for i in 0..n {
                normalized[i][j] = matrix[i][j] / norm;
            }
        }
    }

    // Step 3: Weighted normalized matrix
    let weights = vec![0.40, 0.30, 0.20, 0.10]; // Criteria weights
    let mut weighted: Vec<Vec<f64>> = vec![vec![0.0; num_criteria]; n];
    for i in 0..n {
        for j in 0..num_criteria {
            weighted[i][j] = normalized[i][j] * weights[j];
        }
    }

    // Step 4: Ideal and anti-ideal solutions
    // All criteria are "lower is better" (cost criteria)
    let mut ideal = vec![f64::MAX; num_criteria];
    let mut anti_ideal = vec![f64::MIN; num_criteria];
    for j in 0..num_criteria {
        for i in 0..n {
            ideal[j] = ideal[j].min(weighted[i][j]);
            anti_ideal[j] = anti_ideal[j].max(weighted[i][j]);
        }
    }

    // Step 5: Distances to ideal and anti-ideal solutions
    let mut distances_ideal = vec![0.0; n];
    let mut distances_anti = vec![0.0; n];
    for i in 0..n {
        let mut sum_ideal = 0.0;
        let mut sum_anti = 0.0;
        for j in 0..num_criteria {
            sum_ideal += (weighted[i][j] - ideal[j]).powi(2);
            sum_anti += (weighted[i][j] - anti_ideal[j]).powi(2);
        }
        distances_ideal[i] = sum_ideal.sqrt();
        distances_anti[i] = sum_anti.sqrt();
    }

    // Step 6: Closeness coefficients (proximity to ideal)
    let mut scores: Vec<TopsisScore> = Vec::new();
    for (i, m) in measurements.iter().enumerate() {
        let ci_width = m.statistics.confidence_interval_95.1 - m.statistics.confidence_interval_95.0;
        
        // ✅ ЗАЩИТА ОТ ДЕЛЕНИЯ НА 0
        let denominator = distances_ideal[i] + distances_anti[i];
        let cc = if denominator.abs() < 1e-10 {
            0.5  // Нейтральный score если обе дистанции близки к 0
        } else {
            distances_anti[i] / denominator
        };
        
        let final_cc = if cc.is_nan() || cc.is_infinite() {
            0.5  // Return neutral value as fallback
        } else {
            cc
        };
        
        scores.push(TopsisScore {
            resolution_ms: m.resolution_ms,
            closeness_coefficient: final_cc,
            rank: 0, // Will be filled after sorting
            criteria_scores: CriteriaScores {
                p95_delta: m.statistics.p95,
                mad: m.statistics.mad,
                p99_delta: m.statistics.p99,
                confidence_width: ci_width,
            },
        });
    }

    // Step 7: Ranking (higher CC = better)
    // ✅ ЗАЩИТА ОТ NaN В СОРТИРОВКЕ
    scores.sort_by(|a, b| {
        b.closeness_coefficient.partial_cmp(&a.closeness_coefficient).unwrap_or(std::cmp::Ordering::Equal)  // Fallback если NaN
    });
    for (rank, score) in scores.iter_mut().enumerate() {
        score.rank = rank + 1;
    }
    scores
}