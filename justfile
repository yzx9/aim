# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

# Run tests for the project
test:
  cargo test --workspace --all-features

# Run clippy to check for linting issues
lint:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

# Release new version without publish
release version:
  sed -i -E 's/version = "[0-9]*.[0-9]*.[0-9]*"/version = "{{version}}"/' ./Cargo.toml
  cargo update -p aimcal -p aimcal-cli -p aimcal-core
  git add ./Cargo.toml ./Cargo.lock
  git commit -m "🔖 Release v{{version}}"
  git tag -a "v{{version}}" -m "🔖 Release v{{version}}"
  echo "Please check and run 'git push origin v{{version}}' to push the tag to trigger CI/CD."

# Add a new migration to the database
migrate-add name:
  cd core && sqlx migrate add -r --source src/localdb/migrations {{name}}
