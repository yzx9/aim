# AIM Testing Guide

This document describes testing practices and conventions for the AIM project.

## Testing Philosophy

We follow a pragmatic testing approach:

- **Test behavior, not implementation** - Focus on what the code does, not how
- **Fast tests** - Keep unit tests fast for rapid feedback
- **Integration where it matters** - Test real interactions at boundaries
- **Clear error messages** - Tests should fail with helpful messages

## Running Tests

### Run All Tests

```bash
# Using just (recommended)
just test

# Using cargo directly
cargo test --workspace
```

### Run Tests for Specific Crate

```bash
cargo test -p aimcal-core
cargo test -p aimcal-cli
cargo test -p aimcal-ical
```

### Run Specific Test

```bash
cargo test test_name
```

### Run Tests with Output

```bash
cargo test -- --nocapture --test-threads=1
cargo test -- --show-output
```

## Test Organization

### Unit Tests

Place unit tests in the same module as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_function_calculates_correct_result() {
        let result = something();
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

For crate-level integration tests, use the `tests/` directory:

```
ical/
├── src/
│   └── ...
└── tests/
    ├── lexer.rs       # Integration tests for lexer
    ├── syntax.rs      # Integration tests for parser
    ├── typed.rs       # Integration tests for typed analysis
    └── semantic.rs    # Integration tests for semantic analysis
```

## Writing Good Tests

### Test Naming

Use descriptive names following the `{module}_{action}_{scenario}` pattern:

```rust
// Good - Descriptive and follows pattern
#[test]
fn lexer_tokenizes_empty_input() {}

#[test]
fn lexer_tokenizes_simple_icalendar() {}

#[test]
fn syntax_empty_component() {}

#[test]
fn syntax_property_with_parameters() {}

// Bad - Not descriptive
#[test]
fn test_event() {}

// Bad - Inconsistent prefix
#[test]
fn event_creation() {}
```

**Naming pattern:**

- `{module}` - What's being tested (e.g., `lexer`, `syntax`, `parser`)
- `{action}` - What happens (e.g., `tokenizes`, `parses`, `validates`)
- `{scenario}` - Specific case (e.g., `empty_input`, `with_parameters`, `invalid_dates`)

The module prefix is recommended for integration tests but optional for unit tests where the context is clear.

### Test Structure

Follow the Arrange-Act-Assert (AAA) pattern:

```rust
#[test]
fn short_id_store_maps_uuid_to_numeric_id() {
    // Arrange
    let uuid = Uuid::new_v4();
    let mut store = ShortIdStore::new();

    // Act
    let short_id = store.assign(uuid).await;

    // Assert
    assert_eq!(store.get_uuid(short_id).await, Some(uuid));
}
```

### Test Fixtures

Use helper functions to reduce duplication:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test event
    fn mock_event() -> Event {
        Event::new()
            .with_summary("Test Event")
            .with_start(DateTime::now())
    }

    #[test]
    fn event_serializes_to_json() {
        let event = mock_event();
        // Test serialization
    }

    #[test]
    fn event_deserializes_from_json() {
        let event = mock_event();
        // Test deserialization
    }
}
```

## Test Coverage Goals

- Critical business logic: >90% coverage
- Parsing logic: >95% coverage
- CLI formatting: >80% coverage

## Continuous Integration

Tests run automatically on GitHub Actions for:

- Ubuntu (latest)
- macOS (latest)
- Windows (latest)

All tests must pass before merging a PR.

## Pre-Commit Checklist

Before committing changes:

- [ ] **Add tests** for new functionality
- [ ] **Update tests** for refactored code
- [ ] **Run tests**: `just test`
- [ ] **Run linter**: `just lint`
- [ ] **Format code**: `cargo fmt`

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tokio::test Documentation](https://docs.rs/tokio/latest/tokio/attr.test.html)
- [SQLx Testing](https://docs.rs/sqlx/latest/sqlx/testing/index.html)
