//! Benchmarks for Nexus kernel message throughput

use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

struct BenchAgent {
    id: Uuid,
    name: String,
    counter: Arc<AtomicUsize>,
}

impl BenchAgent {
    fn new(name: String, counter: Arc<AtomicUsize>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            counter,
        }
    }
}

#[async_trait]
impl Agent for BenchAgent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_message(&self, _msg: Message) -> Option<Message> {
        self.counter.fetch_add(1, Ordering::Relaxed);
        None
    }
}

fn spawn_agents_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_agents");

    for num_agents in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*num_agents as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_agents),
            num_agents,
            |b, &num| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async move {
                        let kernel = Kernel::new(KernelConfig::default());
                        for i in 0..num {
                            let counter = Arc::new(AtomicUsize::new(0));
                            let agent = BenchAgent::new(format!("Agent-{}", i), counter);
                            kernel.spawn_agent(Box::new(agent)).await.unwrap();
                        }
                        kernel.shutdown().await;
                    });
            },
        );
    }
    group.finish();
}

fn message_send_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_send");

    for num_messages in [10, 100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*num_messages as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_messages),
            num_messages,
            |b, &num| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async move {
                        let config = KernelConfig {
                            mailbox_capacity: 2048,
                            ..Default::default()
                        };
                        let kernel = Kernel::new(config);

                        let counter = Arc::new(AtomicUsize::new(0));
                        let agent = BenchAgent::new("Receiver".to_string(), counter.clone());
                        let agent_id = kernel.spawn_agent(Box::new(agent)).await.unwrap();

                        let system = Uuid::new_v4();

                        for i in 0..num {
                            kernel
                                .send(Message::new(
                                    system,
                                    agent_id,
                                    MessageKind::Event,
                                    json!({"index": i}),
                                ))
                                .await
                                .unwrap();
                        }

                        // Wait for processing
                        sleep(Duration::from_millis(50)).await;

                        kernel.shutdown().await;
                        black_box(counter.load(Ordering::Relaxed));
                    });
            },
        );
    }
    group.finish();
}

fn agent_to_agent_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_to_agent");

    for num_agents in [5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*num_agents as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_agents),
            num_agents,
            |b, &num| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| async move {
                        let kernel = Kernel::new(KernelConfig::default());

                        let mut agent_ids = Vec::new();
                        for i in 0..num {
                            let counter = Arc::new(AtomicUsize::new(0));
                            let agent = BenchAgent::new(format!("Agent-{}", i), counter);
                            let id = kernel.spawn_agent(Box::new(agent)).await.unwrap();
                            agent_ids.push(id);
                        }

                        // Create a chain of messages
                        for i in 0..agent_ids.len() - 1 {
                            kernel
                                .send(Message::new(
                                    agent_ids[i],
                                    agent_ids[i + 1],
                                    MessageKind::Command,
                                    json!({"chain": i}),
                                ))
                                .await
                                .unwrap();
                        }

                        sleep(Duration::from_millis(20)).await;
                        kernel.shutdown().await;
                    });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    spawn_agents_benchmark,
    message_send_benchmark,
    agent_to_agent_benchmark
);
criterion_main!(benches);
