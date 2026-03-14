# CalDAV Client Module Architecture

The CalDAV client module provides a comprehensive implementation of the CalDAV protocol (RFC 4791)
for accessing and managing calendars on CalDAV servers. The architecture provides a clean separation
between HTTP operations, XML parsing, and client-level functionality.

## Architecture Overview

The client follows a **layered architecture** with clear separation of concerns:

1. **HTTP Layer** (`http.rs`) - HTTP client wrapper with authentication and ETag handling
   - Manages HTTP connections with reqwest
   - Handles authentication (Basic, Bearer token, None)
   - Provides ETag extraction and conditional request headers
   - Executes requests and handles HTTP errors

2. **Request Builders** (`request.rs`) - Type-safe builders for CalDAV request bodies
   - PROPFIND requests for property queries
   - REPORT requests for calendar queries and free-busy
   - Calendar multiget for batch operations
   - XML serialization following RFC 4791

3. **Response Parsers** (`response.rs`) - Parses WebDAV/CalDAV XML responses
   - Multistatus response parsing
   - Property extraction and validation
   - Conversion to domain types (resources, collections)

4. **Client Layer** (`client.rs`) - High-level CalDAV operations
   - Calendar discovery and listing
   - Event CRUD operations (create, read, update, delete)
   - Calendar query with filters
   - Free-busy queries

5. **Synchronization** (`sync.rs`) - Two-way sync utilities (stub implementation)
   - Change detection with ETags and CTAGs
   - Local state management

## Design Principles

- **Protocol Compliance**: Full RFC 4791 CalDAV and RFC 4918 WebDAV support
- **Type Safety**: Strongly-typed representations for all CalDAV concepts
- **Error Handling**: Comprehensive error types with context
- **Extensibility**: Support for custom properties and X- names
- **Testability**: Uses wiremock for integration tests without network dependencies
- **Async/Await**: Full async support with tokio

## RFC Compliance

### RFC 4791 - CalDAV

- **Calendar Access**: Read and write calendar data
- **Calendar Queries**: Time-range and component-type filters
- **Multiget**: Batch operations for efficiency
- **Free-Busy**: Query availability information
- **ETag Support**: Optimistic concurrency control

### RFC 4918 - WebDAV

- **PROPFIND**: Property queries
- **MKCALENDAR**: Calendar collection creation
- **REPORT**: Extended query methods

### RFC 5545 - iCalendar

- Full integration with `aimcal-ical` for parsing and formatting
- Preserves unknown properties for round-trip compatibility

## Testing Strategy

### Unit Tests

- **Request builders**: Verify XML serialization
- **Response parsers**: Test XML parsing with various server responses

### Integration Tests

- **Client operations**: Full CRUD workflow with wiremock
- **Authentication**: Basic and bearer token auth headers
- **Error handling**: ETag mismatches, 404 responses, etc.

All tests use wiremock for local HTTP mocking - no real network access required.

## Future Enhancements

- **Synchronization**: Complete two-way sync implementation
- **WebDAV ACL**: Permission and access control
- **Calendar Sharing**: Share and unshare operations
- **Sync Token**: Efficient change detection with sync-token REPORT
