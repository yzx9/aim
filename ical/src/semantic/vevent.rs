// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use std::fmt::{self, Display};

use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::parameter::Parameter;
use crate::property::{
    Attendee, Categories, Classification, Description, DtEnd, DtStamp, DtStart, Duration, ExDate,
    Geo, LastModified, Location, Organizer, Priority, Property, PropertyKind, RDate, RRule,
    Resources, Sequence, Status, StatusValue, Summary, TimeTransparency, Uid, Url, XNameProperty,
};
use crate::semantic::tz_validator::{TzContext, ValidateTzids};
use crate::semantic::{SemanticError, VAlarm};
use crate::string_storage::{Segments, StringStorage};
use crate::syntax::RawParameter;
use crate::typed::TypedComponent;

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent<S: StringStorage> {
    /// Unique identifier for the event
    pub uid: Uid<S>,
    /// Date/time the event was created
    pub dt_stamp: DtStamp<S>,
    /// Date/time the event starts
    pub dt_start: DtStart<S>,
    /// Date/time the event ends
    pub dt_end: Option<DtEnd<S>>,
    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<Duration<S>>,
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
    pub rrule: Option<RRule<S>>,
    /// Recurrence dates
    pub rdates: Vec<RDate<S>>,
    /// Exception dates
    pub ex_dates: Vec<ExDate<S>>,
    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<XNameProperty<S>>,
    /// Unrecognized / Non-standard properties (preserved for round-trip)
    pub retained_properties: Vec<Property<S>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm<S>>,
}

/// Parse a `TypedComponent` into a `VEvent`
impl<'src> TryFrom<TypedComponent<'src>> for VEvent<Segments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let span = comp.span();
        if !comp.name.eq_str_ignore_ascii_case(KW_VEVENT) {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VEVENT,
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
                Property::DtEnd(dt) => match props.dt_end {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::DtEnd,
                        span: dt.span(),
                    }),
                    None => props.dt_end = Some(dt),
                },
                Property::Duration(dur) => match props.duration {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Duration,
                        span: dur.span(),
                    }),
                    None => props.duration = Some(dur),
                },
                Property::Summary(s) => match props.summary {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Summary,
                        span: s.span(),
                    }),
                    None => props.summary = Some(s),
                },
                Property::Description(desc) => match props.description {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Description,
                        span: desc.span(),
                    }),
                    None => props.description = Some(desc),
                },
                Property::Location(loc) => match props.location {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Location,
                        span: loc.span(),
                    }),
                    None => props.location = Some(loc),
                },
                Property::Geo(geo) => match props.geo {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Geo,
                        span: geo.span(),
                    }),
                    None => props.geo = Some(geo),
                },
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Url,
                        span: url.span(),
                    }),
                    None => props.url = Some(url),
                },
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
                    None => match status.clone().try_into() {
                        Ok(v) => props.status = Some(v),
                        Err(e) => errors.push(e),
                    },
                },
                Property::Transp(transp) => match props.transparency {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Transp,
                        span: transp.span(),
                    }),
                    None => props.transparency = Some(transp),
                },
                Property::Sequence(seq) => match props.sequence {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Sequence,
                        span: seq.span(),
                    }),
                    None => props.sequence = Some(seq),
                },
                Property::Priority(pri) => match props.priority {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Priority,
                        span: pri.span(),
                    }),
                    None => props.priority = Some(pri),
                },
                Property::Class(class) => match props.classification {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Class,
                        span: class.span(),
                    }),
                    None => props.classification = Some(class),
                },
                Property::Resources(resources) => match props.resources {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: resources.span(),
                        property: PropertyKind::Resources,
                    }),
                    None => props.resources = Some(resources),
                },
                Property::Categories(categories) => match props.categories {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        span: categories.span(),
                        property: PropertyKind::Categories,
                    }),
                    None => props.categories = Some(categories),
                },
                Property::RRule(rrule) => match props.rrule {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::RRule,
                        span: rrule.span(),
                    }),
                    None => props.rrule = Some(rrule),
                },
                Property::RDate(rdate) => props.rdates.push(rdate),
                Property::ExDate(exdate) => props.ex_dates.push(exdate),
                // Preserve unknown properties for round-trip
                Property::XName(prop) => props.x_properties.push(prop),
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

        // Parse sub-components (alarms)
        let alarms = comp
            .children
            .into_iter()
            .filter_map(|child| {
                if child.name.eq_str_ignore_ascii_case(KW_VALARM) {
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
                uid: props.uid.unwrap(),           // SAFETY: checked above
                dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
                dt_start: props.dt_start.unwrap(), // SAFETY: checked above
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
                rdates: props.rdates,
                ex_dates: props.ex_dates,
                x_properties: props.x_properties,
                retained_properties: props.unrecognized_properties,
                alarms,
            })
        } else {
            Err(errors)
        }
    }
}

