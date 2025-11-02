//! Adaptive context management with token-aware pruning
//!
//! Provides intelligent conversation history management that:
//! - Respects token budgets (soft and hard limits)
//! - Preserves important messages (user questions, tool calls)
//! - Prunes less important content (old reasoning, filler)
//! - Maintains conversation coherence
//!
//! # Strategies
//!
//! - **RecentFirst**: Keep newest messages (FIFO, simplest)
//! - **PreferToolUse**: Prioritize messages with tool calls (preserve actions)
//! - **PreferUserMessages**: Keep user queries over assistant text
//! - **Smart**: Hybrid (recent + tool use + user messages)

use crate::types::{ContentBlock, Message, Role};
use std::cmp::Ordering;

/// Token-aware adaptive context strategy
///
/// Intelligently prunes conversation history while:
/// - Respecting hard token limits
/// - Approaching soft target gradually
/// - Preserving semantic importance
#[derive(Debug, Clone)]
pub struct AdaptiveStrategy {
    /// Target token budget (soft limit - try to stay under this)
    pub target_tokens: usize,

    /// Hard limit (never exceed this)
    pub max_tokens: usize,

    /// Messages to always preserve (by ID)
    pub always_keep: Vec<String>,

    /// Pruning policy determining what's important
    pub policy: PruningPolicy,

    /// Enable verbose logging of pruning decisions
    pub verbose: bool,
}

/// Pruning policy for adaptive context management
#[derive(Debug, Clone, Copy)]
pub enum PruningPolicy {
    /// Keep most recent messages first (FIFO order)
    RecentFirst,

    /// Prefer messages with tool use (preserve actions)
    PreferToolUse,

    /// Prefer user messages over assistant text
    PreferUserMessages,

    /// Hybrid: recent + tool use + user messages + thinking importance
    Smart,
}

impl AdaptiveStrategy {
    /// Create a new adaptive strategy
    pub fn new(target_tokens: usize, max_tokens: usize, policy: PruningPolicy) -> Self {
        Self {
            target_tokens,
            max_tokens,
            always_keep: Vec::new(),
            policy,
            verbose: false,
        }
    }

    /// Enable verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Add message IDs to always preserve
    pub fn always_keep(mut self, ids: Vec<String>) -> Self {
        self.always_keep = ids;
        self
    }

