// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Recurrence-related property types.
//!
//! This module contains property types related to date/time recurrence.
//! All types implement `kind()` methods and validate their property kind
//! during conversion from `ParsedProperty`:
//!
//! - 3.8.2.6: `FreeBusy` - Free/busy time information
//! - 3.8.5.1: `ExDate` - Exception date-times
//! - 3.8.5.2: `RDate` - Recurrence date-times
//!
//! Value types:
//! - `ExDateValue` - Exception date/time value (DATE or DATE-TIME)
//! - `RDateValue` - Recurrence date/time value (DATE, DATE-TIME, or PERIOD)

use std::convert::TryFrom;

use crate::parameter::{FreeBusyType, Parameter, ValueKind};
use crate::property::PropertyKind;
use crate::property::datetime::{DateTime, Period};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDate};

/// Exception date-time value (can be DATE or DATE-TIME).
#[derive(Debug, Clone)]
pub enum ExDateValue<'src> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<'src>),
}

/// Recurrence date-time value (can be DATE, DATE-TIME, or PERIOD).
#[derive(Debug, Clone)]
pub enum RDateValue<'src> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<'src>),
    /// Period value
    Period(Period<'src>),
}

/// Free/Busy Time (RFC 5545 Section 3.8.2.6)
///
/// This property defines one or more free or busy time intervals.
#[derive(Debug, Clone)]
pub struct FreeBusy<'src> {
    /// Free/Busy type parameter
    pub fb_type: FreeBusyType,
    /// List of free/busy time periods
    pub values: Vec<Period<'src>>,
}

impl FreeBusy<'_> {
    /// Get the property kind for `FreeBusy`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::FreeBusy
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for FreeBusy<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        // Extract FBTYPE parameter (defaults to BUSY)
        let mut fb_type = FreeBusyType::Busy;
        for param in &prop.parameters {
            if let Parameter::FreeBusyType { value, .. } = param {
                fb_type = *value;
                break; // Found it, no need to continue
            }
        }

        let mut errors = Vec::new();
        if prop.values.is_empty() {
            errors.push(TypedError::PropertyMissingValue {
                property: prop.kind,
                span: prop.span,
            });
        }

        let mut values = Vec::with_capacity(prop.values.len());
        for value in prop.values {
            match Period::try_from(value) {
                Ok(period) => values.push(period),
                Err(e) => errors.extend(e),
            }
        }

        Ok(FreeBusy { fb_type, values })
    }
}

/// Exception Date-Times (RFC 5545 Section 3.8.5.1)
///
/// This property defines the list of date-time exceptions for a recurring
/// calendar component.
#[derive(Debug, Clone)]
pub struct ExDate<'src> {
    /// List of exception dates/times
    pub dates: Vec<ExDateValue<'src>>,
}

impl ExDate<'_> {
    /// Get the property kind for `ExDate`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::ExDate
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for ExDate<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        prop.values
            .into_iter()
            .map(|v| match v {
                Value::Date(d) => Ok(ExDateValue::Date(d)),
                Value::DateTime(dt) => Ok(ExDateValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                })),
                _ => Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::DateTime,
                    found: ValueKind::Text,
                    span: prop.span.clone(),
                }]),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|dates| Self { dates })
    }
}

/// Recurrence Date-Times (RFC 5545 Section 3.8.5.2)
///
/// This property defines the list of date-times for a recurring calendar component.
#[derive(Debug, Clone)]
pub struct RDate<'src> {
    /// List of recurrence dates/times/periods
    pub dates: Vec<RDateValue<'src>>,
}

impl RDate<'_> {
    /// Get the property kind for `RDate`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::RDate
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for RDate<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        prop.values
            .into_iter()
            .map(|v| match v {
                Value::Date(d) => Ok(RDateValue::Date(d)),
                Value::DateTime(dt) => Ok(RDateValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                })),
                Value::Period(_) => {
                    // Period values need to be handled at semantic level
                    // For now, just return an error
                    Err(vec![TypedError::PropertyInvalidValue {
                        property: prop.kind,
                        value: "Period values must be processed at semantic level".to_string(),
                        span: prop.span.clone(),
                    }])
                }
                _ => Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::Period,
                    found: ValueKind::Text,
                    span: prop.span.clone(),
                }]),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|dates| Self { dates })
    }
}