impl VEvent<Segments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> VEvent<String> {
        VEvent {
            uid: self.uid.to_owned(),
            dt_stamp: self.dt_stamp.to_owned(),
            dt_start: self.dt_start.to_owned(),
            dt_end: self.dt_end.as_ref().map(DtEnd::to_owned),
            duration: self.duration.as_ref().map(Duration::to_owned),
            summary: self.summary.as_ref().map(Summary::to_owned),
            description: self.description.as_ref().map(Description::to_owned),
            location: self.location.as_ref().map(Location::to_owned),
            geo: self.geo.as_ref().map(Geo::to_owned),
            url: self.url.as_ref().map(Url::to_owned),
            organizer: self.organizer.as_ref().map(Organizer::to_owned),
            attendees: self.attendees.iter().map(Attendee::to_owned).collect(),
            last_modified: self.last_modified.as_ref().map(LastModified::to_owned),
            status: self.status.as_ref().map(EventStatus::to_owned),
            transparency: self.transparency.as_ref().map(TimeTransparency::to_owned),
            sequence: self.sequence.as_ref().map(Sequence::to_owned),
            priority: self.priority.as_ref().map(Priority::to_owned),
            classification: self.classification.as_ref().map(Classification::to_owned),
            resources: self.resources.as_ref().map(Resources::to_owned),
            categories: self.categories.as_ref().map(Categories::to_owned),
            rrule: self.rrule.as_ref().map(RRule::to_owned),
            rdates: self.rdates.iter().map(RDate::to_owned).collect(),
            ex_dates: self.ex_dates.iter().map(ExDate::to_owned).collect(),
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
            alarms: self.alarms.iter().map(VAlarm::to_owned).collect(),
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

/// Event status (RFC 5545 Section 3.8.1.11)
#[derive(Debug, Clone)]
pub struct EventStatus<S: StringStorage> {
    /// Status value
    pub value: EventStatusValue,
    /// Custom X- parameters (preserved for round-trip)
    pub x_parameters: Vec<RawParameter<S>>,
    /// Unknown IANA parameters (preserved for round-trip)
    pub retained_parameters: Vec<Parameter<S>>,
}

impl<'src> TryFrom<Status<Segments<'src>>> for EventStatus<Segments<'src>> {
    type Error = SemanticError<'src>;

    fn try_from(property: Status<Segments<'src>>) -> Result<Self, Self::Error> {
        let Ok(value) = property.value.try_into() else {
            return Err(SemanticError::InvalidValue {
                property: PropertyKind::Status,
                value: format!("Invalid event status value: {}", property.value),
                span: property.span(),
            });
        };

        Ok(EventStatus {
            value,
            x_parameters: property.x_parameters,
            retained_parameters: property.retained_parameters,
        })
    }
}

impl EventStatus<Segments<'_>> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> EventStatus<String> {
        EventStatus {
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

impl EventStatus<String> {
    /// Create a new `EventStatus<String>` from a status value.
    #[must_use]
    pub fn new(value: EventStatusValue) -> Self {
        Self {
            value,
            x_parameters: Vec::new(),
            retained_parameters: Vec::new(),
        }
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector< S: StringStorage> {
    uid:            Option<Uid<S>>,
    dt_stamp:       Option<DtStamp<S>>,
    dt_start:       Option<DtStart<S>>,
    dt_end:         Option<DtEnd<S>>,
    duration:       Option<Duration<S>>,
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
    rrule:          Option<RRule<S>>,
    rdates:         Vec<RDate<S>>,
    ex_dates:       Vec<ExDate<S>>,
    x_properties:   Vec<XNameProperty<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

impl ValidateTzids for VEvent<Segments<'_>> {
    fn validate_tzids(&mut self, ctx: &TzContext<'_>) -> Result<(), Vec<SemanticError<'static>>> {
        let mut errors = Vec::new();

        // Validate DtStart
        if let Err(e) = ctx.validate_dt(&mut self.dt_start) {
            errors.push(e);
        }

        // Validate DtEnd if present
        if let Some(ref mut dt_end) = self.dt_end
            && let Err(e) = ctx.validate_dt(dt_end)
        {
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
