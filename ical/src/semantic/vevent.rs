// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::{convert::TryFrom, fmt};

use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::property::{Attendee, Classification, DateTime, Geo, Organizer, Period, Text};
use crate::property::{Property, Status, TimeTransparency};
use crate::semantic::{SemanticError, VAlarm};
use crate::typed::{PropertyKind, TypedComponent};
use crate::value::{RecurrenceRule, ValueDuration};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent<'src> {
    /// Unique identifier for the event
    pub uid: Text<'src>,

    /// Date/time the event was created
    pub dt_stamp: DateTime<'src>,

    /// Date/time the event starts
    pub dt_start: DateTime<'src>,

    /// Date/time the event ends
    pub dt_end: Option<DateTime<'src>>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<ValueDuration>,

    /// Summary/title of the event
    pub summary: Option<Text<'src>>,

    /// Description of the event
    pub description: Option<Text<'src>>,

    /// Location of the event
    pub location: Option<Text<'src>>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the event
    pub url: Option<Text<'src>>,

    /// Organizer of the event
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the event
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

    /// Status of the event
    pub status: Option<Status>,

    /// Time transparency
    pub transparency: Option<TimeTransparency>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

    /// Classification
    pub classification: Option<Classification>,

    /// Resources
    pub resources: Vec<Text<'src>>,

    /// Categories
    pub categories: Vec<Text<'src>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<'src>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<'src>>,

    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<'src>>,
}

/// Parse a `TypedComponent` into a `VEvent`
#[allow(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VEvent<'src> {
    type Error = Vec<SemanticError>;

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
            match Property::try_from(prop) {
                Ok(property) => {
                    match property {
                        Property::Uid(text) => match props.uid {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Uid,
                            }),
                            None => props.uid = Some(text),
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
                            None => props.duration = Some(dur),
                        },
                        Property::Summary(text) => match props.summary {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Summary,
                            }),
                            None => props.summary = Some(text),
                        },
                        Property::Description(text) => match props.description {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Description,
                            }),
                            None => props.description = Some(text),
                        },
                        Property::Location(text) => match props.location {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Location,
                            }),
                            None => props.location = Some(text),
                        },
                        Property::Geo(geo) => match props.geo {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Geo,
                            }),
                            None => props.geo = Some(geo),
                        },
                        Property::Url(text) => match props.url {
                            Some(_) => errors.push(SemanticError::DuplicateProperty {
                                property: PropertyKind::Url,
                            }),
                            None => props.url = Some(text),
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
                            None => match u32::try_from(seq) {
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
                            None => props.priority = Some(pri),
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
                            for rdate in rdates {
                                match rdate {
                                    crate::property::RDateValue::Period(p) => props.rdate.push(p),
                                    _ => {
                                        // RDate Date/DateTime not yet implemented for events
                                    }
                                }
                            }
                        }
                        Property::ExDate(exdates) => {
                            for exdate in exdates {
                                if let crate::property::ExDateValue::DateTime(dt) = exdate {
                                    props.ex_dates.push(dt);
                                }
                                // ExDate Date-only not yet implemented for events
                            }
                        }
                        // Ignore other properties not used by VEvent
                        _ => {}
                    }
                }
                Err(e) => errors.extend(e),
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
            resources: props.resources.unwrap_or_default(),
            categories: props.categories.unwrap_or_default(),
            rrule: props.rrule,
            rdate: props.rdate,
            ex_date: props.ex_dates,
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

impl TryFrom<Status> for EventStatus {
    type Error = String;

    fn try_from(value: Status) -> Result<Self, Self::Error> {
        match value {
            Status::Tentative => Ok(Self::Tentative),
            Status::Confirmed => Ok(Self::Confirmed),
            Status::Cancelled => Ok(Self::Cancelled),
            _ => Err(format!("Invalid event status: {value}")),
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Status::from(*self).fmt(f)
    }
}

impl From<EventStatus> for Status {
    fn from(value: EventStatus) -> Self {
        match value {
            EventStatus::Tentative => Status::Tentative,
            EventStatus::Confirmed => Status::Confirmed,
            EventStatus::Cancelled => Status::Cancelled,
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:            Option<Text<'src>>,
    dt_stamp:       Option<DateTime<'src>>,
    dt_start:       Option<DateTime<'src>>,
    dt_end:         Option<DateTime<'src>>,
    duration:       Option<ValueDuration>,
    summary:        Option<Text<'src>>,
    description:    Option<Text<'src>>,
    location:       Option<Text<'src>>,
    geo:            Option<Geo>,
    url:            Option<Text<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<DateTime<'src>>,
    status:         Option<Status>,
    transparency:   Option<TimeTransparency>,
    sequence:       Option<u32>,
    priority:       Option<u8>,
    classification: Option<Classification>,
    resources:      Option<Vec<Text<'src>>>,
    categories:     Option<Vec<Text<'src>>>,
    rrule:          Option<RecurrenceRule>,
    rdate:          Vec<Period<'src>>,
    ex_dates:       Vec<DateTime<'src>>,
}
