# iCal module architecture

The iCal module provides a comprehensive parser and formatter for the iCalendar format (RFC 5545)
using a multi-phase analysis approach. The architecture separates concerns through distinct layers
for syntax parsing, typed analysis and semantic validation.

## Architecture overview

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

## String storage abstraction

The parser uses a generic storage parameter system built on the `StringStorage` trait to enable
both zero-copy parsing and owned data representations with a unified API.

**Implementations:**

- `Segments<'src>` - For zero-copy borrowed segments with position information
- `String` - For owned string data (after calling `.to_owned()`)

This abstraction enables the entire type system to use generic bounds like `S: StringStorage`
instead of being tied to specific string types, providing flexibility while maintaining type safety.

Types use `Segments<'src>` for zero-copy borrowed data and `String` for owned data.
Conversion between the two is done through the `to_owned()` method on each type.

## Dependencies

- logos: Lexer generation
- chumsky: Parser combinators
- lexical: Numeric parsing
- thiserror: Error handling
- jiff (optional): Datetime and timezone validation

## Crate features

- **jiff** (default) - Datetime integration. ALWAYS add feature condition for jiff use.

## Design principles

- **Phase Separation**: Each parsing phase has clear responsibilities and well-defined interfaces
- **RFC 5545 Compliance**: Complete support for all required value types and parameters
- **RFC 5545 Serialization**: Complete formatter module for writing iCalendar data
- **Error Aggregation**: Collects and reports errors from all phases
- **Type Safety**: Strongly typed representation of iCalendar data with dedicated wrapper types
  for each property (e.g., `Created`, `DtStart`, `Summary`)
- **Performance**: Zero-copy parsing where possible, minimal allocations

## Error handling

The architecture provides comprehensive error reporting with:

- Source location information for all errors
- Detailed error messages explaining RFC 5545 violations
- Phase-specific error categorization (syntax vs. validation)

## Feature support

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
