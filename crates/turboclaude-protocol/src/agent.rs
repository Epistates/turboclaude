//! Agent protocol types and definitions
//!
//! Provides types specific to the agent SDK protocol, including control requests,
//! hooks, permissions, and agent definitions.

use serde::{Deserialize, Serialize};

/// Agent definition for specialized agent personas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDefinition {
    /// Unique name for the agent
    pub name: String,

    /// Description of the agent's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// System prompt for the agent
    pub system_prompt: String,

    /// Model to use for this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// List of tools the agent can use
    #[serde(default)]
    pub tool_allowlist: Vec<String>,
}

impl AgentDefinition {
    /// Create a new agent definition
    pub fn new(name: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            system_prompt: system_prompt.into(),
            model: None,
            tool_allowlist: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set allowed tools
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tool_allowlist = tools;
        self
    }
}

/// Control request from Claude to the client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlRequest {
    /// Permission check for tool use
    #[serde(rename = "permission_check")]
    PermissionCheck(ToolPermissionRequest),

    /// Hook event
    #[serde(rename = "hook")]
    Hook {
        /// The name of the hook event (e.g., "pre_tool_use")
        event_type: String,
        /// The data payload for the hook event
        data: serde_json::Value,
    },

    /// Permission mode change request
    #[serde(rename = "permission_mode")]
    PermissionModeChange {
        /// The new permission mode to apply
        mode: PermissionMode,
    },

    /// Interrupt request
    #[serde(rename = "interrupt")]
    Interrupt,
}

/// Permission check request for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolPermissionRequest {
    /// Name of the tool
    pub tool: String,

    /// Input parameters for the tool
    pub input: serde_json::Value,

    /// CLI-provided suggestions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_suggestion: Option<String>,
}

/// Response to a control request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlResponse {
    /// Unique identifier for this control request
    pub request_id: String,

    /// Whether the action was approved
    pub approved: bool,

    /// Modified input if action was modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_input: Option<serde_json::Value>,

    /// Reason if action was denied
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Permission response for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionResponse {
    /// Whether the tool use is allowed
    pub allow: bool,

    /// Modified input if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_input: Option<serde_json::Value>,

    /// Permission request suggestion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_request_suggestion: Option<String>,
}

impl PermissionResponse {
    /// Create an allow response
    pub fn allow() -> Self {
        Self {
            allow: true,
            modified_input: None,
            permission_request_suggestion: None,
        }
    }

    /// Create a deny response
    pub fn deny() -> Self {
        Self {
            allow: false,
            modified_input: None,
            permission_request_suggestion: None,
        }
    }
}

/// Hook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    /// Fired before a tool is executed.
    #[serde(rename = "pre_tool_use")]
    PreToolUse {
        /// Data about the tool being used.
        tool: ToolHookData,
    },

    /// Fired after a tool is executed.
    #[serde(rename = "post_tool_use")]
    PostToolUse {
        /// Data about the tool that was used.
        tool: ToolHookData,
        /// The result of the tool execution.
        result: ToolResultHookData,
    },

    /// Fired when the user submits a prompt.
    #[serde(rename = "user_prompt_submit")]
    UserPromptSubmit {
        /// The content of the user's prompt.
        prompt: String,
    },

    /// Fired when the main agent session stops.
    #[serde(rename = "stop")]
    Stop,

    /// Fired when a subagent session stops.
    #[serde(rename = "subagent_stop")]
    SubagentStop,

    /// Fired before the conversation transcript is compacted.
    #[serde(rename = "pre_compact")]
    PreCompact,
}

/// Tool data for hooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolHookData {
    /// Tool ID
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool input
    pub input: serde_json::Value,
}

/// Tool result data for hooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResultHookData {
    /// Tool use ID
    pub tool_use_id: String,

    /// Result content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Whether the result is an error
    #[serde(default)]
    pub is_error: bool,
}

/// Response to a hook event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookResponse {
    /// Whether to continue execution
    pub continue_: bool,

    /// Modified inputs if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_inputs: Option<serde_json::Value>,

    /// Context to inject
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Whether to hide from transcript
    #[serde(default)]
    pub hide_from_transcript: bool,
}

impl HookResponse {
    /// Create a continue response
    pub fn continue_exec() -> Self {
        Self {
            continue_: true,
            modified_inputs: None,
            context: None,
            hide_from_transcript: false,
        }
    }

    /// Create a stop response
    pub fn stop() -> Self {
        Self {
            continue_: false,
            modified_inputs: None,
            context: None,
            hide_from_transcript: false,
        }
    }
}

/// Permission mode for agent sessions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Ask for permission for each tool (default)
    Default,

    /// Accept edits automatically
    AcceptEdits,

    /// Bypass all permission checks
    BypassPermissions,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_definition() {
        let agent = AgentDefinition::new("DataAnalyzer", "Analyze data")
            .with_description("Analyzes datasets")
            .with_model("claude-3-5-sonnet")
            .with_tools(vec!["bash".to_string(), "file_editor".to_string()]);

        assert_eq!(agent.name, "DataAnalyzer");
        assert_eq!(agent.tool_allowlist.len(), 2);
    }

    #[test]
    fn test_permission_response_serialization() {
        let resp = PermissionResponse::allow();
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_hook_response() {
        let resp = HookResponse::continue_exec();
        assert!(resp.continue_);
        let resp = HookResponse::stop();
        assert!(!resp.continue_);
    }
}
