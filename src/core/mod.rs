use std::io::{self, Error, ErrorKind, Write, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fs};
use tokio::time::{sleep, timeout};
use serde::{Deserialize, Serialize};
use serde_json;
use os_info;
use raw_cpuid;
use std::mem;
use std::ptr;
use std::mem::size_of;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Mutex;

use crate::stats::robust_statistics::{RobustStatistics, PerformanceWeights};
use crate::stats::timer_measurement::TimerMeasurement;
use crate::optimization::bayesian_optimizer::BayesianOptimizer;
use crate::optimization::topsis::{topsis_ranking, TopsisScore};
use crate::ui::localization::{Localization, LocalizationKey, select_language};

// ============================================================================ 
// CONFIGURATION STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
struct BenchmarkingParameters {
    #[serde(rename = "StartValue", deserialize_with = "validate_positive_f64")]
    start_value: f64,
    #[serde(rename = "IncrementValue", deserialize_with = "validate_positive_f64")]
    increment_value: f64,
    #[serde(rename = "EndValue", deserialize_with = "validate_positive_f64")]
    end_value: f64,
    #[serde(rename = "SampleValue", deserialize_with = "validate_positive_i32")]
    sample_value: i32,
}

fn validate_positive_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("Value must be positive"))
    }
}

fn validate_positive_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    if value > 0 {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("Value must be positive"))
    }
}

// ============================================================================ 
// ADMIN PRIVILEGES CHECK
// ============================================================================

static IS_ADMIN: AtomicBool = AtomicBool::new(false);
static INIT: Once = Once::new();

fn is_admin() -> bool {
    INIT.call_once(|| {
        unsafe {
            let mut token: HANDLE = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0 {
                let mut elevation: TOKEN_ELEVATION = mem::zeroed();
                let mut size = size_of::<TOKEN_ELEVATION>() as u32;

                if GetTokenInformation(
                    token,
                    TokenElevation,
                    &mut elevation as *mut _ as *mut std::ffi::c_void,
                    size,
                    &mut size,
                ) != 0 && elevation.TokenIsElevated != 0
                {
                    IS_ADMIN.store(true, Ordering::Relaxed);
                }
                windows_sys::Win32::Foundation::CloseHandle(token);
            }
        }
    });

    IS_ADMIN.load(Ordering::Relaxed)
}

// ============================================================================ 
// SYSTEM CONFIGURATION
// ============================================================================

lazy_static::lazy_static! {
    static ref HPET_STATUS: Mutex<Option<String>> = Mutex::new(None);
}

fn check_hpet_status(localization: &Localization) -> io::Result<()> {
    let mut status = HPET_STATUS.lock().unwrap();

    if let Some(ref cached_status) = *status {
        println!("{}", localization.get_hpet_status_cached(cached_status));
        return Ok(());
    }

    let output = Command::new("bcdedit")
        .arg("/enum")
        .arg("{current}")
        .output()?;

    if !output.status.success() {
        eprintln!("{}", localization.get(LocalizationKey::ErrorHpetStatus));
        return Err(Error::new(ErrorKind::Other, localization.get(LocalizationKey::ErrorHpetStatus)));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut useplatformclock_value: Option<String> = None;
    let mut disabledynamictick_value: Option<String> = None;

    for line in output_str.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            match key.to_lowercase().as_str() {
                "useplatformclock" => useplatformclock_value = Some(value.to_lowercase()),
                "disabledynamictick" => disabledynamictick_value = Some(value.to_lowercase()),
                _ => {}
            }
        }
    }

    let hpet_status = match (
        useplatformclock_value.as_deref(),
        disabledynamictick_value.as_deref(),
    ) {
        (Some("no"), Some("yes")) | (None, Some("yes")) | (None, None) => "disabled",
        _ => "enabled",
    };

    println!("{}", localization.get_hpet_status(hpet_status));

    if hpet_status == "enabled" {
        println!("{}", localization.get(LocalizationKey::HpetEnabledWarning));
        println!("{}", localization.get(LocalizationKey::HpetTroubleshooting));
        print!("{}", localization.get(LocalizationKey::HpetDisablePrompt));
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("y") {
            if let Err(e) = disable_hpet(localization) {
                eprintln!("{}", localization.get_error_hpet_disable(&e.to_string()));
                return Err(e.into());
            }
            println!("{}", localization.get(LocalizationKey::HpetDisabledSuccess));
        }
    }

    *status = Some(hpet_status.to_string());
    Ok(())
}

fn disable_hpet(localization: &Localization) -> io::Result<()> {
    if let Err(e) = apply_registry_tweak(localization) {
        eprintln!("{}", localization.get(LocalizationKey::ErrorRegistryTweak));
        return Err(e.into());
    }

    let commands = vec![
        ("bcdedit", vec!["/deletevalue", "useplatformclock"]),
        ("bcdedit", vec!["/set", "disabledynamictick", "yes"]),
    ];

    for (command, args) in commands {
        let output = Command::new(command).args(&args).output()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, localization.get_error_hpet_disable(&e.to_string())))?;
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{} {}", localization.get(LocalizationKey::ErrorHpetDisable), output.status),
            ));
        }
    }

    Ok(())
}

fn apply_registry_tweak(localization: &Localization) -> io::Result<()> {
    let output = Command::new("reg")
        .args(&[
            "add",
            r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\kernel",
            "/v",
            "GlobalTimerResolutionRequests",
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ])
        .output()?;

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            localization.get(LocalizationKey::ErrorRegistryTweak),
        ));
    }

    Ok(())
}

// ============================================================================ 
// UTILITY FUNCTIONS
// ============================================================================

