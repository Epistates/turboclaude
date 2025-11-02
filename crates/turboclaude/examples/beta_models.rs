//! Example: Beta Models API
//!
//! Demonstrates listing and retrieving model information using the Beta Models API.
//!
//! # Usage
//!
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example beta_models
//! ```

use turboclaude::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from environment variable
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = Client::new(&api_key);

    println!("=== Beta Models API Example ===\n");

    // Example 1: List all models
    println!("1. Listing all available models:");
    println!("{}", "-".repeat(50));

    let models_page = client.beta().models().list().limit(20).send().await?;

    println!("Found {} models:", models_page.data.len());
    for model in &models_page.data {
        println!("  - {} ({})", model.display_name, model.id);
        println!("    Created: {}", model.created_at);
    }

    if models_page.has_more {
        println!("\nMore models available (has_more: true)");
        if let Some(last_id) = &models_page.last_id {
            println!("Use .after(\"{}\") to get next page", last_id);
        }
    }

    // Example 2: Retrieve specific model by ID
    println!("\n2. Retrieving specific model by ID:");
    println!("{}", "-".repeat(50));

    let model_id = "claude-3-5-sonnet-20241022";
    match client.beta().models().retrieve(model_id).await {
        Ok(model) => {
            println!("Model Details:");
            println!("  ID: {}", model.id);
            println!("  Display Name: {}", model.display_name);
            println!("  Type: {}", model.model_type);
            println!("  Created: {}", model.created_at);
        }
        Err(e) => {
            eprintln!("Error retrieving model: {}", e);
        }
    }

    // Example 3: Retrieve model by alias
    println!("\n3. Retrieving model by alias:");
    println!("{}", "-".repeat(50));

    let alias = "claude-3-5-sonnet";
    match client.beta().models().retrieve(alias).await {
        Ok(model) => {
            println!("Alias '{}' resolved to:", alias);
            println!("  ID: {}", model.id);
            println!("  Display Name: {}", model.display_name);
        }
        Err(e) => {
            eprintln!("Error retrieving model by alias: {}", e);
        }
    }

    // Example 4: Pagination with after_id
    println!("\n4. Pagination example:");
    println!("{}", "-".repeat(50));

    let first_page = client.beta().models().list().limit(3).send().await?;

    println!("First page: {} models", first_page.data.len());
    for model in &first_page.data {
        println!("  - {}", model.display_name);
    }

    if first_page.has_more {
        if let Some(last_id) = &first_page.last_id {
            println!("\nFetching next page after '{}'...", last_id);

            let second_page = client
                .beta()
                .models()
                .list()
                .after(last_id)
                .limit(3)
                .send()
                .await?;

            println!("Second page: {} models", second_page.data.len());
            for model in &second_page.data {
                println!("  - {}", model.display_name);
            }
        }
    }

    // Example 5: Check specific models availability
    println!("\n5. Checking specific models:");
    println!("{}", "-".repeat(50));

    let model_ids = vec![
        "claude-3-5-sonnet-20241022",
        "claude-3-5-haiku-20241022",
        "claude-3-opus-20240229",
    ];

    for model_id in model_ids {
        match client.beta().models().retrieve(model_id).await {
            Ok(model) => {
                println!("✓ {} - Available", model.display_name);
            }
            Err(_) => {
                println!("✗ {} - Not available", model_id);
            }
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
