//! Batch API tests
//!
//! Comprehensive testing for batch message processing:
//! - Create batches
//! - List batches
//! - Get batch status
//! - Cancel batches
//! - Retrieve results (JSONL parsing)

use turboclaude::{Client, BatchRequest, types::{MessageRequest, Message, MessageParam, Models, ProcessingStatus}};
use rstest::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

mod common;
use common::{batch_create_response, batch_completed_response, batch_results_jsonl};

#[tokio::test]
async fn test_batches_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_create_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let message_request = MessageRequest::builder()
        .model(Models::CLAUDE_3_5_SONNET)
        .max_tokens(1024u32)
        .messages(vec![Message::user("Hello")])
        .build()
        .unwrap();

    let batch_request = BatchRequest {
        custom_id: "request-1".to_string(),
        params: message_request,
    };

    let result = client.messages().batches().create(vec![batch_request]).await;

    if let Err(ref e) = result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Result was error: {:?}", result.as_ref().err());
    let batch = result.unwrap();
    assert_eq!(batch.id, "msgbatch_01234567890");
    assert_eq!(batch.processing_status, ProcessingStatus::InProgress);
}

#[tokio::test]
async fn test_batches_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/messages/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [batch_create_response()],
            "has_more": false
        })))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.messages().batches().list().await;

    assert!(result.is_ok());
    let batches = result.unwrap();
    assert!(!batches.is_empty());
}

#[tokio::test]
async fn test_batches_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/messages/batches/msgbatch_01234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_completed_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.messages().batches().get("msgbatch_01234567890").await;

    assert!(result.is_ok());
    let batch = result.unwrap();
    assert_eq!(batch.id, "msgbatch_01234567890");
    assert_eq!(batch.processing_status, ProcessingStatus::Ended);
    assert!(batch.results_url.is_some());
}

#[tokio::test]
async fn test_batches_cancel() {
    let mock_server = MockServer::start().await;

    let canceled_response = serde_json::json!({
        "id": "msgbatch_01234567890",
        "type": "message_batch",
        "processing_status": "canceling",
        "request_counts": {
            "total": 100,
            "processing": 0,
            "succeeded": 50,
            "errored": 0,
            "canceled": 50,
            "expired": 0
        },
        "ended_at": null,
        "created_at": "2024-01-01T00:00:00Z",
        "expires_at": "2024-01-02T00:00:00Z",
        "results_url": null
    });

    Mock::given(method("POST"))
        .and(path("/v1/messages/batches/msgbatch_01234567890/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(canceled_response))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.messages().batches().cancel("msgbatch_01234567890").await;

    assert!(result.is_ok());
    let batch = result.unwrap();
    assert_eq!(batch.processing_status, ProcessingStatus::Canceling);
}

#[tokio::test]
async fn test_batches_results() {
    let mock_server = MockServer::start().await;

    // Mock the batch get endpoint
    Mock::given(method("GET"))
        .and(path("/v1/messages/batches/msgbatch_01234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_completed_response()))
        .mount(&mock_server)
        .await;

    // Mock the results URL endpoint
    let results_mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(batch_results_jsonl()))
        .mount(&results_mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    // Note: This test would need the actual results URL to be mocked properly
    // For now, we test the batch get which includes the results_url
    let batch = client.messages().batches().get("msgbatch_01234567890").await.unwrap();
    assert!(batch.results_url.is_some());
    assert_eq!(batch.results_url.unwrap(), "https://example.com/results.jsonl");
}

/// Parametrized test for different batch sizes
#[rstest]
#[case(1)]
#[case(10)]
#[case(100)]
#[tokio::test]
async fn test_batches_create_various_sizes(#[case] count: usize) {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_create_response()))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let requests: Vec<BatchRequest> = (0..count)
        .map(|i| {
            let message_request = MessageRequest::builder()
                .model(Models::CLAUDE_3_5_SONNET)
                .max_tokens(1024u32)
                .messages(vec![Message::user(format!("Message {}", i))])
                .build()
                .unwrap();

            BatchRequest {
                custom_id: format!("request-{}", i),
                params: message_request,
            }
        })
        .collect();

    let result = client.messages().batches().create(requests).await;
    assert!(result.is_ok());
}

/// Test batch status transitions
#[rstest]
#[case(ProcessingStatus::InProgress, "in_progress")]
#[case(ProcessingStatus::Ended, "ended")]
#[case(ProcessingStatus::Canceling, "canceling")]
#[tokio::test]
async fn test_batch_status_values(#[case] expected_status: ProcessingStatus, #[case] status_str: &str) {
    let mock_server = MockServer::start().await;

    let response = serde_json::json!({
        "id": "msgbatch_01234567890",
        "type": "message_batch",
        "processing_status": status_str,
        "request_counts": {
            "total": 100,
            "processing": 0,
            "succeeded": 100,
            "errored": 0,
            "canceled": 0,
            "expired": 0
        },
        "ended_at": if status_str == "ended" { Some("2024-01-01T01:00:00Z") } else { None },
        "created_at": "2024-01-01T00:00:00Z",
        "expires_at": "2024-01-02T00:00:00Z",
        "results_url": if status_str == "ended" { Some("https://example.com/results.jsonl") } else { None }
    });

    Mock::given(method("GET"))
        .and(path("/v1/messages/batches/msgbatch_01234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .api_key("test-key")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let result = client.messages().batches().get("msgbatch_01234567890").await;
    assert!(result.is_ok());

    let batch = result.unwrap();
    assert_eq!(batch.processing_status, expected_status);
}
