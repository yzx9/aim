# iCal Module

The iCal module provides parsing and serialization capabilities for the iCalendar format (RFC 5545). This crate handles the low-level parsing of iCalendar data using efficient lexical analysis and parsing techniques.

## Folder Structure

```
ical/src/
├── lib.rs    # Public API exports
├── lexer.rs  # Lexical analysis of iCalendar format
└── parser.rs # Parsing logic for iCalendar components
```

## Main Components

### Lexer (src/lexer.rs)

Handles tokenization of iCalendar data:

- Uses the `logos` crate for efficient lexical analysis
- Defines `Token` enum for all iCalendar syntax elements
- Recognizes words, delimiters, control characters, symbols, and escape sequences
- Handles iCalendar folding (CRLF whitespace sequences)

#### Token Types

- `Word`: Alphanumeric characters, underscores, hyphens, and other ASCII word characters (0-9, A-Z, a-z, _, -)
- `Semi`: Semicolon (;) used to separate properties
- `Colon`: Colon (:) separator between property names and values
- `Eq`: Equal sign (=) for parameter values
- `Comma`: Comma (,) for parameter value lists
- `Quote`: Double quote (") character
- `Control`: Control characters (ASCII 0x00-0x1F and 0x7F)
- `Symbol`: ASCII symbols and special characters
- `Escape`: Escape sequences (backslash followed by specific characters)
- `Folding`: CRLF followed by space or tab for iCalendar folding
- `UnicodeText`: Non-ASCII Unicode text

### Parser (src/parser.rs)

Handles parsing of iCalendar components using the `chumsky` parsing framework:

- Provides `parse()` function for parsing iCalendar strings
- Defines `Component` struct for representing iCalendar components
- Defines `Property` struct for representing component properties
- Defines `Param` struct for representing property parameters
- Defines `RawValue` struct for representing unescaped property values
- Implements error reporting with `ariadne` for detailed diagnostics

#### Component Structure

- `name`: Component name (e.g., "VCALENDAR", "VEVENT", "VTODO")
- `props`: Vector of properties with preserved ordering
- `subcomponents`: Vector of nested components

#### Property Structure

- `group`: Optional property group
- `name`: Property name (case-insensitive but preserved for output)
- `params`: Vector of parameters (allowing duplicates and multi-values)
- `value`: Raw property value (unfolded and unescaped)

#### Value Processing

- `ValueSegs`: Tracks segments of values that may span multiple lines
- `unescape_into()`: Handles unescaping of iCalendar values according to RFC 5545
- `resolve_unescaped()`: Combines segments and unescapes values as needed

## Dependencies

- **logos**: Fast lexical analysis for tokenizing
- **chumsky**: Parser combinator library for building the parser
- **ariadne**: Error reporting library for detailed parse error diagnostics

## Code Standards

- Full compliance with iCalendar specification (RFC 5545)
  - Support for iCalendar folding and unfolding
  - Proper handling of escaped characters in values
- Efficient parsing with minimal allocations
- Comprehensive error reporting with source context
- Zero-copy parsing where possible for performance
- Extensive test coverage for parser functionality
- Always write code and comments in English

## Usage Examples

```rust
use aimcal_ical::parse;

let ical_src = "\
BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
";

match parse(ical_src) {
    Ok(component) => {
        // Process parsed component
        println!("Parsed component: {}", component.name);
    }
    Err(reports) => {
        // Handle parse errors
        for report in reports {
            report.print(ariadne::Source::from(ical_src));
        }
    }
}
```
