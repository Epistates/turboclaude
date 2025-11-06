# Git Branch Strategies

This reference document provides detailed information about effective Git branching strategies for modern development teams.

## Overview

Choosing the right branching strategy is crucial for team productivity, code quality, and release management. This guide covers popular strategies and when to use them.

## Common Branching Strategies

### 1. Git Flow

**Best for:** Projects with scheduled releases and multiple versions in production.

**Structure:**
- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - Individual feature branches
- `release/*` - Release preparation branches
- `hotfix/*` - Emergency fixes for production

**Workflow:**
```
feature/new-login → develop → release/v1.2.0 → main
                                              ↓
                                           tag v1.2.0
```

**Advantages:**
- Clear separation of concerns
- Supports multiple production versions
- Well-defined release process

**Disadvantages:**
- Complex for small teams
- Can slow down deployment velocity
- Requires discipline to maintain

### 2. GitHub Flow

**Best for:** Continuous deployment environments and web applications.

**Structure:**
- `main` - Always deployable
- `feature/*` - Short-lived feature branches

**Workflow:**
```
feature/user-profile → main (+ deploy)
```

**Advantages:**
- Simple and easy to understand
- Encourages continuous deployment
- Minimal merge conflicts

**Disadvantages:**
- Less suitable for versioned releases
- Requires robust CI/CD
- Main must always be stable

### 3. Trunk-Based Development

**Best for:** High-velocity teams with strong testing practices.

**Structure:**
- `main` (trunk) - Primary development branch
- Short-lived feature branches (< 1 day)

**Workflow:**
```
feature-123 → main (deploy via feature flags)
```

**Advantages:**
- Fastest integration cycle
- Reduces merge complexity
- Encourages small, incremental changes

**Disadvantages:**
- Requires excellent test coverage
- Needs feature flag infrastructure
- Higher risk without proper safeguards

### 4. GitLab Flow

**Best for:** Teams using GitLab with multiple environments.

**Structure:**
- `main` - Production environment
- `staging` - Staging environment
- `feature/*` - Feature branches

**Workflow:**
```
feature/api-v2 → main → staging → production
```

**Advantages:**
- Environment-based workflow
- Clear deployment path
- Works well with GitLab CI/CD

**Disadvantages:**
- Environment branch management overhead
- Can delay production deployment
- Requires careful merge order

## Branch Lifecycle Management

### Creating Branches

**Naming Conventions:**
```
feature/description      - New features
bugfix/issue-number     - Bug fixes
hotfix/critical-fix     - Emergency production fixes
release/version         - Release preparation
experiment/idea         - Experimental work
```

**Best Practices:**
- Use descriptive names (avoid `temp`, `test`, `fix`)
- Include issue/ticket numbers when applicable
- Keep names concise but meaningful
- Use lowercase with hyphens

### Merging Strategies

**1. Merge Commit (Default)**
```bash
git merge feature/login --no-ff
```
- Preserves complete history
- Shows feature branch existence
- Creates merge commits

**2. Squash and Merge**
```bash
git merge feature/login --squash
```
- Combines all commits into one
- Cleaner history on main branch
- Loses individual commit context

**3. Rebase and Merge**
```bash
git rebase main
git checkout main
git merge feature/login --ff-only
```
- Linear history
- No merge commits
- Can rewrite public history

### Branch Protection

**Recommended Settings:**
- Require pull request reviews (minimum 1-2)
- Require status checks to pass
- Require branches to be up to date
- No force pushes to protected branches
- Require signed commits (optional)

### Stale Branch Management

**Identify Stale Branches:**
```bash
# Find branches not updated in 30 days
for branch in $(git branch -r --merged); do
  if [ -z "$(git log -1 --since='30 days ago' -s $branch)" ]; then
    echo "$branch is stale"
  fi
done
```

**Cleanup Policy:**
- Delete branches merged to main after 7 days
- Archive long-lived feature branches
- Document branch deletion process
- Notify owners before deletion

## Advanced Patterns

### Feature Flags

Enable trunk-based development without incomplete features in production:

```python
if feature_flag_enabled('new_checkout'):
    new_checkout_flow()
else:
    legacy_checkout_flow()
```

### Branch Dependencies

When feature B depends on feature A:
```
feature/A → main
feature/B (branched from feature/A) → feature/A → main
```

### Emergency Hotfixes

Critical production fixes:
```
hotfix/security-patch → main → immediate deploy
                      → backport to release branches
```

## Decision Matrix

| Criterion | Git Flow | GitHub Flow | Trunk-Based | GitLab Flow |
|-----------|----------|-------------|-------------|-------------|
| Team Size | Large | Any | Small-Medium | Any |
| Release Frequency | Scheduled | Continuous | Continuous | Scheduled/Continuous |
| Environment Complexity | High | Low | Low | High |
| Testing Maturity | Medium | High | Very High | Medium-High |
| Learning Curve | Steep | Easy | Medium | Medium |

## Migration Strategies

### From Git Flow to GitHub Flow

1. Merge all feature branches to develop
2. Merge develop to main
3. Delete develop branch
4. Train team on new workflow
5. Update CI/CD pipelines

### From Long-Lived to Trunk-Based

1. Implement feature flags
2. Improve test coverage to >80%
3. Set up continuous integration
4. Reduce branch lifetime gradually
5. Enforce branch age limits

## Monitoring and Metrics

**Key Metrics to Track:**
- Average branch lifetime
- Number of active branches
- Merge conflict frequency
- Time from branch creation to deployment
- Branch abandonment rate

**Health Indicators:**
- Branch lifetime < 3 days (trunk-based) or < 2 weeks (git flow)
- < 5 long-lived branches at any time
- Merge conflict rate < 5%
- 90%+ branches eventually merged

## Common Pitfalls

### 1. Long-Lived Feature Branches
**Problem:** Leads to merge hell and integration issues
**Solution:** Break features into smaller incremental changes

### 2. Inconsistent Naming
**Problem:** Difficult to understand branch purpose
**Solution:** Enforce naming conventions via hooks

### 3. Abandoned Branches
**Problem:** Repository clutter and confusion
**Solution:** Automated stale branch detection and cleanup

### 4. Merge Conflicts
**Problem:** Frequent conflicts slow development
**Solution:** Merge main into feature branches daily

### 5. No Branch Protection
**Problem:** Accidental force pushes or direct commits
**Solution:** Enable branch protection rules

## Conclusion

The best branching strategy depends on your team's specific needs, deployment frequency, and tooling. Start simple (GitHub Flow) and add complexity (Git Flow) only when necessary. Regardless of strategy, maintain branch hygiene through regular cleanup and clear naming conventions.

## Further Reading

- [Git Branching Model](https://nvie.com/posts/a-successful-git-branching-model/) - Original Git Flow
- [GitHub Flow](https://guides.github.com/introduction/flow/) - Official guide
- [Trunk Based Development](https://trunkbaseddevelopment.com/) - Comprehensive resource
- [GitLab Flow](https://docs.gitlab.com/ee/topics/gitlab_flow.html) - Environment-based approach
