//! CLI transport implementation
//!
//! Manages bidirectional communication with Claude Code CLI process.
//! Handles JSON message serialization/deserialization over stdin/stdout.

use crate::error::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use super::process::{ProcessConfig, ProcessHandle};

/// CLI transport for Claude Code agent communication
///
/// Spawns and manages the Claude Code CLI process with bidirectional
/// JSON message passing.
pub struct CliTransport {
    process: Arc<Mutex<ProcessHandle>>,
}

impl CliTransport {
    /// Create a new CLI transport by spawning the Claude CLI process
    pub async fn spawn(config: ProcessConfig) -> Result<Self> {
        let process = ProcessHandle::spawn(config).await?;
        Ok(Self {
            process: Arc::new(Mutex::new(process)),
        })
    }

    /// Send a message to the CLI process
    pub async fn send_message(&self, message: serde_json::Value) -> Result<()> {
        let mut process = self.process.lock().await;
        process.send_message(message).await
    }

    /// Receive a message from the CLI process
    pub async fn recv_message(&self) -> Result<Option<serde_json::Value>> {
        let mut process = self.process.lock().await;
        process.recv_message().await
    }

    /// Check if the process is still alive
    pub async fn is_alive(&self) -> bool {
        let process = self.process.lock().await;
        process.is_alive().await
    }

    /// Terminate the CLI process
    pub async fn kill(&self) -> Result<()> {
        let process = self.process.lock().await;
        process.kill().await
    }

    /// Get process configuration
    pub async fn config(&self) -> ProcessConfig {
        let process = self.process.lock().await;
        process.config().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_transport_config_creation() {
        let config = ProcessConfig::default();
        assert_eq!(config.cli_path, "claude");
    }
}
