//! Type translation between turboclaude and AWS Bedrock formats
//!
//! # Overview
//!
//! This module handles bidirectional translation between turboclaude's types
//! and AWS Bedrock's Converse API types. It serves as an adapter layer between
//! two different type systems, allowing the SDK to present a unified interface
//! to users while supporting multiple backend providers.
//!
//! # Key Concepts
//!
//! ## Message Flow
//!
//! For non-streaming requests:
//! ```text
//! MessageRequest (turboclaude)
//!   ↓ [translate_messages, translate_system_prompt, etc.]
//! ConverseRequest (Bedrock API)
//!   ↓ [send to Bedrock]
//! ConverseOutput (Bedrock API)
//!   ↓ [translate_response, translate_bedrock_content_block]
//! Message (turboclaude)
//! ```
//!
//! For streaming requests:
//! ```text
//! MessageRequest (turboclaude)
//!   ↓ [translate_messages, translate_system_prompt, etc.]
//! ConverseStreamRequest (Bedrock API)
//!   ↓ [send to Bedrock, receive event stream]
//! Event Stream (Bedrock API native format)
//!   ↓ [translate_stream: converts to SSE format]
//! Server-Sent Events (standard HTTP format)
//!   ↓ [consumed by MessageStream in streaming.rs]
//! Message (turboclaude, via event accumulation)
//! ```
//!
//! ## Type System Differences
//!
//! ### serde_json::Value ↔ aws_smithy_types::Document
//!
//! Bedrock uses AWS's proprietary `Document` type for structured JSON data (tool inputs),
//! while turboclaude uses the standard `serde_json::Value`. The conversion functions
//! `json_value_to_document` and `document_to_json_value` handle this impedance mismatch.
//!
//! **Why this matters for tool use:**
//! - Tool schemas are defined as `serde_json::Value` in turboclaude (standard JSON Schema)
//! - Bedrock's Converse API expects tool input schemas as `Document` types
//! - Tool use responses from Bedrock come back as `Document` types
//! - We convert back to `serde_json::Value` for consistency
//!
//! ### Streaming Format Conversion
//!
//! Bedrock's ConverseStream API returns events in its native format (channel-based),
//! not Server-Sent Events (SSE). The `translate_stream` function converts these to SSE
//! for compatibility with turboclaude's standard streaming implementation.
//!
//! **Why this is necessary:**
//! - turboclaude's `MessageStream` expects a stream of `Bytes` containing SSE-formatted data
//! - SSE is a standard HTTP protocol, making it interoperable and testable
//! - The conversion allows Bedrock to use the same streaming infrastructure as Anthropic's API
//!
//! ## Limitations and Trade-offs
//!
//! ### URL Document Sources
//!
//! Bedrock's Converse API doesn't support URL-based document sources (e.g., `DocumentSource::URL`).
//! Users must provide documents as base64-encoded bytes or plain text.
//!
//! **Future improvement:** Fetch documents from URLs before sending to Bedrock, but this
//! adds complexity and network round-trips.
//!
//! ### Message IDs
//!
//! Bedrock doesn't return message IDs in responses. We generate UUIDs (`uuid::Uuid::new_v4()`)
//! to maintain compatibility with turboclaude's `Message` type, which expects an `id` field.
//!
//! ### Stop Sequences
//!
//! Bedrock doesn't return the actual stop sequence that triggered the stop. We track the
//! stop reason but set `stop_sequence` to `None` in responses.
//!
//! # Example: Non-Streaming Message
//!
//! ```ignore
//! use turboclaude::types::MessageRequest;
//! use turboclaude::Client;
//!
//! let request = MessageRequest::builder()
//!     .model("anthropic.claude-3-sonnet-20240229-v1:0")
//!     .max_tokens(1024)
//!     .messages(vec![Message::user("Hello!")])
//!     .build()?;
//!
//! // Behind the scenes:
//! // 1. translate_messages() converts turboclaude Message to Bedrock Message
//! // 2. converse_non_streaming() sends to Bedrock API
//! // 3. translate_response() converts Bedrock response back to turboclaude Message
//! let response = client.messages().create(request).await?;
//! ```
//!
//! # Example: Streaming Message with Tools
//!
//! ```ignore
//! let request = MessageRequest::builder()
//!     .model("anthropic.claude-3-sonnet-20240229-v1:0")
//!     .max_tokens(1024)
//!     .messages(vec![Message::user("Call a tool")])
//!     .tools(vec![my_tool])
//!     .stream(true)
//!     .build()?;
//!
//! // Behind the scenes:
//! // 1. translate_messages() + translate_tool_config() prepare request
//! // 2. converse_streaming() sends to Bedrock and receives event stream
//! // 3. translate_stream() converts Bedrock events to SSE format
//! // 4. MessageStream accumulates events from SSE stream
//! let mut stream = client.messages().stream(request).await?;
//! ```

use aws_sdk_bedrockruntime::{
    Client as BedrockRuntimeClient,
    primitives::Blob,
    types::{
        ContentBlock as BedrockContentBlock, ConversationRole, InferenceConfiguration,
        Message as BedrockMessage, SystemContentBlock, Tool as BedrockTool, ToolConfiguration,
        ToolInputSchema, ToolSpecification,
    },
};
use bytes::Bytes;
use futures::Stream;
use serde_json::Value as JsonValue;
use std::pin::Pin;

