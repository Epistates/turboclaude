//! Semantic matching for skill discovery

use async_trait::async_trait;
use std::collections::HashSet;

use crate::error::Result;
use crate::skill::Skill;

/// Trait for matching skills to queries
#[async_trait]
pub trait SkillMatcher: Send + Sync {
    /// Find skills matching the given query
    ///
    /// Returns skills ranked by relevance.
    async fn find_matching(&self, skills: &[Skill], query: &str) -> Result<Vec<Skill>>;
}

/// Simple keyword-based matcher (MVP implementation)
///
/// Matches skills by counting keyword occurrences in descriptions.
/// Case-insensitive matching.
pub struct KeywordMatcher;

#[async_trait]
impl SkillMatcher for KeywordMatcher {
    async fn find_matching(&self, skills: &[Skill], query: &str) -> Result<Vec<Skill>> {
        // Tokenize query into keywords
        let query_lower = query.to_lowercase();
        let keywords: HashSet<_> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip very short words
            .collect();

        if keywords.is_empty() {
            return Ok(Vec::new());
        }

        // Score each skill by keyword matches
        let mut scored_skills: Vec<_> = skills
            .iter()
            .map(|skill| {
                let desc_lower = skill.metadata.description.to_lowercase();
                let name_lower = skill.metadata.name.to_lowercase();

                // Count keyword matches
                let desc_matches = keywords
                    .iter()
                    .filter(|kw| desc_lower.contains(**kw))
                    .count();

                let name_matches = keywords
                    .iter()
                    .filter(|kw| name_lower.contains(**kw))
                    .count();

                // Name matches worth more than description matches
                let score = (name_matches * 3) + desc_matches;

                (skill.clone(), score)
            })
            .filter(|(_, score)| *score > 0) // Only include matches
            .collect();

        // Sort by score (descending)
        scored_skills.sort_by(|a, b| b.1.cmp(&a.1));

        // Return sorted skills without scores
        Ok(scored_skills.into_iter().map(|(skill, _)| skill).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillMetadata;
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_skill(name: &str, description: &str) -> Skill {
        Skill {
            metadata: SkillMetadata {
                name: name.to_string(),
                description: description.to_string(),
                license: None,
                allowed_tools: None,
                metadata: HashMap::new(),
            },
            content: String::new(),
            root: PathBuf::from("/test"),
            references: OnceCell::default(),
            scripts: OnceCell::default(),
        }
    }

    #[tokio::test]
    async fn test_keyword_matcher_finds_matches() {
        let skills = vec![
            create_test_skill("pdf", "PDF processing and manipulation"),
            create_test_skill("slack-gif", "Create animated GIFs for Slack"),
            create_test_skill("mcp-builder", "Build MCP servers"),
        ];

        let matcher = KeywordMatcher;
        let results = matcher.find_matching(&skills, "PDF").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.name, "pdf");
    }

    #[tokio::test]
    async fn test_keyword_matcher_case_insensitive() {
        let skills = vec![
            create_test_skill("pdf", "PDF processing"),
            create_test_skill("slack-gif", "Create GIFs"),
        ];

        let matcher = KeywordMatcher;

        // Different cases should all match
        let results = matcher.find_matching(&skills, "pdf").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = matcher.find_matching(&skills, "PDF").await.unwrap();
        assert_eq!(results.len(), 1);

        let results = matcher.find_matching(&skills, "Pdf").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_keyword_matcher_multiple_keywords() {
        let skills = vec![
            create_test_skill("pdf", "PDF processing and manipulation"),
            create_test_skill("slack-gif", "Create animated GIFs for Slack"),
            create_test_skill("mcp-builder", "Build MCP servers"),
        ];

        let matcher = KeywordMatcher;
        let results = matcher.find_matching(&skills, "create GIF").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.name, "slack-gif");
    }

    #[tokio::test]
    async fn test_keyword_matcher_ranking() {
        let skills = vec![
            create_test_skill("skill1", "This skill uses GIF processing"),
            create_test_skill("skill2", "Create GIF animations with amazing tool"),
            create_test_skill("skill3", "Unrelated skill"),
        ];

        let matcher = KeywordMatcher;
        let results = matcher
            .find_matching(&skills, "GIF animations")
            .await
            .unwrap();

        // Should return 2 skills (skill1 and skill2)
        assert_eq!(results.len(), 2);

        // skill2 should rank higher (matches both "GIF" and "animations")
        assert_eq!(results[0].metadata.name, "skill2");
        assert_eq!(results[1].metadata.name, "skill1");
    }

    #[tokio::test]
    async fn test_keyword_matcher_name_matches_rank_higher() {
        let skills = vec![
            create_test_skill("pdf-tool", "Manipulate documents"),
            create_test_skill("document-tool", "Process PDF files extensively"),
        ];

        let matcher = KeywordMatcher;
        let results = matcher.find_matching(&skills, "PDF").await.unwrap();

        // pdf-tool should rank higher because "pdf" is in the name
        assert_eq!(results[0].metadata.name, "pdf-tool");
    }

    #[tokio::test]
    async fn test_keyword_matcher_no_matches() {
        let skills = vec![
            create_test_skill("pdf", "PDF processing"),
            create_test_skill("slack-gif", "Create GIFs"),
        ];

        let matcher = KeywordMatcher;
        let results = matcher
            .find_matching(&skills, "unrelated query")
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_keyword_matcher_empty_query() {
        let skills = vec![create_test_skill("pdf", "PDF processing")];

        let matcher = KeywordMatcher;
        let results = matcher.find_matching(&skills, "").await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_keyword_matcher_short_words_filtered() {
        let skills = vec![create_test_skill("pdf", "PDF processing tool")];

        let matcher = KeywordMatcher;

        // Short words (≤2 chars) should be filtered
        let results = matcher.find_matching(&skills, "a to is").await.unwrap();
        assert_eq!(results.len(), 0);

        // Longer words should match
        let results = matcher.find_matching(&skills, "PDF").await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
