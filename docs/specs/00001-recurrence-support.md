# Recurring Event/Todo Support Implementation Plan

## Overview

This document outlines the comprehensive plan for adding full recurring event and todo support to AIM, following RFC 5545 standards with on-the-fly expansion and exception handling.

## Architecture Decision

**Mixed Approach:**

- **ical crate**: Provides pure recurrence algorithms (`RecurrenceIterator`)
- **core crate**: Provides application-layer expansion logic (`EventExpander`)

This balances reusability with business logic control, keeping ical complete while allowing core to optimize for AIM's needs.

## Phase 1: Core Recurrence Expansion Engine (aimcal-ical)

### 1.1 Recurrence Iterator Module

**New file**: `ical/src/recurrence.rs`

```rust
/// Iterator that generates occurrences from an RRULE
pub struct RecurrenceIterator<'a> {
    rule: &'a ValueRecurrenceRule,
    base_start: Zoned,
    current: Zoned,
    count: u32,
}

impl<'a> RecurrenceIterator<'a> {
    pub fn new(rule: &'a ValueRecurrenceRule, base_start: Zoned) -> Self;

    /// Generate occurrences within a date range
    pub fn within_range(&mut self, range: Range<Zoned>) -> Vec<Zoned>;
}

impl<'a> Iterator for RecurrenceIterator<'a> {
    type Item = Zoned;
    fn next(&mut self) -> Option<Self::Item>;
}
```

**Features:**

- Full RFC 5545 RRULE support (FREQ, INTERVAL, UNTIL, COUNT, BY\* rules)
- Timezone-aware calculations using jiff
- Stateful iteration with proper bounds checking
- Optimized for lazy evaluation

**Testing:**

- Unit tests for each recurrence pattern
- Edge cases: leap years, DST transitions, month/year boundaries
- RFC 5545 test suite compliance

### 1.2 Library Integration

**Modify**: `ical/src/lib.rs`

```rust
pub mod recurrence;

pub use crate::recurrence::{RecurrenceIterator, occurrences_between};
```

---

## Phase 2: Core Library Extensions (aimcal-core)

### 2.1 Recurrence Types

**New file**: `core/src/recurrence.rs`

```rust
/// Recurrence rule representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecurrenceRule {
    pub freq: RecurrenceFrequency,
    pub interval: Option<u32>,
    pub until: Option<Zoned>,
    pub count: Option<u32>,
    pub by_day: Option<Vec<WeekDay>>,
    pub by_month: Option<Vec<u8>>,
    pub by_month_day: Option<Vec<i8>>,
    pub by_year_day: Option<Vec<i16>>,
    pub by_week_no: Option<Vec<i8>>,
    pub by_hour: Option<Vec<u8>>,
    pub by_minute: Option<Vec<u8>>,
    pub by_second: Option<Vec<u8>>,
    pub by_set_pos: Option<Vec<i16>>,
    pub wkst: Option<WeekDay>,
}

/// Set of recurrence information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecurrenceSet {
    pub rule: Option<RecurrenceRule>,
    pub rdates: Vec<Zoned>,
    pub ex_dates: Vec<Zoned>,
}

impl RecurrenceRule {
    /// Parse from RRULE string
    pub fn from_str(s: &str) -> Result<Self, ParseError>;

    /// Serialize to RRULE string
    pub fn to_string(&self) -> String;
}

impl TryFrom<&ValueRecurrenceRule> for RecurrenceRule { }
impl TryFrom<RecurrenceRule> for ValueRecurrenceRule { }
```

### 2.2 Extended Event Drafts

**Modify**: `core/src/event.rs`

```rust
pub struct EventDraft {
    // ... existing fields
    pub recurrence: Option<RecurrenceRule>,
    pub rdates: Vec<Zoned>,
    pub ex_dates: Vec<Zoned>,
}

pub struct ResolvedEventDraft<'a> {
    // ... existing fields
    pub recurrence: Option<RecurrenceRule>,
    pub rdates: Vec<Zoned>,
    pub ex_dates: Vec<Zoned>,
}

impl ResolvedEventDraft<'_> {
    pub fn into_ics(self, uid: &str) -> VEvent<String> {
        VEvent {
            // ... existing fields
            rrule: self.recurrence.map(|r| RRule::new(r.into())),
            rdates: self.rdates.into_iter().map(RDate::new).collect(),
            ex_dates: self.ex_dates.into_iter().map(ExDate::new).collect(),
        }
    }
}
```