use crate::{
    error::Result,
    types::{
        ContentBlock, ContentBlockParam, Message, MessageParam, MessageRequest, Role, StopReason,
        SystemPrompt, SystemPromptBlock, Tool, ToolChoice, Usage,
    },
};

use super::{error::BedrockError, http::BedrockHttpProvider};

/// Send a non-streaming message request to AWS Bedrock's Converse API.
///
/// # Overview
///
/// This function translates a turboclaude `MessageRequest` into Bedrock's API format,
/// sends it to the Converse API, and translates the response back to turboclaude's `Message` type.
///
/// # Arguments
///
/// * `bedrock` - The AWS Bedrock SDK client (must be properly configured with credentials)
/// * `request` - The message request containing model, messages, and optional parameters
///
/// # Returns
///
/// A turboclaude `Message` containing the model's response, or an error if the request fails.
///
/// # Translation Process
///
/// 1. **Model ID Normalization**: Bedrock uses full model ARNs or cross-region inference aliases.
///    The `normalize_model_id` function converts user-friendly names to Bedrock format.
///
/// 2. **Message Translation**: `translate_messages` converts turboclaude's message format to Bedrock's,
///    handling all content types (text, images, documents, tool results).
///
/// 3. **System Prompt**: If provided, `translate_system_prompt` converts it to Bedrock's format.
///    Bedrock system prompts use `SystemContentBlock::Text`, not full `SystemPromptBlock` support.
///
/// 4. **Inference Configuration**: Maps turboclaude parameters to Bedrock's `InferenceConfiguration`:
///    - `max_tokens` (u32) → `max_tokens` (i32)
///    - `temperature` (optional) → included if present
///    - `top_p` (optional) → included if present
///    - `stop_sequences` (optional) → included if present
///
/// 5. **Tool Configuration**: If tools are provided, `translate_tool_config` converts them.
///    This includes translating tool schemas and tool choice (auto/any/specific).
///
/// 6. **API Call**: Sends the constructed request to Bedrock's Converse API.
///
/// 7. **Response Translation**: `translate_response` converts Bedrock's response back to turboclaude's format.
///
/// # Parameters Not Translated
///
/// - `top_k`: Not supported by Bedrock's Converse API
/// - `extended_thinking` / `thinking`: Beta features not available on Bedrock
/// - Model-specific parameters: Bedrock uses a different parameter schema
///
/// # Errors
///
/// Returns `BedrockError` variants for:
/// - `Service`: Bedrock API errors (invalid model, quota exceeded, etc.)
/// - `Translation`: Type conversion failures between turboclaude and Bedrock formats
/// - Network errors from the AWS SDK
///
/// # Example
///
/// ```ignore
/// use turboclaude::types::{MessageRequest, Message};
/// use aws_sdk_bedrockruntime::Client as BedrockClient;
///
/// let bedrock = BedrockClient::new(&config);
/// let request = MessageRequest::builder()
///     .model("claude-3-5-sonnet-20241022")
///     .max_tokens(1024)
///     .messages(vec![Message::user("What is 2+2?")])
///     .build()?;
///
/// let response = converse_non_streaming(&bedrock, &request).await?;
/// println!("Response: {}", response.text());
/// ```
pub async fn converse_non_streaming(
    bedrock: &BedrockRuntimeClient,
    request: &MessageRequest,
) -> Result<Message> {
    let model_id = BedrockHttpProvider::normalize_model_id(&request.model);

    // Transform messages to Bedrock format
    let bedrock_messages = translate_messages(&request.messages)?;

    // Build Bedrock request
    let mut bedrock_request = bedrock
        .converse()
        .model_id(model_id.clone())
        .set_messages(Some(bedrock_messages));

    // Add system prompt if present
    if let Some(system) = &request.system {
        let system_blocks = translate_system_prompt(system);
        bedrock_request = bedrock_request.set_system(Some(system_blocks));
    }

    // Set inference configuration
    let mut inference_config =
        InferenceConfiguration::builder().max_tokens(request.max_tokens as i32);

    if let Some(temp) = request.temperature {
        inference_config = inference_config.temperature(temp);
    }
    if let Some(top_p) = request.top_p {
        inference_config = inference_config.top_p(top_p);
    }
    if let Some(stop_seqs) = &request.stop_sequences {
        inference_config = inference_config.set_stop_sequences(Some(stop_seqs.clone()));
    }

    bedrock_request = bedrock_request.inference_config(inference_config.build());

    // Add tools if present
    if let Some(tools) = &request.tools {
        let tool_config = translate_tool_config(tools, request.tool_choice.as_ref())?;
        bedrock_request = bedrock_request.tool_config(tool_config);
    }

    // Handle top_k via additional_model_request_fields
    if let Some(top_k) = request.top_k {
        let additional_fields = serde_json::json!({ "top_k": top_k });
        let additional_fields_doc = json_value_to_document(&additional_fields)?;
        bedrock_request =
            bedrock_request.additional_model_request_fields(additional_fields_doc);
    }

    // Send request
    let response = bedrock_request
        .send()
        .await
        .map_err(|e| BedrockError::Service(format!("Converse API error: {}", e)))?;

    // Transform response back to turboclaude format
    translate_response(response, &model_id)
}

