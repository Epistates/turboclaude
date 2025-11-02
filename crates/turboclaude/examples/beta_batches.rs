//! Example: Beta Message Batches API
//!
//! Demonstrates creating and managing batch message processing.
//!
//! # Usage
//!
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example beta_batches
//! ```

use turboclaude::resources::messages::BatchRequest;
use turboclaude::{Client, Message, MessageRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from environment variable
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = Client::new(&api_key);

    println!("=== Beta Message Batches API Example ===\n");

    // Example 1: Create a batch
    println!("1. Creating a message batch:");
    println!("{}", "-".repeat(50));

    let batch_requests = vec![
        BatchRequest {
            custom_id: "req-001".to_string(),
            params: MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user("What is 2+2?")])
                .build()?,
        },
        BatchRequest {
            custom_id: "req-002".to_string(),
            params: MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user("What is the capital of France?")])
                .build()?,
        },
        BatchRequest {
            custom_id: "req-003".to_string(),
            params: MessageRequest::builder()
                .model("claude-3-5-sonnet-20241022")
                .max_tokens(100u32)
                .messages(vec![Message::user("Tell me a short joke.")])
                .build()?,
        },
    ];

    let batch = client.messages().batches().create(batch_requests).await?;

    println!("Batch created successfully!");
    println!("  Batch ID: {}", batch.id);
    println!("  Status: {:?}", batch.processing_status);
    println!("  Total requests: {}", batch.request_counts.total);
    println!("  Created at: {}", batch.created_at);
    println!("  Expires at: {}", batch.expires_at);

    let batch_id = batch.id.clone();

    // Example 2: Retrieve batch status
    println!("\n2. Checking batch status:");
    println!("{}", "-".repeat(50));

    let retrieved_batch = client.messages().batches().get(&batch_id).await?;

    println!("Batch Status:");
    println!(
        "  Processing status: {:?}",
        retrieved_batch.processing_status
    );
    println!("  Request counts:");
    println!("    Total: {}", retrieved_batch.request_counts.total);
    println!(
        "    Processing: {}",
        retrieved_batch.request_counts.processing
    );
    println!(
        "    Succeeded: {}",
        retrieved_batch.request_counts.succeeded
    );
    println!("    Errored: {}", retrieved_batch.request_counts.errored);
    println!("    Canceled: {}", retrieved_batch.request_counts.canceled);
    println!("    Expired: {}", retrieved_batch.request_counts.expired);

    // Example 3: List all batches
    println!("\n3. Listing all batches:");
    println!("{}", "-".repeat(50));

    let batches = client.messages().batches().list().await?;

    println!("Found {} batch(es):", batches.len());
    for (i, batch) in batches.iter().enumerate().take(5) {
        println!("  {}. {} - {:?}", i + 1, batch.id, batch.processing_status);
    }

    if batches.len() > 5 {
        println!("  ... and {} more", batches.len() - 5);
    }

    // Example 4: Poll for completion (with timeout)
    println!("\n4. Polling for batch completion:");
    println!("{}", "-".repeat(50));
    println!("Note: This will poll for up to 60 seconds...");

    use std::time::Duration;
    use tokio::time::sleep;

    let mut poll_count = 0;
    let max_polls = 12; // 60 seconds with 5-second intervals

    loop {
        poll_count += 1;

        let status_batch = client.messages().batches().get(&batch_id).await?;

        println!(
            "  Poll {}/{}: Status = {:?}, Succeeded = {}",
            poll_count,
            max_polls,
            status_batch.processing_status,
            status_batch.request_counts.succeeded
        );

        if status_batch.processing_status == turboclaude::types::batch::ProcessingStatus::Ended {
            println!("✓ Batch processing completed!");
            break;
        }

        if poll_count >= max_polls {
            println!("⚠ Reached polling timeout. Batch may still be processing.");
            break;
        }

        sleep(Duration::from_secs(5)).await;
    }

    // Example 5: Get batch results (if available)
    println!("\n5. Retrieving batch results:");
    println!("{}", "-".repeat(50));

    let final_batch = client.messages().batches().get(&batch_id).await?;

    if let Some(results_url) = &final_batch.results_url {
        println!("Results URL available: {}", results_url);

        match client.messages().batches().results(&batch_id).await {
            Ok(results) => {
                println!("Retrieved {} result(s):", results.len());

                for result in &results {
                    println!("\n  Custom ID: {}", result.custom_id);
                    match &result.result {
                        turboclaude::resources::messages::BatchResultType::Success { message } => {
                            println!("  Status: Success");
                            println!("  Message ID: {}", message.id);
                            println!("  Response: {}", message.text());
                            println!(
                                "  Tokens: input={}, output={}",
                                message.usage.input_tokens, message.usage.output_tokens
                            );
                        }
                        turboclaude::resources::messages::BatchResultType::Error { error } => {
                            println!("  Status: Error");
                            println!("  Error type: {}", error.error_type);
                            println!("  Error message: {}", error.message);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error retrieving results: {}", e);
            }
        }
    } else {
        println!("Results not yet available. Batch may still be processing.");
    }

    // Example 6: Cancel a batch (demonstration only - we'll create a new one)
    println!("\n6. Cancel batch demonstration:");
    println!("{}", "-".repeat(50));
    println!("Creating a new batch to demonstrate cancellation...");

    let cancel_batch_requests = vec![BatchRequest {
        custom_id: "cancel-001".to_string(),
        params: MessageRequest::builder()
            .model("claude-3-5-sonnet-20241022")
            .max_tokens(100u32)
            .messages(vec![Message::user("Test message for cancellation")])
            .build()?,
    }];

    let cancel_batch = client
        .messages()
        .batches()
        .create(cancel_batch_requests)
        .await?;

    println!("Created batch: {}", cancel_batch.id);
    println!("Canceling batch...");

    let canceled_batch = client.messages().batches().cancel(&cancel_batch.id).await?;

    println!("✓ Batch canceled");
    println!("  New status: {:?}", canceled_batch.processing_status);

    println!("\n=== Example Complete ===");
    println!("\nNote: Batch processing can take several minutes to hours depending on");
    println!("the queue. For production use, implement polling with exponential backoff");
    println!("or use webhooks if available.");

    Ok(())
}