/// Parse MeasureSleep.exe output including resolution verification
/// Example input: "Resolution: 0.5186ms, Sleep(1) slept 1.0310ms (delta: 0.0310)"
/// Returns: (delta_ms, stdev_ms, actual_resolution_ms)
fn parse_measurement_output_with_resolution(output: &[u8]) -> io::Result<(f64, f64, Option<f64>)> {
    let output_str = std::str::from_utf8(output).map_err(|e| Error::new(ErrorKind::InvalidData, format!("UTF-8 decode error: {}", e)))?;

    let mut avg = None;
    let mut stdev = None;
    let mut resolution_ms = None;

    // Parse line by line
    for line in output_str.lines() {
        let trimmed = line.trim();
        
        // Parse line: "Resolution: 0.5186ms, Sleep(1) slept 1.0310ms (delta: 0.0310)"
        if resolution_ms.is_none() && trimmed.contains("Resolution: ") {
            // Extract "0.5186" from "Resolution: 0.5186ms"
            if let Some(res_part) = trimmed.split("Resolution: ").nth(1) {
                // Take everything before first "ms"
                if let Some(res_str) = res_part.split("ms").next() {
                    resolution_ms = res_str.trim().parse::<f64>().ok();
                }
            }
        }
        
        // Parse line: "Avg: 0.1439"
        if avg.is_none() && trimmed.starts_with("Avg: ") {
            avg = trimmed[5..].trim().parse().ok();
        }
        
        // Parse line: "STDEV: 0.0029"
        if stdev.is_none() && trimmed.starts_with("STDEV: ") {
            stdev = trimmed[7..].trim().parse().ok();
        }
        
        // Optimization: exit if everything is found
        if avg.is_some() && stdev.is_some() && resolution_ms.is_some() {
            break;
        }
    }

    match (avg, stdev) {
        (Some(a), Some(s)) => Ok((a, s, resolution_ms)),
        _ => {
            eprintln!("Failed to parse MeasureSleep output:");
            eprintln!("{}", output_str);
            Err(Error::new(ErrorKind::InvalidData,"Invalid MeasureSleep output format"))
        }
    }
}

// Return the old function for compatibility with the rest of the code
fn parse_measurement_output(output: &[u8]) -> io::Result<(f64, f64)> {
    let (avg, stdev, _) = parse_measurement_output_with_resolution(output)?;
    Ok((avg, stdev))
}

fn cleanup_processes() -> io::Result<()> {
    // Placeholder for actual process cleanup implementation
    Ok(())
}

// ============================================================================ 
// OPTIMIZATION FUNCTIONS
// ============================================================================

pub struct OptimizationResult {
    pub optimal_resolution: f64,
    pub topsis_score: f64,
    aggregated_measurements: Vec<TimerMeasurement>,
    topsis_rankings: Vec<TopsisScore>,
}