**Modify**: `core/src/todo.rs` (similar changes for VTodo)

### 2.3 Expanded Event Types

**New file**: `core/src/expanded.rs`

```rust
/// Expanded event with recurrence information
#[derive(Debug, Clone)]
pub struct ExpandedEvent {
    /// The base event (or exception if this is an exception occurrence)
    pub event: VEvent<String>,
    /// The specific occurrence date/time
    pub occurrence_start: Zoned,
    pub occurrence_end: Zoned,
    /// Whether this is an exception occurrence
    pub is_exception: bool,
    /// Original recurrence ID if this is an exception
    pub recurrence_id: Option<Zoned>,
}

impl ExpandedEvent {
    pub fn start(&self) -> LooseDateTime { }
    pub fn end(&self) -> Option<LooseDateTime> { }
    pub fn summary(&self) -> Cow<'_, str> { }
    pub fn description(&self) -> Option<Cow<'_, str>> { }
    pub fn status(&self) -> Option<EventStatus> { }
    pub fn uid(&self) -> Cow<'_, str> { }
}

impl Event for ExpandedEvent { }
```

### 2.4 Occurrence Query API

**Modify**: `core/src/aim.rs`

```rust
impl Aim {
    /// List events within a date range, expanding recurrences on-the-fly
    pub async fn list_events_expanded(
        &self,
        conds: &ResolvedEventConditions,
        pager: &Pager,
    ) -> Result<Vec<ExpandedEvent>>;

    /// Get a specific occurrence (handles base event or exception)
    pub async fn get_event_occurrence(
        &self,
        uid: &str,
        recurrence_id: Option<Zoned>,
    ) -> Result<VEvent<String>>;

    /// Update a specific occurrence (creates exception)
    pub async fn update_occurrence(
        &mut self,
        uid: &str,
        recurrence_id: Zoned,
        patch: EventPatch,
    ) -> Result<VEvent<String>>;

    /// Delete a specific occurrence (adds to EXDATE)
    pub async fn delete_occurrence(
        &mut self,
        uid: &str,
        recurrence_id: Zoned,
    ) -> Result<()>;
}
```

### 2.5 Expansion Logic

**New file**: `core/src/expander.rs`

```rust
/// Expander for recurring events
pub struct EventExpander<'a> {
    base_event: &'a VEvent<String>,
    exceptions: Vec<&'a VEvent<String>>,
}

impl<'a> EventExpander<'a> {
    pub fn new(base_event: &'a VEvent<String>) -> Self {
        Self {
            base_event,
            exceptions: Vec::new(),
        }
    }

    pub fn with_exceptions(mut self, exceptions: Vec<&'a VEvent<String>>) -> Self {
        self.exceptions = exceptions;
        self
    }

    /// Expand event to specified date range
    pub fn expand(
        &self,
        range: Range<Zoned>,
    ) -> Result<Vec<ExpandedEvent>> {
        // 1. Get base occurrences from RRULE (using ical's RecurrenceIterator)
        let mut occurrences = if let Some(rrule) = &self.base_event.rrule {
            let base_start = self.base_event.dt_start.to_zoned();
            ical::recurrence::occurrences_between(&rrule.value, base_start, range.clone())
        } else {
            vec![]
        };

        // 2. Add RDATE occurrences
        for rdate in &self.base_event.rdates {
            occurrences.extend(/* parse and add RDATEs */);
        }

        // 3. Remove EXDATE occurrences
        occurrences.retain(|dt| {
            !self.base_event.ex_dates.iter().any(|ex| {
                // Compare datetime values
            })
        });

        // 4. Apply exceptions (RECURRENCE-ID)
        let mut expanded_events: Vec<ExpandedEvent> = occurrences
            .into_iter()
            .map(|dt| {
                // Check if there's an exception for this occurrence
                if let Some(exception) = self.exceptions.iter().find(|ex| {
                    ex.recurrence_id.as_ref()
                        .map(|rid| rid == &dt)
                        .unwrap_or(false)
                }) {
                    ExpandedEvent {
                        event: (*exception).clone(),
                        occurrence_start: dt,
                        occurrence_end: /* calculate based on duration */,
                        is_exception: true,
                        recurrence_id: Some(dt),
                    }
                } else {
                    // Create expanded event from base
                    ExpandedEvent {
                        event: self.base_event.clone(),
                        occurrence_start: dt,
                        occurrence_end: /* calculate based on duration */,
                        is_exception: false,
                        recurrence_id: None,
                    }
                }
            })
            .collect();

        // 5. Sort by occurrence start time
        expanded_events.sort_by(|a, b| a.occurrence_start.cmp(&b.occurrence_start));

        Ok(expanded_events)
    }
}
```

