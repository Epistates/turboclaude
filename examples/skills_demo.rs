//! Comprehensive Skills Demo
//!
//! This example demonstrates the full capabilities of the TurboClaude Skills system:
//! 1. Loading skills from directories
//! 2. Using the skill registry for discovery
//! 3. Executing skill scripts
//! 4. Accessing reference documentation
//! 5. Checking tool permissions
//!
//! Run from the root of the repository:
//! ```bash
//! cargo run --example skills_demo
//! ```

use std::path::PathBuf;
use std::time::Duration;
use turboclaude_skills::{Skill, SkillRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("  TurboClaude Skills - Comprehensive Demo");
    println!("{}\n", "=".repeat(70));

    // Part 1: Direct Skill Loading
    demo_direct_loading().await?;

    // Part 2: Skill Registry
    demo_skill_registry().await?;

    // Part 3: Script Execution
    demo_script_execution().await?;

    // Part 4: Reference Documentation
    demo_references().await?;

    // Part 5: Tool Permissions
    demo_tool_permissions().await?;

    println!("\n{}", "=".repeat(70));
    println!("  Demo Complete!");
    println!("{}\n", "=".repeat(70));

    Ok(())
}

/// Demonstrates loading a skill directly from a file
async fn demo_direct_loading() -> anyhow::Result<()> {
    println!("{}", "-".repeat(70));
    println!("PART 1: Direct Skill Loading");
    println!("{}\n", "-".repeat(70));

    let skill_path = "crates/turboclaude-skills/skills/git-helper/SKILL.md";

    match Skill::from_file(skill_path).await {
        Ok(skill) => {
            println!("✓ Successfully loaded skill: {}", skill.metadata.name);
            println!("  Description: {}", skill.metadata.description);

            if let Some(license) = &skill.metadata.license {
                println!("  License: {}", license);
            }

            // Show metadata
            if let Some(metadata) = &skill.metadata.metadata {
                println!("\n  Metadata:");
                if let Some(author) = metadata.get("author") {
                    println!("    Author: {}", author);
                }
                if let Some(version) = metadata.get("version") {
                    println!("    Version: {}", version);
                }
                if let Some(category) = metadata.get("category") {
                    println!("    Category: {}", category);
                }
            }

            println!("\n  Skill directory: {}", skill.root.display());
            println!("  Content length: {} bytes", skill.content.len());
        }
        Err(e) => {
            println!("✗ Could not load skill: {}", e);
            println!("  Make sure you're running from the repository root");
        }
    }

    Ok(())
}

