/// Real-world Streaming API tests
///
/// Run with: cargo test --ignored real_world_streaming

use turboclaude::{Client, Message, MessageRequest};
use turboclaude::streaming::StreamEvent;
use futures::StreamExt;
use crate::real_world::common::{TestConfig, TestMetrics};

#[tokio::test]
#[ignore]
async fn real_world_streaming_basic() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Basic streaming");

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(100u32)
        .messages(vec![Message::user("Count from 1 to 5")])
        .stream(true)
        .build()?;

    let mut stream = client.messages().stream(request).await?;
    let mut text_accumulated = String::new();
    let mut ttfb = None;

    while let Some(event) = stream.next().await {
        metrics.event_count += 1;

        if ttfb.is_none() {
            ttfb = Some(metrics.start_time.unwrap().elapsed());
        }

        match event? {
            StreamEvent::ContentBlockDelta(delta) => {
                if let Some(text) = delta.delta.text {
                    print!("{}", text);
                    text_accumulated.push_str(&text);
                }
            }
            StreamEvent::MessageStop => {
                println!("\n✅ Stream complete");
                break;
            }
            _ => {}
        }
    }

    metrics.finish();

    println!("\n✅ TTFB: {:?}", ttfb.unwrap());
    println!("✅ Total events: {}", metrics.event_count);
    println!("✅ Total text: {} chars", text_accumulated.len());
    println!("✅ Text: {}", text_accumulated);

    assert!(!text_accumulated.is_empty(), "Expected text in stream");
    assert!(metrics.event_count > 3, "Expected multiple stream events");
    assert!(ttfb.unwrap() < std::time::Duration::from_secs(2), "TTFB should be < 2s");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_streaming_long_response() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Long streaming response");

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(500u32)
        .messages(vec![Message::user("Tell me a very short story about a robot")])
        .stream(true)
        .build()?;

    let mut stream = client.messages().stream(request).await?;
    let mut text_accumulated = String::new();
    let mut chunk_sizes = Vec::new();

    while let Some(event) = stream.next().await {
        metrics.event_count += 1;

        match event? {
            StreamEvent::ContentBlockDelta(delta) => {
                if let Some(text) = delta.delta.text {
                    chunk_sizes.push(text.len());
                    text_accumulated.push_str(&text);
                }
            }
            StreamEvent::MessageStop => break,
            _ => {}
        }
    }

    metrics.finish();

    println!("\n✅ Total chunks: {}", chunk_sizes.len());
    println!("✅ Total characters: {}", text_accumulated.len());
    println!("✅ Average chunk size: {:.1}",
        chunk_sizes.iter().sum::<usize>() as f64 / chunk_sizes.len() as f64
    );

    assert!(text_accumulated.len() > 100, "Expected substantial response");
    assert!(chunk_sizes.len() > 5, "Expected multiple chunks");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_streaming_all_events() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: All stream event types");

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(50u32)
        .messages(vec![Message::user("Hi")])
        .stream(true)
        .build()?;

    let mut stream = client.messages().stream(request).await?;
    let mut event_types = std::collections::HashSet::new();

    while let Some(event) = stream.next().await {
        let event = event?;
        let event_type = match &event {
            StreamEvent::MessageStart(_) => "MessageStart",
            StreamEvent::ContentBlockStart(_) => "ContentBlockStart",
            StreamEvent::ContentBlockDelta(_) => "ContentBlockDelta",
            StreamEvent::ContentBlockStop(_) => "ContentBlockStop",
            StreamEvent::MessageDelta(_) => "MessageDelta",
            StreamEvent::MessageStop => "MessageStop",
            StreamEvent::Ping => "Ping",
            _ => "Unknown",
        };

        event_types.insert(event_type);
        println!("✅ Received: {}", event_type);

        if matches!(event, StreamEvent::MessageStop) {
            break;
        }
    }

    metrics.finish();

    println!("\n✅ Event types received: {:?}", event_types);

    // Verify we got essential events
    assert!(event_types.contains("MessageStart"), "Missing MessageStart");
    assert!(event_types.contains("ContentBlockStart"), "Missing ContentBlockStart");
    assert!(event_types.contains("ContentBlockDelta"), "Missing ContentBlockDelta");
    assert!(event_types.contains("MessageStop"), "Missing MessageStop");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_streaming_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);
    let mut metrics = TestMetrics::new();

    println!("\n🧪 Testing: Stream metadata extraction");

    let request = MessageRequest::builder()
        .model("claude-3-5-sonnet-20241022")
        .max_tokens(50u32)
        .messages(vec![Message::user("Say hello")])
        .stream(true)
        .build()?;

    let mut stream = client.messages().stream(request).await?;
    let mut message_id = None;
    let mut usage_info = None;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::MessageStart(start) => {
                message_id = Some(start.message.id.clone());
                println!("✅ Message ID: {}", start.message.id);
            }
            StreamEvent::MessageDelta(delta) => {
                println!("✅ Usage: {:?}", delta.usage);
                usage_info = Some(delta.usage);
            }
            StreamEvent::MessageStop => break,
            _ => {}
        }
    }

    metrics.finish();

    assert!(message_id.is_some(), "Should have message ID");
    assert!(usage_info.is_some(), "Should have usage info");

    println!("\n✅ Got usage delta");

    metrics.print_summary();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn real_world_streaming_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = TestConfig::from_env()?;
    let client = Client::new(&config.api_key);

    println!("\n🧪 Testing: Streaming performance metrics");

    let request = MessageRequest::builder()
        .model("claude-3-5-haiku-20241022") // Fastest model
        .max_tokens(200u32)
        .messages(vec![Message::user("Write a haiku about coding")])
        .stream(true)
        .build()?;

    let start = std::time::Instant::now();
    let mut stream = client.messages().stream(request).await?;

    let mut ttfb = None;
    let mut chunk_times = Vec::new();
    let mut last_chunk = start;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockDelta(delta) => {
                if delta.delta.text.is_some() {
                    if ttfb.is_none() {
                        ttfb = Some(start.elapsed());
                    }

                    let now = std::time::Instant::now();
                    chunk_times.push(now.duration_since(last_chunk));
                    last_chunk = now;
                }
            }
            StreamEvent::MessageStop => break,
            _ => {}
        }
    }

    let total_time = start.elapsed();

    println!("\n✅ TTFB: {:?}", ttfb.unwrap());
    println!("✅ Total time: {:?}", total_time);
    println!("✅ Chunks received: {}", chunk_times.len());

    if !chunk_times.is_empty() {
        let avg_chunk_time = chunk_times.iter().sum::<std::time::Duration>() / chunk_times.len() as u32;
        println!("✅ Average inter-chunk time: {:?}", avg_chunk_time);
    }

    // Performance assertions
    assert!(
        ttfb.unwrap() < std::time::Duration::from_millis(1000),
        "TTFB should be < 1s for Haiku"
    );
    assert!(chunk_times.len() > 0, "Should receive chunks");

    Ok(())
}
