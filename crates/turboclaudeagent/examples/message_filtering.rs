//! Example: Message Filtering
//!
//! Demonstrates the ergonomic message filtering methods for processing
//! specific message types from the agent session stream.
//!
//! This example shows the API usage patterns. In a real application, you would
//! need a running Claude CLI process and proper session initialization.

fn main() {
    println!("Message Filtering API Examples\n");
    println!("===============================\n");

    println!("Example 1: Filtering for Assistant Messages Only");
    println!("------------------------------------------------");
    println!(
        r#"
use futures::StreamExt;

// Create session
let session = AgentSession::new(config).await?;

// Send a query
session.query_str("What is 2+2?").await?;

// Receive ONLY assistant messages (other message types are filtered out)
let mut stream = Box::pin(session.receive_assistant_messages().await);

while let Some(result) = stream.next().await {{
    let msg = result?; // msg is AssistantMessage, not ParsedMessage!
    println!("Model: {{}}", msg.model);
    for content in msg.content {{
        if let Some(text) = content.as_text() {{
            println!("  {{}}", text);
        }}
    }}
}}
"#
    );

    println!("\nExample 2: Filtering for Stream Events Only");
    println!("------------------------------------------");
    println!(
        r#"
// Receive ONLY stream events (partial message updates)
let mut stream = Box::pin(session.receive_stream_events().await);

while let Some(result) = stream.next().await {{
    let event = result?; // event is StreamEvent, not ParsedMessage!
    println!("Event UUID: {{}}", event.uuid);
    println!("Event data: {{:?}}", event.event);
}}
"#
    );

    println!("\nExample 3: Filtering for Result Messages (Query Completion)");
    println!("----------------------------------------------------------");
    println!(
        r#"
// Receive ONLY result messages (final query completion info)
let mut stream = Box::pin(session.receive_results().await);

while let Some(result) = stream.next().await {{
    let msg = result?; // msg is ResultMessage, not ParsedMessage!
    println!("Duration: {{}}ms", msg.duration_ms);
    println!("Turns: {{}}", msg.num_turns);
    println!("Error: {{}}", msg.is_error);
    if let Some(cost) = msg.total_cost_usd {{
        println!("Cost: ${{:.4}}", cost);
    }}
}}
"#
    );

    println!("\nExample 4: Filtering for User Messages");
    println!("-------------------------------------");
    println!(
        r#"
// Receive ONLY user messages
let mut stream = Box::pin(session.receive_user_messages().await);

while let Some(result) = stream.next().await {{
    let msg = result?; // msg is UserMessage, not ParsedMessage!
    for content in msg.content {{
        if let Some(text) = content.as_text() {{
            println!("User said: {{}}", text);
        }}
    }}
}}
"#
    );

    println!("\nExample 5: Filtering for System Messages");
    println!("---------------------------------------");
    println!(
        r#"
// Receive ONLY system messages
let mut stream = Box::pin(session.receive_system_messages().await);

while let Some(result) = stream.next().await {{
    let msg = result?; // msg is SystemMessage, not ParsedMessage!
    println!("System event: {{}}", msg.subtype);
}}
"#
    );

    println!("\nBenefits:");
    println!("--------");
    println!("✓ Strongly-typed: No more ParsedMessage enum matching");
    println!("✓ Less boilerplate: Direct access to message types");
    println!("✓ Better errors: Type system catches misuse at compile time");
    println!("✓ More readable: Intent is clear from the method name");
    println!("✓ Error propagation: Errors pass through, not filtered out");
    println!("\nOld way (before filtering methods):");
    println!("----------------------------------");
    println!(
        r#"
let mut stream = session.receive_messages().await;
while let Some(result) = stream.next().await {{
    match result {{
        Ok(ParsedMessage::Assistant(msg)) => {{
            // Handle assistant message
        }}
        Ok(_) => continue, // Skip other types (verbose!)
        Err(e) => return Err(e),
    }}
}}
"#
    );

    println!("\nNew way (with filtering methods):");
    println!("--------------------------------");
    println!(
        r#"
let mut stream = Box::pin(session.receive_assistant_messages().await);
while let Some(result) = stream.next().await {{
    let msg = result?; // Already AssistantMessage!
    // Handle msg directly
}}
"#
    );
}
