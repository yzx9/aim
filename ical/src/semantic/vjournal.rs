// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use crate::semantic::enums::{Classification, Period};
use crate::semantic::properties::{Attendee, DateTime, Organizer, Text};

/// Journal entry component (VJOURNAL)
#[derive(Debug, Clone)]
pub struct VJournal {
    /// Unique identifier for the journal entry
    pub uid: String,

    /// Date/time the journal entry was created
    pub dt_stamp: DateTime,

    /// Date/time of the journal entry
    pub dt_start: DateTime,

    /// Summary/title of the journal entry
    pub summary: Option<Text>,

    /// Description of the journal entry
    pub description: Option<Text>,

    /// Organizer of the journal entry
    pub organizer: Option<Organizer>,

    /// Attendees of the journal entry
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the journal entry
    pub status: Option<JournalStatus>,

    /// Classification
    pub classification: Option<Classification>,

    /// Categories
    pub categories: Vec<Text>,

    /// Recurrence rule
    pub rrule: Option<crate::typed::RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period>,

    /// Exception dates
    pub ex_date: Vec<DateTime>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Journal status
#[derive(Debug, Clone, Copy)]
pub enum JournalStatus {
    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Journal entry is cancelled
    Cancelled,
    // /// Custom status
    // Custom(String),
}

/// Parse a `TypedComponent` into a `VJournal`
pub fn parse_vjournal(
    _comp: crate::typed::TypedComponent,
) -> Result<VJournal, crate::semantic::SemanticError> {
    todo!("Implement parse_vjournal")
}