pub async fn run_benchmark() -> io::Result<()> {
    use colored::*;

    let selected_language = select_language();
    let localization = Localization::new(selected_language);
    
    let separator = "=".repeat(60);
    
    println!("\n{}", separator);
    println!("{:^60}", localization.get(LocalizationKey::Title).bold().cyan());
    println!("{}\n", separator);

    if !is_admin() {
        eprintln!("{}", localization.get(LocalizationKey::ErrorAdminPrivileges).bold().red());
        eprintln!("{}", localization.get(LocalizationKey::RunAsAdmin).bold().red());
        return Err(Error::new(ErrorKind::PermissionDenied, "Administrator privileges required"));
    }

    println!("{}", localization.get(LocalizationKey::SystemInfo).bold().yellow());
    println!("━━━━━━━━━━━━━━━━━━━");
    println!("{}", localization.get_working_dir(&env::current_dir()?.display().to_string()));
    println!("{}", localization.get(LocalizationKey::AdminPrivileges).bold().green());

    let os_info = os_info::get();
    if let os_info::Type::Windows = os_info.os_type() {
        if let Some(build_number) = os_info.version().to_string().split('.').nth(2).and_then(|s| s.parse::<u32>().ok()) {
            let version = if build_number >= 22000 { "Windows 11" } else { "Windows 10" };
            println!("{}", localization.get_windows_version(&format!("{} (Build {})", version, build_number)));
        } else {
            println!("{}", localization.get_windows_version(localization.get(LocalizationKey::UnknownBuild)));
        }
    }

    let cpuid = raw_cpuid::CpuId::new();
    if let Some(brand) = cpuid.get_processor_brand_string() {
        println!("{}", localization.get_cpu(brand.as_str().trim()));
    } else {
        println!("{}", localization.get_cpu(localization.get(LocalizationKey::UnknownCpu)));
    }
    println!();

    println!("{}", localization.get(LocalizationKey::SystemConfig).bold().yellow());
    println!("━━━━━━━━━━━━━━━━━━━━");
    check_hpet_status(&localization)?;
    println!();

    println!("{}", localization.get(LocalizationKey::OptimizationMethod).bold().yellow());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("{}", localization.get(LocalizationKey::AvailableMethods));
    println!();
    println!("  {}  {}", "1.".bold().cyan(), localization.get(LocalizationKey::LinearMethod).bold());
    println!("     • {}", localization.get(LocalizationKey::LinearMethodDesc1));
    println!("     • {}", localization.get(LocalizationKey::LinearMethodDesc2));
    println!("     • {}", localization.get(LocalizationKey::LinearMethodDesc3));
    println!("     • {}", localization.get(LocalizationKey::LinearMethodDesc4));
    println!();
    println!("  {}  {}", "2.".bold().cyan(), localization.get(LocalizationKey::HybridMethod).bold().green());
    println!("     • {}", localization.get(LocalizationKey::HybridMethodDesc1));
    println!("     • {}", localization.get(LocalizationKey::HybridMethodDesc2));
    println!("     • {}", localization.get(LocalizationKey::HybridMethodDesc3));
    println!();
    let mut method_input = String::new();
    print!("{}", localization.get(LocalizationKey::MethodChoice));
    io::stdout().flush()?;
    io::stdin().read_line(&mut method_input)?;
    let optimization_method = method_input.trim();
    println!();

    let parameters = match fs::read_to_string("appsettings.json")
        .and_then(|content| serde_json::from_str::<BenchmarkingParameters>(&content)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e)))
    {
        Ok(mut params) => {
            let mut input = String::new();
            let mut prompt = |desc: &str, current: &str| -> io::Result<Option<String>> {
                println!("▸ {}: {}{}", desc, current, localization.get(LocalizationKey::KeepCurrent));
                println!("{}", localization.get(LocalizationKey::EnterNewValue));
                input.clear();
                io::stdin().read_line(&mut input)?;
                let trimmed = input.trim();
                Ok(if trimmed.is_empty() { None } else { Some(trimmed.to_string()) })
            };

            println!("{}", localization.get(LocalizationKey::BenchmarkParams));
            println!("━━━━━━━━━━━━━━━━━━━");

            if let Some(new_value) = prompt(&localization.get(LocalizationKey::StartValue), &format!("{:.4} ms", params.start_value))? {
                params.start_value = new_value.parse().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            }
            if optimization_method == "1" {
                if let Some(new_value) = prompt(&localization.get(LocalizationKey::IncrementValue), &format!("{:.4} ms", params.increment_value))? {
                    params.increment_value = new_value.parse().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
                }
            } else {
                println!("▸ {}: {:.4} ms {}", localization.get(LocalizationKey::IncrementValue), params.increment_value, localization.get(LocalizationKey::IncrementNotUsed));
            }
            if let Some(new_value) = prompt(&localization.get(LocalizationKey::EndValue), &format!("{:.4} ms", params.end_value))? {
                params.end_value = new_value.parse().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            }
            if let Some(new_value) = prompt(&localization.get(LocalizationKey::SampleValue), &params.sample_value.to_string())? {
                params.sample_value = new_value.parse().map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
            }

            match optimization_method {
                "1" => {
                    let iterations = ((params.end_value - params.start_value) / params.increment_value).ceil();
                    println!("▸ {}\n", localization.get_iterations_linear(iterations as i32));
                },
                _ => {
                    println!("▸ {}\n", localization.get(LocalizationKey::IterationsHybrid));
                }
            }

            if let Err(e) = fs::write("appsettings.json", serde_json::to_string_pretty(&params)?) {
                eprintln!("{}", localization.get_error_save_parameters(&e.to_string()));
            }

            params
        },
        Err(e) => {
            eprintln!("{}", localization.get_error_configuration(&e.to_string()));
            return Err(e);
        }
    };

    let exe_dir = env::current_exe()?.parent()
        .ok_or_else(|| {
            eprintln!("{}", localization.get(LocalizationKey::ErrorGetExePath));
            Error::new(ErrorKind::Other, localization.get(LocalizationKey::ErrorGetExePath))
        })?
        .to_path_buf();

    let set_timer_resolution_path = exe_dir.join("SetTimerResolution.exe");
    let measure_sleep_path = exe_dir.join("MeasureSleep.exe");

    println!("\n{}", localization.get(LocalizationKey::Dependencies));
    println!("━━━━━━━━━━━━━━━━━━━━━");

    let dependencies = [
        ("SetTimerResolution.exe", &set_timer_resolution_path),
        ("MeasureSleep.exe", &measure_sleep_path),
    ];

    let missing_dependencies: Vec<_> = dependencies.iter()
        .filter_map(|(name, path)| {
            if path.exists() {
                println!("{}", localization.get_found(&path.file_name().unwrap_or_default().to_string_lossy()));
                None
            } else {
                Some(*name)
            }
        })
        .collect();

    if !missing_dependencies.is_empty() {
        eprintln!("{}", localization.get_missing_deps(&missing_dependencies.join(", ")));
        return Err(Error::new(ErrorKind::NotFound, "Missing dependencies"));
    }
    println!();

    println!("{}", localization.get(LocalizationKey::MeasureSleepTest));
    let test_output = Command::new(&measure_sleep_path)
        .arg("--samples")
        .arg("5")
        .output()?;
    if !test_output.status.success() {
        eprintln!("{}", localization.get(LocalizationKey::ErrorMeasureSleep));
        eprintln!("{}", String::from_utf8_lossy(&test_output.stderr));
        return Err(Error::new(ErrorKind::Other, "MeasureSleep.exe failed"));
    }
    let (test_delta, test_stdev) = parse_measurement_output(&test_output.stdout)?;
    println!("{}", localization.get_test_passed(test_delta, test_stdev));

    println!("{}", localization.get(LocalizationKey::CleaningUp));
    force_kill_all_timer_processes()?;
    sleep(Duration::from_millis(1000)).await;
    
    let remaining = count_timer_processes();
    if remaining > 0 {
        eprintln!("{}", localization.get_critical_process_remaining(remaining));
        eprintln!("{}", localization.get(LocalizationKey::ManualCleanupInstructions));
        eprintln!("{}", localization.get(LocalizationKey::ManualCleanup1));
        eprintln!("{}", localization.get(LocalizationKey::ManualCleanup2));
        eprintln!("{}", localization.get(LocalizationKey::ManualCleanup3));
        eprintln!("{}", localization.get(LocalizationKey::ManualCleanup4));
        return Err(Error::new(ErrorKind::Other, localization.get(LocalizationKey::ErrorCannotProceed)));
    }
    println!("{}", localization.get(LocalizationKey::CleanupCompleted));
    
    prompt_user(&localization.get(LocalizationKey::PressEnter))?;
    
    fn prompt_user(message: &str) -> io::Result<()> {
        println!("{}", message);
        io::stdin().read_line(&mut String::new())?;
        Ok(())
    }

    let result = match optimization_method {
        "1" => {
            match linear_exhaustive_search(
                &parameters,
                &set_timer_resolution_path,
                &measure_sleep_path,
                &localization,
            ).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}", localization.get_error_linear_search(&e.to_string()));
                    kill_all_timer_processes()?;
                    return Err(e);
                }
            }
        },
        "2" | "" => {
            match optimize_timer_resolution(
                &parameters,
                &set_timer_resolution_path,
                &measure_sleep_path,
                &localization,
            ).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}", localization.get_error_optimization(&e.to_string()));
                    kill_all_timer_processes()?;
                    return Err(e);
                }
            }
        },
        _ => {
            eprintln!("{}", localization.get(LocalizationKey::ErrorInvalidMethod));
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid method"));
        }
    };

    save_detailed_results(&result, "results.txt")?;
    println!("{}", localization.get(LocalizationKey::BenchmarkComplete));

    if let Err(e) = cleanup_processes() {
        eprintln!("{}", localization.get_warning_cleanup(&e.to_string()));
    }

    prompt_exit(&localization)?;
    
    fn prompt_exit(localization: &Localization) -> io::Result<()> {
        println!("{}", localization.get(LocalizationKey::GetExitPrompt));
        io::stdin().read_line(&mut String::new())?;
        Ok(())
    }

    Ok(())
}

