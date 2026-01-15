# iCal Module Architecture

The iCal module provides a comprehensive parser and formatter for the iCalendar format (RFC 5545)
using a multi-phase analysis approach. The architecture separates concerns through distinct layers
for syntax parsing, typed analysis and semantic validation.

## Architecture Overview

The parser follows a **four-phase pipeline**:

1. **Syntax Analysis** - Tokenizes raw iCalendar text and builds component structure
   - **Lexer** - Tokenizes raw iCalendar text into structured tokens
   - **Scanner** - Scans token streams into content lines
   - **Tree Builder** - Builds component tree from content lines using stack-based algorithm
2. **Typed Analysis** - Validates and converts components to strongly-typed representations through
   three sub-passes:
   - **Parameter Pass** - Parses and validates iCalendar parameters (RFC 5545 Section 3.2)
   - **Value Pass** - Parses and validates property value types (RFC 5545 Section 3.3)
   - **Property Pass** - Validates property-specific constraints
3. **Semantic Analysis** - Validates RFC 5545 semantics (required properties, constraints,
   relationships)
4. **Formatter** - Format icalendar to RFC 5545

### Syntax Analysis Phase

Transforms raw iCalendar text into a hierarchical component tree through three sub-phases:

1. **Lexer** - Tokenizes raw iCalendar text into structured tokens while preserving source position
   information for error reporting
2. **Scanner** - Scans token streams into content lines, validating structure and collecting errors
3. **Tree Builder** - Builds a tree of components with properties and parameters using a stack-
   based algorithm, validates component nesting (BEGIN/END matching)

### Typed Analysis Phase

Validates all components against RFC 5545 specifications through three sub-passes:

1. **Parameter Pass**
   - Parses and validates iCalendar parameters per RFC 5545 Section 3.2
   - Converts parameter strings to strongly-typed representations using `S: StringStorage`
   - Validates parameter values (e.g., enum values for CUTYPE, ENCODING, etc.)
   - Provides `Parameter<S>` enum with `ParameterKind<S>` for type-safe handling

2. **Value Pass**
   - Parses and validates property value types per RFC 5545 Section 3.3
   - Converts value strings to appropriate Rust types (dates, durations, integers, etc.)
   - Handles type inference when VALUE parameter is not specified
   - Processes escape sequences (e.g., `\n`, `\;`, `\,`) in text values
   - Provides `Value<S>` enum with specific types (`ValueDate`, `ValueDateTime`, etc.)

3. **Property Pass**
   - Validates property-specific constraints and relationships
   - Handles property cardinality and multiplicity rules
   - Validates inter-property dependencies
   - Implements property kind validation to ensure type safety
   - Creates strongly-typed wrapper types for each property using `Property<S>`

**Property Organization**: Property type definitions are organized in the `property/` module by
RFC 5545 sections. Each property type has:

- A dedicated wrapper type (e.g., `Created`, `DtStart`, `Summary`)
- A `kind()` method returning the corresponding `PropertyKind`
- Type validation in `TryFrom<ParsedProperty>` implementations that verify the property kind
  matches the expected type
- A unified `Property` enum in `property.rs` that provides type-safe access to all property
  variants

### Semantic Analysis Phase

Performs RFC 5545 semantic validation to ensure iCalendar data is logically correct and complete.
This phase goes beyond syntax checking to validate business rules and constraints defined in the
specification.

### Unified Parser

The main `parse()` function coordinates all phases, returns `Result<Vec<ICalendar<Segments<'_>>>, Vec<ParseError<'_>>>`
which provides zero-copy parsing by default, with the option to convert to owned types using the
`to_owned()` method.

### Formatter

The formatter module provides RFC 5545-compliant serialization of iCalendar data. The formatter
supports serialization of both borrowed and owned data representations.

## String Storage Abstraction

The parser uses a generic storage parameter system built on the `StringStorage` trait to enable
both zero-copy parsing and owned data representations with a unified API.

**Implementations:**

- `Segments<'src>` - For zero-copy borrowed segments with position information
- `String` - For owned string data (after calling `.to_owned()`)

This abstraction enables the entire type system to use generic bounds like `S: StringStorage`
instead of being tied to specific string types, providing flexibility while maintaining type safety.

