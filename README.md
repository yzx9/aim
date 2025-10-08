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

Built on the [iCalendar standard (RFC 5545)](https://icalendar.org/RFC-Specifications/iCalendar-RFC-5545/) and compatible with [CalDAV](https://en.wikipedia.org/wiki/CalDAV) servers like [Radicale](https://radicale.org/), AIM ensures interoperability and flexibility across systems.
From smart reminders to personalized insights, AIM helps you work smarter, not harder.

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

## Goals

- **Enable command-line calendar management**: Perform queries and manage events and todos directly from the CLI.
- **Leverage LLMs for intelligent assistance**: Offer smart scheduling and reminder suggestions tailored to user preferences.
- **Integrate with external systems**: Support CalDAV providers and expose Webhook/REST APIs for triggers and calendar access.

## Roadmap

### 📅 Calendar Features

- [x] Listing event and todos
- [x] Creating and editing event and todo
- [ ] Undo history editing
- [ ] Full text search (grepping)
- [ ] Recurring events
- [ ] TUI: Markdown support

### 🤖 AI Capabilities

- [ ] AI operation, parse and execute user commands on calendar
- [ ] Intelligent suggestions
- [ ] Personalized

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

Any help in form of descriptive and friendly [issues](https://github.com/yzx9/aim/issues) or comprehensive pull requests are welcome!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in aim by you, as defined in the [Apache-2.0](www.apache.org/licenses/LICENSE-2.0) license, without any additional terms or conditions.

Thanks goes to these wonderful people:

[![Contributors](https://contrib.rocks/image?repo=yzx9/aim)](https://github.com/yzx9/aim/graphs/contributors)

## LICENSE

This work is licensed under a <a rel="license" href="https://www.apache.org/licenses/">Apache-2.0</a>.

Copyright (c) 2025-2025, Zexin Yuan
