// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use crate::Uid;
use crate::keyword::KW_VFREEBUSY;
use crate::parameter::FreeBusyType;
use crate::property::{
    Contact, DtEnd, DtStamp, DtStart, Organizer, Period, Property, PropertyKind, Url,
};
use crate::semantic::SemanticError;
use crate::typed::TypedComponent;
use crate::value::ValueDuration;

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy<'src> {
    /// Unique identifier for the free/busy info
    pub uid: Uid<'src>,

    /// Date/time the free/busy info was created
    pub dt_stamp: DtStamp<'src>,

    /// Start of the free/busy period
    pub dt_start: DtStart<'src>,

    /// End of the free/busy period
    pub dt_end: Option<DtEnd<'src>>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer<'src>,

    /// Contact information
    pub contact: Option<Contact<'src>>,

    /// URL for additional free/busy info
    pub url: Option<Url<'src>>,

    /// Busy periods
    pub busy: Vec<Period<'src>>,

    /// Free periods
    pub free: Vec<Period<'src>>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period<'src>>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period<'src>>,

    /// Custom X- properties (preserved for round-trip)
    pub x_properties: Vec<Property<'src>>,

    /// Unknown IANA properties (preserved for round-trip)
    pub unrecognized_properties: Vec<Property<'src>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
impl<'src> TryFrom<TypedComponent<'src>> for VFreeBusy<'src> {
    type Error = Vec<SemanticError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VFREEBUSY {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VFREEBUSY,
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

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

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
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:                Option<Uid<'src>>,
    dt_stamp:           Option<DtStamp<'src>>,
    dt_start:           Option<DtStart<'src>>,
    dt_end:             Option<DtEnd<'src>>,
    duration:           Option<ValueDuration>,
    organizer:          Option<Organizer<'src>>,
    contact:            Option<Contact<'src>>,
    url:                Option<Url<'src>>,
    busy:               Vec<Period<'src>>,
    free:               Vec<Period<'src>>,
    busy_tentative:     Vec<Period<'src>>,
    busy_unavailable:   Vec<Period<'src>>,
    x_properties:       Vec<Property<'src>>,
    unrecognized_properties: Vec<Property<'src>>,
}
