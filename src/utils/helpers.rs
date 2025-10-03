//! Utility functions and helpers for timer resolution benchmarking
//!
//! This module provides various utility functions used throughout the application.

use std::io;

/// Parse measurement output from MeasureSleep.exe
pub fn parse_measurement_output(output: &[u8]) -> io::Result<(f64, f64)> {
    let output_str = std::str::from_utf8(output)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut avg = None;
    let mut stdev = None;

    for line in output_str.lines() {
        if avg.is_none() && line.starts_with("Avg: ") {
            avg = line[5..].parse().ok();
        } else if stdev.is_none() && line.starts_with("STDEV: ") {
            stdev = line[7..].parse().ok();
        }

        if avg.is_some() && stdev.is_some() {
            break;
        }
    }

    avg.zip(stdev).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid measurement output"))
}

/// Interactive prompt for configuration values
pub fn prompt(
    description: &str,
    current: &str,
    localization: &crate::ui::localization::Localization,
) -> io::Result<Option<String>> {
    let mut input = String::new();
    println!("▸ {}: {}{}", description, current, localization.get_keep_current());
    println!("{}", localization.get_enter_new_value());
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Prompt user to continue (generic version)
pub fn prompt_user(message: &str) -> io::Result<()> {
    println!("{}", message);
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}

/// Prompt user before exit
pub fn prompt_exit() -> io::Result<()> {
    println!("Press Enter to exit...");
    io::stdin().read_line(&mut String::new())?;
    Ok(())
}

/// Clean up running processes
pub fn cleanup_processes() -> io::Result<()> {
    // Placeholder implementation
    println!("Cleaning up processes...");
    Ok(())
}

/// Check if running with admin privileges
pub fn is_admin() -> bool {
    // Placeholder implementation
    true
}