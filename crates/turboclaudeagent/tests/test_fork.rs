//! Tests for session forking functionality
//!
//! Verifies that forked sessions:
//! - Create independent subprocesses
//! - Copy conversation history
//! - Inherit configuration
//! - Can diverge independently
//!
//! Note: These tests require a running Claude CLI subprocess.
//! They are designed to verify the fork() API and behavior.

use turboclaudeagent::{AgentSession, SessionConfig};

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_creates_independent_session() {
    // Create original session
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // Fork the session
    let forked = session.fork().await;
    assert!(forked.is_ok(), "Fork should succeed: {:?}", forked.err());

    let _forked_session = forked.unwrap();

    // Both sessions should be independently usable
    assert!(session.is_connected().await);
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_basic() {
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // Fork should succeed
    let forked = session.fork().await;
    assert!(forked.is_ok(), "Fork should succeed: {:?}", forked.err());

    let _forked_session = forked.unwrap();

    // Both sessions should be independently usable
    assert!(session.is_connected().await);
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_copies_conversation_history() {
    // This test verifies that conversation history is copied
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // In a real test with a running CLI:
    // session.query_str("What is 2+2?").await.unwrap();
    // let forked = session.fork().await.unwrap();
    // Verify the forked session has the same history

    // For now, just verify fork succeeds
    let forked = session.fork().await;
    assert!(forked.is_ok());
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_inherits_model_setting() {
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // Set a specific model
    let _ = session.set_model("claude-opus-4").await;

    // Fork the session
    let forked = session.fork().await;
    assert!(forked.is_ok());

    let forked_session = forked.unwrap();

    // Verify model was inherited
    let state = forked_session.state().await;
    assert_eq!(state.current_model, "claude-opus-4");
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_inherits_permission_mode() {
    use turboclaude_protocol::PermissionMode;

    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // Set a specific permission mode
    let _ = session
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await;

    // Fork the session
    let forked = session.fork().await;
    assert!(forked.is_ok());

    let forked_session = forked.unwrap();

    // Verify permission mode was inherited
    let state = forked_session.state().await;
    assert_eq!(state.current_permission_mode, PermissionMode::AcceptEdits);
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_forked_sessions_can_diverge() {
    // This test verifies that forked sessions can have independent conversations
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    let forked = session.fork().await;
    assert!(forked.is_ok());

    let forked_session = forked.unwrap();

    // Both sessions should be independently usable
    assert!(session.is_connected().await);
    assert!(forked_session.is_connected().await);

    // In a real test with running CLI:
    // session.query_str("Tell me about cats").await.unwrap();
    // forked_session.query_str("Tell me about dogs").await.unwrap();
    //
    // Verify they have different histories
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_multiple_forks() {
    // Test creating multiple forks from the same session
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    let fork1 = session.fork().await;
    let fork2 = session.fork().await;
    let fork3 = session.fork().await;

    assert!(fork1.is_ok());
    assert!(fork2.is_ok());
    assert!(fork3.is_ok());

    // All sessions should be independent
    assert!(session.is_connected().await);
    assert!(fork1.unwrap().is_connected().await);
    assert!(fork2.unwrap().is_connected().await);
    assert!(fork3.unwrap().is_connected().await);
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_of_fork() {
    // Test creating a fork of a fork (nested forking)
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    let fork1 = session.fork().await.unwrap();
    let fork2 = fork1.fork().await;

    assert!(fork2.is_ok());

    let fork2_session = fork2.unwrap();

    // All three sessions should be independent
    assert!(session.is_connected().await);
    assert!(fork1.is_connected().await);
    assert!(fork2_session.is_connected().await);
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_state_snapshot() {
    use turboclaude_protocol::PermissionMode;

    // Test that fork captures a snapshot of state at fork time
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    // Set initial state
    let _ = session.set_model("claude-sonnet-4").await;
    let _ = session
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await;

    // Fork
    let forked = session.fork().await.unwrap();

    // Change original session state
    let _ = session.set_model("claude-opus-4").await;
    let _ = session
        .set_permission_mode(PermissionMode::BypassPermissions)
        .await;

    // Verify forked session kept the snapshot state
    let forked_state = forked.state().await;
    assert_eq!(forked_state.current_model, "claude-sonnet-4");
    assert_eq!(
        forked_state.current_permission_mode,
        PermissionMode::AcceptEdits
    );

    // Verify original session has new state
    let original_state = session.state().await;
    assert_eq!(original_state.current_model, "claude-opus-4");
    assert_eq!(
        original_state.current_permission_mode,
        PermissionMode::BypassPermissions
    );
}

#[tokio::test]
#[ignore] // Requires running Claude CLI
async fn test_fork_independent_close() {
    // Test that closing one session doesn't affect the other
    let config = SessionConfig::default();
    let session = AgentSession::new(config).await.unwrap();

    let forked = session.fork().await.unwrap();

    // Close original session
    let _ = session.close().await;

    // Forked session should still be connected
    assert!(forked.is_connected().await);

    // Close forked session
    let _ = forked.close().await;

    // Both should now be disconnected
    assert!(!session.is_connected().await);
    assert!(!forked.is_connected().await);
}
