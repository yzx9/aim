// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use std::fmt::Display;

use crate::keyword::KW_VFREEBUSY;
use crate::parameter::FreeBusyType;
use crate::property::Uid;
use crate::property::{
    Contact, DtEnd, DtStamp, DtStart, Organizer, Period, Property, PropertyKind, Url,
};
use crate::semantic::SemanticError;
use crate::syntax::SpannedSegments;
use crate::typed::TypedComponent;
use crate::value::ValueDuration;

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy<S: Clone + Display> {
    /// Unique identifier for the free/busy info
    pub uid: Uid<S>,

    /// Date/time the free/busy info was created
    pub dt_stamp: DtStamp<S>,

    /// Start of the free/busy period
    pub dt_start: DtStart<S>,

    /// End of the free/busy period
    pub dt_end: Option<DtEnd<S>>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer<S>,

    /// Contact information
    pub contact: Option<Contact<S>>,

    /// URL for additional free/busy info
    pub url: Option<Url<S>>,

    /// Busy periods
    pub busy: Vec<Period<S>>,

    /// Free periods
    pub free: Vec<Period<S>>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period<S>>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period<S>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<S>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<S>>,
}

/// Type alias for `VFreeBusy` with borrowed data
pub type VFreeBusyRef<'src> = VFreeBusy<SpannedSegments<'src>>;

/// Type alias for `VFreeBusy` with owned data
pub type VFreeBusyOwned<'src> = VFreeBusy<String>;

/// Parse a `TypedComponent` into a `VFreeBusy`
impl<'src> TryFrom<TypedComponent<'src>> for VFreeBusy<SpannedSegments<'src>> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        if comp.name != KW_VFREEBUSY {
            errors.push(SemanticError::ExpectedComponent {
                expected: KW_VFREEBUSY,
                got: comp.name,
                span: comp.span,
            });
        }

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            // TODO: Use property span instead of component span for DuplicateProperty
            match prop {
                Property::FreeBusy(freebusy) => {
                    for value in freebusy.values {
                        // Categorize by FBTYPE
                        // Per RFC 5545: applications MUST treat x-name and iana-token
                        // values they don't recognize the same way as BUSY
                        match &freebusy.fb_type {
                            FreeBusyType::Free => props.free.push(value),
                            FreeBusyType::Busy => props.busy.push(value),
                            FreeBusyType::BusyTentative => props.busy_tentative.push(value),
                            FreeBusyType::BusyUnavailable => props.busy_unavailable.push(value),
                            // XName and Unrecognized values treated as BUSY
                            FreeBusyType::XName(_) | FreeBusyType::Unrecognized(_) => {
                                props.busy.push(value);
                            }
                        }
                    }
                }
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
                Property::Organizer(org) => match props.organizer {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Organizer,
                        span: comp.span,
                    }),
                    None => props.organizer = Some(org),
                },
                Property::Contact(contact) => match props.contact {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Contact,
                        span: comp.span,
                    }),
                    None => props.contact = Some(contact),
                },
                Property::Url(url) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Url,
                        span: comp.span,
                    }),
                    None => props.url = Some(url),
                },
                // Preserve unknown properties for round-trip
                prop @ Property::XName { .. } => props.x_properties.push(prop),
                prop @ Property::Unrecognized { .. } => props.unrecognized_properties.push(prop),
                prop => {
                    // Preserve other properties not used by VFreeBusy for round-trip
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
        if props.dt_start.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::DtStart,
                span: comp.span,
            });
        }
        if props.organizer.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Organizer,
                span: comp.span,
            });
        }

        if errors.is_empty() {
            Ok(VFreeBusy {
                uid: props.uid.unwrap(),           // SAFETY: checked above
                dt_stamp: props.dt_stamp.unwrap(), // SAFETY: checked above
                dt_start: props.dt_start.unwrap(), // SAFETY: checked above
                dt_end: props.dt_end,
                duration: props.duration,
                organizer: props.organizer.unwrap(), // SAFETY: checked above
                contact: props.contact,
                url: props.url,
                busy: props.busy,
                free: props.free,
                busy_tentative: props.busy_tentative,
                busy_unavailable: props.busy_unavailable,
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
    uid:                Option<Uid<S>>,
    dt_stamp:           Option<DtStamp<S>>,
    dt_start:           Option<DtStart<S>>,
    dt_end:             Option<DtEnd<S>>,
    duration:           Option<ValueDuration>,
    organizer:          Option<Organizer<S>>,
    contact:            Option<Contact<S>>,
    url:                Option<Url<S>>,
    busy:               Vec<Period<S>>,
    free:               Vec<Period<S>>,
    busy_tentative:     Vec<Period<S>>,
    busy_unavailable:   Vec<Period<S>>,
    x_properties:       Vec<Property<S>>,
    unrecognized_properties: Vec<Property<S>>,
}

impl<'src> VFreeBusyRef<'src> {
    /// Convert borrowed data to owned data
    pub fn to_owned(&self) -> VFreeBusyOwned<'src> {
        VFreeBusyOwned {
            uid: self.uid.to_owned(),
            dt_stamp: self.dt_stamp.to_owned(),
            dt_start: self.dt_start.to_owned(),
            dt_end: self.dt_end.as_ref().map(DtEnd::to_owned),
            duration: self.duration,
            organizer: self.organizer.to_owned(),
            contact: self.contact.as_ref().map(Contact::to_owned),
            url: self.url.as_ref().map(Url::to_owned),
            busy: self.busy.iter().map(Period::to_owned).collect(),
            free: self.free.iter().map(Period::to_owned).collect(),
            busy_tentative: self.busy_tentative.iter().map(Period::to_owned).collect(),
            busy_unavailable: self.busy_unavailable.iter().map(Period::to_owned).collect(),
            x_properties: self.x_properties.iter().map(Property::to_owned).collect(),
            unrecognized_properties: self
                .unrecognized_properties
                .iter()
                .map(Property::to_owned)
                .collect(),
        }
    }
}
