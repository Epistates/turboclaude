# TurboClaude Skills

A Rust library for managing and executing skills in Claude agents. This crate provides the core infrastructure for loading, validating, and executing skills that follow the [Agent Skills Spec v1.0](https://github.com/anthropics/agent-skills-spec).

## Features

- **Skill Loading**: Parse and validate SKILL.md files with YAML frontmatter
- **Skill Registry**: Discover and manage multiple skills across directories
- **Semantic Matching**: Find relevant skills based on keywords
- **Script Execution**: Execute Python and Bash scripts with timeout support
- **Lazy Loading**: Load references and scripts on-demand for performance
- **Validation**: Enforce naming conventions and structure requirements
- **Thread-Safe**: Concurrent skill access with Arc<RwLock<>>

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
turboclaude-skills = "0.1.0"
```

## Quick Start

### Loading a Single Skill

```rust
use turboclaude_skills::Skill;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load skill from SKILL.md file
    let skill = Skill::from_file("path/to/skill/SKILL.md").await?;

    println!("Skill: {}", skill.metadata.name);
    println!("Description: {}", skill.metadata.description);

    // Check tool permissions
    if skill.metadata.allows_tool("bash") {
        println!("Skill can use bash");
    }

    Ok(())
}
```

### Using a Skill Registry

```rust
use turboclaude_skills::SkillRegistry;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create registry with skill directories
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("./skills"))
        .skill_dir(PathBuf::from("/path/to/more/skills"))
        .build()?;

    // Discover all skills
    let report = registry.discover().await?;
    println!("Loaded {} skills", report.loaded);

    // List all skills
    let skills = registry.list().await;
    for skill in skills {
        println!("• {} - {}", skill.name, skill.description);
    }

    // Find skills by keyword
    let matches = registry.find("git").await?;

    // Get specific skill
    let skill = registry.get("git-helper").await?;

    Ok(())
}
```

### Executing Scripts

```rust
use turboclaude_skills::Skill;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let skill = Skill::from_file("skills/git-helper/SKILL.md").await?;

    // Execute script with arguments and timeout
    let output = skill.execute_script(
        "analyze_branches",  // script name (without extension)
        &["--days=30"],      // arguments
        Some(Duration::from_secs(60))  // timeout
    ).await?;

    if output.success() {
        println!("Output: {}", output.stdout);
    } else if output.timed_out {
        eprintln!("Script timed out");
    } else {
        eprintln!("Error: {}", output.stderr);
    }

    Ok(())
}
```

## Skill Structure

Skills follow this directory structure:

```
my-skill/
├── SKILL.md              # Required: Main skill definition
├── reference/            # Optional: Additional documentation
│   ├── guide.md
│   └── advanced.md
└── scripts/              # Optional: Executable utilities
    ├── process.py        # Python scripts
    └── analyze.sh        # Bash scripts
```

### SKILL.md Format

```markdown
---
name: my-skill
description: What this skill does and when to use it
license: MIT
allowed-tools:
  - bash
  - read
  - write
metadata:
  author: Your Name
  version: "1.0.0"
  tags:
    - category
    - keywords
---

# Skill Title

Your skill documentation in Markdown format...

## Usage

Instructions for using this skill...
```

**Required Fields:**
- `name`: Skill identifier (hyphen-case: lowercase + hyphens only)
- `description`: Clear description for semantic matching

**Optional Fields:**
- `license`: License type
- `allowed-tools`: Tool whitelist (empty = no tools, missing = all tools)
- `metadata`: Custom key-value pairs

### Naming Rules

Skill names must be in hyphen-case:

```rust
// Valid names
"my-skill"
"git-helper"
"pdf-processor"

