// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! This module provides strongly-typed structures that represent the semantic
//! meaning of iCalendar data parsed from the raw syntax. These types follow
//! the RFC 5545 specification and provide a convenient API for working with
//! calendar data without dealing with string parsing and validation.

mod extensions;
mod icalendar;
mod tz_validator;
mod valarm;
mod vevent;
mod vfreebusy;
mod vjournal;
mod vtimezone;
mod vtodo;

pub use extensions::{UnrecognizedComponent, XComponent};
pub use icalendar::{CalendarComponent, ICalendar};
pub use tz_validator::validate_tzids;
pub use valarm::VAlarm;
pub use vevent::{EventStatus, EventStatusValue, VEvent};
pub use vfreebusy::VFreeBusy;
pub use vjournal::{JournalStatus, VJournal};
pub use vtimezone::{TimeZoneObservance, VTimeZone};
pub use vtodo::{TodoStatus, TodoStatusValue, VTodo};

use crate::keyword::KW_VCALENDAR;
use crate::property::PropertyKind;
use crate::string_storage::{Segments, Span};
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
) -> Result<Vec<ICalendar<Segments<'_>>>, Vec<SemanticError<'_>>> {
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
        got: Segments<'src>,
        /// The span of the error
        span: Span,
    },

    /// Duplicate property
    #[error("Duplicate property '{property}'")]
    DuplicateProperty {
        /// The property that is duplicated
        property: PropertyKind<Segments<'src>>,
        /// The span of the error
        span: Span,
    },

    /// Missing required property
    #[error("Missing required property '{property}'")]
    MissingProperty {
        /// The property that is missing
        property: PropertyKind<Segments<'src>>,
        /// The span of the error
        span: Span,
    },

    /// Invalid property value
    #[error("Invalid value '{value}' for property: {property}")]
    InvalidValue {
        /// The property that has the invalid value
        property: PropertyKind<Segments<'src>>,
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

    /// Timezone identifier not found in VTIMEZONE components or local database
    /// This variant does not use the lifetime parameter, as it owns all its data
    #[error(
        "Timezone identifier '{tzid}' not found. Add a VTIMEZONE component or ensure the timezone is in the IANA database"
    )]
    TimezoneNotFound {
        /// The timezone identifier that was not found (owned)
        tzid: String,
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
            | Self::ConstraintViolation { span, .. }
            | Self::TimezoneNotFound { span, .. } => *span,
        }
    }
}
