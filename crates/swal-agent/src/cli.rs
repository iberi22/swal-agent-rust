use crate::config::Config;
use crate::tools;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swal_loop::provider::MockProvider;
use swal_loop::r#loop::AgentLoop;
use swal_loop::skills::SkillLoader;

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

            // Wire default tools and make registry
            let reg = tools::make_registry();

            // Open session (best-effort)
            let session = match crate::session::SessionHandle::open(&config) {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("Warning: Failed to open session: {}", e);
                    None
                }
            };

            if let Some(ref s) = session {
                if let Err(e) = s.append("user", &task).await {
                    eprintln!("Warning: Failed to append user message to session: {}", e);
                }
            }

            // Create skills directory if it does not exist, then load skills
            let skills_path = Path::new("skills");
            if !skills_path.exists() {
                let _ = std::fs::create_dir_all(skills_path);
            }
            let loader = SkillLoader::new("skills")?;

            // Setup mock provider with final_response
            let final_response = swal_loop::provider::ProviderResponse {
                content: "done".to_string(),
                tool_calls: vec![],
            };
            let provider = Arc::new(MockProvider::new(vec![final_response]));

            // Initialize and execute AgentLoop
            let loop_ = AgentLoop::new(provider, reg, loader);
            let out = loop_.run(&task).await?;

            // Print final content to stdout
            println!("{}", out.content);

            // Persist final content (best-effort)
            if let Some(ref s) = session {
                if let Err(e) = s.append("assistant", &out.content).await {
                    eprintln!("Warning: Failed to append assistant message to session: {}", e);
                }
            }
        }
    }

    Ok(())
}
