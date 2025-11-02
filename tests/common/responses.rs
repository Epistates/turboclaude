//! Common test response data
//!
//! Provides realistic API response fixtures matching Python SDK's test data

use serde_json::json;

/// Standard message response
pub fn message_response() -> serde_json::Value {
    json!({
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": "Hello! How can I help you today?"
        }],
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 12,
            "output_tokens": 25
        }
    })
}

/// Message response with tool use
pub fn message_with_tool_use() -> serde_json::Value {
    json!({
        "id": "msg_01234567890",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I'll help you with that calculation."
            },
            {
                "type": "tool_use",
                "id": "toolu_01A09q90qw90lq917835lq9",
                "name": "calculator",
                "input": {
                    "operation": "multiply",
                    "a": 42,
                    "b": 17
                }
            }
        ],
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "tool_use",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 20,
            "output_tokens": 50
        }
    })
}

/// Token count response
pub fn token_count_response() -> serde_json::Value {
    json!({
        "input_tokens": 15
    })
}

/// Batch create response
#[allow(dead_code)]
pub fn batch_create_response() -> serde_json::Value {
    json!({
        "id": "msgbatch_01234567890",
        "type": "message_batch",
        "processing_status": "in_progress",
        "request_counts": {
            "total": 100,
            "processing": 100,
            "succeeded": 0,
            "errored": 0,
            "canceled": 0,
            "expired": 0
        },
        "ended_at": null,
        "created_at": "2024-01-01T00:00:00Z",
        "expires_at": "2024-01-02T00:00:00Z",
        "results_url": null
    })
}

/// Batch get response (completed)
#[allow(dead_code)]
pub fn batch_completed_response() -> serde_json::Value {
    json!({
        "id": "msgbatch_01234567890",
        "type": "message_batch",
        "processing_status": "ended",
        "request_counts": {
            "total": 100,
            "processing": 0,
            "succeeded": 95,
            "errored": 5,
            "canceled": 0,
            "expired": 0
        },
        "ended_at": "2024-01-01T01:00:00Z",
        "created_at": "2024-01-01T00:00:00Z",
        "expires_at": "2024-01-02T00:00:00Z",
        "results_url": "https://example.com/results.jsonl"
    })
}

/// Batch results JSONL
#[allow(dead_code)]
pub fn batch_results_jsonl() -> String {
    r#"{"custom_id":"request-1","result":{"type":"succeeded","message":{"id":"msg_01","type":"message","role":"assistant","content":[{"type":"text","text":"Result 1"}],"model":"claude-3-5-sonnet-20241022","stop_reason":"end_turn","usage":{"input_tokens":10,"output_tokens":5}}}}
{"custom_id":"request-2","result":{"type":"errored","error":{"type":"invalid_request_error","message":"Invalid request"}}}
"#.to_string()
}

/// Model list response
#[allow(dead_code)]
pub fn model_list_response() -> serde_json::Value {
    json!({
        "data": [
            {
                "type": "model",
                "id": "claude-3-5-sonnet-20241022",
                "display_name": "Claude 3.5 Sonnet",
                "created_at": "2024-10-22T00:00:00Z"
            },
            {
                "type": "model",
                "id": "claude-3-opus-20240229",
                "display_name": "Claude 3 Opus",
                "created_at": "2024-02-29T00:00:00Z"
            }
        ],
        "has_more": false,
        "first_id": "claude-3-5-sonnet-20241022",
        "last_id": "claude-3-opus-20240229"
    })
}

/// Model get response
#[allow(dead_code)]
pub fn model_get_response() -> serde_json::Value {
    json!({
        "type": "model",
        "id": "claude-3-5-sonnet-20241022",
        "display_name": "Claude 3.5 Sonnet",
        "created_at": "2024-10-22T00:00:00Z"
    })
}

/// Error response - invalid request
#[allow(dead_code)]
pub fn error_invalid_request() -> serde_json::Value {
    json!({
        "type": "error",
        "error": {
            "type": "invalid_request_error",
            "message": "Invalid request: missing required field 'model'"
        }
    })
}

/// Error response - authentication
#[allow(dead_code)]
pub fn error_authentication() -> serde_json::Value {
    json!({
        "type": "error",
        "error": {
            "type": "authentication_error",
            "message": "Invalid API key"
        }
    })
}

/// Error response - rate limit
#[allow(dead_code)]
pub fn error_rate_limit() -> serde_json::Value {
    json!({
        "type": "error",
        "error": {
            "type": "rate_limit_error",
            "message": "Rate limit exceeded"
        }
    })
}

/// SSE streaming events for a simple text response
#[allow(dead_code)]
pub fn sse_text_stream() -> Vec<String> {
    vec![
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_01\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-3-5-sonnet-20241022\",\"stop_reason\":null,\"usage\":{\"input_tokens\":12,\"output_tokens\":0}}}".to_string(),
        "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}".to_string(),
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}".to_string(),
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" there!\"}}".to_string(),
        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}".to_string(),
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":5}}".to_string(),
        "event: message_stop\ndata: {\"type\":\"message_stop\"}".to_string(),
    ]
}

/// SSE streaming events with tool use
#[allow(dead_code)]
pub fn sse_tool_use_stream() -> Vec<String> {
    vec![
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_02\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-3-5-sonnet-20241022\",\"stop_reason\":null,\"usage\":{\"input_tokens\":20,\"output_tokens\":0}}}".to_string(),
        "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01\",\"name\":\"calculator\",\"input\":{}}}".to_string(),
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"operation\\\":\"}}".to_string(),
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"add\\\",\\\"a\\\":42,\\\"b\\\":17}\"}}".to_string(),
        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}".to_string(),
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":30}}".to_string(),
        "event: message_stop\ndata: {\"type\":\"message_stop\"}".to_string(),
    ]
}
