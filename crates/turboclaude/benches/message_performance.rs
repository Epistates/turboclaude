//! Performance benchmarks for message operations
//!
//! Run with: cargo bench --bench message_performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use turboclaude::types::{ContentBlockParam, Message, MessageParam, MessageRequest, Role};

fn bench_message_request_creation(c: &mut Criterion) {
    c.bench_function("create_message_request", |b| {
        b.iter(|| {
            MessageRequest::builder()
                .model(black_box("claude-3-5-sonnet-20241022"))
                .messages(black_box(vec![MessageParam {
                    role: Role::User,
                    content: vec![ContentBlockParam::Text {
                        text: "Hello, world!".to_string(),
                    }],
                }]))
                .max_tokens(black_box(1024u32))
                .build()
                .unwrap()
        });
    });
}

fn bench_message_serialization(c: &mut Criterion) {
    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .messages(vec![MessageParam {
            role: Role::User,
            content: vec![ContentBlockParam::Text {
                text: "Hello, world!".to_string(),
            }],
        }])
        .max_tokens(1024u32)
        .build()
        .unwrap();

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
        b.iter(|| serde_json::from_str::<Message>(black_box(json)).unwrap());
    });
}

fn bench_varying_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_sizes");

    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        let text = "a".repeat(*size);
        let message = MessageParam {
            role: Role::User,
            content: vec![ContentBlockParam::Text { text }],
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
