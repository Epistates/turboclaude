//! Protocol message types for bidirectional Agent SDK communication
//!
//! Defines all message types exchanged between the Agent SDK client and Claude Code CLI.
//! All messages are serialized as newline-delimited JSON (NDJSON).

use crate::error::Result;
use crate::message::Message;
use crate::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a protocol request
///
/// Format: `<uuid>` for main request, `<uuid>.<sequence>` for related messages (e.g., hooks)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(String);

impl RequestId {
    /// Generate a new random request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from raw string
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the base request ID (prefix before first `.`)
    pub fn base(&self) -> String {
        self.0.split('.').next().unwrap_or(&self.0).to_string()
    }

    /// Generate a sequenced request ID (for hooks, permission checks, etc.)
    pub fn with_sequence(&self, sequence: usize) -> RequestId {
        RequestId(format!("{}.{}", self.0, sequence))
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this ID matches another by base (ignoring sequence)
    pub fn matches_base(&self, other: &RequestId) -> bool {
        self.base() == other.base()
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Query request sent from client to Claude Code CLI
///
/// Contains the user query, configuration, and message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// The user query string
    pub query: String,

    /// Optional system prompt override
    pub system_prompt: Option<String>,

    /// Model to use for this query
    pub model: String,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Available tools
    pub tools: Vec<ToolDefinition>,

    /// Message history
    pub messages: Vec<Message>,
}

/// Query response from Claude Code CLI to client
///
/// Contains the response message and completion status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// The response message from Claude
    pub message: Message,

    /// Whether the query is complete (no more messages expected)
    pub is_complete: bool,
}

/// Hook event request from Claude Code CLI to client
///
/// Triggered when specific events occur during query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRequest {
    /// Type of hook event
    pub event_type: String,

    /// Event-specific data (tool_name, input, step, etc.)
    pub data: serde_json::Value,
}

/// Hook event response from client to Claude Code CLI
///
/// Tells Claude how to proceed after a hook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookResponse {
    /// Whether to continue execution
    #[serde(rename = "continue")]
    pub continue_: bool,

    /// Optional modified inputs (tool_name and input)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_inputs: Option<ModifiedInputs>,

    /// Optional context/feedback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,

    /// Permission decision for PreToolUse hooks
    ///
    /// Controls whether the tool should be allowed, denied, or require user confirmation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<String>,

    /// Reason for the permission decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,

    /// Additional context specific to the hook type
    ///
    /// For PostToolUse, UserPromptSubmit, etc. Contains arbitrary JSON data
    /// that provides hook-specific output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<serde_json::Value>,

    /// Reason for continuing execution (semantic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_reason: Option<String>,

    /// Reason for stopping execution (semantic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// System message to display to user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,

    /// Reason/feedback for Claude (not shown to user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Whether to suppress output from transcript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

impl HookResponse {
    /// Create a simple "continue" response
    pub fn continue_exec() -> Self {
        Self {
            continue_: true,
            modified_inputs: None,
            context: None,
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            continue_reason: None,
            stop_reason: None,
            system_message: None,
            reason: None,
            suppress_output: None,
        }
    }

    /// Create a simple "stop" response
    pub fn stop() -> Self {
        Self {
            continue_: false,
            modified_inputs: None,
            context: None,
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            continue_reason: None,
            stop_reason: None,
            system_message: None,
            reason: None,
            suppress_output: None,
        }
    }

    /// Create a response with permission decision
    pub fn with_permission_decision(mut self, decision: impl Into<String>) -> Self {
        self.permission_decision = Some(decision.into());
        self
    }

    /// Create a response with permission decision reason
    pub fn with_permission_reason(mut self, reason: impl Into<String>) -> Self {
        self.permission_decision_reason = Some(reason.into());
        self
    }

    /// Create a response with additional context
    pub fn with_additional_context(mut self, context: serde_json::Value) -> Self {
        self.additional_context = Some(context);
        self
    }

    /// Create a response with continue reason
    pub fn with_continue_reason(mut self, reason: impl Into<String>) -> Self {
        self.continue_reason = Some(reason.into());
        self
    }

