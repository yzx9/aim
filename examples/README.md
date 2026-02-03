# AIM Example iCalendar Files

This directory contains example iCalendar (.ics) files to demonstrate various features of the AIM application and the iCalendar (RFC 5545) format.

## Example Files

### simple-meeting.ics
A basic one-time meeting event demonstrating:
- Event creation with summary and description
- Location specification
- Organizer information
- Status tracking (CONFIRMED)
- Date/time in UTC format

### recurring-task.ics
A daily recurring event demonstrating:
- RRULE (Recurrence Rule) for daily repetition
- COUNT parameter to limit occurrences
- Priority setting
- Status tracking (TENTATIVE)

### todo-with-priority.ics
A todo item demonstrating:
- Priority levels (2 = High)
- Due date and start date
- Status tracking (NEEDS-ACTION)
- Progress tracking with PERCENT-COMPLETE
- Categorization

### multi-event.ics
A complex calendar with multiple components demonstrating:
- Multiple VEVENT components in one calendar
- Multiple VTODO components
- Timezone definitions (VTIMEZONE)
- Daylight saving time rules (DAYLIGHT)
- Standard time rules (STANDARD)
- Attendees with ATTENDEE
- Organizer information
- Multiple categories

## Usage Examples

### Initialize Development Calendar

To quickly populate your development calendar with all example files:

```bash
just init-examples
```

This command:
- Creates `.dev-calendar/` directory if it doesn't exist
- Copies all example files (`.ics`) to `.dev-calendar/`
- Creates a marker file `.dev-calendar/.dev-marker` to track initialization
- Is idempotent - safe to run multiple times

After running `just init-examples`, the first `aim` command will automatically load all examples into the development database.

### Load with AIM

```bash
# Copy an example to your calendar directory
cp examples/simple-meeting.ics ~/.local/share/aim/calendar/

# Or use AIM CLI to list events
aim event list

# Or use TUI to interact
aim new
```

### Import to Calendar Applications

These files can be imported into any iCalendar-compatible application:
- Google Calendar
- Apple Calendar
- Microsoft Outlook
- Mozilla Thunderbird
- Evolution

### Testing with AIM

```bash
# Test parsing and formatting
cargo test -p aimcal-ical

# Test with CLI
AIM_CONFIG=cli/config.dev.toml cargo run -- event list

# Create a test calendar directory
mkdir -p .dev-calendar
cp examples/*.ics .dev-calendar/
```

## iCalendar (RFC 5545) Resources

- [RFC 5545 Specification](https://icalendar.org/RFC-Specifications/iCalendar-RFC-5545/)
- [iCalendar.org](https://icalendar.org/) - Official iCalendar website

## Common Properties Reference

### Event Properties (VEVENT)
- `DTSTART` - Event start date/time
- `DTEND` - Event end date/time
- `SUMMARY` - Event title
- `DESCRIPTION` - Detailed description
- `LOCATION` - Location name
- `STATUS` - CONFIRMED, TENTATIVE, CANCELLED
- `PRIORITY` - 1-9 (1 is highest)
- `RRULE` - Recurrence rule

### Todo Properties (VTODO)
- `DTSTART` - Start date/time
- `DUE` - Due date/time
- `STATUS` - NEEDS-ACTION, IN-PROCESS, COMPLETED, CANCELLED
- `PERCENT-COMPLETE` - 0-100 completion percentage
- `PRIORITY` - 1-9 (1 is highest)

### Timezone Properties (VTIMEZONE)
- `TZID` - Timezone identifier (e.g., America/New_York)
- `DAYLIGHT` - Daylight saving time definition
- `STANDARD` - Standard time definition