/// Send a streaming message request to AWS Bedrock's ConverseStream API.
///
/// # Overview
///
/// This function sends a message request to Bedrock's ConverseStream API and returns
/// a stream of bytes containing Server-Sent Events (SSE). This allows clients to receive
/// the model's response in real-time as it's generated.
///
/// # Arguments
///
/// * `bedrock` - The AWS Bedrock SDK client (must be properly configured with credentials)
/// * `request` - The message request (same format as `converse_non_streaming`)
///
/// # Returns
///
/// A boxed stream of `Result<Bytes>` containing SSE-formatted data, which can be consumed
/// by turboclaude's `MessageStream` to accumulate the response.
///
/// # Streaming Process
///
/// The function follows the same request translation and configuration steps as
/// `converse_non_streaming`, with one key difference:
///
/// 1. **Request Setup**: Identical to non-streaming (translate messages, system prompt, tools, etc.)
/// 2. **ConverseStream Call**: Instead of `converse()`, calls `converse_stream()`
/// 3. **Event Stream Translation**: `translate_stream` converts Bedrock's native event format
///    to SSE-formatted bytes that turboclaude's streaming infrastructure understands
/// 4. **Stream Return**: Returns a boxed stream that yields `Result<Bytes>` tuples
///
/// # SSE Format
///
/// The returned stream contains events in the following SSE format:
///
/// ```text
/// event: message_start
/// data: {"type":"message_start",...}
///
/// event: content_block_delta
/// data: {"type":"content_block_delta","index":0,"delta":{"text":"..."},"...}
///
/// event: message_stop
/// data: {"type":"message_stop"}
/// ```
///
/// Each event is separated by a blank line (`\n\n`). Text in delta events is JSON-escaped.
///
/// # Event Types Translated
///
/// - `BedrockStreamEvent::ContentBlockDelta` → `event: content_block_delta`
/// - `BedrockStreamEvent::MessageStart` → `event: message_start`
/// - `BedrockStreamEvent::MessageStop` → `event: message_stop`
/// - `BedrockStreamEvent::Metadata` → `event: message_delta` (with usage info)
/// - Unknown types → Skipped (empty bytes)
///
/// # Streaming Differences from Non-Streaming
///
/// Bedrock doesn't provide all metadata in streaming responses:
/// - **Message ID**: Not available until metadata event
/// - **Full Stop Reason**: Limited information in streaming
/// - **Tool Use**: Returned as partial JSON deltas, reconstructed by `MessageStream`
///
/// # Errors
///
/// Returns errors for:
/// - Stream errors (network disconnection, Bedrock service errors)
/// - Response parsing errors (malformed Bedrock events)
/// - The stream may return `Err` items that must be handled by the consumer
///
/// # Example
///
/// ```ignore
/// use turboclaude::types::MessageRequest;
/// use futures::StreamExt;
///
/// let request = MessageRequest::builder()
///     .model("claude-3-5-sonnet-20241022")
///     .max_tokens(1024)
///     .messages(vec![Message::user("Tell me a story...")])
///     .stream(true)
///     .build()?;
///
/// let byte_stream = converse_streaming(&bedrock, &request).await?;
///
/// // Convert byte stream to MessageStream for high-level event handling
/// let message_stream = MessageStream::new(byte_stream);
/// let mut text = String::new();
/// use futures::StreamExt;
/// while let Some(event) = message_stream.text_stream().next().await {
///     if let Ok(chunk) = event {
///         text.push_str(&chunk);
///     }
/// }
/// println!("Story: {}", text);
/// ```
pub async fn converse_streaming(
    bedrock: &BedrockRuntimeClient,
    request: &MessageRequest,
) -> Result<Box<dyn Stream<Item = Result<Bytes>> + Send + Unpin>> {
    let model_id = BedrockHttpProvider::normalize_model_id(&request.model);

    // Transform messages to Bedrock format
    let bedrock_messages = translate_messages(&request.messages)?;

    // Build Bedrock streaming request
    let mut bedrock_request = bedrock
        .converse_stream()
        .model_id(model_id)
        .set_messages(Some(bedrock_messages));

    // Add system prompt if present
    if let Some(system) = &request.system {
        let system_blocks = translate_system_prompt(system);
        bedrock_request = bedrock_request.set_system(Some(system_blocks));
    }

    // Set inference configuration
    let mut inference_config =
        InferenceConfiguration::builder().max_tokens(request.max_tokens as i32);

    if let Some(temp) = request.temperature {
        inference_config = inference_config.temperature(temp);
    }
    if let Some(top_p) = request.top_p {
        inference_config = inference_config.top_p(top_p);
    }
    if let Some(stop_seqs) = &request.stop_sequences {
        inference_config = inference_config.set_stop_sequences(Some(stop_seqs.clone()));
    }

    bedrock_request = bedrock_request.inference_config(inference_config.build());

    // Add tools if present
    if let Some(tools) = &request.tools {
        let tool_config = translate_tool_config(tools, request.tool_choice.as_ref())?;
        bedrock_request = bedrock_request.tool_config(tool_config);
    }

    // Handle top_k via additional_model_request_fields
    if let Some(top_k) = request.top_k {
        let additional_fields = serde_json::json!({ "top_k": top_k });
        let additional_fields_doc = json_value_to_document(&additional_fields)?;
        bedrock_request =
            bedrock_request.additional_model_request_fields(additional_fields_doc);
    }

    // Send request and get stream
    let output = bedrock_request
        .send()
        .await
        .map_err(|e| BedrockError::Service(format!("ConverseStream API error: {}", e)))?;

    // Transform stream to SSE format expected by turboclaude
    let stream = translate_stream(output);
    Ok(Box::new(stream))
}

