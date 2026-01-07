// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use std::fmt;

use crate::keyword::KW_VJOURNAL;
use crate::property::{
    Attendee, Classification, DateTime, ExDateValue, Organizer, Period, Property, PropertyKind,
    RDateValue, Status, StatusValue, Text,
};
use crate::semantic::SemanticError;
use crate::typed::TypedComponent;
use crate::value::RecurrenceRule;
use crate::value::ValueText;

/// Journal entry component (VJOURNAL)
#[derive(Debug, Clone)]
pub struct VJournal<'src> {
    /// Unique identifier for the journal entry
    pub uid: Text<'src>,

    /// Date/time the journal entry was created
    pub dt_stamp: DateTime<'src>,

    /// Date/time of the journal entry
    pub dt_start: DateTime<'src>,

    /// Summary/title of the journal entry
    pub summary: Option<Text<'src>>,

    /// Description of the journal entry (can appear multiple times)
    pub descriptions: Vec<ValueText<'src>>,

    /// Organizer of the journal entry
    pub organizer: Option<Organizer<'src>>,

    /// Attendees of the journal entry
    pub attendees: Vec<Attendee<'src>>,

    /// Last modification date/time
    pub last_modified: Option<DateTime<'src>>,

    /// Status of the journal entry
    pub status: Option<JournalStatus>,

    /// Classification
    pub classification: Option<Classification<'src>>,

    /// Categories
    pub categories: Vec<ValueText<'src>>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period<'src>>,

    /// Exception dates
    pub ex_date: Vec<DateTime<'src>>,

    /// URL associated with the journal entry
    pub url: Option<Text<'src>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<'src>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,
}

/// Parse a `TypedComponent` into a `VJournal`
#[expect(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VJournal<'src> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VJOURNAL {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VJOURNAL,
                got: comp.name,
                span: comp.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            // TODO: Use property span instead of component span for DuplicateProperty
            match prop {
                Property::Uid(uid) => match props.uid {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Uid,
                        span: comp.span,
                    }),
                    None => props.uid = Some(uid.0.clone()),
                },
                Property::DtStamp(dt) => match props.dt_stamp {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStamp,
                        span: comp.span,
                    }),
                    None => props.dt_stamp = Some(dt.0.clone()),
                },
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStart,
                        span: comp.span,
                    }),
                    None => props.dt_start = Some(dt.0.clone()),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                        span: comp.span,
                    }),
                    None => props.summary = Some(s.0.clone()),
                },
                // VJOURNAL allows multiple DESCRIPTION properties
                Property::Description(desc) => props.descriptions.push(desc.content.clone()),
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
                    None => props.last_modified = Some(dt.0.clone()),
                },
                Property::Status(status) => match JournalStatus::try_from(status) {
                    Ok(journal_status) => match props.status {
                        Some(_) => errors.push(SemanticError::DuplicateProperty {
                            property: PropertyKind::Status,
                            span: comp.span,
                        }),
                        None => props.status = Some(journal_status),
                    },
                    Err(e) => errors.push(SemanticError::InvalidValue {
                        property: PropertyKind::Status,
                        value: e,
                        span: comp.span, // TODO: Should use property span
                    }),
                },
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                        span: comp.span,
                    }),
                    None => props.classification = Some(class),
                },
                Property::Categories(categories) => match props.categories {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Categories,
                        span: comp.span,
                    }),
                    None => props.categories = Some(categories.values.clone()),
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
                        // TODO: RDate Date/DateTime not yet implemented for journals
                    }
                }
                Property::ExDate(exdates) => {
                    for exdate in exdates.dates {
                        if let ExDateValue::DateTime(dt) = exdate {
                            props.ex_dates.push(dt);
                        }
                        // TODO: ExDate Date-only not yet implemented for journals
                    }
                }
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: comp.span,
                        property: PropertyKind::Url,
                    }),
                    None => props.url = Some(url.0.clone()),
                },
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                // Ignore other properties not used by VJournal
                _ => {}
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

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(VJournal {
            uid: props.uid.unwrap(),
            dt_stamp: props.dt_stamp.unwrap(),
            dt_start: props.dt_start.unwrap(),
            summary: props.summary,
            descriptions: props.descriptions,
            organizer: props.organizer,
            attendees: props.attendees,
            last_modified: props.last_modified,
            status: props.status,
            classification: props.classification,
            categories: props.categories.unwrap_or_default(),
            rrule: props.rrule,
            rdate: props.rdate,
            ex_date: props.ex_dates,
            url: props.url,
            x_properties: props.x_properties,
            unrecognized_properties: props.unrecognized_properties,
        })
    }
}

/// Journal status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatus {
    /// Journal entry is draft
    Draft,

    /// Journal entry is final
    Final,

    /// Journal entry is cancelled
    Cancelled,
}

impl<'src> TryFrom<Status<'src>> for JournalStatus {
    type Error = String;
    fn try_from(value: Status<'src>) -> Result<Self, Self::Error> {
        match value.value {
            StatusValue::Draft => Ok(Self::Draft),
            StatusValue::Final => Ok(Self::Final),
            StatusValue::Cancelled => Ok(Self::Cancelled),
            _ => Err(format!("Invalid journal status: {value}")),
        }
    }
}

impl fmt::Display for JournalStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Status::from(*self).fmt(f)
    }
}

impl From<JournalStatus> for Status<'_> {
    fn from(value: JournalStatus) -> Self {
        Status {
            value: match value {
                JournalStatus::Draft => StatusValue::Draft,
                JournalStatus::Final => StatusValue::Final,
                JournalStatus::Cancelled => StatusValue::Cancelled,
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
    uid:            Option<Text<'src>>,
    dt_stamp:       Option<DateTime<'src>>,
    dt_start:       Option<DateTime<'src>>,
    summary:        Option<Text<'src>>,
    descriptions:   Vec<ValueText<'src>>,
    organizer:      Option<Organizer<'src>>,
    attendees:      Vec<Attendee<'src>>,
    last_modified:  Option<DateTime<'src>>,
    status:         Option<JournalStatus>,
    classification: Option<Classification<'src>>,
    categories:     Option<Vec<ValueText<'src>>>,
    rrule:          Option<RecurrenceRule>,
    rdate:          Vec<Period<'src>>,
    ex_dates:       Vec<DateTime<'src>>,
    url:            Option<Text<'src>>,
    x_properties:   Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}
