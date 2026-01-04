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
use crate::parameter::{AlarmTriggerRelationship, Parameter, ValueKind};
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

impl Action {
    /// Get the property kind for `Action`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Action
    }
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
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let span = prop.span;
        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Action,
                value: e,
                span,
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

impl Repeat {
    /// Get the property kind for `Repeat`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Repeat
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Repeat {
    type Error = Vec<TypedError<'src>>;

    #[allow(clippy::cast_sign_loss)]
    fn try_from(mut prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
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
                Value::Integer(i) if i > 0 => Ok(Self { value: i as u32 }),
                Value::Integer(i) => Err(vec![TypedError::PropertyInvalidValue {
                    property: prop.kind,
                    value: format!("Repeat count must be non-negative: {i}"),
                    span: prop.span,
                }]),
                v => Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueKind::Integer,
                    found: v.kind(),
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
}

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue<'src> {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime<'src>),
}

impl Trigger<'_> {
    /// Get the property kind for `Trigger`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Trigger
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Trigger<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect the RELATED parameter (optional, default is START)
        let mut related = None;

        for param in &prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            #[expect(clippy::single_match)]
            match param {
                Parameter::AlarmTriggerRelationship { value, .. } => match related {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => related = Some(*value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        let (value, _) =
            take_single_value(PropertyKind::Trigger, prop.values).map_err(|e| vec![e])?;

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Duration(dur) => Ok(Trigger {
                value: TriggerValue::Duration(dur),
                related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
            }),
            Value::DateTime(dt) => Ok(Trigger {
                value: TriggerValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                related: None,
            }),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Trigger,
                value: "Expected duration or date-time value".to_string(),
                span: prop.span,
            }]),
        }
    }
}
