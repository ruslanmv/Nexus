# Project Nexus

**The Universal Standard Kernel for Agentic AI**

[![CI](https://github.com/ruslanmv/Nexus/workflows/CI/badge.svg)](https://github.com/ruslanmv/Nexus/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Nexus** is a production-grade async kernel for autonomous agents. Built in Rust with Tokio, it provides the execution substrate for spawning, supervising, and routing thousands of concurrent agents with deterministic behavior and operational safety.

> *"Just as operating systems standardized on kernels to manage processes, memory, and I/O, Project Nexus defines the first universal Agent Kernel—a low-level, high-performance runtime that governs how autonomous agents are spawned, isolated, scheduled, and allowed to interact."*

## Features

### Core Capabilities
- **High-Performance Agent Spawning** - Launch thousands of concurrent agents with minimal overhead
- **Message-Based Communication** - Type-safe, structured messaging between agents
- **Graceful Lifecycle Management** - Supervised execution with clean shutdown semantics
- **Backpressure & Flow Control** - Bounded mailboxes prevent resource exhaustion
- **Timeout Enforcement** - Configurable timeouts for send and handle operations
- **Structured Logging** - Production-ready observability with `tracing`
- **Configuration Management** - TOML-based configuration with defaults

### Production Ready
- **Signal Handling** - Responds to SIGTERM/SIGINT for graceful shutdown
- **CLI Interface** - Full-featured command-line interface with clap
- **Systemd Integration** - Native systemd service support
- **Security Hardened** - Follows Rust security best practices
- **Comprehensive Testing** - Unit tests, integration tests, load tests, and benchmarks
- **CI/CD Pipeline** - Automated testing and validation

## Quick Start

### Installation

#### Quick Install (Recommended)
```bash
./install.sh
```

#### Manual Installation
```bash
make install
```

#### Build from Source
```bash
# Build release binary
make build

# Run tests
make test

# Run demo
make run
```

### Running Nexus

```bash
# Run with default configuration
nexus-runtime

# Run with custom config
nexus-runtime --config config.toml

# Run with 10 agents for 30 seconds
nexus-runtime -n 10 -d 30

# Show all options
nexus-runtime --help

# Generate default config file
nexus-runtime --generate-config my-config.toml
```

### Basic Usage

```rust
use nexus::{Agent, Kernel, KernelConfig, Message, MessageKind};
use async_trait::async_trait;
use uuid::Uuid;
use serde_json::json;

// Define your agent
struct MyAgent {
    id: Uuid,
    name: String,
}

#[async_trait]
impl Agent for MyAgent {
    fn id(&self) -> Uuid { self.id }
    fn name(&self) -> &str { &self.name }

    async fn handle_message(&self, msg: Message) -> Option<Message> {
        // Process message and optionally return a response
        println!("Received: {:?}", msg);
        None
    }
}

#[tokio::main]
async fn main() {
    // Create kernel
    let kernel = Kernel::new(KernelConfig::default());

    // Spawn agents
    let agent = MyAgent {
        id: Uuid::new_v4(),
        name: "Agent-1".to_string()
    };
    let agent_id = kernel.spawn_agent(Box::new(agent)).await.unwrap();

    // Send messages
    let system = Uuid::new_v4();
    kernel.send(Message::new(
        system,
        agent_id,
        MessageKind::Command,
        json!({"action": "process"})
    )).await.unwrap();

    // Graceful shutdown
    kernel.shutdown().await;
}
```

## Configuration

Nexus uses TOML configuration files. Configuration is loaded from:
1. `./config.toml` (current directory)
2. `~/.config/nexus/config.toml`
3. `/etc/nexus/config.toml`

Example configuration:

```toml
[kernel]
mailbox_capacity = 1024
send_timeout_ms = 500
handle_timeout_ms = 30000

[logging]
level = "info"
format = "compact"

[runtime]
worker_threads = 0  # 0 = auto-detect CPU count
max_blocking_threads = 512
```

## Development

### Prerequisites
- Rust 1.78 or later
- Cargo

### Build Commands

```bash
make help          # Show all available commands
make build         # Build release binary
make run           # Run demo application
make test          # Run unit and integration tests
make test-all      # Run all tests including ignored ones
make bench         # Run benchmarks
make fmt           # Format code
make clippy        # Run linter
make doctor        # Check development environment
```

### Testing

```bash
# Run all tests
cargo test

# Run with ignored tests (including load tests)
cargo test -- --include-ignored

# Run benchmarks
cargo bench

# Run specific test
cargo test routes_messages_and_shutdowns
```

### Project Structure

```
Nexus/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── kernel.rs           # Core kernel implementation
│   ├── agent.rs            # Agent trait and messaging
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types
│   └── bin/
│       └── demo.rs         # Runtime binary
├── tests/
│   ├── kernel_basic.rs     # Basic integration tests
│   └── load_test.rs        # Load and stress tests
├── benches/
│   └── message_throughput.rs  # Performance benchmarks
├── scripts/
│   ├── deploy.sh           # Production deployment script
│   └── nexus.service       # Systemd service file
├── Makefile                # Build automation
├── install.sh              # Installation script
└── Cargo.toml              # Rust package manifest
```

## Production Deployment

### System-Wide Installation

```bash
# Install as system service (requires sudo)
sudo ./scripts/deploy.sh
```

This will:
- Create `nexus` system user
- Install binary to `/usr/local/bin`
- Create configuration in `/etc/nexus`
- Set up systemd service
- Start and enable the service

### Service Management

```bash
# Check status
systemctl status nexus

# View logs
journalctl -u nexus -f

# Restart service
systemctl restart nexus

# Stop service
systemctl stop nexus
```

## Architecture

### Design Principles

1. **Process Model** - Each agent is a long-lived, stateful process managed by the kernel
2. **Message Passing** - Agents communicate exclusively through structured messages
3. **Isolation** - Agents are isolated from each other and from the kernel
4. **Supervision** - The kernel supervises agent lifecycle and handles failures
5. **Backpressure** - Bounded mailboxes provide natural flow control

### Core Components

- **Kernel** - Central orchestrator managing agent lifecycle and message routing
- **Agent** - Autonomous entity with unique ID and message handler
- **Message** - Structured communication unit with type-safe payloads
- **Registry** - Lock-free concurrent hashmap for agent discovery
- **Mailbox** - Bounded MPSC channel for agent message queue

### Performance Characteristics

- **Agent Spawn Time** - ~100µs per agent
- **Message Latency** - ~50µs end-to-end
- **Throughput** - 100k+ messages/second (single machine)
- **Concurrency** - Tested with 10,000+ concurrent agents
- **Memory** - ~1KB overhead per agent

## Benchmarks

Run benchmarks to measure performance on your system:

```bash
make bench
```

Results from reference system (AMD Ryzen 9 / 32GB RAM):
- Spawn 100 agents: ~10ms
- Send 1000 messages: ~50ms
- Agent-to-agent chain (10 hops): ~5ms

## Roadmap

### Version 0.3.0 (Q1 2026)
- [ ] WebAssembly agent isolation
- [ ] Python language bridge
- [ ] JavaScript language bridge
- [ ] Distributed agent registry

### Version 0.4.0 (Q2 2026)
- [ ] Agent discovery and service mesh
- [ ] Built-in metrics and monitoring
- [ ] Persistent agent state
- [ ] Hot code reload

### Future
- [ ] Multi-node clustering
- [ ] Advanced scheduling policies
- [ ] Resource quotas and limits
- [ ] Standard agent protocol (SAP)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Security

For security issues, please see [SECURITY.md](SECURITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Citation

If you use Nexus in your research or project, please cite:

```bibtex
@software{nexus2026,
  title = {Nexus: A Universal Standard Kernel for Agentic AI},
  author = {Project Nexus Contributors},
  year = {2026},
  url = {https://github.com/ruslanmv/Nexus}
}
```

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) async runtime
- Inspired by Erlang/OTP and the Actor model
- Part of the vision for universal agentic infrastructure

---

**Project Nexus** - The Linux Kernel of Agentic Intelligence
