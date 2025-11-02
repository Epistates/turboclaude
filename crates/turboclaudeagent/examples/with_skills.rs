//! Example: Using skills with an agent session
//!
//! This example demonstrates the full skills integration:
//! - Discovering skills from filesystem
//! - Loading skills into a session
//! - Skills automatically injecting context into queries
//! - Tool validation against skill constraints
//! - Semantic skill search
//!
//! # Prerequisites
//!
//! 1. Create a `./skills` directory with SKILL.md files
//! 2. Enable the `skills` feature: `cargo run --example with_skills --features skills`
//!
//! # Skill Directory Structure
//!
//! ```text
//! skills/
//! ├── example-skill/
//! │   └── SKILL.md
//! └── another-skill/
//!     └── SKILL.md
//! ```
//!
//! # Example SKILL.md
//!
//! ```markdown
//! ---
//! name: example-skill
//! description: An example skill for demonstration
//! allowed-tools:
//!   - bash
//!   - read
//!   - write
//! ---
//!
//! # Example Skill
//!
//! This skill demonstrates how to structure skill documentation.
//! ```

#[cfg(feature = "skills")]
use std::path::PathBuf;
#[cfg(feature = "skills")]
use turboclaudeagent::{AgentSession, SessionConfig};

#[cfg(feature = "skills")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TurboClaude Agent Skills Integration Example ===\n");

    // Configure session with skills directory
    let mut config = SessionConfig::default();
    config.skill_dirs = vec![
        PathBuf::from("./skills"),
        PathBuf::from("./crates/turboclaude-skills/tests/fixtures/skills"),
    ];

    println!("📁 Skill directories:");
    for dir in &config.skill_dirs {
        println!("   - {}", dir.display());
    }
    println!();

    // Create session directly (note: requires Claude Code CLI to be available)
    println!("🚀 Creating agent session...");
    let session = AgentSession::new(config).await?;
    println!("✅ Session created\n");

    //
    // Step 1: Discover skills
    //
    println!("🔍 Discovering skills...");
    let result = session.discover_skills().await?;
    println!(
        "✅ Discovery complete: {} loaded, {} failed",
        result.loaded, result.failed
    );

    if !result.errors.is_empty() {
        println!("\n⚠️  Errors during discovery:");
        for error in &result.errors {
            println!("   - {}", error);
        }
    }
    println!();

    //
    // Step 2: List available skills
    //
    println!("📚 Available skills:");
    let available = session.available_skills().await;
    if available.is_empty() {
        println!("   (none found - try creating skills in ./skills directory)");
        return Ok(());
    }

    for skill in &available {
        // Get full skill details
        let skill_obj = session.get_skill(skill).await?;
        println!("   - {} - {}", skill, skill_obj.metadata.description);
    }
    println!();

    //
    // Step 3: Semantic search
    //
    if !available.is_empty() {
        println!("🔎 Testing semantic search (query: 'minimal')...");
        let matches = session.find_skills("minimal").await?;
        println!("   Found {} matching skills:", matches.len());
        for skill in &matches {
            println!("     - {}", skill.metadata.name);
        }
        println!();
    }

    //
    // Step 4: Load a skill
    //
    if let Some(first_skill) = available.first() {
        println!("📥 Loading skill '{}'...", first_skill);
        session.load_skill(first_skill).await?;
        println!("✅ Skill loaded\n");

        //
        // Step 5: Verify active skills
        //
        let active = session.active_skills().await;
        println!("✨ Active skills:");
        for skill in &active {
            println!("   - {}", skill);
        }
        println!();

        //
        // Step 6: Tool validation
        //
        println!("🔒 Testing tool validation...");
        let test_tools = vec!["bash", "read", "write", "dangerous-tool"];

        for tool in &test_tools {
            let validation = session.validate_tool(tool).await;
            let status = if validation.allowed {
                "✅ ALLOWED"
            } else {
                "❌ BLOCKED"
            };

            print!("   {} - {}", tool, status);
            if let Some(blocker) = validation.blocked_by {
                print!(" (by {})", blocker);
            }
            println!();
        }
        println!();

        //
        // Step 7: Query with skill context (automatic injection)
        //
        println!("💬 Sending query with skill context...");
        println!("   (Skill context will be automatically injected into the prompt)\n");

        // Note: This would normally execute a real query
        // For this example, we just demonstrate the API
        println!("   Example query: 'What skills do you have access to?'");
        println!("   The skill's content would be appended to the system prompt automatically.");
        println!();

        /*
        // Uncomment to run a real query (requires Claude Code CLI):
        let response = session
            .query_str("What skills do you have access to?")
            .max_tokens(500)
            .await?;
        println!("Response: {}\n", response.content);
        */

        //
        // Step 8: Unload the skill
        //
        println!("📤 Unloading skill '{}'...", first_skill);
        session.unload_skill(first_skill).await?;
        println!("✅ Skill unloaded\n");

        let active = session.active_skills().await;
        println!("✨ Active skills: {:?}", active);
    }

    //
    // Cleanup
    //
    println!("\n🧹 Closing session...");
    session.close().await?;
    println!("✅ Session closed");

    Ok(())
}

#[cfg(not(feature = "skills"))]
fn main() {
    eprintln!("❌ This example requires the 'skills' feature.");
    eprintln!("   Run with: cargo run --example with_skills --features skills");
    std::process::exit(1);
}
