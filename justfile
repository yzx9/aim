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
  cargo release --workspace --no-publish {{version}}

# Add a new migration to the database
migrate-add name:
  cd core && sqlx migrate add -r --source src/localdb/migrations {{name}}
