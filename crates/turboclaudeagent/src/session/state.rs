//! Session state management
//!
//! Provides structures and operations for tracking session state including
//! connection status, model settings, permission modes, and conversation history.

use turboclaude_protocol::{Message, PermissionMode};

/// Current state of the agent session
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Whether the session is connected to CLI
    pub is_connected: bool,

    /// Current model
    pub current_model: String,

    /// Current permission mode
    pub current_permission_mode: PermissionMode,

    /// Number of active queries
    pub active_queries: u32,

    /// Conversation history (for fork support)
    pub(crate) conversation_history: Vec<Message>,
}

impl SessionState {
    /// Create a new session state
    pub(crate) fn new(model: String, permission_mode: PermissionMode) -> Self {
        Self {
            is_connected: true,
            current_model: model,
            current_permission_mode: permission_mode,
            active_queries: 0,
            conversation_history: Vec::new(),
        }
    }

    /// Add a message to the conversation history
    pub(crate) fn add_to_history(&mut self, message: Message) {
        self.conversation_history.push(message);
    }

    /// Get a clone of the conversation history
    pub(crate) fn get_history(&self) -> Vec<Message> {
        self.conversation_history.clone()
    }

    /// Clear conversation history
    #[allow(dead_code)]
    pub(crate) fn clear_history(&mut self) {
        self.conversation_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_new() {
        let state = SessionState::new("claude-3-5-sonnet".to_string(), PermissionMode::Default);

        assert!(state.is_connected);
        assert_eq!(state.current_model, "claude-3-5-sonnet");
        assert_eq!(state.current_permission_mode, PermissionMode::Default);
        assert_eq!(state.active_queries, 0);
        assert!(state.conversation_history.is_empty());
    }

    #[test]
    fn test_conversation_history() {
        use turboclaude_protocol::{
            message::MessageRole,
            types::{CacheUsage, StopReason, Usage},
        };

        let mut state = SessionState::new("claude-3-5-sonnet".to_string(), PermissionMode::Default);

        let msg1 = Message {
            id: "msg_1".to_string(),
            message_type: "message".to_string(),
            role: MessageRole::User,
            content: vec![],
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
            cache_usage: CacheUsage {
                cache_read_input_tokens: 0,
                cache_creation_input_tokens: 0,
            },
            created_at: String::new(),
        };
        let msg2 = Message {
            id: "msg_2".to_string(),
            message_type: "message".to_string(),
            role: MessageRole::Assistant,
            content: vec![],
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
            },
            cache_usage: CacheUsage {
                cache_read_input_tokens: 0,
                cache_creation_input_tokens: 0,
            },
            created_at: String::new(),
        };

        state.add_to_history(msg1.clone());
        state.add_to_history(msg2.clone());

        let history = state.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, MessageRole::User);
        assert_eq!(history[1].role, MessageRole::Assistant);

        state.clear_history();
        assert!(state.conversation_history.is_empty());
    }

    #[test]
    fn test_session_state_clone() {
        let state = SessionState {
            is_connected: true,
            current_model: "claude-3-5-sonnet".to_string(),
            current_permission_mode: PermissionMode::Default,
            active_queries: 0,
            conversation_history: Vec::new(),
        };

        let state2 = state.clone();
        assert_eq!(state.is_connected, state2.is_connected);
        assert_eq!(state.current_model, state2.current_model);
    }
}
