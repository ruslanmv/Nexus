use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// High-level type of message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageKind {
    Command,
    Event,
    Query,
    Response,
}

/// The unit of communication in Nexus.
///
/// This structure is intentionally generic and stable; it supports
/// correlation/tracing and typed payloads via JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub from: Uuid,
    pub to: Uuid,
    pub kind: MessageKind,
    /// Correlation id for multi-step workflows.
    pub conversation_id: Option<Uuid>,
    /// If this message is a direct reply, points to the original message id.
    pub reply_to: Option<Uuid>,
    /// Optional trace id to integrate with distributed tracing.
    pub trace_id: Option<String>,
    /// The payload. Use a JSON object/array/string/number etc.
    pub payload: Value,
}

impl Message {
    pub fn new(from: Uuid, to: Uuid, kind: MessageKind, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            from,
            to,
            kind,
            conversation_id: None,
            reply_to: None,
            trace_id: None,
            payload,
        }
    }

    pub fn reply(&self, from: Uuid, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            from,
            to: self.from,
            kind: MessageKind::Response,
            conversation_id: self.conversation_id.or(Some(self.id)),
            reply_to: Some(self.id),
            trace_id: self.trace_id.clone(),
            payload,
        }
    }
}

/// Autonomous agent contract.
///
/// Agents are spawned by the [`Kernel`](crate::Kernel). Each agent processes messages
/// serially (one mailbox consumer task) to preserve per-agent ordering.
#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;

    /// Process an incoming message. Returning `Some(Message)` will cause the kernel
    /// to route the response (e.g. reply to the sender). Returning `None` means
    /// the message was handled without a response.
    async fn handle_message(&self, message: Message) -> Option<Message>;
}
