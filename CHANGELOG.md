# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `aim done` support multiple todos now
- sqlite database is persistent now

### Changed

- `short_id` has been move into sqlite, all events or todos will be assign a new id

### Deprecated

- `aim undo` is a deprecated shortcut of `aim todo undo` now, will be remove in v0.4.0

## [v0.2.3] - 2025-07-30

### Added

- Add shortcuts `new`/`edit` to launch the TUI editor
- Add output format argument for `todo new` command
- Add a config option for placing no-priority todos at the top

### Changed

- Column title has been renamed form "Display Number" to "ID"
- Move `default_due` config to core
- Move `default_priority` config to core
- Move `short_id` to core

## [v0.2.2] - 2025-07-28

### Added

- Add TUI switch for status and priority
- Add default due config entry
- Support numbered priority config
- Allow set percent_complete and status for new todo
- Add TUI todo creator support

### Changed

- Set default status for todo if not available
- TodoDraft no longer contains a uid. Instead, an ID will be generated when it's added.

## [v0.2.1] - 2025-07-27

### Added

- Old events are now filtered when listing
- Display a hint when nothing is found during listing.
- Enable colored output for clap
- Add default priority configuration
- Add TUI-based todo editor
- Colorize event time range

### Fixed

- Always set status for new todo

### Removed

- Remove intuitive priority alias

## [v0.2.0] - 2025-07-25

### Added

- Add command to create a new todo
- Add command to eidt an existing todo

### Changed

- Organize subcommands

### Fixed

- Append app name to state directory

## [v0.1.2] - 2025-07-23

### Added

- Add done/undo command
- Add output format options
- Handle no priority sorting

### Changed

- Hide completion generation command in help message

### Fixed

- Format single row data correctly

## [v0.1.1] - 2025-07-20

### Added

- Add short id
- Add json output format
- Generate shell completion

### Changed

- Improve help message of command line interface
- Remove uid from default output columns for simplification

### Fixed

- Format empty table correctly

## [v0.1.0] - 2025-07-19

🎉 Init project

### Added

- Add events and todos command
- Add dashboard command

[unreleased]: https://github.com/yzx9/aim/compare/v0.2.3...HEAD
[v0.2.3]: https://github.com/yzx9/aim/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/yzx9/aim/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/yzx9/aim/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/yzx9/aim/compare/v0.1.2...v0.2.0
[v0.1.2]: https://github.com/yzx9/aim/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/yzx9/aim/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/yzx9/aim/releases/tag/v0.1.0
