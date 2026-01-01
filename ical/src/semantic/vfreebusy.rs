// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use crate::keyword::KW_VFREEBUSY;
use crate::parameter::FreeBusyType;
use crate::property::{DateTime, Organizer, Period, Property, PropertyKind, Text};
use crate::semantic::SemanticError;
use crate::typed::TypedComponent;
use crate::value::ValueDuration;

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy<'src> {
    /// Unique identifier for the free/busy info
    pub uid: Text<'src>,

    /// Date/time the free/busy info was created
    pub dt_stamp: DateTime<'src>,

    /// Start of the free/busy period
    pub dt_start: DateTime<'src>,

    /// End of the free/busy period
    pub dt_end: Option<DateTime<'src>>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer<'src>,

    /// Contact information
    pub contact: Option<Text<'src>>,

    /// URL for additional free/busy info
    pub url: Option<Text<'src>>,

    /// Busy periods
    pub busy: Vec<Period<'src>>,

    /// Free periods
    pub free: Vec<Period<'src>>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period<'src>>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period<'src>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
impl<'src> TryFrom<TypedComponent<'src>> for VFreeBusy<'src> {
    type Error = Vec<SemanticError>;

    #[expect(clippy::too_many_lines)]
    fn try_from(comp: TypedComponent<'src>) -> Result<Self, Self::Error> {
        if comp.name != KW_VFREEBUSY {
            return Err(vec![SemanticError::ExpectedComponent {
                expected: KW_VFREEBUSY,
                got: comp.name.to_string(),
            }]);
        }

        let mut errors = Vec::new();

        // Collect all properties in a single pass
        let mut props = PropertyCollector::default();
        for prop in comp.properties {
            match prop {
                Property::FreeBusy(freebusy) => {
                    for value in freebusy.values {
                        // Categorize by FBTYPE
                        match freebusy.fb_type {
                            FreeBusyType::Free => props.free.push(value),
                            FreeBusyType::Busy => props.busy.push(value),
                            FreeBusyType::BusyTentative => props.busy_tentative.push(value),
                            FreeBusyType::BusyUnavailable => props.busy_unavailable.push(value),
                        }
                    }
                }
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
                    None => props.duration = Some(dur.value),
                },
                Property::Organizer(org) => match props.organizer {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Organizer,
                    }),
                    None => props.organizer = Some(org),
                },
                Property::Contact(text) => match props.contact {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Contact,
                    }),
                    None => props.contact = Some(text),
                },
                Property::Url(text) => match props.url {
                    Some(_) => errors.push(SemanticError::DuplicateProperty {
                        property: PropertyKind::Url,
                    }),
                    None => props.url = Some(text),
                },
                // Ignore other properties not used by VFreeBusy
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
        if props.organizer.is_none() {
            errors.push(SemanticError::MissingProperty {
                property: PropertyKind::Organizer,
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
        })
    }
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'src> {
    uid:              Option<Text<'src>>,
    dt_stamp:         Option<DateTime<'src>>,
    dt_start:         Option<DateTime<'src>>,
    dt_end:           Option<DateTime<'src>>,
    duration:         Option<ValueDuration>,
    organizer:        Option<Organizer<'src>>,
    contact:          Option<Text<'src>>,
    url:              Option<Text<'src>>,
    busy:             Vec<Period<'src>>,
    free:             Vec<Period<'src>>,
    busy_tentative:   Vec<Period<'src>>,
    busy_unavailable: Vec<Period<'src>>,
}
