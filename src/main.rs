use timer_res_benchmark::run_benchmark;

#[tokio::main]
async fn main() {
    // On Windows, set the console output codepage to UTF-8
    #[cfg(windows)]
    {
        if !std::process::Command::new("chcp")
            .arg("65001")
            .status()
            .map_or(false, |s| s.success())
        {
            eprintln!("Warning: Failed to set console codepage to UTF-8. Some characters may not display correctly.");
        }
    }

    if let Err(e) = run_benchmark().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}