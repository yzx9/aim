// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use crate::TriggerValue;
use crate::keyword::{
    KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE, KW_VALARM,
};
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    find_parameter, get_language, get_single_value, parse_attendee_property, value_to_date_time,
    value_to_int, value_to_string,
};
use crate::semantic::properties::{Attachment, AttachmentValue, Attendee, Text, Trigger};
use crate::typed::ValueDuration;
use crate::typed::parameter_type::AlarmTriggerRelationship;
use crate::typed::{
    Encoding, PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty,
    Value,
};

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
    pub duration: Option<ValueDuration>,

    /// Description for display alarm
    pub description: Option<Text>,

    /// Summary for email alarm
    pub summary: Option<Text>,

    /// Attendees for email alarm
    pub attendees: Vec<Attendee>,

    /// Attachment for audio/procedure alarm
    pub attach: Option<Attachment>,
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
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    action:     Option<&'a TypedProperty<'a>>,
    trigger:    Option<&'a TypedProperty<'a>>,
    duration:   Option<&'a TypedProperty<'a>>,
    repeat:     Option<&'a TypedProperty<'a>>,
    description: Option<&'a TypedProperty<'a>>,
    summary:    Option<&'a TypedProperty<'a>>,
    attendees:  Vec<&'a TypedProperty<'a>>,
    attach:     Option<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VAlarm`
#[allow(clippy::too_many_lines)]
pub fn parse_valarm(comp: &TypedComponent) -> Result<VAlarm, Vec<SemanticError>> {
    if comp.name != KW_VALARM {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VALARM component, got '{}'",
            comp.name
        ))]);
    }

    let mut errors = Vec::new();

    // Collect all properties in a single pass
    let mut props = PropertyCollector::default();
    for prop in &comp.properties {
        match prop.name {
            name if name == PropertyKind::Action.as_str() => {
                if props.action.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Action.as_str()
                    )));
                } else {
                    props.action = Some(prop);
                }
            }
            name if name == PropertyKind::Trigger.as_str() => {
                if props.trigger.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Trigger.as_str()
                    )));
                } else {
                    props.trigger = Some(prop);
                }
            }
            name if name == PropertyKind::Duration.as_str() => {
                if props.duration.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Duration.as_str()
                    )));
                } else {
                    props.duration = Some(prop);
                }
            }
            name if name == PropertyKind::Repeat.as_str() => {
                if props.repeat.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Repeat.as_str()
                    )));
                } else {
                    props.repeat = Some(prop);
                }
            }
            name if name == PropertyKind::Description.as_str() => {
                if props.description.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Description.as_str()
                    )));
                } else {
                    props.description = Some(prop);
                }
            }
            name if name == PropertyKind::Summary.as_str() => {
                if props.summary.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Summary.as_str()
                    )));
                } else {
                    props.summary = Some(prop);
                }
            }
            name if name == PropertyKind::Attendee.as_str() => {
                props.attendees.push(prop);
            }
            name if name == PropertyKind::Attach.as_str() => {
                if props.attach.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Attach.as_str()
                    )));
                } else {
                    props.attach = Some(prop);
                }
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // ACTION is required
    let action = match props.action {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(text) => match text.to_uppercase().as_str() {
                    KW_ACTION_AUDIO => AlarmActionType::Audio,
                    KW_ACTION_DISPLAY => AlarmActionType::Display,
                    KW_ACTION_EMAIL => AlarmActionType::Email,
                    KW_ACTION_PROCEDURE => AlarmActionType::Procedure,
                    _ => {
                        errors.push(SemanticError::InvalidValue(
                            PropertyKind::Action.as_str().to_string(),
                            format!("Invalid action: {text}"),
                        ));
                        // Default to AUDIO for parsing to continue
                        AlarmActionType::Audio
                    }
                },
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Action.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    AlarmActionType::Audio
                }
            },
            Err(e) => {
                errors.push(e);
                AlarmActionType::Audio
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::Action.as_str().to_string(),
            ));
            AlarmActionType::Audio
        }
    };

    // TRIGGER is required
    let trigger = match props.trigger {
        Some(prop) => match parse_trigger(prop) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                // Return a dummy trigger to continue parsing
                Trigger {
                    value: TriggerValue::Duration(ValueDuration::DateTime {
                        positive: true,
                        day: 0,
                        hour: 0,
                        minute: 0,
                        second: 0,
                    }),
                    related: None,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::Trigger.as_str().to_string(),
            ));
            Trigger {
                value: TriggerValue::Duration(ValueDuration::DateTime {
                    positive: true,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                }),
                related: None,
            }
        }
    };

    // DURATION and REPEAT must appear together or not at all
    let has_duration = props.duration.is_some();
    let has_repeat = props.repeat.is_some();

    if has_duration != has_repeat {
        errors.push(SemanticError::InvalidStructure(
            "DURATION and REPEAT must appear together or not at all".to_string(),
        ));
    }

    // DURATION is optional
    let duration = props.duration.map(|prop| match get_single_value(prop) {
        Ok(Value::Duration(v)) => *v,
        Ok(_) => {
            errors.push(SemanticError::InvalidValue(
                PropertyKind::Duration.as_str().to_string(),
                "Expected duration value".to_string(),
            ));
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
        Err(e) => {
            errors.push(e);
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
    });

    // REPEAT is optional
    let repeat = props.repeat.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_int::<u32>(value) {
            Some(v) => v,
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Repeat.as_str().to_string(),
                    "Expected integer value".to_string(),
                ));
                0
            }
        },
        Err(e) => {
            errors.push(e);
            0
        }
    });

    // DESCRIPTION is REQUIRED for DISPLAY and EMAIL actions
    let description = match props.description {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => Some(Text {
                    content: v,
                    language: get_language(&prop.parameters),
                }),
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Description.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => {
            if matches!(action, AlarmActionType::Display | AlarmActionType::Email) {
                errors.push(SemanticError::MissingProperty(
                    PropertyKind::Description.as_str().to_string(),
                ));
            }
            None
        }
    };

    // SUMMARY is REQUIRED for EMAIL action
    let summary = match props.summary {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => Some(Text {
                    content: v,
                    language: get_language(&prop.parameters),
                }),
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Summary.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    None
                }
            },
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => {
            if matches!(action, AlarmActionType::Email) {
                errors.push(SemanticError::MissingProperty(
                    PropertyKind::Summary.as_str().to_string(),
                ));
            }
            None
        }
    };

    // ATTENDEE is REQUIRED for EMAIL action
    let attendees: Vec<Attendee> = props
        .attendees
        .into_iter()
        .filter_map(|prop| match parse_attendee_property(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        })
        .collect();

    if matches!(action, AlarmActionType::Email) && attendees.is_empty() {
        errors.push(SemanticError::MissingProperty(
            PropertyKind::Attendee.as_str().to_string(),
        ));
    }

    // ATTACH is REQUIRED for PROCEDURE action, optional for AUDIO
    let attach = match props.attach {
        Some(prop) => match parse_attachment(prop) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        },
        None => {
            if matches!(action, AlarmActionType::Procedure) {
                errors.push(SemanticError::MissingProperty(
                    PropertyKind::Attach.as_str().to_string(),
                ));
            }
            None
        }
    };

    // If we have errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(VAlarm {
        action,
        trigger,
        repeat,
        duration,
        description,
        summary,
        attendees,
        attach,
    })
}

/// Parse a TRIGGER property into a Trigger
fn parse_trigger(prop: &TypedProperty<'_>) -> Result<Trigger, SemanticError> {
    // Get the RELATED parameter (optional, default is START)
    let related = find_parameter(
        &prop.parameters,
        TypedParameterKind::AlarmTriggerRelationship,
    )
    .and_then(|p| match p {
        TypedParameter::AlarmTriggerRelationship { value, .. } => Some(*value),
        _ => None,
    });

    let value = get_single_value(prop)?;

    // Try to parse as duration first (most common)
    if let Value::Duration(dur) = value {
        return Ok(Trigger {
            value: TriggerValue::Duration(*dur),
            related: Some(related.unwrap_or(AlarmTriggerRelationship::Start)),
        });
    }

    // Try to parse as date-time
    if let Some(dt) = value_to_date_time(value) {
        return Ok(Trigger {
            value: TriggerValue::DateTime(dt),
            related: None,
        });
    }

    Err(SemanticError::InvalidValue(
        PropertyKind::Trigger.as_str().to_string(),
        "Expected duration or date-time value".to_string(),
    ))
}

/// Parse an ATTACH property into an Attachment
fn parse_attachment(prop: &TypedProperty<'_>) -> Result<Attachment, SemanticError> {
    let value = get_single_value(prop)?;

    // Get FMTTYPE parameter
    let fmt_type =
        find_parameter(&prop.parameters, TypedParameterKind::FormatType).and_then(|p| match p {
            TypedParameter::FormatType { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Get ENCODING parameter
    let encoding =
        find_parameter(&prop.parameters, TypedParameterKind::Encoding).and_then(|p| match p {
            TypedParameter::Encoding { value, .. } => Some(*value),
            _ => None,
        });

    match value {
        Value::Text(uri) => Ok(Attachment {
            value: AttachmentValue::Uri(crate::semantic::properties::Uri {
                uri: uri.resolve().to_string(),
            }),
            fmt_type,
            encoding: encoding.map(|e| match e {
                Encoding::Bit8 => Encoding::Bit8,
                Encoding::Base64 => Encoding::Base64,
            }),
        }),
        Value::Binary(data) => {
            // Convert SpannedSegments to String, then to Vec<u8>
            let data_str = data.resolve().to_string();
            Ok(Attachment {
                value: AttachmentValue::Binary(data_str.into_bytes()),
                fmt_type,
                encoding: encoding.map(|e| match e {
                    Encoding::Bit8 => Encoding::Bit8,
                    Encoding::Base64 => Encoding::Base64,
                }),
            })
        }
        _ => Err(SemanticError::InvalidValue(
            PropertyKind::Attach.as_str().to_string(),
            "Expected URI or binary value".to_string(),
        )),
    }
}