### 2.6 Library Integration

**Modify**: `core/src/lib.rs`

```rust
mod recurrence;
mod expanded;
mod expander;

pub use crate::recurrence::{RecurrenceRule, RecurrenceSet};
pub use crate::expanded::{ExpandedEvent, ExpandedTodo};
pub use crate::expander::EventExpander;
```

---

## Phase 3: Database Schema Updates

### 3.1 Schema Migration

**New file**: `core/src/db/migrations/YYYYMMDD_add_recurrence.sql`

```sql
-- Add recurrence columns to events table
ALTER TABLE events ADD COLUMN rrule TEXT;
ALTER TABLE events ADD COLUMN rdates TEXT;  -- JSON array of ISO 8601 strings
ALTER TABLE events ADD COLUMN ex_dates TEXT; -- JSON array of ISO 8601 strings

-- Add recurrence_id column for exception events
ALTER TABLE events ADD COLUMN recurrence_id TEXT; -- ISO 8601 datetime or NULL for base events

-- Add index for querying base events and their exceptions
CREATE INDEX idx_events_uid_recurrence ON events(uid, recurrence_id);

-- Add index for querying by recurrence_id
CREATE INDEX idx_events_recurrence_id ON events(recurrence_id) WHERE recurrence_id IS NOT NULL;

-- Similar changes for todos
ALTER TABLE todos ADD COLUMN rrule TEXT;
ALTER TABLE todos ADD COLUMN rdates TEXT;
ALTER TABLE todos ADD COLUMN ex_dates TEXT;
ALTER TABLE todos ADD COLUMN recurrence_id TEXT;
CREATE INDEX idx_todos_uid_recurrence ON todos(uid, recurrence_id);
CREATE INDEX idx_todos_recurrence_id ON todos(recurrence_id) WHERE recurrence_id IS NOT NULL;
```

### 3.2 Storage Logic

**Modify**: `core/src/db/events.rs`

```rust
impl EventRow {
    fn from_vevent(vevent: &VEvent<String>) -> Self {
        Self {
            // ... existing fields
            rrule: vevent.rrule.as_ref().map(|r| r.to_string()),
            rdates: if !vevent.rdates.is_empty() {
                Some(serde_json::to_string(&vevent.rdates).unwrap())
            } else {
                None
            },
            ex_dates: if !vevent.ex_dates.is_empty() {
                Some(serde_json::to_string(&vevent.ex_dates).unwrap())
            } else {
                None
            },
            recurrence_id: vevent.recurrence_id.as_ref().map(|rid| {
                rid.to_string()  // ISO 8601 format
            }),
        }
    }

    fn to_vevent(&self) -> VEvent<String> {
        // ... existing conversions
        rrule: self.rrule.as_ref().map(|s| {
            RRule::new(s.parse().unwrap())
        }),
        rdates: self.rdates.as_ref()
            .map(|s| serde_json::from_str(s).unwrap())
            .unwrap_or_default(),
        ex_dates: self.ex_dates.as_ref()
            .map(|s| serde_json::from_str(s).unwrap())
            .unwrap_or_default(),
        recurrence_id: self.recurrence_id.as_ref().map(|s| {
            RecurrenceId::new(s.parse().unwrap())
        }),
    }
}

impl EventRepo {
    /// Get base event and all its exceptions
    pub async fn get_with_exceptions(
        &self,
        uid: &str,
    ) -> Result<(VEvent<String>, Vec<VEvent<String>>)> {
        let base = self.get_by_uid(uid).await?;
        let exceptions = sqlx::query_as!(
            EventRow,
            "SELECT * FROM events WHERE uid = ? AND recurrence_id IS NOT NULL",
            uid
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| row.to_vevent())
        .collect();

        Ok((base, exceptions))
    }

    /// Save exception event with RECURRENCE-ID
    pub async fn save_exception(
        &self,
        base_uid: &str,
        exception: VEvent<String>,
        recurrence_id: Zoned,
    ) -> Result<()> {
        let mut row = EventRow::from_vevent(&exception);
        row.uid = base_uid.to_string();
        row.recurrence_id = Some(recurrence_id.to_string());

        // Use INSERT OR REPLACE to handle duplicates
        sqlx::query(r#"
            INSERT INTO events (uid, recurrence_id, data)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(uid, recurrence_id) DO UPDATE SET data = ?3
        "#)
        .bind(&row.uid)
        .bind(&row.recurrence_id)
        .bind(serde_json::to_string(&row).unwrap())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

**Similar changes for**: `core/src/db/todos.rs`

---

## Phase 4: CLI Extensions

### 4.1 Recurrence Argument Handling

**Modify**: `cli/src/arg.rs`

```rust
impl EventArgs {
    pub fn recurrence(self) -> Arg {
        arg!(--recurrence <RULE>)
            .help("Recurrence rule (e.g., 'FREQ=DAILY;INTERVAL=2' or 'FREQ=WEEKLY;BYDAY=MO,WE,FR')")
    }

