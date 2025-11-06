# Commit Message Conventions

This reference document provides comprehensive guidelines for writing clear, consistent, and meaningful commit messages.

## Why Commit Conventions Matter

Good commit messages:
- Facilitate code review and collaboration
- Enable automated changelog generation
- Help future developers understand changes
- Support debugging and bisecting
- Improve project documentation

## Conventional Commits Specification

### Basic Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Examples

**Simple commit:**
```
feat: add user authentication
```

**With scope:**
```
fix(api): handle null response in user endpoint
```

**With body:**
```
feat(checkout): implement one-click payment

Add support for saved payment methods to enable
one-click checkout for returning customers.
```

**With breaking change:**
```
feat(api)!: restructure user authentication endpoints

BREAKING CHANGE: The /auth/login endpoint now requires
a POST request instead of GET. Update all clients accordingly.
```

## Commit Types

### Primary Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat: add dark mode toggle` |
| `fix` | Bug fix | `fix: resolve memory leak in image loader` |
| `docs` | Documentation only | `docs: update API reference` |
| `style` | Code style changes (formatting, semicolons, etc.) | `style: format with prettier` |
| `refactor` | Code refactoring without feature changes | `refactor: extract validation logic` |
| `perf` | Performance improvements | `perf: optimize database queries` |
| `test` | Adding or updating tests | `test: add integration tests for auth` |
| `build` | Build system or dependencies | `build: upgrade to webpack 5` |
| `ci` | CI/CD configuration | `ci: add automated deployment` |
| `chore` | Maintenance tasks | `chore: update dependencies` |
| `revert` | Revert previous commit | `revert: revert "feat: add dark mode"` |

### When to Use Each Type

**feat (feature):**
- Adding new user-facing functionality
- Implementing new API endpoints
- Creating new UI components
- Adding business logic

**fix (bug fix):**
- Correcting incorrect behavior
- Resolving errors or exceptions
- Fixing UI issues
- Patching security vulnerabilities

**docs (documentation):**
- README updates
- API documentation
- Code comments
- Architecture diagrams

**refactor:**
- Restructuring code without changing behavior
- Extracting functions or classes
- Renaming for clarity
- Removing code duplication

**perf (performance):**
- Optimizing algorithms
- Reducing memory usage
- Improving load times
- Database query optimization

**test:**
- Unit tests
- Integration tests
- End-to-end tests
- Test utilities

## Scopes

Scopes provide additional context about what part of the codebase changed.

### Common Scopes by Project Type

**Web Application:**
```
feat(auth): add OAuth2 support
fix(ui): correct navbar alignment
refactor(api): simplify error handling
```

**Library/Package:**
```
feat(parser): support new syntax
fix(validator): handle edge cases
docs(readme): add installation steps
```

**Monorepo:**
```
feat(web-app): add user dashboard
fix(mobile-app): resolve crash on startup
chore(shared-components): update dependencies
```

### Scope Guidelines

- Use lowercase
- Keep scopes consistent across the project
- Document scopes in CONTRIBUTING.md
- Limit scopes to 5-10 common categories
- Make scopes optional but encouraged

## Writing Descriptions

### Rules for Descriptions

1. **Use imperative mood**: "add" not "added" or "adds"
2. **No capitalization**: Start with lowercase
3. **No period at end**: Keep it concise
4. **Limit to 50-72 characters**: Be brief but descriptive
5. **Focus on what, not how**: The diff shows the how

### Good Examples

```
✓ add user profile page
✓ fix null pointer exception in parser
✓ update installation documentation
✓ remove deprecated API endpoints
✓ optimize image compression algorithm
```

### Bad Examples

```
✗ Added user profile page (past tense)
✗ Fix bug (too vague)
✗ Update (missing context)
✗ fixed a really weird bug that only happens sometimes when you... (too long)
✗ Fixed the null pointer exception by adding a check. (explains how, not what)
```

## Writing Body Content

The body should explain **what** and **why**, not **how**.

### Structure

```
<type>: <description>

[Detailed explanation of what changed and why]

[Additional context, links to issues, migration notes]
```

### Example with Body

```
refactor(database): migrate from MongoDB to PostgreSQL

The application now uses PostgreSQL instead of MongoDB for
better transaction support and complex query capabilities.

This change improves data consistency and enables future
features requiring ACID compliance.

Migration guide: docs/migration/mongodb-to-postgres.md
Closes #123
```

### When to Include a Body

**Include body when:**
- Change is non-obvious
- Breaking changes exist
- Migration steps needed
- Multiple files affected
- Complex business logic

**Skip body when:**
- Change is self-explanatory
- Simple typo fix
- Minor formatting change
- Documentation update

## Footer Conventions

### Breaking Changes

```
feat(api)!: change authentication flow

BREAKING CHANGE: JWT tokens now expire after 1 hour
instead of 24 hours. Clients must implement token refresh.
```

### Issue References

```
fix(login): prevent race condition

Closes #234
Fixes #567
Resolves #890
Related to #456
```

### Co-authors

```
feat: implement data export

Co-authored-by: Jane Doe <jane@example.com>
Co-authored-by: John Smith <john@example.com>
```

### Other Footers

```
Reviewed-by: Alice <alice@example.com>
Refs: #123, #456
Signed-off-by: Developer <dev@example.com>
```

## Atomic Commits

### Principles

1. **One logical change per commit**: Don't mix refactoring with features
2. **Complete changes**: Each commit should leave the codebase in a working state
3. **Related changes together**: Group related file changes in one commit

### Examples

