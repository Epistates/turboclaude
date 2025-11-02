# TurboClaude Skills - Dynamic Skill System

Flexible skill framework for registering, managing, and executing dynamic skills in Claude agents.

## Features

- **Skill Registry**: Register and manage skills
- **Skill Matching**: Intelligent skill matching based on context
- **Skill Execution**: Safe, sandboxed skill execution
- **Validation**: Comprehensive skill validation
- **Parser**: Parse skills from SKILL.md files

## Quick Start

```rust
use turboclaude_skills::{SkillRegistry, Skill};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = SkillRegistry::new();

    // Register a skill
    let skill = Skill::new("calculate", "Perform calculations");
    registry.register(skill)?;

    // Find matching skills
    let matches = registry.find("perform math")?;
    println!("Found {} skills", matches.len());

    Ok(())
}
```

## Skill Format

Skills are defined in `SKILL.md` files with structure:

```markdown
# Skill: calculate

Calculate mathematical expressions.

## Input
- expression: string

## Output
- result: number

## Implementation
...
```

## Components

### SkillRegistry
Central skill management and discovery.

### SkillExecutor
Safe skill execution with timeout and resource controls.

### SkillMatcher
Intelligent matching of skills to agent needs.

### SkillValidator
Comprehensive skill validation before registration.

## Architecture

```
turboclaude-skills
├── executor    (Skill execution engine)
├── registry    (Skill registry)
├── matcher     (Skill matching logic)
├── parser      (SKILL.md parser)
└── validation  (Skill validation)
```

## Examples

See `examples/` for complete skill demonstrations.

## Testing

```bash
cargo test
```

## Documentation

Full API docs: `cargo doc --open`

---

**Part of TurboClaude** 🎯
