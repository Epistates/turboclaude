//! Example demonstrating the Beta Skills API
//!
//! This example shows how to:
//! - Create a skill from files
//! - List skills with pagination and filtering
//! - Retrieve skill details
//! - Manage skill versions
//! - Delete skills
//!
//! Run with:
//! ```bash
//! ANTHROPIC_API_KEY=sk-ant-... cargo run --example beta_skills
//! ```

use turboclaude::Client;
use turboclaude::types::beta::SkillSource;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from environment variable
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = Client::new(api_key);

    println!("Beta Skills API Example\n");

    // ========================================
    // Example 1: Create a new skill
    // ========================================
    println!("1. Creating a skill...");

    // Define a simple skill with SKILL.md
    let skill_md = br#"---
name: example-calculator
description: A simple calculator skill that performs basic arithmetic
---

# Example Calculator

This skill provides basic calculator functionality.

## Usage

The calculator can perform:
- Addition
- Subtraction
- Multiplication
- Division
"#;

    // Create the skill
    match client
        .beta()
        .skills()
        .create()
        .file("example-calculator/SKILL.md", skill_md.to_vec())
        .display_title("Example Calculator")
        .send()
        .await
    {
        Ok(skill) => {
            println!("   Created skill: {}", skill.id);
            println!(
                "   Display title: {}",
                skill.display_title.as_deref().unwrap_or("(none)")
            );
            println!("   Source: {}", skill.source);
            println!("   Latest version: {:?}\n", skill.latest_version);

            // Store the skill ID for later operations
            let skill_id = skill.id.clone();

            // ========================================
            // Example 2: Retrieve the skill
            // ========================================
            println!("2. Retrieving skill details...");

            match client.beta().skills().retrieve(&skill_id).await {
                Ok(skill) => {
                    println!("   Retrieved: {}", skill.id);
                    println!("   Created at: {}", skill.created_at);
                    println!("   Updated at: {}\n", skill.updated_at);
                }
                Err(e) => eprintln!("   Error retrieving skill: {}\n", e),
            }

            // ========================================
            // Example 3: Create a new version
            // ========================================
            println!("3. Creating a new version...");

            let updated_skill_md = br#"---
name: example-calculator
description: Enhanced calculator with more operations
---

# Example Calculator v2

Now with square root support!
"#;

            match client
                .beta()
                .skills()
                .versions(&skill_id)
                .create()
                .file("example-calculator/SKILL.md", updated_skill_md.to_vec())
                .send()
                .await
            {
                Ok(version) => {
                    println!("   Created version: {}", version.version);
                    println!("   Name: {}", version.name);
                    println!("   Description: {}", version.description);
                    println!("   Directory: {}\n", version.directory);
                }
                Err(e) => eprintln!("   Error creating version: {}\n", e),
            }

            // ========================================
            // Example 4: List all versions
            // ========================================
            println!("4. Listing skill versions...");

            match client
                .beta()
                .skills()
                .versions(&skill_id)
                .list()
                .limit(10)
                .send()
                .await
            {
                Ok(page) => {
                    println!("   Found {} version(s)", page.data.len());
                    for version in &page.data {
                        println!("   - {} ({})", version.version, version.name);
                    }
                    if page.has_more {
                        println!("   (more versions available)\n");
                    } else {
                        println!();
                    }
                }
                Err(e) => eprintln!("   Error listing versions: {}\n", e),
            }

            // ========================================
            // Example 5: Delete the skill
            // ========================================
            println!("5. Cleaning up - deleting skill...");

            match client.beta().skills().delete(&skill_id).await {
                Ok(deleted) => {
                    println!("   Deleted: {}", deleted.id);
                    println!("   Type: {}", deleted.type_);
                    println!("   Success: {}\n", deleted.deleted);
                }
                Err(e) => eprintln!("   Error deleting skill: {}\n", e),
            }
        }
        Err(e) => {
            eprintln!("   Error creating skill: {}\n", e);
        }
    }

    // ========================================
    // Example 6: List all skills
    // ========================================
    println!("6. Listing all skills...");

    match client.beta().skills().list().limit(20).send().await {
        Ok(page) => {
            println!("   Found {} skill(s) on this page", page.data.len());
            for skill in &page.data {
                println!(
                    "   - {} ({}): {}",
                    skill.id,
                    skill.source,
                    skill.display_title.as_deref().unwrap_or("(no title)")
                );
            }

            if page.has_more {
                println!("\n   More skills available. Use pagination to fetch next page:");
                println!("   .list().page(next_token).send().await");
            }
            println!();
        }
        Err(e) => eprintln!("   Error listing skills: {}\n", e),
    }

    // ========================================
    // Example 7: List only custom skills
    // ========================================
    println!("7. Listing custom skills only...");

    match client
        .beta()
        .skills()
        .list()
        .source(SkillSource::Custom)
        .limit(10)
        .send()
        .await
    {
        Ok(page) => {
            println!("   Found {} custom skill(s)", page.data.len());
            for skill in &page.data {
                println!(
                    "   - {}: {}",
                    skill.id,
                    skill.display_title.as_deref().unwrap_or("(no title)")
                );
            }
            println!();
        }
        Err(e) => eprintln!("   Error listing custom skills: {}\n", e),
    }

    // ========================================
    // Example 8: List Anthropic skills
    // ========================================
    println!("8. Listing Anthropic-created skills...");

    match client
        .beta()
        .skills()
        .list()
        .source(SkillSource::Anthropic)
        .limit(10)
        .send()
        .await
    {
        Ok(page) => {
            println!("   Found {} Anthropic skill(s)", page.data.len());
            for skill in &page.data {
                println!(
                    "   - {}: {}",
                    skill.id,
                    skill.display_title.as_deref().unwrap_or("(no title)")
                );
            }
            println!();
        }
        Err(e) => eprintln!("   Error listing Anthropic skills: {}\n", e),
    }

    println!("Done! Beta Skills API example completed successfully.");

    Ok(())
}
