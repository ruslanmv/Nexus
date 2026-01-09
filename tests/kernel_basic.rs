use async_trait::async_trait;
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

struct CountingAgent {
    id: Uuid,
    name: String,
    count: Arc<AtomicUsize>,
}

impl CountingAgent {
    fn new(name: &str, count: Arc<AtomicUsize>) -> Self {
        Self { id: Uuid::new_v4(), name: name.to_string(), count }
    }
}

#[async_trait]
impl Agent for CountingAgent {
    fn id(&self) -> Uuid { self.id }
    fn name(&self) -> &str { &self.name }

    async fn handle_message(&self, _msg: Message) -> Option<Message> {
        self.count.fetch_add(1, Ordering::SeqCst);
        None
    }
}

#[tokio::test]
async fn routes_messages_and_shutdowns() {
    let mut cfg = KernelConfig::default();
    cfg.mailbox_capacity = 8;
    let kernel = Kernel::new(cfg);

    let c1 = Arc::new(AtomicUsize::new(0));
    let agent = CountingAgent::new("A", c1.clone());
    let id = kernel.spawn_agent(Box::new(agent)).await.unwrap();

    let system = Uuid::new_v4();
    for _ in 0..5 {
        kernel
            .send(Message::new(system, id, MessageKind::Event, json!({"ping": true})))
            .await
            .unwrap();
    }

    // Allow the agent task to drain.
    sleep(Duration::from_millis(100)).await;

    assert_eq!(c1.load(Ordering::SeqCst), 5);

    kernel.shutdown().await;
}

#[tokio::test]
async fn returns_error_when_agent_missing() {
    let kernel = Kernel::default();
    let system = Uuid::new_v4();
    let missing = Uuid::new_v4();

    let err = kernel
        .send(Message::new(system, missing, MessageKind::Event, json!({"hello": "world"})))
        .await
        .unwrap_err();

    assert!(matches!(err, nexus::KernelError::AgentNotFound(_)));
    kernel.shutdown().await;
}