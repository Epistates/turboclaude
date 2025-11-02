//! Example demonstrating extended thinking with context management
//!
//! This example shows how to:
//! 1. Enable extended thinking for conversations
//! 2. Clear old thinking blocks to manage token usage
//! 3. Control which thinking turns to preserve

use turboclaude::types::beta::{BetaClearThinking20251015EditParam, ThinkingConfig};

fn main() {
    println!("=== Extended Thinking with Context Management ===\n");

    // 1. Configure extended thinking with a token budget
    println!("1. Creating thinking configuration:");
    let thinking_config = ThinkingConfig::new(3000);
    match thinking_config.validate() {
        Ok(_) => println!(
            "✓ Valid thinking config: {} budget tokens\n",
            thinking_config.budget_tokens
        ),
        Err(e) => println!("✗ Invalid config: {}\n", e),
    }

    // 2. Demonstrate clearing thinking with specific turn preservation
    println!("2. Clearing old thinking (keeping last 3 turns):");
    let clear_param = BetaClearThinking20251015EditParam::with_turns(3);
    println!("✓ Clear thinking param created");
    println!("  - Type: {}", clear_param.param_type);
    println!("  - Keep: Last 3 turns\n");

    // 3. Simulate response from clearing operation
    println!("3. Simulated response from clear thinking operation:");
    let response_json = r#"{
      "type": "clear_thinking_20251015",
      "cleared_input_tokens": 2048,
      "cleared_thinking_turns": 5
    }"#;

    match serde_json::from_str::<turboclaude::types::beta::BetaClearThinking20251015EditResponse>(
        response_json,
    ) {
        Ok(response) => {
            println!("✓ Successfully parsed response:");
            println!(
                "  - Cleared input tokens: {}",
                response.cleared_input_tokens
            );
            println!(
                "  - Cleared thinking turns: {}",
                response.cleared_thinking_turns
            );
            println!("  - Response type: {}\n", response.response_type);
        }
        Err(e) => println!("✗ Failed to parse response: {}\n", e),
    }

    // 4. Different clearing strategies
    println!("4. Different clearing strategies:");

    println!("\n   a) Keep specific number of turns:");
    let strategy1 = BetaClearThinking20251015EditParam::with_turns(5);
    println!("      Strategy: Keep last 5 thinking turns");

    println!("\n   b) Keep all thinking blocks:");
    let _strategy2 = BetaClearThinking20251015EditParam::keep_all();
    println!("      Strategy: Preserve all thinking");

    println!("\n   c) Clear all thinking blocks:");
    let strategy3 = BetaClearThinking20251015EditParam::clear_all();
    println!("      Strategy: Remove all thinking blocks\n");

    // 5. Serialization example
    println!("5. Serialization examples:");
    println!("\n   Strategy 1 (Keep 5 turns):");
    let json1 = serde_json::to_string_pretty(&strategy1).unwrap_or_default();
    println!("{}", json1);

    println!("\n   Strategy 3 (Clear all):");
    let json3 = serde_json::to_string_pretty(&strategy3).unwrap_or_default();
    println!("{}", json3);

    // 6. Usage pattern
    println!("\n6. Typical usage pattern:\n");
    println!("   // Start conversation with extended thinking");
    println!("   let config = ThinkingConfig::new(4000);");
    println!("   // ... continue conversation ...");
    println!("   // After many turns, clear old thinking to save tokens");
    println!("   let clear = BetaClearThinking20251015EditParam::with_turns(10);");
    println!("   // ... continue with fresh context ...\n");

    println!("=== Example Complete ===");
}