async fn optimize_timer_resolution(
    params: &BenchmarkingParameters,
    set_timer_path: &PathBuf,
    measure_sleep_path: &PathBuf,
    localization: &Localization,
) -> io::Result<OptimizationResult> {
    let weights = PerformanceWeights::default();
    let bounds = (params.start_value, params.end_value);
    let max_iterations = 15;
    let samples_per_run = params.sample_value;
    let runs_per_measurement = 3;

    println!("\n{}", localization.get(LocalizationKey::RobustOptimization));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", localization.get(LocalizationKey::Parameters));
    println!("   {}", localization.get_range(bounds.0, bounds.1));
    println!("   {}", localization.get_iterations_linear(max_iterations));
        println!("   {}", localization.get(LocalizationKey::LinearMethodRuns));
        println!("   {}", localization.get_linear_method_samples(samples_per_run));
    println!("   {}", localization.get_weights(weights.accuracy * 100.0, weights.consistency * 100.0, weights.worst_case * 100.0));
    println!();

    let range = bounds.1 - bounds.0;
    let kernel_width = range * 0.15;
    println!("{}", localization.get_kernel_width(kernel_width));
    let mut optimizer = BayesianOptimizer::new(kernel_width, weights.clone());

    fn latin_hypercube_sampling(bounds: (f64, f64), n_points: usize) -> Vec<f64> {
        let (low, high) = bounds;
        let segment_size = (high - low) / n_points as f64;
        (0..n_points).map(|i| low + (i as f64 + 0.5) * segment_size).collect()
    }
    let initial_points = latin_hypercube_sampling(bounds, 5);
    println!("{}", localization.get_initial_points(&format!("{:?}", initial_points.iter().map(|&x| format!("{:.4}", x)).collect::<Vec<_>>())));
    println!("{}", localization.get_phase1(initial_points.len()));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let init_pb = ProgressBar::new(initial_points.len() as u64);
    init_pb.set_style(
        ProgressStyle::default_bar()
            .template(&localization.get(LocalizationKey::InitProgressBar))
            .unwrap()
            .progress_chars("##-")
    );
    
    for (i, &x) in initial_points.iter().enumerate() {
        init_pb.set_message(localization.get_init_point_message(x));
        println!("{}", localization.get_point_info(i + 1, initial_points.len(), x));
        let measurement = measure_resolution_robust(
            x,
            samples_per_run,
            runs_per_measurement,
            set_timer_path,
            measure_sleep_path,
            localization,
        ).await?;
        optimizer.add_observation(measurement);
        init_pb.inc(1);
    }
    init_pb.finish_with_message(localization.get(LocalizationKey::InitCompleted));

    println!("\n{}", localization.get(LocalizationKey::Phase2));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let total_iterations = max_iterations as usize - initial_points.len();
    let opt_pb = ProgressBar::new(total_iterations as u64);
    opt_pb.set_style(
        ProgressStyle::default_bar()
            .template(&localization.get(LocalizationKey::OptProgressBar))
            .unwrap()
            .progress_chars("##-")
    );
    
    for iter in initial_points.len()..max_iterations as usize {
        let kappa = 2.5 - (2.0 * (iter - initial_points.len()) as f64 / (max_iterations as usize - initial_points.len()) as f64);
        let next_x = optimizer.suggest_next(bounds, 200, kappa);
        println!("  {}", localization.get_iterations_with_kappa(iter + 1, max_iterations as usize, next_x, kappa));
        let measurement = measure_resolution_robust(
            next_x,
            samples_per_run,
            runs_per_measurement,
            set_timer_path,
            measure_sleep_path,
            localization,
        ).await?;
        optimizer.add_observation(measurement);
        
        let current_best = optimizer.observations.iter()
            .min_by(|a, b| {
                let score_a = a.statistics.performance_score(&weights);
                let score_b = b.statistics.performance_score(&weights);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .unwrap();
        println!("       {}", localization.get_current_best(current_best.resolution_ms, current_best.statistics.performance_score(&weights)));
        
        kill_all_timer_processes()?;
        sleep(Duration::from_millis(300)).await;
        opt_pb.inc(1);
    }
    opt_pb.finish_with_message(localization.get(LocalizationKey::OptCompleted));

    println!("\n{}", localization.get(LocalizationKey::Phase3));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let aggregated_measurements = aggregate_measurements(&optimizer.observations);
    println!("{}", localization.get_unique_points(aggregated_measurements.len(), optimizer.observations.len()));
    let topsis_results = topsis_ranking(&aggregated_measurements);

    println!("\n{}", localization.get(LocalizationKey::TopsisRanking));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    for (i, result) in topsis_results.iter().take(5).enumerate() {
        let marker = if i == 0 { "🥇" } else if i == 1 { "🥈" } else if i == 2 { "🥉" } else { "  " };
        println!("{}  {}: {:.4} ms", marker, localization.get_rank(result.rank), result.resolution_ms);
        println!("{}", localization.get_topsis_score(result.closeness_coefficient));
        println!("{}", localization.get_p95_delta(result.criteria_scores.p95_delta));
        println!("{}", localization.get_mad(result.criteria_scores.mad));
        println!("{}", localization.get_p99_delta(result.criteria_scores.p99_delta));
        println!("{}", localization.get_ci_width(result.criteria_scores.confidence_width));
        println!();
    }

    let best = &topsis_results[0];
    println!("{}", localization.get_optimal_value(best.resolution_ms));
    println!("   {}\n", localization.get_optimal_recommendation((best.resolution_ms * 10_000.0) as i32));

    Ok(OptimizationResult {
        optimal_resolution: best.resolution_ms,
        topsis_score: best.closeness_coefficient,
        aggregated_measurements,
        topsis_rankings: topsis_results,
    })
}

fn save_detailed_results(result: &OptimizationResult, filename: &str) -> io::Result<()> {
    use std::fs::File;
    use std::io::BufWriter;
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "# Timer Resolution Optimization Results")?;
    writeln!(writer, "# Generated: {:?}", std::time::SystemTime::now())?;
    writeln!(writer, "")?;
    writeln!(writer, "Resolution_ms,P50_Delta,P95_Delta,P99_Delta,Mean_Delta,StdDev,MAD,Outliers_Removed,CI_Lower,CI_Upper,TOPSIS_Score,Rank")?;
    for topsis in &result.topsis_rankings {
        // ИСПОЛЬЗУЕМ aggregated_measurements! и сравнение с tolerance для float
        let m = result.aggregated_measurements.iter()
            .find(|m| (m.resolution_ms - topsis.resolution_ms).abs() < 0.0001)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound, 
                    format!("Measurement not found for resolution {:.4} ms", topsis.resolution_ms)
                )
            })?;
        writeln!(
            writer,
            "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.4},{:.4},{}",
            m.resolution_ms,
            m.statistics.median,
            m.statistics.p95,
            m.statistics.p99,
            m.statistics.mean,
            m.statistics.stdev,
            m.statistics.mad,
            m.statistics.outliers_removed,
            m.statistics.confidence_interval_95.0,
            m.statistics.confidence_interval_95.1,
            topsis.closeness_coefficient,
            topsis.rank,
        )?;
    }
    writeln!(writer, "")?;
    writeln!(writer, "# Optimal Resolution: {:.4} ms", result.optimal_resolution)?;
    writeln!(writer, "# TOPSIS Score: {:.4}", result.topsis_score)?;
    Ok(())
}

