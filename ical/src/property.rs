// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
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
//! - **spec**: Property specifications, `PropertyKind` enum, and metadata
//! - **alarm** (Section 3.8.6): Alarm properties - `Action`, `Trigger`
//! - **cal** (Section 3.7): Calendar properties - `CalendarScale`, `Method`, `ProductId`, `Version`
//! - **datetime** (Section 3.8.2): Date/time properties - `DateTime`, `Period`, `Time`
//! - **descriptive** (Section 3.8.1): Descriptive properties - `Attachment`, `Classification`, `Geo`, `Organizer`, `Text`
//! - **relationship** (Section 3.8.4): Relationship properties - `Attendee`
//! - **status** (Section 3.8.1.11): Status properties - `EventStatus`, `TodoStatus`, `JournalStatus`
//! - **timezone** (Section 3.8.3): Time zone properties - `TimeZoneOffset`
//! - **transp** (Section 3.8.2.7): Time transparency - `TimeTransparency`
//!
//! ## Property Specifications
//!
//! The `spec` submodule provides metadata about all standard iCalendar properties
//! including cardinality rules, allowed parameters, and value types.

mod spec;

// Property type modules organized by RFC 5545 sections
pub mod alarm; // Section 3.8.6 - Alarm Component Properties
pub mod ast; // Unified Property enum
pub mod cal; // Section 3.7 - Calendar Properties
pub mod datetime; // Section 3.8.2 - Date and Time Properties
pub mod descriptive; // Section 3.8.1 - Descriptive Component Properties
pub mod relationship; // Section 3.8.4 - Component Relationship Properties
pub mod status; // Section 3.8.1.11 - Status Properties
pub mod timezone; // Section 3.8.3 - Time Zone Component Properties
pub mod transp; // Section 3.8.2.7 - Time Transparency Property
pub mod util;

pub use alarm::{Action, Trigger, TriggerValue};
pub use ast::{ExDateValue, Property, RDateValue};
pub use cal::{CalendarScale, Method, ProductId, Version};
pub use datetime::{DateTime, Period, Time};
pub use descriptive::{
    Attachment, AttachmentValue, Classification, Geo, Organizer, Text, parse_multi_text_property,
};
pub use relationship::Attendee;
pub use spec::{PropertyCardinality, PropertyKind, PropertySpec, ValueCardinality};
pub use status::Status;
pub use timezone::TimeZoneOffset;
pub use transp::TimeTransparency;
