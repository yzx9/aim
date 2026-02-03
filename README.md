<div align="center" id="madewithlua">
  <img src="./aim.svg" width="150" height="130" />
</div>

<h1 align="center">AIM</h1>
<h3 align="center">Analyze. Interact. Manage Your Time, with calendar support</h3>

<p align="center">
  <a href="https://github.com/yzx9/aim"
    ><img
      title="Repo: github.com/yzx9/aim"
      src="https://img.shields.io/badge/AIM-8b36db?style=for-the-badge&logo=GitHub"
  /></a>
  <a href="https://www.rust-lang.org/"
    ><img
      title="Lang: Rust"
      src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white"
  /></a>
  <a href="http://www.apache.org/licenses/LICENSE-2.0"
    ><img
      title="LICENSE: Apache-2.0"
      src="https://img.shields.io/badge/Apache--2.0-green?style=for-the-badge"
  /></a>
  <a href="https://icalendar.org/RFC-Specifications/iCalendar-RFC-5545/"
    ><img
      title="iCalendar (RFC 5545)"
      src="https://img.shields.io/badge/iCalendar-6096e8?style=for-the-badge"
  /></a>
</p>

<p align="center">
  <a href="https://github.com/yzx9/aim/actions/workflows/ci.yaml"
    ><img
      title="Continuous integration"
      src="https://img.shields.io/github/actions/workflow/status/yzx9/aim/ci.yaml?label=CI"
  /></a>
  <a href="https://crates.io/crates/aimcal"
    ><img
      title="Crates.io version"
      src="https://img.shields.io/crates/v/aimcal"
  /></a>
  <a href="https://docs.rs/crate/aimcal/latest"
    ><img
      title="docs.rs"
      src="https://img.shields.io/docsrs/aimcal"
  /></a>
</p>

AIM is your intelligent assistant for managing time and tasks.
It **analyzes** your schedule using AI-driven insights,
**interacts** naturally to understand your needs and preferences,
and enables you to **manage** your time with clarity, control, and confidence.

