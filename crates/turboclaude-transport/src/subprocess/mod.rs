//! Subprocess transport for CLI communication
//!
//! Implements bidirectional communication with the Claude Code CLI
//! via stdin/stdout JSON message passing.

pub mod cli;
pub mod process;

pub use cli::CliTransport;
pub use process::{ProcessConfig, ProcessHandle};