/// Translate turboclaude messages to Bedrock's Converse API format.
///
/// # Conversion Details
///
/// Converts a sequence of turboclaude `MessageParam` (user/assistant messages) into
/// Bedrock's `Message` type. Each message is processed individually with its content blocks.
///
/// ## Role Translation
///
/// - `Role::User` → `ConversationRole::User`
/// - `Role::Assistant` → `ConversationRole::Assistant`
///
/// ## Content Block Translation
///
/// Each content block within a message is translated via `translate_content_block_param`:
/// - **Text**: Passed through as-is
/// - **Image**: Decoded from base64 and wrapped in Bedrock's `ImageBlock`
/// - **Document**: Decoded and wrapped in Bedrock's `DocumentBlock`
/// - **ToolResult**: Translated to Bedrock's `ToolResultBlock`
///
/// # Arguments
///
/// * `messages` - A slice of turboclaude messages to translate
///
/// # Returns
///
/// A vector of Bedrock `Message` types, or an error if any content block translation fails.
///
/// # Errors
///
/// Returns `BedrockError::Translation` if:
/// - Any content block fails to translate
/// - The Bedrock message builder fails to construct a valid message
///
/// # Example
///
/// ```ignore
/// let messages = vec![
///     Message::user("What time is it?"),
///     Message::assistant("I need to call a tool"),
/// ];
/// let bedrock_messages = translate_messages(&messages)?;
/// assert_eq!(bedrock_messages.len(), 2);
/// ```
fn translate_messages(messages: &[MessageParam]) -> Result<Vec<BedrockMessage>> {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                Role::User => ConversationRole::User,
                Role::Assistant => ConversationRole::Assistant,
            };

            let content = msg
                .content
                .iter()
                .map(translate_content_block_param)
                .collect::<Result<Vec<_>>>()?;

            BedrockMessage::builder()
                .role(role)
                .set_content(Some(content))
                .build()
                .map_err(|e| {
                    crate::error::Error::from(BedrockError::Translation(format!(
                        "Failed to build Bedrock message: {}",
                        e
                    )))
                })
        })
        .collect()
}

