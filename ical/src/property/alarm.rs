// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm Component Properties (RFC 5545 Section 3.8.6)
//!
//! This module contains property types for the "Alarm Component Properties"
//! section of RFC 5545, including:
//! - 3.8.6.1 `Action`
//! - 3.8.6.3 `Trigger`

use std::{convert::TryFrom, fmt::Display, str::FromStr};

use crate::keyword::{KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE};
use crate::parameter::{AlarmTriggerRelationship, TypedParameter, TypedParameterKind};
use crate::property::DateTime;
use crate::semantic::SemanticError;
use crate::typed::{PropertyKind, TypedProperty, Value, ValueType};
use crate::value::ValueDuration;

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
            KW_ACTION_PROCEDURE => Err("PROCEDURE action has been deprecated".to_string()),
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

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<TypedProperty<'src>> for Action {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let text = prop
            .values
            .first()
            .and_then(|v| match v {
                Value::Text(t) => Some(t.resolve().to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                vec![SemanticError::UnexpectedType {
                    property: PropertyKind::Action,
                    expected: ValueType::Text,
                }]
            })?;

        text.parse().map_err(|e| {
            vec![SemanticError::InvalidValue {
                property: PropertyKind::Action,
                value: e,
            }]
        })
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

impl<'src> TryFrom<TypedProperty<'src>> for Trigger<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(prop: TypedProperty<'_>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        // Collect the RELATED parameter (optional, default is START)
        let mut related = None;

        for param in &prop.parameters {
            #[allow(clippy::single_match)]
            match param {
                TypedParameter::AlarmTriggerRelationship { value, .. } => match related {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::AlarmTriggerRelationship,
                    }),
                    None => related = Some(*value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        let Some(value) = prop.values.first() else {
            return Err(vec![SemanticError::MissingValue {
                property: PropertyKind::Trigger,
            }]);
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Duration(dur) => Ok(Trigger {
                value: TriggerValue::Duration(*dur),
                related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
            }),
            Value::DateTime(dt) => Ok(Trigger {
                value: TriggerValue::DateTime(DateTime::Floating {
                    date: dt.date,
                    time: dt.time.into(),
                }),
                related: None,
            }),
            _ => Err(vec![SemanticError::InvalidValue {
                property: PropertyKind::Trigger,
                value: "Expected duration or date-time value".to_string(),
            }]),
        }
    }
}
