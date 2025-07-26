# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

# Run tests for the project
test:
  cargo test --all-features

# Run clippy to check for linting issues
lint:
  cargo clippy --all-targets --all-features -- -D warnings

# Release new version without publish
release version:
  cargo release --workspace --no-publish {{version}}
