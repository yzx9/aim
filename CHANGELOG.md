# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- cli: When the end timestamp is date-only and its time precedes the start, adjust it to the
  following day.

## [v0.5.0] - 2025-08-15

### Added

- cli: Add event support for command `edit`
- cli: Add command `todo cancel` to cancel a todo
- cli: When only time is provided, the end time defaults to the same date as the start time
- cli: Add TUI editor for draft event and todo with tab
- core: Add api for get the kind of the given id, which can be either an event or a todo

### Changed

- cli: The TUI form no longer supports wrapping navigation

### Deprecated

- Remove the `'HH:MM'` format for the default `due` configuration

## [v0.4.0] - 2025-08-09

### Added

- Add event TUI editor, `event new` and `event edit` now work without arguments to open the TUI
- `Percent complete` now accepts only numeric input in todo editor

### Changed

- Status is now required for drafts since it has a default value

### Removed

- Command `undo` has been removed, please use `todo undo` instead

### Fixed

- `Percent complete` should be range from 0 to 100
- Fix CJK font input panic in the TUI editor

## [v0.3.2] - 2025-08-06

### Added

- Add command `event new` and `event edit`

### Fixed

- Handle Unicode boundary in substring width calculation
- Fix todo color

## [v0.3.1] - 2025-08-04

### Added

- Add alias `add` for command `new`

### Changed

- Moved all config entries to the `[core]` sub-table

### Deprecated

- The `'HH:MM'` format for the default `due` configuration is now deprecated and will be removed
  in v0.5.0

## [v0.3.0] - 2025-08-02

### Added

- Command `done` support multiple todos now
- sqlite database is persistent now

### Changed

- `short_id` has been move into sqlite, all events or todos will be assign a new id

### Deprecated

- Command `undo` is a deprecated shortcut of `todo undo` now, will be remove in v0.4.0

## [v0.2.3] - 2025-07-30

### Added

- Add shortcut commands `new`/`edit` to launch the TUI editor
- Add output format argument for command `todo new`
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
- TodoDraft no longer contains a uid. Instead, an ID will be generated when it's added

## [v0.2.1] - 2025-07-27

### Added

- Old events are now filtered when listing
- Display a hint when nothing is found during listing
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

[unreleased]: https://github.com/yzx9/aim/compare/v0.5.0...HEAD
[v0.5.0]: https://github.com/yzx9/aim/compare/v0.4.0...v0.5.0
[v0.4.0]: https://github.com/yzx9/aim/compare/v0.3.2...v0.4.0
[v0.3.2]: https://github.com/yzx9/aim/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/yzx9/aim/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/yzx9/aim/compare/v0.2.3...v0.3.0
[v0.2.3]: https://github.com/yzx9/aim/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/yzx9/aim/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/yzx9/aim/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/yzx9/aim/compare/v0.1.2...v0.2.0
[v0.1.2]: https://github.com/yzx9/aim/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/yzx9/aim/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/yzx9/aim/releases/tag/v0.1.0
