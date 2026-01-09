//! Nexus — a production-oriented async kernel for autonomous agents.
//!
//! Nexus provides:
//! - **Spawn & supervision**: run many independent agents under Tokio tasks.
//! - **Message routing**: send structured messages between agents by id.
//! - **Lifecycle management**: graceful shutdown with task tracking.
//! - **Operational safety**: backpressure, timeouts, and structured errors.

pub mod agent;
pub mod config;
pub mod error;
pub mod kernel;

pub use agent::{Agent, Message, MessageKind};
pub use config::{KernelConfig, LoggingConfig, NexusConfig, RuntimeConfig};
pub use error::{KernelError, KernelResult};
pub use kernel::Kernel;
