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
   representations
4. **Semantic Analysis** - Validates RFC 5545 semantics (required properties,
   constraints, relationships)

### Lexer Phase

Transforms raw iCalendar text into tokens while preserving source position
information for error reporting.

### Syntax Phase

Builds a tree of components with properties and parameters, validates component
nesting (BEGIN/END matching), and processes escape sequences.

### Typed Analysis Phase

Validates all components against RFC 5545 specifications, converts string
values to appropriate Rust types, and enforces property multiplicity and
parameter constraints.

### Semantic Analysis Phase

Performs RFC 5545 semantic validation to ensure iCalendar data is logically
correct and complete. This phase goes beyond syntax checking to validate
business rules and constraints defined in the specification.

### Unified Parser

Coordinates all phases, aggregates errors from each phase, and provides a
single entry point for parsing operations.

## Module Structure

```
ical
├── Cargo.toml
├── CLAUDE.toml
├── RFC5545.txt     # If you have questions, check the RFC first and use search—it's very long
└── src/
    ├── lib.rs      # Public API exports
    ├── keyword.rs  # RFC 5545 keyword constants
    ├── lexer.rs    # Lexical analysis phase
    ├── syntax.rs   # Syntax analysis phase
    ├── parser.rs   # Unified parser orchestration
    └── typed/      # Typed analysis phase
        ├── mod.rs  # Public API and re-exports
        ├── analysis.rs   # Main typed analysis coordinator
        ├── property_spec.rs   # RFC 5545 property specifications
        ├── parameter_types.rs # Parameter type definitions
        ├── value.rs      # Value type implementations
        ├── value_datetime.rs  # Date/time value handling
        ├── value_duration.rs  # Duration value handling
        ├── value_numeric.rs   # Numeric value handling
        └── value_text.rs      # Text value handling
```

## Design Principles

- **Phase Separation**: Each parsing phase has clear responsibilities and well-
  defined interfaces
- **RFC 5545 Compliance**: Comprehensive validation against the iCalendar
  specification
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data
- **Performance**: Zero-copy parsing where possible, minimal allocations
- **Extensibility**: Modular design allows for easy addition of new features
- **Optional datetime dependencies**: All types use the typed module's
  `ValueDate`, `ValueTime`, and `ValueDateTime` instead of directly using
  datetime types from `jiff` or `chorno` (planned)

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
