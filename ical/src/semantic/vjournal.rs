// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Journal entry component (VJOURNAL) for iCalendar semantic components.

use std::fmt;

use crate::keyword::KW_VJOURNAL;
use crate::parameter::Parameter;
use crate::property::{
    Attendee, Categories, Classification, Description, DtStamp, DtStart, ExDate, LastModified,
    Organizer, Property, PropertyKind, RDate, RRule, Status, StatusValue, Summary, Uid, Url,
    XNameProperty,
};
use crate::semantic::SemanticError;
use crate::semantic::tz_validator::{TzContext, ValidateTzids};
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::TypedComponent;

/// Journal entry component (VJOURNAL)
#[derive(Debug, Clone)]
pub struct VJournal<S: StringStorage> {
    /// Unique identifier for the journal entry
    pub uid: Uid<S>,
    /// Date/time the journal entry was created
    pub dt_stamp: DtStamp<S>,
    /// Date/time of the journal entry
    pub dt_start: DtStart<S>,
    /// Summary/title of the journal entry
    pub summary: Option<Summary<S>>,
    /// Description of the journal entry (can appear multiple times)
    pub descriptions: Vec<Description<S>>,
    /// Organizer of the journal entry
    pub organizer: Option<Organizer<S>>,
    /// Attendees of the journal entry
    pub attendees: Vec<Attendee<S>>,
    /// Last modification date/time
    pub last_modified: Option<LastModified<S>>,
    /// Status of the journal entry
    pub status: Option<JournalStatus<S>>,
    /// Classification
    pub classification: Option<Classification<S>>,
    /// Categories
    pub categories: Vec<Categories<S>>,
    /// Recurrence rule
    pub rrule: Option<RRule<S>>,
    /// Recurrence dates (can be `Period`, `Date`, `or DateTime`)
    pub rdates: Vec<RDate<S>>,
    /// Exception dates (can be `Date` or `DateTime`)
    pub ex_dates: Vec<ExDate<S>>,
    /// URL associated with the journal entry
    pub url: Option<Url<S>>,
    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<XNameProperty<S>>,
    /// Unrecognized / Non-standard properties (preserved for round-trip)
    pub retained_properties: Vec<Property<S>>,
}

/// Parse a `TypedComponent` into a `VJournal`
#[expect(clippy::too_many_lines)]
impl<'src> TryFrom<TypedComponent<'src>> for VJournal<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let span = comp.span();
        if !comp.name.eq_str_ignore_ascii_case(KW_VJOURNAL) {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VJOURNAL,
                got: comp.name,
                span,
            });
        }

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::Uid(uid) => match props.uid {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Uid,
                        span: uid.span(),
                    }),
                    None => props.uid = Some(uid),
                },
                Property::DtStamp(dt) => match props.dt_stamp {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStamp,
                        span: dt.span(),
                    }),
                    None => props.dt_stamp = Some(dt),
                },
                Property::DtStart(dt) => match props.dt_start {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtStart,
                        span: dt.span(),
                    }),
                    None => props.dt_start = Some(dt),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                        span: s.span(),
                    }),
                    None => props.summary = Some(s),
                },
                // VJOURNAL allows multiple DESCRIPTION properties
                Property::Description(desc) => props.descriptions.push(desc),
                Property::Organizer(org) => match props.organizer {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Organizer,
                        span: org.span(),
                    }),
                    None => props.organizer = Some(org),
                },
                Property::Attendee(attendee) => props.attendees.push(attendee),
                Property::LastModified(dt) => match props.last_modified {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::LastModified,
                        span: dt.span(),
                    }),
                    None => props.last_modified = Some(dt),
                },
                Property::Status(status) => match props.status {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Status,
                        span: status.span(),
                    }),
                    None => match status.try_into() {
                        Ok(v) => props.status = Some(v),
                        Err(e) => errors.push(e),
                    },
                },
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                        span: class.span(),
                    }),
                    None => props.classification = Some(class),
                },
                Property::Categories(categories) => props.categories.push(categories),
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::RRule,
                        span: rrule.span(),
                    }),
                    None => props.rrule = Some(rrule),
                },
                Property::RDate(rdate) => props.rdates.push(rdate),
                Property::ExDate(exdate) => props.ex_dates.push(exdate),
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: url.span(),
                        property: PropertyKind::Url,
                    }),
                    None => props.url = Some(url),
                },
                // Preserve unknown properties for round-trip
                Property::XName(prop) => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VJournal for round-trip
                    props.unrecognized_properties.push(prop);
                }
            }
        }

        // Check required fields
        if props.uid.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Uid,
                span,
            });
        }
        if props.dt_stamp.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStamp,
                span,
            });
        }
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStart,
                span,
            });
        }

        if errors.is_empty() {
            Ok(VJournal {
                uid: props.uid.unwrap(),           // SAFETY: checked above
                dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
                dt_start: props.dt_start.unwrap(), // SAFETY: checked above
                summary: props.summary,
                descriptions: props.descriptions,
                organizer: props.organizer,
                attendees: props.attendees,
                last_modified: props.last_modified,
                status: props.status,
                classification: props.classification,
                categories: props.categories,
                rrule: props.rrule,
                rdates: props.rdates,
                ex_dates: props.ex_dates,
                url: props.url,
                x_properties: props.x_properties,
                retained_properties: props.unrecognized_properties,
            })
        } else {
            Err(errors)
        }
    }
}

