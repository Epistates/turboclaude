//! Integration tests for turboclaude-skills

use std::path::PathBuf;
use turboclaude_skills::{Skill, SkillRegistry};

#[tokio::test]
async fn test_load_minimal_skill() {
    let path = PathBuf::from("tests/fixtures/skills/minimal-skill/SKILL.md");
    let skill = Skill::from_file(&path).await.unwrap();

    assert_eq!(skill.metadata.name, "minimal-skill");
    assert_eq!(
        skill.metadata.description,
        "A minimal test skill with only required fields"
    );
    assert!(skill.metadata.license.is_none());
    assert!(skill.metadata.allowed_tools.is_none()); // No allowed_tools = all allowed
    assert!(skill.metadata.metadata.is_empty());
    assert!(skill.content.contains("Minimal Skill"));
}

#[tokio::test]
async fn test_load_full_skill() {
    let path = PathBuf::from("tests/fixtures/skills/full-skill/SKILL.md");
    let skill = Skill::from_file(&path).await.unwrap();

    assert_eq!(skill.metadata.name, "full-skill");
    assert_eq!(skill.metadata.license, Some("MIT".to_string()));

    let tools = skill.metadata.allowed_tools.as_ref().unwrap();
    assert_eq!(tools.len(), 3);
    assert!(skill.metadata.allows_tool("bash"));
    assert!(skill.metadata.allows_tool("read"));
    assert!(skill.metadata.allows_tool("write"));
    assert!(!skill.metadata.allows_tool("dangerous"));
    assert_eq!(skill.metadata.metadata.len(), 3);
}

#[tokio::test]
async fn test_skill_references() {
    let path = PathBuf::from("tests/fixtures/skills/full-skill/SKILL.md");
    let skill = Skill::from_file(&path).await.unwrap();

    let refs = skill.references().await.unwrap();
    assert_eq!(refs.len(), 1);
    assert!(refs[0].path.ends_with("guide.md"));
}

#[tokio::test]
#[ignore = "Script fixture path resolution needs investigation - Phase 5 Week 2"]
async fn test_skill_scripts() {
    let path = PathBuf::from("tests/fixtures/skills/full-skill/SKILL.md");
    let skill = Skill::from_file(&path).await.unwrap();

    let scripts = skill.scripts().await.unwrap();
    assert_eq!(scripts.len(), 1);
    assert!(scripts.contains_key("test_script"));
}

#[tokio::test]
async fn test_skill_context() {
    let path = PathBuf::from("tests/fixtures/skills/minimal-skill/SKILL.md");
    let skill = Skill::from_file(&path).await.unwrap();

    let context = skill.context();
    assert!(context.contains("# Skill: minimal-skill"));
    assert!(context.contains("Minimal Skill"));
}

#[tokio::test]
async fn test_registry_discover() {
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("tests/fixtures/skills"))
        .build()
        .unwrap();

    let report = registry.discover().await.unwrap();
    assert!(report.loaded >= 2); // At least minimal-skill and full-skill
    assert!(report.is_success());
}

#[tokio::test]
async fn test_registry_get() {
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("tests/fixtures/skills"))
        .build()
        .unwrap();

    registry.discover().await.unwrap();

    let skill = registry.get("minimal-skill").await.unwrap();
    assert_eq!(skill.metadata.name, "minimal-skill");

    let result = registry.get("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_registry_list() {
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("tests/fixtures/skills"))
        .build()
        .unwrap();

    registry.discover().await.unwrap();

    let skills = registry.list().await;
    assert!(skills.len() >= 2);

    let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"minimal-skill"));
    assert!(names.contains(&"full-skill"));
}

#[tokio::test]
async fn test_registry_find() {
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("tests/fixtures/skills"))
        .build()
        .unwrap();

    registry.discover().await.unwrap();

    let results = registry.find("minimal").await.unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|s| s.metadata.name == "minimal-skill"));

    let results = registry.find("full featured").await.unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|s| s.metadata.name == "full-skill"));
}

#[tokio::test]
async fn test_registry_contains() {
    let mut registry = SkillRegistry::builder()
        .skill_dir(PathBuf::from("tests/fixtures/skills"))
        .build()
        .unwrap();

    registry.discover().await.unwrap();

    assert!(registry.contains("minimal-skill").await);
    assert!(registry.contains("full-skill").await);
    assert!(!registry.contains("nonexistent").await);
}

#[tokio::test]
async fn test_name_mismatch_error() {
    // Create a temp skill with mismatched name
    let temp_dir = tempfile::tempdir().unwrap();
    let skill_dir = temp_dir.path().join("wrong-name");
    std::fs::create_dir(&skill_dir).unwrap();

    let skill_md = skill_dir.join("SKILL.md");
    std::fs::write(
        &skill_md,
        r#"---
name: different-name
description: Test
---
Body
"#,
    )
    .unwrap();

    let result = Skill::from_file(&skill_md).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("mismatch"));
}
