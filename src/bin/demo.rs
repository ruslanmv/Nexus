use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser;
use nexus::{Agent, Kernel, Message, MessageKind, NexusConfig};
use serde_json::json;
use std::path::PathBuf;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tracing::{error, info, Level};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// Nexus - Universal Agent Kernel Runtime
///
/// A production-grade async kernel for spawning, supervising, and routing autonomous agents.
#[derive(Parser, Debug)]
#[command(name = "nexus")]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to configuration file (TOML format)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Log format (compact, json, pretty)
    #[arg(short = 'f', long, default_value = "compact")]
    log_format: String,

    /// Number of demo agents to spawn
    #[arg(short = 'n', long, default_value = "2")]
    num_agents: usize,

    /// Duration to run demo (seconds, 0 = run until Ctrl-C)
    #[arg(short = 'd', long, default_value = "0")]
    duration: u64,

    /// Generate default config file
    #[arg(long, value_name = "FILE")]
    generate_config: Option<PathBuf>,
}

struct MathAgent {
    id: Uuid,
    name: String,
}

impl MathAgent {
    fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Agent for MathAgent {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_message(&self, msg: Message) -> Option<Message> {
        info!(
            agent = %self.name,
            from = %msg.from,
            kind = ?msg.kind,
            payload = %msg.payload,
            "received message"
        );

        // Simulate some work
        sleep(Duration::from_millis(100)).await;

        // Toy behavior: respond to commands that include a `calculate` field.
        if msg.kind == MessageKind::Command && msg.payload.get("calculate").is_some() {
            return Some(msg.reply(self.id, json!({"result": 42, "agent": self.name})));
        }
        None
    }
}

fn setup_logging(level: &str, format: &str) -> Result<()> {
    let level = match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = EnvFilter::from_default_env().add_directive(level.into());

    match format.to_lowercase().as_str() {
        "json" => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .init();
        }
        "pretty" => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .pretty()
                .init();
        }
        _ => {
            // Default: compact
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(false)
                .compact()
                .init();
        }
    }

    Ok(())
}

async fn run_demo(config: NexusConfig, num_agents: usize, duration: u64) -> Result<()> {
    info!("Starting Nexus kernel with {} agents", num_agents);

    let kernel = Kernel::new(config.kernel);

    // Spawn agents
    let mut agent_ids = Vec::new();
    for i in 0..num_agents {
        let agent_name = format!("Agent-{}", i);
        let agent = MathAgent::new(&agent_name);
        let agent_id = kernel
            .spawn_agent(Box::new(agent))
            .await
            .context(format!("Failed to spawn {}", agent_name))?;
        agent_ids.push(agent_id);
        info!("Spawned {} with id {}", agent_name, agent_id);
    }

    // Send some demo messages
    if agent_ids.len() >= 2 {
        let system = Uuid::new_v4();

        // System -> First agent
        kernel
            .send(Message::new(
                system,
                agent_ids[0],
                MessageKind::Event,
                json!({"status": "ping"}),
            ))
            .await
            .context("Failed to send ping message")?;

        // First agent -> Second agent (request calc)
        kernel
            .send(Message::new(
                agent_ids[0],
                agent_ids[1],
                MessageKind::Command,
                json!({"calculate": "trajectory"}),
            ))
            .await
            .context("Failed to send calculate message")?;

        // Create a message chain
        for i in 0..agent_ids.len().saturating_sub(1) {
            kernel
                .send(Message::new(
                    agent_ids[i],
                    agent_ids[i + 1],
                    MessageKind::Command,
                    json!({"calculate": format!("task-{}", i)}),
                ))
                .await
                .context("Failed to send chain message")?;
        }
    }

    info!("All demo messages sent, agents are processing...");

    // Wait for duration or shutdown signal
    if duration > 0 {
        info!(
            "Running for {} seconds (press Ctrl-C to stop early)",
            duration
        );
        tokio::select! {
            _ = sleep(Duration::from_secs(duration)) => {
                info!("Duration elapsed, shutting down...");
            }
            _ = shutdown_signal() => {
                info!("Shutdown signal received, stopping...");
            }
        }
    } else {
        info!("Running indefinitely (press Ctrl-C to stop)");
        shutdown_signal().await;
        info!("Shutdown signal received, stopping...");
    }

    // Graceful shutdown
    kernel.shutdown().await;
    info!("Nexus kernel shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle config generation
    if let Some(path) = args.generate_config {
        let config = NexusConfig::default();
        config
            .save_to_file(&path)
            .context("Failed to save configuration")?;
        println!("Generated default configuration at: {}", path.display());
        return Ok(());
    }

    // Setup logging
    setup_logging(&args.log_level, &args.log_format)?;

    info!("Nexus Agent Kernel v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = if let Some(config_path) = args.config {
        info!("Loading configuration from: {}", config_path.display());
        NexusConfig::from_file(config_path).context("Failed to load configuration")?
    } else {
        NexusConfig::from_default_locations()
    };

    info!(
        mailbox_capacity = config.kernel.mailbox_capacity,
        send_timeout_ms = config.kernel.send_timeout_ms,
        handle_timeout_ms = config.kernel.handle_timeout_ms,
        "Kernel configuration loaded"
    );

    // Run the demo
    if let Err(e) = run_demo(config, args.num_agents, args.duration).await {
        error!("Fatal error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}