impl VJournal<Segments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> VJournal<String> {
        VJournal {
            uid: self.uid.to_owned(),
            dt_stamp: self.dt_stamp.to_owned(),
            dt_start: self.dt_start.to_owned(),
            summary: self.summary.as_ref().map(Summary::to_owned),
            descriptions: self
                .descriptions
                .iter()
                .map(Description::to_owned)
                .collect(),
            organizer: self.organizer.as_ref().map(Organizer::to_owned),
            attendees: self.attendees.iter().map(Attendee::to_owned).collect(),
            last_modified: self.last_modified.as_ref().map(LastModified::to_owned),
            status: self.status.as_ref().map(JournalStatus::to_owned),
            classification: self.classification.as_ref().map(Classification::to_owned),
            categories: self.categories.iter().map(Categories::to_owned).collect(),
            rrule: self.rrule.as_ref().map(RRule::to_owned),
            rdates: self.rdates.iter().map(RDate::to_owned).collect(),
            ex_dates: self.ex_dates.iter().map(ExDate::to_owned).collect(),
            url: self.url.as_ref().map(Url::to_owned),
            x_properties: self
                .x_properties
                .iter()
                .map(XNameProperty::to_owned)
                .collect(),
            retained_properties: self
                .retained_properties
                .iter()
                .map(Property::to_owned)
                .collect(),
        }
    }
}

/// Journal status value (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalStatusValue {
    /// Journal entry is draft
    Draft,
    /// Journal entry is final
    Final,
    /// Journal entry is cancelled
    Cancelled,
}

impl TryFrom<StatusValue> for JournalStatusValue {
    type Error = ();

    fn try_from(value: StatusValue) -> Result<Self, Self::Error> {
        match value {
            StatusValue::Draft => Ok(Self::Draft),
            StatusValue::Final => Ok(Self::Final),
            StatusValue::Cancelled => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl fmt::Display for JournalStatusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        StatusValue::from(*self).fmt(f)
    }
}

impl From<JournalStatusValue> for StatusValue {
    fn from(value: JournalStatusValue) -> Self {
        match value {
            JournalStatusValue::Draft => StatusValue::Draft,
            JournalStatusValue::Final => StatusValue::Final,
            JournalStatusValue::Cancelled => StatusValue::Cancelled,
        }
    }
}

/// Journal status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone)]
pub struct JournalStatus<S: StringStorage> {
    /// Status value
    pub value: JournalStatusValue,
    /// Custom X- parameters (preserved for round-trip)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unknown IANA parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<Status<Segments<'src>>> for JournalStatus<Segments<'src>> {
    type Error = SemanticError<'src>;

    fn try_from(property: Status<Segments<'src>>) -> Result<Self, Self::Error> {
        let Ok(value) = property.value.try_into() else {
            return Err(SemanticError::InvalidValue {
                property: PropertyKind::Status,
                value: format!("Invalid journal status value: {}", property.value),
                span: property.span(),
            });
        };

        Ok(JournalStatus {
            value,
            x_parameters: property.x_parameters,
            retained_parameters: property.retained_parameters,
        })
    }
}

impl JournalStatus<Segments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> JournalStatus<String> {
        JournalStatus {
            value: self.value,
            x_parameters: self
                .x_parameters
                .iter()
                .map(RawParameter::to_owned)
                .collect(),
            retained_parameters: self
                .retained_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<S: StringStorage> {
    uid:            Option<Uid<S>>,
    dt_stamp:       Option<DtStamp<S>>,
    dt_start:       Option<DtStart<S>>,
    summary:        Option<Summary<S>>,
    descriptions:   Vec<Description<S>>,
    organizer:      Option<Organizer<S>>,
    attendees:      Vec<Attendee<S>>,
    last_modified:  Option<LastModified<S>>,
    status:         Option<JournalStatus<S>>,
    classification: Option<Classification<S>>,
    categories:     Vec<Categories<S>>,
    rrule:          Option<RRule<S>>,
    rdates:         Vec<RDate<S>>,
    ex_dates:       Vec<ExDate<S>>,
    url:            Option<Url<S>>,
    x_properties:   Vec<XNameProperty<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

impl ValidateTzids for VJournal<Segments<'_>> {
    fn validate_tzids(&mut self, ctx: &TzContext<'_>) -> Result<(), Vec<SemanticError<'static>>> {
        let mut errors = Vec::new();

        // Validate DtStart
        if let Err(e) = ctx.validate_dt(&mut self.dt_start) {
            errors.push(e);
        }

        // Validate RDate properties
        errors.extend(ctx.validate_rdates(&mut self.rdates));

        // Validate ExDate properties
        errors.extend(ctx.validate_exdates(&mut self.ex_dates));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
