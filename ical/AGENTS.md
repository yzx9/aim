# iCal Module Architecture

The iCal module provides a comprehensive parser and formatter for the iCalendar format (RFC 5545)
using a multi-phase analysis approach. The architecture separates concerns through distinct layers
for syntax parsing, typed analysis and semantic validation.

## Architecture Overview

The parser follows a **four-phase pipeline**:

1. **Syntax Analysis** - Tokenizes raw iCalendar text and builds component structure
   1. Lexer: Tokenizes raw iCalendar text into structured tokens while preserving source position
      information for error reporting
   2. Scanner: Scans token streams into content lines, validating structure and collecting errors
   3. Tree Builder: Builds a tree of components with properties and parameters using a stack-based
      algorithm, validates component nesting (BEGIN/END matching)
2. **Typed Analysis** - Validates and converts components to strongly-typed representations through
   three sub-passes
   1. Parameter Pass: Parses and validates iCalendar parameters into strongly-typed representations.
   2. Value Pass: Parses and validates property value types, handling type inference and escape
      sequences.
   3. Property Pass: Validates property-specific constraints and creates strongly-typed wrapper
      types for each property.
3. **Semantic Analysis** - Validates RFC 5545 semantics (required properties, constraints,
   relationships)
4. **Unifyied Parser** - main `parse()` function coordinates all phases
5. **Formatter** - Format icalendar to RFC 5545

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
├── RFC5545.txt # Internet Calendaring and Scheduling Core Object Specification (iCalendar)
├── src/
│   ├── lib.rs              # Public API exports
│   ├── keyword.rs          # RFC 5545 keyword constants
│   ├── string_storage.rs   # String storage abstraction
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
- **thiserror** - Error handling
- **jiff** (optional) - Datetime and timezone validation

## Features

- **jiff** (default) - Datetime integration

## Design Principles

- **Phase Separation**: Each parsing phase has clear responsibilities and well-defined interfaces
- **RFC 5545 Compliance**: Complete support for all required value types and parameters
- **RFC 5545 Serialization**: Complete formatter module for writing iCalendar data
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data with dedicated wrapper types
  for each property (e.g., `Created`, `DtStart`, `Summary`)
- **Performance**: Zero-copy parsing where possible, minimal allocations

## Error Handling

The architecture provides comprehensive error reporting with:

- Source location information for all errors
- Detailed error messages explaining RFC 5545 violations
- Phase-specific error categorization (syntax vs. validation)

## Feature Support

- **Generic Storage Parameter System**: Unified type system using generic storage parameter
  `S: StringStorage` for flexibility. This enables both zero-copy parsing (borrowed data
  `Type<Segments<'src>>`) and owned data (`Type<String>`) representations
- **Unknown/Custom Property Support**: Full RFC 5545 compliance for extensibility:
  - Parsing never fails due to unknown content (per RFC 5545 Section 4.1)
  - Preserves original data for round-trip serialization compatibility
- **Semantic Type System**: High-level semantic representations of iCalendar components using
  `Type<Segments<'src>>` for borrowed data and `Type<String>` for owned data
- **RFC 5545 Compliance**: Comprehensive validation and serialization against the iCalendar
  specification with property-to-value-type mappings defined in `PropertyKind`
- **Bidirectional Support**: Complete parser and formatter for read/write operations:
  - Parse iCalendar data into strongly-typed representations
  - Serialize components, properties, parameters, and values back to RFC 5545 format
  - Zero-copy parameter writer functions for efficient serialization
- **(feature-gated) Timezone Validation**: Optional integration with `jiff` for timezone database
  validation
