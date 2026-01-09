use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Runtime knobs for the [`Kernel`](crate::Kernel).
///
/// These defaults are conservative for production:
/// - bounded mailboxes (backpressure)
/// - timeouts for sending + handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    /// Bounded mailbox size per agent.
    #[serde(default = "default_mailbox_capacity")]
    pub mailbox_capacity: usize,

    /// Maximum time we will wait to enqueue a message into an agent's mailbox (milliseconds).
    #[serde(default = "default_send_timeout_ms")]
    pub send_timeout_ms: u64,

    /// Maximum time an agent is allowed to spend processing a single message (milliseconds).
    #[serde(default = "default_handle_timeout_ms")]
    pub handle_timeout_ms: u64,
}

impl KernelConfig {
    /// Get send timeout as Duration
    pub fn send_timeout(&self) -> Duration {
        Duration::from_millis(self.send_timeout_ms)
    }

    /// Get handle timeout as Duration
    pub fn handle_timeout(&self) -> Duration {
        Duration::from_millis(self.handle_timeout_ms)
    }
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            mailbox_capacity: default_mailbox_capacity(),
            send_timeout_ms: default_send_timeout_ms(),
            handle_timeout_ms: default_handle_timeout_ms(),
        }
    }
}

fn default_mailbox_capacity() -> usize {
    1024
}
fn default_send_timeout_ms() -> u64 {
    500
}
fn default_handle_timeout_ms() -> u64 {
    30000
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (compact, json, pretty)
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "compact".to_string()
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Number of worker threads (0 = auto-detect CPU count)
    #[serde(default)]
    pub worker_threads: usize,

    /// Maximum blocking threads
    #[serde(default = "default_max_blocking_threads")]
    pub max_blocking_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0,
            max_blocking_threads: default_max_blocking_threads(),
        }
    }
}

fn default_max_blocking_threads() -> usize {
    512
}

/// Complete Nexus configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NexusConfig {
    #[serde(default)]
    pub kernel: KernelConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub runtime: RuntimeConfig,
}

impl NexusConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents =
            fs::read_to_string(path.as_ref()).context("Failed to read configuration file")?;

        let config: NexusConfig =
            toml::from_str(&contents).context("Failed to parse configuration file")?;

        Ok(config)
    }

    /// Load configuration from default locations
    ///
    /// Tries in order:
    /// 1. ./config.toml (current directory)
    /// 2. ~/.config/nexus/config.toml
    /// 3. /etc/nexus/config.toml
    ///
    /// Returns default config if no file is found
    pub fn from_default_locations() -> Self {
        let paths = [
            PathBuf::from("./config.toml"),
            dirs::config_dir()
                .map(|p| p.join("nexus/config.toml"))
                .unwrap_or_else(|| PathBuf::from("~/.config/nexus/config.toml")),
            PathBuf::from("/etc/nexus/config.toml"),
        ];

        for path in &paths {
            if path.exists() {
                if let Ok(config) = Self::from_file(path) {
                    tracing::info!("Loaded configuration from: {}", path.display());
                    return config;
                }
            }
        }

        tracing::info!("No configuration file found, using defaults");
        Self::default()
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("Failed to create configuration directory")?;
        }

        fs::write(path.as_ref(), contents).context("Failed to write configuration file")?;

        Ok(())
    }
}
