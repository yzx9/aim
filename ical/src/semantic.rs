// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod custom;
mod icalendar;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

pub use custom::{CustomComponent, CustomComponentOwned, CustomComponentRef};
pub use icalendar::{CalendarComponent, ICalendar, ICalendarOwned, ICalendarRef};
pub use valarm::{VAlarm, VAlarmOwned, VAlarmRef};
pub use vevent::{EventStatus, EventStatusOwned, EventStatusRef, VEvent, VEventOwned, VEventRef};
pub use vfreebusy::{VFreeBusy, VFreeBusyOwned, VFreeBusyRef};
pub use vjournal::{
    JournalStatus, JournalStatusOwned, JournalStatusRef, VJournal, VJournalOwned, VJournalRef,
};
pub use vtimezone::{TimeZoneObservance, VTimeZone, VTimeZoneOwned, VTimeZoneRef};
pub use vtodo::{TodoStatus, TodoStatusOwned, VTodo, VTodoOwned, VTodoRef};

use crate::keyword::KW_VCALENDAR;
use crate::property::PropertyKindRef;
use crate::string_storage::Span;
use crate::typed::TypedComponent;

/// Perform semantic analysis on typed components.
///
/// # Errors
///
/// Returns a vector of errors if:
/// - No VCALENDAR components are found
/// - Any components failed to parse
pub fn semantic_analysis(
    typed_components: Vec<TypedComponent<'_>>,
) -> Result<Vec<ICalendarRef<'_>>, Vec<SemanticError<'_>>> {
    // Return error only if no calendars
    if typed_components.is_empty() {
        return Err(vec![SemanticError::ConstraintViolation {
            span: Span { start: 0, end: 0 },
            message: format!("No {KW_VCALENDAR} components found"),
        }]);
    }

    let mut calendars = Vec::with_capacity(typed_components.len());
    let mut all_errors = Vec::new();

    for component in typed_components {
        match ICalendar::try_from(component) {
            Ok(calendar) => calendars.push(calendar),
            Err(errors) => all_errors.extend(errors),
        }
    }

    if all_errors.is_empty() {
        Ok(calendars)
    } else {
        Err(all_errors)
    }
}

/// Error type for parsing operations
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
pub enum SemanticError<'src> {
    /// Unknown component type
    #[error("Unknown component type: {component}")]
    UnknownComponent {
        /// The unknown component name
        component: String,
        /// The span of the error
        span: Span,
    },

    /// Expected a different component type
    #[error("Expected '{expected}' component, got '{got}'")]
    ExpectedComponent {
        /// The expected component name
        expected: &'src str,
        /// The actual component name that was found
        got: &'src str,
        /// The span of the error
        span: Span,
    },

    /// Duplicate property
    #[error("Duplicate property '{property}'")]
    DuplicateProperty {
        /// The property that is duplicated
        property: PropertyKindRef<'src>,
        /// The span of the error
        span: Span,
    },

    /// Missing required property
    #[error("Missing required property '{property}'")]
    MissingProperty {
        /// The property that is missing
        property: PropertyKindRef<'src>,
        /// The span of the error
        span: Span,
    },

    /// Invalid property value
    #[error("Invalid value '{value}' for property: {property}")]
    InvalidValue {
        /// The property that has the invalid value
        property: PropertyKindRef<'src>,
        /// The invalid value description
        value: String,
        /// The span of the error
        span: Span,
    },

    /// Business rule constraint violation
    #[error("Constraint violation: {message}")]
    ConstraintViolation {
        /// Error message describing the constraint violation
        message: String,
        /// The span of the error
        span: Span,
    },
}

impl SemanticError<'_> {
    /// Get the span of this error.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::UnknownComponent { span, .. }
            | Self::ExpectedComponent { span, .. }
            | Self::DuplicateProperty { span, .. }
            | Self::MissingProperty { span, .. }
            | Self::InvalidValue { span, .. }
            | Self::ConstraintViolation { span, .. } => *span,
        }
    }
}
