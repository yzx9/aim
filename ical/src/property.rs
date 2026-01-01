// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property module for iCalendar properties organized by RFC 5545 sections.
//!
//! This module provides property specifications and typed property structures
//! as defined in RFC 5545. Property types are organized by their corresponding
//! RFC 5545 sections for better code organization and maintainability:
//!
//! ## Module Organization
//!
//! - **ast**: Unified `Property` enum with type-safe variants for all properties
//! - **kind**: Property kinds and allowed value types
//! - **alarm** (Section 3.8.6): Alarm properties - `Action`, `Trigger`
//! - **cal** (Section 3.7): Calendar properties - `CalendarScale`, `Method`, `ProductId`, `Version`
//! - **datetime** (Section 3.8.2): Date/time properties - `DateTime`, `Period`, `Time`
//! - **descriptive** (Section 3.8.1): Descriptive properties - `Attachment`, `Classification`, `Geo`, `Organizer`, `Text`
//! - **numeric**: Numeric properties - `Duration`, `PercentComplete`, `Priority`, `Repeat`, `Sequence`
//! - **recurrence**: Recurrence properties - `ExDate`, `ExDateValue`, `FreeBusy`, `RDate`, `RDateValue`
//! - **relationship** (Section 3.8.4): Relationship properties - `Attendee`
//! - **status** (Section 3.8.1.11): Status properties - `EventStatus`, `TodoStatus`, `JournalStatus`
//! - **timezone** (Section 3.8.3): Time zone properties - `TimeZoneOffset`
//! - **transp** (Section 3.8.2.7): Time transparency - `TimeTransparency`
//!
//! ## Property Kinds
//!
//! The `kind` submodule defines the `PropertyKind` enum representing all standard
//! iCalendar properties and their allowed value types.

mod kind;

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
mod util;

pub use alarm::{Action, Trigger, TriggerValue};
pub use ast::Property;
pub use cal::{CalendarScale, Method, ProductId, Version};
pub use datetime::{DateTime, Period, Time};
pub use descriptive::{Attachment, AttachmentValue, Classification, Geo, Organizer, Text, Texts};
pub use kind::PropertyKind;
pub use numeric::{Duration, PercentComplete, Priority, Repeat, Sequence};
pub use recurrence::{ExDate, ExDateValue, FreeBusy, RDate, RDateValue};
pub use relationship::Attendee;
pub use status::Status;
pub use timezone::TimeZoneOffset;
pub use transp::TimeTransparency;
