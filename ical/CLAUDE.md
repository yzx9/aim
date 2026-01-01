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
nesting (BEGIN/END matching), and processes escape sequences.

### Typed Analysis Phase

Validates all components against RFC 5545 specifications through three sub-passes:

1. **Parameter Pass**
   - Parses and validates iCalendar parameters per RFC 5545 Section 3.2
   - Converts parameter strings to strongly-typed representations
   - Validates parameter values (e.g., enum values for CUTYPE, ENCODING, etc.)
   - Provides `Parameter` and `ParameterKind` types

2. **Value Pass**
   - Parses and validates property value types per RFC 5545 Section 3.3
   - Converts value strings to appropriate Rust types (dates, durations, integers, etc.)
   - Handles type inference when VALUE parameter is not specified
   - Provides `Value` enum and specific value types (`ValueDate`, `ValueDateTime`, etc.)

3. **Property Pass**
   - Will validate property-specific constraints and relationships
   - Will handle property cardinality and multiplicity rules
   - Will validate inter-property dependencies

**Note**: Property type definitions (e.g., `Attendee`, `DateTime`, `Geo`) are organized
in the `property/` module by RFC 5545 sections for better code organization and
maintainability.

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
│   ├── parameter/          # Parameter pass implementation
│   │   ├── ast.rs          # Parameter definitions and parsing
│   │   └── definition.rs   # Parameter type enums
│   ├── property/           # Property types organized by RFC 5545 sections
│   │   ├── kind.rs         # Property kinds (PropertyKind)
│   │   ├── alarm.rs        # Section 3.8.6 - Alarm properties (Action, Trigger)
│   │   ├── cal.rs          # Section 3.7 - Calendar properties (CalendarScale, Method, etc.)
│   │   ├── datetime.rs     # Section 3.8.2 - Date/time properties (DateTime, Period, Time)
│   │   ├── descriptive.rs  # Section 3.8.1 - Descriptive properties (Attachment, Geo, etc.)
│   │   ├── relationship.rs # Section 3.8.4 - Relationship properties (Attendee, Organizer)
│   │   ├── status.rs       # Section 3.8.1.11 - Status properties (EventStatus, etc.)
│   │   ├── timezone.rs     # Section 3.8.3 - Time zone properties (TimeZoneOffset)
│   │   └── transp.rs       # Section 3.8.2.7 - Time transparency property
│   ├── value/              # Value pass implementation
│   │   ├── ast.rs          # Value enum and parsing
│   │   ├── datetime.rs     # Date/time value types
│   │   ├── duration.rs     # Duration value type
│   │   ├── numeric.rs      # Numeric value types
│   │   ├── period.rs       # Period value type
│   │   ├── rrule.rs        # Recurrence rule type
│   │   └── text.rs         # Text value type
│   └── semantic/           # Semantic module declaration
│       ├── analysis.rs     # Main semantic coordinator
│       ├── icalendar.rs    # ICalendar root component
│       ├── property_util.rs    # Property parsing helper functions
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

**Features:**

- `jiff` (default) - Datetime integration

## Design Principles

- **Phase Separation**: Each parsing phase has clear responsibilities and well-
  defined interfaces
- **Three-Pass Typed Analysis**: The typed analysis phase is split into three
  independent passes for better modularity and maintainability:
  - **Parameter Pass** handles all parameter-related parsing and validation
  - **Value Pass** handles all value type parsing and validation
  - **Property Pass** (planned) will handle property-level constraints
- **RFC 5545 Compliance**: Comprehensive validation against the iCalendar
  specification
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data
- **Performance**: Zero-copy parsing where possible, minimal allocations
- **Extensibility**: Modular design allows for easy addition of new features
- **Optional datetime dependencies**: All types use the value module's
  `ValueDate`, `ValueTime`, and `ValueDateTime` instead of directly using
  datetime types from `jiff` or `chrono` (planned)

## Error Handling

The architecture provides comprehensive error reporting with:

- Source location information for all errors
- Detailed error messages explaining RFC 5545 violations
- Phase-specific error categorization (syntax vs. validation)

## Feature Support

- **Timezone Validation**: Optional integration with `jiff` for timezone
  database validation (feature-gated)
- **Extensible Property Support**: Registry-based property specifications
- **RFC 5545 Compliance**: Complete support for all required value types and
  parameters
- **Semantic Type System**: High-level semantic representations of iCalendar
  components
