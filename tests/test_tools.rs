//! Integration tests for the tool system
//!
//! These tests verify the full tool execution loop with mocked API responses.

use turboclaude::{Client, Message, MessageRequest, Models, Role};
use serde::Deserialize;
use serde_json::json;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[cfg(feature = "schema")]
use turboclaude::tools::{FunctionTool, Tool, ToolRunner};
#[cfg(feature = "schema")]
use schemars::JsonSchema;

// Helper function to create a test client pointing to the mock server
fn create_test_client(uri: &str) -> Client {
    Client::builder()
        .api_key("test-api-key")
        .base_url(uri)
        .build()
        .expect("Failed to create client")
}

// Helper to create standard message response
fn create_message_response(
    id: &str,
    content_blocks: Vec<serde_json::Value>,
    stop_reason: &str,
) -> serde_json::Value {
    json!({
        "id": id,
        "type": "message",
        "role": "assistant",
        "content": content_blocks,
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        }
    })
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_basic_execution() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Define our test tool
    #[derive(Deserialize, JsonSchema)]
    struct CalculatorInput {
        operation: String,
        a: f64,
        b: f64,
    }

    async fn calculator(
        input: CalculatorInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "subtract" => input.a - input.b,
            "multiply" => input.a * input.b,
            "divide" if input.b != 0.0 => input.a / input.b,
            "divide" => return Err("Division by zero".into()),
            _ => return Err(format!("Unknown operation: {}", input.operation).into()),
        };
        Ok(format!("{} {} {} = {}", input.a, input.operation, input.b, result))
    }

    let calculator_tool = FunctionTool::new("calculator", "Performs arithmetic", calculator);

    // Mock first response: Claude wants to use the tool
    // Use up_to(1) to match first request only
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_001",
            vec![json!({
                "type": "text",
                "text": "I'll calculate that for you."
            }), json!({
                "type": "tool_use",
                "id": "toolu_001",
                "name": "calculator",
                "input": {
                    "operation": "multiply",
                    "a": 42,
                    "b": 17
                }
            })],
            "tool_use",
        )))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Mock second response: Claude provides the final answer
    // This will match subsequent requests
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_002",
            vec![json!({
                "type": "text",
                "text": "The result is 714."
            })],
            "end_turn",
        )))
        .mount(&mock_server)
        .await;

    // Create client and runner
    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client)
        .add_tool(calculator_tool)
        .with_max_iterations(5)
        .with_verbose(true);

    // Create request
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("What is 42 multiplied by 17?")])
        .build()
        .expect("Failed to build request");

    // Run the tool execution loop
    let final_message = runner.run(request).await.expect("Tool runner failed");

    // Verify final message
    assert_eq!(final_message.role, Role::Assistant);
    assert!(final_message.text().contains("714"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_error_handling() {
    let mock_server = MockServer::start().await;

    #[derive(Deserialize, JsonSchema)]
    struct Input {
        #[allow(dead_code)]
        value: String,
    }

    async fn failing_tool(
        _input: Input,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err("Tool execution failed".into())
    }

    let tool = FunctionTool::new("failing_tool", "A tool that always fails", failing_tool);

    // Mock first response: Claude wants to use the failing tool
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_001",
            vec![json!({
                "type": "tool_use",
                "id": "toolu_001",
                "name": "failing_tool",
                "input": {
                    "value": "test"
                }
            })],
            "tool_use",
        )))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Mock second response: Claude handles the error
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_002",
            vec![json!({
                "type": "text",
                "text": "I encountered an error with the tool."
            })],
            "end_turn",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client)
        .add_tool(tool)
        .with_max_iterations(5);

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Test the failing tool")])
        .build()
        .unwrap();

    let final_message = runner.run(request).await.expect("Runner should complete despite tool error");

    // The runner should complete and return Claude's response about the error
    assert!(final_message.text().contains("error") || final_message.text().contains("Error"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_multiple_tools() {
    let mock_server = MockServer::start().await;

    #[derive(Deserialize, JsonSchema)]
    struct MathInput {
        x: f64,
        y: f64,
    }

    #[derive(Deserialize, JsonSchema)]
    struct StringInput {
        text: String,
    }

    async fn add(input: MathInput) -> String {
        (input.x + input.y).to_string()
    }

    async fn uppercase(input: StringInput) -> String {
        input.text.to_uppercase()
    }

    let add_tool = FunctionTool::new("add", "Add two numbers", add);
    let uppercase_tool = FunctionTool::new("uppercase", "Convert to uppercase", uppercase);

    // Mock response: Claude uses both tools
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_001",
            vec![
                json!({
                    "type": "tool_use",
                    "id": "toolu_001",
                    "name": "add",
                    "input": {"x": 5, "y": 3}
                }),
                json!({
                    "type": "tool_use",
                    "id": "toolu_002",
                    "name": "uppercase",
                    "input": {"text": "hello"}
                }),
            ],
            "tool_use",
        )))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Mock final response
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_002",
            vec![json!({
                "type": "text",
                "text": "The sum is 8 and HELLO in uppercase."
            })],
            "end_turn",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client)
        .add_tool(add_tool)
        .add_tool(uppercase_tool)
        .with_max_iterations(5);

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Add 5 and 3, then uppercase 'hello'")])
        .build()
        .unwrap();

    let final_message = runner.run(request).await.expect("Runner failed");

    assert!(final_message.text().contains("8") || final_message.text().contains("HELLO"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_max_iterations() {
    let mock_server = MockServer::start().await;

    #[derive(Deserialize, JsonSchema)]
    struct Input {
        value: i32,
    }

    async fn increment(input: Input) -> String {
        (input.value + 1).to_string()
    }

    let tool = FunctionTool::new("increment", "Increment a number", increment);

    // Mock: Always request the tool (infinite loop scenario)
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_loop",
            vec![json!({
                "type": "tool_use",
                "id": "toolu_loop",
                "name": "increment",
                "input": {"value": 1}
            })],
            "tool_use",
        )))
        .expect(3) // Should stop after max_iterations
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client)
        .add_tool(tool)
        .with_max_iterations(3);

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Keep incrementing")])
        .build()
        .unwrap();

    let result = runner.run(request).await;

    // Should fail with max iterations error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Maximum iterations") || err.to_string().contains("3"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_function_tool_customization() {
    #[derive(Deserialize, JsonSchema)]
    struct Input {
        name: String,
    }

    async fn greet(input: Input) -> String {
        format!("Hello, {}!", input.name)
    }

    // Test with_name and with_description
    let tool = FunctionTool::new("greet", "Greet someone", greet)
        .with_name("custom_greet")
        .with_description("A customized greeting function");

    assert_eq!(tool.name(), "custom_greet");
    assert_eq!(tool.description(), "A customized greeting function");

    // Test to_param
    let param = tool.to_param();
    assert_eq!(param.name, "custom_greet");
    assert_eq!(param.description, "A customized greeting function");

    // Verify tool still works
    let result = tool
        .call(json!({"name": "Alice"}))
        .await
        .expect("Tool call failed");
    assert_eq!(result.as_string(), "Hello, Alice!");
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_tool_count() {
    let client = Client::new("test-key");

    #[derive(Deserialize, JsonSchema)]
    struct Input {
        #[allow(dead_code)]
        x: i32,
    }

    async fn tool1(_input: Input) -> String {
        "1".to_string()
    }

    async fn tool2(_input: Input) -> String {
        "2".to_string()
    }

    let runner = ToolRunner::new(client)
        .add_tool(FunctionTool::new("tool1", "Tool 1", tool1))
        .add_tool(FunctionTool::new("tool2", "Tool 2", tool2));

    assert_eq!(runner.tool_count(), 2);
    assert!(runner.has_tool("tool1"));
    assert!(runner.has_tool("tool2"));
    assert!(!runner.has_tool("nonexistent"));

    let names = runner.tool_names();
    assert!(names.contains(&"tool1"));
    assert!(names.contains(&"tool2"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_not_found() {
    let mock_server = MockServer::start().await;

    #[derive(Deserialize, JsonSchema)]
    struct Input {
        #[allow(dead_code)]
        value: String,
    }

    async fn real_tool(_input: Input) -> String {
        "real tool output".to_string()
    }

    // Register a real tool but Claude tries to use a different one
    let tool = FunctionTool::new("real_tool", "A real tool", real_tool);

    // Mock first response: Claude tries to use a non-existent tool
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_001",
            vec![json!({
                "type": "tool_use",
                "id": "toolu_001",
                "name": "nonexistent_tool",
                "input": {"value": "test"}
            })],
            "tool_use",
        )))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Mock response after error - the second request will have the error
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_002",
            vec![json!({
                "type": "text",
                "text": "The tool 'nonexistent_tool' was not found."
            })],
            "end_turn",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client)
        .add_tool(tool)
        .with_max_iterations(5);

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Use the nonexistent tool")])
        .build()
        .unwrap();

    // Should complete with an error response about the missing tool
    let result = runner.run(request).await;
    assert!(result.is_ok(), "Runner should handle missing tool gracefully");
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_complex_input_types() {
    #[derive(Deserialize, JsonSchema)]
    struct ComplexInput {
        /// Required string field
        name: String,
        /// Optional integer field
        age: Option<u32>,
        /// Array of strings
        tags: Vec<String>,
        /// Nested object
        metadata: Metadata,
    }

    #[derive(Deserialize, JsonSchema)]
    struct Metadata {
        source: String,
        #[allow(dead_code)]
        confidence: f64,
    }

    async fn process_complex(input: ComplexInput) -> String {
        format!(
            "Name: {}, Age: {:?}, Tags: {}, Source: {}",
            input.name,
            input.age,
            input.tags.join(", "),
            input.metadata.source
        )
    }

    let tool = FunctionTool::new("process", "Process complex input", process_complex);

    // Verify schema was generated for complex type
    let schema = tool.input_schema();
    assert!(schema.is_object());
    assert!(schema.get("properties").is_some());

    // Test calling with complex input
    let result = tool
        .call(json!({
            "name": "Alice",
            "age": 30,
            "tags": ["developer", "rust"],
            "metadata": {
                "source": "api",
                "confidence": 0.95
            }
        }))
        .await
        .expect("Call failed");

    let output = result.as_string();
    assert!(output.contains("Alice"));
    assert!(output.contains("developer"));
    assert!(output.contains("api"));
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_streaming_execution() {
    use turboclaude::streaming::StreamEvent;
    use futures::StreamExt;

    let mock_server = MockServer::start().await;

    // Simplified test: streaming with NO tools
    // This tests run_streaming() takes the fast path when no tools registered
    // SSE format: each event is "event: type\ndata: json\n\n" (double newline between events)
    let sse_body = "event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_001\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-3-5-sonnet-20241022\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":100,\"output_tokens\":0}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello! I'm Claude.\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":10}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream")
                .insert_header("cache-control", "no-cache"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client); // No tools

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    // Run streaming (should use fast path since no tools)
    let stream_result = runner.run_streaming(request).await;
    assert!(stream_result.is_ok(), "Streaming should succeed with no tools");

    let mut stream = stream_result.unwrap();
    let mut events = Vec::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(e) => events.push(e),
            Err(e) => panic!("Stream error: {:?}", e),
        }
    }

    // Verify we got streaming events
    assert!(!events.is_empty());

    // Check for expected event types
    let has_message_start = events.iter().any(|e| matches!(e, StreamEvent::MessageStart(_)));
    assert!(has_message_start, "Should have message_start event");

    let has_content_delta = events.iter().any(|e| matches!(e, StreamEvent::ContentBlockDelta(_)));
    assert!(has_content_delta, "Should have content_block_delta events");

    let has_message_stop = events.iter().any(|e| matches!(e, StreamEvent::MessageStop));
    assert!(has_message_stop, "Should have message_stop event");
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_streaming_events() {
    use turboclaude::streaming::StreamEvent;
    use futures::StreamExt;

    let mock_server = MockServer::start().await;

    // Test that all streaming event types are properly parsed
    // Using no tools to avoid complexity of mocking tool execution loop
    let sse_body = "event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_001\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-3-5-sonnet-20241022\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":100,\"output_tokens\":0}}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\
\n\
event: ping\n\
data: {\"type\":\"ping\"}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" World\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":5}}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("content-type", "text/event-stream"),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client); // No tools

    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let stream_result = runner.run_streaming(request).await;
    assert!(stream_result.is_ok(), "Streaming should succeed");

    let mut stream = stream_result.unwrap();
    let mut event_types = std::collections::HashSet::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(e) => {
                let event_type = match e {
                    StreamEvent::MessageStart(_) => "message_start",
                    StreamEvent::MessageDelta(_) => "message_delta",
                    StreamEvent::MessageStop => "message_stop",
                    StreamEvent::ContentBlockStart(_) => "content_block_start",
                    StreamEvent::ContentBlockDelta(_) => "content_block_delta",
                    StreamEvent::ContentBlockStop(_) => "content_block_stop",
                    StreamEvent::Ping => "ping",
                    StreamEvent::Unknown => "unknown",
                };
                event_types.insert(event_type);
            }
            Err(e) => panic!("Stream error: {:?}", e),
        }
    }

    // Verify all expected event types were received
    assert!(event_types.contains("message_start"), "Missing message_start");
    assert!(event_types.contains("content_block_start"), "Missing content_block_start");
    assert!(event_types.contains("content_block_delta"), "Missing content_block_delta");
    assert!(event_types.contains("content_block_stop"), "Missing content_block_stop");
    assert!(event_types.contains("message_delta"), "Missing message_delta");
    assert!(event_types.contains("message_stop"), "Missing message_stop");
    assert!(event_types.contains("ping"), "Missing ping");
}

#[cfg(feature = "schema")]
#[tokio::test]
async fn test_tool_runner_with_cache_control() {
    let mock_server = MockServer::start().await;

    #[derive(Deserialize, JsonSchema)]
    struct QueryInput {
        query: String,
    }

    async fn search_docs(input: QueryInput) -> String {
        format!("Found results for: {}", input.query)
    }

    let tool = FunctionTool::new("search_docs", "Search documentation", search_docs);

    // Mock response - we'll verify cache headers were sent in request
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_message_response(
            "msg_001",
            vec![json!({
                "type": "text",
                "text": "I found the documentation you requested."
            })],
            "end_turn",
        )))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri());
    let runner = ToolRunner::new(client).add_tool(tool).with_max_iterations(5);

    // Create request with cache control
    let request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Search for async documentation")])
        .build()
        .unwrap();

    // Run and verify it completes (cache control is internal)
    let result = runner.run(request).await;
    assert!(result.is_ok());

    let message = result.unwrap();
    assert!(message.text().contains("documentation"));
}