// Invalid names
"MySkill"      // uppercase
"my_skill"     // underscores
"-skill"       // leading hyphen
"skill-"       // trailing hyphen
```

## Examples

See the `examples/` directory for complete examples:

- **basic.rs**: Load skills and use the registry
- See also `../../examples/skills_demo.rs` for a comprehensive demonstration

### Example Skills

The `skills/` directory contains example skills:

- **git-helper**: Comprehensive Git repository analysis
  - Branch analysis and cleanup recommendations
  - Commit statistics and patterns
  - Includes Python and Bash scripts
  - Reference documentation for branching and commits

## API Overview

### Core Types

- **`Skill`**: Main skill object with metadata, content, and resources
- **`SkillMetadata`**: Parsed YAML frontmatter
- **`SkillRegistry`**: Discovery and management system
- **`Reference`**: Additional documentation files
- **`ScriptOutput`**: Script execution results

### Key Methods

#### Skill

```rust
// Load from file
let skill = Skill::from_file(path).await?;

// Access metadata
let name = &skill.metadata.name;
let description = &skill.metadata.description;

// Check tool permissions
skill.metadata.allows_tool("bash");

// Get skill context (full SKILL.md content)
let context = skill.context();

// Lazy load references
let references = skill.references().await?;

// Lazy load scripts
let scripts = skill.scripts().await?;

// Execute script
let output = skill.execute_script(name, args, timeout).await?;
```

#### SkillRegistry

```rust
// Build registry
let registry = SkillRegistry::builder()
    .skill_dir(path)
    .build()?;

// Discover skills
let report = registry.discover().await?;

// List all skills
let skills = registry.list().await;

// Find by keyword
let matches = registry.find("query").await?;

// Get specific skill
let skill = registry.get("skill-name").await?;
```

## Validation

The library validates:

- **Name format**: Must be hyphen-case
- **Directory matching**: Directory name must match `name` field
- **File sizes**: SKILL.md max 10MB, references max 50MB each
- **YAML syntax**: Valid frontmatter
- **Required fields**: Name and description must be present

## Script Execution

Supports Python (.py) and Bash (.sh) scripts:

```rust
// Automatic executor selection based on extension
let output = skill.execute_script("process", &[], None).await?;

// Check results
if output.success() {
    // exit_code == 0 && !timed_out
}
```

**ScriptOutput fields:**
- `exit_code`: Process exit code
- `stdout`: Standard output
- `stderr`: Standard error
- `duration`: Execution time
- `timed_out`: Whether timeout occurred

## Error Handling

All errors use the `SkillError` enum:

```rust
use turboclaude_skills::SkillError;

match skill.execute_script("test", &[], None).await {
    Ok(output) => println!("{}", output.stdout),
    Err(SkillError::ScriptNotFound(name)) => {
        eprintln!("Script {} not found", name);
    }
    Err(SkillError::ScriptTimeout) => {
        eprintln!("Script timed out");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance

- **Lazy loading**: References and scripts loaded only when accessed
- **Concurrent access**: Thread-safe skill registry
- **Caching**: Loaded skills cached in registry
- **Efficient discovery**: Parallel skill loading during discovery

## Testing

Run tests:

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_skill_loading
```

Run examples:

```bash
# Basic example
cargo run --example basic

# With example skills
cd crates/turboclaude-skills
cargo run --example basic
```

## Documentation

Generate and view API documentation:

```bash
cargo doc --open
```

## Size Limits

- **SKILL.md**: Maximum 10 MB
- **Reference files**: Maximum 50 MB each
- **Scripts**: No size limit

## Thread Safety

All types are thread-safe:

- `SkillRegistry` uses `Arc<RwLock<SkillRegistryInner>>`
- Skills are cloneable (via `Arc` internally)
- Concurrent access is safe and efficient

## Agent Skills Spec Compliance

This library implements the [Agent Skills Spec v1.0](https://github.com/anthropics/agent-skills-spec):

- ✅ SKILL.md with YAML frontmatter
- ✅ Required fields (name, description)
- ✅ Optional fields (license, allowed-tools, metadata)
- ✅ Reference documentation support
- ✅ Script execution support
- ✅ Naming convention enforcement
- ✅ Tool access control

## Related Crates

- **`turboclaude`**: Claude API client with Skills API support
- **`turboclaudeagent`**: Agent framework with skill integration

## License

See LICENSE file in the repository root.

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines.

## Resources

- [Agent Skills Spec](https://github.com/anthropics/agent-skills-spec)
- [TurboClaude Repository](https://github.com/epistates/turboclaude)
- [Example Skills](./skills/)