/// Translate a single turboclaude content block to Bedrock format.
///
/// # Content Block Types
///
/// ## Text Blocks
/// - **Conversion**: Direct pass-through from turboclaude to Bedrock
/// - **Format**: Plain UTF-8 string
/// - **Use Case**: Regular message text, tool inputs/outputs
///
/// ## Image Blocks
/// - **Conversion**: Base64-decoded bytes wrapped in Bedrock's `ImageBlock`
/// - **Supported Formats**: JPEG, PNG, GIF, WebP
/// - **Input**: Base64-encoded bytes with media type
/// - **Output**: Bedrock `ImageBlock` with decoded bytes and format
/// - **Error Handling**: Invalid base64 or unsupported formats return translation errors
///
/// ## Document Blocks
/// - **Conversion**: Format-specific handling based on source type
/// - **Base64 PDF**: Decoded bytes wrapped with PDF format metadata
/// - **Plain Text**: Converted to bytes with text/plain metadata
/// - **URL Sources**: **Not supported** - Bedrock API limitation
/// - **Note**: Documents are handled differently than images; they preserve media type info
///
/// ## Tool Result Blocks
/// - **Conversion**: Maps to Bedrock's `ToolResultBlock`
/// - **Fields**:
///   - `tool_use_id`: Pass-through (must match a tool use in the message)
///   - `content`: Tool output text
///   - `is_error`: Translated to `ToolResultStatus::Error` or `Success`
/// - **Use Case**: Returning tool execution results to the model
///
/// # Arguments
///
/// * `block` - A turboclaude content block to translate
///
/// # Returns
///
/// A Bedrock `ContentBlock`, or an error if translation fails.
///
/// # Errors
///
/// - **Text blocks**: Should not error (unless builder fails)
/// - **Image blocks**: Errors if base64 decoding fails or format is unsupported
/// - **Document blocks**: Errors if base64 decoding fails; URL sources always error
/// - **Tool results**: Errors if Bedrock builder fails
///
/// # Example
///
/// ```ignore
/// // Text block
/// let text_block = ContentBlockParam::Text { text: "Hello".to_string() };
/// let bedrock_text = translate_content_block_param(&text_block)?;
///
/// // Image block (must be pre-encoded as base64)
/// let image_block = ContentBlockParam::Image {
///     source: ImageSource {
///         media_type: "image/jpeg".to_string(),
///         data: "iVBORw0KGgoAAAANS...".to_string(), // base64
///     }
/// };
/// let bedrock_image = translate_content_block_param(&image_block)?;
///
/// // Tool result
/// let tool_result = ContentBlockParam::ToolResult {
///     tool_use_id: "tool_123".to_string(),
///     content: "Result: 42".to_string(),
///     is_error: Some(false),
/// };
/// let bedrock_result = translate_content_block_param(&tool_result)?;
/// ```
fn translate_content_block_param(block: &ContentBlockParam) -> Result<BedrockContentBlock> {
    match block {
        ContentBlockParam::Text { text } => Ok(BedrockContentBlock::Text(text.clone())),
        ContentBlockParam::Image { source } => {
            // Convert base64 image to Blob
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&source.data)
                .map_err(|e| {
                    BedrockError::Translation(format!("Invalid base64 image data: {}", e))
                })?;

            let format = match source.media_type.as_str() {
                "image/jpeg" => aws_sdk_bedrockruntime::types::ImageFormat::Jpeg,
                "image/png" => aws_sdk_bedrockruntime::types::ImageFormat::Png,
                "image/gif" => aws_sdk_bedrockruntime::types::ImageFormat::Gif,
                "image/webp" => aws_sdk_bedrockruntime::types::ImageFormat::Webp,
                _ => {
                    return Err(BedrockError::Translation(format!(
                        "Unsupported image format: {}",
                        source.media_type
                    ))
                    .into());
                }
            };

            let image_source = aws_sdk_bedrockruntime::types::ImageSource::Bytes(Blob::new(bytes));

            let image = aws_sdk_bedrockruntime::types::ImageBlock::builder()
                .format(format)
                .source(image_source)
                .build()
                .map_err(|e| {
                    BedrockError::Translation(format!("Failed to build image block: {}", e))
                })?;

            Ok(BedrockContentBlock::Image(image))
        }
        ContentBlockParam::Document { source, .. } => {
            // Convert document to Bedrock format based on source type
            use base64::Engine;

            let (bytes, format, name) = match source {
                crate::types::DocumentSource::Base64PDF { media_type, data } => {
                    let decoded = base64::engine::general_purpose::STANDARD
                        .decode(data)
                        .map_err(|e| {
                            BedrockError::Translation(format!(
                                "Invalid base64 document data: {}",
                                e
                            ))
                        })?;
                    (
                        decoded,
                        aws_sdk_bedrockruntime::types::DocumentFormat::Pdf,
                        media_type.clone(),
                    )
                }
                crate::types::DocumentSource::PlainText { text } => (
                    text.as_bytes().to_vec(),
                    aws_sdk_bedrockruntime::types::DocumentFormat::Txt,
                    "text/plain".to_string(),
                ),
                crate::types::DocumentSource::URL { url: _ } => {
                    // Bedrock's Converse API doesn't support URL sources
                    // We would need to fetch the document first
                    return Err(BedrockError::UnsupportedFeature(
                        "Document URL sources not supported in Bedrock Converse API",
                    )
                    .into());
                }
            };

            let doc_source = aws_sdk_bedrockruntime::types::DocumentSource::Bytes(Blob::new(bytes));

            let document = aws_sdk_bedrockruntime::types::DocumentBlock::builder()
                .format(format)
                .name(name)
                .source(doc_source)
                .build()
                .map_err(|e| {
                    BedrockError::Translation(format!("Failed to build document block: {}", e))
                })?;

            Ok(BedrockContentBlock::Document(document))
        }
        ContentBlockParam::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            let status = if is_error.unwrap_or(false) {
                aws_sdk_bedrockruntime::types::ToolResultStatus::Error
            } else {
                aws_sdk_bedrockruntime::types::ToolResultStatus::Success
            };

            let tool_result = aws_sdk_bedrockruntime::types::ToolResultBlock::builder()
                .tool_use_id(tool_use_id.clone())
                .content(aws_sdk_bedrockruntime::types::ToolResultContentBlock::Text(
                    content.clone(),
                ))
                .status(status)
                .build()
                .map_err(|e| {
                    BedrockError::Translation(format!("Failed to build tool result block: {}", e))
                })?;

            Ok(BedrockContentBlock::ToolResult(tool_result))
        }
    }
}

/// Translate turboclaude system prompt to Bedrock format.
///
/// # Overview
///
/// Converts a turboclaude `SystemPrompt` (either a single string or a sequence of blocks)
/// into Bedrock's `SystemContentBlock` format. Bedrock's system prompt is simpler than
/// turboclaude's full block structure.
///
/// # Conversion
///
/// - **String Prompt**: Wrapped in a single `SystemContentBlock::Text`
/// - **Block Prompts**: Each `SystemPromptBlock::Text` becomes a separate `SystemContentBlock::Text`
///   - Multiple blocks are preserved as separate content blocks (Bedrock supports this)
///   - Currently, turboclaude only supports text blocks in system prompts
///
/// # Arguments
///
/// * `system` - The turboclaude system prompt to translate
///
/// # Returns
///
/// A vector of Bedrock `SystemContentBlock` items (usually 1 for strings, N for block sequences).
///
/// # Note
///
/// This function **does not return a Result**. System prompt translation should not fail
/// because the formats are simple text-only and structurally compatible. If text is empty,
/// Bedrock will validate that on the API call.
///
/// # Example
///
/// ```ignore
/// // String system prompt
/// let system = SystemPrompt::String("You are a helpful assistant".to_string());
/// let bedrock_system = translate_system_prompt(&system);
/// assert_eq!(bedrock_system.len(), 1);
///
/// // Block system prompt
/// let system = SystemPrompt::Blocks(vec![
///     SystemPromptBlock::Text {
///         text: "You are helpful".to_string(),
///         cache_control: None,
///     },
/// ]);
/// let bedrock_system = translate_system_prompt(&system);
/// assert_eq!(bedrock_system.len(), 1);
/// ```
fn translate_system_prompt(system: &SystemPrompt) -> Vec<SystemContentBlock> {
    match system {
        SystemPrompt::String(s) => vec![SystemContentBlock::Text(s.clone())],
        SystemPrompt::Blocks(blocks) => blocks
            .iter()
            .map(|block| match block {
                SystemPromptBlock::Text { text, .. } => SystemContentBlock::Text(text.clone()),
            })
            .collect(),
    }
}

