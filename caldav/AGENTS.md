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

## Module Structure

```
caldav/
├── Cargo.toml
├── CLAUDE.md
├── RFC4791.txt # Calendaring Extensions to WebDAV (CalDAV)
├── RFC4918.txt # HTTP Extensions for Web Distributed Authoring and Versioning (WebDAV)
├── TODO.md     # Check current plan
├── src/
│   ├── lib.rs      # Public API exports
│   ├── client.rs   # Main CalDavClient with CRUD operations
│   ├── config.rs   # Configuration (AuthMethod, CalDavConfig)
│   ├── error.rs    # Error types and conversions
│   ├── http.rs     # HTTP client wrapper
│   ├── request.rs  # Request builders (PropFind, CalendarQuery, etc.)
│   ├── response.rs # Response parsers (MultiStatus)
│   ├── sync.rs     # Synchronization utilities
│   ├── types.rs    # Domain types (Href, ETag, CalendarResource, CalendarCollection)
│   └── xml.rs      # XML namespaces and utilities
└── tests/
    ├── client.rs   # Client integration tests (with wiremock)
    ├── request.rs  # Request building tests
    └── response.rs # Response parsing tests
```

## Dependencies

- **reqwest** - HTTP client
- **quick-xml** - XML parsing and writing
- **thiserror** - Error handling
- **serde** - Configuration deserialization
- **aimcal-ical** - iCalendar parsing and formatting
- **wiremock** (dev) - HTTP mocking for tests

## Features

No feature flags. All functionality is available by default.

## Design Principles

- **Protocol Compliance**: Full RFC 4791 CalDAV and RFC 4918 WebDAV support
- **Type Safety**: Strongly-typed representations for all CalDAV concepts
- **Error Handling**: Comprehensive error types with context
- **Extensibility**: Support for custom properties and X- names
- **Testability**: Uses wiremock for integration tests without network dependencies
- **Async/Await**: Full async support with tokio

## Error Handling

The architecture provides comprehensive error reporting through `CalDavError`:

- **HTTP errors**: Network and protocol-level failures
- **XML errors**: Parsing and serialization issues
- **iCalendar errors**: Parsing failures from calendar data
- **Authentication errors**: Credential and authorization failures
- **Not found errors**: Missing resources
- **Precondition failures**: ETag mismatches for optimistic concurrency
- **Server capability errors**: Server doesn't support CalDAV
- **Response validation**: Invalid server responses
- **Configuration errors**: Invalid client configuration

## API Components

### Configuration

- **AuthMethod**: Authentication strategies (None, Basic, Bearer)
- **CalDavConfig**: Server connection settings (base_url, calendar_home, auth, timeout)

### Domain Types

- **Href**: Calendar resource paths (e.g., `/calendars/user/event1.ics`)
- **ETag**: Entity tags for change detection and optimistic concurrency
- **CalendarResource**: Calendar objects with href, ETag, and iCalendar data
- **CalendarCollection**: Calendar metadata (display name, description, components)

### Request Builders

- **PropFindRequest**: Query properties (DisplayName, ResourceType, CalendarData, etc.)
- **CalendarQueryRequest**: Filter events by time range and component type
- **CalendarMultiGetRequest**: Batch retrieve multiple calendar objects
- **FreeBusyQueryRequest**: Query free/busy information

### Client Operations

- **discover()**: Detect CalDAV support and find calendar home set
- **list_calendars()**: Get all calendar collections
- **mkcalendar()**: Create a new calendar collection
- **get_event()**: Retrieve a single calendar object
- **create_event()**: Create a new calendar object
- **update_event()**: Update an existing calendar object (with ETag check)
- **delete_event()**: Delete a calendar object (with ETag check)
- **query()**: Search calendar objects with filters
- **multiget()**: Batch retrieve calendar objects
- **free_busy()**: Get free/busy information

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
