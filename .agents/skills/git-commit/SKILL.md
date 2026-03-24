---
name: git-commit
description: Create a git commit with proper message
---

# Git Commit

## Context

- Current git status: !`git status`
- Current git diff: !`git diff HEAD`
- Recent commits: !`git log --oneline -5`

## Task

Based on the changes above, run the necessary check steps, including formatting and linting.
Then stage the changes and create a concise, descriptive git commit message following the
Gitmoji convention.

## Pre-Commit Checklist

Before committing, always run:

1. `cargo fmt` - Format code
2. `just lint` - Run linter
3. For user-facing changes (new features, bug fixes, breaking changes), update the changelog
   using the `/changelog` skill first

## Commit Message Format

Follow the [Gitmoji](https://gitmoji.dev/) convention:

```
<gitmoji> (<scope>): <description>
```

Examples:

```
✨ (core): Add event recurrence support
🐛 (cli): Fix crash when parsing invalid dates
📝 (docs): Update installation instructions
♻️ (ical): Refactor parser for better error messages
```

Common gitmojis:

| Gitmoji | Meaning            |
| ------- | ------------------ |
| ✨      | New feature        |
| 🐛      | Bug fix            |
| 📝      | Documentation      |
| ♻️      | Refactor           |
| ✅      | Tests              |
| 🔧      | Configuration      |
| ⬆️      | Dependency upgrade |

## Notes

- Check unstaged changes. If there are no staged changes, or if the unstaged changes are
  only minor formatting or comment fixes, stage them. Otherwise, do not modify the current
  staged changes and proceed to the next step.
- Analyze the changes to determine the appropriate commit type and scope.
- In sandboxed environments, avoid heredocs when possible and use alternatives like printf
  or direct string expansion that don't require file creation.