/// Translate turboclaude tools and tool choice to Bedrock format.
///
/// # Overview
///
/// Converts turboclaude's tool definitions and tool choice strategy into Bedrock's
/// `ToolConfiguration`. This involves:
/// 1. Building `ToolSpecification` for each tool
/// 2. Converting tool input schemas from `serde_json::Value` to AWS `Document`
/// 3. Translating tool choice strategy (auto/any/specific)
///
/// # Tool Specification Translation
///
/// Each tool is converted to a Bedrock `ToolSpecification`:
/// - **name**: Pass-through from turboclaude
/// - **description**: Pass-through from turboclaude
/// - **input_schema**: Converted from `serde_json::Value` to AWS `Document` via `json_value_to_document`
///
/// The input schema defines the JSON Schema for the tool's input parameters.
///
/// # Tool Choice Translation
///
/// - **`ToolChoice::Auto`**: Model decides whether to use tools and which one
/// - **`ToolChoice::Any`**: Model must use a tool (any tool)
/// - **`ToolChoice::Tool { name }`**: Model must use the specified tool by name
///
/// # Arguments
///
/// * `tools` - A slice of turboclaude tool definitions
/// * `tool_choice` - Optional tool choice strategy (None = no preference)
///
/// # Returns
///
/// A Bedrock `ToolConfiguration`, or an error if:
/// - JSON schema conversion fails (via `json_value_to_document`)
/// - Bedrock tool builder fails
/// - Bedrock tool choice builder fails
///
/// # Errors
///
/// - `BedrockError::Translation` for schema or builder failures
///
/// # Example
///
/// ```ignore
/// let tools = vec![
///     Tool {
///         name: "get_weather".to_string(),
///         description: "Get weather for a location".to_string(),
///         input_schema: json!({
///             "type": "object",
///             "properties": {
///                 "location": { "type": "string" }
///             },
///             "required": ["location"]
///         }),
///         cache_control: None,
///     }
/// ];
/// let tool_choice = Some(&ToolChoice::Auto);
///
/// let config = translate_tool_config(&tools, tool_choice)?;
/// // config now ready for Bedrock API
/// ```
fn translate_tool_config(
    tools: &[Tool],
    tool_choice: Option<&ToolChoice>,
) -> Result<ToolConfiguration> {
    let bedrock_tools = tools
        .iter()
        .map(|tool| {
            // Convert input_schema to ToolInputSchema (AWS Document type)
            // Convert serde_json::Value to aws_smithy_types::Document
            let input_schema_doc = json_value_to_document(&tool.input_schema)?;
            let input_schema = ToolInputSchema::Json(input_schema_doc);

            let spec = ToolSpecification::builder()
                .name(&tool.name)
                .description(&tool.description)
                .input_schema(input_schema)
                .build()
                .map_err(|e| -> crate::error::Error {
                    BedrockError::Translation(format!("Failed to build tool spec: {}", e)).into()
                })?;

            Ok(BedrockTool::ToolSpec(spec))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut config = ToolConfiguration::builder().set_tools(Some(bedrock_tools));

    // Translate tool choice
    if let Some(choice) = tool_choice {
        let bedrock_choice = match choice {
            ToolChoice::Auto => aws_sdk_bedrockruntime::types::ToolChoice::Auto(
                aws_sdk_bedrockruntime::types::AutoToolChoice::builder().build(),
            ),
            ToolChoice::Any => aws_sdk_bedrockruntime::types::ToolChoice::Any(
                aws_sdk_bedrockruntime::types::AnyToolChoice::builder().build(),
            ),
            ToolChoice::Tool { name } => aws_sdk_bedrockruntime::types::ToolChoice::Tool(
                aws_sdk_bedrockruntime::types::SpecificToolChoice::builder()
                    .name(name)
                    .build()
                    .map_err(|e| {
                        BedrockError::Translation(format!(
                            "Failed to build specific tool choice: {}",
                            e
                        ))
                    })?,
            ),
        };

        config = config.tool_choice(bedrock_choice);
    }

    config.build().map_err(|e| {
        BedrockError::Translation(format!("Failed to build tool config: {}", e)).into()
    })
}

/// Translate Bedrock response to turboclaude Message
fn translate_response(
    response: aws_sdk_bedrockruntime::operation::converse::ConverseOutput,
    model_id: &str,
) -> Result<Message> {
    let output = response.output().ok_or_else(|| -> crate::error::Error {
        BedrockError::Translation("Missing output in Bedrock response".to_string()).into()
    })?;

    let message = match output {
        aws_sdk_bedrockruntime::types::ConverseOutput::Message(msg) => msg,
        _ => {
            return Err(BedrockError::Translation("Output is not a message".to_string()).into());
        }
    };

    // Translate content blocks
    let content = message
        .content()
        .iter()
        .filter_map(translate_bedrock_content_block)
        .collect();

    // Translate stop reason (AWS always provides this, we wrap in Some)
    let stop_reason = match response.stop_reason().as_str() {
        "end_turn" => Some(StopReason::EndTurn),
        "max_tokens" => Some(StopReason::MaxTokens),
        "stop_sequence" => Some(StopReason::StopSequence),
        "tool_use" => Some(StopReason::ToolUse),
        "content_filtered" => Some(StopReason::EndTurn),
        _ => None, // Unknown stop reason, gracefully handle
    };

    // Extract usage
    let usage = response
        .usage()
        .map(|u| Usage {
            input_tokens: u.input_tokens() as u32,
            output_tokens: u.output_tokens() as u32,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        })
        .unwrap_or(Usage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        });

    Ok(Message {
        id: uuid::Uuid::new_v4().to_string(), // Bedrock doesn't provide message IDs
        message_type: "message".to_string(),
        role: Role::Assistant,
        content,
        model: model_id.to_string(),
        stop_reason,
        stop_sequence: None, // Bedrock doesn't return the actual stop sequence
        usage,
    })
}

/// Translate Bedrock content block to turboclaude format
fn translate_bedrock_content_block(block: &BedrockContentBlock) -> Option<ContentBlock> {
    match block {
        BedrockContentBlock::Text(text) => Some(ContentBlock::Text {
            text: text.clone(),
            citations: None,
        }),
        BedrockContentBlock::ToolUse(tool_use) => {
            // Convert AWS Document to serde_json::Value
            let input = document_to_json_value(tool_use.input());

            Some(ContentBlock::ToolUse {
                id: tool_use.tool_use_id().to_string(),
                name: tool_use.name().to_string(),
                input,
            })
        }
        _ => None, // Other block types not supported in responses
    }
}

/// Translate Bedrock stream to SSE format
fn translate_stream(
    output: aws_sdk_bedrockruntime::operation::converse_stream::ConverseStreamOutput,
) -> Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>> {
    use futures::stream;

    let stream = stream::unfold(output.stream, |mut receiver| async move {
        match receiver.recv().await {
            Ok(Some(event)) => {
                // Convert Bedrock stream events to SSE format
                use aws_sdk_bedrockruntime::types::ConverseStreamOutput as BedrockStreamEvent;

                let sse_data = match event {
                    BedrockStreamEvent::ContentBlockDelta(delta) => {
                        if let Some(delta_inner) = delta.delta() {
                            match delta_inner {
                                aws_sdk_bedrockruntime::types::ContentBlockDelta::Text(text) => {
                                    format!(
                                        "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":{},\"delta\":{{\"type\":\"text_delta\",\"text\":\"{}\"}}}}\n\n",
                                        delta.content_block_index(),
                                        text.replace('\\', "\\\\").replace('"', "\\\"")
                                    )
                                }
                                _ => String::new(), // Skip unknown delta types
                            }
                        } else {
                            String::new()
                        }
                    }
                    BedrockStreamEvent::MessageStart(_) => {
                        "event: message_start\ndata: {\"type\":\"message_start\"}\n\n".to_string()
                    }
                    BedrockStreamEvent::MessageStop(_) => {
                        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string()
                    }
                    BedrockStreamEvent::Metadata(metadata) => {
                        // Include usage information
                        if let Some(usage) = metadata.usage() {
                            format!(
                                "event: message_delta\ndata: {{\"type\":\"message_delta\",\"usage\":{{\"input_tokens\":{},\"output_tokens\":{}}}}}\n\n",
                                usage.input_tokens(),
                                usage.output_tokens()
                            )
                        } else {
                            String::new()
                        }
                    }
                    _ => String::new(), // Skip other event types
                };

                if sse_data.is_empty() {
                    // Skip empty events, continue to next
                    Some((Ok(Bytes::new()), receiver))
                } else {
                    Some((Ok(Bytes::from(sse_data)), receiver))
                }
            }
            Ok(None) => None, // Stream ended
            Err(e) => {
                let err: crate::error::Error =
                    BedrockError::Service(format!("Stream error: {}", e)).into();
                Some((Err(err), receiver))
            }
        }
    });

    Box::pin(stream)
}

/// Convert a standard JSON value to AWS Bedrock's Document type.
///
/// # Why This Conversion Exists
///
/// Bedrock uses AWS's proprietary `Document` type for representing JSON data, while
/// turboclaude (and most Rust JSON handling) uses the standard `serde_json::Value`.
/// This function bridges the gap by recursively converting between the two.
///
/// # Type Mapping
///
/// | serde_json::Value | aws_smithy_types::Document |
/// |---|---|
/// | Null | Document::Null |
/// | Bool(b) | Document::Bool(b) |
/// | Number(n) | Document::Number(PosInt/NegInt/Float) depending on value |
/// | String(s) | Document::String(s) |
/// | Array(arr) | Document::Array(recursive conversion) |
/// | Object(map) | Document::Object(recursive conversion) |
///
/// # Number Conversion Details
///
/// JSON numbers are converted based on their actual value:
/// - **Positive integers**: `Number::PosInt`
/// - **Negative integers**: `Number::NegInt`
/// - **Floating point**: `Number::Float`
/// - **Out of range**: Defaults to `Number::Float(0.0)` (edge case)
///
/// # Recursion
///
/// Arrays and objects are recursively converted, preserving nested structure.
///
/// # Arguments
///
/// * `value` - The JSON value to convert
///
/// # Returns
///
/// An AWS `Document` representing the same data, or an error if recursive conversion fails.
///
/// # Example: JSON Schema Conversion
///
/// ```ignore
/// use serde_json::json;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" },
///         "age": { "type": "integer" },
///         "tags": {
///             "type": "array",
///             "items": { "type": "string" }
///         }
///     },
///     "required": ["name"]
/// });
///
/// let doc = json_value_to_document(&schema)?;
/// // doc is now a Document representation suitable for Bedrock's tool schema
/// ```
fn json_value_to_document(value: &JsonValue) -> Result<aws_smithy_types::Document> {
    use aws_smithy_types::{Document, Number};
    use std::collections::HashMap;

    match value {
        JsonValue::Null => Ok(Document::Null),
        JsonValue::Bool(b) => Ok(Document::Bool(*b)),
        JsonValue::Number(n) => {
            // AWS Number type has different variants - use PosInt/NegInt/Float
            if let Some(i) = n.as_i64() {
                if i >= 0 {
                    Ok(Document::Number(Number::PosInt(i as u64)))
                } else {
                    Ok(Document::Number(Number::NegInt(i)))
                }
            } else if let Some(f) = n.as_f64() {
                Ok(Document::Number(Number::Float(f)))
            } else {
                Ok(Document::Number(Number::Float(0.0)))
            }
        }
        JsonValue::String(s) => Ok(Document::String(s.clone())),
        JsonValue::Array(arr) => {
            let docs: Result<Vec<_>> = arr.iter().map(json_value_to_document).collect();
            Ok(Document::Array(docs?))
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_value_to_document(v)?);
            }
            Ok(Document::Object(map))
        }
    }
}

