//! Distributed agent registry
//!
//! This module provides distributed coordination for Nexus agents across multiple nodes,
//! enabling cluster-wide agent discovery and routing.

use crate::KernelResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Node information in a Nexus cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique node identifier
    pub id: Uuid,
    /// Node address (host:port)
    pub address: SocketAddr,
    /// Node name/hostname
    pub name: String,
    /// Timestamp of last heartbeat
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// Node status
    pub status: NodeStatus,
    /// Number of agents on this node
    pub agent_count: usize,
}

/// Node status in the cluster
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is healthy and accepting work
    Healthy,
    /// Node is degraded but still functioning
    Degraded,
    /// Node is not responding
    Unreachable,
    /// Node is shutting down
    ShuttingDown,
}

/// Agent registration in the distributed registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    /// Agent ID
    pub id: Uuid,
    /// Agent name
    pub name: String,
    /// Node hosting this agent
    pub node_id: Uuid,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Distributed agent registry
///
/// Provides cluster-wide agent discovery and routing across multiple Nexus nodes.
pub struct DistributedRegistry {
    /// Local node information
    node_info: Arc<RwLock<NodeInfo>>,
    /// Known nodes in the cluster
    nodes: Arc<DashMap<Uuid, NodeInfo>>,
    /// Agent-to-node mapping
    agents: Arc<DashMap<Uuid, AgentRegistration>>,
    /// Local agent count
    local_agent_count: Arc<RwLock<usize>>,
}

