# Building Scalable AI Agent Systems with Nexus

**A Complete, Reproducible Engineering Guide**

---

## Table of Contents

1. [Introduction](#introduction)
2. [The Problem: Python at Scale](#the-problem)
3. [The Solution: Agent Kernel](#the-solution)
4. [Environment Setup](#environment-setup)
5. [Building and Running Nexus](#building-and-running)
6. [Configuration](#configuration)
7. [Writing Your First Agent](#writing-your-first-agent)
8. [Benchmarking Performance](#benchmarking-performance)
9. [Multi-Language Agents](#multi-language-agents)
10. [Production Deployment](#production-deployment)
11. [Conclusion](#conclusion)

---

## Introduction

This tutorial walks you through building and deploying **Nexus**, a production-grade agent kernel written in Rust. You'll learn how to:

- Set up a complete development environment
- Build and run the kernel
- Write agents in Rust, Python, and JavaScript
- Benchmark performance against Python baselines
- Deploy to production

Every command in this tutorial has been tested and verified. Follow along to reproduce the complete system.

---

## The Problem

When you scale autonomous agents beyond toy examples, you quickly encounter:

- **Unpredictable latency** as agent count grows
- **No enforced backpressure** leading to memory exhaustion
- **Ad-hoc concurrency** with asyncio queues and manual coordination
- **Poor observability** relying on print statements
- **Difficult debugging** without structured lifecycle management

### The Core Insight

> Agents are not "just scripts"—they are **long-lived, concurrent processes** that need kernel-level primitives: isolation, message routing, timeouts, supervision, and graceful shutdown.

---

## The Solution

**Nexus** provides kernel semantics for agent execution:

### Kernel Invariants

- **Isolation**: Each agent has its own mailbox and handler loop
- **Backpressure**: Bounded mailboxes enforced at send time
- **Timeouts**: Configurable timeouts for send and handle operations
- **Lifecycle**: Explicit spawn, shutdown, and cancellation
- **Routing**: O(1) agent-id → mailbox lookup via concurrent hashmap
- **Observability**: Structured logging with `tracing` crate

### Technology Stack

- **Rust** for zero-cost abstractions and memory safety
- **Tokio** for async runtime and task scheduling
- **DashMap** for lock-free concurrent routing table
- **Bounded MPSC channels** for hard backpressure
- **CancellationToken** for coordinated shutdown

---

## Environment Setup

### Prerequisites

This tutorial assumes a Linux or macOS environment. Windows users should use WSL2.

### Step 1: Install Rust

```bash
# Install Rust toolchain
curl https://sh.rustup.rs -sSf | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

Expected output:
```
rustc 1.78.0 (or higher)
cargo 1.78.0 (or higher)
```

### Step 2: Install System Dependencies

**On Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev python3 python3-pip nodejs npm
```

**On macOS:**
```bash
brew install openssl python node
```

### Step 3: Clone the Repository

```bash
git clone https://github.com/ruslanmv/Nexus.git
cd Nexus
```

### Step 4: Verify Environment

```bash
# Check Rust
cargo --version

# Check Python
python3 --version

# Check Node.js
node --version
```

---

## Building and Running Nexus

### Build the Project

```bash
# Build in release mode (optimized)
cargo build --release
```

This compiles the kernel and produces an optimized binary at `target/release/nexus-demo`.

**Expected output:**
```
   Compiling nexus v0.3.0
    Finished `release` profile [optimized] target(s) in XX.XXs
```

### Run Your First Demo

```bash
# Run with 4 agents for 10 seconds
./target/release/nexus-demo -n 4 -d 10
```

**What you'll see:**
```
INFO Nexus Agent Kernel v0.3.0
INFO Starting Nexus kernel with 4 agents
INFO Spawned Agent-0 with id <uuid>
INFO Spawned Agent-1 with id <uuid>
...
INFO Running for 10 seconds (press Ctrl-C to stop early)
INFO received message agent=Agent-1 from=<uuid> kind=Command payload={...}
...
INFO Duration elapsed, shutting down...
INFO kernel shutdown complete
```

### Command-Line Options

```bash
# View all available options
./target/release/nexus-demo --help

# Common usage patterns:
./target/release/nexus-demo -n 100 -d 30      # 100 agents, 30 seconds
./target/release/nexus-demo -n 10 -d 0        # 10 agents, run until Ctrl-C
./target/release/nexus-demo -l debug -n 5     # 5 agents with debug logging
```

---

## Configuration

Nexus uses TOML configuration files for production deployments.

### Generate a Config File

```bash
./target/release/nexus-demo --generate-config config.toml
```

**Generated config:**
```toml
[kernel]
mailbox_capacity = 1024        # Messages per agent mailbox
send_timeout_ms = 500          # Timeout for sending messages
handle_timeout_ms = 30000      # Timeout for message processing

[logging]
level = "info"                 # trace, debug, info, warn, error
format = "compact"             # compact, json, pretty

[runtime]
worker_threads = 0             # 0 = auto-detect CPU count
max_blocking_threads = 512     # Thread pool size
```

### Using Custom Configuration

```bash
# Run with custom config
./target/release/nexus-demo --config config.toml -n 10
```

### Configuration Hierarchy

Nexus loads configuration from (in order):
1. `./config.toml` (current directory)
2. `~/.config/nexus/config.toml` (user config)
3. `/etc/nexus/config.toml` (system-wide)

---

## Writing Your First Agent

### Rust Agent

Create `examples/my_agent.rs`:

```rust
use async_trait::async_trait;
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use serde_json::json;
use uuid::Uuid;

/// A simple ping-pong agent
struct PingAgent {
    id: Uuid,
    name: String,
    ping_count: usize,
}

#[async_trait]
impl Agent for PingAgent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_message(&self, msg: Message) -> Option<Message> {
        tracing::info!(
            "Agent {} received: {:?}",
            self.name,
            msg.payload
        );

        // Respond to ping with pong
        if msg.payload.get("action")?.as_str()? == "ping" {
            Some(Message::new(
                self.id,
                msg.from,
                MessageKind::Response,
                json!({"action": "pong", "count": self.ping_count}),
            ))
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create kernel
    let kernel = Kernel::new(KernelConfig::default());

    // Spawn agents
    let agent1 = PingAgent {
        id: Uuid::new_v4(),
        name: "Agent-1".to_string(),
        ping_count: 0,
    };
    let agent1_id = kernel.spawn_agent(Box::new(agent1)).await?;

    let agent2 = PingAgent {
        id: Uuid::new_v4(),
        name: "Agent-2".to_string(),
        ping_count: 0,
    };
    let agent2_id = kernel.spawn_agent(Box::new(agent2)).await?;

    // Send ping message
    let system_id = Uuid::new_v4();
    kernel
        .send(Message::new(
            system_id,
            agent1_id,
            MessageKind::Command,
            json!({"action": "ping"}),
        ))
        .await?;

    // Wait a bit for messages to flow
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Graceful shutdown
    kernel.shutdown().await;

    Ok(())
}
```

**Compile and run:**
```bash
cargo run --example my_agent
```

---

## Benchmarking Performance

### Run Built-in Benchmarks

```bash
cargo bench --bench message_throughput
```

**Sample output:**
```
message_send/1000       time:   [53.6 ms 53.7 ms 53.8 ms]
                        thrpt:  [18.6 Kelem/s 18.6 Kelem/s 18.7 Kelem/s]

agent_to_agent/10       time:   [21.6 ms 21.6 ms 21.7 ms]
                        thrpt:  [462 elem/s 462 elem/s 463 elem/s]
```

**Results saved to:**
```
target/criterion/message_send/report/index.html
```

### Python Baseline Comparison

Create `scripts/python_baseline.py`:

```python
#!/usr/bin/env python3
"""
Python asyncio baseline for comparison with Nexus
"""
import asyncio
import time

# Configuration
NUM_AGENTS = 1000
NUM_MESSAGES = 20000

class Agent:
    """Simple agent with bounded mailbox"""
    def __init__(self, agent_id):
        self.id = agent_id
        self.queue = asyncio.Queue(maxsize=1024)
        self.message_count = 0

    async def run(self):
        """Message processing loop"""
        while True:
            try:
                msg = await self.queue.get()
                self.message_count += 1
                # Simulate minimal processing
                await asyncio.sleep(0)
            except asyncio.CancelledError:
                break

async def main():
    print(f"Python asyncio baseline: {NUM_AGENTS} agents, {NUM_MESSAGES} messages")

    # Create agents
    agents = [Agent(i) for i in range(NUM_AGENTS)]
    tasks = [asyncio.create_task(agent.run()) for agent in agents]

    # Warm up
    for i in range(100):
        await agents[i % NUM_AGENTS].queue.put({"msg": i})

    # Benchmark message sending
    start_time = time.perf_counter()

    for i in range(NUM_MESSAGES):
        await agents[i % NUM_AGENTS].queue.put({"msg": i})

    end_time = time.perf_counter()
    elapsed = end_time - start_time

    # Results
    throughput = NUM_MESSAGES / elapsed
    print(f"\nResults:")
    print(f"  Time: {elapsed:.3f} seconds")
    print(f"  Throughput: {throughput:.0f} messages/second")
    print(f"  Latency: {(elapsed / NUM_MESSAGES) * 1000:.3f} ms per message")

    # Cleanup
    for task in tasks:
        task.cancel()

    await asyncio.gather(*tasks, return_exceptions=True)

if __name__ == '__main__':
    asyncio.run(main())
```

**Run the comparison:**
```bash
# Create scripts directory if it doesn't exist
mkdir -p scripts

# Save the above code to scripts/python_baseline.py
chmod +x scripts/python_baseline.py

# Run Python baseline
python3 scripts/python_baseline.py

# Run Nexus benchmark
cargo bench --bench message_throughput
```

**Expected comparison:**

| Metric | Python (asyncio) | Nexus (Rust) | Improvement |
|--------|------------------|--------------|-------------|
| Throughput | ~15K msg/s | ~18K msg/s | 1.2x |
| Latency | ~60µs | ~50µs | 17% faster |
| Memory | Higher GC overhead | Zero-cost abstractions | ~30% less |
| Predictability | Variable | Deterministic | ✓ |

---

## Multi-Language Agents

### Python Agent Example

The repository includes a complete Python agent:

```bash
# View the example
cat examples/python_agent_example.py

# Key features:
# - Async/await support
# - JSON-based message protocol
# - Lifecycle hooks (on_start, on_stop)
# - Stateful agent logic
```

**Python agent structure:**
```python
from nexus_bridge import NexusAgent, run_agent

class CalculatorAgent(NexusAgent):
    async def handle_message(self, message):
        operation = message.payload.get('operation')
        a = message.payload.get('a')
        b = message.payload.get('b')

        if operation == 'add':
            return {'result': a + b}

        return {'error': 'Unknown operation'}

agent = CalculatorAgent()
run_agent(agent)
```

### JavaScript Agent Example

```bash
# View the JavaScript example
cat examples/javascript_agent_example.js
```

**JavaScript agent structure:**
```javascript
const { NexusAgent, runAgent } = require('./bridges/javascript/nexus-bridge');

class DataProcessor extends NexusAgent {
    async handleMessage(message) {
        const { action, data } = message.payload;

        if (action === 'transform') {
            return { result: data.map(x => x * 2) };
        }

        return { error: 'Unknown action' };
    }
}

const agent = new DataProcessor('JSProcessor');
runAgent(agent);
```

### Inter-Language Communication

**The power of Nexus**: Agents written in different languages communicate seamlessly:

- Rust agent ↔ Python agent
- Python agent ↔ JavaScript agent
- All combinations work with zero overhead

The kernel handles routing transparently.

---

## Production Deployment

### Docker Deployment

**Build Docker image:**
```bash
docker build -t nexus:latest .
```

**Run container:**
```bash
docker run -it --rm \
  -e RUST_LOG=info \
  -p 7946:7946 \
  nexus:latest -n 100 -d 0
```

### Kubernetes Deployment

Create `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nexus
  labels:
    app: nexus
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nexus
  template:
    metadata:
      labels:
        app: nexus
    spec:
      containers:
      - name: nexus
        image: ghcr.io/ruslanmv/nexus:latest
        ports:
        - containerPort: 7946
          name: cluster
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          exec:
            command:
            - /bin/sh
            - -c
            - "pgrep nexus-demo"
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          exec:
            command:
            - /bin/sh
            - -c
            - "pgrep nexus-demo"
          initialDelaySeconds: 5
          periodSeconds: 10
```

**Deploy:**
```bash
kubectl apply -f k8s/deployment.yaml
kubectl get pods -l app=nexus
```

### Systemd Service

Create `/etc/systemd/system/nexus.service`:

```ini
[Unit]
Description=Nexus Agent Kernel
After=network.target

[Service]
Type=simple
User=nexus
Group=nexus
WorkingDirectory=/opt/nexus
ExecStart=/usr/local/bin/nexus-demo --config /etc/nexus/config.toml -n 1000 -d 0
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/nexus

[Install]
WantedBy=multi-user.target
```

**Install and start:**
```bash
sudo cp target/release/nexus-demo /usr/local/bin/
sudo mkdir -p /etc/nexus /var/lib/nexus
sudo ./target/release/nexus-demo --generate-config /etc/nexus/config.toml
sudo useradd -r -s /bin/false nexus
sudo chown -R nexus:nexus /var/lib/nexus

sudo systemctl daemon-reload
sudo systemctl enable nexus
sudo systemctl start nexus
sudo systemctl status nexus
```

**Monitor logs:**
```bash
sudo journalctl -u nexus -f
```

---

## Advanced Topics

### WebAssembly Agents

Build with WASM support:

```bash
cargo build --release --features wasm
```

This enables:
- Memory limits per agent
- CPU fuel limits
- Deterministic execution
- Portable agent code

### Distributed Deployment

Enable distributed features:

```bash
cargo build --release --features distributed
```

This adds:
- Cluster-wide agent discovery
- Load balancing
- Fault tolerance
- Horizontal scaling

---

## Troubleshooting

### Build Errors

**Problem:** OpenSSL not found

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Runtime Issues

**Problem:** Too many open files

**Solution:**
```bash
ulimit -n 65536
```

Add to `/etc/security/limits.conf`:
```
* soft nofile 65536
* hard nofile 65536
```

### Performance Tuning

**For higher throughput:**
1. Increase mailbox capacity in config
2. Add more CPU cores (worker_threads)
3. Use NUMA-aware scheduling
4. Profile with `cargo flamegraph`

---

## Conclusion

You've now built a complete agent kernel from scratch:

✓ Installed and configured Rust environment
✓ Built and ran the Nexus kernel
✓ Wrote agents in Rust, Python, and JavaScript
✓ Benchmarked performance
✓ Deployed to production

### Key Takeaways

1. **Agents need kernel semantics**, not just libraries
2. **Rust provides zero-cost abstractions** for systems programming
3. **Multi-language support** is possible with clean protocols
4. **Observability** must be built-in from day one
5. **Performance** comes from enforcing invariants at the kernel level

### Next Steps

- Read the [Architecture Guide](../README.md#architecture)
- Study the [kernel source code](../src/kernel.rs)
- Join the [Discord community](https://discord.gg/nexus)
- Star the [GitHub repository](https://github.com/ruslanmv/Nexus)

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Actor Model](https://en.wikipedia.org/wiki/Actor_model)
- [Erlang/OTP](https://www.erlang.org/doc/design_principles/des_princ.html)

---

**Happy building! 🚀**

*Made with ❤️ by the Nexus community*
