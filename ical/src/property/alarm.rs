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

use std::convert::TryFrom;

use crate::keyword::{KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE};
use crate::parameter::{AlarmTriggerRelationship, Parameter, ValueTypeRef};
use crate::property::common::{take_single_text, take_single_value};
use crate::property::{DateTime, PropertyKind};
use crate::string_storage::{SpannedSegments, StringStorage};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueDuration};

define_prop_value_enum! {
    /// Alarm action value (RFC 5545 Section 3.8.6.1)
    pub enum ActionValue {
        /// Audio alarm
        Audio => KW_ACTION_AUDIO,

        /// Display alarm
        Display => KW_ACTION_DISPLAY,

        /// Email alarm
        Email => KW_ACTION_EMAIL,
    }
}

impl ActionValue {
    /// Get the keyword string for this action value
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Audio => KW_ACTION_AUDIO,
            Self::Display => KW_ACTION_DISPLAY,
            Self::Email => KW_ACTION_EMAIL,
        }
    }
}

impl AsRef<str> for ActionValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Alarm action (RFC 5545 Section 3.8.6.1)
#[derive(Debug, Clone)]
pub struct Action<S: StringStorage> {
    /// Action value
    pub value: ActionValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Action<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Action) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Action,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Action, prop.value)?;

        // Check for deprecated PROCEDURE action first
        if text.eq_str_ignore_ascii_case(KW_ACTION_PROCEDURE) {
            return Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Action,
                value: format!("{KW_ACTION_PROCEDURE} action has been deprecated"),
                span: value_span,
            }]);
        }

        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Action,
                value: format!("Invalid alarm action: {text}"),
                span: value_span,
            }]
        })?;

        Ok(Action {
            value,
            x_parameters,
            unrecognized_parameters,
            span: prop.span,
        })
    }
}

impl Action<SpannedSegments<'_>> {
    /// Convert borrowed Action to owned Action
    #[must_use]
    pub fn to_owned(&self) -> Action<String> {
        Action {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

/// Repeat Count (RFC 5545 Section 3.8.6.2)
///
/// This property defines the number of times the alarm should repeat.
#[derive(Debug, Clone)]
pub struct Repeat<S: StringStorage> {
    /// Number of repetitions
    pub value: u32,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Repeat<SpannedSegments<'src>> {
    type Error = Vec<TypedError<'src>>;

    #[allow(clippy::cast_sign_loss)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Repeat) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Repeat,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        match prop.value {
            Value::Integer { values, .. } if values.is_empty() => {
                Err(vec![TypedError::PropertyMissingValue {
                    property: prop.kind,
                    span: prop.span,
                }])
            }
            Value::Integer {
                values: mut ints, ..
            } if ints.len() == 1 => {
                let i = ints.pop().unwrap();
                if i >= 0 {
                    Ok(Repeat {
                        value: i as u32, // SAFETY: i < i32::MAX < u32::MAX
                        x_parameters,
                        unrecognized_parameters,
                        span: prop.span,
                    })
                } else {
                    Err(vec![TypedError::PropertyInvalidValue {
                        property: prop.kind,
                        value: format!("Repeat count must be non-negative: {i}"),
                        span: value_span,
                    }])
                }
            }
            Value::Integer { values: ints, .. } => {
                Err(vec![TypedError::PropertyInvalidValueCount {
                    property: PropertyKind::Repeat,
                    expected: 1,
                    found: ints.len(),
                    span: value_span,
                }])
            }
            v => {
                let span = v.span();
                Err(vec![TypedError::PropertyUnexpectedValue {
                    property: prop.kind,
                    expected: ValueTypeRef::Integer,
                    found: v.kind().into(),
                    span,
                }])
            }
        }
    }
}

impl Repeat<SpannedSegments<'_>> {
    /// Convert borrowed Repeat to owned Repeat
    #[must_use]
    pub fn to_owned(&self) -> Repeat<String> {
        Repeat {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

/// Trigger for alarms (RFC 5545 Section 3.8.6.3)
#[derive(Debug, Clone)]
pub struct Trigger<S: StringStorage> {
    /// When to trigger (relative or absolute)
    pub value: TriggerValue<S>,

    /// Related parameter for relative triggers
    pub related: Option<AlarmTriggerRelationship>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

/// Type alias for borrowed trigger
pub type TriggerRef<'src> = Trigger<SpannedSegments<'src>>;

/// Type alias for owned trigger
pub type TriggerOwned = Trigger<String>;

/// Trigger value (relative duration or absolute date/time)
#[derive(Debug, Clone)]
pub enum TriggerValue<S: StringStorage> {
    /// Relative duration before/after the event
    Duration(ValueDuration),

    /// Absolute date/time
    DateTime(DateTime<S>),
}

/// Type alias for borrowed trigger value
pub type TriggerValueRef<'src> = TriggerValue<SpannedSegments<'src>>;

/// Type alias for owned trigger value
pub type TriggerValueOwned = TriggerValue<String>;

impl<'src> TryFrom<ParsedProperty<'src>> for Trigger<SpannedSegments<'src>> {
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
                        parameter: p.kind().into(),
                    });
                }
                Parameter::AlarmTriggerRelationship { value, .. } => related = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value = take_single_value(&PropertyKind::Trigger, prop.value)?;

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Duration { values: durs, .. } if durs.len() == 1 => Ok(Trigger {
                value: TriggerValue::Duration(durs.into_iter().next().unwrap()),
                related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
                x_parameters,
                unrecognized_parameters,
                span: prop.span,
            }),
            Value::DateTime { values: dts, .. } if dts.len() == 1 => {
                let dt = dts.into_iter().next().unwrap();
                Ok(Trigger {
                    value: TriggerValue::DateTime(DateTime::Floating {
                        date: dt.date,
                        time: dt.time.into(),
                        x_parameters: Vec::new(),
                        unrecognized_parameters: Vec::new(),
                    }),
                    related: None,
                    x_parameters,
                    unrecognized_parameters,
                    span: prop.span,
                })
            }
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Trigger,
                value: "Expected duration or date-time value".to_string(),
                span: value.span(),
            }]),
        }
    }
}

impl Trigger<SpannedSegments<'_>> {
    /// Convert borrowed Trigger to owned Trigger
    #[must_use]
    pub fn to_owned(&self) -> Trigger<String> {
        Trigger {
            value: self.value.to_owned(),
            related: self.related,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

impl TriggerValue<SpannedSegments<'_>> {
    /// Convert borrowed `TriggerValue` to owned `TriggerValue`
    #[must_use]
    pub fn to_owned(&self) -> TriggerValue<String> {
        match self {
            TriggerValue::Duration(duration) => TriggerValue::Duration(*duration),
            TriggerValue::DateTime(dt) => TriggerValue::DateTime(dt.to_owned()),
        }
    }
}