Built on the [iCalendar standard (RFC 5545)](https://icalendar.org/RFC-Specifications/iCalendar-RFC-5545/)
and compatible with [CalDAV](https://en.wikipedia.org/wiki/CalDAV) servers like [Radicale](https://radicale.org/),
AIM ensures interoperability and flexibility across systems.
From smart reminders to personalized insights, AIM helps you work smarter, not harder.

## What AIM Provides

AIM is more than just a calendar tool—it's a comprehensive platform consisting of:

### Core Library

A comprehensive Rust library providing the foundation for calendar and task management, with:

- Event and todo data models and management
- Local SQLite storage with efficient querying
- Timezone-aware datetime handling
- Configuration management
- Short ID mapping for efficient references

### Comprehensive RFC 5545 Library

A robust Rust library for reading and writing iCalendar (RFC 5545) data with:

- Full RFC 5545 specification compliance
- RRULE (recurrence rule) support for recurring events
- Type-safe parsing and serialization
- Zero-copy parsing for optimal performance
- Extensive error reporting

### Command-Line Interface

An intuitive CLI for power users who prefer terminal-based workflows, featuring:

- Quick event and todo management
- Interactive TUI mode
- Multiple output formats (table, JSON)
- Shell completion support

### Chat Interface (Planned)

Natural language interface for interacting with your calendar through conversation, powered by LLMs
for intelligent command parsing and execution.

### REST API (Planned)

A complete web API for programmatic access, enabling:

- Third-party integrations
- Webhook support
- CalDAV synchronization
- Cross-platform accessibility

## Features

Built on top of the core libraries, AIM delivers these key capabilities:

- **RFC 5545 Compliance**: Full iCalendar standard support ensuring compatibility with Google
  Calendar, Apple Calendar, Outlook, and other calendar applications
- **High-Performance Parsing**: Zero-copy parsing with the `aimcal-ical` library for efficient
  iCalendar data processing
- **Recurring Events**: RRULE support for complex recurring patterns (daily, weekly, monthly,
  yearly, and custom schedules)
- **Interactive TUI**: Terminal-based user interface with keyboard navigation for efficient
  calendar management
- **Smart Queries**: Filter and search events by date, location, summary, and custom properties
  with convenient short numeric IDs for quick event and todo references
- **Cross-Platform**: Runs on Linux, macOS, and Windows with consistent behavior
- **Developer-Friendly**: Modular library design lets you use individual components in your own
  Rust projects

## Usage

### ▶️ Run with Cargo

To run the CLI using Cargo:

```sh
cargo install aimcal
aim --help
```

### ❄️ Run with Nix

```sh
nix run . -- --help
```

## Configuration

AIM can be configured via three methods (in priority order):

1. **CLI flag**: `aim --config /path/to/config.toml`
2. **Environment variable**: `AIM_CONFIG=/path/to/config.toml aim`
3. **Default location**: `$XDG_CONFIG_HOME/aim/config.toml` (Unix) or `%LOCALAPPDATA%/aim/config.toml` (Windows)

See `cli/config.example.toml` in the repository for a sample configuration file.

### Development Setup

For local development, `.envrc` file sets `AIM_CONFIG` to point to `cli/config.dev.toml`, which uses isolated development directories (`.dev-calendar/` and `.dev-state/`) to keep your work separate from your actual calendar data.

If using Nix without direnv, the development environment is automatically configured with the same `cli/config.dev.toml`.

#### Initialize with Examples

To quickly populate your development calendar with all example files:

```bash
just init-examples
```

This command copies all example files to `.dev-calendar/` directory, which will be automatically loaded on the first `aim` run.

### Example iCalendar Files

The `examples/` directory contains sample iCalendar (`.ics`) files demonstrating various features:

- **simple-meeting.ics** - Basic one-time meeting event
- **recurring-task.ics** - Daily recurring task with RRULE
- **todo-with-priority.ics** - Todo with priority and progress tracking
- **multi-event.ics** - Complex calendar with multiple events, todos, and timezone definitions

See `examples/README.md` for detailed documentation of each example file.

## Goals

- **Enable command-line calendar management**: Perform queries and manage events and todos directly from the CLI.
- **Leverage LLMs for intelligent assistance**: Offer smart scheduling and reminder suggestions tailored to user preferences.
- **Integrate with external systems**: Support CalDAV providers and expose Webhook/REST APIs for triggers and calendar access.

## Roadmap

### 📅 Calendar Features

- [x] Listing events and todos
- [x] Creating and editing events and todos
- [ ] Undo history editing
- [ ] Full text search (grepping)
- [ ] Recurring events
- [ ] TUI: Markdown support

### 🤖 AI Capabilities

- [ ] AI operations: parse and execute user commands on calendar
- [ ] Intelligent suggestions
- [ ] Personalized experience

### 🔌 Integrations

- [ ] CalDAV support
- [ ] Webhook/REST API

## Acknowledgements

We'd like to thank all FOSS projects, particularly:

- [khal](https://github.com/pimutils/khal) - A CLI calendar application
- [todoman](https://github.com/pimutils/todoman) - A simple task manager
- [icalendar](https://github.com/hoodie/icalendar) - icalendar library, in Rust of course

Their work has been a significant inspiration for AIM's design and functionality.

## Contribution

Any help in the form of descriptive and friendly [issues](https://github.com/yzx9/aim/issues) or
comprehensive pull requests are welcome!

Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
AIM by you, as defined in the [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) license,
without any additional terms or conditions.

Thanks goes to these wonderful people:

[![Contributors](https://contrib.rocks/image?repo=yzx9/aim)](https://github.com/yzx9/aim/graphs/contributors)

## LICENSE

This work is licensed under a <a rel="license" href="https://www.apache.org/licenses/">Apache-2.0</a>.

Copyright (c) 2025-2026, Zexin Yuan