    /// Create a response with stop reason
    pub fn with_stop_reason(mut self, reason: impl Into<String>) -> Self {
        self.stop_reason = Some(reason.into());
        self
    }

    /// Create a response with system message
    pub fn with_system_message(mut self, message: impl Into<String>) -> Self {
        self.system_message = Some(message.into());
        self
    }

    /// Create a response with reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Create a response with suppress output flag
    pub fn with_suppress_output(mut self, suppress: bool) -> Self {
        self.suppress_output = Some(suppress);
        self
    }
}

/// Modified inputs that can be sent in hook response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifiedInputs {
    /// Modified tool name (if applicable)
    pub tool_name: Option<String>,

    /// Modified tool input (if applicable)
    pub input: Option<serde_json::Value>,
}

/// Permission check request from Claude Code CLI to client
///
/// Asks the client if a tool use should be allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckRequest {
    /// Name of the tool to be used
    pub tool: String,

    /// Input to the tool
    pub input: serde_json::Value,

    /// Human-readable suggestion for the user
    pub suggestion: String,
}

/// Permission response from client to Claude Code CLI
///
/// Grants or denies permission for a tool use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionResponse {
    /// Whether to allow the tool use
    pub allow: bool,

    /// Modified input if approved (optional)
    pub modified_input: Option<serde_json::Value>,

    /// Reason for the decision (for audit trail)
    pub reason: Option<String>,
}

/// Control request from client to Claude Code CLI
///
/// Sends runtime control commands (interrupt, change model, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "payload")]
pub enum ControlCommand {
    /// Interrupt the current query
    #[serde(rename = "interrupt")]
    Interrupt,

    /// Change the model for future queries
    #[serde(rename = "set_model")]
    SetModel(String),

    /// Change the permission mode
    #[serde(rename = "set_permission_mode")]
    SetPermissionMode(String),

    /// Get current session state
    #[serde(rename = "get_state")]
    GetState,
}

/// Control request wrapper with request ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    /// The control command
    #[serde(flatten)]
    pub command: ControlCommand,
}

/// Control response from Claude Code CLI to client
///
/// Acknowledges control request and returns result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    /// Command was successful
    pub success: bool,

    /// Optional message
    pub message: Option<String>,

    /// Optional response data
    pub data: Option<serde_json::Value>,
}

/// Protocol error message sent by either party
///
/// Indicates an error in message processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolErrorMessage {
    /// Error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Optional detailed information
    pub details: Option<serde_json::Value>,
}

/// Union of all possible protocol messages
///
/// Used for routing and type-safe message handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ProtocolMessage {
    /// Query request (client → CLI)
    #[serde(rename = "query")]
    Query(QueryRequest),

    /// Query response (CLI → client)
    #[serde(rename = "response")]
    Response(QueryResponse),

    /// Hook request (CLI → client)
    #[serde(rename = "hook_request")]
    HookRequest(HookRequest),

    /// Hook response (client → CLI)
    #[serde(rename = "hook_response")]
    HookResponse(Box<HookResponse>),

    /// Permission check (CLI → client)
    #[serde(rename = "permission_check")]
    PermissionCheck(PermissionCheckRequest),

    /// Permission response (client → CLI)
    #[serde(rename = "permission_response")]
    PermissionResponse(PermissionResponse),

    /// Control request (client → CLI)
    #[serde(rename = "control_request")]
    ControlRequest(ControlRequest),

    /// Control response (CLI → client)
    #[serde(rename = "control_response")]
    ControlResponse(ControlResponse),

    /// Error message (either direction)
    #[serde(rename = "error")]
    Error(ProtocolErrorMessage),
}

