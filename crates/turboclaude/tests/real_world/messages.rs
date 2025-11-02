use crate::real_world::common::{TestConfig, TestMetrics};
use turboclaude::types::StopReason;
/// Real-world Messages API tests
///
/// Run with: cargo test --ignored real_world_messages
use turboclaude::{Client, Message, MessageRequest};

#[tokio::test]
#[ignore]
async fn real_world_messages_simple() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Simple message request");

    let message = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user(
                    "What is 2+2? Answer with just the number.",
                )])
                .build()?,
        )
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    let response = message.text();
    println!("✅ Response: {}", response);

    assert!(
        response.contains("4"),
        "Expected '4' in response, got: {}",
        response
    );
    assert_eq!(message.stop_reason, Some(StopReason::EndTurn));
    assert!(message.usage.input_tokens > 0, "Expected input tokens");
    assert!(message.usage.output_tokens > 0, "Expected output tokens");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_messages_conversation() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Multi-turn conversation");

    // First message
    let msg1 = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user("My name is Alice.")])
                .build()?,
        )
        .await?;

    println!("✅ Turn 1: {}", msg1.text());

    // Second message with context
    let msg2 = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![
                    Message::user("My name is Alice."),
                    Message::assistant(&msg1.text()),
                    Message::user("What is my name?"),
                ])
                .build()?,
        )
        .await?;

    metrics.finish();
    metrics.input_tokens = msg2.usage.input_tokens;
    metrics.output_tokens = msg2.usage.output_tokens;

    let response = msg2.text();
    println!("✅ Turn 2: {}", response);

    assert!(
        response.to_lowercase().contains("alice"),
        "Expected 'alice' in response, got: {}",
        response
    );

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_messages_system_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: System prompt");

    let message = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(50u32)
                .system("You are a pirate. Always respond like a pirate.")
                .messages(vec![Message::user("Hello!")])
                .build()?,
        )
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    let response = message.text();
    println!("✅ Response: {}", response);

    // Check for pirate-like language (common pirate words)
    let pirate_words = ["ahoy", "matey", "arr", "ye", "aye"];
    let has_pirate_language = pirate_words
        .iter()
        .any(|word| response.to_lowercase().contains(word));

    assert!(
        has_pirate_language,
        "Expected pirate language in response, got: {}",
        response
    );

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_messages_haiku() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Claude 3 Haiku model");

    let message = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-haiku-20241022")
                .max_tokens(50u32)
                .messages(vec![Message::user("Say 'hello' in Spanish")])
                .build()?,
        )
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    let response = message.text();
    println!("✅ Response: {}", response);

    assert!(
        response.to_lowercase().contains("hola"),
        "Expected 'hola' in response, got: {}",
        response
    );
    assert_eq!(message.model, "claude-3-5-haiku-20241022");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_messages_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Response metadata validation");

    let message = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(20u32)
                .messages(vec![Message::user("Hi")])
                .build()?,
        )
        .await?;

    metrics.finish();

    println!("✅ Message ID: {}", message.id);
    println!("✅ Model: {}", message.model);
    println!("✅ Role: {:?}", message.role);
    println!("✅ Stop Reason: {:?}", message.stop_reason);
    println!("✅ Usage: {:?}", message.usage);

    // Validate metadata
    assert!(!message.id.is_empty(), "Message ID should not be empty");
    assert_eq!(message.model, "claude-3-5-sonnet-20241022");
    assert_eq!(message.role, turboclaude::types::Role::Assistant);
    assert!(
        message.stop_reason.is_some(),
        "Stop reason should be present"
    );
    assert!(message.usage.input_tokens > 0, "Input tokens should be > 0");
    assert!(
        message.usage.output_tokens > 0,
        "Output tokens should be > 0"
    );

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_messages_max_tokens_limit() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Max tokens limit");

    let message = client
        .messages()
        .create(
            MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(10u32)
                .messages(vec![Message::user("Write a long story about a dragon")])
                .build()?,
        )
        .await?;

    metrics.finish();
    metrics.output_tokens = message.usage.output_tokens;

    println!("✅ Response: {}", message.text());
    println!("✅ Output tokens: {}", message.usage.output_tokens);

    // With max_tokens=10, we should hit the limit
    assert!(
        message.usage.output_tokens <= 12,
        "Expected ~10 tokens or less, got: {}",
        message.usage.output_tokens
    );
    assert_eq!(
        message.stop_reason,
        Some(StopReason::MaxTokens),
        "Expected max_tokens stop reason"
    );

    metrics.print_summary();
    Ok(())
}
