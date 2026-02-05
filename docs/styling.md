# AIM Coding Style Guide

This document describes the coding conventions and style guidelines used throughout the AIM project.
Following these conventions helps maintain consistency across the codebase.

## 1. No `mod.rs` Files

We do NOT use `mod.rs` files. Declare modules directly in parent modules:

```
src/
├── lib.rs          # Crate root
├── db.rs           # Module declared with `mod db;`
└── datetime/
    ├── mod.rs      # NOT used
    └── parser.rs   # Use src/datetime_parser.rs instead
```

## 2. Import Organization

Group imports with blank lines separating categories, sort by 1. Standard library; 2. External
crates; 3. Local modules:

```rust
use std::collections::HashMap;

use sqlx::SqlitePool;
use tracing::instrument;

use crate::event::Event;
```

**Anti-patterns to avoid:**

```rust
// BAD: Imports not grouped
use std::collections::HashMap;
use crate::event::Event;
use sqlx::SqlitePool;

// BAD: Local imports before standard library
use crate::types::Href;
use std::collections::HashMap;

// BAD: Imports inside function bodies
fn build_xml() -> Result<String, Error> {
    use quick_xml::events::Event;  // Don't do this!
    // ...
}

// BAD: Using full paths instead of importing
fn parse_data(input: &str) -> Result<Data> {
    let duration = std::time::Duration::from_secs(60);  // Import instead!
    let cursor = std::io::Cursor::new(vec);             // Import instead!
    match crate::config::AuthMethod::Basic { }          // Import instead!
}
```

**Acceptable exceptions:**

```rust
// OK: Disambiguating between same-named types
use std::fmt;
use crate::fmt as custom_fmt;

// OK: Very rare one-off usage where import adds noise
let _ = std::any::TypeId::of::<T>();  // Only used once in entire file
```

## 3. Git Commit

Always run lint and format before commit.

Follow the [Gitmoji](https://gitmoji.dev/) convention:

```
✨ (core): Add event recurrence support
🐛 (cli): Fix crash when parsing invalid dates
📝 (docs): Update installation instructions
♻️ (ical): Refactor parser for better error messages
```

## 4. Comments and Documentation

- **Language**: All comments and documentation MUST be in English.
- All `pub` items of library MUST have documentation:

```rust
/// Parses an iCalendar string into a calendar object.
///
/// # Errors
///
/// Returns an error if the input is not valid iCalendar format.
pub fn parse(input: &str) -> Result<ICalendar, ParseError> {
    // ...
}

/// Represents a calendar event.
pub struct Event { }
```

### Complex Algorithms

Add explanatory comments for non-obvious logic:

```rust
// GOOD: Explains WHY, not WHAT
// Use binary search for O(log n) lookup performance
let index = events.binary_search_by_key(&timestamp, |e| e.timestamp);

// GOOD: Explains complex invariant
// The invariant here is that parent.timestamp is always >= children.timestamp
// This allows us to prune entire subtrees when searching
fn search_tree(node: &Node, timestamp: i64) -> Option<&Event> {
    // ...
}
```

Use inline comments sparingly - prefer self-documenting code:

```rust
// GOOD: Clear code, minimal comments needed
let unique_events: HashSet<_> = events.into_iter().collect();

// BAD: Verbosely explaining what's already clear
// Create a new HashSet from the events iterator
// This will remove duplicates automatically
let unique_events: HashSet<_> = events.into_iter().collect();
```

### TODO/FIXME/PERF

Use standard markers for action items:

```rust
// TODO: Implement recurring events support
// FIXME: This algorithm is O(n²), should be O(n log n)
// PERF: Consider caching this computation
// HACK: Workaround for https://github.com/yzx9/aim/issues/123
```

## 5. Keep It Simple

Avoid over-engineering:

- Don't add features "for later" - YAGNI
- Don't create abstractions for one-off operations
- Don't add error handling for impossible scenarios
- Three similar lines of code > premature abstraction

## License Headers

All source files must start with:

```rust
// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0
```
