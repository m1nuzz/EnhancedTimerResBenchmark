use timer_res_benchmark::run_benchmark;

#[tokio::main]
async fn main() {
    if let Err(e) = run_benchmark().await {
        eprintln!("Fatal error: {}", e);
        std::process::exit(1);
    }
}