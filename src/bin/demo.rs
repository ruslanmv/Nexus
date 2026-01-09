use async_trait::async_trait;
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

struct MathAgent {
    id: Uuid,
    name: String,
}

impl MathAgent {
    fn new(name: &str) -> Self {
        Self { id: Uuid::new_v4(), name: name.to_string() }
    }
}

#[async_trait]
impl Agent for MathAgent {
    fn id(&self) -> Uuid { self.id }
    fn name(&self) -> &str { &self.name }

    async fn handle_message(&self, msg: Message) -> Option<Message> {
        info!(agent=%self.name, from=%msg.from, kind=?msg.kind, payload=%msg.payload, "received");

        sleep(Duration::from_millis(200)).await;

        // Toy behavior: respond to commands that include a `calculate` field.
        if msg.kind == MessageKind::Command && msg.payload.get("calculate").is_some() {
            return Some(msg.reply(self.id, json!({"result": 42}))); // replace with real work
        }
        None
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_target(false)
        .compact()
        .init();

    let kernel = Kernel::new(KernelConfig::default());

    let alpha = MathAgent::new("Alpha");
    let beta = MathAgent::new("Beta");
    let alpha_id = kernel.spawn_agent(Box::new(alpha)).await.expect("spawn alpha");
    let beta_id = kernel.spawn_agent(Box::new(beta)).await.expect("spawn beta");

    // System -> Alpha
    let system = Uuid::new_v4();
    kernel
        .send(Message::new(system, alpha_id, MessageKind::Event, json!({"status": "ping"})))
        .await
        .unwrap();

    // Alpha -> Beta (request calc)
    kernel
        .send(Message::new(alpha_id, beta_id, MessageKind::Command, json!({"calculate": "trajectory"})))
        .await
        .unwrap();

    sleep(Duration::from_secs(1)).await;
    kernel.shutdown().await;
}
