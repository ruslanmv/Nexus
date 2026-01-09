//! Nexus — a production-oriented async kernel for autonomous agents.
//!
//! Nexus provides:
//! - **Spawn & supervision**: run many independent agents under Tokio tasks.
//! - **Message routing**: send structured messages between agents by id.
//! - **Lifecycle management**: graceful shutdown with task tracking.
//! - **Operational safety**: backpressure, timeouts, and structured errors.
//! - **WebAssembly isolation**: secure agent execution in WASM sandboxes (feature: `wasm`).
//! - **Language bridges**: Python and JavaScript agent support.
//! - **Distributed registry**: cluster-wide agent discovery (feature: `distributed`).

pub mod agent;
pub mod config;
pub mod error;
pub mod kernel;

// Optional modules based on features
#[cfg(feature = "wasm")]
pub mod wasm;

pub mod distributed;

pub use agent::{Agent, Message, MessageKind};
pub use config::{KernelConfig, LoggingConfig, NexusConfig, RuntimeConfig};
pub use error::{KernelError, KernelResult};
pub use kernel::Kernel;

#[cfg(feature = "wasm")]
pub use wasm::{WasmAgent, WasmAgentBuilder, WasmConfig};

pub use distributed::{ClusterConfig, ClusterStats, DistributedRegistry, NodeInfo, NodeStatus};
