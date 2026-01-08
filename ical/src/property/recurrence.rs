// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Recurrence-related property types.
//!
//! This module contains property types related to date/time recurrence
//! (RFC 5545 Section 3.8.5). All types implement `kind()` methods and
//! validate their property kind during conversion from `ParsedProperty`:
//!
//! - 3.8.5.1: `ExDate` - Exception date-times
//! - 3.8.5.2: `RDate` - Recurrence date-times
//!
//! Value types:
//! - `ExDateValue` - Exception date/time value (DATE or DATE-TIME)
//! - `RDateValue` - Recurrence date/time value (DATE, DATE-TIME, or PERIOD)

use std::convert::TryFrom;
use std::fmt::Display;

use crate::parameter::{Parameter, ValueTypeRef};
use crate::property::{DateTime, Period, PropertyKind};
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDate};

/// Exception date-time value (can be DATE or DATE-TIME).
#[derive(Debug, Clone)]
pub enum ExDateValue<S: Clone + Display> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<S>),
}

/// Type alias for borrowed exception date-time value
pub type ExDateValueRef<'src> = ExDateValue<SpannedSegments<'src>>;

/// Type alias for owned exception date-time value
pub type ExDateValueOwned = ExDateValue<String>;

impl ExDateValue<SpannedSegments<'_>> {
    /// Convert borrowed `ExDateValue` to owned `ExDateValue`
    #[must_use]
    pub fn to_owned(&self) -> ExDateValue<String> {
        match self {
            ExDateValue::Date(date) => ExDateValue::Date(*date),
            ExDateValue::DateTime(dt) => ExDateValue::DateTime(dt.to_owned()),
        }
    }
}

/// Recurrence date-time value (can be DATE, DATE-TIME, or PERIOD).
#[derive(Debug, Clone)]
pub enum RDateValue<S: Clone + Display> {
    /// Date-only value
    Date(ValueDate),
    /// Date-time value
    DateTime(DateTime<S>),
    /// Period value
    Period(Period<S>),
}

/// Type alias for borrowed recurrence date-time value
pub type RDateValueRef<'src> = RDateValue<SpannedSegments<'src>>;

/// Type alias for owned recurrence date-time value
pub type RDateValueOwned = RDateValue<String>;

impl RDateValue<SpannedSegments<'_>> {
    /// Convert borrowed `RDateValue` to owned `RDateValue`
    #[must_use]
    pub fn to_owned(&self) -> RDateValue<String> {
        match self {
            RDateValue::Date(date) => RDateValue::Date(*date),
            RDateValue::DateTime(dt) => RDateValue::DateTime(dt.to_owned()),
            RDateValue::Period(period) => RDateValue::Period(period.to_owned()),
        }
    }
}

/// Exception Date-Times (RFC 5545 Section 3.8.5.1)
///
/// This property defines the list of date-time exceptions for a recurring
/// calendar component.
#[derive(Debug, Clone)]
pub struct ExDate<S: Clone + Display> {
    /// List of exception dates/times
    pub dates: Vec<ExDateValue<S>>,

    /// Timezone identifier (optional)
    pub tz_id: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for ExDate<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::ExDate) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::ExDate,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();
        let mut tz_id = None;

        for param in prop.parameters {
            match param {
                p @ Parameter::TimeZoneIdentifier { .. } if tz_id.is_some() => {
                    return Err(vec![TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    }]);
                }
                Parameter::TimeZoneIdentifier { value, .. } => tz_id = Some(value),
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let dates = match prop.value {
            Value::Date { values: dates, .. } => dates
                .into_iter()
                .map(|d| Ok(ExDateValue::Date(d)))
                .collect::<Result<Vec<_>, _>>(),
            Value::DateTime { values: dts, .. } => dts
                .into_iter()
                .map(|dt| {
                    Ok(ExDateValue::DateTime(DateTime::Floating {
                        date: dt.date,
                        time: dt.time.into(),
                        x_parameters: Vec::new(),
                        unrecognized_parameters: Vec::new(),
                    }))
                })
                .collect::<Result<Vec<_>, _>>(),
            v => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueTypeRef::Date,
                    found: v.into_kind(),
                    span,
                }])
            }
        }?;

        Ok(Self {
            dates,
            tz_id,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

impl ExDate<SpannedSegments<'_>> {
    /// Convert borrowed `ExDate` to owned `ExDate`
    #[must_use]
    pub fn to_owned(&self) -> ExDate<String> {
        ExDate {
            dates: self.dates.iter().map(ExDateValue::to_owned).collect(),
            tz_id: self.tz_id.as_ref().map(SpannedSegments::concatnate),
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}

/// Recurrence Date-Times (RFC 5545 Section 3.8.5.2)
///
/// This property defines the list of date-times for a recurring calendar component.
#[derive(Debug, Clone)]
pub struct RDate<S: Clone + Display> {
    /// List of recurrence dates/times/periods
    pub dates: Vec<RDateValue<S>>,

    /// Timezone identifier (optional)
    pub tz_id: Option<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for RDate<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::RDate) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::RDate,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let value_span = prop.value.span();
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();
        let mut tz_id = None;

        for param in prop.parameters {
            match param {
                p @ Parameter::TimeZoneIdentifier { .. } if tz_id.is_some() => {
                    return Err(vec![TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    }]);
                }
                Parameter::TimeZoneIdentifier { value, .. } => tz_id = Some(value),
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let dates = match prop.value {
            Value::Date { values: dates, .. } => dates
                .into_iter()
                .map(|d| Ok(RDateValue::Date(d)))
                .collect::<Result<Vec<_>, _>>(),
            Value::DateTime { values: dts, .. } => dts
                .into_iter()
                .map(|dt| {
                    Ok(RDateValue::DateTime(DateTime::Floating {
                        date: dt.date,
                        time: dt.time.into(),
                        x_parameters: Vec::new(),
                        unrecognized_parameters: Vec::new(),
                    }))
                })
                .collect::<Result<Vec<_>, _>>(),
            Value::Period { .. } => {
                // Period values need to be handled at semantic level
                // For now, just return an error
                return Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: "Period values must be processed at semantic level".to_string(),
                    span: value_span,
                }]);
            }
            v => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueTypeRef::Period,
                    found: v.into_kind(),
                    span,
                }])
            }
        }?;

        Ok(Self {
            dates,
            tz_id,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

impl RDate<SpannedSegments<'_>> {
    /// Convert borrowed `RDate` to owned `RDate`
    #[must_use]
    pub fn to_owned(&self) -> RDate<String> {
        RDate {
            dates: self.dates.iter().map(RDateValue::to_owned).collect(),
            tz_id: self.tz_id.as_ref().map(SpannedSegments::concatnate),
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}
