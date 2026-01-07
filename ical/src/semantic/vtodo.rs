// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! To-do component (VTODO) for iCalendar semantic components.

use std::fmt;

use crate::Uid;
use crate::keyword::{KW_VALARM, KW_VTODO};
use crate::parameter::Parameter;
use crate::property::{
    Attendee, Categories, Classification, Completed, DateTime, Description, DtStamp, DtStart, Due,
    ExDateValue, Geo, LastModified, Location, Organizer, PercentComplete, Period, Priority,
    Property, PropertyKind, RDateValue, Resources, Sequence, Status, StatusValue, Summary, Url,
};
use crate::semantic::{SemanticError, VAlarm};
use crate::typed::TypedComponent;
use crate::value::{RecurrenceRule, ValueDuration};

/// To-do component (VTODO)
#[derive(Debug, Clone)]
pub struct VTodo<'src> {
    /// Unique identifier for the todo
    pub uid: Uid<'src>,

    /// Date/time the todo was created
    pub dt_stamp: DtStamp<'src>,

    /// Date/time to start the todo
    pub dt_start: Option<DtStart<'src>>,

    /// Date/time the todo is due
    pub due: Option<Due<'src>>,

    /// Completion date/time
    pub completed: Option<Completed<'src>>,

    /// Duration of the todo
    pub duration: Option<ValueDuration>,

    /// Summary/title of the todo
    pub summary: Option<Summary<'src>>,

    /// Description of the todo
    pub description: Option<Description<'src>>,

    /// Location of the todo
    pub location: Option<Location<'src>>,

    /// Geographic position
    pub geo: Option<Geo<'src>>,

    /// URL associated with the todo
    pub url: Option<Url<'src>>,

    /// Organizer of the todo
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the todo
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<LastModified<'src>>,

    /// Status of the todo
    pub status: Option<TodoStatus<'src>>,

    /// Sequence number for revisions
    pub sequence: Option<Sequence<'src>>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<Priority<'src>>,

    /// Percentage complete (0-100)
    pub percent_complete: Option<PercentComplete<'src>>,

    /// Classification
    pub classification: Option<Classification<'src>>,

    /// Resources
    pub resources: Option<Resources<'src>>,

    /// Categories
    pub categories: Option<Categories<'src>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<'src>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<'src>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<'src>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,

    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<'src>>,
}

/// Parse a `TypedComponent` into a `VTodo`
impl<'src> TryFrom<TypedComponent<'src>> for VTodo<'src> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VTODO {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VTODO,
                got: comp.name,
                span: comp.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                // TODO: Use property span instead of component span for DuplicateProperty
                Property::Uid(uid) => match props.uid {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Uid,
                        span: comp.span,
                    }),
                    None => props.uid = Some(uid),
                },
                Property::DtStamp(dt) => match props.dt_stamp {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStamp,
                        span: comp.span,
                    }),
                    None => props.dt_stamp = Some(dt),
                },
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStart,
                        span: comp.span,
                    }),
                    None => props.dt_start = Some(dt),
                },
                Property::Due(dt) => match props.due {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Due,
                        span: comp.span,
                    }),
                    None => props.due = Some(dt),
                },
                Property::Completed(dt) => match props.completed {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Completed,
                        span: comp.span,
                    }),
                    None => props.completed = Some(dt),
                },
                Property::Duration(dur) => match props.duration {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                        span: comp.span,
                    }),
                    None => props.duration = Some(dur.value),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                        span: comp.span,
                    }),
                    None => props.summary = Some(s),
                },
                Property::Description(desc) => match props.description {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                        span: comp.span,
                    }),
                    None => props.description = Some(desc),
                },
                Property::Location(loc) => match props.location {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Location,
                        span: comp.span,
                    }),
                    None => props.location = Some(loc),
                },
                Property::Geo(geo) => match props.geo {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Geo,
                        span: comp.span,
                    }),
                    None => props.geo = Some(geo),
                },
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Url,
                        span: comp.span,
                    }),
                    None => props.url = Some(url),
                },
                Property::Organizer(org) => match props.organizer {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Organizer,
                        span: comp.span,
                    }),
                    None => props.organizer = Some(org),
                },
                Property::Attendee(attendee) => props.attendees.push(attendee),
                Property::LastModified(dt) => match props.last_modified {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                        span: comp.span,
                    }),
                    None => props.last_modified = Some(dt),
                },
                Property::Status(status) => match props.status {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Status,
                        span: comp.span,
                    }),
                    None => match status.try_into() {
                        Ok(v) => props.status = Some(v),
                        Err(e) => errors.push(e),
                    },
                },
                Property::Sequence(seq) => match props.sequence {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Sequence,
                        span: comp.span,
                    }),
                    None => props.sequence = Some(seq),
                },
                Property::Priority(pri) => match props.priority {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Priority,
                        span: comp.span,
                    }),
                    None => props.priority = Some(pri),
                },
                Property::PercentComplete(pct) => match props.percent_complete {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::PercentComplete,
                        span: comp.span,
                    }),
                    None => props.percent_complete = Some(pct),
                },
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                        span: comp.span,
                    }),
                    None => props.classification = Some(class),
                },
                Property::Resources(resources) => match props.resources {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Resources,
                        span: comp.span,
                    }),
                    None => props.resources = Some(resources),
                },
                Property::Categories(categories) => match props.categories {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Categories,
                        span: comp.span,
                    }),
                    None => props.categories = Some(categories),
                },
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::RRule,
                        span: comp.span,
                    }),
                    None => props.rrule = Some(rrule),
                },
                Property::RDate(rdates) => {
                    for rdate in rdates.dates {
                        if let RDateValue::Period(p) = rdate {
                            props.rdate.push(p);
                        }
                        // RDate Date/DateTime not yet implemented for todos
                    }
                }
                Property::ExDate(exdates) => {
                    for exdate in exdates.dates {
                        if let ExDateValue::DateTime(dt) = exdate {
                            props.ex_dates.push(dt);
                        }
                        // ExDate Date-only not yet implemented for todos
                    }
                }
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VTodo for round-trip
                    props.unrecognized_properties.push(prop);
                }
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Uid,
                span: comp.span,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStamp,
                span: comp.span,
            });
        }

        // Parse sub-components (alarms)
        let alarms = comp
            .children
            .into_iter()
            .filter_map(|child| {
                if child.name == KW_VALARM {
                    Some(VAlarm::try_from(child))
                } else {
                    None
                }
            })
            .filter_map(|result| match result {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.extend(e);
                    None
                }
            })
            .collect();

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VTodo {
            uid: props.uid.unwrap(),           // SAFETY: checked above
            dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
            dt_start: props.dt_start,
            due: props.due,
            completed: props.completed,
            duration: props.duration,
            summary: props.summary,
            description: props.description,
            location: props.location,
            geo: props.geo,
            url: props.url,
            organizer: props.organizer,
            attendees: props.attendees,
            last_modified: props.last_modified,
            status: props.status,
            sequence: props.sequence,
            priority: props.priority,
            percent_complete: props.percent_complete,
            classification: props.classification,
            resources: props.resources,
            categories: props.categories,
            rrule: props.rrule,
            rdate: props.rdate,
            ex_date: props.ex_dates,
            x_properties: props.x_properties,
            unrecognized_properties: props.unrecognized_properties,
            alarms,
        })
    }
}

