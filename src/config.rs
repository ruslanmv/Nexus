use std::time::Duration;

/// Runtime knobs for the [`Kernel`](crate::Kernel).
///
/// These defaults are conservative for production:
/// - bounded mailboxes (backpressure)
/// - timeouts for sending + handling
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Bounded mailbox size per agent.
    pub mailbox_capacity: usize,
    /// Maximum time we will wait to enqueue a message into an agent's mailbox.
    pub send_timeout: Duration,
    /// Maximum time an agent is allowed to spend processing a single message.
    pub handle_timeout: Duration,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            mailbox_capacity: 1024,
            send_timeout: Duration::from_millis(500),
            handle_timeout: Duration::from_secs(30),
        }
    }
}
