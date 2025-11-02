/// Real-world Error Handling tests
///
/// Run with: cargo test --ignored real_world_errors

use turboclaude::{Client, Message, MessageRequest};
use crate::real_world::common::TestConfig;

#[tokio::test]
#[ignore]
async fn real_world_errors_invalid_api_key() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Testing: Invalid API key");

    let client = Client::new("sk-ant-invalid-key-12345");

    let result = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(100u32)
            .messages(vec![Message::user("Hello")])
            .build()?)
        .await;

    println!("✅ Got error (expected): {:?}", result);

    assert!(result.is_err(), "Should fail with invalid API key");

    let err = result.unwrap_err();
    let err_str = err.to_string().to_lowercase();

    assert!(
        err_str.contains("authentication") ||
        err_str.contains("401") ||
        err_str.contains("unauthorized"),
        "Error should mention authentication: {}",
        err
    );

    println!("✅ Error correctly indicates authentication failure");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_errors_invalid_model() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);

    println!("\n🧪 Testing: Invalid model name");

    let result = client.messages()
        .create(MessageRequest::builder()
            .model("invalid-model-xyz-does-not-exist")
            .max_tokens(100u32)
            .messages(vec![Message::user("Hello")])
            .build()?)
        .await;

    println!("✅ Got error (expected): {:?}", result);

    assert!(result.is_err(), "Should fail with invalid model");

    let err = result.unwrap_err();
    let err_str = err.to_string().to_lowercase();

    assert!(
        err_str.contains("model") ||
        err_str.contains("404") ||
        err_str.contains("not found"),
        "Error should mention model: {}",
        err
    );

    println!("✅ Error correctly indicates invalid model");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_errors_missing_required_field() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);

    println!("\n🧪 Testing: Missing required fields");

    // Try to create request with empty messages
    let result = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(100u32)
            .messages(vec![]) // Empty!
            .build()?)
        .await;

    println!("✅ Got error (expected): {:?}", result);

    assert!(result.is_err(), "Should fail with empty messages");

    let err = result.unwrap_err();
    println!("✅ Error: {}", err);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_errors_invalid_max_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);

    println!("\n🧪 Testing: Invalid max_tokens value");

    let result = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(0u32) // Invalid - must be > 0
            .messages(vec![Message::user("Hello")])
            .build()?)
        .await;

    println!("✅ Got error (expected): {:?}", result);

    assert!(result.is_err(), "Should fail with max_tokens=0");

    Ok(())
}

// NOTE: Tool and batch tests removed - these features require schema feature flag
// and batches API is not yet implemented. Add these tests when features are available.

#[tokio::test]
#[ignore]
async fn real_world_errors_timeout_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;

    println!("\n🧪 Testing: Client with very short timeout");

    // Create client with 1ms timeout (will definitely timeout)
    let client = Client::builder()
        .api_key(&config.api_key)
        .timeout(std::time::Duration::from_millis(1))
        .build()?;

    let result = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(100u32)
            .messages(vec![Message::user("Hello")])
            .build()?)
        .await;

    println!("✅ Got error (expected): {:?}", result);

    assert!(result.is_err(), "Should timeout with 1ms timeout");

    let err = result.unwrap_err();
    let err_str = err.to_string().to_lowercase();

    // Might be timeout or connection error
    assert!(
        err_str.contains("timeout") ||
        err_str.contains("timed out") ||
        err_str.contains("connection"),
        "Error should indicate timeout/connection issue: {}",
        err
    );

    println!("✅ Error correctly indicates timeout");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_errors_invalid_base_url() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;

    println!("\n🧪 Testing: Invalid base URL");

    let result = Client::builder()
        .api_key(&config.api_key)
        .base_url("https://invalid-domain-that-does-not-exist-xyz.com")
        .build();

    // URL validation should happen at build time or request time
    if let Ok(client) = result {
        let result = client.messages()
            .create(MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user("Hello")])
                .build()?)
            .await;

        println!("✅ Got error (expected): {:?}", result);
        assert!(result.is_err(), "Should fail with invalid base URL");

        let err = result.unwrap_err();
        println!("✅ Error: {}", err);
    } else {
        println!("✅ Client builder rejected invalid URL");
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_errors_empty_content() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);

    println!("\n🧪 Testing: Empty user message");

    let result = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(100u32)
            .messages(vec![Message::user("")]) // Empty content
            .build()?)
        .await;

    println!("✅ Result: {:?}", result);

    // API might accept empty message or reject it
    match result {
        Ok(_) => println!("✅ API accepted empty message"),
        Err(e) => println!("✅ API rejected empty message: {}", e),
    }

    Ok(())
}
