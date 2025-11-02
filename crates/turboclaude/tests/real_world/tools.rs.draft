/// Real-world Tool Use tests
///
/// Run with: cargo test --ignored real_world_tools

use turboclaude::{Client, Message, MessageRequest};
use turboclaude::types::{StopReason, ContentBlock};
use turboclaude::tools::{Tool, ToolChoice};
use serde_json::json;
use crate::real_world::common::{TestConfig, TestMetrics};

#[tokio::test]
#[ignore]
async fn real_world_tools_basic() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Basic tool use");

    let calculator = Tool::new(
        "calculator",
        "Perform basic arithmetic operations",
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        })
    );

    let message = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(512u32)
            .messages(vec![Message::user("What is 15 multiplied by 7?")])
            .tools(vec![calculator])
            .tool_choice(ToolChoice::Auto)
            .build()?)
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    println!("✅ Stop reason: {:?}", message.stop_reason);
    assert_eq!(message.stop_reason, Some(StopReason::ToolUse), "Expected tool_use stop reason");

    // Find the tool use block
    let tool_use = message.content.iter()
        .find_map(|block| match block {
            ContentBlock::ToolUse(tu) => Some(tu),
            _ => None
        })
        .expect("Expected tool_use block");

    println!("✅ Tool called: {}", tool_use.name);
    println!("✅ Tool input: {}", tool_use.input);

    assert_eq!(tool_use.name, "calculator");

    let input: serde_json::Value = serde_json::from_str(&tool_use.input)?;
    println!("✅ Parsed input: {}", serde_json::to_string_pretty(&input)?);

    assert_eq!(input["operation"], "multiply");
    assert_eq!(input["a"], 15);
    assert_eq!(input["b"], 7);

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_tools_with_response() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Tool use with result continuation");

    let get_weather = Tool::new(
        "get_weather",
        "Get the current weather for a location",
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        })
    );

    // First request - Claude calls tool
    let msg1 = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(512u32)
            .messages(vec![Message::user("What's the weather in San Francisco?")])
            .tools(vec![get_weather.clone()])
            .build()?)
        .await?;

    println!("✅ First response - stop reason: {:?}", msg1.stop_reason);
    assert_eq!(msg1.stop_reason, Some(StopReason::ToolUse));

    let tool_use = msg1.content.iter()
        .find_map(|block| match block {
            ContentBlock::ToolUse(tu) => Some(tu),
            _ => None
        })
        .expect("Expected tool_use block");

    println!("✅ Tool called: {}", tool_use.name);
    let input: serde_json::Value = serde_json::from_str(&tool_use.input)?;
    println!("✅ Input: {}", serde_json::to_string_pretty(&input)?);

    // Simulate tool execution
    let weather_result = json!({
        "temperature": 68,
        "condition": "sunny",
        "humidity": 65
    });

    // Second request - provide tool result
    let msg2 = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(512u32)
            .messages(vec![
                Message::user("What's the weather in San Francisco?"),
                Message::with_content(msg1.content.clone()), // Claude's tool use
                Message::tool_result(&tool_use.id, &serde_json::to_string(&weather_result)?),
            ])
            .tools(vec![get_weather])
            .build()?)
        .await?;

    metrics.finish();
    metrics.input_tokens = msg2.usage.input_tokens;
    metrics.output_tokens = msg2.usage.output_tokens;

    let response = msg2.text();
    println!("✅ Final response: {}", response);

    // Verify Claude incorporated the weather data
    assert!(
        response.to_lowercase().contains("68") || response.to_lowercase().contains("sunny"),
        "Expected weather info in response: {}",
        response
    );

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_tools_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Multiple tool definitions");

    let get_time = Tool::new(
        "get_time",
        "Get the current time",
        json!({
            "type": "object",
            "properties": {
                "timezone": {"type": "string"}
            },
            "required": ["timezone"]
        })
    );

    let get_weather = Tool::new(
        "get_weather",
        "Get weather information",
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        })
    );

    let search_web = Tool::new(
        "search_web",
        "Search the web",
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        })
    );

    let message = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(512u32)
            .messages(vec![Message::user("What time is it in Tokyo?")])
            .tools(vec![get_time, get_weather, search_web])
            .build()?)
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    println!("✅ Stop reason: {:?}", message.stop_reason);

    if message.stop_reason == Some(StopReason::ToolUse) {
        let tool_use = message.content.iter()
            .find_map(|block| match block {
                ContentBlock::ToolUse(tu) => Some(tu),
                _ => None
            })
            .expect("Expected tool_use block");

        println!("✅ Tool selected: {}", tool_use.name);
        println!("✅ Input: {}", tool_use.input);

        assert_eq!(tool_use.name, "get_time", "Expected get_time tool for time query");

        let input: serde_json::Value = serde_json::from_str(&tool_use.input)?;
        assert!(input["timezone"].as_str().unwrap().to_lowercase().contains("tokyo"));
    }

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_tools_forced() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Forced tool use");

    let calculator = Tool::new(
        "calculator",
        "Perform calculations",
        json!({
            "type": "object",
            "properties": {
                "expression": {"type": "string"}
            },
            "required": ["expression"]
        })
    );

    let message = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(512u32)
            .messages(vec![Message::user("Hi, how are you?")]) // Not a calculator question!
            .tools(vec![calculator.clone()])
            .tool_choice(ToolChoice::Tool { name: "calculator".to_string() }) // Force it
            .build()?)
        .await?;

    metrics.finish();

    println!("✅ Stop reason: {:?}", message.stop_reason);
    assert_eq!(message.stop_reason, Some(StopReason::ToolUse));

    let tool_use = message.content.iter()
        .find_map(|block| match block {
            ContentBlock::ToolUse(tu) => Some(tu),
            _ => None
        })
        .expect("Expected tool_use block");

    println!("✅ Tool called: {}", tool_use.name);
    assert_eq!(tool_use.name, "calculator", "Should have used calculator even though question wasn't about math");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_tools_complex_schema() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Complex nested schema");

    let create_user = Tool::new(
        "create_user",
        "Create a new user account",
        json!({
            "type": "object",
            "properties": {
                "username": {"type": "string"},
                "email": {"type": "string"},
                "profile": {
                    "type": "object",
                    "properties": {
                        "first_name": {"type": "string"},
                        "last_name": {"type": "string"},
                        "age": {"type": "integer"},
                        "interests": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["first_name", "last_name"]
                }
            },
            "required": ["username", "email", "profile"]
        })
    );

    let message = client.messages()
        .create(MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(1024u32)
            .messages(vec![
                Message::user("Create a user account for Alice Smith (alice@example.com), age 30, interested in hiking and photography")
            ])
            .tools(vec![create_user])
            .build()?)
        .await?;

    metrics.finish();
    metrics.input_tokens = message.usage.input_tokens;
    metrics.output_tokens = message.usage.output_tokens;

    assert_eq!(message.stop_reason, Some(StopReason::ToolUse));

    let tool_use = message.content.iter()
        .find_map(|block| match block {
            ContentBlock::ToolUse(tu) => Some(tu),
            _ => None
        })
        .expect("Expected tool_use block");

    let input: serde_json::Value = serde_json::from_str(&tool_use.input)?;
    println!("✅ Parsed complex input:\n{}", serde_json::to_string_pretty(&input)?);

    // Verify nested structure
    assert!(input["email"].as_str().unwrap().contains("alice"));
    assert!(input["profile"].is_object());
    assert_eq!(input["profile"]["first_name"], "Alice");
    assert_eq!(input["profile"]["last_name"], "Smith");
    assert!(input["profile"]["interests"].is_array());

    metrics.print_summary();
    Ok(())
}
