---
name: git-helper
description: Advanced Git repository helper that provides branch analysis, commit statistics, and automated workflow suggestions. Use this skill when you need to analyze repository state, generate insights about commit patterns, or get recommendations for Git workflows.
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
    - repository-analysis
  requirements:
    - git >= 2.0.0
  homepage: https://github.com/example/turboclaude
---

# Git Helper Skill

This skill provides advanced Git repository analysis and workflow automation capabilities. It helps developers understand their repository state, analyze commit patterns, and receive intelligent workflow suggestions.

## Capabilities

### 1. Branch Analysis
- Identify stale branches (no commits in 30+ days)
- Find branches ahead/behind main
- Detect unmerged branches
- Calculate branch divergence metrics

### 2. Commit Statistics
- Analyze commit frequency by author
- Generate commit message quality reports
- Identify commit patterns (time of day, day of week)
- Calculate code churn metrics

### 3. Repository Health
- Detect large files that should be in .gitignore
- Identify uncommitted changes
- Check for merge conflicts
- Analyze repository size and growth

### 4. Workflow Suggestions
- Recommend when to merge or rebase
- Suggest branch cleanup opportunities
- Identify technical debt patterns
- Recommend commit message improvements

## Usage

### Basic Branch Analysis

Ask Claude to analyze your repository branches:
```
"Use the git-helper skill to analyze stale branches in this repository"
```

Claude will execute the analysis scripts and provide:
- List of branches not updated in 30+ days
- Recommendations for branch cleanup
- Potential merge candidates

### Commit Statistics

Request commit pattern analysis:
```
"Use git-helper to show commit statistics for the last 6 months"
```

You'll receive:
- Commits per author with percentages
- Commit frequency over time
- Average commits per week
- Most active development periods

### Repository Health Check

Run a comprehensive health check:
```
"Use git-helper to perform a repository health check"
```

This provides:
- Uncommitted changes summary
- Large file detection
- .gitignore recommendations
- Repository size metrics

## Script Reference

This skill includes two utility scripts:

### analyze_branches.py
Python script that analyzes all branches in the repository and identifies:
- Stale branches (configurable threshold, default 30 days)
- Branch commit counts
- Last commit information

**Usage:**
```bash
python analyze_branches.py [--days=30]
```

### commit_stats.sh
Bash script that generates commit statistics including:
- Commits per author
- Timeline analysis
- Message quality metrics

**Usage:**
```bash
bash commit_stats.sh [--since="6 months ago"]
```

## Best Practices

1. **Regular Health Checks**: Run repository health checks weekly to maintain code quality
2. **Branch Hygiene**: Use branch analysis monthly to clean up stale branches
3. **Team Insights**: Share commit statistics during team retrospectives
4. **Pre-Release**: Run comprehensive analysis before major releases

## Advanced Features

### Custom Thresholds
All scripts support custom thresholds for stale branch detection, commit frequency analysis, and file size warnings.

### CI/CD Integration
Scripts can be integrated into CI/CD pipelines to enforce repository hygiene policies.

### Team Metrics
Generate team-wide metrics across multiple repositories for leadership visibility.

## Reference Documentation

For detailed information about specific features, see:
- `reference/branch-strategies.md` - Git branching strategy recommendations
- `reference/commit-conventions.md` - Commit message best practices

## Limitations

- Requires Git 2.0.0 or higher
- Analysis performance degrades with repositories over 10GB
- Remote branch analysis requires network access
- Some metrics require full repository history (shallow clones not supported)

## Examples

### Example 1: Find Branches to Clean Up
```
"Show me all branches that haven't been updated in 60 days"
```

### Example 2: Author Contribution Analysis
```
"Analyze commit contributions by author for the last year"
```

### Example 3: Pre-Merge Health Check
```
"Check if it's safe to merge feature-xyz into main"
```

## Error Handling

The skill gracefully handles common scenarios:
- Not a Git repository: Returns clear error message
- No remote configured: Provides local-only analysis
- Permission errors: Suggests resolution steps
- Network issues: Falls back to local branch analysis

## Security Considerations

- Scripts never push changes without explicit confirmation
- No credential storage or handling
- Read-only operations by default
- All destructive operations require user confirmation
