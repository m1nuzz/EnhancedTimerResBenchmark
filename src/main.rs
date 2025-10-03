use timer_res_benchmark::run_benchmark;

#[tokio::main]
async fn main() {
    // Enable UTF-8 support in Windows console
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(&["/c", "chcp 65001 > nul"]).status();
    }
    
    if let Err(e) = run_benchmark().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}