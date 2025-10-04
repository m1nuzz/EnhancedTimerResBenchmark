//! Localization module for multilingual support
//! 
//! This module provides internationalization support for all UI elements
//! in the timer resolution benchmark tool.

use crate::ui::language::Language;
pub use crate::ui::localization_key::LocalizationKey;

/// Localization system for multilingual support
pub struct Localization {
    pub language: Language,
}

impl Localization {
    /// Create a new localization instance for the specified language
    pub fn new(language: Language) -> Self {
        Self { language }
    }
    
    /// Get localized string for a given key
    pub fn get(&self, key: LocalizationKey) -> &'static str {
        match self.language {
            Language::English => key.get_english(),
            Language::Ukrainian => key.get_ukrainian(),
            Language::Russian => key.get_russian(),
            Language::Chinese => key.get_chinese(),
        }
    }

    pub fn get_working_dir(&self, path: &str) -> String {
        self.get(LocalizationKey::WorkingDir).replace("{}", path)
    }

    pub fn get_windows_version(&self, info: &str) -> String {
        self.get(LocalizationKey::WindowsVersion).replace("{}", info)
    }

    pub fn get_cpu(&self, cpu: &str) -> String {
        self.get(LocalizationKey::Cpu).replace("{}", cpu)
    }

    pub fn get_range(&self, low: f64, high: f64) -> String {
        self.get(LocalizationKey::Range)
            .replace("{:.4}", &format!("{:.4}", low))
            .replace("{:.4}", &format!("{:.4}", high))
    }
    
    pub fn get_current_best(&self, value: f64, score: f64) -> String {
        self.get(LocalizationKey::CurrentBest)
            .replace("{:.4}", &format!("{:.4}", value))
            .replace("{:.4}", &format!("{:.4}", score))
    }

    pub fn get_optimal_value(&self, value: f64) -> String {
        self.get(LocalizationKey::OptimalValue).replace("{:.4}", &format!("{:.4}", value))
    }

    pub fn get_optimal_recommendation(&self, resolution: i32) -> String {
        self.get(LocalizationKey::OptimalRecommendation).replace("{}", &resolution.to_string())
    }

    pub fn get_rank(&self, rank: usize) -> String {
        self.get(LocalizationKey::Rank).replace("{}", &rank.to_string())
    }

    pub fn get_iterations_with_kappa(&self, iteration: usize, max_iterations: usize, value: f64, kappa: f64) -> String {
        self.get(LocalizationKey::IterationsWithKappa)
            .replace("{}", &iteration.to_string())
            .replace("{}", &max_iterations.to_string())
            .replace("{:.4}", &format!("{:.4}", value))
            .replace("{:.2}", &format!("{:.2}", kappa))
    }

    pub fn get_phase1(&self, count: usize) -> String {
        self.get(LocalizationKey::Phase1).replace("{}", &count.to_string())
    }

    pub fn get_point_info(&self, current: usize, total: usize, resolution: f64) -> String {
        self.get(LocalizationKey::PointInfo)
            .replace("{}", &current.to_string())
            .replace("{}", &total.to_string())
            .replace("{:.4}", &format!("{:.4}", resolution))
    }

    pub fn get_measurement_with_runs(&self, resolution: f64, runs: usize, samples: i32) -> String {
        self.get(LocalizationKey::GetMeasurementWithRuns)
            .replace("{:.4}", &format!("{:.4}", resolution))
            .replace("{}", &runs.to_string())
            .replace("{}", &samples.to_string())
    }

    pub fn get_measurement_stats(&self, mean: f64, p95: f64, mad: f64, outliers: usize) -> String {
        self.get(LocalizationKey::GetMeasurementStats)
            .replace("{:.4}", &format!("{:.4}", mean))
            .replace("{:.4}", &format!("{:.4}", p95))
            .replace("{:.4}", &format!("{:.4}", mad))
            .replace("{}", &outliers.to_string())
    }

    pub fn get_measure_sleep_error(&self, error: &str) -> String {
        self.get(LocalizationKey::MeasureSleepError).replace("{}", error)
    }

    pub fn get_join_error(&self, error: &str) -> String {
        self.get(LocalizationKey::JoinError).replace("{}", error)
    }

    pub fn get_timeout_error(&self) -> &'static str {
        self.get(LocalizationKey::TimeoutError)
    }

    pub fn get_keep_current(&self) -> &'static str {
        self.get(LocalizationKey::KeepCurrent)
    }

    pub fn get_enter_new_value(&self) -> &'static str {
        self.get(LocalizationKey::EnterNewValue)
    }

    pub fn get_exit_prompt(&self) -> &'static str {
        self.get(LocalizationKey::GetExitPrompt)
    }

    // Add new formatting functions here
    pub fn get_hpet_status_cached(&self, status: &str) -> String {
        self.get(LocalizationKey::HpetStatusCached).replace("{}", status)
    }

    pub fn get_hpet_status(&self, status: &str) -> String {
        self.get(LocalizationKey::HpetStatus).replace("{}", status)
    }

    pub fn get_error_hpet_disable(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorHpetDisable).replace("{}", error)
    }
    
    pub fn get_error_save_parameters(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorSaveParameters).replace("{}", error)
    }

    pub fn get_error_configuration(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorConfiguration).replace("{}", error)
    }

    pub fn get_found(&self, path: &str) -> String {
        self.get(LocalizationKey::Found).replace("{}", path)
    }

    pub fn get_missing_deps(&self, deps: &str) -> String {
        self.get(LocalizationKey::MissingDeps).replace("{}", deps)
    }

    pub fn get_test_passed(&self, delta: f64, stdev: f64) -> String {
        self.get(LocalizationKey::TestPassed)
            .replace("{:.4}", &format!("{:.4}", delta))
            .replace("{:.4}", &format!("{:.4}", stdev))
    }

    pub fn get_critical_process_remaining(&self, remaining: usize) -> String {
        self.get(LocalizationKey::CriticalProcessRemaining).replace("{}", &remaining.to_string())
    }

    pub fn get_error_linear_search(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorLinearSearch).replace("{}", error)
    }

    pub fn get_error_optimization(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorOptimization).replace("{}", error)
    }

    pub fn get_warning_cleanup(&self, error: &str) -> String {
        self.get(LocalizationKey::WarningCleanup).replace("{}", error)
    }

    pub fn get_kernel_width(&self, width: f64) -> String {
        self.get(LocalizationKey::KernelWidth).replace("{:.4}", &format!("{:.4}", width))
    }

    pub fn get_initial_points(&self, points: &str) -> String {
        self.get(LocalizationKey::InitialPoints).replace("{:?}", points)
    }

    pub fn get_init_point_message(&self, point: f64) -> String {
        self.get(LocalizationKey::InitPointMessage).replace("{:.4}", &format!("{:.4}", point))
    }

    pub fn get_unique_points(&self, unique: usize, total: usize) -> String {
        self.get(LocalizationKey::UniquePoints)
            .replace("{}", &unique.to_string())
            .replace("{}", &total.to_string())
    }

    pub fn get_topsis_score(&self, score: f64) -> String {
        self.get(LocalizationKey::TopsisScore).replace("{:.4}", &format!("{:.4}", score))
    }

    pub fn get_p95_delta(&self, delta: f64) -> String {
        self.get(LocalizationKey::P95Delta).replace("{:.4}", &format!("{:.4}", delta))
    }

    pub fn get_mad(&self, mad: f64) -> String {
        self.get(LocalizationKey::Mad).replace("{:.4}", &format!("{:.4}", mad))
    }

    pub fn get_p99_delta(&self, delta: f64) -> String {
        self.get(LocalizationKey::P99Delta).replace("{:.4}", &format!("{:.4}", delta))
    }

    pub fn get_ci_width(&self, width: f64) -> String {
        self.get(LocalizationKey::CiWidth).replace("{:.4}", &format!("{:.4}", width))
    }

    pub fn get_mutex_error_message(&self, message: &str) -> String {
        self.get(LocalizationKey::MutexErrorMessage).replace("{}", message)
    }

    pub fn get_error_process_exited(&self, error: &str) -> String {
        self.get(LocalizationKey::ErrorProcessExited).replace("{}", error)
    }

    pub fn get_warning_cannot_check_process(&self, error: &str) -> String {
        self.get(LocalizationKey::WarningCannotCheckProcess).replace("{}", error)
    }

    pub fn get_critical_mismatch(&self, expected: f64, reported: f64) -> String {
        self.get(LocalizationKey::CriticalMismatch)
            .replace("{:.4}", &format!("{:.4}", expected))
            .replace("{:.4}", &format!("{:.4}", reported))
    }

    pub fn get_verified(&self, reported: f64) -> String {
        self.get(LocalizationKey::Verified).replace("{:.4}", &format!("{:.4}", reported))
    }

    pub fn get_output_preview(&self, preview: &str) -> String {
        self.get(LocalizationKey::OutputPreview).replace("{}", preview)
    }

    pub fn get_warning_kill_child(&self, error: &str) -> String {
        self.get(LocalizationKey::WarningKillChild).replace("{}", error)
    }

    pub fn get_points_checked(&self, count: usize) -> String {
        self.get(LocalizationKey::PointsChecked).replace("{}", &count.to_string())
    }

    pub fn get_unique(&self, count: usize) -> String {
        self.get(LocalizationKey::Unique).replace("{}", &count.to_string())
    }

    pub fn get_kill_warning_ps(&self, error: &str) -> String {
        self.get(LocalizationKey::KillWarningPS).replace("{}", error)
    }

    pub fn get_kill_error_ps(&self, error: &str) -> String {
        self.get(LocalizationKey::KillErrorPS).replace("{}", error)
    }

    pub fn get_kill_warning_taskkill(&self, error: &str) -> String {
        self.get(LocalizationKey::KillWarningTaskkill).replace("{}", error)
    }

    pub fn get_kill_warning_remaining(&self, remaining: usize) -> String {
        self.get(LocalizationKey::KillWarningRemaining).replace("{}", &remaining.to_string())
    }

    pub fn get_kill_error_remaining(&self, remaining: usize) -> String {
        self.get(LocalizationKey::KillErrorRemaining).replace("{}", &remaining.to_string())
    }

    pub fn get_linear_method_samples(&self, value: i32) -> String {
        self.get(LocalizationKey::LinearMethodSamples).replace("{}", &value.to_string())
    }

    pub fn get_iterations_linear(&self, iterations: i32) -> String {
        self.get(LocalizationKey::IterationsLinear).replace("{}", &iterations.to_string())
    }

    pub fn get_runs_per_point(&self, runs: usize) -> String {
        self.get(LocalizationKey::RunsPerPoint).replace("{}", &runs.to_string())
    }

    pub fn get_samples_per_run(&self, samples: i32) -> String {
        self.get(LocalizationKey::SamplesPerRun).replace("{}", &samples.to_string())
    }

    pub fn get_weights(&self, accuracy: f64, consistency: f64, worst_case: f64) -> String {
        self.get(LocalizationKey::Weights)
            .replace("{:.1}", &format!("{:.1}", accuracy))
            .replace("{:.1}", &format!("{:.1}", consistency))
            .replace("{:.1}", &format!("{:.1}", worst_case))
    }

    pub fn get_expected(&self, value: f64) -> String {
        self.get(LocalizationKey::Expected).replace("{:.4}", &format!("{:.4}", value))
    }

    pub fn get_reported(&self, value: f64) -> String {
        self.get(LocalizationKey::Reported).replace("{:.4}", &format!("{:.4}", value))
    }

    pub fn get_diff(&self, value: f64) -> String {
        self.get(LocalizationKey::Diff).replace("{:.4}", &format!("{:.4}", value))
    }
}

/// Language selection function that allows users to choose their preferred language
pub fn select_language() -> Language {
    use std::io::{self, Write};
    use crate::ui::language::Language;
    
    println!("\n🌍 Select Language / Виберіть мову / Выберите язык / 选择语言");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let languages = Language::all();
    for (i, lang) in languages.iter().enumerate() {
        println!("{}. {}", i + 1, lang.name());
    }
    
    print!("\nSelect language (1-{}): ", languages.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let choice = input.trim().parse::<usize>().unwrap_or(1);
    let index = choice.saturating_sub(1).min(languages.len() - 1);
    
    languages[index]
}