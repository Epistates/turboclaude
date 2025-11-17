//! Structured Outputs Example
//!
//! This example demonstrates Claude's structured outputs feature, which allows you to
//! get type-safe JSON responses that match a predefined schema.
//!
//! Run this example with:
//! ```bash
//! cargo run --example structured_outputs --features schema
//! ```
//!
//! Requires the ANTHROPIC_API_KEY environment variable to be set.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use turboclaude::{Client, Message};
use turboclaude_protocol::types::models;

/// Example 1: Simple order extraction from text
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Order {
    /// Name of the product being ordered
    product_name: String,
    /// Price per unit in USD
    price: f64,
    /// Quantity to order
    quantity: u32,
}

/// Example 2: Complex user profile with nested data
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct UserProfile {
    /// Full name of the user
    name: String,
    /// Email address
    email: String,
    /// Age in years
    age: u32,
    /// Home address
    address: Address,
    /// List of interests/hobbies
    interests: Vec<String>,
    /// Whether the user is a premium member
    is_premium: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Address {
    street: String,
    city: String,
    state: String,
    zip_code: String,
}

/// Example 3: Code analysis output
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CodeAnalysis {
    /// Programming language detected
    language: String,
    /// Complexity score (1-10)
    complexity: u8,
    /// Issues or suggestions
    issues: Vec<Issue>,
    /// Overall code quality (1-100)
    quality_score: u8,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Issue {
    /// Line number where issue occurs
    line: u32,
    /// Severity level (info, warning, error)
    severity: String,
    /// Description of the issue
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable not set");
    let client = Client::new(api_key);

    println!("=== Structured Outputs Examples ===\n");

    // Example 1: Extract order information from natural language
    println!("Example 1: Order Extraction");
    println!("{}", "-".repeat(50));

    let order_text = "I'd like to order 3 Green Tea boxes at $15.99 each please.";
    println!("Input: {}", order_text);

    let parsed = client
        .beta()
        .messages()
        .parse::<Order>()
        .model(models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS.to_string())
        .messages(vec![Message::user(format!(
            "Extract the order details from this text: '{}'",
            order_text
        ))])
        .max_tokens(1024)
        .send()
        .await?;

    let order = parsed.parsed_output()?;
    println!("Parsed Order:");
    println!("  Product: {}", order.product_name);
    println!("  Price: ${:.2}", order.price);
    println!("  Quantity: {}", order.quantity);
    println!("  Total: ${:.2}\n", order.price * order.quantity as f64);

    // Example 2: Extract structured user profile
    println!("Example 2: User Profile Extraction");
    println!("{}", "-".repeat(50));

    let profile_text = r#"
    John Smith is 28 years old and lives at 123 Main St, San Francisco, CA 94102.
    His email is john.smith@example.com. He's interested in hiking, photography, and coding.
    He has a premium membership.
    "#;
    println!("Input: {}", profile_text.trim());

    let parsed = client
        .beta()
        .messages()
        .parse::<UserProfile>()
        .model(models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS.to_string())
        .messages(vec![Message::user(format!(
            "Extract the user profile from this text: {}",
            profile_text
        ))])
        .max_tokens(1024)
        .send()
        .await?;

    let profile = parsed.parsed_output()?;
    println!("Parsed Profile:");
    println!("  Name: {}", profile.name);
    println!("  Email: {}", profile.email);
    println!("  Age: {}", profile.age);
    println!("  Address: {}, {}, {} {}",
        profile.address.street,
        profile.address.city,
        profile.address.state,
        profile.address.zip_code
    );
    println!("  Interests: {:?}", profile.interests);
    println!("  Premium: {}\n", profile.is_premium);

    // Example 3: Analyze code quality
    println!("Example 3: Code Analysis");
    println!("{}", "-".repeat(50));

    let code = r#"
    def calculate(x, y):
        result = x + y
        print(result)
        return result
    "#;
    println!("Code to analyze:\n{}", code);

    let parsed = client
        .beta()
        .messages()
        .parse::<CodeAnalysis>()
        .model(models::CLAUDE_SONNET_4_5_20250929_STRUCTURED_OUTPUTS.to_string())
        .messages(vec![Message::user(format!(
            "Analyze this code and return a structured analysis:\n{}",
            code
        ))])
        .max_tokens(1024)
        .send()
        .await?;

    let analysis = parsed.parsed_output()?;
    println!("Analysis Results:");
    println!("  Language: {}", analysis.language);
    println!("  Complexity: {}/10", analysis.complexity);
    println!("  Quality Score: {}/100", analysis.quality_score);
    println!("  Issues:");
    for issue in &analysis.issues {
        println!("    Line {}: [{}] {}",
            issue.line,
            issue.severity,
            issue.message
        );
    }

    // Show usage statistics
    println!("\n=== Usage Statistics ===");
    println!("Input Tokens: {}", parsed.message().usage.input_tokens);
    println!("Output Tokens: {}", parsed.message().usage.output_tokens);

    Ok(())
}
