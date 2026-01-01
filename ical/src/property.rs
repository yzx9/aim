// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property module for iCalendar properties organized by RFC 5545 sections.
//!
//! This module provides property specifications and typed property structures
//! as defined in RFC 5545. Property types are organized by their corresponding
//! RFC 5545 sections for better code organization and maintainability.
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
//!
//! ## Module Organization
//!
//! - **ast**: Unified `Property` enum with type-safe variants for all properties
//! - **kind**: Property kinds and allowed value types
//! - **util**: Text property utilities (`Text`, `Texts`, helper functions)
//! - **alarm** (Section 3.8.6): Alarm properties - `Action`, `Trigger`
//! - **cal** (Section 3.7): Calendar properties - `CalendarScale`, `Method`, `ProductId`, `Version`
//! - **datetime** (Section 3.8.2): Date/time properties including wrapper types:
//!   - Base types: `DateTime`, `Period`, `Time`
//!   - Wrapper types: `Created`, `DtStamp`, `LastModified`, `DtStart`, `DtEnd`, `Due`,
//!     `Completed`, `RecurrenceId`
//! - **descriptive** (Section 3.8.1): Descriptive properties including wrapper types:
//!   - Complex types: `Attachment`, `Classification`, `Geo`, `Organizer`
//!   - Text wrapper types: `Categories`, `Comment`, `Description`, `Location`, `Contact`,
//!     `RelatedTo`, `RequestStatus`, `Resources`, `Summary`
//!   - URI wrapper types: `TzId`, `TzName`, `TzUrl`, `Url`, `Uid`
//! - **numeric** (Section 3.8.1.9): Numeric properties - `Duration`, `PercentComplete`,
//!   `Priority`, `Repeat`, `Sequence`
//! - **recurrence** (Section 3.8.5): Recurrence properties - `ExDate`, `RDate`, `FreeBusy`
//! - **relationship** (Section 3.8.4): Relationship properties - `Attendee`
//! - **status** (Section 3.8.1.11): Status properties - `Status`
//! - **timezone** (Section 3.8.3): Time zone properties - `TzOffsetFrom`, `TzOffsetTo`
//! - **transp** (Section 3.8.2.7): Time transparency - `TimeTransparency`
//!
//! ## Property Kinds
//!
//! The `kind` submodule defines the `PropertyKind` enum representing all standard
//! iCalendar properties and their allowed value types.

// Property type modules organized by RFC 5545 sections
mod alarm; // Section 3.8.6 - Alarm Component Properties
mod ast; // Unified Property enum
mod cal; // Section 3.7 - Calendar Properties
mod datetime; // Section 3.8.2 - Date and Time Properties
mod descriptive; // Section 3.8.1 - Descriptive Component Properties
mod numeric; // Numeric properties
mod recurrence; // Recurrence properties
mod relationship; // Section 3.8.4 - Component Relationship Properties
mod status; // Section 3.8.1.11 - Status Properties
mod timezone; // Section 3.8.3 - Time Zone Component Properties
mod transp; // Section 3.8.2.7 - Time Transparency Property

mod kind;
mod util;

pub use alarm::{Action, Trigger, TriggerValue};
pub use ast::Property;
pub use cal::{CalendarScale, Method, ProductId, Version};
pub use datetime::{
    Completed, Created, DateTime, DtEnd, DtStamp, DtStart, Due, LastModified, Period,
    RecurrenceId, Time,
};
pub use descriptive::{
    Attachment, AttachmentValue, Categories, Classification, Comment, Contact, Description,
    Geo, Location, Organizer, RelatedTo, RequestStatus, Resources, Summary, TzId, TzName,
    TzUrl, Url, Uid,
};
pub use kind::PropertyKind;
pub use numeric::{Duration, PercentComplete, Priority, Repeat, Sequence};
pub use recurrence::{ExDate, ExDateValue, FreeBusy, RDate, RDateValue};
pub use relationship::Attendee;
pub use status::Status;
pub use timezone::{TzOffsetFrom, TzOffsetTo};
pub use transp::TimeTransparency;
pub use util::{Text, Texts};
