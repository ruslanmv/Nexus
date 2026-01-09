# Changelog

All notable changes to Project Nexus will be documented in this file.

## 0.3.0 - 2026-01-09

### 🚀 MAJOR FEATURES - Version 0.3.0 Complete!

**WebAssembly Agent Isolation**
- Secure WASM sandbox execution with wasmtime
- Resource limits (memory, fuel, stack)
- WASI support for system calls
- `WasmAgent` and `WasmAgentBuilder` API
- Feature flag: `wasm`

**Python Language Bridge**
- Full Python agent support via `nexus_bridge.py`
- Async/await with asyncio
- JSON protocol over stdin/stdout
- Example calculator agent included
- Zero external dependencies

**JavaScript/Node.js Language Bridge**
- Complete Node.js agent support via `nexus-bridge.js`
- Promise-based async handling
- JSON protocol communication
- Example data processor agent
- Node.js 14+ compatible

**Distributed Agent Registry**
- Cluster-wide agent discovery
- Multi-node coordination
- Health checking and heartbeats
- Agent-to-node mapping
- `DistributedRegistry` API

**Multi-Language Interoperability**
- Seamless Rust ↔ Python ↔ JavaScript communication
- Zero-overhead protocol bridges
- Type-safe cross-language messaging

### Enhanced
- Comprehensive language bridge documentation
- Example agents for all supported languages
- Updated README with v0.3.0 tutorials
- Completed roadmap items marked

### Dependencies Added
- wasmtime 28.0 (optional)
- wasmtime-wasi 28.0 (optional)
- bincode 1.3, base64 0.22
- reqwest 0.12 (optional)
- hyper 1.0 (optional)

### Features
- New cargo features: `wasm`, `distributed`, `full`

## 0.2.0

- Renamed project to **Nexus**.
- Added library crate (`src/lib.rs`) with a production-oriented `Kernel`.
- Implemented bounded mailboxes, send/handle timeouts, and structured errors.
- Added graceful shutdown with task tracking.
- Replaced `println!` with structured logging (`tracing`).
- Added CI (fmt, clippy, tests, release build).
- Added integration tests and Dockerfile.
- Production-ready deployment infrastructure
- Comprehensive build system (Makefile)
- TOML configuration system
- CLI with clap
- Installation scripts and systemd service
