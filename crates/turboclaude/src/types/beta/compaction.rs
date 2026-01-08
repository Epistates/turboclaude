//! Compaction Control Types
//!
//! Types for controlling automatic conversation summarization/compaction
//! to manage context window limits in long-running conversations.

use serde::{Deserialize, Serialize};

/// Compaction control configuration
///
/// Controls how the tool runner handles conversation history when
/// approaching context window limits. When enabled, older messages
/// are automatically summarized to make room for new content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompactionControl {
    /// Whether compaction is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Target token count to compact to (when compaction is triggered)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tokens: Option<u32>,
    /// Token threshold that triggers compaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_tokens: Option<u32>,
    /// Number of recent messages to preserve during compaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_recent_messages: Option<usize>,
}

impl CompactionControl {
    /// Create a new disabled compaction control
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Create a new enabled compaction control with defaults
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            target_tokens: Some(100_000),
            threshold_tokens: Some(180_000),
            preserve_recent_messages: Some(5),
        }
    }

    /// Set target token count for compaction
    pub fn with_target_tokens(mut self, tokens: u32) -> Self {
        self.target_tokens = Some(tokens);
        self
    }

    /// Set threshold that triggers compaction
    pub fn with_threshold_tokens(mut self, tokens: u32) -> Self {
        self.threshold_tokens = Some(tokens);
        self
    }

    /// Set number of recent messages to preserve
    pub fn with_preserve_recent(mut self, count: usize) -> Self {
        self.preserve_recent_messages = Some(count);
        self
    }

    /// Check if compaction should be triggered based on current token count
    pub fn should_compact(&self, current_tokens: u32) -> bool {
        if !self.enabled {
            return false;
        }
        match self.threshold_tokens {
            Some(threshold) => current_tokens >= threshold,
            None => false,
        }
    }
}

/// Summary of what was compacted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSummary {
    /// Number of messages that were compacted
    pub messages_compacted: usize,
    /// Number of tokens before compaction
    pub tokens_before: u32,
    /// Number of tokens after compaction
    pub tokens_after: u32,
    /// The summary text generated for compacted content
    pub summary_text: String,
}

/// Result of a compaction operation
#[derive(Debug, Clone)]
pub enum CompactionResult {
    /// Compaction was not needed
    NotNeeded,
    /// Compaction was skipped (disabled or insufficient messages)
    Skipped {
        /// Reason why compaction was skipped
        reason: String,
    },
    /// Compaction was performed successfully
    Compacted(CompactionSummary),
    /// Compaction failed
    Failed {
        /// Error that caused compaction to fail
        error: String,
    },
}

impl CompactionResult {
    /// Check if compaction was performed
    pub fn was_compacted(&self) -> bool {
        matches!(self, Self::Compacted(_))
    }

    /// Get the compaction summary if compaction was performed
    pub fn summary(&self) -> Option<&CompactionSummary> {
        match self {
            Self::Compacted(summary) => Some(summary),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compaction_control_disabled() {
        let control = CompactionControl::disabled();
        assert!(!control.enabled);
        assert!(!control.should_compact(200_000));
    }

    #[test]
    fn test_compaction_control_enabled() {
        let control = CompactionControl::enabled();
        assert!(control.enabled);
        assert_eq!(control.target_tokens, Some(100_000));
        assert_eq!(control.threshold_tokens, Some(180_000));
        assert_eq!(control.preserve_recent_messages, Some(5));
    }

    #[test]
    fn test_should_compact() {
        let control = CompactionControl::enabled();

        // Below threshold
        assert!(!control.should_compact(100_000));

        // At threshold
        assert!(control.should_compact(180_000));

        // Above threshold
        assert!(control.should_compact(200_000));
    }

    #[test]
    fn test_compaction_result() {
        let summary = CompactionSummary {
            messages_compacted: 10,
            tokens_before: 200_000,
            tokens_after: 100_000,
            summary_text: "Previous conversation summary...".to_string(),
        };

        let result = CompactionResult::Compacted(summary);
        assert!(result.was_compacted());
        assert!(result.summary().is_some());

        let not_needed = CompactionResult::NotNeeded;
        assert!(!not_needed.was_compacted());
        assert!(not_needed.summary().is_none());
    }

    #[test]
    fn test_compaction_control_builder() {
        let control = CompactionControl::enabled()
            .with_target_tokens(50_000)
            .with_threshold_tokens(100_000)
            .with_preserve_recent(10);

        assert!(control.enabled);
        assert_eq!(control.target_tokens, Some(50_000));
        assert_eq!(control.threshold_tokens, Some(100_000));
        assert_eq!(control.preserve_recent_messages, Some(10));
    }
}
