//! Example demonstrating context management in conversations
//!
//! This example shows how to:
//! 1. Manage conversation context with thinking blocks
//! 2. Clear old thinking to reduce token usage
//! 3. Control which thinking turns are preserved

use turboclaude::types::beta::{
    BetaClearThinking20251015EditParam, ContextManagementEdit, ContextManagementEditResponse,
    ThinkingConfig,
};

fn main() {
    println!("=== Context Management in Conversations ===\n");

    // 1. Understanding conversation context
    println!("1. Conversation Context Overview:");
    println!("   - Extended thinking creates reasoning blocks");
    println!("   - Thinking accumulates over many turns");
    println!("   - Context management helps optimize token usage\n");

    // 2. Creating thinking configuration
    println!("2. Enable extended thinking:");
    let thinking_config = ThinkingConfig::new(5000);
    println!(
        "✓ Thinking budget: {} tokens",
        thinking_config.budget_tokens
    );
    println!("✓ Configuration type: {}\n", thinking_config.config_type);

    // 3. Long conversation scenario
    println!("3. Simulated conversation flow:");
    let conversation_state = vec![
        ("Turn 1", 200, "Initial analysis"),
        ("Turn 2", 350, "Follow-up reasoning"),
        ("Turn 3", 280, "Clarification"),
        ("Turn 4", 400, "Complex reasoning"),
        ("Turn 5", 350, "Final analysis"),
        ("Turn 6", 300, "Summary"),
        ("Turn 7", 250, "Verification"),
    ];

    let total_thinking_tokens: u32 = conversation_state.iter().map(|t| t.1).sum();
    println!("   Turn | Thinking Tokens | Content");
    println!("   -----|-----------------|-------------------");
    for (turn, tokens, content) in &conversation_state {
        println!("   {} | {} | {}", turn, tokens, content);
    }
    println!(
        "\n   Total thinking tokens used: {}\n",
        total_thinking_tokens
    );

    // 4. Decision to clear old thinking
    println!("4. Managing token usage:");
    println!("   Problem: Too much context accumulated");
    println!("   Solution: Clear old thinking blocks");
    println!("   Strategy: Keep only recent thinking for context\n");

    // 5. Creating context management edit
    println!("5. Creating clear thinking request:");
    let clear_request = BetaClearThinking20251015EditParam::with_turns(3);
    println!("✓ Created clear request: keep last 3 turns\n");

    // 6. Wrapping in context management edit
    println!("6. Wrapping in context management edit:");
    let edit = ContextManagementEdit::clear_thinking(clear_request);
    println!("✓ Edit type: {}", edit.edit_type());
    match serde_json::to_string_pretty(&edit) {
        Ok(json) => {
            println!("✓ Edit request as JSON:");
            println!("{}\n", json);
        }
        Err(e) => println!("✗ Serialization error: {}\n", e),
    }

    // 7. Processing response
    println!("7. Processing context management response:");
    let response_json = r#"{
      "type": "clear_thinking_20251015",
      "cleared_input_tokens": 1230,
      "cleared_thinking_turns": 4
    }"#;

    match serde_json::from_str::<turboclaude::types::beta::BetaClearThinking20251015EditResponse>(
        response_json,
    ) {
        Ok(response) => {
            let response_wrapper = ContextManagementEditResponse::clear_thinking(response);
            println!("✓ Response received:");
            println!("  - Response type: {}", response_wrapper.response_type());
            println!(
                "  - Cleared {} input tokens",
                response_wrapper.cleared_input_tokens().unwrap_or(0)
            );
            println!(
                "  - Cleared {} thinking turns\n",
                response_wrapper.cleared_thinking_turns().unwrap_or(0)
            );
        }
        Err(e) => println!("✗ Failed to parse: {}\n", e),
    }

    // 8. Different clearing strategies
    println!("8. Available clearing strategies:");
    println!();
    println!("   Strategy A: Keep recent turns");
    let _strategy_a = BetaClearThinking20251015EditParam::with_turns(5);
    println!("   - Keeps last 5 turns of thinking");
    println!("   - Best for: Maintaining context while reducing tokens");
    println!();

    println!("   Strategy B: Keep all thinking");
    let _strategy_b = BetaClearThinking20251015EditParam::keep_all();
    println!("   - Preserves all thinking blocks");
    println!("   - Best for: Critical reasoning chains");
    println!();

    println!("   Strategy C: Clear all thinking");
    let _strategy_c = BetaClearThinking20251015EditParam::clear_all();
    println!("   - Removes all thinking blocks");
    println!("   - Best for: Starting fresh context");
    println!();

    // 9. Benefits of context management
    println!("9. Benefits of context management:");
    println!("   ✓ Reduces token consumption");
    println!("   ✓ Maintains conversation coherence");
    println!("   ✓ Keeps relevant reasoning");
    println!("   ✓ Improves API efficiency");
    println!("   ✓ Enables longer conversations");
    println!();

    // 10. Implementation patterns
    println!("10. Implementation patterns:");
    println!();
    println!("    Pattern 1: Periodic cleaning");
    println!("    - After every N turns, clear old thinking");
    println!("    - Keep last K turns for context");
    println!();

    println!("    Pattern 2: Token-based cleaning");
    println!("    - Monitor thinking tokens used");
    println!("    - Clear when exceeding threshold");
    println!();

    println!("    Pattern 3: Adaptive cleaning");
    println!("    - Analyze conversation complexity");
    println!("    - Adjust strategy based on needs");
    println!();

    println!("=== Context Management Example Complete ===");
}

// Helper trait for easier access to response fields
trait ClearThinkingResponseExt {
    fn cleared_input_tokens(&self) -> Option<u32>;
    fn cleared_thinking_turns(&self) -> Option<u32>;
}

impl ClearThinkingResponseExt for ContextManagementEditResponse {
    fn cleared_input_tokens(&self) -> Option<u32> {
        match self {
            ContextManagementEditResponse::ClearThinking(response) => {
                Some(response.cleared_input_tokens)
            }
        }
    }

    fn cleared_thinking_turns(&self) -> Option<u32> {
        match self {
            ContextManagementEditResponse::ClearThinking(response) => {
                Some(response.cleared_thinking_turns)
            }
        }
    }
}
