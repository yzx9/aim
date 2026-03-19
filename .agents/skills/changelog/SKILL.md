---
name: changelog
description: Update CHANGELOG.md with new entry
disable-model-invocation: true
---

# Changelog Update

Update `CHANGELOG.md` using the repository's existing format.

## Read First

1. Inspect the current `CHANGELOG.md` before editing.
2. Preserve the existing structure and wording style instead of normalizing old entries.
3. Only edit the changelog sections needed for the user's request.

## Repository-Specific Rules

- The file follows Keep a Changelog structure:
  - top intro
  - `## [Unreleased]`
  - subsection headings such as `### Added`, `### Changed`, `### Deprecated`, `### Removed`, `### Fixed`
  - released versions in reverse chronological order
  - compare links at the bottom
- New unreleased items belong under `## [Unreleased]`.
- Keep subsection order consistent with the current file. If a requested section does not exist under
  `Unreleased`, create it in the standard order:
  `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`.
- Entries are bullet points and usually use the form `<area>: <message>`, for example
  `- cli: Fix ...` or `- core: Add ...`.
- Wrap long bullets onto the next line using the existing indentation style.
- Do not rewrite older entries just to make them stylistically consistent.

## Versioning Conventions In This Repo

- Recent release headings use no `v` prefix, for example `## [0.11.0] - 2026-01-18`.
- Bottom compare-link labels use a `v` prefix, for example `[v0.11.0]: ...`.
- `Unreleased` compare link should point from the latest released tag to `HEAD`.
- When cutting a release, keep this mixed convention unless the user explicitly asks to normalize it.

## Common Tasks

### Add an unreleased entry

1. Parse the change type and message from the user request.
2. Open `CHANGELOG.md` and find `## [Unreleased]`.
3. Insert the bullet into the matching subsection.
4. If the subsection does not exist, create it in the correct order.
5. Keep the newest unreleased bullets near related items; do not reorder the whole file.

### Cut a new release

When the user asks to release a version:

1. Check the current unreleased changes before proposing a version:
   - inspect `## [Unreleased]` in `CHANGELOG.md`
   - inspect the current workspace version in `Cargo.toml`
2. Suggest a semantic version bump to the user before running any release command:
   - prefer `major` only for clearly breaking changes
   - prefer `minor` when `Added` contains new user-facing capabilities
   - prefer `patch` for fixes, small improvements, or documentation-only releases
3. Ask the user to confirm the exact version to release. Do not run the release command until the
   user confirms the version.
4. After confirmation, do not edit `CHANGELOG.md` manually if the repository release automation
   already covers it.
5. In this repository, use the `justfile` recipe:
   `just release <version>`
6. Treat `just release <version>` as the source of truth for release-cutting behavior. It updates
   `CHANGELOG.md`, updates crate versions, refreshes `Cargo.lock`, creates the release commit, and
   creates the git tag.
7. Only fall back to manual changelog editing if the user explicitly asks not to use the script or
   the script is unavailable/broken.
8. If falling back manually, preserve the existing heading and compare-link conventions.

## Editing Constraints

- Use concise, release-note style phrasing.
- Avoid duplicate bullets for the same change.
- Do not infer a release date other than today's date unless the user specifies otherwise.
- If the request is ambiguous about section placement, infer the best fit from the existing style.
