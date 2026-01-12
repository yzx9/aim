# iCal Module Architecture

The iCal module provides a comprehensive parser and formatter for the iCalendar format (RFC 5545)
using a multi-phase analysis approach. The architecture separates concerns through distinct layers
for lexical analysis, syntax parsing, and type validation.

## Architecture Overview

The parser follows a **four-phase pipeline**:

1. **Lexical Analysis** - Tokenizes raw iCalendar text into structured tokens
2. **Syntax Analysis** - Assembles tokens into component structure
3. **Typed Analysis** - Validates and converts components to strongly-typed representations through
   three sub-passes:
   - **Parameter Pass** - Parses and validates iCalendar parameters (RFC 5545 Section 3.2)
   - **Value Pass** - Parses and validates property value types (RFC 5545 Section 3.3)
   - **Property Pass** - Validates property-specific constraints
4. **Semantic Analysis** - Validates RFC 5545 semantics (required properties, constraints,
   relationships)
5. **Formatter** - Format icalendar to RFC 5545

### Lexer Phase

Transforms raw iCalendar text into tokens while preserving source position information for error
reporting.

### Syntax Phase

Builds a tree of components with properties and parameters, validates component nesting (BEGIN/END
matching), and collects text for further processing in the Value Pass.

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

The main `parse()` function coordinates all phases, returns `Result<Vec<ICalendarRef<'_>>, Vec<ParseError<'_>>>`
which provides zero-copy parsing by default, with the option to convert to owned types using the
`to_owned()` method.

### Formatter

The formatter module provides RFC 5545-compliant serialization of iCalendar data. The formatter
supports both zero-copy writing (with `Ref` types) and owned data serialization.

## String Storage Abstraction

The parser uses a generic storage parameter system built on the `StringStorage` trait to enable
both zero-copy parsing and owned data representations with a unified API.

**Implementations:**

- `SpannedSegments<'src>` - For zero-copy borrowed segments with position information
- `String` - For owned string data (after calling `.to_owned()`)

This abstraction enables the entire type system to use generic bounds like `S: StringStorage`
instead of being tied to specific string types, providing flexibility while maintaining type safety.

### Convenience Aliases

For each generic type using `S: StringStorage`, the crate provides convenience aliases:

**For Zero-Copy Parsing (Borrowed Data):**

- `ParameterRef<'src>` = `Parameter<SpannedSegments<'src>>`
- `ValueRef<'src>` = `Value<SpannedSegments<'src>>`
- `PropertyRef<'src>` = `Property<SpannedSegments<'src>>`
- `ICalendarRef<'src>` = `ICalendar<SpannedSegments<'src>>`
- `VEventRef<'src>`, `VTodoRef<'src>`, etc.

**For Owned Data:**

- `ParameterOwned` = `Parameter<String>`
- `ValueOwned` = `Value<String>`
- `PropertyOwned` = `Property<String>`
- `ICalendarOwned` = `ICalendar<String>`
- `VEventOwned`, `VTodoOwned`, etc.

This unified API allows seamless conversion between zero-copy and owned representations through the
`to_owned()` method, enabling efficient parsing when needed and data ownership when required (e.g.,
for serialization or long-term storage).

## Module Structure

```
ical/
├── Cargo.toml
├── CLAUDE.md
├── RFC5545.txt     # If you have questions, check the RFC first and use search—it's very long
├── src/
│   ├── lib.rs              # Public API exports
│   ├── keyword.rs          # RFC 5545 keyword constants
│   ├── lexer.rs            # Lexical analysis (tokenization)
│   ├── syntax.rs           # Syntax analysis
│   ├── string_storage.rs   # String storage abstraction (StringStorage trait, Span, SpannedSegments)
│   ├── parser.rs           # Unified parser orchestration
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
    └── to_owned_basic.rs # To owned conversion tests
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
  - **Parameters**: `Parameter<S: StringStorage>` with convenience aliases:
    - `ParameterRef<'src>` = `Parameter<SpannedSegments<'src>>` for zero-copy parsing
    - `ParameterOwned` = `Parameter<String>` for owned data
  - **Properties**: `Property<S: StringStorage>` with convenience aliases:
    - `PropertyRef<'src>` = `Property<SpannedSegments<'src>>`
    - `PropertyOwned` = `Property<String>`
  - **Values**: `Value<S: StringStorage>` with convenience aliases:
    - `ValueRef<'src>` = `Value<SpannedSegments<'src>>`
    - `ValueOwned` = `Value<String>`
  - **Semantic Types**: All component types (e.g., `VEvent`, `VTodo`, `ICalendar`) use the same
    pattern with `Ref` and `Owned` variants
  - This enables both zero-copy parsing (borrowed data) and owned data representations with a
    unified API
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
- **Timezone Validation**: Optional integration with `jiff` for timezone database validation
  (feature-gated)
- **Extensible Property Support**: Property types organized by RFC 5545 sections for easy
  maintenance and extension
- **RFC 5545 Compliance**: Complete support for all required value types and parameters
- **Semantic Type System**: High-level semantic representations of iCalendar components with `Ref`
  and `Owned` variants for all component types
- **RFC 5545 Serialization**: Complete zero-copy (planned) formatter module for writing iCalendar data
