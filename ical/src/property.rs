// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property module for iCalendar properties organized by RFC 5545 sections.
//!
//! This module provides property specifications and typed property structures
//! as defined in RFC 5545. Property types are organized by their corresponding
//! RFC 5545 sections for better code organization and maintainability.
//!
//! ## Property Organization
//!
//! - 3.7. Calendar Properties (calendar.rs)
//! - 3.8.1. Descriptive Component Properties (descriptive.rs)
//! - 3.8.2. Date and Time Properties (datetime.rs)
//! - 3.8.3. Time Zone Component Properties (timezone.rs)
//! - 3.8.4. Relationship Component Properties (relationship.rs)
//! - 3.8.5. Recurrence Properties (recurrence.rs)
//! - 3.8.6. Alarm Component Properties (alarm.rs)
//! - 3.8.7. Change Management Component Properties (changemgmt.rs)
//! - 3.8.8. Miscellaneous Properties (miscellaneous.rs)
//!
//! ## Type Safety
//!
//! All property types implement kind validation through:
//! - A `kind()` method returning the corresponding `PropertyKind`
//! - Type checking in `TryFrom<ParsedProperty>` implementations that verify
//!   the property kind matches the expected type
//! - Dedicated wrapper types for specific properties (e.g., `Created`, `DtStart`, `Summary`)
//!
//! This ensures that properties are correctly typed during parsing and prevents
//! invalid property assignments.

#[macro_use]
mod util;

// Property type modules organized by RFC 5545 sections
mod alarm;
mod ast;
mod calendar;
mod changemgmt;
mod datetime;
mod descriptive;
mod kind;
mod miscellaneous;
mod recurrence;
mod relationship;
mod timezone;

pub use alarm::{Action, Repeat, Trigger, TriggerValue};
pub use ast::Property;
pub use calendar::{CalendarScale, Method, ProductId, Version};
pub use changemgmt::{Created, DtStamp, LastModified, Sequence};
pub use datetime::{
    Completed, DateTime, DtEnd, DtStart, Due, Duration, FreeBusy, Period, Time, TimeTransparency,
};
pub use descriptive::{
    Attachment, AttachmentValue, Categories, Classification, Comment, Description, Geo, Location,
    PercentComplete, Priority, Resources, Status, Summary,
};
pub use kind::PropertyKind;
pub use miscellaneous::RequestStatus;
pub use recurrence::{ExDate, ExDateValue, RDate, RDateValue};
pub use relationship::{Attendee, Contact, Organizer, RecurrenceId, RelatedTo, Uid, Url};
pub use timezone::{TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl};
pub use util::{Text, Texts};
