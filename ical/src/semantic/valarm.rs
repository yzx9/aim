// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use crate::SemanticError;
use crate::semantic::properties::{Attachment, Attendee, Duration, Text, Trigger};
use crate::typed::TypedComponent;

/// Alarm component (VALARM)
#[derive(Debug, Clone)]
pub struct VAlarm {
    /// Action to perform when alarm triggers
    pub action: AlarmActionType,

    /// When to trigger the alarm
    pub trigger: Trigger,

    /// Repeat count for the alarm
    pub repeat: Option<u32>,

    /// Duration between repeats
    pub duration: Option<Duration>,

    /// Description for display alarm
    pub description: Option<Text>,

    /// Summary for email alarm
    pub summary: Option<Text>,

    /// Attendees for email alarm
    pub attendees: Vec<Attendee>,

    /// Attachment for audio alarm
    pub attach: Option<Attachment>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Alarm action types
#[derive(Debug, Clone, Copy)]
pub enum AlarmActionType {
    /// Audio alarm
    Audio,

    /// Display alarm
    Display,

    /// Email alarm
    Email,

    /// Procedure alarm
    Procedure,
    // /// Custom action
    // Custom(String),
}

/// Parse a `TypedComponent` into a `VAlarm`
pub fn parse_valarm(_comp: TypedComponent) -> Result<VAlarm, SemanticError> {
    todo!("Implement parse_valarm")
}
