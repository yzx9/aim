// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parameter parsing module for iCalendar parameters.
//!
//! This module handles the parsing and validation of iCalendar parameters
//! as defined in RFC 5545 Section 3.2.

mod ast;
mod definition;

pub use ast::{Parameter, ParameterKind};
pub use definition::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, FreeBusyType, ParticipationRole,
    ParticipationStatus, ValueKind,
};