// ============================================================================ 
// ROBUST TIMER RESOLUTION MEASUREMENT
// ============================================================================

async fn measure_resolution_robust(
    resolution_ms: f64,
    samples_per_run: i32,
    num_runs: usize,
    set_timer_path: &PathBuf,
    measure_sleep_path: &PathBuf,
    localization: &Localization,
) -> io::Result<TimerMeasurement> {
    kill_all_timer_processes()?;
    sleep(Duration::from_millis(300)).await;
    
    let mut all_deltas = Vec::new();
    println!("{}", localization.get_measurement_with_runs(resolution_ms, num_runs, samples_per_run));
    
    for run in 1..=num_runs {
        let resolution = (resolution_ms * 10_000.0) as i32;
        
        kill_all_timer_processes()?;
        sleep(Duration::from_millis(200)).await;
        
        let mut timer_child = Command::new(set_timer_path)
            .args(&["--resolution", &resolution.to_string(), "--no-console"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to spawn SetTimerResolution: {}", e)))?;

        sleep(Duration::from_millis(50)).await;
        
        match timer_child.try_wait() {
            Ok(Some(_exit_status)) => {
                let mut stderr_output = String::new();
                let mut stdout_output = String::new();
                if let Some(mut stderr) = timer_child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_output);
                }
                if let Some(mut stdout) = timer_child.stdout.take() {
                    let _ = stdout.read_to_string(&mut stdout_output);
                }
                let error_msg = format!("{}{}", stderr_output, stdout_output);
                if error_msg.contains("already running") || error_msg.contains("Another instance") {
                    eprintln!("{}", localization.get(LocalizationKey::ErrorMutexConflict));
                    eprintln!("{}", localization.get_mutex_error_message(&error_msg.trim()));
                    eprintln!("{}", localization.get(LocalizationKey::MutexErrorHint));
                    eprintln!("{}", localization.get(LocalizationKey::MutexErrorRestart));
                    kill_all_timer_processes()?;
                    return Err(Error::new(ErrorKind::AlreadyExists, localization.get(LocalizationKey::ErrorMutexRunning)));
                }
                return Err(Error::new(ErrorKind::Other, localization.get_error_process_exited(&error_msg)));
            },
            Ok(None) => {},
            Err(e) => {
                eprintln!("{}", localization.get_warning_cannot_check_process(&e.to_string()));
            }
        }

        sleep(Duration::from_millis(350)).await;
        
        let measure_path = measure_sleep_path.clone();
        let samples = samples_per_run;
        let output_result = timeout(
            Duration::from_secs(30),
            tokio::task::spawn_blocking(move || {
                Command::new(&measure_path)
                    .arg("--samples")
                    .arg(samples.to_string())
                    .output()
            })
        ).await;
        
        let output = match output_result {
            Ok(Ok(Ok(output))) => output,
            Ok(Ok(Err(e))) => {
                let _ = timer_child.kill();
                kill_all_timer_processes()?;
                eprintln!("{}", localization.get_measure_sleep_error(&e.to_string()));
                return Err(e);
            },
            Ok(Err(e)) => {
                let _ = timer_child.kill();
                kill_all_timer_processes()?;
                eprintln!("{}", localization.get_join_error(&e.to_string()));
                return Err(Error::new(ErrorKind::Other, e));
            },
            Err(_) => {
                let _ = timer_child.kill();
                kill_all_timer_processes()?;
                eprintln!("{}", localization.get(LocalizationKey::TimeoutError));
                return Err(Error::new(ErrorKind::TimedOut, "MeasureSleep timeout"));
            }
        };
        
        if !output.status.success() {
            let _ = timer_child.kill();
            kill_all_timer_processes()?;
            eprintln!("{}", localization.get(LocalizationKey::ErrorMeasureSleepFailed));
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(Error::new(ErrorKind::Other, "MeasureSleep execution failed"));
        }
        
        let (delta, _stdev, measure_reported_res) = parse_measurement_output_with_resolution(&output.stdout)?;
        
        if let Some(reported) = measure_reported_res {
            let tolerance = 0.05;
            let diff = (reported - resolution_ms).abs();
            if diff > tolerance {
                eprintln!("{}", localization.get(LocalizationKey::WarningResolutionMismatch));
                eprintln!("{}", localization.get_expected(resolution_ms));
                eprintln!("{}", localization.get_reported(reported));
                eprintln!("{}", localization.get_diff(diff));
                
                if diff > 0.1 {
                    let _ = timer_child.kill();
                    kill_all_timer_processes()?;
                    return Err(Error::new(ErrorKind::Other,
                        localization.get_critical_mismatch(resolution_ms, reported)));
                }
            } else {
                println!("{}", localization.get_verified(reported));
            }
        } else {
            eprintln!("{}", localization.get(LocalizationKey::WarningParseResolution));
            eprintln!("{}",
                localization.get_output_preview(
                    &String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or(localization.get(LocalizationKey::Empty))
                )
            );
        }
        
        all_deltas.push(delta);
        print!(".");
        io::stdout().flush()?;
        
        if let Err(e) = timer_child.kill() {
            eprintln!("{}", localization.get_warning_kill_child(&e.to_string()));
        }
        
        kill_all_timer_processes()?;
        
        if run < num_runs {
            sleep(Duration::from_millis(600)).await;
        }
    }
    println!(" ✓");
    
    let statistics = RobustStatistics::from_samples(all_deltas.clone());
    
    println!("{}", 
        localization.get_measurement_stats(
            statistics.mean, 
            statistics.p95, 
            statistics.mad, 
            statistics.outliers_removed
        ));
    
    Ok(TimerMeasurement {
        resolution_ms,
        statistics,
        raw_samples: all_deltas,
    })
}