    pub fn get_recurrence(matches: &ArgMatches) -> Option<String> {
        matches.get_one("recurrence").cloned()
    }

    pub fn rdate(self) -> Arg {
        arg!(--rdate <DATE>)
            .help("Additional recurrence date (ISO 8601, can be specified multiple times)")
            .action(ArgAction::Append)
    }

    pub fn get_rdates(matches: &ArgMatches) -> Vec<String> {
        matches.get_many("rdate")
            .map(|v| v.cloned().collect())
            .unwrap_or_default()
    }

    pub fn exdate(self) -> Arg {
        arg!(--exdate <DATE>)
            .help("Exception date (ISO 8601, can be specified multiple times)")
            .action(ArgAction::Append)
    }

    pub fn get_exdates(matches: &ArgMatches) -> Vec<String> {
        matches.get_many("exdate")
            .map(|v| v.cloned().collect())
            .unwrap_or_default()
    }
}
```

### 4.2 Enhanced Event Commands

**Modify**: `cli/src/cmd_event.rs`

```rust
pub struct CmdEventNew {
    // ... existing fields
    pub recurrence: Option<String>,
    pub rdates: Vec<String>,
    pub ex_dates: Vec<String>,
}

impl CmdEventNew {
    pub const NAME: &str = "new";

    pub fn command() -> Command {
        let (args, event_args) = args();
        Command::new(Self::NAME)
            .alias("add")
            .about("Add a new event")
            .arg(args.summary(true))
            .arg(event_args.start())
            .arg(event_args.end())
            .arg(args.description())
            .arg(event_args.status())
            .arg(event_args.recurrence())
            .arg(event_args.rdate())
            .arg(event_args.exdate())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            // ... existing fields
            recurrence: EventArgs::get_recurrence(matches),
            rdates: EventArgs::get_rdates(matches),
            ex_dates: EventArgs::get_exdates(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        // ... existing draft preparation
        let mut draft = aim.default_event_draft();

        // Parse recurrence rule if provided
        if let Some(recurrence_str) = self.recurrence {
            draft.recurrence = Some(RecurrenceRule::from_str(&recurrence_str)?);
        }

        // Parse RDATEs and EXDATEs
        for rdate_str in self.rdates {
            draft.rdates.push(parse_datetime(&now, &rdate_str)?);
        }
        for exdate_str in self.ex_dates {
            draft.ex_dates.push(parse_datetime(&now, &exdate_str)?);
        }

        // ... rest of existing logic
    }
}
```

### 4.3 Occurrence-Specific Commands

**New struct in `cli/src/cmd_event.rs`**:

```rust
pub struct CmdEventEditOccurrence {
    pub id: Id,
    pub recurrence_date: String,  // The specific occurrence date/time
    pub description: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub status: Option<EventStatus>,
    pub summary: Option<String>,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

impl CmdEventEditOccurrence {
    pub const NAME: &str = "edit-occurrence";

