// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use std::{convert::TryFrom, fmt::Display, str::FromStr};

use crate::keyword::{
    KW_ACTION_AUDIO, KW_ACTION_DISPLAY, KW_ACTION_EMAIL, KW_ACTION_PROCEDURE, KW_VALARM,
};
use crate::semantic::property_common::{
    take_single_int, take_single_value, take_single_value_string,
};
use crate::semantic::{Attachment, Attendee, SemanticError, Text, Trigger, TriggerValue};
use crate::typed::ValueDuration;
use crate::typed::{PropertyKind, TypedComponent, Value, ValueType};

/// Alarm component (VALARM)
#[derive(Debug, Clone)]
pub struct VAlarm<'src> {
    /// Action to perform when alarm triggers
    pub action: Action,

    /// When to trigger the alarm
    pub trigger: Trigger<'src>,

    /// Repeat count for the alarm
    pub repeat: Option<u32>,

    /// Duration between repeats
    pub duration: Option<ValueDuration>,

    /// Description for display alarm
    pub description: Option<Text<'src>>,

    /// Summary for email alarm
    pub summary: Option<Text<'src>>,

    /// Attendees for email alarm
    pub attendees: Vec<Attendee<'src>>,

    /// Attachment for audio/procedure alarm
    pub attach: Option<Attachment<'src>>,
}

/// Parse a `TypedComponent` into a `VAlarm`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VAlarm<'src> {
    type Error = Vec<SemanticError>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
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
                PropertyKind::Action => {
                    let value = match take_single_value_string(prop.kind, prop.values) {
                        Ok(text) => match text.to_uppercase().parse() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                errors.push(SemanticError::InvalidValue {
                                    property: PropertyKind::Action,
                                    value: e,
                                });
                                Some(Action::Audio)
                            }
                        },
                        Err(e) => {
                            errors.push(e);
                            Some(Action::Audio)
                        }
                    };

                    match props.action {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Action,
                        }),
                        None => props.action = value,
                    }
                }
                PropertyKind::Trigger => {
                    let value = match Trigger::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            Some(Trigger {
                                value: TriggerValue::Duration(ValueDuration::DateTime {
                                    positive: true,
                                    day: 0,
                                    hour: 0,
                                    minute: 0,
                                    second: 0,
                                }),
                                related: None,
                            })
                        }
                    };

                    match props.trigger {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Trigger,
                        }),
                        None => props.trigger = value,
                    }
                }
                PropertyKind::Duration => {
                    let value = match take_single_value(prop.kind, prop.values) {
                        Ok(Value::Duration(v)) => Some(v),
                        Ok(_) => {
                            errors.push(SemanticError::UnexpectedType {
                                property: PropertyKind::Duration,
                                expected: ValueType::Duration,
                            });
                            Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            })
                        }
                        Err(e) => {
                            errors.push(e);
                            Some(ValueDuration::DateTime {
                                positive: true,
                                day: 0,
                                hour: 0,
                                minute: 0,
                                second: 0,
                            })
                        }
                    };

                    match props.duration {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Duration,
                        }),
                        None => props.duration = value,
                    }
                }
                PropertyKind::Repeat => {
                    let value = match take_single_int(prop.kind, prop.values) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.push(e);
                            Some(0)
                        }
                    };

                    match props.repeat {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Repeat,
                        }),
                        None => props.repeat = value,
                    }
                }
                PropertyKind::Description => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.description {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Description,
                        }),
                        None => props.description = value,
                    }
                }
                PropertyKind::Summary => {
                    let value = match Text::try_from(prop) {
                        Ok(text) => Some(text),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.summary {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Summary,
                        }),
                        None => props.summary = value,
                    }
                }
                PropertyKind::Attendee => match Attendee::try_from(prop) {
                    Ok(v) => props.attendees.push(v),
                    Err(e) => errors.extend(e),
                },
                PropertyKind::Attach => {
                    let value = match Attachment::try_from(prop) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            errors.extend(e);
                            None
                        }
                    };

                    match props.attach {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Attach,
                        }),
                        None => props.attach = value,
                    }
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
        let action = props.action.unwrap_or(Action::Audio);

        // Validate DESCRIPTION is present for DISPLAY and EMAIL actions
        if props.description.is_none() && matches!(action, Action::Display | Action::Email) {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Description,
            });
        }

        // Validate SUMMARY is present for EMAIL action
        if props.summary.is_none() && matches!(action, Action::Email) {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Summary,
            });
        }

        // Validate ATTENDEE is present for EMAIL action
        if matches!(action, Action::Email) && props.attendees.is_empty() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Attendee,
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
            attendees: props.attendees,
            attach: props.attach,
        })
    }
}

/// Alarm action
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

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    action:     Option<Action>,
    trigger:    Option<Trigger<'src>>,
    duration:   Option<ValueDuration>,
    repeat:     Option<u32>,
    description: Option<Text<'src>>,
    summary:    Option<Text<'src>>,
    attendees:  Vec<Attendee<'src>>,
    attach:     Option<Attachment<'src>>,
}