/// To-do status value (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatusValue {
    /// To-do needs action
    NeedsAction,

    /// To-do is completed
    Completed,

    /// To-do is in process
    InProcess,

    /// To-do is cancelled
    Cancelled,
}

impl TryFrom<StatusValue> for TodoStatusValue {
    type Error = ();
    fn try_from(value: StatusValue) -> Result<Self, Self::Error> {
        match value {
            StatusValue::NeedsAction => Ok(Self::NeedsAction),
            StatusValue::Completed => Ok(Self::Completed),
            StatusValue::InProcess => Ok(Self::InProcess),
            StatusValue::Cancelled => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl fmt::Display for TodoStatusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        StatusValue::from(*self).fmt(f)
    }
}

impl From<TodoStatusValue> for StatusValue {
    fn from(value: TodoStatusValue) -> Self {
        match value {
            TodoStatusValue::NeedsAction => StatusValue::NeedsAction,
            TodoStatusValue::Completed => StatusValue::Completed,
            TodoStatusValue::InProcess => StatusValue::InProcess,
            TodoStatusValue::Cancelled => StatusValue::Cancelled,
        }
    }
}

/// To-do status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone)]
pub struct TodoStatus<'src> {
    /// Status value
    pub value: TodoStatusValue,
    /// Custom X- parameters (preserved for round-trip)
    pub x_parameters: Vec<Parameter<'src>>,
    /// Unknown IANA parameters (preserved for round-trip)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<Status<'src>> for TodoStatus<'src> {
    type Error = SemanticError<'src>;

    fn try_from(property: Status<'src>) -> Result<Self, Self::Error> {
        let Ok(value) = property.value.try_into() else {
            return Err(SemanticError::InvalidValue {
                property: PropertyKind::Status,
                value: format!("Invalid todo status value: {}", property.value),
                span: property.span,
            });
        };

        Ok(TodoStatus {
            value,
            x_parameters: property.x_parameters,
            unrecognized_parameters: property.unrecognized_parameters,
        })
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:            Option<Uid<'src>>,
    dt_stamp:       Option<DtStamp<'src>>,
    dt_start:       Option<DtStart<'src>>,
    due:            Option<Due<'src>>,
    completed:      Option<Completed<'src>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Summary<'src>>,
    description:    Option<Description<'src>>,
    location:       Option<Location<'src>>,
    geo:            Option<Geo<'src>>,
    url:            Option<Url<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<LastModified<'src>>,
    status:         Option<TodoStatus<'src>>,
    sequence:       Option<Sequence<'src>>,
    priority:       Option<Priority<'src>>,
    percent_complete: Option<PercentComplete<'src>>,
    classification: Option<Classification<'src>>,
    resources:      Option<Resources<'src>>,
    categories:     Option<Categories<'src>>,
    rrule:          Option<RecurrenceRule>,
    rdate:          Vec<Period<'src>>,
    ex_dates:       Vec<DateTime<'src>>,
    x_properties:   Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}
