// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Alarm component (VALARM) for iCalendar semantic components.

use std::fmt::Display;

use crate::keyword::KW_VALARM;
use crate::property::{
    Action, ActionValue, Attachment, Attendee, Description, Property, PropertyKind, Repeat,
    Summary, Trigger,
};
use crate::semantic::SemanticError;
use crate::syntax::SpannedSegments;
use crate::typed::TypedComponent;
use crate::value::ValueDuration;

/// Alarm component (VALARM)
#[derive(Debug, Clone)]
pub struct VAlarm<S: Clone + Display> {
    /// Action to perform when alarm triggers
    pub action: Action<S>,

    /// When to trigger the alarm
    pub trigger: Trigger<S>,

    /// Repeat count for the alarm
    pub repeat: Option<Repeat<S>>,

    /// Duration between repeats
    pub duration: Option<ValueDuration>,

    /// Description for display alarm
    pub description: Option<Description<S>>,

    /// Summary for email alarm
    pub summary: Option<Summary<S>>,

    /// Attendees for email alarm
    pub attendees: Vec<Attendee<S>>,

    /// Attachment for audio/procedure alarm
    pub attach: Option<Attachment<S>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<S>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<S>>,
}

/// Type alias for `VAlarm` with borrowed data
pub type VAlarmRef<'src> = VAlarm<SpannedSegments<'src>>;

/// Type alias for `VAlarm` with owned data
pub type VAlarmOwned<'src> = VAlarm<String>;

/// Parse a `TypedComponent` into a `VAlarm`
impl<'src> TryFrom<TypedComponent<'src>> for VAlarm<SpannedSegments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        if comp.name != KW_VALARM {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VALARM,
                got: comp.name,
                span: comp.span,
            });
        }

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                // TODO: Use property span instead of component span for DuplicateProperty
                Property::Action(action) => match props.action {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Action,
                        span: comp.span,
                    }),
                    None => props.action = Some(action),
                },
                Property::Trigger(trigger) => match props.trigger {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Trigger,
                        span: comp.span,
                    }),
                    None => props.trigger = Some(trigger),
                },
                Property::Duration(duration) => match props.duration {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                        span: comp.span,
                    }),
                    None => props.duration = Some(duration.value),
                },
                Property::Repeat(repeat) => match props.repeat {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Repeat,
                        span: comp.span,
                    }),
                    None => props.repeat = Some(repeat),
                },
                Property::Description(desc) => match props.description {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                        span: comp.span,
                    }),
                    None => props.description = Some(desc),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                        span: comp.span,
                    }),
                    None => props.summary = Some(s),
                },
                Property::Attendee(attendee) => props.attendees.push(attendee),
                Property::Attach(attach) => match props.attach {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Attach,
                        span: comp.span,
                    }),
                    None => props.attach = Some(attach),
                },
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VAlarm for round-trip
                    props.unrecognized_properties.push(prop);
                }
            }
        }

        // Check required fields
        if props.action.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Action,
                span: comp.span,
            });
        }
        if props.trigger.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Trigger,
                span: comp.span,
            });
        }

        // DURATION and REPEAT must appear together or not at all
        let has_duration = props.duration.is_some();
        let has_repeat = props.repeat.is_some();
        if has_duration != has_repeat {
            errors.push(SemanticError::ConstraintViolation {
                message: "DURATION and REPEAT must appear together or not at all".to_string(),
                span: comp.span,
            });
        }

        // Get action for validation checks
        let default_action = Action {
            value: ActionValue::Audio,
            x_parameters: Vec::new(),
            unrecognized_parameters: Vec::new(),
        };
        let action = props.action.as_ref().unwrap_or(&default_action);

        // Validate DESCRIPTION is present for DISPLAY and EMAIL actions
        if props.description.is_none()
            && matches!(action.value, ActionValue::Display | ActionValue::Email)
        {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Description,
                span: comp.span,
            });
        }

        // Validate SUMMARY is present for EMAIL action
        if props.summary.is_none() && matches!(action.value, ActionValue::Email) {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Summary,
                span: comp.span,
            });
        }

        // Validate ATTENDEE is present for EMAIL action
        if matches!(action.value, ActionValue::Email) && props.attendees.is_empty() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Attendee,
                span: comp.span,
            });
        }

        if errors.is_empty() {
            Ok(VAlarm {
                action: props.action.unwrap_or(default_action),
                trigger: props.trigger.unwrap(), // SAFETY: checked above
                repeat: props.repeat,
                duration: props.duration,
                description: props.description,
                summary: props.summary,
                attendees: props.attendees,
                attach: props.attach,
                x_properties: props.x_properties,
                unrecognized_properties: props.unrecognized_properties,
            })
        } else {
            Err(errors)
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<S: Clone + Display> {
    action:     Option<Action<S>>,
    trigger:    Option<Trigger<S>>,
    duration:   Option<ValueDuration>,
    repeat:     Option<Repeat<S>>,
    description: Option<Description<S>>,
    summary:    Option<Summary<S>>,
    attendees:  Vec<Attendee<S>>,
    attach:     Option<Attachment<S>>,
    x_properties: Vec<Property<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

impl<'src> VAlarmRef<'src> {
    /// Convert borrowed data to owned data
    #[must_use]
    pub fn to_owned(&self) -> VAlarmOwned<'src> {
        VAlarmOwned {
            action: self.action.to_owned(),
            trigger: self.trigger.to_owned(),
            repeat: self.repeat.as_ref().map(Repeat::to_owned),
            duration: self.duration,
            description: self.description.as_ref().map(Description::to_owned),
            summary: self.summary.as_ref().map(Summary::to_owned),
            attendees: self.attendees.iter().map(Attendee::to_owned).collect(),
            attach: self.attach.as_ref().map(Attachment::to_owned),
            x_properties: self.x_properties.iter().map(Property::to_owned).collect(),
            unrecognized_properties: self
                .unrecognized_properties
                .iter()
                .map(Property::to_owned)
                .collect(),
        }
    }
}
