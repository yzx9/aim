// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm Component Properties (RFC 5545 Section 3.8.6)
//!
//! This module contains property types for the "Alarm Component Properties"
//! section of RFC 5545. All types implement `kind()` methods and validate
//! their property kind during conversion from `ParsedProperty`:
//!
//! - 3.8.6.1: `Action` - Alarm action type (AUDIO, DISPLAY, EMAIL)
//! - 3.8.6.2: `Repeat` - Alarm repeat count
//! - 3.8.6.3: `Trigger` - Alarm trigger time or duration
//!   - `TriggerValue` - Trigger value variant (duration or date-time)

use std::{convert::TryFrom, fmt, str::FromStr};

use crate::keyword::{KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE};
use crate::parameter::{AlarmTriggerRelationship, Parameter, ValueType};
use crate::property::util::{take_single_string, take_single_value};
use crate::property::{DateTime, PropertyKind};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDuration};

/// Alarm action (RFC 5545 Section 3.8.6.1)
#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// Audio alarm
    Audio,

    /// Display alarm
    Display,

    /// Email alarm
    Email,
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_ACTION_AUDIO => Ok(Self::Audio),
            KW_ACTION_DISPLAY => Ok(Self::Display),
            KW_ACTION_EMAIL => Ok(Self::Email),
            KW_ACTION_PROCEDURE => Err(format!("{KW_ACTION_PROCEDURE} action has been deprecated")),
            _ => Err(format!("Invalid alarm action: {s}")),
        }
    }
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        match self {
            Self::Audio => KW_ACTION_AUDIO,
            Self::Display => KW_ACTION_DISPLAY,
            Self::Email => KW_ACTION_EMAIL,
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Action {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Action) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Action,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(&PropertyKind::Action, prop.values)?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Action,
                value: e,
                span: prop.span,
            }]
        })
    }
}

/// Repeat Count (RFC 5545 Section 3.8.6.2)
///
/// This property defines the number of times the alarm should repeat.
#[derive(Debug, Clone, Copy)]
pub struct Repeat {
    /// Number of repetitions
    pub value: u32,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Repeat {
    type Error = Vec<TypedError<'src>>;

    #[allow(clippy::cast_sign_loss)]
    fn try_from(mut prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Repeat) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Repeat,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        match prop.values.len() {
            0 => Err(vec![TypedError::PropertyMissingValue {
                property: prop.kind,
                span: prop.span,
            }]),
            1 => match prop.values.pop().unwrap() {
                Value::Integer(i) if i >= 0 => Ok(Self { value: i as u32 }), // SAFETY: i < i32::MAX < u32::MAX
                Value::Integer(i) => Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: format!("Repeat count must be non-negative: {i}"),
                    span: prop.span,
                }]),
                v => Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueType::Integer,
                    found: v.into_kind(),
                    span: prop.span,
                }]),
            },
            len => Err(vec![TypedError::PropertyInvalidValueCount {
                property: PropertyKind::Repeat,
                expected: 1,
                found: len,
                span: prop.span,
            }]),
        }
    }
}

/// Trigger for alarms (RFC 5545 Section 3.8.6.3)
#[derive(Debug, Clone)]
pub struct Trigger<'src> {
    /// When to trigger (relative or absolute)
    pub value: TriggerValue<'src>,

    /// Related parameter for relative triggers
    pub related: Option<AlarmTriggerRelationship>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue<'src> {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime<'src>),
}

impl<'src> TryFrom<ParsedProperty<'src>> for Trigger<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Trigger) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Trigger,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect the RELATED parameter (optional, default is START)
        let mut related = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::AlarmTriggerRelationship { .. } if related.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::AlarmTriggerRelationship { value, .. } => related = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let (value, _) = take_single_value(&PropertyKind::Trigger, prop.values)?;

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Duration(dur) => Ok(Trigger {
                value: TriggerValue::Duration(dur),
                related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
                x_parameters,
                unrecognized_parameters,
            }),
            Value::DateTime(dt) => Ok(Trigger {
                value: TriggerValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                }),
                related: None,
                x_parameters,
                unrecognized_parameters,
            }),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Trigger,
                value: "Expected duration or date-time value".to_string(),
                span: prop.span,
            }]),
        }
    }
}
