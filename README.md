<div align="center" id="madewithlua">
  <img src="./aim.svg" width="150" height="130" />
</div>

<h1 align="center">AIM</h1>
<h3 align="center">Analyze. Interact. Manage Your Time, with calendar support</h3>

<p align="center">
  <a href="https://www.rust-lang.org/"
    ><img
      title="Badge: Rust"
      src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white"
  /></a>
  <a href="http://www.apache.org/licenses/LICENSE-2.0"
    ><img
      title="Badge: Apache-2.0"
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

## Roadmap (by priority)

### 📅 Calendar Features

- [x] Todo query & edting – Support CLI queries, add, edit, and delete todos
- [ ] Event query & edting – Support CLI queries, add, edit, and delete events
- [ ] Revert editing - Undo history action, including adding, edit and delete
- [ ] Recurring events – Handle creation and editing of repeating events

### 🤖 AI Capabilities

- [ ] AI operation – Parse and execute user commands on calendar
- [ ] AI suggestions – Provide intelligent scheduling and reminder suggestions
- [ ] AI memory (long-term) – Store user preferences and interaction history

### 🔌 Integrations

- [ ] CalDAV support – Work with Google, Outlook, iCloud, CalDAV, etc.
- [ ] Webhook/REST API – Allow external triggers and calendar access via API

## LICENSE

This work is licensed under a <a rel="license" href="https://www.apache.org/licenses/">Apache-2.0</a>.

Copyright (c) 2025-2025, Zexin Yuan
