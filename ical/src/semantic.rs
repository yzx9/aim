// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod analysis;
mod icalendar;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

pub use analysis::{SemanticError, semantic_analysis};
pub use icalendar::{CalendarComponent, ICalendar};
pub use valarm::VAlarm;
pub use vevent::VEvent;
pub use vfreebusy::VFreeBusy;
pub use vjournal::VJournal;
pub use vtimezone::{TimeZoneObservance, VTimeZone};
pub use vtodo::VTodo;