Types use `Segments<'src>` for zero-copy borrowed data and `String` for owned data.
Conversion between the two is done through the `to_owned()` method on each type.

## Module Structure

```
ical/
├── Cargo.toml
├── CLAUDE.md
├── RFC5545.txt     # If you have questions, check the RFC first and use search—it's very long
├── src/
│   ├── lib.rs              # Public API exports
│   ├── keyword.rs          # RFC 5545 keyword constants
│   ├── string_storage.rs   # String storage abstraction (StringStorage trait, Span, Segments)
│   ├── parser.rs           # Unified parser orchestration
│   ├── syntax.rs           # Syntax analysis
│   ├── syntax/             # Syntax analysis implementation
│   │   ├── lexer.rs        # Lexical analysis (tokenization)
│   │   ├── scanner.rs      # Scans tokens into content lines
│   │   └── tree_builder.rs # Builds component tree from content lines
│   ├── typed.rs            # Typed module entry point
│   ├── parameter.rs        # Parameter enum and TryFrom implementation
│   ├── parameter/          # Parameter pass implementation
│   │   ├── definition.rs   # Parameter type enums and parsing functions
│   │   ├── kind.rs         # ParameterKind enum
│   │   └── util.rs         # Parameter parsing utilities
│   ├── value.rs            # Value enum and parsing
│   ├── value/              # Value pass implementation
│   │   ├── datetime.rs     # Date/time value types (ValueDate, ValueDateTime, ValueTime, ValueUtcOffset)
│   │   ├── duration.rs     # Duration value type (ValueDuration)
│   │   ├── miscellaneous.rs # Miscellaneous value types (Binary, Boolean)
│   │   ├── numeric.rs      # Numeric value types (Float, Integer)
│   │   ├── period.rs       # Period value type (ValuePeriod)
│   │   ├── rrule.rs        # Recurrence rule type (RecurrenceRule, Day, WeekDay)
│   │   └── text.rs         # Text value type (ValueText)
│   ├── property.rs         # Property enum and TryFrom implementation
│   ├── property/           # Property types organized by RFC 5545 sections
│   │   ├── kind.rs         # PropertyKind enum with value type mappings
│   │   ├── calendar.rs     # Section 3.7 - Calendar properties (CalScale, Method, ProdId, Version)
│   │   ├── descriptive.rs  # Section 3.8.1 - Descriptive properties (Attach, Categories, Class, Comment, etc.)
│   │   ├── datetime.rs     # Section 3.8.2 - Date/time properties (Completed, DtStart, DtEnd,Due, Duration, etc.)
│   │   ├── timezone.rs     # Section 3.8.3 - Time zone properties (TzId, TzName, TzOffsetFrom, etc.)
│   │   ├── relationship.rs # Section 3.8.4 - Relationship properties (Attendee, Contact, Organizer, etc.)
│   │   ├── recurrence.rs   # Section 3.8.5 - Recurrence properties (ExDate, RDate)
│   │   ├── alarm.rs        # Section 3.8.6 - Alarm properties (Action, Repeat, Trigger)
│   │   ├── changemgmt.rs   # Section 3.8.7 - Change management properties (Created, DtStamp, etc.)
│   │   ├── miscellaneous.rs # Section 3.8.8 - Miscellaneous properties (RequestStatus)
│   │   └── util.rs         # Common properties and utilities (Text, macros, helpers)
│   ├── semantic.rs         # Semantic analysis entry point and error types
│   ├── semantic/           # Semantic component implementations
│   │   ├── icalendar.rs    # ICalendar root component
│   │   ├── valarm.rs       # VAlarm component
│   │   ├── vevent.rs       # VEvent component
│   │   ├── vfreebusy.rs    # VFreeBusy component
│   │   ├── vjournal.rs     # VJournal component
│   │   ├── vtimezone.rs    # VTimeZone component
│   │   └── vtodo.rs        # VTodo component
│   ├── formatter.rs        # RFC 5545 formatter orchestration
│   └── formatter/          # RFC 5545 formatter
│       ├── component.rs    # Component formatting
│       ├── property.rs     # Property formatting
│       ├── parameter.rs    # Parameter formatting
│       └── value.rs        # Value formatting
└── tests/
    ├── lexer.rs         # Lexer tests
    ├── syntax.rs        # Syntax tests
    ├── typed.rs         # Typed analysis tests
    ├── semantic.rs      # Semantic tests
    ├── formatter.rs     # Formatter tests
    └── round_trip.rs    # Round-trip tests (parse → to_owned → format → parse)
```