    /// Prune messages to fit within token budget
    ///
    /// # Algorithm
    /// 1. Separate always-keep messages
    /// 2. Calculate remaining budget
    /// 3. Score messages by importance (based on policy)
    /// 4. Take highest-scoring messages until budget exhausted
    /// 5. Sort by original order to preserve conversation flow
    pub fn prune(&self, messages: Vec<Message>) -> Vec<Message> {
        let total_tokens = Self::count_tokens(&messages);
        let original_message_count = messages.len();

        // Already under target? Return unchanged
        if total_tokens <= self.target_tokens {
            if self.verbose {
                eprintln!(
                    "✓ Context within budget: {}/{} tokens",
                    total_tokens, self.target_tokens
                );
            }
            return messages;
        }

        if self.verbose {
            eprintln!(
                "⚠ Context exceeds budget: {}/{} tokens. Pruning...",
                total_tokens, self.target_tokens
            );
        }

        // 1. Separate always-keep messages
        let mut keep_messages = Vec::new();
        let mut candidate_messages = Vec::new();

        for msg in messages {
            if self.always_keep.contains(&msg.id) {
                keep_messages.push(msg);
            } else {
                candidate_messages.push(msg);
            }
        }

        let keep_tokens = Self::count_tokens(&keep_messages);
        let budget = self.max_tokens.saturating_sub(keep_tokens);

        if self.verbose {
            eprintln!(
                "  Keep {} always-keep messages: {} tokens",
                keep_messages.len(),
                keep_tokens
            );
            eprintln!("  Budget for candidates: {} tokens", budget);
        }

        // 2. Score candidates by importance
        let mut scored: Vec<_> = candidate_messages
            .into_iter()
            .map(|msg| {
                let score = self.score_message(&msg);
                (score, msg)
            })
            .collect();

        // Sort by score (highest first)
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    // Tiebreaker: preserve message order by ID (stable sort)
                    a.1.id.cmp(&b.1.id)
                })
        });

        // 3. Take messages until budget exhausted
        let mut result = keep_messages;
        let mut used_tokens = 0;

        for (score, msg) in scored {
            let msg_tokens = Self::estimate_tokens_for_message(&msg);

            if used_tokens + msg_tokens <= budget {
                if self.verbose {
                    eprintln!(
                        "    Keep: {} (score: {:.1}, {} tokens)",
                        msg.id, score, msg_tokens
                    );
                }
                result.push(msg);
                used_tokens += msg_tokens;
            } else if self.verbose {
                eprintln!(
                    "    Drop: {} (score: {:.1}, {} tokens)",
                    msg.id, score, msg_tokens
                );
            }
        }

        // 4. Sort by original order to preserve conversation flow (using ID as proxy)
        result.sort_by_key(|m| m.id.clone());

        if self.verbose {
            let final_tokens = Self::count_tokens(&result);
            eprintln!(
                "✓ Pruned to {}/{} tokens (removed {} messages)",
                final_tokens,
                self.max_tokens,
                original_message_count - result.len()
            );
        }

        result
    }

    /// Score message by importance (higher = more important)
    fn score_message(&self, msg: &Message) -> f64 {
        let mut score = 0.0;

        match self.policy {
            PruningPolicy::RecentFirst => {
                // Simple recency - all messages equal except by time
                score = 1.0;
            }

            PruningPolicy::PreferToolUse => {
                // Strong preference for tool use (actions matter more than talk)
                if msg
                    .content
                    .iter()
                    .any(|c| matches!(c, ContentBlock::ToolUse { .. }))
                {
                    score += 10.0;
                }
                if msg
                    .content
                    .iter()
                    .any(|c| matches!(c, ContentBlock::ToolResult { .. }))
                {
                    score += 8.0;
                }
            }

            PruningPolicy::PreferUserMessages => {
                // Prefer user queries over assistant responses
                if msg.role == Role::User {
                    score += 5.0;
                } else {
                    // Assistant message - downweight
                    score += 1.0;
                }
            }

            PruningPolicy::Smart => {
                // Hybrid scoring
                match msg.role {
                    Role::User => {
                        // User messages are most important
                        score += 5.0;
                        // User messages with tool calls are critical
                        if msg
                            .content
                            .iter()
                            .any(|c| matches!(c, ContentBlock::ToolUse { .. }))
                        {
                            score += 5.0;
                        }
                    }
                    Role::Assistant => {
                        // Assistant messages score based on content
                        if msg
                            .content
                            .iter()
                            .any(|c| matches!(c, ContentBlock::ToolUse { .. }))
                        {
                            score += 8.0; // Tool calls are important
                        } else if msg
                            .content
                            .iter()
                            .any(|c| matches!(c, ContentBlock::ToolResult { .. }))
                        {
                            score += 6.0; // Tool results provide context
                        } else {
                            score += 1.0; // Pure text responses are least important
                        }
                    }
                }
            }
        }

        score
    }

    /// Estimate tokens for a single message
    fn estimate_tokens_for_message(msg: &Message) -> usize {
        let mut tokens = 0;

        // Rough heuristic: 4 characters per token (conservative)
        for content in &msg.content {
            tokens += match content {
                ContentBlock::Text { text, citations: _ } => text.len().div_ceil(4),
                ContentBlock::ToolUse { input, .. } => {
                    let json = serde_json::to_string(input).unwrap_or_default();
                    json.len().div_ceil(4)
                }
                ContentBlock::ToolResult { content, .. } => content.len().div_ceil(4),
                _ => 50, // Conservative estimate for other types
            };
        }

        // Add overhead for message metadata
        tokens + 10
    }

    /// Count total tokens in a set of messages
    pub fn count_tokens(messages: &[Message]) -> usize {
        messages.iter().map(Self::estimate_tokens_for_message).sum()
    }

    /// Check if messages are within budget
    pub fn is_within_budget(&self, messages: &[Message]) -> bool {
        Self::count_tokens(messages) <= self.target_tokens
    }

    /// Get utilization percentage
    pub fn utilization(&self, messages: &[Message]) -> f64 {
        let tokens = Self::count_tokens(messages);
        (tokens as f64 / self.max_tokens as f64) * 100.0
    }
}

// Tests are in turboclaudeagent integration tests to avoid circular dependencies
// and to use real Message types from the protocol layer