    pub fn command() -> Command {
        let args = EventOrTodoArgs::new(Some(Kind::Event));
        let event_args = EventArgs::new(false);
        Command::new(Self::NAME)
            .about("Edit a specific occurrence of a recurring event")
            .arg(args.id())
            .arg(Arg::new("recurrence_date")
                .help("Date/time of the occurrence to edit (ISO 8601)")
                .required(true))
            .arg(args.summary(false))
            .arg(event_args.start())
            .arg(event_args.end())
            .arg(args.description())
            .arg(event_args.status())
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            id: EventOrTodoArgs::get_id(matches),
            recurrence_date: matches.get_one("recurrence_date").unwrap().clone(),
            description: EventOrTodoArgs::get_description(matches),
            start: EventArgs::get_start(matches),
            end: EventArgs::get_end(matches),
            status: EventArgs::get_status(matches),
            summary: EventOrTodoArgs::get_summary(matches),
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
        }
    }

    pub async fn run(self, aim: &mut Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "editing event occurrence...");

        // Parse recurrence_id
        let recurrence_id = parse_datetime(&aim.now(), &self.recurrence_date)?;

        // Prepare the patch
        let (start, end) = match (self.start, self.end) {
            (Some(start), Some(end)) => {
                let (a, b) = parse_datetime_range(&aim.now(), &start, &end)?;
                (Some(a), Some(b))
            }
            (Some(start), None) => (Some(parse_datetime(&aim.now(), &start)?), None),
            (None, Some(end)) => (None, Some(parse_datetime(&aim.now(), &end)?)),
            (None, None) => (None, None),
        };

        let patch = EventPatch {
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            end,
            start,
            status: self.status,
            summary: self.summary,
        };

        // Update the occurrence (creates exception)
        let event = aim.update_occurrence(&self.id, recurrence_id, patch).await?;
        print_events(aim, &[event], self.output_format, self.verbose);
        Ok(())
    }
}
```

### 4.4 Enhanced List Command

**Modify**: `cli/src/cmd_event.rs`

```rust
pub struct CmdEventList {
    pub conds: EventConditions,
    pub output_format: OutputFormat,
    pub verbose: bool,
    pub expanded: bool,  // New: whether to expand recurrences
}

impl CmdEventList {
    pub const NAME: &str = "list";

    pub fn command() -> Command {
        Command::new(Self::NAME)
            .about("List events")
            .arg(arg!(--expanded "Expand recurring events into individual occurrences")
                 .action(ArgAction::SetTrue))
            .arg(CommonArgs::output_format())
            .arg(CommonArgs::verbose())
    }

    pub fn from(matches: &ArgMatches) -> Self {
        Self {
            conds: EventConditions {
                startable: Some(DateTimeAnchor::today()),
                ..Default::default()
            },
            output_format: CommonArgs::get_output_format(matches),
            verbose: CommonArgs::get_verbose(matches),
            expanded: matches.get_flag("expanded"),
        }
    }

    pub async fn run(self, aim: &Aim) -> Result<(), Box<dyn Error>> {
        tracing::debug!(?self, "listing events...");
        Self::list(aim, &self.conds, self.output_format, self.verbose, self.expanded).await
    }

    pub async fn list(
        aim: &Aim,
        conds: &EventConditions,
        output_format: OutputFormat,
        verbose: bool,
        expanded: bool,
    ) -> Result<(), Box<dyn Error>> {
        const LIMIT: i64 = 128;

        if expanded {
            // Use expanded listing
            let events = aim.list_events_expanded(conds, &(LIMIT, 0).into()).await?;
            print_events(aim, &events, output_format, verbose);
        } else {
            // Use non-expanded listing (existing behavior)
            let events = aim.list_events(conds, &(LIMIT, 0).into()).await?;
            print_events(aim, &events, output_format, verbose);
        }
        Ok(())
    }
}
```

### 4.5 Register New Commands

**Modify**: `cli/src/cli.rs` or appropriate command registration file

```rust
// Register edit-occurrence subcommand
.events(subcommands![
    CmdEventNew::command(),
    CmdEventEdit::command(),
    CmdEventEditOccurrence::command(),  // New
    CmdEventDelay::command(),
    CmdEventReschedule::command(),
    CmdEventList::command(),
    // ... other commands
])
```

---

## Phase 5: TUI Support

### 5.1 Recurrence Input Form

**New file**: `cli/src/tui/recurrence.rs`

```rust
use ratatui::widgets::*;

pub struct RecurrenceForm {
    freq: RecurrenceFrequency,
    interval: u32,
    until: Option<String>,
    count: Option<u32>,
    by_day: Vec<WeekDay>,
    by_month: Vec<u8>,
    by_month_day: Vec<i8>,
}

impl RecurrenceForm {
    pub fn new() -> Self { }

