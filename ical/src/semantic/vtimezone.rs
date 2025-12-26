// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Timezone component (VTIMEZONE) for iCalendar semantic components.

use crate::semantic::properties::{Text, TimeZoneOffset};
use crate::semantic::{DateTime, Uri};
use crate::typed::TypedComponent;
use crate::{RecurrenceRule, SemanticError};

/// Timezone component (VTIMEZONE)
#[derive(Debug, Clone)]
pub struct VTimeZone {
    /// Timezone identifier
    pub tz_id: String,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Timezone URL
    pub tz_url: Option<Uri>,

    /// Standard time observance
    pub standard: Option<TimeZoneObservance>,

    /// Daylight saving time observance
    pub daylight: Option<TimeZoneObservance>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Timezone observance (standard or daylight)
#[derive(Debug, Clone)]
pub struct TimeZoneObservance {
    /// Start date/time for this observance
    pub dt_start: DateTime,

    /// Offset from UTC for this observance
    pub tz_offset_from: TimeZoneOffset,

    /// Offset from UTC for this observance
    pub tz_offset_to: TimeZoneOffset,

    /// Timezone name
    pub tz_name: Vec<Text>,

    /// Recurrence rule for this observance
    pub rrule: Option<RecurrenceRule>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Parse a `TypedComponent` into a `VTimeZone`
pub fn parse_vtimezone(_comp: TypedComponent) -> Result<VTimeZone, Vec<SemanticError>> {
    todo!("Implement parse_vtimezone")
}
