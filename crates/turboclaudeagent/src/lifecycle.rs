//! Session lifecycle management with events and RAII cleanup
//!
//! Provides:
//! - SessionEvent enum for lifecycle visibility
//! - Lifecycle callbacks for observability
//! - SessionGuard for automatic cleanup (RAII pattern)
//!
//! # Example
//!
//! ```ignore
//! // Option 1: Track lifecycle events
//! let session = AgentSession::new_with_lifecycle(config, |event| {
//!     match event {
//!         SessionEvent::Created { session_id } => println!("Session born: {}", session_id),
//!         SessionEvent::Closed { session_id } => println!("Session died: {}", session_id),
//!         _ => {}
//!     }
//! }).await?;
//!
//! // Option 2: RAII auto-cleanup
//! {
//!     let session = Arc::new(AgentSession::new(config).await?);
//!     let _guard = session.into_guard();
//!     // ... use session ...
//! } // Auto-closed on drop
//! ```

use serde::{Deserialize, Serialize};

/// Lifecycle events for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// Session was created
    Created {
        /// Unique session identifier
        session_id: String,
    },

    /// Session was forked (parent spawned child)
    Forked {
        /// Parent session ID
        parent_id: String,
        /// Child session ID
        child_id: String,
    },

    /// Session is about to close
    Closing {
        /// Session ID being closed
        session_id: String,
    },

    /// Session has been closed
    Closed {
        /// Session ID that was closed
        session_id: String,
    },

    /// Session is attempting to reconnect
    Reconnecting {
        /// Session ID
        session_id: String,
        /// Attempt number (1-indexed)
        attempt: u32,
    },

    /// Session successfully reconnected
    Reconnected {
        /// Session ID
        session_id: String,
    },

    /// Session encountered an error
    Error {
        /// Session ID
        session_id: String,
        /// Error description
        error: String,
    },

    /// Session queries increased context usage
    ContextUsageIncreased {
        /// Session ID
        session_id: String,
        /// Estimated current token usage
        tokens_used: usize,
        /// Target token limit
        target_tokens: usize,
    },

    /// Context was pruned to manage token usage
    ContextPruned {
        /// Session ID
        session_id: String,
        /// Messages removed
        messages_removed: usize,
        /// Tokens freed
        tokens_freed: usize,
    },
}

impl SessionEvent {
    /// Get the session ID associated with this event
    pub fn session_id(&self) -> &str {
        match self {
            SessionEvent::Created { session_id } => session_id,
            SessionEvent::Forked { child_id, .. } => child_id,
            SessionEvent::Closing { session_id } => session_id,
            SessionEvent::Closed { session_id } => session_id,
            SessionEvent::Reconnecting { session_id, .. } => session_id,
            SessionEvent::Reconnected { session_id } => session_id,
            SessionEvent::Error { session_id, .. } => session_id,
            SessionEvent::ContextUsageIncreased { session_id, .. } => session_id,
            SessionEvent::ContextPruned { session_id, .. } => session_id,
        }
    }

    /// Get a human-readable description of this event
    pub fn description(&self) -> String {
        match self {
            SessionEvent::Created { .. } => "Session created".to_string(),
            SessionEvent::Forked {
                parent_id,
                child_id,
            } => {
                format!("Session {} forked from {}", child_id, parent_id)
            }
            SessionEvent::Closing { .. } => "Session closing".to_string(),
            SessionEvent::Closed { .. } => "Session closed".to_string(),
            SessionEvent::Reconnecting { attempt, .. } => {
                format!("Reconnecting (attempt {})", attempt)
            }
            SessionEvent::Reconnected { .. } => "Reconnected successfully".to_string(),
            SessionEvent::Error { error, .. } => format!("Error: {}", error),
            SessionEvent::ContextUsageIncreased {
                tokens_used,
                target_tokens,
                ..
            } => {
                let percent = (*tokens_used as f64 / *target_tokens as f64) * 100.0;
                format!(
                    "Context usage: {:.1}% ({}/{})",
                    percent, tokens_used, target_tokens
                )
            }
            SessionEvent::ContextPruned {
                messages_removed,
                tokens_freed,
                ..
            } => {
                format!(
                    "Context pruned: {} messages, {} tokens freed",
                    messages_removed, tokens_freed
                )
            }
        }
    }
}