impl DistributedRegistry {
    /// Create a new distributed registry
    pub fn new(node_name: String, bind_address: SocketAddr) -> Self {
        let node_info = NodeInfo {
            id: Uuid::new_v4(),
            address: bind_address,
            name: node_name,
            last_heartbeat: chrono::Utc::now(),
            status: NodeStatus::Healthy,
            agent_count: 0,
        };

        info!(
            node_id = %node_info.id,
            address = %bind_address,
            "Created distributed registry node"
        );

        Self {
            node_info: Arc::new(RwLock::new(node_info)),
            nodes: Arc::new(DashMap::new()),
            agents: Arc::new(DashMap::new()),
            local_agent_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Get local node information
    pub async fn node_info(&self) -> NodeInfo {
        self.node_info.read().await.clone()
    }

    /// Register a local agent
    pub async fn register_agent(&self, id: Uuid, name: String) -> KernelResult<()> {
        let node_info = self.node_info.read().await;

        let registration = AgentRegistration {
            id,
            name: name.clone(),
            node_id: node_info.id,
            registered_at: chrono::Utc::now(),
        };

        self.agents.insert(id, registration);

        let mut count = self.local_agent_count.write().await;
        *count += 1;

        debug!(agent_id = %id, agent_name = %name, "Registered agent in distributed registry");

        Ok(())
    }

    /// Unregister a local agent
    pub async fn unregister_agent(&self, id: Uuid) -> KernelResult<()> {
        if let Some((_, _registration)) = self.agents.remove(&id) {
            let mut count = self.local_agent_count.write().await;
            if *count > 0 {
                *count -= 1;
            }

            debug!(agent_id = %id, "Unregistered agent from distributed registry");
        }

        Ok(())
    }

    /// Find which node hosts a given agent
    pub fn find_agent_node(&self, agent_id: Uuid) -> Option<NodeInfo> {
        self.agents
            .get(&agent_id)
            .and_then(|reg| self.nodes.get(&reg.node_id).map(|n| n.clone()))
    }

    /// Check if an agent is on the local node
    pub async fn is_local_agent(&self, agent_id: Uuid) -> bool {
        let node_info = self.node_info.read().await;

        self.agents
            .get(&agent_id)
            .map(|reg| reg.node_id == node_info.id)
            .unwrap_or(false)
    }

    /// Add a peer node to the cluster
    pub fn add_node(&self, node: NodeInfo) {
        info!(node_id = %node.id, address = %node.address, "Adding peer node to cluster");
        self.nodes.insert(node.id, node);
    }

    /// Remove a node from the cluster
    pub fn remove_node(&self, node_id: Uuid) {
        if let Some((_, _node)) = self.nodes.remove(&node_id) {
            warn!(node_id = %node_id, "Removed node from cluster");

            // Remove agents from removed node
            self.agents.retain(|_, reg| reg.node_id != node_id);
        }
    }

    /// Update node heartbeat
    pub fn update_node_heartbeat(&self, node_id: Uuid) {
        if let Some(mut node) = self.nodes.get_mut(&node_id) {
            node.last_heartbeat = chrono::Utc::now();
            node.status = NodeStatus::Healthy;
        }
    }

    /// Get all nodes in the cluster
    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all registered agents
    pub fn get_all_agents(&self) -> Vec<AgentRegistration> {
        self.agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get cluster statistics
    pub async fn stats(&self) -> ClusterStats {
        let node_count = self.nodes.len() + 1; // +1 for local node
        let agent_count = self.agents.len();
        let local_agents = *self.local_agent_count.read().await;

        ClusterStats {
            node_count,
            agent_count,
            local_agent_count: local_agents,
        }
    }

    /// Perform health check on cluster nodes
    pub async fn health_check(&self) -> Vec<(Uuid, NodeStatus)> {
        let mut results = Vec::new();
        let mut updates = Vec::new();
        let now = chrono::Utc::now();
        let timeout_threshold = chrono::Duration::seconds(30);

        // First pass: collect status
        for entry in self.nodes.iter() {
            let node = entry.value();
            let age = now - node.last_heartbeat;

            let status = if age > timeout_threshold {
                NodeStatus::Unreachable
            } else {
                node.status
            };

            results.push((node.id, status));

            // Track nodes that need status updates
            if status != node.status {
                updates.push((node.id, status));
            }
        }

        // Second pass: update status
        for (node_id, new_status) in updates {
            if let Some(mut node_mut) = self.nodes.get_mut(&node_id) {
                node_mut.status = new_status;
            }
        }

        results
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    /// Total number of nodes in cluster
    pub node_count: usize,
    /// Total number of agents across all nodes
    pub agent_count: usize,
    /// Number of agents on local node
    pub local_agent_count: usize,
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Node name
    pub node_name: String,
    /// Bind address for cluster communication
    pub bind_address: String,
    /// Seed nodes to connect to
    pub seed_nodes: Vec<String>,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
    /// Node timeout (seconds)
    pub node_timeout: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node_name: format!("nexus-node-{}", Uuid::new_v4()),
            bind_address: "127.0.0.1:7001".to_string(),
            seed_nodes: Vec::new(),
            heartbeat_interval: 10,
            node_timeout: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_registry_creation() {
        let registry =
            DistributedRegistry::new("test-node".to_string(), "127.0.0.1:7001".parse().unwrap());

        let info = registry.node_info().await;
        assert_eq!(info.name, "test-node");
        assert_eq!(info.status, NodeStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let registry =
            DistributedRegistry::new("test-node".to_string(), "127.0.0.1:7001".parse().unwrap());

        let agent_id = Uuid::new_v4();
        registry
            .register_agent(agent_id, "test-agent".to_string())
            .await
            .unwrap();

        assert!(registry.is_local_agent(agent_id).await);

        registry.unregister_agent(agent_id).await.unwrap();

        assert!(!registry.is_local_agent(agent_id).await);
    }

    #[tokio::test]
    async fn test_cluster_stats() {
        let registry =
            DistributedRegistry::new("test-node".to_string(), "127.0.0.1:7001".parse().unwrap());

        let agent_id = Uuid::new_v4();
        registry
            .register_agent(agent_id, "test-agent".to_string())
            .await
            .unwrap();

        let stats = registry.stats().await;
        assert_eq!(stats.node_count, 1);
        assert_eq!(stats.agent_count, 1);
        assert_eq!(stats.local_agent_count, 1);
    }
}
