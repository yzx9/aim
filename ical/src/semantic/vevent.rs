// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::fmt;

use crate::Uid;
use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::property::{
    Attendee, Categories, Classification, DateTime, Description, DtEnd, DtStamp, DtStart, Geo,
    LastModified, Location, Organizer, Period, Property, PropertyKind, Resources, Status,
    StatusValue, Summary, TimeTransparency, Url,
};
use crate::semantic::{SemanticError, VAlarm};
use crate::typed::TypedComponent;
use crate::value::{RecurrenceRule, ValueDuration};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent<'src> {
    /// Unique identifier for the event
    pub uid: Uid<'src>,

    /// Date/time the event was created
    pub dt_stamp: DtStamp<'src>,

    /// Date/time the event starts
    pub dt_start: DtStart<'src>,

    /// Date/time the event ends
    pub dt_end: Option<DtEnd<'src>>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<ValueDuration>,

    /// Summary/title of the event
    pub summary: Option<Summary<'src>>,

    /// Description of the event
    pub description: Option<Description<'src>>,

    /// Location of the event
    pub location: Option<Location<'src>>,

    /// Geographic position
    pub geo: Option<Geo<'src>>,

    /// URL associated with the event
    pub url: Option<Url<'src>>,

    /// Organizer of the event
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the event
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<LastModified<'src>>,

    /// Status of the event
    pub status: Option<Status<'src>>,

    /// Time transparency
    pub transparency: Option<TimeTransparency<'src>>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

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

    /// Unrecognized properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,

    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<'src>>,
}

/// Parse a `TypedComponent` into a `VEvent`
impl<'src> TryFrom<TypedComponent<'src>> for VEvent<'src> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VEVENT {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VEVENT,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::Uid(uid) => match props.uid {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Uid,
                    }),
                    None => props.uid = Some(uid),
                },
                Property::DtStamp(dt) => match props.dt_stamp {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStamp,
                    }),
                    None => props.dt_stamp = Some(dt),
                },
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStart,
                    }),
                    None => props.dt_start = Some(dt),
                },
                Property::DtEnd(dt) => match props.dt_end {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtEnd,
                    }),
                    None => props.dt_end = Some(dt),
                },
                Property::Duration(dur) => match props.duration {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                    }),
                    None => props.duration = Some(dur.value),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                    }),
                    None => props.summary = Some(s),
                },
                Property::Description(desc) => match props.description {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                    }),
                    None => props.description = Some(desc),
                },
                Property::Location(loc) => match props.location {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Location,
                    }),
                    None => props.location = Some(loc),
                },
                Property::Geo(geo) => match props.geo {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Geo,
                    }),
                    None => props.geo = Some(geo),
                },
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Url,
                    }),
                    None => props.url = Some(url),
                },
                Property::Organizer(org) => match props.organizer {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Organizer,
                    }),
                    None => props.organizer = Some(org),
                },
                Property::Attendee(attendee) => {
                    props.attendees.push(attendee);
                }
                Property::LastModified(dt) => match props.last_modified {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                    }),
                    None => props.last_modified = Some(dt),
                },
                Property::Status(status) => match props.status {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Status,
                    }),
                    None => props.status = Some(status),
                },
                Property::Transp(transp) => match props.transparency {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Transp,
                    }),
                    None => props.transparency = Some(transp),
                },
                Property::Sequence(seq) => match props.sequence {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Sequence,
                    }),
                    None => match u32::try_from(seq.value) {
                        Ok(v) => props.sequence = Some(v),
                        Err(_) => {
                            errors.push(SemanticError::InvalidValue {
                                property: PropertyKind::Sequence,
                                value: "Sequence must be non-negative".to_string(),
                            });
                        }
                    },
                },
                Property::Priority(pri) => match props.priority {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Priority,
                    }),
                    None => props.priority = Some(pri.value),
                },
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                    }),
                    None => props.classification = Some(class),
                },
                Property::Resources(resources) => match props.resources {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Resources,
                    }),
                    None => props.resources = Some(resources),
                },
                Property::Categories(categories) => match props.categories {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Categories,
                    }),
                    None => props.categories = Some(categories),
                },
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::RRule,
                    }),
                    None => props.rrule = Some(rrule),
                },
                Property::RDate(rdates) => {
                    for rdate in rdates.dates {
                        match rdate {
                            crate::property::RDateValue::Period(p) => props.rdate.push(p),
                            _ => {
                                // RDate Date/DateTime not yet implemented for events
                            }
                        }
                    }
                }
                Property::ExDate(exdates) => {
                    for exdate in exdates.dates {
                        if let crate::property::ExDateValue::DateTime(dt) = exdate {
                            props.ex_dates.push(dt);
                        }
                        // ExDate Date-only not yet implemented for events
                    }
                }
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => {
                    props.x_properties.push(prop);
                }
                prop @ Property::Unrecognized { .. } => {
                    props.unrecognized_properties.push(prop);
                }
                // Ignore other properties not used by VEvent
                _ => {}
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Uid,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStamp,
            });
        }
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
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

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

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
    }
}

/// Event status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// Event is cancelled
    Cancelled,
}

impl<'src> TryFrom<Status<'src>> for EventStatus {
    type Error = String;

    fn try_from(value: Status<'src>) -> Result<Self, Self::Error> {
        match value.value {
            StatusValue::Tentative => Ok(Self::Tentative),
            StatusValue::Confirmed => Ok(Self::Confirmed),
            StatusValue::Cancelled => Ok(Self::Cancelled),
            _ => Err(format!("Invalid event status: {value}")),
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Status::from(*self).fmt(f)
    }
}

impl From<EventStatus> for Status<'_> {
    fn from(value: EventStatus) -> Self {
        Status {
            value: match value {
                EventStatus::Tentative => StatusValue::Tentative,
                EventStatus::Confirmed => StatusValue::Confirmed,
                EventStatus::Cancelled => StatusValue::Cancelled,
            },
            x_parameters: Vec::new(),
            unrecognized_parameters: Vec::new(),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:            Option<Uid<'src>>,
    dt_stamp:       Option<DtStamp<'src>>,
    dt_start:       Option<DtStart<'src>>,
    dt_end:         Option<DtEnd<'src>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Summary<'src>>,
    description:    Option<Description<'src>>,
    location:       Option<Location<'src>>,
    geo:            Option<Geo<'src>>,
    url:            Option<Url<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<LastModified<'src>>,
    status:         Option<Status<'src>>,
    transparency:   Option<TimeTransparency<'src>>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    classification: Option<Classification<'src>>,
    resources:      Option<Resources<'src>>,
    categories:     Option<Categories<'src>>,
    rrule:          Option<RecurrenceRule>,
    rdate:          Vec<Period<'src>>,
    ex_dates:       Vec<DateTime<'src>>,
    x_properties:   Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}
