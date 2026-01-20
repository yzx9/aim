# CalDAV Client TODO

## Completed ✅

### Core Implementation

- [x] Create crate structure and workspace integration
- [x] Implement core types (Href, ETag, CalendarResource, CalendarCollection)
- [x] Implement error types with thiserror
- [x] Implement config and auth types (None, Basic, Bearer)
- [x] Implement HTTP client wrapper with reqwest
- [x] Implement XML utilities with quick-xml
- [x] Implement request builders (PropFindRequest, CalendarQueryRequest, CalendarMultiGetRequest, FreeBusyQueryRequest)
- [x] Implement response parsers (MultiStatusResponse with namespace handling)
- [x] Implement main CalDavClient with all RFC 4791 methods
- [x] Add support for self-closing XML elements (`Event::Empty`)
- [x] Full test suite (34 tests passing).

### CalDAV Operations (RFC 4791)

- [x] Calendar discovery (`discover()`)
- [x] List calendars (`list_calendars()`)
- [x] Get event and todos (`get_event`, `get_todo`)
- [x] Create event and todos (`create_event`, `create_todo`)
- [x] Update event and todos with ETag (`update_event`, `update_todo`)
- [x] Delete event and todos with ETag (`delete_event`, `delete_todo`)
- [x] Free-busy query (`free_busy()`)
- [x] Calendar query with filters (`query()`)
  - [x] Todo query helpers (`get_pending_todos`, `get_completed_todos`, `query_todos`, `get_todos_by_date_range`)
- [x] Calendar multiget (`multiget()`)
- [x] MKCALENDAR (`mkcalendar()`)
- [x] Basic authentication support

## Future Work 📋

### High Priority

- [ ] Bearer token authentication support
- [ ] Digest authentication support
- [ ] OAuth2 integration
- [ ] Sync state management with ETag tracking
- [ ] Conflict resolution strategies
- [ ] Retry logic with exponential backoff
- [ ] Connection pooling and keep-alive

### Medium Priority

- [ ] WebDAV sync collection support (RFC 6578)
- [ ] CalDAV scheduling (iTip/iMIP)
- [ ] Calendar sharing/ACL support
- [ ] Attendee management
- [ ] Recurring event expansion
- [ ] Timezone database integration
- [ ] Batch operations support
- [ ] Progress callbacks for long operations

### Low Priority

- [ ] CalDAV extensions support:
  - [ ] Calendar auto-provisioning (RFC 7641)
  - [ ] Calendar color (RFC 7909)
  - [ ] Managed attachments (RFC 8607)
  - [ ] Calendar availability (RFC 6638)
- [ ] Server capabilities detection
- [ ] HTTP/2 and HTTP/3 support
- [ ] Custom property filtering
- [ ] Query performance optimization
- [ ] Caching layer

### Documentation

- [ ] Add module-level documentation
- [ ] Add example code snippets
- [ ] Add integration guide
- [ ] Add error handling guide
- [ ] Add performance tuning guide

### Testing

- [ ] Add performance benchmarks
- [ ] Add conformance tests against real CalDAV servers:
  - [ ] Google Calendar
  - [ ] Apple iCloud
  - [ ] Nextcloud/ownCloud
  - [ ] Radicale
  - [ ] Baikal
- [ ] Add fuzzing tests for XML parsing
- [ ] Add property-based tests

### Quality of Life

- [ ] Add debug logging support
- [ ] Add tracing instrumentation
- [ ] Improve error messages with context
- [ ] Add CLI for testing CalDAV operations
- [ ] Add example applications

## Notes

### Implementation Decisions

- XML namespace handling: Use `local_name().into_inner()` to strip namespace prefixes
- Empty elements: Handle both `Event::Start` + `Event::End` and `Event::Empty`
- iCalendar format: Use CRLF line endings (`\r\n`) for RFC 5545 compliance
- ETag format: Preserve quotes from server responses (e.g., `"abc123"`)
- Authentication: Headers injected via HTTP client wrapper
- Todo overlap logic: Implement full RFC 4791 §9.9 table with all 8 property combinations
- Todo helpers: Use real iCalendar strings in tests (parse-based approach) for better maintainability
- Todo methods: Convenience aliases that delegate to existing event methods (non-breaking API)

### Known Limitations

- Sync state module exists but is not integrated
- Some error types lack detailed context
- No automatic retry for transient failures
- Limited to basic CalDAV operations (no scheduling/sharing yet)
- Todo overlap logic uses client-side filtering after server query (RFC 4791 §9.9 compliance)

### Dependencies

- `aimcal-ical` - iCalendar parsing and formatting
- `jiff` - Modern datetime and timezone library
- `reqwest` - HTTP client
- `quick-xml` - XML parsing and generation
- `thiserror` - Error handling
- `wiremock` - HTTP mocking for tests
