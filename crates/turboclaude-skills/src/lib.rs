#![deny(unsafe_code)]

//! # `TurboClaude` Skills
//!
//! Skills support for `TurboClaude` - modular, self-describing capability packages for Claude agents.
//!
//! ## Overview
//!
//! Skills are directories containing a `SKILL.md` file that provides:
//! - Metadata (name, description, license, etc.)
//! - Knowledge and instructions (Markdown body)
//! - Optional references (additional documentation)
//! - Optional scripts (Python/Bash utilities)
//! - Optional assets (templates, fonts, etc.)
//!
//! ## Quick Start
//!
//! ```no_run
//! use turboclaude_skills::{Skill, SkillRegistry};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load a single skill
//!     let skill = Skill::from_file("skills/my-skill/SKILL.md").await?;
//!     println!("Loaded: {} - {}", skill.metadata.name, skill.metadata.description);
//!
//!     // Or use a registry for discovery
//!     let mut registry = SkillRegistry::builder()
//!         .skill_dir(PathBuf::from("./skills"))
//!         .build()?;
//!
//!     registry.discover().await?;
//!     let skills = registry.list().await;
//!     println!("Discovered {} skills", skills.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Local Skills**: Load skills from filesystem directories
//! - **Discovery**: Automatic skill discovery via directory scanning
//! - **Validation**: Strict validation of SKILL.md format and metadata
//! - **Lazy Loading**: References and scripts loaded on-demand
//! - **Semantic Matching**: Find skills by description keywords
//! - **Agent Integration**: Easy integration with turboclaudeagent
//!
//! ## SKILL.md Format
//!
//! ```yaml
//! ---
//! name: skill-name
//! description: What the skill does and when to use it
//! license: MIT
//! allowed-tools:
//!   - bash
//!   - read
//! metadata:
//!   custom: value
//! ---
//!
//! # Markdown Body
//!
//! Instructions and documentation here...
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod error;
mod parser;
mod skill;
mod validation;

pub mod executor;
pub mod matcher;
pub mod registry;

// Re-exports
pub use error::{Result, SkillError};
pub use executor::{BashExecutor, CompositeExecutor, PythonExecutor, ScriptExecutor, ScriptOutput};
pub use matcher::{KeywordMatcher, SkillMatcher};
pub use registry::{SkillRegistry, SkillRegistryBuilder};
pub use skill::{Reference, Skill, SkillMetadata};

/// Prelude module for convenient imports
///
/// Commonly used types and traits
pub mod prelude {
    pub use crate::{
        KeywordMatcher, Result, Skill, SkillError, SkillMatcher, SkillMetadata, SkillRegistry,
        SkillRegistryBuilder,
    };
}
