# iCal Module Architecture

The iCal module provides a comprehensive parser for the iCalendar format
(RFC 5545) using a multi-phase analysis approach. The architecture separates
concerns through distinct layers for lexical analysis, syntax parsing, and type
validation.

## Architecture Overview

The parser follows a **four-phase pipeline**:

1. **Lexical Analysis** - Tokenizes raw iCalendar text into structured tokens
2. **Syntax Analysis** - Assembles tokens into component structure
3. **Typed Analysis** - Validates and converts components to strongly-typed
   representations through three sub-passes:
   - **Parameter Pass** - Parses and validates iCalendar parameters (RFC 5545 Section 3.2)
   - **Value Pass** - Parses and validates property value types (RFC 5545 Section 3.3)
   - **Property Pass** - Validates property-specific constraints
4. **Semantic Analysis** - Validates RFC 5545 semantics (required properties,
   constraints, relationships)

### Lexer Phase

Transforms raw iCalendar text into tokens while preserving source position
information for error reporting.

### Syntax Phase

Builds a tree of components with properties and parameters, validates component
nesting (BEGIN/END matching), and collects text for further processing in the
Value Pass.

### Typed Analysis Phase

Validates all components against RFC 5545 specifications through three sub-passes:

1. **Parameter Pass**
   - Parses and validates iCalendar parameters per RFC 5545 Section 3.2
   - Converts parameter strings to strongly-typed representations
   - Validates parameter values (e.g., enum values for CUTYPE, ENCODING, etc.)
   - Provides `Parameter` enum and `ParameterKind` type for type-safe parameter handling

2. **Value Pass**
   - Parses and validates property value types per RFC 5545 Section 3.3
   - Converts value strings to appropriate Rust types (dates, durations, integers, etc.)
   - Handles type inference when VALUE parameter is not specified
   - Processes escape sequences (e.g., `\n`, `\;`, `\,`) in text values
   - Provides `Value` enum and specific value types (`ValueDate`, `ValueDateTime`, etc.)

3. **Property Pass**
   - Validates property-specific constraints and relationships
   - Handles property cardinality and multiplicity rules
   - Validates inter-property dependencies
   - Implements property kind validation to ensure type safety
   - Creates strongly-typed wrapper types for each property

**Property Organization**: Property type definitions are organized in the `property/`
module by RFC 5545 sections. Each property type has:

- A dedicated wrapper type (e.g., `Created`, `DtStart`, `Summary`)
- A `kind()` method returning the corresponding `PropertyKind`
- Type validation in `TryFrom<ParsedProperty>` implementations that verify the
  property kind matches the expected type
- A unified `Property` enum in `property.rs` that provides type-safe access to all
  property variants

### Semantic Analysis Phase

Performs RFC 5545 semantic validation to ensure iCalendar data is logically
correct and complete. This phase goes beyond syntax checking to validate
business rules and constraints defined in the specification.

### Unified Parser

Coordinates all phases, aggregates errors from each phase, and provides a
single entry point for parsing operations.

