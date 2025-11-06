//! Basic example of loading and using a skill

use std::path::PathBuf;
use turboclaude_skills::{Skill, SkillRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("TurboClaude Skills - Basic Example\n");

    // Example 1: Load a single skill directly
    println!("=== Example 1: Load Single Skill ===\n");

    // For this example to work, you need a skill directory
    // We'll use the example skills in the skills/ directory
    let skill_path = "skills/git-helper/SKILL.md";

    match Skill::from_file(skill_path).await {
        Ok(skill) => {
            println!("Loaded skill: {}", skill.metadata.name);
            println!("Description: {}", skill.metadata.description);

            if let Some(license) = &skill.metadata.license {
                println!("License: {}", license);
            }

            let allowed_tools = skill.get_allowed_tools();
            if allowed_tools.is_empty() {
                println!("Allowed tools: ALL");
            } else {
                println!("Allowed tools: {:?}", allowed_tools);
            }

            println!("\n--- Skill Context ---");
            println!("{}", skill.context());
        }
        Err(e) => {
            eprintln!("Note: Could not load example skill: {}", e);
            eprintln!(
                "Create a test skill at {} to see this example work",
                skill_path
            );
        }
    }

    // Example 2: Use a registry for discovery
    println!("\n=== Example 2: Skill Registry ===\n");

    // Discover skills from the skills directory
    let skill_dirs = vec![
        PathBuf::from("skills"),
    ];

    let mut registry = SkillRegistry::builder()
        .skill_dirs(skill_dirs.into_iter().filter(|p| p.exists()).collect())
        .build()?;

    let report = registry.discover().await?;
    println!("Discovery complete:");
    println!("  Loaded: {} skills", report.loaded);
    println!("  Failed: {} directories", report.failed);

    if !report.errors.is_empty() {
        println!("\nErrors:");
        for (path, err) in &report.errors {
            println!("  {:?}: {}", path, err);
        }
    }

    // List all discovered skills
    println!("\n--- Available Skills ---");
    let skills = registry.list().await;
    for skill_meta in &skills {
        println!("• {} - {}", skill_meta.name, skill_meta.description);
    }

    // Example 3: Semantic search
    if !skills.is_empty() {
        println!("\n=== Example 3: Semantic Search ===\n");

        let queries = vec!["git", "branch", "commit", "repository"];

        for query in queries {
            let matches = registry.find(query).await?;
            println!("Query '{}': {} matches", query, matches.len());
            for skill in matches.iter().take(3) {
                println!("  → {}", skill.metadata.name);
            }
        }
    }

    // Example 4: Get specific skill
    if !skills.is_empty() {
        println!("\n=== Example 4: Get Specific Skill ===\n");

        let skill_name = &skills[0].name;
        match registry.get(skill_name).await {
            Ok(skill) => {
                println!("Retrieved: {}", skill.metadata.name);
                println!("Content length: {} bytes", skill.content.len());

                // Check for references
                let refs = skill.references().await?;
                println!("References: {} files", refs.len());

                // Check for scripts
                let scripts = skill.scripts().await?;
                println!("Scripts: {} files", scripts.len());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}
