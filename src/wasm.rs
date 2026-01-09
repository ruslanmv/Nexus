//! WebAssembly agent isolation support
//!
//! This module provides secure execution of agents within WebAssembly sandboxes,
//! enabling deterministic, portable, and isolated agent execution.

#[cfg(feature = "wasm")]
use anyhow::{Context, Result};
#[cfg(feature = "wasm")]
use async_trait::async_trait;
#[cfg(feature = "wasm")]
use std::sync::Arc;
#[cfg(feature = "wasm")]
use tokio::sync::Mutex;
#[cfg(feature = "wasm")]
use uuid::Uuid;
#[cfg(feature = "wasm")]
use wasmtime::*;
#[cfg(feature = "wasm")]
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

#[cfg(feature = "wasm")]
use crate::{Agent, Message};

/// Configuration for WASM agent runtime
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
pub struct WasmConfig {
    /// Maximum memory pages (64KB per page)
    pub max_memory_pages: u32,
    /// Enable WASI support
    pub enable_wasi: bool,
    /// Fuel limit for execution (prevents infinite loops)
    pub fuel_limit: u64,
}

#[cfg(feature = "wasm")]
impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 256, // 16MB default
            enable_wasi: true,
            fuel_limit: 100_000_000,
        }
    }
}

/// WebAssembly agent runtime
///
/// Executes agents in isolated WASM sandboxes with controlled resource access.
#[cfg(feature = "wasm")]
pub struct WasmAgent {
    id: Uuid,
    name: String,
    engine: Engine,
    module: Module,
    store: Arc<Mutex<Store<WasmAgentState>>>,
    instance: Arc<Mutex<Option<Instance>>>,
}

#[cfg(feature = "wasm")]
struct WasmAgentState {
    _wasi: Option<WasiCtx>,
    message_buffer: Vec<u8>,
    limits: StoreLimits,
}

#[cfg(feature = "wasm")]
impl WasmAgent {
    /// Create a new WASM agent from bytecode
    pub fn new(name: String, wasm_bytes: &[u8], config: WasmConfig) -> Result<Self> {
        // Configure engine with resource limits
        let mut engine_config = Config::new();
        engine_config
            .consume_fuel(true)
            .max_wasm_stack(1024 * 1024) // 1MB stack
            .async_support(true);

        let engine = Engine::new(&engine_config).context("Failed to create WASM engine")?;

        let module = Module::new(&engine, wasm_bytes).context("Failed to compile WASM module")?;

        // Create WASI context if enabled
        let wasi = if config.enable_wasi {
            Some(
                WasiCtxBuilder::new()
                    .inherit_stdin()
                    .inherit_stdout()
                    .inherit_stderr()
                    .build(),
            )
        } else {
            None
        };

        let limits = StoreLimitsBuilder::new()
            .memory_size(config.max_memory_pages as usize * 65536)
            .build();

        let state = WasmAgentState {
            _wasi: wasi,
            message_buffer: Vec::new(),
            limits,
        };

        let mut store = Store::new(&engine, state);
        store.limiter(|state| &mut state.limits);

        // Set fuel limit
        store
            .set_fuel(config.fuel_limit)
            .context("Failed to set fuel")?;

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            engine,
            module,
            store: Arc::new(Mutex::new(store)),
            instance: Arc::new(Mutex::new(None)),
        })
    }

    /// Initialize the WASM instance
    pub async fn initialize(&self) -> Result<()> {
        let mut store = self.store.lock().await;

        // Create linker
        let mut linker = Linker::new(&self.engine);

        // Add custom host functions
        linker
            .func_wrap(
                "env",
                "log",
                |_caller: Caller<'_, WasmAgentState>, ptr: i32, len: i32| {
                    tracing::info!("WASM agent log: ptr={}, len={}", ptr, len);
                },
            )
            .context("Failed to link log function")?;

        let instance = linker
            .instantiate_async(&mut *store, &self.module)
            .await
            .context("Failed to instantiate WASM module")?;

        *self.instance.lock().await = Some(instance);

        Ok(())
    }

    /// Call a WASM function
    async fn call_function(&self, name: &str, params: &[Val]) -> Result<Vec<Val>> {
        let mut store = self.store.lock().await;
        let instance = *self
            .instance
            .lock()
            .await
            .as_ref()
            .context("WASM instance not initialized")?;

        let func = instance
            .get_func(&mut *store, name)
            .context(format!("Function '{}' not found", name))?;

        let mut results = vec![Val::I32(0); func.ty(&*store).results().len()];

        func.call_async(&mut *store, params, &mut results)
            .await
            .context(format!("Failed to call function '{}'", name))?;

        Ok(results)
    }
}

#[cfg(feature = "wasm")]
#[async_trait]
impl Agent for WasmAgent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn handle_message(&self, msg: Message) -> Option<Message> {
        // Serialize message to JSON
        let msg_json = match serde_json::to_string(&msg) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failed to serialize message: {}", e);
                return None;
            }
        };

        // Store message in buffer
        {
            let mut store = self.store.lock().await;
            store.data_mut().message_buffer = msg_json.into_bytes();
        }

        // Call WASM handler function
        // This is a simplified implementation - real impl would pass memory pointers
        match self.call_function("handle_message", &[]).await {
            Ok(_results) => {
                // In a real implementation, we'd read the response from WASM memory
                tracing::debug!("WASM agent {} handled message", self.name);
                None
            }
            Err(e) => {
                tracing::error!("WASM agent {} error: {}", self.name, e);
                None
            }
        }
    }
}

/// Builder for WASM agents
#[cfg(feature = "wasm")]
pub struct WasmAgentBuilder {
    name: String,
    wasm_bytes: Vec<u8>,
    config: WasmConfig,
}

#[cfg(feature = "wasm")]
impl WasmAgentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            wasm_bytes: Vec::new(),
            config: WasmConfig::default(),
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.wasm_bytes = bytes;
        self
    }

    pub fn with_file(mut self, path: impl AsRef<std::path::Path>) -> Result<Self> {
        self.wasm_bytes = std::fs::read(path)?;
        Ok(self)
    }

    pub fn with_config(mut self, config: WasmConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<WasmAgent> {
        WasmAgent::new(self.name, &self.wasm_bytes, self.config)
    }
}

#[cfg(test)]
#[cfg(feature = "wasm")]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_config_default() {
        let config = WasmConfig::default();
        assert_eq!(config.max_memory_pages, 256);
        assert!(config.enable_wasi);
        assert_eq!(config.fuel_limit, 100_000_000);
    }
}
