//! End-to-end tests module - tests with real Claude API calls
//!
//! These tests are separate from integration tests and require:
//! - ANTHROPIC_API_KEY environment variable
//! - Real subprocess transport (Claude CLI)
//! - Network connectivity
//!
//! Run with: cargo test --test e2e_* -- --nocapture

pub mod common;
