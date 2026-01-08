// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::fmt::{self, Display};

use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::parameter::Parameter;
use crate::property::{
    Attendee, Categories, Classification, DateTime, Description, DtEnd, DtStamp, DtStart,
    ExDateValueRef, Geo, LastModified, Location, Organizer, Period, Priority, Property,
    PropertyKind, RDateValueRef, Resources, Sequence, Status, StatusValue, Summary,
    TimeTransparency, Uid, Url,
};
use crate::semantic::{SemanticError, VAlarm};
use crate::syntax::SpannedSegments;
use crate::typed::TypedComponent;
use crate::value::{RecurrenceRule, ValueDuration};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent<S: Clone + Display> {
    /// Unique identifier for the event
    pub uid: Uid<S>,

    /// Date/time the event was created
    pub dt_stamp: DtStamp<S>,

    /// Date/time the event starts
    pub dt_start: DtStart<S>,

    /// Date/time the event ends
    pub dt_end: Option<DtEnd<S>>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<ValueDuration>,

    /// Summary/title of the event
    pub summary: Option<Summary<S>>,

    /// Description of the event
    pub description: Option<Description<S>>,

    /// Location of the event
    pub location: Option<Location<S>>,

    /// Geographic position
    pub geo: Option<Geo<S>>,

    /// URL associated with the event
    pub url: Option<Url<S>>,

    /// Organizer of the event
    pub organizer: Option<Organizer<S>>,

    /// Attendees of the event
    pub attendees: Vec<Attendee<S>>,

    /// Last modification date/time
    pub last_modified: Option<LastModified<S>>,

    /// Status of the event
    pub status: Option<EventStatus<S>>,

    /// Time transparency
    pub transparency: Option<TimeTransparency<S>>,

    /// Sequence number for revisions
    pub sequence: Option<Sequence<S>>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<Priority<S>>,

    /// Classification
    pub classification: Option<Classification<S>>,

    /// Resources
    pub resources: Option<Resources<S>>,

    /// Categories
    pub categories: Option<Categories<S>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<S>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<S>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<S>>,

    /// Unrecognized properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<S>>,

    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<S>>,
}

/// Type alias for `VEvent` with borrowed data
pub type VEventRef<'src> = VEvent<SpannedSegments<'src>>;

/// Type alias for `VEvent` with owned data
pub type VEventOwned = VEvent<String>;

/// Parse a `TypedComponent` into a `VEvent`
impl<'src> TryFrom<TypedComponent<'src>> for VEvent<SpannedSegments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        if comp.name != KW_VEVENT {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VEVENT,
                got: comp.name,
                span: comp.span,
            });
        }

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
                Property::DtEnd(dt) => match props.dt_end {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtEnd,
                        span: comp.span,
                    }),
                    None => props.dt_end = Some(dt),
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
                Property::Transp(transp) => match props.transparency {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Transp,
                        span: comp.span,
                    }),
                    None => props.transparency = Some(transp),
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
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                        span: comp.span,
                    }),
                    None => props.classification = Some(class),
                },
                Property::Resources(resources) => match props.resources {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::Resources,
                    }),
                    None => props.resources = Some(resources),
                },
                Property::Categories(categories) => match props.categories {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::Categories,
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
                        match rdate {
                            RDateValueRef::Period(p) => props.rdate.push(p),
                            _ => {
                                // TODO: RDate Date/DateTime not yet implemented for events
                            }
                        }
                    }
                }
                Property::ExDate(exdates) => {
                    for exdate in exdates.dates {
                        if let ExDateValueRef::DateTime(dt) = exdate {
                            props.ex_dates.push(dt);
                        }
                        // ExDate Date-only not yet implemented for events
                    }
                }
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VEvent for round-trip
                    props.unrecognized_properties.push(prop);
                }
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
                property: PropertyKind::Uid,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
                property: PropertyKind::DtStamp,
            });
        }
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
                span: comp.span,
                property: PropertyKind::DtStart,
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

        if errors.is_empty() {
            Ok(VEvent {
                uid: props.uid.unwrap(),
                dt_stamp: props.dt_stamp.unwrap(),
                dt_start: props.dt_start.unwrap(),
                dt_end: props.dt_end,
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
                transparency: props.transparency,
                sequence: props.sequence,
                priority: props.priority,
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
        } else {
            Err(errors)
        }
    }
}

/// Event status value (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatusValue {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// Event is cancelled
    Cancelled,
}

impl TryFrom<StatusValue> for EventStatusValue {
    type Error = ();

    fn try_from(value: StatusValue) -> Result<Self, Self::Error> {
        match value {
            StatusValue::Tentative => Ok(Self::Tentative),
            StatusValue::Confirmed => Ok(Self::Confirmed),
            StatusValue::Cancelled => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl Display for EventStatusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        StatusValue::from(*self).fmt(f)
    }
}

impl From<EventStatusValue> for StatusValue {
    fn from(value: EventStatusValue) -> Self {
        match value {
            EventStatusValue::Tentative => StatusValue::Tentative,
            EventStatusValue::Confirmed => StatusValue::Confirmed,
            EventStatusValue::Cancelled => StatusValue::Cancelled,
        }
    }
}

/// Type alias for `EventStatus` with borrowed data
pub type EventStatusRef<'src> = EventStatus<SpannedSegments<'src>>;

/// Type alias for `EventStatus` with owned data
pub type EventStatusOwned = EventStatus<String>;

/// Event status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone)]
pub struct EventStatus<S: Clone + Display> {
    /// Status value
    pub value: EventStatusValue,
    /// Custom X- parameters (preserved for round-trip)
    pub x_parameters: Vec<Parameter<S>>,
    /// Unknown IANA parameters (preserved for round-trip)
    pub unrecognized_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<Status<SpannedSegments<'src>>> for EventStatus<SpannedSegments<'src>> {
    type Error = SemanticError<'src>;

    fn try_from(property: Status<SpannedSegments<'src>>) -> Result<Self, Self::Error> {
        let Ok(value) = property.value.try_into() else {
            return Err(SemanticError::InvalidValue {
                property: PropertyKind::Status,
                value: format!("Invalid event status value: {}", property.value),
                span: property.span,
            });
        };

        Ok(EventStatus {
            value,
            x_parameters: property.x_parameters,
            unrecognized_parameters: property.unrecognized_parameters,
        })
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector< S: Clone + Display> {
    uid:            Option<Uid<S>>,
    dt_stamp:       Option<DtStamp<S>>,
    dt_start:       Option<DtStart<S>>,
    dt_end:         Option<DtEnd<S>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Summary<S>>,
    description:    Option<Description<S>>,
    location:       Option<Location<S>>,
    geo:            Option<Geo<S>>,
    url:            Option<Url<S>>,
    organizer:      Option<Organizer<S>>,
    attendees:      Vec<Attendee<S>>,
    last_modified:  Option<LastModified<S>>,
    status:         Option<EventStatus<S>>,
    transparency:   Option<TimeTransparency<S>>,
    sequence:       Option<Sequence<S>>,
    priority:       Option<Priority<S>>,
    classification: Option<Classification<S>>,
    resources:      Option<Resources<S>>,
    categories:     Option<Categories<S>>,
    rrule:          Option<RecurrenceRule>,
    rdate:          Vec<Period<S>>,
    ex_dates:       Vec<DateTime<S>>,
    x_properties:   Vec<Property<S>>,
    unrecognized_properties: Vec<Property<S>>,
}
