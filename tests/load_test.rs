//! Load tests for the Nexus kernel
//!
//! These tests verify system behavior under high load with many agents
//! and high message throughput.

use async_trait::async_trait;
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

struct LoadTestAgent {
    id: Uuid,
    name: String,
    messages_received: Arc<AtomicUsize>,
}

impl LoadTestAgent {
    fn new(name: String, counter: Arc<AtomicUsize>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            messages_received: counter,
        }
    }
}

#[async_trait]
impl Agent for LoadTestAgent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_message(&self, _msg: Message) -> Option<Message> {
        self.messages_received.fetch_add(1, Ordering::SeqCst);
        // Simulate minimal work
        tokio::task::yield_now().await;
        None
    }
}

#[tokio::test]
async fn spawn_100_agents() {
    let kernel = Kernel::new(KernelConfig::default());
    let mut agent_ids = Vec::new();

    for i in 0..100 {
        let counter = Arc::new(AtomicUsize::new(0));
        let agent = LoadTestAgent::new(format!("Agent-{}", i), counter);
        let id = kernel.spawn_agent(Box::new(agent)).await.unwrap();
        agent_ids.push(id);
    }

    assert_eq!(agent_ids.len(), 100);

    // Verify all agents are registered
    for id in &agent_ids {
        assert!(kernel.contains(*id));
    }

    kernel.shutdown().await;
}

#[tokio::test]
async fn high_message_throughput() {
    let config = KernelConfig {
        mailbox_capacity: 2048,
        ..Default::default()
    };
    let kernel = Kernel::new(config);

    let counter = Arc::new(AtomicUsize::new(0));
    let agent = LoadTestAgent::new("Receiver".to_string(), counter.clone());
    let agent_id = kernel.spawn_agent(Box::new(agent)).await.unwrap();

    let system = Uuid::new_v4();
    let num_messages = 1000;

    // Send many messages rapidly
    for i in 0..num_messages {
        kernel
            .send(Message::new(
                system,
                agent_id,
                MessageKind::Event,
                json!({"index": i}),
            ))
            .await
            .expect("Failed to send message");
    }

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    let received = counter.load(Ordering::SeqCst);
    assert_eq!(received, num_messages, "Not all messages were processed");

    kernel.shutdown().await;
}

#[tokio::test]
async fn many_agents_many_messages() {
    let config = KernelConfig {
        mailbox_capacity: 512,
        ..Default::default()
    };
    let kernel = Kernel::new(config);

    let num_agents = 50;
    let messages_per_agent = 20;

    let mut agents_and_counters = Vec::new();

    // Spawn agents
    for i in 0..num_agents {
        let counter = Arc::new(AtomicUsize::new(0));
        let agent = LoadTestAgent::new(format!("Agent-{}", i), counter.clone());
        let id = kernel.spawn_agent(Box::new(agent)).await.unwrap();
        agents_and_counters.push((id, counter));
    }

    let system = Uuid::new_v4();

    // Send messages to each agent
    for (agent_id, _) in &agents_and_counters {
        for j in 0..messages_per_agent {
            kernel
                .send(Message::new(
                    system,
                    *agent_id,
                    MessageKind::Command,
                    json!({"task": j}),
                ))
                .await
                .expect("Failed to send message");
        }
    }

    // Wait for all messages to be processed
    sleep(Duration::from_millis(1000)).await;

    // Verify all agents received all messages
    for (agent_id, counter) in &agents_and_counters {
        let received = counter.load(Ordering::SeqCst);
        assert_eq!(
            received, messages_per_agent,
            "Agent {} did not receive all messages",
            agent_id
        );
    }

    let total_messages: usize = agents_and_counters
        .iter()
        .map(|(_, c)| c.load(Ordering::SeqCst))
        .sum();

    assert_eq!(
        total_messages,
        num_agents * messages_per_agent,
        "Total message count mismatch"
    );

    kernel.shutdown().await;
}

#[tokio::test]
async fn agent_to_agent_communication_chain() {
    let kernel = Kernel::new(KernelConfig::default());
    let num_agents = 10;

    let mut counters = Vec::new();
    let mut agent_ids = Vec::new();

    // Create a chain of agents
    for i in 0..num_agents {
        let counter = Arc::new(AtomicUsize::new(0));
        let agent = LoadTestAgent::new(format!("ChainAgent-{}", i), counter.clone());
        let id = kernel.spawn_agent(Box::new(agent)).await.unwrap();
        counters.push(counter);
        agent_ids.push(id);
    }

    // Send messages down the chain: 0 -> 1 -> 2 -> ... -> n-1
    for i in 0..agent_ids.len() - 1 {
        kernel
            .send(Message::new(
                agent_ids[i],
                agent_ids[i + 1],
                MessageKind::Event,
                json!({"step": i}),
            ))
            .await
            .unwrap();
    }

    // Wait for processing
    sleep(Duration::from_millis(200)).await;

    // Verify that agents 1 through n-1 received messages (not agent 0)
    assert_eq!(counters[0].load(Ordering::SeqCst), 0);
    for (i, counter) in counters.iter().enumerate().skip(1) {
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "Agent {} did not receive message",
            i
        );
    }

    kernel.shutdown().await;
}

#[tokio::test]
#[ignore] // This test is slow, run with --include-ignored
async fn stress_test_1000_agents() {
    let kernel = Kernel::new(KernelConfig::default());
    let num_agents = 1000;

    for i in 0..num_agents {
        let counter = Arc::new(AtomicUsize::new(0));
        let agent = LoadTestAgent::new(format!("StressAgent-{}", i), counter);
        kernel.spawn_agent(Box::new(agent)).await.unwrap();
    }

    // Just verify we can spawn and shutdown cleanly
    sleep(Duration::from_millis(100)).await;
    kernel.shutdown().await;
}