/// RAII guard for automatic session cleanup
///
/// Ensures that a session is properly closed when dropped,
/// even if an error occurs or the scope is exited early.
pub struct SessionGuard {
    // We store a boxed closure instead of a concrete type to avoid generic constraints
    on_drop: Option<Box<dyn FnOnce() + Send>>,
}

impl SessionGuard {
    /// Create a new session guard with a cleanup function
    pub fn new<F>(on_drop: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            on_drop: Some(Box::new(on_drop)),
        }
    }

    /// Consume the guard without running cleanup
    ///
    /// Useful if you want to transfer ownership elsewhere
    pub fn into_inner(mut self) -> Option<Box<dyn FnOnce() + Send>> {
        self.on_drop.take()
    }

    /// Manually run cleanup and consume the guard
    pub fn cleanup(mut self) {
        if let Some(cleanup) = self.on_drop.take() {
            cleanup();
        }
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Some(cleanup) = self.on_drop.take() {
            cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_event_created() {
        let event = SessionEvent::Created {
            session_id: "sess_123".to_string(),
        };
        assert_eq!(event.session_id(), "sess_123");
        assert!(event.description().contains("created"));
    }

    #[test]
    fn test_session_event_forked() {
        let event = SessionEvent::Forked {
            parent_id: "parent".to_string(),
            child_id: "child".to_string(),
        };
        assert_eq!(event.session_id(), "child");
        assert!(event.description().contains("forked"));
    }

    #[test]
    fn test_session_event_context_usage() {
        let event = SessionEvent::ContextUsageIncreased {
            session_id: "sess_123".to_string(),
            tokens_used: 3000,
            target_tokens: 4000,
        };
        let desc = event.description();
        assert!(desc.contains("75.0%"));
        assert!(desc.contains("3000"));
    }

    #[test]
    fn test_session_guard_cleanup() {
        let cleaned_up = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = cleaned_up.clone();

        {
            let guard = SessionGuard::new(move || {
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            // Guard dropped here
            drop(guard);
        }

        assert!(cleaned_up.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_session_guard_manual_cleanup() {
        let cleaned_up = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = cleaned_up.clone();

        let guard = SessionGuard::new(move || {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        assert!(!cleaned_up.load(std::sync::atomic::Ordering::SeqCst));
        guard.cleanup();
        assert!(cleaned_up.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_session_guard_into_inner() {
        let guard = SessionGuard::new(|| {
            // noop
        });

        let inner = guard.into_inner();
        assert!(inner.is_some());
    }

    #[test]
    fn test_all_event_types_have_session_id() {
        let events = vec![
            SessionEvent::Created {
                session_id: "1".to_string(),
            },
            SessionEvent::Forked {
                parent_id: "1".to_string(),
                child_id: "2".to_string(),
            },
            SessionEvent::Closing {
                session_id: "1".to_string(),
            },
            SessionEvent::Closed {
                session_id: "1".to_string(),
            },
            SessionEvent::Reconnecting {
                session_id: "1".to_string(),
                attempt: 1,
            },
            SessionEvent::Reconnected {
                session_id: "1".to_string(),
            },
            SessionEvent::Error {
                session_id: "1".to_string(),
                error: "test".to_string(),
            },
            SessionEvent::ContextUsageIncreased {
                session_id: "1".to_string(),
                tokens_used: 100,
                target_tokens: 1000,
            },
            SessionEvent::ContextPruned {
                session_id: "1".to_string(),
                messages_removed: 5,
                tokens_freed: 100,
            },
        ];

        for event in events {
            assert!(!event.session_id().is_empty());
            assert!(!event.description().is_empty());
        }
    }
}
