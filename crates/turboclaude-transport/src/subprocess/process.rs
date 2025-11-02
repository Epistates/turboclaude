//! Process management for CLI subprocess

use crate::error::{Result, TransportError};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufWriter};
use tokio::process::{Child as TokioChild, Command};

/// Configuration for spawning a CLI process
#[derive(Clone, Debug)]
pub struct ProcessConfig {
    /// Path to the CLI executable
    pub cli_path: String,

    /// Arguments to pass to the CLI
    pub args: Vec<String>,

    /// Environment variables to set
    pub env: HashMap<String, String>,

    /// Process timeout
    pub timeout: std::time::Duration,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            cli_path: "claude".to_string(),
            args: vec!["agent".to_string()],
            env: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl ProcessConfig {
    /// Create a new process configuration
    pub fn new(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            args: vec!["agent".to_string()],
            env: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set an environment variable
    ///
    /// # Security Note
    ///
    /// When the process is spawned, the parent process's environment is cleared
    /// and only the variables explicitly set here are passed to the child process.
    /// This prevents unintended information leakage.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Handle to a running CLI process
pub struct ProcessHandle {
    process: std::sync::Arc<tokio::sync::Mutex<TokioChild>>,
    stdin: BufWriter<tokio::process::ChildStdin>,
    stdout: BufReader<tokio::process::ChildStdout>,
    config: ProcessConfig,
}

impl ProcessHandle {
    /// Spawn a new CLI process
    ///
    /// # Security
    ///
    /// The spawned process's environment is isolated from the parent process.
    /// Only environment variables explicitly set via [ProcessConfig::with_env]
    /// are passed to the child process. This prevents unintended leakage of
    /// sensitive information (e.g., API keys, credentials) from the parent.
    pub async fn spawn(config: ProcessConfig) -> Result<Self> {
        let mut cmd = Command::new(&config.cli_path);

        // Add arguments
        for arg in &config.args {
            cmd.arg(arg);
        }

        // SECURITY: Clear inherited environment variables
        // Only explicitly set variables are passed to the child
        cmd.env_clear();

        // Add explicitly configured environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        // Spawn process
        let mut process = cmd
            .spawn()
            .map_err(|e| TransportError::Process(format!("Failed to spawn CLI: {}", e)))?;

        // Get stdin/stdout
        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| TransportError::Process("Failed to get stdin".to_string()))?;
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| TransportError::Process("Failed to get stdout".to_string()))?;

        Ok(Self {
            process: std::sync::Arc::new(tokio::sync::Mutex::new(process)),
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            config,
        })
    }

    /// Send a JSON message to the process
    pub async fn send_message(&mut self, message: serde_json::Value) -> Result<()> {
        let json = serde_json::to_string(&message)
            .map_err(|e| TransportError::Serialization(e.to_string()))?;

        // Write message followed by newline
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        Ok(())
    }

    /// Receive a JSON message from the process
    pub async fn recv_message(&mut self) -> Result<Option<serde_json::Value>> {
        let mut line = String::new();

        // Read line from stdout
        match self.stdout.read_line(&mut line).await? {
            0 => Ok(None), // EOF
            _ => {
                let message = serde_json::from_str(line.trim())
                    .map_err(|e| TransportError::Serialization(e.to_string()))?;
                Ok(Some(message))
            }
        }
    }

    /// Check if the process is still alive
    pub async fn is_alive(&self) -> bool {
        let mut process = self.process.lock().await;
        process.try_wait().ok().flatten().is_none()
    }

    /// Kill the process
    pub async fn kill(&self) -> Result<()> {
        let mut process = self.process.lock().await;
        process
            .kill()
            .await
            .map_err(|e| TransportError::Process(format!("Failed to kill process: {}", e)))
    }

    /// Get the process configuration
    pub fn config(&self) -> &ProcessConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config_default() {
        let config = ProcessConfig::default();
        assert_eq!(config.cli_path, "claude");
        assert!(config.args.contains(&"agent".to_string()));
    }

    #[test]
    fn test_process_config_builder() {
        let config = ProcessConfig::new("my-claude")
            .with_arg("--verbose")
            .with_env("API_KEY", "sk-123")
            .with_timeout(std::time::Duration::from_secs(60));

        assert_eq!(config.cli_path, "my-claude");
        assert!(config.args.contains(&"--verbose".to_string()));
        assert_eq!(config.env.get("API_KEY"), Some(&"sk-123".to_string()));
        assert_eq!(config.timeout, std::time::Duration::from_secs(60));
    }
}
