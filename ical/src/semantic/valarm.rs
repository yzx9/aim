// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use crate::keyword::KW_VALARM;
use crate::property::{Action, Attachment, Attendee, Property, PropertyKind, Text, Trigger};
use crate::semantic::SemanticError;
use crate::typed::TypedComponent;
use crate::value::ValueDuration;

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
impl<'src> TryFrom<TypedComponent<'src>> for VAlarm<'src> {
    type Error = Vec<SemanticError>;

    #[expect(clippy::too_many_lines)]
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
            match prop {
                Property::Action(action) => match props.action {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Action,
                    }),
                    None => props.action = Some(action),
                },
                Property::Trigger(trigger) => match props.trigger {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Trigger,
                    }),
                    None => props.trigger = Some(trigger),
                },
                Property::Duration(duration) => match props.duration {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                    }),
                    None => props.duration = Some(duration.value),
                },
                Property::Repeat(repeat) => match props.repeat {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Repeat,
                    }),
                    None => props.repeat = Some(repeat.value),
                },
                Property::Description(text) => match props.description {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                    }),
                    None => props.description = Some(text),
                },
                Property::Summary(text) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                    }),
                    None => props.summary = Some(text),
                },
                Property::Attendee(attendee) => {
                    props.attendees.push(attendee);
                }
                Property::Attach(attach) => match props.attach {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Attach,
                    }),
                    None => props.attach = Some(attach),
                },
                // Ignore other properties not used by VAlarm
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