**Good (atomic):**
```
Commit 1: feat(auth): add login form UI
Commit 2: feat(auth): implement login API endpoint
Commit 3: feat(auth): connect login form to API
Commit 4: test(auth): add login integration tests
```

**Bad (non-atomic):**
```
Commit 1: feat: add login, fix navbar bug, update README, refactor utils
```

## Advanced Patterns

### Fixup Commits (Interactive Rebase)

Create fixup commits during development:
```bash
git commit --fixup=abc123
```

Then squash before merging:
```bash
git rebase -i --autosquash main
```

### Conventional Commits in Merge Commits

```
Merge pull request #123 from feature/new-login

feat(auth): implement new login system
```

### Monorepo with Multiple Scopes

```
feat(packages/web,packages/mobile): add shared authentication
```

## Automation

### Commit Message Linting

Use `commitlint` to enforce conventions:

```javascript
// commitlint.config.js
module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [2, 'always', [
      'feat', 'fix', 'docs', 'style', 'refactor',
      'perf', 'test', 'build', 'ci', 'chore', 'revert'
    ]],
    'subject-case': [2, 'never', ['upper-case']],
    'subject-full-stop': [2, 'never', '.'],
    'header-max-length': [2, 'always', 72]
  }
};
```

### Git Hooks

Pre-commit hook to validate messages:

```bash
#!/bin/sh
# .git/hooks/commit-msg

MSG_FILE=$1
MSG=$(cat "$MSG_FILE")

PATTERN="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .{1,50}"

if ! echo "$MSG" | grep -qE "$PATTERN"; then
  echo "Invalid commit message format"
  echo "Format: <type>[optional scope]: <description>"
  exit 1
fi
```

### Changelog Generation

Generate changelogs automatically:

```bash
# Using conventional-changelog
npm install -g conventional-changelog-cli
conventional-changelog -p angular -i CHANGELOG.md -s
```

### Semantic Versioning

Automatically determine version bumps:
- `feat`: Minor version bump (0.X.0)
- `fix`: Patch version bump (0.0.X)
- `BREAKING CHANGE`: Major version bump (X.0.0)

## Team Adoption

### Onboarding Steps

1. **Document conventions** in CONTRIBUTING.md
2. **Set up commit linting** in CI/CD
3. **Create commit message template**:
   ```bash
   git config commit.template .gitmessage
   ```
4. **Provide examples** in documentation
5. **Review in pull requests**

### Commit Message Template

Create `.gitmessage`:
```
# <type>[optional scope]: <description>
# |<----  Using a Maximum Of 50 Characters  ---->|

# Explain why this change is being made
# |<----   Try To Limit Each Line to a Maximum Of 72 Characters   ---->|

# Provide links or keys to any relevant tickets, articles or other resources

# --- COMMIT END ---
# Type can be:
#   feat     (new feature)
#   fix      (bug fix)
#   refactor (refactoring code)
#   style    (formatting, missing semicolons, etc.)
#   doc      (changes to documentation)
#   test     (adding or refactoring tests)
#   chore    (updating build tasks, package manager configs, etc.)
# --------------------
# Remember to:
#   - Use imperative mood in the description
#   - Don't capitalize first letter
#   - Don't end the description with a period
```

## Tools and Resources

### Commit Message Helpers

- **commitizen**: Interactive commit message builder
- **commitlint**: Validate commit messages
- **husky**: Git hooks manager
- **conventional-changelog**: Generate changelogs

### Installation Example

```bash
npm install --save-dev @commitlint/cli @commitlint/config-conventional husky

# Enable Git hooks
npx husky install
npx husky add .husky/commit-msg 'npx --no -- commitlint --edit ${1}'
```

### VSCode Extensions

- Conventional Commits
- Git Commit Template
- GitLens

## Real-World Examples

### Linux Kernel Style

```
subsystem: short description

Longer explanation of the change, wrapping at 72 characters.
Include motivation for the change and contrast with previous behavior.

Signed-off-by: Developer Name <email@example.com>
```

### Angular Style

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Semantic Commits

```
type(scope): subject

body

BREAKING CHANGE: description
Closes #123
```

## Common Mistakes

### Mistake 1: Vague Messages
```
✗ fix bug
✓ fix(parser): handle empty string input
```

### Mistake 2: Too Much Detail in Subject
```
✗ fix: fixed the bug where users couldn't login because of session timeout
✓ fix(auth): prevent session timeout during login
```

### Mistake 3: Wrong Type
```
✗ feat: fix typo in documentation
✓ docs: fix typo in README
```

### Mistake 4: Multiple Changes
```
✗ feat: add login, refactor database, update README
✓ feat: add login form (separate commits for others)
```

### Mistake 5: Past Tense
```
✗ fixed authentication bug
✓ fix: resolve authentication issue
```

## Quick Reference Card

```
┌─────────────────────────────────────────────┐
│  COMMIT MESSAGE FORMAT                      │
├─────────────────────────────────────────────┤
│  <type>[scope]: <description>               │
│                                             │
│  [optional body]                            │
│                                             │
│  [optional footer]                          │
└─────────────────────────────────────────────┘

TYPES:
  feat, fix, docs, style, refactor, perf,
  test, build, ci, chore, revert

RULES:
  ✓ Imperative mood ("add" not "added")
  ✓ Lowercase first letter
  ✓ No period at end
  ✓ Max 50 chars in description
  ✓ Wrap body at 72 chars

EXAMPLES:
  feat(api): add user registration
  fix(ui): correct navbar alignment
  docs: update installation guide
```

## Conclusion

Consistent commit messages improve collaboration, enable automation, and create better project documentation. Start with basic conventional commits, then add scopes and body content as your team matures. Use tooling to enforce standards and make compliance easy.
