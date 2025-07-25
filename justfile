# just is a command runner, Justfile is very similar to Makefile, but simpler.

default:
  @just --list

# Release new version without publish
release version:
  cargo release --workspace --no-publish {{version}}
