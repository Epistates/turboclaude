//! Performance benchmarks for message operations
//!
//! Run with: cargo bench --bench message_performance

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use turboclaude::types::{ContentBlock, Message, MessageRequest, Role};

fn bench_message_request_creation(c: &mut Criterion) {
    c.bench_function("create_message_request", |b| {
        b.iter(|| MessageRequest {
            model: black_box("claude-3-5-sonnet-20241022".to_string()),
            messages: black_box(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "Hello, world!".to_string(),
                }],
            }]),
            max_tokens: black_box(1024),
            system: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            metadata: None,
            tools: None,
            tool_choice: None,
        });
    });
}

fn bench_message_serialization(c: &mut Criterion) {
    let request = MessageRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        messages: vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: "Hello, world!".to_string(),
            }],
        }],
        max_tokens: 1024,
        system: None,
        temperature: None,
        top_p: None,
        top_k: None,
        stop_sequences: None,
        stream: None,
        metadata: None,
        tools: None,
        tool_choice: None,
    };

    c.bench_function("serialize_message_request", |b| {
        b.iter(|| serde_json::to_string(&black_box(&request)).unwrap());
    });
}

fn bench_message_deserialization(c: &mut Criterion) {
    let json = r#"{
        "id": "msg_01XFDUDYJgAACzvnptvVoYEL",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "Hello! I'm Claude."
            }
        ],
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 12,
            "output_tokens": 25
        }
    }"#;

    c.bench_function("deserialize_message_response", |b| {
        b.iter(|| {
            serde_json::from_str::<turboclaude::types::MessageResponse>(black_box(json)).unwrap()
        });
    });
}

fn bench_varying_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_sizes");

    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        let text = "a".repeat(*size);
        let message = Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text }],
        };

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| serde_json::to_string(&black_box(&message)).unwrap());
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_message_request_creation,
    bench_message_serialization,
    bench_message_deserialization,
    bench_varying_message_sizes
);
criterion_main!(benches);
