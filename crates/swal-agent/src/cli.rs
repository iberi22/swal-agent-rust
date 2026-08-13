use crate::config::Config;
use crate::tools;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swal_loop::provider::MockProvider;
use swal_loop::r#loop::AgentLoop;
use swal_loop::skills::SkillLoader;

struct AgentLoopRunner {
    loop_: Arc<AgentLoop>,
}

#[async_trait::async_trait]
impl swal_sched::ticker::RunTask for AgentLoopRunner {
    async fn run(&self, task: &str) -> Result<(), String> {
        match self.loop_.run(task).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

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
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: std::net::SocketAddr,
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
        Commands::Serve { addr, config } => {
            let config_loaded = Config::load(config.clone())?;
            println!("Resolved Config: {:?}", config_loaded);
            println!("Serving on: {}", addr);

            // Wire default tools and make registry
            let reg = tools::make_registry();

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
            let loop_shared = Arc::new(loop_);

            // Optional scheduled tasks from config
            let mut scheduled_tasks = None;
            if let Some(ref path) = config {
                if let Ok(content) = std::fs::read_to_string(path) {
                    #[derive(serde::Deserialize)]
                    struct ConfigWithTasks {
                        tasks: Option<Vec<swal_sched::ticker::ScheduledTask>>,
                    }
                    if let Ok(parsed) = serde_json::from_str::<ConfigWithTasks>(&content) {
                        scheduled_tasks = parsed.tasks;
                    }
                }
            }

            if let Some(tasks) = scheduled_tasks {
                if !tasks.is_empty() {
                    let runner = Arc::new(AgentLoopRunner { loop_: loop_shared.clone() });
                    let mut scheduler = swal_sched::ticker::Scheduler::new(runner);
                    scheduler.tasks = tasks;
                    tokio::spawn(async move {
                        scheduler.run_forever().await;
                    });
                }
            }

            // Pass Arc to gateway
            let gateway_agent: Arc<dyn swal_gateway::http::AgentHandle> = loop_shared;
            swal_gateway::http::serve(gateway_agent, addr).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}
