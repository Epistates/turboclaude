//! Model-related types

use serde::{Deserialize, Serialize};

/// Information about a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique identifier for the model
    pub id: String,

    /// Type of the object (always "model")
    #[serde(rename = "type")]
    pub model_type: String,

    /// Display name of the model
    pub display_name: String,

    /// When the model was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Predefined model identifiers.
pub struct Models;

impl Models {
    // ============================================================================
    // LATEST MODELS (October 2025) - RECOMMENDED FOR NEW PROJECTS
    // ============================================================================

    /// Claude Sonnet 4.5 (September 2025) - **RECOMMENDED FOR PRODUCTION**
    /// Best coding model in the world, strongest for building complex agents,
    /// best at computer use. $3/$15 per million tokens.
    pub const CLAUDE_SONNET_4_5: &'static str = "claude-sonnet-4-5-20250514";

    /// Claude Haiku 4.5 (October 2025) - **RECOMMENDED FOR SPEED & COST**
    /// Small, fast model optimized for low latency. Near-frontier coding quality,
    /// matches Sonnet 4 on coding, surpasses Sonnet 4 on some computer-use tasks.
    /// $1/$5 per million tokens.
    pub const CLAUDE_HAIKU_4_5: &'static str = "claude-haiku-4-5-20251007";

    // ============================================================================
    // LEGACY MODELS (Kept for backward compatibility)
    // ============================================================================

    /// Claude Opus 4.1 (January 2025)
    /// Improved agentic tasks, coding, and reasoning.
    /// Scores 74.5% on SWE-bench Verified.
    pub const CLAUDE_OPUS_4_1: &'static str = "claude-opus-4-1-20250805";

    /// Claude 3.5 Sonnet (October 2024)
    /// Deprecated: Use CLAUDE_SONNET_4_5 instead.
    pub const CLAUDE_3_5_SONNET: &'static str = "claude-3-5-sonnet-20241022";

    /// Claude 3.5 Haiku (October 2024)
    /// Deprecated: Use CLAUDE_HAIKU_4_5 instead.
    pub const CLAUDE_3_5_HAIKU: &'static str = "claude-3-5-haiku-20241022";

    /// Claude 3 Opus (February 2024)
    /// Deprecated: Use CLAUDE_OPUS_4_1 instead.
    pub const CLAUDE_3_OPUS: &'static str = "claude-3-opus-20240229";

    /// Claude 3 Sonnet (February 2024)
    /// Deprecated: Use CLAUDE_SONNET_4_5 instead.
    pub const CLAUDE_3_SONNET: &'static str = "claude-3-sonnet-20240229";

    /// Claude 3 Haiku (March 2024)
    /// Deprecated: Use CLAUDE_HAIKU_4_5 instead.
    pub const CLAUDE_3_HAIKU: &'static str = "claude-3-haiku-20240307";

    // ============================================================================
    // DEFAULT MODEL
    // ============================================================================

    /// Default model for new requests (Sonnet 4.5)
    pub const DEFAULT: &'static str = "claude-sonnet-4-5-20250514";
}