impl ProtocolMessage {
    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| crate::error::ProtocolError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| crate::error::ProtocolError::SerializationError(e.to_string()))
    }

    /// Get the request ID if this message has one
    pub fn request_id(&self) -> Option<RequestId> {
        // Messages with IDs embedded would need those types updated
        // For now, return None as the current design doesn't embed IDs
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let id = RequestId::new();
        assert!(!id.as_str().is_empty());
        assert_eq!(id.base(), id.as_str());
    }

    #[test]
    fn test_request_id_sequence() {
        let id = RequestId::from_string("550e8400");
        let seq = id.with_sequence(1);
        assert_eq!(seq.as_str(), "550e8400.1");
        assert_eq!(seq.base(), "550e8400");
    }

    #[test]
    fn test_request_id_matches_base() {
        let id1 = RequestId::from_string("550e8400");
        let id2 = RequestId::from_string("550e8400.1");
        let id3 = RequestId::from_string("other");

        assert!(id1.matches_base(&id2));
        assert!(id2.matches_base(&id1));
        assert!(!id1.matches_base(&id3));
    }

    #[test]
    fn test_hook_request_serialization() {
        let hook = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({ "tool": "search" }),
        };

        let json = serde_json::to_string(&hook).unwrap();
        let deserialized: HookRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_type, "PreToolUse");
        assert_eq!(deserialized.data["tool"], "search");
    }

    #[test]
    fn test_permission_check_serialization() {
        let check = PermissionCheckRequest {
            tool: "web_search".to_string(),
            input: serde_json::json!({ "query": "test" }),
            suggestion: "Use web_search? (yes/no)".to_string(),
        };

        let json = serde_json::to_string(&check).unwrap();
        let deserialized: PermissionCheckRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool, "web_search");
    }

    #[test]
    fn test_hook_response_serialization() {
        let response = Box::new(HookResponse {
            continue_: true,
            modified_inputs: None,
            context: None,
            permission_decision: None,
            permission_decision_reason: None,
            additional_context: None,
            continue_reason: None,
            stop_reason: None,
            system_message: None,
            reason: None,
            suppress_output: None,
        });

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""continue":true"#));

        let deserialized: HookResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.continue_);
    }

    #[test]
    fn test_permission_response_serialization() {
        let response = PermissionResponse {
            allow: true,
            modified_input: None,
            reason: Some("User approved".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PermissionResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.allow);
    }

    #[test]
    fn test_protocol_message_query_roundtrip() {
        let request = QueryRequest {
            query: "What is the capital of France?".to_string(),
            system_prompt: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            tools: vec![],
            messages: vec![],
        };

        let msg = ProtocolMessage::Query(request.clone());
        let json = msg.to_json().unwrap();
        let deserialized = ProtocolMessage::from_json(&json).unwrap();

        match deserialized {
            ProtocolMessage::Query(q) => {
                assert_eq!(q.query, request.query);
                assert_eq!(q.model, request.model);
            }
            _ => panic!("Expected Query message"),
        }
    }

    #[test]
    fn test_protocol_message_hook_request_roundtrip() {
        let hook = HookRequest {
            event_type: "PreToolUse".to_string(),
            data: serde_json::json!({ "tool": "search", "step": 1 }),
        };

        let msg = ProtocolMessage::HookRequest(hook.clone());
        let json = msg.to_json().unwrap();
        let deserialized = ProtocolMessage::from_json(&json).unwrap();

        match deserialized {
            ProtocolMessage::HookRequest(h) => {
                assert_eq!(h.event_type, "PreToolUse");
            }
            _ => panic!("Expected HookRequest message"),
        }
    }

    #[test]
    fn test_control_command_interrupt() {
        let cmd = ControlCommand::Interrupt;
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("interrupt"));
    }

    #[test]
    fn test_control_command_set_model() {
        let cmd = ControlCommand::SetModel("claude-3-5-haiku-20241022".to_string());
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("set_model"));
        assert!(json.contains("claude-3-5-haiku-20241022"));
    }

    #[test]
    fn test_protocol_error_message_serialization() {
        let error = ProtocolErrorMessage {
            code: "parse_error".to_string(),
            message: "Invalid JSON".to_string(),
            details: Some(serde_json::json!({ "line": 5 })),
        };

        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ProtocolErrorMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.code, "parse_error");
    }
}