## Dependencies

- **logos** - Lexer generation
- **chumsky** - Parser combinators
- **lexical** - Numeric parsing
- **strum** - Enum utilities
- **thiserror** - Error handling
- **jiff** (optional) - Datetime and timezone validation

## Features

- **jiff** (default) - Datetime integration

## Design Principles

- **Phase Separation**: Each parsing phase has clear responsibilities and well- defined interfaces
- **Three-Pass Typed Analysis**: The typed analysis phase is split into three independent passes
  for better modularity and maintainability:
  - **Parameter Pass** handles all parameter-related parsing and validation
  - **Value Pass** handles all value type parsing and validation
  - **Property Pass** handles property-level constraints and type validation
- **Property Kind Validation**: Comprehensive type safety through:
  - `PropertyKind` enum (`property/kind.rs`) representing all RFC 5545 properties
  - Each property has a `kind()` method returning its corresponding `PropertyKind`
  - `TryFrom<ParsedProperty>` implementations verify property kind matches expected type
  - Unified `Property` enum providing type-safe access to all property variants
- **RFC 5545 Compliance**: Comprehensive validation and serialization against the iCalendar
  specification with property-to-value-type mappings defined in `PropertyKind`
- **Extensibility Support**: Full support for custom and experimental properties. Per RFC 5545,
  unknown content does not cause parsing to fail
- **Bidirectional Support**: Complete parser and formatter for read/write operations:
  - Parse iCalendar data into strongly-typed representations
  - Serialize components, properties, parameters, and values back to RFC 5545 format
  - Zero-copy parameter writer functions for efficient serialization
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data with
  dedicated wrapper types for each property (e.g., `Created`, `DtStart`, `Summary`)
- **Generic Storage Parameter System**: Unified type system using generic storage parameter
  `S: StringStorage` for flexibility:
  - **Parameters**: `Parameter<S: StringStorage>`
  - **Properties**: `Property<S: StringStorage>`
  - **Values**: `Value<S: StringStorage>`
  - **Semantic Types**: All component types (e.g., `VEvent`, `VTodo`, `ICalendar`) use the same
    pattern
  - This enables both zero-copy parsing (borrowed data) and owned data representations
  - Use `Type<Segments<'src>>` for borrowed data and `Type<String>` for owned data
  - Convert between representations using the `to_owned()` method
- **Performance**: Zero-copy parsing where possible, minimal allocations
- **Optional datetime dependencies**: All types use the value module's `ValueDate`, `ValueTime`,
  and `ValueDateTime` instead of directly using datetime types from `jiff` or `chrono` (planned)

## Error Handling

The architecture provides comprehensive error reporting with:

- Source location information for all errors
- Detailed error messages explaining RFC 5545 violations
- Phase-specific error categorization (syntax vs. validation)

## Feature Support

- **Generic Storage Parameter System**: Unified type system using `S: StringStorage`
  across all layers.
- **Property Kind System**: Complete `PropertyKind<S>` enum with value type mappings for all RFC
  5545 properties, enabling compile-time type safety
- **Unified Property Enum**: Single `Property<S>` enum providing type-safe access to all property
  variants with `TryFrom<ParsedProperty>` validation
- **Unknown/Custom Property Support**: Full RFC 5545 compliance for extensibility:
  - Parsing never fails due to unknown content (per RFC 5545 Section 4.1)
  - Preserves original data for round-trip serialization compatibility
- **Extensible Property Support**: Property types organized by RFC 5545 sections for easy
  maintenance and extension
- **Semantic Type System**: High-level semantic representations of iCalendar components using
  `Type<Segments<'src>>` for borrowed data and `Type<String>` for owned data
- **RFC 5545 Compliance**: Complete support for all required value types and parameters
- **RFC 5545 Serialization**: Complete formatter module for writing iCalendar data
- **(feature-gated) Timezone Validation**: Optional integration with `jiff` for timezone database
  validation