/// Demonstrates using the skill registry for discovery
async fn demo_skill_registry() -> anyhow::Result<()> {
    println!("\n{}", "-".repeat(70));
    println!("PART 2: Skill Registry & Discovery");
    println!("{}\n", "-".repeat(70));

    // Create registry with skill directory
    let skill_dir = PathBuf::from("crates/turboclaude-skills/skills");

    if !skill_dir.exists() {
        println!("✗ Skills directory not found: {}", skill_dir.display());
        return Ok(());
    }

    let mut registry = SkillRegistry::builder()
        .skill_dir(skill_dir.clone())
        .build()?;

    println!("Registry created with directory: {}", skill_dir.display());

    // Discover skills
    println!("\nDiscovering skills...");
    let report = registry.discover().await?;

    println!("✓ Discovery complete:");
    println!("  Loaded: {} skills", report.loaded);
    println!("  Failed: {} directories", report.failed);

    if !report.errors.is_empty() {
        println!("\n  Errors encountered:");
        for (path, error) in &report.errors {
            println!("    {:?}: {}", path, error);
        }
    }

    // List all skills
    println!("\n--- Available Skills ---");
    let skills = registry.list().await;

    if skills.is_empty() {
        println!("  (no skills found)");
    } else {
        for skill_meta in &skills {
            println!("  • {} - {}", skill_meta.name, skill_meta.description);
        }
    }

    // Semantic search
    if !skills.is_empty() {
        println!("\n--- Semantic Search ---");

        let queries = vec!["git", "branch", "commit", "analysis"];

        for query in queries {
            let matches = registry.find(query).await?;
            if !matches.is_empty() {
                println!("  Query '{}': {} match(es)", query, matches.len());
                for skill in matches.iter().take(3) {
                    println!("    → {}", skill.metadata.name);
                }
            }
        }
    }

    // Get specific skill
    if !skills.is_empty() {
        println!("\n--- Get Specific Skill ---");

        let skill_name = &skills[0].name;
        match registry.get(skill_name).await {
            Ok(skill) => {
                println!("  ✓ Retrieved: {}", skill.metadata.name);

                // Count lines in content
                let lines = skill.content.lines().count();
                println!("    Content: {} lines", lines);

                // Check for scripts
                let scripts = skill.scripts().await?;
                println!("    Scripts: {}", scripts.len());
                for (name, _path) in scripts.iter() {
                    println!("      - {}", name);
                }

                // Check for references
                let refs = skill.references().await?;
                println!("    References: {} file(s)", refs.len());
                for reference in refs.iter() {
                    println!("      - {} ({} bytes)", reference.path.display(), reference.content.len());
                }
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Demonstrates executing skill scripts
async fn demo_script_execution() -> anyhow::Result<()> {
    println!("\n{}", "-".repeat(70));
    println!("PART 3: Script Execution");
    println!("{}\n", "-".repeat(70));

    let skill_path = "crates/turboclaude-skills/skills/git-helper/SKILL.md";

    match Skill::from_file(skill_path).await {
        Ok(skill) => {
            let scripts = skill.scripts().await?;

            if scripts.is_empty() {
                println!("  No scripts found in this skill");
                return Ok(());
            }

            println!("Available scripts:");
            for (name, path) in scripts.iter() {
                println!("  • {} ({})", name, path.display());
            }

            // Example: Execute analyze_branches script
            if scripts.contains_key("analyze_branches") {
                println!("\n--- Executing: analyze_branches ---");

                match skill
                    .execute_script(
                        "analyze_branches",
                        &["--days=90", "--json"],
                        Some(Duration::from_secs(10)),
                    )
                    .await
                {
                    Ok(output) => {
                        if output.success() {
                            println!("✓ Script executed successfully");
                            println!("  Duration: {:?}", output.duration);

                            // Parse JSON output if it's valid
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output.stdout) {
                                println!("\n  JSON Output:");
                                if let Some(summary) = json.get("summary") {
                                    println!("    Total branches: {}", summary.get("total_branches").unwrap_or(&serde_json::json!(0)));
                                    println!("    Stale branches: {}", summary.get("stale_branches").unwrap_or(&serde_json::json!(0)));
                                    println!("    Active branches: {}", summary.get("active_branches").unwrap_or(&serde_json::json!(0)));
                                }
                            } else {
                                // Show first few lines of output
                                let lines: Vec<_> = output.stdout.lines().take(10).collect();
                                println!("\n  Output (first 10 lines):");
                                for line in lines {
                                    println!("    {}", line);
                                }
                            }
                        } else if output.timed_out {
                            println!("✗ Script timed out");
                        } else {
                            println!("✗ Script failed with exit code: {}", output.exit_code);
                            if !output.stderr.is_empty() {
                                println!("  Error: {}", output.stderr);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Execution error: {}", e);
                    }
                }
            }

            // Example: Execute commit_stats script
            if scripts.contains_key("commit_stats") {
                println!("\n--- Executing: commit_stats ---");

                match skill
                    .execute_script(
                        "commit_stats",
                        &["--since=1 month ago", "--json"],
                        Some(Duration::from_secs(10)),
                    )
                    .await
                {
                    Ok(output) => {
                        if output.success() {
                            println!("✓ Script executed successfully");
                            println!("  Duration: {:?}", output.duration);

                            // Try to parse JSON
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output.stdout) {
                                println!("\n  JSON Output:");
                                if let Some(summary) = json.get("summary") {
                                    println!("    Total commits: {}", summary.get("total_commits").unwrap_or(&serde_json::json!(0)));
                                    println!("    Authors: {}", summary.get("author_count").unwrap_or(&serde_json::json!(0)));
                                }
                            }
                        } else if output.timed_out {
                            println!("✗ Script timed out");
                        } else {
                            println!("✗ Script failed with exit code: {}", output.exit_code);
                        }
                    }
                    Err(e) => {
                        println!("✗ Execution error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Could not load skill: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates accessing reference documentation
async fn demo_references() -> anyhow::Result<()> {
    println!("\n{}", "-".repeat(70));
    println!("PART 4: Reference Documentation");
    println!("{}\n", "-".repeat(70));

    let skill_path = "crates/turboclaude-skills/skills/git-helper/SKILL.md";

    match Skill::from_file(skill_path).await {
        Ok(skill) => {
            let references = skill.references().await?;

            if references.is_empty() {
                println!("  No reference documentation found");
                return Ok(());
            }

            println!("Reference files:");
            for reference in &references {
                println!("\n  • {}", reference.path.display());
                println!("    Size: {} bytes", reference.content.len());
                println!("    Lines: {}", reference.content.lines().count());

                // Show first few lines
                let preview: Vec<_> = reference
                    .content
                    .lines()
                    .filter(|line| !line.is_empty())
                    .take(3)
                    .collect();

                println!("    Preview:");
                for line in preview {
                    println!("      {}", line.trim());
                }
            }

            // Demonstrate using reference content
            println!("\n--- Using Reference Content ---");
            if let Some(first_ref) = references.first() {
                println!("  Reference: {}", first_ref.path.display());

                // Count headings
                let headings: Vec<_> = first_ref
                    .content
                    .lines()
                    .filter(|line| line.starts_with('#'))
                    .collect();

                println!("  Sections: {}", headings.len());
                println!("  First 5 sections:");
                for heading in headings.iter().take(5) {
                    let level = heading.chars().take_while(|&c| c == '#').count();
                    let title = heading.trim_start_matches('#').trim();
                    println!("    {}{}", "  ".repeat(level - 1), title);
                }
            }
        }
        Err(e) => {
            println!("Could not load skill: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates tool permission checking
async fn demo_tool_permissions() -> antml:Result<()> {
    println!("\n{}", "-".repeat(70));
    println!("PART 5: Tool Permissions");
    println!("{}\n", "-".repeat(70));

    let skill_path = "crates/turboclaude-skills/skills/git-helper/SKILL.md";

    match Skill::from_file(skill_path).await {
        Ok(skill) => {
            println!("Skill: {}", skill.metadata.name);

            let allowed_tools = skill.get_allowed_tools();

            if allowed_tools.is_empty() {
                println!("\n  Tool Access: ALL TOOLS ALLOWED");
                println!("  (No 'allowed-tools' field specified in SKILL.md)");
            } else {
                println!("\n  Tool Access: RESTRICTED");
                println!("  Allowed tools:");
                for tool in &allowed_tools {
                    println!("    ✓ {}", tool);
                }
            }

            // Test specific tools
            println!("\n--- Permission Checks ---");
            let tools_to_check = vec!["bash", "read", "write", "grep", "edit", "web_fetch"];

            for tool in tools_to_check {
                let allowed = skill.metadata.allows_tool(tool);
                let status = if allowed { "✓ Allowed" } else { "✗ Denied" };
                println!("  {} - {}", tool, status);
            }

            // Show metadata
            if let Some(metadata) = &skill.metadata.metadata {
                println!("\n--- Custom Metadata ---");
                for (key, value) in metadata.iter() {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("Could not load skill: {}", e);
        }
    }

    Ok(())
}