/// Convert AWS Bedrock's Document type to a standard JSON value.
///
/// # Inverse of json_value_to_document
///
/// This is the reverse operation of `json_value_to_document`, converting AWS `Document`
/// types back to standard `serde_json::Value` for turboclaude's type system.
///
/// # Type Mapping
///
/// | aws_smithy_types::Document | serde_json::Value |
/// |---|---|
/// | Document::Null | Null |
/// | Document::Bool(b) | Bool(b) |
/// | Document::Number(n) | Number (PosInt/NegInt/Float variants) |
/// | Document::String(s) | String(s) |
/// | Document::Array(arr) | Array(recursive conversion) |
/// | Document::Object(map) | Object(recursive conversion) |
///
/// # Number Conversion
///
/// AWS `Number` variants are converted back to `serde_json::Number`:
/// - **PosInt(u64)**: Converted directly
/// - **NegInt(i64)**: Converted directly
/// - **Float(f64)**: Converted via `from_f64`, or `Null` if NaN/infinite
///
/// # Recursion
///
/// Arrays and objects are recursively converted, preserving nested structure.
///
/// # Use Cases
///
/// This function is primarily used when:
/// 1. Converting tool use inputs from Bedrock responses (returned as `Document`)
/// 2. Working with Bedrock API responses that include structured JSON data
/// 3. Need to integrate Bedrock responses with turboclaude's type system
///
/// # Arguments
///
/// * `doc` - The AWS Document to convert
///
/// # Returns
///
/// A `serde_json::Value` representing the same data.
/// This function **cannot fail** - all Document values are convertible to JSON.
///
/// # Example: Tool Use Response
///
/// ```ignore
/// // Bedrock returns tool use with Document input
/// let tool_use_input: aws_smithy_types::Document = /* from Bedrock */;
///
/// // Convert to JSON for turboclaude's ToolUse content block
/// let input_json = document_to_json_value(&tool_use_input);
///
/// let content_block = ContentBlock::ToolUse {
///     id: "tool_123".to_string(),
///     name: "get_weather".to_string(),
///     input: input_json,  // Now matches turboclaude's expected type
/// };
/// ```
fn document_to_json_value(doc: &aws_smithy_types::Document) -> JsonValue {
    use aws_smithy_types::{Document, Number};

    match doc {
        Document::Null => JsonValue::Null,
        Document::Bool(b) => JsonValue::Bool(*b),
        Document::Number(n) => match n {
            Number::PosInt(i) => JsonValue::Number(serde_json::Number::from(*i)),
            Number::NegInt(i) => JsonValue::Number(serde_json::Number::from(*i)),
            Number::Float(f) => serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
        },
        Document::String(s) => JsonValue::String(s.clone()),
        Document::Array(arr) => JsonValue::Array(arr.iter().map(document_to_json_value).collect()),
        Document::Object(obj) => {
            let map: serde_json::Map<String, JsonValue> = obj
                .iter()
                .map(|(k, v)| (k.clone(), document_to_json_value(v)))
                .collect();
            JsonValue::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_text_content() {
        let param = ContentBlockParam::Text {
            text: "Hello, world!".to_string(),
        };

        let result = translate_content_block_param(&param).unwrap();
        match result {
            BedrockContentBlock::Text(text) => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text block"),
        }
    }

    #[test]
    fn test_translate_tool_result() {
        let param = ContentBlockParam::ToolResult {
            tool_use_id: "test-id".to_string(),
            content: "result".to_string(),
            is_error: Some(false),
        };

        let result = translate_content_block_param(&param).unwrap();
        match result {
            BedrockContentBlock::ToolResult(tr) => {
                assert_eq!(tr.tool_use_id(), "test-id");
                assert_eq!(
                    tr.status(),
                    Some(&aws_sdk_bedrockruntime::types::ToolResultStatus::Success)
                );
            }
            _ => panic!("Expected tool result block"),
        }
    }

    #[test]
    fn test_translate_system_prompt_string() {
        let system = SystemPrompt::String("You are a helpful assistant.".to_string());
        let result = translate_system_prompt(&system);

        assert_eq!(result.len(), 1);
        match &result[0] {
            SystemContentBlock::Text(text) => {
                assert_eq!(text, "You are a helpful assistant.");
            }
            _ => panic!("Expected text block"),
        }
    }
}
