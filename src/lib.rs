//! Timer Resolution Benchmark Library
//!
//! This library provides modular components for timer resolution benchmarking.

pub mod core;
pub mod stats;
pub mod optimization;
pub mod ui;
pub mod utils;
pub mod language;

pub use core::run_benchmark;

/// Library version
pub const VERSION: &str = "0.3.2";