//! swal-agent — TUI/CLI entry: wires gestalt state, skills cache, starts loop/gateway/sched

mod cli;
mod config;
mod session;
mod tools;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
