//! Error types for the Agent SDK
//!
//! Provides self-documenting errors with recovery guidance.
//! Errors implement `ErrorRecovery` trait which provides:
//! - Retrievability assessment (should this be retried?)
//! - Suggested user action (what should the user do?)
//! - Retry limits (how many times to try?)
//! - Backoff strategy (how long to wait between retries?)

use std::fmt;
use std::time::Duration;

/// Result type for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;

/// Backoff strategy for retrying operations
#[derive(Debug, Clone, Copy)]
pub enum BackoffStrategy {
    /// No backoff (don't retry)
    None,

    /// Linear backoff: base_ms * attempt_number
    Linear {
        /// Base delay in milliseconds
        base_ms: u64,
    },

    /// Exponential backoff: base_ms * 2^(attempt-1), capped at max_ms
    Exponential {
        /// Base delay in milliseconds
        base_ms: u64,
        /// Maximum delay cap in milliseconds
        max_ms: u64,
    },
}

impl BackoffStrategy {
    /// Calculate delay for a given attempt number (1-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            BackoffStrategy::None => None,
            BackoffStrategy::Linear { base_ms } => {
                let ms = base_ms * attempt as u64;
                Some(Duration::from_millis(ms))
            }
            BackoffStrategy::Exponential { base_ms, max_ms } => {
                let delay_ms =
                    base_ms.saturating_mul(2_u64.saturating_pow(attempt.saturating_sub(1)));
                let capped = delay_ms.min(*max_ms);
                Some(Duration::from_millis(capped))
            }
        }
    }
}

/// Error recovery guidance trait
///
/// Errors implement this trait to provide actionable recovery guidance.
/// This enables self-documenting errors and automatic retry logic.
pub trait ErrorRecovery {
    /// Whether this error should be retried
    fn is_retriable(&self) -> bool;

    /// User-facing action to take
    fn suggested_action(&self) -> &str;

    /// Maximum number of retry attempts (None = don't retry)
    fn max_retries(&self) -> Option<u32>;

    /// Backoff strategy for retries
    fn backoff_strategy(&self) -> BackoffStrategy;
}

/// Errors that can occur in agent operations
#[derive(Debug)]
pub enum AgentError {
    /// Transport error (subprocess communication, network issues)
    Transport(String),

    /// Protocol error (malformed requests/responses)
    Protocol(String),

    /// Permission denied (insufficient permissions)
    PermissionDenied(String),

    /// Hook error (hook callback failed)
    Hook(String),

    /// Configuration error (invalid config)
    Config(String),

    /// I/O error (file system)
    Io(std::io::Error),

    /// Generic error
    Other(String),
}

