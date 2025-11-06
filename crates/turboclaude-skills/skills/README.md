# TurboClaude Skills Example

This directory contains example skills that demonstrate the TurboClaude Skills system. Skills are self-contained packages that extend Claude's capabilities with specialized knowledge, tools, and utilities.

## What are Skills?

Skills in TurboClaude are structured bundles of:
- **Knowledge**: Domain-specific information and instructions in SKILL.md
- **References**: Additional documentation in the `reference/` directory
- **Scripts**: Executable utilities in the `scripts/` directory (Python, Bash)

Skills follow the [Agent Skills Spec v1.0](https://github.com/anthropics/agent-skills-spec) standard.

## Directory Structure

```
skills/
└── git-helper/                    # Example skill
    ├── SKILL.md                  # Main skill definition (required)
    ├── reference/                # Additional documentation (optional)
    │   ├── branch-strategies.md
    │   └── commit-conventions.md
    └── scripts/                  # Executable utilities (optional)
        ├── analyze_branches.py
        └── commit_stats.sh
```

## Example Skill: git-helper

The `git-helper` skill demonstrates all key features of the skills system:

### 1. SKILL.md Format

The main skill file uses YAML frontmatter with metadata:

```yaml
---
name: git-helper
description: Advanced Git repository helper that provides branch analysis...
license: MIT
allowed-tools:
  - bash
  - read
  - write
  - grep
metadata:
  author: TurboClaude Team
  version: "1.0.0"
  category: development-tools
  tags:
    - git
    - version-control
---

# Skill content in Markdown...
```

**Required Fields:**
- `name`: Skill identifier (hyphen-case, must match directory name)
- `description`: What the skill does and when to use it

**Optional Fields:**
- `license`: License type (MIT, Apache-2.0, etc.)
- `allowed-tools`: Whitelist of tools the skill can use
- `metadata`: Custom key-value pairs

### 2. Reference Documentation

The `reference/` directory contains supplementary documentation:

- **branch-strategies.md**: Comprehensive guide to Git branching strategies
  - Git Flow, GitHub Flow, Trunk-Based Development
  - Branch lifecycle management
  - Best practices and decision matrices

- **commit-conventions.md**: Commit message best practices
  - Conventional Commits specification
  - Commit types and formatting
  - Automation and tooling

These files are lazily loaded when needed, keeping the main skill definition concise.

### 3. Executable Scripts

The `scripts/` directory contains utility programs:

#### analyze_branches.py (Python)
Analyzes repository branches and identifies:
- Stale branches (no activity for N days)
- Merged vs. unmerged branches
- Branch statistics and recommendations

**Usage:**
```bash
python analyze_branches.py --days=30 --remote
```

**Features:**
- Configurable staleness threshold
- Remote branch support
- JSON output option
- Comprehensive reporting

#### commit_stats.sh (Bash)
Generates commit statistics including:
- Commits per author
- Timeline analysis (monthly, daily, hourly)
- Conventional commit compliance
- Code churn metrics

**Usage:**
```bash
bash commit_stats.sh --since="6 months ago" --branch=main
```

**Features:**
- Flexible time range
- Colored terminal output
- Commit type analysis
- Team collaboration metrics

## Using Skills

### With TurboClaude Agent

```rust
use turboclaudeagent::{AgentSession, SessionConfig};
use std::path::PathBuf;

let mut config = SessionConfig::default();
config.skill_dirs = vec![PathBuf::from("./skills")];

let mut session = AgentSession::new(config).await?;
session.discover_skills().await?;

// Load specific skill
session.load_skill("git-helper").await?;
```

### With Local Skill Registry

```rust
use turboclaude_skills::SkillRegistry;
use std::path::PathBuf;

let mut registry = SkillRegistry::builder()
    .skill_dir(PathBuf::from("./skills"))
    .build()?;

// Discover all skills
registry.discover().await?;

// Find skills by keyword
let matches = registry.find("git branch").await?;

// Get specific skill
let skill = registry.get("git-helper").await?;

// Execute script
let output = skill.execute_script(
    "analyze_branches",
    &["--days=30"],
    None
).await?;
```

### With Claude Skills API

```rust
use turboclaude::Client;
use std::fs;

let client = Client::new("sk-ant-...");

// Upload skill to Claude
let skill_content = fs::read("skills/git-helper/SKILL.md")?;

let skill = client.beta().skills()
    .create()
    .file("git-helper/SKILL.md", skill_content)
    .display_title("Git Helper")
    .send()
    .await?;

println!("Created skill: {}", skill.id);
```

## Creating Your Own Skills

### 1. Create Directory Structure

```bash
mkdir -p skills/my-skill/{reference,scripts}
```

### 2. Write SKILL.md

```yaml
---
name: my-skill
description: Brief description of what your skill does
license: MIT
allowed-tools:
  - bash
  - read
metadata:
  author: Your Name
  version: "1.0.0"
---

# My Skill

Detailed description and usage instructions...
```

**Naming Rules:**
- Use lowercase with hyphens: `my-skill-name`
- Match directory name exactly
- No uppercase, underscores, or leading/trailing hyphens

### 3. Add Reference Documentation (Optional)

Create Markdown files in `reference/`:
```bash
echo "# Advanced Usage" > skills/my-skill/reference/advanced.md
```

### 4. Add Scripts (Optional)

Create executable scripts in `scripts/`:

**Python:**
```python
#!/usr/bin/env python3
# skills/my-skill/scripts/process.py

def main():
    print("Processing...")

if __name__ == '__main__':
    main()
```

**Bash:**
```bash
#!/bin/bash
# skills/my-skill/scripts/analyze.sh

echo "Analyzing..."
```

Make scripts executable:
```bash
chmod +x skills/my-skill/scripts/*
```

### 5. Test Your Skill

```rust
use turboclaude_skills::Skill;

// Load skill
let skill = Skill::from_file("skills/my-skill/SKILL.md").await?;

// Validate
assert_eq!(skill.metadata.name, "my-skill");

// Execute script
let output = skill.execute_script("process", &[], None).await?;
assert!(output.success());
```

## Skill Best Practices

### 1. Clear, Semantic Descriptions
Write descriptions that help Claude understand when to use the skill:

```yaml
# Good
description: Use this skill for analyzing large CSV files, performing data validation, and generating statistical summaries

# Bad
description: CSV tool
```

### 2. Comprehensive Documentation
Include:
- Clear usage examples
- Common use cases
- Limitations and requirements
- Error handling guidance

### 3. Minimal Tool Access
Only request tools you actually need:

```yaml
# Good: Specific tools
allowed-tools:
  - bash
  - read

# Less good: All tools
allowed-tools: []  # or omit field entirely
```

### 4. Well-Structured References
Break complex topics into separate reference files:
- One topic per file
- Clear hierarchy
- Practical examples

### 5. Robust Scripts
- Handle errors gracefully
- Provide clear error messages
- Support common options (--help, --json)
- Include usage documentation

### 6. Versioning
Use semantic versioning in metadata:
```yaml
metadata:
  version: "1.2.3"  # MAJOR.MINOR.PATCH
```

### 7. Testing
Test skills with:
- Valid and invalid inputs
- Edge cases
- Different configurations
- Script execution

## Validation

### Name Validation

```rust
use turboclaude_skills::validate_skill_name;

validate_skill_name("my-skill")?;     // ✓ Valid
validate_skill_name("my_skill")?;     // ✗ Error: underscores
validate_skill_name("MySkill")?;      // ✗ Error: uppercase
validate_skill_name("-skill")?;       // ✗ Error: leading hyphen
```

### Size Limits
- SKILL.md: Maximum 10 MB
- Reference files: Maximum 50 MB each
- No limit on script sizes

### Directory Matching
Directory name must match `name` field in SKILL.md:
```
skills/
└── git-helper/              # Directory name
    └── SKILL.md             # name: git-helper ✓
```

## Advanced Features

### Semantic Matching

The skill registry uses keyword matching to find relevant skills:

```rust
let matches = registry.find("analyze git branches").await?;
// Returns skills with "analyze", "git", or "branches" in name/description
```

### Script Execution with Timeout

```rust
use std::time::Duration;

let output = skill.execute_script(
    "analyze_branches",
    &["--days=60"],
    Some(Duration::from_secs(300))  // 5 minute timeout
).await?;

if output.timed_out {
    println!("Script exceeded timeout");
} else if output.success() {
    println!("Output: {}", output.stdout);
} else {
    eprintln!("Error: {}", output.stderr);
}
```

### Tool Access Control

Skills can restrict available tools:

```rust
// Check if skill allows a tool
if skill.metadata.allows_tool("bash") {
    // Execute bash command
}
```

### Lazy Loading

References and scripts are loaded on-demand:

```rust
// Loads SKILL.md only
let skill = Skill::from_file("skills/git-helper/SKILL.md").await?;

// References loaded when first accessed
let references = skill.references().await?;

// Scripts discovered when first needed
let scripts = skill.scripts().await?;
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/branch-cleanup.yml
name: Branch Cleanup Report
on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Analyze stale branches
        run: |
          python skills/git-helper/scripts/analyze_branches.py --days=60
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run commit message validation
if ! grep -qE "^(feat|fix|docs|style|refactor|test|chore)" "$1"; then
    echo "❌ Commit message doesn't follow conventional commits"
    echo "See: skills/git-helper/reference/commit-conventions.md"
    exit 1
fi
```

### Automation Script

```python
#!/usr/bin/env python3
# Monthly repository report

import subprocess
import datetime

# Generate reports
branches = subprocess.run(
    ['python', 'skills/git-helper/scripts/analyze_branches.py', '--json'],
    capture_output=True, text=True
)

stats = subprocess.run(
    ['bash', 'skills/git-helper/scripts/commit_stats.sh', '--json'],
    capture_output=True, text=True
)

# Send to monitoring system
print(f"Report generated: {datetime.date.today()}")
```

## Resources

- **Agent Skills Spec**: https://github.com/anthropics/agent-skills-spec
- **TurboClaude Docs**: https://github.com/epistates/turboclaude
- **Conventional Commits**: https://www.conventionalcommits.org/
- **Git Documentation**: https://git-scm.com/doc

## License

This example is provided under the MIT License. Individual skills may have different licenses specified in their SKILL.md files.

## Contributing

To add new example skills:

1. Create a new directory under `skills/`
2. Follow the structure shown in `git-helper/`
3. Include comprehensive documentation
4. Add tests if applicable
5. Update this README with a brief description

## Support

For issues or questions about the skills system:
- TurboClaude Issues: https://github.com/epistates/turboclaude/issues
- Skills Spec Issues: https://github.com/anthropics/agent-skills-spec/issues