    pub fn build_rule(&self) -> Result<RecurrenceRule, String> { }

    pub fn render(&self, area: Rect, buf: &mut Buffer) { }
}

/// Interactive TUI for building recurrence rules
pub async fn edit_recurrence() -> Result<Option<RecurrenceRule>> {
    // Use prompt library or custom TUI
    // 1. Select frequency (Daily, Weekly, Monthly, Yearly)
    // 2. Set interval
    // 3. Choose UNTIL or COUNT (optional)
    // 4. Configure BY* rules based on frequency
    // 5. Preview generated occurrences
    // 6. Confirm or cancel
}
```

### 5.2 Occurrence Selection

**Modify**: `cli/src/tui/event.rs`

```rust
/// Select a specific occurrence from a recurring event
pub async fn select_occurrence(
    aim: &Aim,
    event: &VEvent<String>,
) -> Result<Option<Zoned>> {
    // 1. Get date range (e.g., next 3 months)
    let range = calculate_default_range(aim.now());

    // 2. Expand event to show occurrences
    let expander = EventExpander::new(event);
    let occurrences = expander.expand(range)?;

    // 3. Display calendar with occurrence markers
    // 4. User selects specific occurrence
    // 5. Return selected recurrence_id
}

/// Edit event or specific occurrence
pub async fn edit_event_or_occurrence(
    aim: &Aim,
    id: &Id,
) -> Result<Option<EventPatch>> {
    // 1. Get event
    let event = aim.get_event(id).await?;

    // 2. Check if recurring
    if event.rrule.is_some() {
        // Ask: edit this instance only, or entire series?
        let choice = ask_series_or_instance()?;

        if choice == SeriesChoice::Instance {
            // Select occurrence
            let recurrence_id = select_occurrence(aim, &event).await?;
            // Edit as occurrence
            return edit_occurrence(aim, id, recurrence_id).await;
        }
    }

    // Edit normally (entire series or non-recurring)
    edit_event(aim, &event).await
}
```

### 5.3 Calendar Visualization

**New file**: `cli/src/tui/calendar.rs`

```rust
/// Calendar widget with recurrence indicators
pub struct RecurrenceCalendar {
    year: i32,
    month: u8,
    occurrences: HashMap<Date, Vec<usize>>,  // date -> occurrence indices
}

impl RecurrenceCalendar {
    pub fn new(year: i32, month: u8) -> Self { }

    pub fn add_occurrence(&mut self, date: Date, index: usize) { }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Show month grid
        // Mark dates with occurrences (e.g., with dots or symbols)
        // Handle multiple occurrences on same day
    }
}
```

---

## Phase 6: Testing

### 6.1 Unit Tests

**ical/src/recurrence/tests.rs**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_recurrence() {
        let rule = ValueRecurrenceRule {
            freq: RecurrenceFrequency::Daily,
            interval: Some(1),
            count: Some(5),
            ..Default::default()
        };

        let start = date(2025, 1, 1).at(10, 0, 0, 0).to_zoned(TimeZone::UTC).unwrap();
        let occurrences: Vec<_> = RecurrenceIterator::new(&rule, start)
            .take(5)
            .collect();

        assert_eq!(occurrences.len(), 5);
        assert_eq!(occurrences[0], start);
        assert_eq!(occurrences[1], start + 1.days());
    }

    #[test]
    fn test_weekly_recurrence_byday() {
        let rule = ValueRecurrenceRule {
            freq: RecurrenceFrequency::Weekly,
            by_day: vec![
                WeekDayNum { day: WeekDay::Monday, occurrence: None },
                WeekDayNum { day: WeekDay::Wednesday, occurrence: None },
                WeekDayNum { day: WeekDay::Friday, occurrence: None },
            ],
            ..Default::default()
        };

        // Test Monday, Wednesday, Friday pattern
    }

    #[test]
    fn test_monthly_recurrence_nth_weekday() {
        let rule = ValueRecurrenceRule {
            freq: RecurrenceFrequency::Monthly,
            by_day: vec![WeekDayNum {
                day: WeekDay::Sunday,
                occurrence: Some(2),  // Second Sunday
            }],
            ..Default::default()
        };

        // Test "second Sunday of every month"
    }

    #[test]
    fn test_yearly_recurrence() {
        let rule = ValueRecurrenceRule {
            freq: RecurrenceFrequency::Yearly,
            by_month: vec![11],
            by_day: vec![WeekDayNum {
                day: WeekDay::Tuesday,
                occurrence: Some(1),  // First Tuesday
            }],
            ..Default::default()
        };

        // Test "first Tuesday of November every year"
    }

    #[test]
    fn test_until_limit() {
        // Test UNTIL date truncates recurrence
    }

    #[test]
    fn test_count_limit() {
        // Test COUNT limits number of occurrences
    }

    #[test]
    fn test_dst_transition() {
        // Test recurrence across DST boundaries
    }

    #[test]
    fn test_leap_year() {
        // Test recurrence on Feb 29 and non-leap years
    }
}
```