fn aggregate_measurements(measurements: &[TimerMeasurement]) -> Vec<TimerMeasurement> {
    use std::collections::HashMap;
    let mut groups: HashMap<i64, Vec<&TimerMeasurement>> = HashMap::new();
    for m in measurements {
        let key = (m.resolution_ms * 10000.0).round() as i64;
        groups.entry(key).or_insert_with(Vec::new).push(m);
    }
    groups.into_iter().map(|(key, group)| {
        let resolution_ms = key as f64 / 10000.0;
        let mut all_samples = Vec::new();
        for m in &group {
            all_samples.extend(m.raw_samples.iter().copied());
        }
        let combined_stats = RobustStatistics::from_samples(all_samples.clone());
        TimerMeasurement {
            resolution_ms,
            statistics: combined_stats,
            raw_samples: all_samples,
        }
    }).collect()
}

// ============================================================================ 
// LINEAR EXHAUSTIVE SEARCH
// ============================================================================

async fn linear_exhaustive_search(
    params: &BenchmarkingParameters,
    set_timer_path: &PathBuf,
    measure_sleep_path: &PathBuf,
    localization: &Localization,
) -> io::Result<OptimizationResult> {
    println!("\n{}", localization.get(LocalizationKey::LinearMethodTitle));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // ✅ ИСПРАВЛЕНИЕ: Используем целочисленный счётчик вместо float
    let total_points = ((params.end_value - params.start_value) / params.increment_value).ceil() as usize;

    // Validation: prevent infinite loop
    if total_points == 0 {
        return Err(Error::new(ErrorKind::InvalidInput,
            format!("Invalid parameters: start={:.4}, end={:.4}, increment={:.4} results in 0 points",
                params.start_value, params.end_value, params.increment_value)));
    }

    if total_points > 100_000 {
        eprintln!("⚠️  WARNING: {} points will be tested!", total_points);
        eprintln!("   This will take approximately {:.1} hours", (total_points as f64 * 6.5) / 3600.0);
        eprintln!("   Press Ctrl+C to abort, or Enter to continue...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    }

    println!("{}", localization.get(LocalizationKey::LinearMethodParameters));
    
    // Format messages based on language
    let range_message = match localization.language {
        crate::ui::language::Language::English => format!("   Range: [{:.4}, {:.4}] ms", params.start_value, params.end_value),
        crate::ui::language::Language::Russian => format!("   Диапазон: [{:.4}, {:.4}] ms", params.start_value, params.end_value),
        crate::ui::language::Language::Ukrainian => format!("   Діапазон: [{:.4}, {:.4}] ms", params.start_value, params.end_value),
        crate::ui::language::Language::Chinese => format!("   范围: [{:.4}, {:.4}] ms", params.start_value, params.end_value),
    };
    println!("{}", range_message);

    let step_message = match localization.language {
        crate::ui::language::Language::English => format!("   Step: {:.4} ms", params.increment_value),
        crate::ui::language::Language::Russian => format!("   Шаг: {:.4} ms", params.increment_value),
        crate::ui::language::Language::Ukrainian => format!("   Крок: {:.4} ms", params.increment_value),
        crate::ui::language::Language::Chinese => format!("   步长: {:.4} ms", params.increment_value),
    };
    println!("{}", step_message);

    let points_message = match localization.language {
        crate::ui::language::Language::English => format!("   Points to check: {}", total_points),
        crate::ui::language::Language::Russian => format!("   Точек для проверки: {}", total_points),
        crate::ui::language::Language::Ukrainian => format!("   Точок для перевірки: {}", total_points),
        crate::ui::language::Language::Chinese => format!("   待检查点数: {}", total_points),
    };
    println!("{}", points_message);

    let runs_message = match localization.language {
        crate::ui::language::Language::English => "   Runs per point: 3",
        crate::ui::language::Language::Russian => "   Прогонов на точку: 3",
        crate::ui::language::Language::Ukrainian => "   Прогонів на точку: 3",
        crate::ui::language::Language::Chinese => "   每点运行次数: 3",
    };
    println!("{}", runs_message);

    let samples_message = match localization.language {
        crate::ui::language::Language::English => format!("   Samples per run: {}", params.sample_value),
        crate::ui::language::Language::Russian => format!("   Выборок на прогон: {}", params.sample_value),
        crate::ui::language::Language::Ukrainian => format!("   Вибірок на прогін: {}", params.sample_value),
        crate::ui::language::Language::Chinese => format!("   每次运行样本数: {}", params.sample_value),
    };
    println!("{}", samples_message);

    println!();

    // ✅ FIX 3: Remove misleading estimate, show realistic warning
    let note_message = match localization.language {
        crate::ui::language::Language::English => format!("📝 Note: {} points will be tested. Real-time ETA will be shown after first {} measurements.", total_points, 5),
        crate::ui::language::Language::Russian => format!("📝 Примечание: Будет протестировано {} точек. Real-time ETA покажется после {} измерений.", total_points, 5),
        crate::ui::language::Language::Ukrainian => format!("📝 Примітка: Буде протестовано {} точок. Real-time ETA з'явиться після {} вимірювань.", total_points, 5),
        crate::ui::language::Language::Chinese => format!("📝 注意: 将测试 {} 个点。前 {} 次测量后将显示实时 ETA。", total_points, 5),
    };
    println!("{}\n", note_message);

    // ✅ НОВОЕ: Progress bar с ETA
    let pb = ProgressBar::new(total_points as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} | ETA: {eta}")
        .unwrap()
        .progress_chars("##-"));

    // ✅ CORRECTED: Check increment for extremely small values only
    if params.increment_value < 0.00001 {  // < 0.01 микросекунды
        eprintln!("⚠️  WARNING: Increment {:.6} ms is extremely small!", params.increment_value);
        eprintln!("   Minimum Windows timer resolution unit: 0.0001 ms (100 ns)");
        eprintln!("   Values smaller than 0.0001 ms will be indistinguishable.");
        eprintln!();
        print!("Continue anyway? (y/N): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Err(Error::new(ErrorKind::InvalidInput, "Increment too small"));
        }
    }

    if total_points > 100_000 {
        eprintln!("⚠️  WARNING: {} points will be tested!", total_points);
        eprintln!("   This will take approximately {:.1} hours", (total_points as f64 * 35.0) / 3600.0);
        eprintln!("   Press Ctrl+C to abort, or Enter to continue...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    }

    let mut measurements = Vec::new();
    let start_time = std::time::Instant::now();
    
    // ✅ FIX 2: EMA for smooth ETA
    let mut ema_time_per_point: Option<f64> = None;
    const EMA_ALPHA: f64 = 0.15;  // Smoothing factor (0.1-0.2 optimal)
    const MIN_SAMPLES_FOR_ETA: usize = 5;  // Минимум измерений перед показом ETA
    
    // Initialize the weights for performance scoring (used in TOPSIS ranking)
    let _weights = PerformanceWeights::default();

    // ✅ КРИТИЧНО: Целочисленный цикл вместо float инкремента!
    for i in 0..total_points {
        // Вычисляем current через целочисленный индекс - нет накопления ошибок!
        let current = params.start_value + (i as f64) * params.increment_value;
        
        // Защита от выхода за bounds из-за округлений
        if current > params.end_value {
            break;
        }
        
        pb.set_message(format!("{:.4} ms", current));
        let measurement = measure_resolution_robust(
            current,
            params.sample_value,
            3,  // 3 runs
            set_timer_path,
            measure_sleep_path,
            localization,
        ).await?;
        measurements.push(measurement);
        pb.inc(1);

        // ✅ НОВОЕ - показывает TOPSIS Score (лучший по всем критериям!)
        // Вычисляем TOPSIS для текущих измерений
        let temp_aggregated = aggregate_measurements(&measurements);
        let temp_topsis = topsis_ranking(&temp_aggregated);
        if !temp_topsis.is_empty() {
            let best = &temp_topsis[0];
            pb.println(format!("       Current best: {:.4} ms (TOPSIS: {:.4})", 
                best.resolution_ms, 
                best.closeness_coefficient));
        }

        // ✅ ADAPTIVE ETA с EMA сглаживанием
        if i + 1 >= MIN_SAMPLES_FOR_ETA {
            let elapsed = start_time.elapsed().as_secs_f64();
            let instant_time_per_point = elapsed / ((i + 1) as f64);

            // Exponential Moving Average (EMA)
            // Formula: EMA_new = alpha * current + (1 - alpha) * EMA_old
            let smoothed_time = match ema_time_per_point {
                Some(prev_ema) => {
                    // EMA сглаживание для плавного перехода
                    EMA_ALPHA * instant_time_per_point + (1.0 - EMA_ALPHA) * prev_ema
                },
                None => instant_time_per_point  // Первый раз
            };
            ema_time_per_point = Some(smoothed_time);
            let remaining_points = total_points - (i + 1);
            let eta_seconds = smoothed_time * (remaining_points as f64);

            // Форматирование ETA
            let eta_display = if eta_seconds < 90.0 {
                format!("{:.0}s", eta_seconds)
            } else if eta_seconds < 5400.0 {  // < 90 min
                format!("{:.1}m", eta_seconds / 60.0)
            } else {
                format!("{:.2}h", eta_seconds / 3600.0)
            };

            // ✅ НОВОЕ: Confidence interval для ETA
            let variance = (instant_time_per_point - smoothed_time).abs() / smoothed_time;
            let confidence_margin = smoothed_time * variance * 0.5;  // ±50% от вариативности
            let eta_min = (eta_seconds - confidence_margin * remaining_points as f64).max(0.0) / 60.0;
            let eta_max = (eta_seconds + confidence_margin * remaining_points as f64) / 60.0;

            // Показываем ETA range для честности
            if variance > 0.15 {  // Высокая вариативность
                pb.set_message(format!("{:.4} ms | ETA: {} (~{:.0}-{:.0}m)", current, eta_display, eta_min, eta_max));
            } else {
                pb.set_message(format!("{:.4} ms | ETA: {}", current, eta_display));
            }
        } else {
            // Первые несколько измерений
            pb.set_message(format!("{:.4} ms | ETA: calculating...", current));
        }
    }
    pb.finish_with_message("linear search completed");

    let aggregated = aggregate_measurements(&measurements);
    let topsis_results = topsis_ranking(&aggregated);

    println!("\n✅ Linear search completed:");
    println!("   Points checked: {}", measurements.len());
    println!("   Unique: {}", aggregated.len());
    println!("   Total time: {:.1} minutes", start_time.elapsed().as_secs_f64() / 60.0);

    // ✅ ДОБАВИТЬ: TOP-5 результаты (как в optimize_timer_resolution!)
    println!("\n{}", localization.get(LocalizationKey::TopsisRanking));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    for (i, result) in topsis_results.iter().take(5).enumerate() {
        let marker = if i == 0 { "🥇" } else if i == 1 { "🥈" } else if i == 2 { "🥉" } else { "  " };
        println!("{}  {}: {:.4} ms", marker, localization.get_rank(result.rank), result.resolution_ms);
        println!("{}", localization.get_topsis_score(result.closeness_coefficient));
        println!("{}", localization.get_p95_delta(result.criteria_scores.p95_delta));
        println!("{}", localization.get_mad(result.criteria_scores.mad));
        println!("{}", localization.get_p99_delta(result.criteria_scores.p99_delta));
        println!("{}", localization.get_ci_width(result.criteria_scores.confidence_width));
        println!();
    }

    let best = &topsis_results[0];
    println!("{}", localization.get_optimal_value(best.resolution_ms));
    println!("   {}\n", localization.get_optimal_recommendation((best.resolution_ms * 10_000.0) as i32));

    Ok(OptimizationResult {
        optimal_resolution: topsis_results[0].resolution_ms,
        topsis_score: topsis_results[0].closeness_coefficient,
        aggregated_measurements: aggregated,
        topsis_rankings: topsis_results,
    })
}
/// Force kill all SetTimerResolution.exe instances using multiple methods (quiet version for internal use)
fn kill_all_timer_processes() -> io::Result<()> {
    // Silent version without output
    let _ = Command::new("powershell")
        .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
            "Get-Process -Name SetTimerResolution -ErrorAction SilentlyContinue | Stop-Process -Force"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(200));
    Ok(())
}

/// Force kill all SetTimerResolution.exe instances using multiple methods
fn force_kill_all_timer_processes() -> io::Result<()> {
    println!("   Attempting to kill SetTimerResolution.exe processes...");

    // Method 1: PowerShell (more reliable if taskkill is disabled)
    let ps_result = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command",
            "Get-Process -Name SetTimerResolution -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.Id -Force }"
        ])
        .output();
    match ps_result {
        Ok(output) if output.status.success() => {
            println!("   ✓ PowerShell kill method succeeded");
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() && !stderr.contains("Cannot find") {
                eprintln!("   ⚠️ PowerShell warning: {}", stderr);
            }
        },
        Err(e) => {
            eprintln!("   ⚠️ PowerShell method failed: {}", e);
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Method 2: taskkill (if service is running)
    let taskkill_result = Command::new("taskkill")
        .args(&["/F", "/IM", "SetTimerResolution.exe", "/T"])
        .output();
    match taskkill_result {
        Ok(output) if output.status.success() => {
            println!("   ✓ taskkill method succeeded");
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("disabled") || stderr.contains("отключена") {
                println!("   ℹ️ taskkill service is disabled (using PowerShell only)");
            } else if !stderr.contains("not found") && !stderr.contains("не найден") {
                eprintln!("   ⚠️ taskkill warning: {}", stderr);
            }
        },
        Err(_) => {
            println!("   ℹ️ taskkill not available");
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Method 3: wmic (last resort)
    let wmic_result = Command::new("wmic")
        .args(&["process", "where", "name='SetTimerResolution.exe'", "delete"])
        .output();
    if let Ok(output) = wmic_result {
        if output.status.success() {
            println!("   ✓ wmic method succeeded");
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Final check
    let remaining = count_timer_processes();
    if remaining > 0 {
        println!("   ⚠️ {} instance(s) still remain after cleanup", remaining);
        Err(Error::new(ErrorKind::Other,
            format!("{} SetTimerResolution.exe instance(s) could not be killed", remaining)))
    } else {
        println!("   ✓ All instances successfully killed");
        Ok(())
    }
}

/// Count running SetTimerResolution.exe processes for diagnostics
fn count_timer_processes() -> usize {
    let output = Command::new("tasklist")
        .arg("/FI")
        .arg("IMAGENAME eq SetTimerResolution.exe")
        .output();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.matches("SetTimerResolution.exe").count()
    } else {
        0
    }
}
