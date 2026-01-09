use crate::agent::{Agent, Message};
use crate::config::KernelConfig;
use crate::error::{KernelError, KernelResult};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AgentMeta {
    name: String,
}

/// Nexus kernel: spawns agents, routes messages, and manages lifecycle.
#[derive(Clone)]
pub struct Kernel {
    config: KernelConfig,
    registry: Arc<DashMap<Uuid, mpsc::Sender<Message>>>,
    metas: Arc<DashMap<Uuid, AgentMeta>>,
    join_handles: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>,
    shutdown_token: CancellationToken,
}

impl Kernel {
    pub fn new(config: KernelConfig) -> Self {
        info!(mailbox_capacity=%config.mailbox_capacity, "kernel initialized");
        Self {
            config,
            registry: Arc::new(DashMap::new()),
            metas: Arc::new(DashMap::new()),
            join_handles: Arc::new(Mutex::new(HashMap::new())),
            shutdown_token: CancellationToken::new(),
        }
    }

    /// Spawn an agent into the kernel.
    ///
    /// Returns the agent id.
    pub async fn spawn_agent(&self, agent: Box<dyn Agent + 'static>) -> KernelResult<Uuid> {
        if self.shutdown_token.is_cancelled() {
            return Err(KernelError::ShuttingDown);
        }

        let id = agent.id();
        let name = agent.name().to_string();

        let (tx, mut rx) = mpsc::channel::<Message>(self.config.mailbox_capacity);

        // Register routing and metadata before starting the task.
        self.registry.insert(id, tx);
        self.metas.insert(id, AgentMeta { name: name.clone() });

        let registry = self.registry.clone();
        let metas = self.metas.clone();
        let shutdown = self.shutdown_token.child_token();
        let cfg = self.config.clone();

        let handle = tokio::spawn(async move {
            info!(agent_id=%id, agent_name=%name, "agent spawned");

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        debug!(agent_id=%id, "agent received shutdown");
                        break;
                    }
                    maybe_msg = rx.recv() => {
                        let Some(msg) = maybe_msg else { break };

                        let started = tokio::time::Instant::now();

                        // Protect the system against hung handlers.
                        let res = timeout(cfg.handle_timeout(), agent.handle_message(msg.clone())).await;

                        match res {
                            Ok(Some(response)) => {
                                debug!(agent_id=%id, elapsed_ms=%started.elapsed().as_millis(), "agent handled message with response");
                                // Best-effort: route response using the registry.
                                if let Some(dst) = registry.get(&response.to) {
                                    let tx = dst.value().clone();
                                    if let Err(e) = tx.send(response).await {
                                        warn!(agent_id=%id, err=%e.to_string(), "failed to route response (mailbox closed)");
                                    }
                                } else {
                                    warn!(agent_id=%id, to=%response.to, "response target not found");
                                }
                            }
                            Ok(None) => {
                                debug!(agent_id=%id, elapsed_ms=%started.elapsed().as_millis(), "agent handled message");
                            }
                            Err(_) => {
                                warn!(agent_id=%id, timeout_ms=%cfg.handle_timeout().as_millis(), "agent handler timeout");
                            }
                        }
                    }
                }
            }

            info!(agent_id=%id, agent_name=%name, "agent exiting");
            registry.remove(&id);
            metas.remove(&id);
        });

        self.join_handles.lock().await.insert(id, handle);
        Ok(id)
    }

    /// Send a message from one agent to another.
    ///
    /// Applies send timeout and returns structured errors.
    pub async fn send(&self, msg: Message) -> KernelResult<()> {
        if self.shutdown_token.is_cancelled() {
            return Err(KernelError::ShuttingDown);
        }

        let to = msg.to;

        let Some(entry) = self.registry.get(&to) else {
            return Err(KernelError::AgentNotFound(to));
        };

        let tx = entry.value().clone();
        drop(entry);

        match timeout(self.config.send_timeout(), tx.send(msg)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_closed)) => Err(KernelError::MailboxClosed(to)),
            Err(_elapsed) => Err(KernelError::SendTimeout(to)),
        }
    }

    /// Returns true if an agent id is currently registered.
    pub fn contains(&self, id: Uuid) -> bool {
        self.registry.contains_key(&id)
    }

    /// Gracefully shut down the kernel.
    ///
    /// This:
    /// - stops all agent loops (cancellation token)
    /// - drops routing senders (close mailboxes)
    /// - awaits agent tasks
    pub async fn shutdown(&self) {
        info!("kernel shutdown initiated");
        self.shutdown_token.cancel();

        // Drop routing senders so mailboxes close.
        self.registry.clear();

        // Await all tasks.
        let mut handles = self.join_handles.lock().await;
        for (id, handle) in handles.drain() {
            match handle.await {
                Ok(_) => debug!(agent_id=%id, "agent joined"),
                Err(e) => error!(agent_id=%id, err=%e.to_string(), "agent task panicked/aborted"),
            }
        }

        self.metas.clear();
        info!("kernel shutdown complete");
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new(KernelConfig::default())
    }
}