### 6.2 Integration Tests

**tests/recurrence_integration.rs**:

```rust
use aimcal_core::*;
use jiff::{civil::date, tz::TimeZone};

#[tokio::test]
async fn test_create_recurring_event() {
    let mut aim = create_test_aim().await;

    let draft = EventDraft {
        summary: "Weekly Meeting".to_string(),
        start: Some(LooseDateTime::Local(
            date(2025, 1, 6).at(10, 0, 0, 0)
                .to_zoned(TimeZone::UTC).unwrap()
        )),
        end: Some(LooseDateTime::Local(
            date(2025, 1, 6).at(11, 0, 0, 0)
                .to_zoned(TimeZone::UTC).unwrap()
        )),
        recurrence: Some(RecurrenceRule {
            freq: RecurrenceFrequency::Weekly,
            by_day: Some(vec![WeekDay::Monday]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let event = aim.new_event(draft).await?;
    assert!(event.rrule.is_some());
}

#[tokio::test]
async fn test_expand_recurring_event() {
    let mut aim = create_test_aim().await;

    // Create recurring event
    // ...

    // Query expanded events
    let start = date(2025, 1, 1).at(0, 0, 0, 0).to_zoned(TimeZone::UTC).unwrap();
    let end = date(2025, 2, 1).at(0, 0, 0, 0).to_zoned(TimeZone::UTC).unwrap();

    let conds = ResolvedEventConditions {
        start_before: Some(end),
        end_after: Some(start),
    };

    let events = aim.list_events_expanded(&conds, &(100, 0).into()).await?;

    // Should have 4 Mondays in January 2025
    assert_eq!(events.len(), 4);
}

#[tokio::test]
async fn test_edit_occurrence_creates_exception() {
    let mut aim = create_test_aim().await;

    // Create recurring event
    let uid = create_recurring_event(&mut aim).await;

    // Edit second occurrence
    let recurrence_id = date(2025, 1, 13).at(10, 0, 0, 0)
        .to_zoned(TimeZone::UTC).unwrap();

    let patch = EventPatch {
        summary: Some("Modified Meeting".to_string()),
        ..Default::default()
    };

    aim.update_occurrence(&uid, recurrence_id, patch).await?;

    // Verify base event unchanged
    let base = aim.get_event(&Id::ShortIdOrUid(uid.clone())).await?;
    assert_eq!(base.summary(), "Weekly Meeting");

    // Verify exception created
    // (Need to query exceptions)
}

#[tokio::test]
async fn test_exdate_excludes_occurrence() {
    // Test that EXDATE properly excludes occurrences
}

#[tokio::test]
async fn test_rdate_adds_occurrences() {
    // Test that RDATE properly adds extra occurrences
}
```

### 6.3 Performance Tests

**tests/recurrence_performance.rs**:

```rust
#[tokio::test]
async fn test_expand_large_recurring_set() {
    // Create recurring event with 10 years of daily occurrences
    // Measure expansion time
    // Should complete in < 100ms for a month range
}

#[tokio::test]
async fn test_query_with_many_recurring_events() {
    // Create 100 recurring events
    // Query month range
    // Measure time
}
```

---

## Phase 7: Advanced Features (Post-MVP)

### 7.1 Edit Entire Series

```rust
pub async fn update_series(
    &mut self,
    uid: &str,
    patch: EventPatch,
) -> Result<Vec<VEvent<String>>>;
```

- Updates base event
- Optionally updates or deletes all exceptions
- Option to keep exceptions

### 7.2 Edit This and Following

```rust
pub async fn update_from_occurrence(
    &mut self,
    uid: &str,
    recurrence_id: Zoned,
    patch: EventPatch,
) -> Result<Vec<VEvent<String>>>;
```

