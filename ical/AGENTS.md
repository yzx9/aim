# iCal Module

The iCal module provides parsing and serialization capabilities for the iCalendar format (RFC 5545). This crate handles the low-level parsing of iCalendar data using efficient lexical analysis and parsing techniques.

## Folder Structure

```
ical/src/
├── lib.rs            # Public API exports
├── keyword.rs        # Keywords defined in RFC 5545
├── lexer.rs          # Lexical analysis
├── property_spec.rs  # Property specification
├── property_value.rs # Parsing property value
├── syntax.rs         # Parsing syntax for components
└── typed.rs          # Parsing types
```

## Main Components

### Lexer (src/lexer.rs)

Handles tokenization of iCalendar data:

- Uses the `logos` crate for efficient lexical analysis
- Defines `Token` enum for all iCalendar syntax elements
- Recognizes words, delimiters, control characters, symbols, and escape sequences
- Handles iCalendar folding (CRLF whitespace sequences)

#### Token Types

- `DQuote`: Double quote (") character
- `Comma`: Comma (,) for parameter value lists
- `Colon`: Colon (:) separator between property names and values
- `Semicolon`: Semicolon (;) used to separate properties
- `Equal`: Equal sign (=) for parameter values
- `Control`: All control characters except HTAB (ASCII 0x00-0x18, 0x0A-1F and 0x7F)
- `Symbol`: ASCII symbols and special characters
- `Newline`: Carriage Return followed by Line Feed
- `Escape`: Escape sequences (backslash followed by specific characters)
- `Word`: Alphanumeric characters, underscores, hyphens, and other ASCII word characters (0-9, A-Z, a-z, \_, -)
- `UnicodeText`: Non-ASCII Unicode text

### Syntax Parser (src/syntax.rs)

Handles parsing of iCalendar components without type using the `chumsky` parsing framework:

- Provides `syntax()` function for parsing iCalendar strings into `RawComponent`
- Defines `RawComponent` struct for representing iCalendar components with name, properties, and nested children
- Defines `RawProperty` struct for representing component properties with name, parameters, and values
- Defines `RawParameter` struct for representing property parameters with name and multiple values
- Defines `RawParameterValue` struct for representing property parameter values (quoted or unquoted)
- Defines `StrSegments` struct for representing unescaped property values with span tracking
- Implements error reporting with `ariadne` for detailed diagnostics

#### Component Structure

- `RawComponent`: Contains component name, ordered properties vector, and nested children vector
- `RawProperty`: Contains name as `StrSegments`, parameters vector, and multi-value vector
- `RawParameter`: Contains name as `StrSegments` and vector of `RawParameterValue`
- `RawParameterValue`: Contains value as `StrSegments` and quoted flag
- `StrSegments`: Efficient string representation that preserves original spans and supports iteration

#### Parser Features

- **Recursive parsing**: Handles nested components using chumsky's recursive parser
- **BEGIN/END validation**: Ensures matching BEGIN and END tags with proper component names
- **Property parsing**: Supports property names with groups, parameters, and multi-values
- **Parameter parsing**: Handles both quoted and unquoted parameter values
- **Value parsing**: Processes escaped characters and various token types
- **Error recovery**: Provides detailed error reports with source context and span information
- **Zero-copy design**: Uses string slices and spans for efficient parsing

#### Key Constants

- `KW_BEGIN`: "BEGIN" token for component start
- `KW_END`: "END" token for component end

#### Helper Types

- `Either<L, R>`: Utility enum for partitioning properties vs components
- `EitherIterExt`: Trait extension for partitioning either iterators
- `StrSegmentsCharsIter`: Iterator for traversing characters across string segments

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
