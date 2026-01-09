use thiserror::Error;
use uuid::Uuid;

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("agent mailbox closed: {0}")]
    MailboxClosed(Uuid),

    #[error("kernel is shutting down")]
    ShuttingDown,

    #[error("send timeout to agent {0}")]
    SendTimeout(Uuid),

    #[error("agent handler timeout: {0}")]
    HandleTimeout(Uuid),
}