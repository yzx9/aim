// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use std::convert::TryFrom;

use crate::TriggerValue;
use crate::keyword::{
    KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE, KW_VALARM,
};
use crate::semantic::property_util::{
    get_language, get_single_value, value_to_int, value_to_string,
};
use crate::semantic::{Attachment, Attendee, SemanticError, Text, Trigger};
use crate::typed::ValueDuration;
use crate::typed::{PropertyKind, TypedComponent, TypedProperty, Value, ValueType};

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

/// Parse a `TypedComponent` into a `VAlarm`
#[allow(clippy::too_many_lines)]
impl TryFrom<TypedComponent<'_>> for VAlarm {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'_>) -> Result<Self, Self::Error> {
        if comp.name != KW_VALARM {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VALARM,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop.kind {
                PropertyKind::Action if props.action.is_none() => match get_single_value(&prop) {
                    Ok(value) => match value_to_string(value) {
                        Some(text) => match text.to_uppercase().as_str() {
                            KW_ACTION_AUDIO => props.action = Some(AlarmActionType::Audio),
                            KW_ACTION_DISPLAY => props.action = Some(AlarmActionType::Display),
                            KW_ACTION_EMAIL => props.action = Some(AlarmActionType::Email),
                            KW_ACTION_PROCEDURE => props.action = Some(AlarmActionType::Procedure),
                            _ => {
                                errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Action,
                                    value: format!("Invalid action: {text}"),
                                });
                                props.action = Some(AlarmActionType::Audio);
                            }
                        },
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Action,
                                expected: ValueType::Text,
                            });
                            props.action = Some(AlarmActionType::Audio);
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        props.action = Some(AlarmActionType::Audio);
                    }
                },
                PropertyKind::Action => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Action,
                    });
                }
                PropertyKind::Trigger if props.trigger.is_none() => match Trigger::try_from(prop) {
                    Ok(v) => props.trigger = Some(v),
                    Err(e) => {
                        errors.push(e);
                        props.trigger = Some(Trigger {
                            value: TriggerValue::Duration(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            }),
                            related: None,
                        });
                    }
                },
                PropertyKind::Trigger => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Trigger,
                    });
                }
                PropertyKind::Duration if props.duration.is_none() => {
                    match get_single_value(&prop) {
                        Ok(Value::Duration(v)) => props.duration = Some(*v),
                        Ok(_) => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Duration,
                                expected: ValueType::Duration,
                            });
                            props.duration = Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            });
                        }
                        Err(e) => {
                            errors.push(e);
                            props.duration = Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            });
                        }
                    }
                }
                PropertyKind::Duration => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                    });
                }
                PropertyKind::Repeat if props.repeat.is_none() => match get_single_value(&prop) {
                    Ok(value) => match value_to_int::<u32>(value) {
                        Some(v) => props.repeat = Some(v),
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Repeat,
                                expected: ValueType::Integer,
                            });
                            props.repeat = Some(0);
                        }
                    },
                    Err(e) => {
                        errors.push(e);
                        props.repeat = Some(0);
                    }
                },
                PropertyKind::Repeat => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Repeat,
                    });
                }
                PropertyKind::Description if props.description.is_none() => {
                    match get_single_value(&prop) {
                        Ok(value) => match value_to_string(value) {
                            Some(v) => {
                                props.description = Some(Text {
                                    content: v,
                                    language: get_language(&prop.parameters),
                                });
                            }
                            None => {
                                errors.push(SemanticError::ExpectedType {
                                    property: PropertyKind::Description,
                                    expected: ValueType::Text,
                                });
                            }
                        },
                        Err(e) => errors.push(e),
                    }
                }
                PropertyKind::Description => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                    });
                }
                PropertyKind::Summary if props.summary.is_none() => match get_single_value(&prop) {
                    Ok(value) => match value_to_string(value) {
                        Some(v) => {
                            props.summary = Some(Text {
                                content: v,
                                language: get_language(&prop.parameters),
                            });
                        }
                        None => {
                            errors.push(SemanticError::ExpectedType {
                                property: PropertyKind::Summary,
                                expected: ValueType::Text,
                            });
                        }
                    },
                    Err(e) => errors.push(e),
                },
                PropertyKind::Summary => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                    });
                }
                PropertyKind::Attendee => {
                    props.attendees.push(prop);
                }
                PropertyKind::Attach if props.attach.is_none() => {
                    match Attachment::try_from(prop) {
                        Ok(v) => props.attach = Some(v),
                        Err(e) => {
                            errors.push(e);
                            // Continue without attach - validation will catch it if required
                        }
                    }
                }
                PropertyKind::Attach => {
                    errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Attach,
                    });
                }
                // Ignore unknown properties
                _ => {}
            }
        }

        // Check required fields
        if props.action.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Action,
            });
        }
        if props.trigger.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Trigger,
            });
        }

        // DURATION and REPEAT must appear together or not at all
        let has_duration = props.duration.is_some();
        let has_repeat = props.repeat.is_some();
        if has_duration != has_repeat {
            errors.push(SemanticError::ConstraintViolation {
                message: "DURATION and REPEAT must appear together or not at all".to_string(),
            });
        }

        // Get action for validation checks
        let action = props.action.unwrap_or(AlarmActionType::Audio);

        // Validate DESCRIPTION is present for DISPLAY and EMAIL actions
        if props.description.is_none()
            && matches!(action, AlarmActionType::Display | AlarmActionType::Email)
        {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Description,
            });
        }

        // Validate SUMMARY is present for EMAIL action
        if props.summary.is_none() && matches!(action, AlarmActionType::Email) {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Summary,
            });
        }

        // Parse ATTENDEE properties (REQUIRED for EMAIL action)
        let attendees: Vec<Attendee> = props
            .attendees
            .into_iter()
            .filter_map(|prop| match Attendee::try_from(prop) {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

        if matches!(action, AlarmActionType::Email) && attendees.is_empty() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Attendee,
            });
        }

        // Validate ATTACH is present for PROCEDURE action
        if props.attach.is_none() && matches!(action, AlarmActionType::Procedure) {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Attach,
            });
        }

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VAlarm {
            action: props.action.unwrap(),   // SAFETY: checked above
            trigger: props.trigger.unwrap(), // SAFETY: checked above
            repeat: props.repeat,
            duration: props.duration,
            description: props.description,
            summary: props.summary,
            attendees,
            attach: props.attach,
        })
    }
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
    action:     Option<AlarmActionType>,
    trigger:    Option<Trigger>,
    duration:   Option<ValueDuration>,
    repeat:     Option<u32>,
    description: Option<Text>,
    summary:    Option<Text>,
    attendees:  Vec<TypedProperty<'a>>,
    attach:     Option<Attachment>,
}
