# Nexus

**Nexus** is a production-oriented **async kernel for autonomous agents**.

It provides:
- Agent spawning and task supervision (Tokio)
- Structured message routing between agents
- Lifecycle management and graceful shutdown
- Backpressure + timeouts with structured errors
- Operational logging via `tracing`

## Quickstart

```bash
cargo run --bin nexus-demo
```

## Dev targets

- `make build`  (release build)
- `make run`    (demo)
- `make test`
- `make fmt`
- `make clippy` (lint, denies warnings)
- `make clean`

## Design notes

- Each agent processes messages **serially** (per-agent FIFO ordering).
- The kernel uses **bounded** mailboxes to provide backpressure.
- `Kernel::send` enforces a configurable send timeout.
- Agent message handling is protected by a configurable timeout.
