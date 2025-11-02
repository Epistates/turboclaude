//! Hook system types and matching for agent SDK
//!
//! Provides types for hook matchers, permission decisions, and advanced hook response fields
//! that enable sophisticated control over tool execution and agent behavior.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Permission decision for PreToolUse hooks
///
/// Indicates whether a tool should be allowed, denied, or require user confirmation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision {
    /// Allow the tool to execute
    Allow,
    /// Deny the tool execution
    Deny,
    /// Ask the user for permission
    Ask,
}

/// Reason for continuing execution
///
/// Provides semantic context for why a hook decided to continue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContinueReason {
    /// Hook approved the action
    Approved,
    /// Hook modified the input
    Modified,
    /// Hook added context
    ContextAdded,
    /// Hook allowed with conditions
    Conditional,
    /// Custom reason
    Custom(String),
}

/// Reason for stopping execution
///
/// Provides semantic context for why a hook decided to stop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Security policy violation
    SecurityViolation,
    /// Error detected
    ErrorDetected,
    /// User-requested stop
    UserRequested,
    /// Critical condition detected
    Critical,
    /// Custom reason
    Custom(String),
}

/// Hook matcher for selective hook invocation
///
/// Provides pattern-based matching to control when hooks are invoked.
/// Hooks are only triggered when the matcher criteria are satisfied.
///
/// # Examples
///
/// ```
/// use turboclaude_protocol::hooks::HookMatcher;
///
/// // Match specific tool by name
/// let matcher = HookMatcher::new()
///     .with_tool_name("Bash");
///
/// // Match tools using regex
/// let matcher = HookMatcher::new()
///     .with_tool_name_regex(r"^(Write|Edit|MultiEdit)$");
///
/// // Match any tool (always trigger)
/// let matcher = HookMatcher::new();
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookMatcher {
    /// Exact tool name to match (case-sensitive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    /// Regex pattern for tool name matching
    ///
    /// Uses Rust regex syntax. Patterns are matched against the full tool name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "serde_regex", default)]
    pub tool_name_regex: Option<Regex>,

    /// Match only if tool input contains specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_input_fields: Option<Vec<String>>,

    /// Match only specific event types
    ///
    /// If None, matches all events. If Some, only matches events in the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_types: Option<Vec<String>>,
}

impl HookMatcher {
    /// Create a new empty matcher (matches everything)
    pub fn new() -> Self {
        Self::default()
    }

    /// Set exact tool name to match
    pub fn with_tool_name(mut self, name: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    /// Set tool name regex pattern
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid. For fallible construction, use `try_with_tool_name_regex`.
    pub fn with_tool_name_regex(mut self, pattern: &str) -> Self {
        self.tool_name_regex = Some(Regex::new(pattern).expect("Invalid regex pattern"));
        self
    }

    /// Try to set tool name regex pattern (fallible)
    pub fn try_with_tool_name_regex(mut self, pattern: &str) -> Result<Self, regex::Error> {
        self.tool_name_regex = Some(Regex::new(pattern)?);
        Ok(self)
    }

    /// Set required input fields
    pub fn with_required_fields(mut self, fields: Vec<String>) -> Self {
        self.required_input_fields = Some(fields);
        self
    }

    /// Set specific event types to match
    pub fn with_event_types(mut self, events: Vec<String>) -> Self {
        self.event_types = Some(events);
        self
    }

    /// Check if this matcher matches the given hook context
    ///
    /// Returns true if all specified criteria are satisfied.
    pub fn matches(&self, context: &HookContext) -> bool {
        // Check tool name exact match
        if let Some(ref name) = self.tool_name
            && context.tool_name.as_ref() != Some(name)
        {
            return false;
        }

        // Check tool name regex match
        if let Some(ref regex) = self.tool_name_regex {
            if let Some(ref tool_name) = context.tool_name {
                if !regex.is_match(tool_name) {
                    return false;
                }
            } else {
                // No tool name but regex specified
                return false;
            }
        }

        // Check required input fields
        if let Some(ref required_fields) = self.required_input_fields {
            if let Some(ref input) = context.tool_input {
                for field in required_fields {
                    if input.get(field).is_none() {
                        return false;
                    }
                }
            } else {
                // No input but required fields specified
                return false;
            }
        }

        // Check event type
        if let Some(ref event_types) = self.event_types
            && !event_types.contains(&context.event_type)
        {
            return false;
        }

        true
    }

    /// Check if this is an empty matcher (matches everything)
    pub fn is_empty(&self) -> bool {
        self.tool_name.is_none()
            && self.tool_name_regex.is_none()
            && self.required_input_fields.is_none()
            && self.event_types.is_none()
    }
}

/// Context information for hook matching
///
/// Provides all available context that matchers can use to decide whether to invoke.
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    /// Event type (PreToolUse, PostToolUse, etc.)
    pub event_type: String,

    /// Tool name if applicable
    pub tool_name: Option<String>,

    /// Tool input if applicable
    pub tool_input: Option<serde_json::Value>,

    /// Tool output if applicable (PostToolUse)
    pub tool_output: Option<serde_json::Value>,

    /// Session ID
    pub session_id: Option<String>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            ..Default::default()
        }
    }

    /// Set tool name
    pub fn with_tool_name(mut self, name: impl Into<String>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    /// Set tool input
    pub fn with_tool_input(mut self, input: serde_json::Value) -> Self {
        self.tool_input = Some(input);
        self
    }

    /// Set tool output
    pub fn with_tool_output(mut self, output: serde_json::Value) -> Self {
        self.tool_output = Some(output);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }
}

