use crate::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "swal-agent", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run {
        task: String,
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { task, config } => {
            let config = Config::load(config)?;
            println!("Resolved Config: {:?}", config);
            println!("Task: {}", task);

            // Wiring call - register defaults with a placeholder ToolRegistry if necessary.
            // When issue 12 implements tools::register_defaults fully:
            let reg = swal_core::tool::ToolRegistry::new();
            crate::tools::register_defaults(&reg);

            // Placeholder loop run & guard for session module (to compile successfully against stubs or future real impls).
            // Once issue 11 is merged or active, we can wire session::start_session(&config).await?
            // For now, call it anyway if it matches the expected signature (start_session is currently just a stub in session.rs)
            // session::start_session(&config).await?; // Guard or placeholder for session run.
        }
    }

    Ok(())
}