- Adds occurrence to EXDATE
- Creates new base event with adjusted RRULE (starting from next occurrence)
- Migrates relevant exceptions

### 7.3 Caching Strategy

```rust
pub struct RecurrenceCache {
    cache_window: Duration,  // e.g., 1 year
    last_refresh: Zoned,
}
```

- Cache expanded occurrences in memory
- Refresh periodically or on demand
- Optional feature for performance optimization

### 7.4 Recurrence Templates

```rust
pub enum RecurrenceTemplate {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Weekdays { days: Vec<WeekDay> },
    // ... more templates
}

impl RecurrenceTemplate {
    pub fn to_rule(&self) -> RecurrenceRule;
}
```

- Predefined common patterns
- Quick CLI shortcuts (e.g., `--recurrence daily`, `--recurrence weekdays`)
- Configurable in config file

---

## Implementation Timeline

### Week 1-2: Phase 1 - Core Recurrence Engine (ical)

- [ ] Design RecurrenceIterator API
- [ ] Implement basic FREQ support (Daily, Weekly, Monthly, Yearly)
- [ ] Implement INTERVAL
- [ ] Implement COUNT/UNTIL
- [ ] Implement BYDAY
- [ ] Implement BYMONTH and BYMONTHDAY
- [ ] Implement other BY\* rules
- [ ] Add comprehensive unit tests
- [ ] RFC 5545 compliance testing

### Week 3: Phase 2 - Core Library Extensions

- [ ] Create RecurrenceRule type
- [ ] Modify EventDraft and TodoDraft
- [ ] Create ExpandedEvent and ExpandedTodo
- [ ] Implement EventExpander
- [ ] Add occurrence query methods to Aim
- [ ] Add database schema migration
- [ ] Update storage logic

### Week 4: Phase 3 - Database and Persistence

- [ ] Write and apply migration
- [ ] Update EventRepo and TodoRepo
- [ ] Add exception storage methods
- [ ] Test round-trip (create → store → retrieve)

### Week 5: Phase 4 - CLI Basic Support

- [ ] Add recurrence arguments
- [ ] Update CmdEventNew
- [ ] Update CmdTodoNew
- [ ] Add expanded listing option
- [ ] Test CLI commands

### Week 6: Phase 5 - Occurrence Management

- [ ] Implement CmdEventEditOccurrence
- [ ] Implement CmdTodoEditOccurrence
- [ ] Implement CmdEventDeleteOccurrence
- [ ] Test occurrence operations
- [ ] Update documentation

### Week 7: Phase 6 - TUI Support

- [ ] Create recurrence form
- [ ] Add occurrence selection
- [ ] Update TUI event editing
- [ ] Add calendar visualization
- [ ] Test TUI workflows

### Week 8+: Phase 7 - Testing and Polish

- [ ] Write integration tests
- [ ] Performance testing and optimization
- [ ] Documentation updates
- [ ] User guide for recurrence features
- [ ] Bug fixes and edge cases

---

## Success Criteria

- [x] All RFC 5545 recurrence patterns supported
- [x] On-the-fly expansion works efficiently
- [x] Exception handling (RECURRENCE-ID) works correctly
- [x] CLI can create recurring events/todos
- [x] CLI can edit individual occurrences
- [x] TUI supports recurrence creation and editing
- [x] Full test coverage (unit, integration, performance)
- [x] Documentation complete
- [x] No performance regressions

---

## Open Questions

1. **Backward compatibility**: Should existing events without recurrence be handled transparently? → **Yes, treat as non-recurring**

2. **Performance**: On-the-fly expansion may be slow for very large recurrence sets (e.g., 10 years of daily events with 100+ such series). Acceptable for MVP? → **Yes, add caching in post-MVP if needed**

3. **CalDAV sync**: How should recurring events sync with external calendars? → **Defer to future work, ensure data structure is compatible**

4. **Max expansion window**: Should there be a configurable limit on how far ahead we expand? → **Yes, default to 1 year, configurable**

5. **Error handling**: What to do if a recurrence rule is invalid? → **Fail fast with clear error message, don't create partial event**

---

## References

- RFC 5545: iCalendar specification
- RFC 5545 Section 3.3.10: Recurrence Rule
- RFC 5545 Section 4.8.5.4: Exception Date-Times
- RFC 5545 Section 4.8.4.4: Recurrence Date-Times
- jiff documentation: Timezone-aware datetime calculations