/// Custom serialization for Regex using serde_regex
mod serde_regex {
    use regex::Regex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(regex: &Option<Regex>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match regex {
            Some(r) => serializer.serialize_some(r.as_str()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => Regex::new(&s).map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_decision_serialization() {
        let allow = PermissionDecision::Allow;
        let json = serde_json::to_string(&allow).unwrap();
        assert_eq!(json, r#""allow""#);

        let deny = PermissionDecision::Deny;
        let json = serde_json::to_string(&deny).unwrap();
        assert_eq!(json, r#""deny""#);

        let ask = PermissionDecision::Ask;
        let json = serde_json::to_string(&ask).unwrap();
        assert_eq!(json, r#""ask""#);
    }

    #[test]
    fn test_continue_reason_serialization() {
        let reason = ContinueReason::Approved;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, r#""approved""#);

        let custom = ContinueReason::Custom("custom reason".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("custom reason"));
    }

    #[test]
    fn test_stop_reason_serialization() {
        let reason = StopReason::SecurityViolation;
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, r#""security_violation""#);

        let custom = StopReason::Custom("critical error".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("critical error"));
    }

    #[test]
    fn test_hook_matcher_empty() {
        let matcher = HookMatcher::new();
        assert!(matcher.is_empty());

        let context = HookContext::new("PreToolUse");
        assert!(matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_tool_name_exact() {
        let matcher = HookMatcher::new().with_tool_name("Bash");

        let context = HookContext::new("PreToolUse").with_tool_name("Bash");
        assert!(matcher.matches(&context));

        let context = HookContext::new("PreToolUse").with_tool_name("Write");
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_tool_name_regex() {
        let matcher = HookMatcher::new().with_tool_name_regex(r"^(Write|Edit|MultiEdit)$");

        let context = HookContext::new("PreToolUse").with_tool_name("Write");
        assert!(matcher.matches(&context));

        let context = HookContext::new("PreToolUse").with_tool_name("Edit");
        assert!(matcher.matches(&context));

        let context = HookContext::new("PreToolUse").with_tool_name("Bash");
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_required_fields() {
        let matcher = HookMatcher::new()
            .with_tool_name("Bash")
            .with_required_fields(vec!["command".to_string()]);

        let input = serde_json::json!({ "command": "echo hello" });
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Bash")
            .with_tool_input(input);
        assert!(matcher.matches(&context));

        let input = serde_json::json!({ "other": "value" });
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Bash")
            .with_tool_input(input);
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_event_types() {
        let matcher = HookMatcher::new()
            .with_event_types(vec!["PreToolUse".to_string(), "PostToolUse".to_string()]);

        let context = HookContext::new("PreToolUse");
        assert!(matcher.matches(&context));

        let context = HookContext::new("UserPromptSubmit");
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_combined() {
        let matcher = HookMatcher::new()
            .with_tool_name_regex(r"^(Write|Edit)$")
            .with_event_types(vec!["PreToolUse".to_string()])
            .with_required_fields(vec!["file_path".to_string()]);

        // All criteria match
        let input = serde_json::json!({ "file_path": "/tmp/test.txt", "content": "test" });
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Write")
            .with_tool_input(input);
        assert!(matcher.matches(&context));

        // Wrong tool name
        let input = serde_json::json!({ "file_path": "/tmp/test.txt" });
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Bash")
            .with_tool_input(input);
        assert!(!matcher.matches(&context));

        // Wrong event type
        let input = serde_json::json!({ "file_path": "/tmp/test.txt" });
        let context = HookContext::new("PostToolUse")
            .with_tool_name("Write")
            .with_tool_input(input);
        assert!(!matcher.matches(&context));

        // Missing required field
        let input = serde_json::json!({ "content": "test" });
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Write")
            .with_tool_input(input);
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_serialization() {
        let matcher = HookMatcher::new()
            .with_tool_name("Bash")
            .with_tool_name_regex(r"^Bash$");

        let json = serde_json::to_string(&matcher).unwrap();
        let deserialized: HookMatcher = serde_json::from_str(&json).unwrap();

        assert_eq!(matcher.tool_name, deserialized.tool_name);
        assert_eq!(
            matcher.tool_name_regex.as_ref().map(|r| r.as_str()),
            deserialized.tool_name_regex.as_ref().map(|r| r.as_str())
        );
    }

    #[test]
    fn test_hook_context_builder() {
        let context = HookContext::new("PreToolUse")
            .with_tool_name("Bash")
            .with_tool_input(serde_json::json!({"command": "ls"}))
            .with_session_id("session-123");

        assert_eq!(context.event_type, "PreToolUse");
        assert_eq!(context.tool_name, Some("Bash".to_string()));
        assert_eq!(context.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_hook_matcher_no_tool_name() {
        let matcher = HookMatcher::new().with_tool_name_regex(r"^Bash$");

        // Context without tool_name should not match when regex is specified
        let context = HookContext::new("UserPromptSubmit");
        assert!(!matcher.matches(&context));
    }

    #[test]
    fn test_hook_matcher_regex_partial_match() {
        // Regex should NOT match partial strings
        let matcher = HookMatcher::new().with_tool_name_regex(r"Write");

        let context = HookContext::new("PreToolUse").with_tool_name("MultiWrite");
        assert!(matcher.matches(&context)); // "Write" is contained

        // To match exact, use anchors
        let matcher = HookMatcher::new().with_tool_name_regex(r"^Write$");
        let context = HookContext::new("PreToolUse").with_tool_name("MultiWrite");
        assert!(!matcher.matches(&context));

        let context = HookContext::new("PreToolUse").with_tool_name("Write");
        assert!(matcher.matches(&context));
    }
}