The main `parse()` function returns `Result<Vec<ICalendar>, Vec<ParseError>>`,
where the vector contains all successfully parsed VCALENDAR objects from the
input stream.

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
│   ├── parser.rs           # Unified parser orchestration
│   ├── typed.rs            # Typed module entry point
│   ├── semantic.rs         # Semantic analysis entry point and error types
│   ├── parameter.rs        # Parameter enum and TryFrom implementation
│   ├── parameter/          # Parameter pass implementation
│   │   ├── definition.rs   # Parameter type enums and parsing functions
│   │   ├── kind.rs         # ParameterKind enum
│   │   └── util.rs         # Parameter parsing utilities
│   ├── property.rs         # Property enum and TryFrom implementation
│   ├── property/           # Property types organized by RFC 5545 sections
│   │   ├── kind.rs         # PropertyKind enum with value type mappings
│   │   ├── alarm.rs        # Section 3.8.6 - Alarm properties (Action, Repeat, Trigger)
│   │   ├── calendar.rs     # Section 3.7 - Calendar properties (CalScale, Method, ProdId, Version)
│   │   ├── changemgmt.rs   # Section 3.8.7 - Change management properties (Created, DtStamp, etc.)
│   │   ├── datetime.rs     # Section 3.8.2 - Date/time properties (Completed, DtEnd, DtStart, Due, Duration, etc.)
│   │   ├── descriptive.rs  # Section 3.8.1 - Descriptive properties (Attach, Categories, Class, Comment, etc.)
│   │   ├── miscellaneous.rs # Section 3.8.8 - Miscellaneous properties (RequestStatus)
│   │   ├── recurrence.rs   # Section 3.8.5 - Recurrence properties (ExDate, RDate)
│   │   ├── relationship.rs # Section 3.8.4 - Relationship properties (Attendee, Contact, Organizer, etc.)
│   │   ├── timezone.rs     # Section 3.8.3 - Time zone properties (TzId, TzName, TzOffsetFrom, etc.)
│   │   └── util.rs         # Text property utilities (Text, Texts, macros, helpers)
│   ├── value.rs            # Value enum and parsing
│   ├── value/              # Value pass implementation
│   │   ├── datetime.rs     # Date/time value types (ValueDate, ValueDateTime, ValueTime, ValueUtcOffset)
│   │   ├── duration.rs     # Duration value type (ValueDuration)
│   │   ├── miscellaneous.rs # Miscellaneous value types (Binary, Boolean)
│   │   ├── numeric.rs      # Numeric value types (Float, Integer)
│   │   ├── period.rs       # Period value type (ValuePeriod)
│   │   ├── rrule.rs        # Recurrence rule type (RecurrenceRule, Day, WeekDay)
│   │   └── text.rs         # Text value type (ValueText)
│   └── semantic/           # Semantic component implementations
│       ├── icalendar.rs    # ICalendar root component
│       ├── valarm.rs       # VAlarm component
│       ├── vevent.rs       # VEvent component
│       ├── vfreebusy.rs    # VFreeBusy component
│       ├── vjournal.rs     # VJournal component
│       ├── vtimezone.rs    # VTimeZone component
│       └── vtodo.rs        # VTodo component
└── tests/
    ├── lexer.rs    # Lexer tests
    ├── syntax.rs   # Syntax tests
    ├── typed.rs    # Typed analysis tests
    └── semantic.rs # Semantic tests
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

- **Phase Separation**: Each parsing phase has clear responsibilities and well-
  defined interfaces
- **Three-Pass Typed Analysis**: The typed analysis phase is split into three
  independent passes for better modularity and maintainability:
  - **Parameter Pass** handles all parameter-related parsing and validation
  - **Value Pass** handles all value type parsing and validation
  - **Property Pass** handles property-level constraints and type validation
- **Property Kind Validation**: Comprehensive type safety through:
  - `PropertyKind` enum (`property/kind.rs`) representing all RFC 5545 properties
  - Each property has a `kind()` method returning its corresponding `PropertyKind`
  - `TryFrom<ParsedProperty>` implementations verify property kind matches expected type
  - Unified `Property` enum providing type-safe access to all property variants
- **RFC 5545 Compliance**: Comprehensive validation against the iCalendar
  specification with property-to-value-type mappings defined in `PropertyKind`
- **Extensibility Support**: Full support for custom and experimental
  properties. Per RFC 5545, unknown content does not cause parsing to fail
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data with
  dedicated wrapper types for each property (e.g., `Created`, `DtStart`, `Summary`)
- **Performance**: Zero-copy parsing where possible, minimal allocations
- **Optional datetime dependencies**: All types use the value module's
  `ValueDate`, `ValueTime`, and `ValueDateTime` instead of directly using
  datetime types from `jiff` or `chrono` (planned)

## Error Handling

The architecture provides comprehensive error reporting with:

- Source location information for all errors
- Detailed error messages explaining RFC 5545 violations
- Phase-specific error categorization (syntax vs. validation)

## Feature Support

- **Property Kind System**: Complete `PropertyKind` enum with value type mappings
  for all RFC 5545 properties, enabling compile-time type safety
- **Unified Property Enum**: Single `Property` enum providing type-safe access to
  all property variants with `TryFrom<ParsedProperty>` validation
- **Unknown/Custom Property Support**: Full RFC 5545 compliance for extensibility:
  - Parsing never fails due to unknown content (per RFC 5545 Section 4.1)
  - Preserves original data for round-trip serialization compatibility
- **Timezone Validation**: Optional integration with `jiff` for timezone
  database validation (feature-gated)
- **Extensible Property Support**: Property types organized by RFC 5545 sections
  for easy maintenance and extension
- **RFC 5545 Compliance**: Complete support for all required value types and
  parameters
- **Semantic Type System**: High-level semantic representations of iCalendar
  components
