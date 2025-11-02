//! Query latency benchmarks
//!
//! Measures response time for various query patterns using MockCliTransport
//! to avoid dependencies on external services.
//!
//! Run with: cargo bench --bench query_latency

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;
use turboclaude_protocol::{
    ContentBlock, Message, MessageRole, ProtocolMessage, QueryRequest, QueryResponse, Usage,
    types::StopReason,
};
use turboclaudeagent::testing::MockCliTransport;

fn create_response(size: usize) -> QueryResponse {
    let text = "x".repeat(size);
    QueryResponse {
        message: Message {
            id: "msg_123".to_string(),
            message_type: "message".to_string(),
            role: MessageRole::Assistant,
            content: vec![ContentBlock::Text { text }],
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: StopReason::EndTurn,
            stop_sequence: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            usage: Usage {
                input_tokens: 100,
                output_tokens: 200,
            },
            cache_usage: Default::default(),
        },
        is_complete: true,
    }
}

#[tokio::main]
async fn bench_simple_query(c: &mut Criterion) {
    c.bench_function("simple_query_send_recv", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();
                mock.enqueue_response(ProtocolMessage::Response(create_response(100)))
                    .await;

                let query = QueryRequest {
                    query: black_box("What is 2+2?".to_string()),
                    model: "claude-3-5-sonnet".to_string(),
                    max_tokens: 1024,
                    tools: vec![],
                    messages: vec![],
                    system_prompt: None,
                };

                let msg = ProtocolMessage::Query(query);
                let json = msg.to_json().unwrap();
                let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

                mock.send_message(json_value).await.unwrap();
                mock.recv_message().await.unwrap();
            })
    });
}

#[tokio::main]
async fn bench_large_response(c: &mut Criterion) {
    c.bench_function("large_response_processing", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();
                // Simulate a large response (4KB)
                mock.enqueue_response(ProtocolMessage::Response(create_response(4096)))
                    .await;

                let query = QueryRequest {
                    query: "Explain the concept in detail".to_string(),
                    model: "claude-3-5-sonnet".to_string(),
                    max_tokens: 2048,
                    tools: vec![],
                    messages: vec![],
                    system_prompt: None,
                };

                let msg = ProtocolMessage::Query(query);
                let json = msg.to_json().unwrap();
                let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

                mock.send_message(json_value).await.unwrap();
                let response = mock.recv_message().await.unwrap();
                black_box(response);
            })
    });
}

#[tokio::main]
async fn bench_message_serialization(c: &mut Criterion) {
    let query = QueryRequest {
        query: "Test query".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        max_tokens: 1024,
        tools: vec![],
        messages: vec![],
        system_prompt: None,
    };

    let msg = ProtocolMessage::Query(query);

    c.bench_function("message_serialization", |b| {
        b.iter(|| {
            black_box(&msg).to_json().unwrap();
        })
    });

    let json = msg.to_json().unwrap();

    c.bench_function("message_deserialization", |b| {
        b.iter(|| {
            ProtocolMessage::from_json(black_box(&json)).unwrap();
        })
    });
}

#[tokio::main]
async fn bench_message_count_tracking(c: &mut Criterion) {
    c.bench_function("message_count_tracking_100", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();

                for i in 0..100 {
                    let test_json = serde_json::json!({
                        "type": "test",
                        "index": i
                    });
                    mock.send_message(test_json).await.ok();
                }

                black_box(mock.message_count().await);
            })
    });
}

#[tokio::main]
async fn bench_response_queue_operations(c: &mut Criterion) {
    let response = create_response(1000);

    c.bench_function("enqueue_response", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();
                for _ in 0..10 {
                    mock.enqueue_response(ProtocolMessage::Response(response.clone()))
                        .await;
                }
            })
    });

    c.bench_function("dequeue_response_100", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();

                for _ in 0..100 {
                    mock.enqueue_response(ProtocolMessage::Response(response.clone()))
                        .await;
                }

                for _ in 0..100 {
                    mock.recv_message().await.ok();
                }
            })
    });
}

#[tokio::main]
async fn bench_concurrent_sends(c: &mut Criterion) {
    c.bench_function("concurrent_sends_10", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();

                let handles: Vec<_> = (0..10)
                    .map(|i| {
                        let mock_clone = mock.clone();
                        tokio::spawn(async move {
                            let test_json = serde_json::json!({
                                "type": "concurrent_test",
                                "id": i
                            });
                            mock_clone.send_message(test_json).await
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.await.ok();
                }

                black_box(mock.message_count().await);
            })
    });

    c.bench_function("concurrent_sends_100", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let mock = MockCliTransport::new();

                let handles: Vec<_> = (0..100)
                    .map(|i| {
                        let mock_clone = mock.clone();
                        tokio::spawn(async move {
                            let test_json = serde_json::json!({
                                "type": "concurrent_test",
                                "id": i
                            });
                            mock_clone.send_message(test_json).await
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.await.ok();
                }

                black_box(mock.message_count().await);
            })
    });
}

criterion_group!(
    benches,
    bench_simple_query,
    bench_large_response,
    bench_message_serialization,
    bench_message_count_tracking,
    bench_response_queue_operations,
    bench_concurrent_sends
);
criterion_main!(benches);
