# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

# Run tests for the project
test:
  cargo test --workspace --all-features

# Run ignored tests for the project
test-ignored:
  cargo test --workspace --all-features -- --ignored

# Run clippy to check for linting issues
lint:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
  cargo fmt

# Release new version without publish
release version:
  # Update CHANGELOG.md
  sed -i -E "s/## \[Unreleased\]/## [Unreleased]\n\n## [{{version}}] - $(date '+%F')/" ./CHANGELOG.md
  URL_CMP="https:\/\/github.com\/yzx9\/aim\/compare" && \
      sed -i -E "s/\[Unreleased\]: ${URL_CMP}\/v([0-9]*.[0-9]*.[0-9]*)...HEAD/[Unreleased\]: ${URL_CMP}\/v{{version}}...HEAD\n[v{{version}}]: ${URL_CMP}\/v\1...v{{version}}/" ./CHANGELOG.md

  # Update version in Cargo.toml
  sed -i -E 's/version = "[0-9]*.[0-9]*.[0-9]*"/version = "{{version}}"/' ./Cargo.toml
  cargo update -p aimcal -p aimcal-cli -p aimcal-core -p aimcal-ical

  # Commit changes and create git tag
  git add CHANGELOG.md ./Cargo.toml ./Cargo.lock
  git commit -m "🔖 Release v{{version}}"
  git tag -a "v{{version}}" -m "🔖 Release v{{version}}"
  echo "Please check and run 'git push origin v{{version}}' to push the tag to trigger CI/CD."

# Add a new migration to the database
migrate-add name:
  cd core && sqlx migrate add -r --source src/db/migrations {{name}}

# Initialize development calendar with example files
init-dev:
  #!/usr/bin/env bash
  set -euxo pipefail
  if [ -d .dev ]; then
    read -r -p ".dev already exists and will be deleted before re-initializing. Continue? [y/N] " confirm
    case "$confirm" in
      [yY]|[yY][eE][sS]) ;;
      *)
        echo "Initialization cancelled"
        exit 0
        ;;
    esac
    rm -rf .dev
  fi
  mkdir -p .dev/calendar
  cp examples/*.ics .dev/calendar/
  touch .dev/calendar/.dev-marker
  echo "Copied $(ls examples/*.ics 2>/dev/null | wc -l) example files to .dev/calendar/"
  echo "Dev database will be initialized on first 'aim' run"

# Create a git worktree under .worktree/<name> with a new branch
add-worktree name: && init-dev
  git worktree add -b {{name}} .worktree/{{name}}

  # Activate direnv for the new worktree if direnv is installed and current directory is allowed
  # `foundRC.allowed`: 0 -> allowed, 2 -> denied
  @if command -v direnv >/dev/null 2>&1; then \
    if direnv status --json \
      | jq -e --arg p "$(pwd -P)/.envrc" '.state.foundRC.path == $p and .state.foundRC.allowed == 0' >/dev/null; \
    then \
      echo "==> current direnv is allowed, propagating to worktree"; \
      direnv allow .worktree/{{name}}; \
    else \
      echo "==> current direnv not allowed (or no .envrc), skip"; \
    fi; \
  fi