impl PartialEq for AgentError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Transport(a), Self::Transport(b)) => a == b,
            (Self::Protocol(a), Self::Protocol(b)) => a == b,
            (Self::PermissionDenied(a), Self::PermissionDenied(b)) => a == b,
            (Self::Hook(a), Self::Hook(b)) => a == b,
            (Self::Config(a), Self::Config(b)) => a == b,
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind() && a.to_string() == b.to_string(),
            (Self::Other(a), Self::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "Transport error: {}", msg),
            Self::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::Hook(msg) => write!(f, "Hook error: {}", msg),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<std::io::Error> for AgentError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl ErrorRecovery for AgentError {
    fn is_retriable(&self) -> bool {
        match self {
            // Transport errors are transient (network, subprocess restarts)
            Self::Transport(_) => true,

            // Protocol errors are permanent (bad request)
            Self::Protocol(_) => false,

            // Permission errors are permanent (user action required)
            Self::PermissionDenied(_) => false,

            // Hook errors are permanent (callback broken)
            Self::Hook(_) => false,

            // Config errors are permanent (fix config)
            Self::Config(_) => false,

            // I/O errors might be transient (e.g., Interrupted)
            Self::Io(err) => err.kind() == std::io::ErrorKind::Interrupted,

            // Other errors are permanent by default
            Self::Other(_) => false,
        }
    }

    fn suggested_action(&self) -> &str {
        match self {
            Self::Transport(msg) => {
                if msg.contains("timeout") || msg.contains("deadline") {
                    "Request timed out. Session will auto-reconnect. \
                    Try again with fewer tokens or simpler query."
                } else if msg.contains("closed") || msg.contains("disconnect") {
                    "Connection lost. Session will auto-reconnect on next query. \
                    Check subprocess health if reconnection fails."
                } else {
                    "Transport error detected. Session will auto-reconnect. \
                    Check process logs for details."
                }
            }
            Self::Protocol(msg) => {
                if msg.contains("max_tokens") {
                    "Query exceeds max_tokens limit. Reduce max_tokens \
                    or simplify your query."
                } else if msg.contains("invalid") {
                    "Invalid request structure. Check message format \
                    and retry with valid input."
                } else {
                    "Protocol violation detected. Review your request \
                    against API specification."
                }
            }
            Self::PermissionDenied(msg) => {
                if msg.contains("edit") {
                    "Edit permission denied. Set PermissionMode to \
                    AcceptEdits or BypassPermissions."
                } else {
                    "Permission denied. Check session configuration \
                    and permissions settings."
                }
            }
            Self::Hook(msg) => {
                if msg.contains("timeout") {
                    "Hook callback timed out. Optimize hook logic \
                    or remove if unnecessary."
                } else {
                    "Hook callback failed. Check hook implementation \
                    for errors."
                }
            }
            Self::Config(msg) => format!(
                "Configuration error: {}. Fix config and \
                create new session.",
                msg
            )
            .leak(),
            Self::Io(err) => match err.kind() {
                std::io::ErrorKind::NotFound => "File not found. Check file path exists.",
                std::io::ErrorKind::PermissionDenied => {
                    "File permission denied. Check file permissions."
                }
                std::io::ErrorKind::Interrupted => "I/O operation interrupted. Will auto-retry.",
                _ => {
                    "I/O error occurred. Check file system health \
                        and disk space."
                }
            },
            Self::Other(_) => {
                "Unknown error occurred. Check error message \
                and logs for details."
            }
        }
    }

    fn max_retries(&self) -> Option<u32> {
        match self {
            // Transport errors: retry up to 5 times
            Self::Transport(_) => Some(5),

            // I/O interrupts: retry up to 3 times
            Self::Io(err) if err.kind() == std::io::ErrorKind::Interrupted => Some(3),

            // Everything else: don't retry
            _ => None,
        }
    }

    fn backoff_strategy(&self) -> BackoffStrategy {
        match self {
            // Transport: exponential backoff (500ms base, 60s cap)
            Self::Transport(_) => BackoffStrategy::Exponential {
                base_ms: 500,
                max_ms: 60_000,
            },

            // I/O: linear backoff (1s base)
            Self::Io(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                BackoffStrategy::Linear { base_ms: 1000 }
            }

            // Everything else: no backoff
            _ => BackoffStrategy::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_retriable() {
        let err = AgentError::Transport("connection lost".to_string());
        assert!(err.is_retriable());
        assert_eq!(err.max_retries(), Some(5));
    }

    #[test]
    fn test_protocol_error_not_retriable() {
        let err = AgentError::Protocol("invalid request".to_string());
        assert!(!err.is_retriable());
        assert_eq!(err.max_retries(), None);
    }

    #[test]
    fn test_permission_error_not_retriable() {
        let err = AgentError::PermissionDenied("denied".to_string());
        assert!(!err.is_retriable());
    }

    #[test]
    fn test_backoff_strategy_linear() {
        let strategy = BackoffStrategy::Linear { base_ms: 100 };
        assert_eq!(
            strategy.delay_for_attempt(1),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            strategy.delay_for_attempt(2),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            strategy.delay_for_attempt(3),
            Some(Duration::from_millis(300))
        );
    }

    #[test]
    fn test_backoff_strategy_exponential() {
        let strategy = BackoffStrategy::Exponential {
            base_ms: 100,
            max_ms: 10_000,
        };
        assert_eq!(
            strategy.delay_for_attempt(1),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            strategy.delay_for_attempt(2),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            strategy.delay_for_attempt(3),
            Some(Duration::from_millis(400))
        );
        // Should cap at max_ms
        assert_eq!(
            strategy.delay_for_attempt(10),
            Some(Duration::from_millis(10_000))
        );
    }

    #[test]
    fn test_transport_error_suggested_action() {
        let err = AgentError::Transport("timeout".to_string());
        let action = err.suggested_action();
        assert!(action.contains("timed out") || action.contains("timeout"));
    }

    #[test]
    fn test_config_error_not_retriable() {
        let err = AgentError::Config("invalid setting".to_string());
        assert!(!err.is_retriable());
        assert_eq!(err.max_retries(), None);
    }
}
