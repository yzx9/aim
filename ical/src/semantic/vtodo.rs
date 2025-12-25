// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! To-do component (VTODO) for iCalendar semantic components.

use crate::RecurrenceRule;
use crate::semantic::enums::{Classification, Period};
use crate::semantic::properties::{Attendee, Geo, Organizer, Text};
use crate::semantic::valarm::VAlarm;
use crate::semantic::{DateTime, Duration, Uri};

/// To-do component (VTODO)
#[derive(Debug, Clone)]
pub struct VTodo {
    /// Unique identifier for the todo
    pub uid: String,

    /// Date/time the todo was created
    pub dt_stamp: DateTime,

    /// Date/time the todo is due
    pub due: Option<DateTime>,

    /// Date/time to start the todo
    pub dt_start: Option<DateTime>,

    /// Completion date/time
    pub completed: Option<DateTime>,

    /// Duration of the todo
    pub duration: Option<Duration>,

    /// Summary/title of the todo
    pub summary: Option<Text>,

    /// Description of the todo
    pub description: Option<Text>,

    /// Location of the todo
    pub location: Option<Text>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the todo
    pub url: Option<Uri>,

    /// Organizer of the todo
    pub organizer: Option<Organizer>,

    /// Attendees of the todo
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the todo
    pub status: Option<TodoStatus>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

    /// Percentage complete (0-100)
    pub percent_complete: Option<u8>,

    /// Classification
    pub classification: Option<Classification>,

    /// Resources
    pub resources: Vec<Text>,

    /// Categories
    pub categories: Vec<Text>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period>,

    /// Exception dates
    pub ex_date: Vec<DateTime>,

    /// Timezone identifier
    pub tz_id: Option<String>,

    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm>,
}

/// To-do status
#[derive(Debug, Clone, Copy)]
pub enum TodoStatus {
    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// To-do is cancelled
    Cancelled,
    // /// Custom status
    // Custom(String),
}

/// Parse a `TypedComponent` into a `VTodo`
pub fn parse_vtodo(
    _comp: crate::typed::TypedComponent,
) -> Result<VTodo, crate::semantic::SemanticError> {
    todo!("Implement parse_vtodo")
}
