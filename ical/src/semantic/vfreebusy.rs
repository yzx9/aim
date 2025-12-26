// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use crate::keyword::KW_VFREEBUSY;
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    find_parameter, get_language, get_single_value, get_tzid, parse_organizer_property,
    value_to_any_date_time, value_to_date_time, value_to_date_time_with_tz, value_to_string,
};
use crate::semantic::properties::{DateTime, Organizer, Period, Text, Uri};
use crate::typed::parameter_type::FreeBusyType;
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty, Value,
    ValueDate, ValueDuration,
};

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy {
    /// Unique identifier for the free/busy info
    pub uid: String,

    /// Date/time the free/busy info was created
    pub dt_stamp: DateTime,

    /// Start of the free/busy period
    pub dt_start: DateTime,

    /// End of the free/busy period
    pub dt_end: Option<DateTime>,

    /// Duration of the free/busy period
    pub duration: Option<ValueDuration>,

    /// Organizer of the free/busy info
    pub organizer: Organizer,

    /// Contact information
    pub contact: Option<Text>,

    /// URL for additional free/busy info
    pub url: Option<Uri>,

    /// Busy periods
    pub busy: Vec<Period>,

    /// Free periods
    pub free: Vec<Period>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period>,
}

/// Helper struct to collect properties during single-pass iteration
#[rustfmt::skip]
#[derive(Debug, Default)]
struct PropertyCollector<'a> {
    uid:        Option<&'a TypedProperty<'a>>,
    dt_stamp:   Option<&'a TypedProperty<'a>>,
    dt_start:   Option<&'a TypedProperty<'a>>,
    dt_end:     Option<&'a TypedProperty<'a>>,
    duration:   Option<&'a TypedProperty<'a>>,
    organizer:  Option<&'a TypedProperty<'a>>,
    contact:    Option<&'a TypedProperty<'a>>,
    url:        Option<&'a TypedProperty<'a>>,
    freebusy:   Vec<&'a TypedProperty<'a>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
#[allow(clippy::too_many_lines)]
pub fn parse_vfreebusy(comp: &TypedComponent) -> Result<VFreeBusy, Vec<SemanticError>> {
    if comp.name != KW_VFREEBUSY {
        return Err(vec![SemanticError::InvalidStructure(format!(
            "Expected VFREEBUSY component, got '{}'",
            comp.name
        ))]);
    }

    let mut errors = Vec::new();

    // Collect all properties in a single pass
    let mut props = PropertyCollector::default();
    for prop in &comp.properties {
        match prop.name {
            name if name == PropertyKind::Uid.as_str() => {
                if props.uid.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Uid.as_str()
                    )));
                } else {
                    props.uid = Some(prop);
                }
            }
            name if name == PropertyKind::DtStamp.as_str() => {
                if props.dt_stamp.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStamp.as_str()
                    )));
                } else {
                    props.dt_stamp = Some(prop);
                }
            }
            name if name == PropertyKind::DtStart.as_str() => {
                if props.dt_start.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtStart.as_str()
                    )));
                } else {
                    props.dt_start = Some(prop);
                }
            }
            name if name == PropertyKind::DtEnd.as_str() => {
                if props.dt_end.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::DtEnd.as_str()
                    )));
                } else {
                    props.dt_end = Some(prop);
                }
            }
            name if name == PropertyKind::Duration.as_str() => {
                if props.duration.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Duration.as_str()
                    )));
                } else {
                    props.duration = Some(prop);
                }
            }
            name if name == PropertyKind::Organizer.as_str() => {
                if props.organizer.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Organizer.as_str()
                    )));
                } else {
                    props.organizer = Some(prop);
                }
            }
            name if name == PropertyKind::Contact.as_str() => {
                if props.contact.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Contact.as_str()
                    )));
                } else {
                    props.contact = Some(prop);
                }
            }
            name if name == PropertyKind::Url.as_str() => {
                if props.url.is_some() {
                    errors.push(SemanticError::InvalidStructure(format!(
                        "Duplicate {} property",
                        PropertyKind::Url.as_str()
                    )));
                } else {
                    props.url = Some(prop);
                }
            }
            name if name == PropertyKind::FreeBusy.as_str() => {
                props.freebusy.push(prop);
            }
            // Ignore unknown properties
            _ => {}
        }
    }

    // UID is required
    let uid = match props.uid {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_string(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::Uid.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                    String::new()
                }
            },
            Err(e) => {
                errors.push(e);
                String::new()
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::Uid.as_str().to_string(),
            ));
            String::new()
        }
    };

    // DTSTAMP is required
    let dt_stamp = match props.dt_stamp {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => match value_to_date_time(value) {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtStamp.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            },
            Err(e) => {
                errors.push(e);
                DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStamp.as_str().to_string(),
            ));
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    };

    // DTSTART is required
    let dt_start = match props.dt_start {
        Some(prop) => match get_single_value(prop) {
            Ok(value) => {
                let tz_id = get_tzid(&prop.parameters);
                let result = match &tz_id {
                    Some(id) => value_to_date_time_with_tz(value, id.clone()),
                    None => value_to_any_date_time(value),
                };
                match result {
                    Some(v) => v,
                    None => {
                        errors.push(SemanticError::InvalidValue(
                            PropertyKind::DtStart.as_str().to_string(),
                            "Expected date-time value".to_string(),
                        ));
                        DateTime::Date {
                            date: ValueDate {
                                year: 0,
                                month: 1,
                                day: 1,
                            },
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(e);
                DateTime::Date {
                    date: ValueDate {
                        year: 0,
                        month: 1,
                        day: 1,
                    },
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::DtStart.as_str().to_string(),
            ));
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    };

    // DTEND is optional
    let dt_end = props.dt_end.map(|prop| match get_single_value(prop) {
        Ok(value) => {
            let tz_id = get_tzid(&prop.parameters);
            let result = match &tz_id {
                Some(id) => value_to_date_time_with_tz(value, id.clone()),
                None => value_to_any_date_time(value),
            };
            match result {
                Some(v) => v,
                None => {
                    errors.push(SemanticError::InvalidValue(
                        PropertyKind::DtEnd.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    ));
                    DateTime::Date {
                        date: ValueDate {
                            year: 0,
                            month: 1,
                            day: 1,
                        },
                    }
                }
            }
        }
        Err(e) => {
            errors.push(e);
            DateTime::Date {
                date: ValueDate {
                    year: 0,
                    month: 1,
                    day: 1,
                },
            }
        }
    });

    // DURATION is optional
    let duration = props.duration.map(|prop| match get_single_value(prop) {
        Ok(Value::Duration(v)) => *v,
        Ok(_) => {
            errors.push(SemanticError::InvalidValue(
                PropertyKind::Duration.as_str().to_string(),
                "Expected duration value".to_string(),
            ));
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
        Err(e) => {
            errors.push(e);
            ValueDuration::DateTime {
                positive: true,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
    });

    // ORGANIZER is required
    let organizer = match props.organizer {
        Some(prop) => match parse_organizer_property(prop) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                // Return a dummy organizer to continue parsing
                Organizer {
                    cal_address: Uri { uri: String::new() },
                    cn: None,
                    dir: None,
                    sent_by: None,
                    language: None,
                }
            }
        },
        None => {
            errors.push(SemanticError::MissingProperty(
                PropertyKind::Organizer.as_str().to_string(),
            ));
            Organizer {
                cal_address: Uri { uri: String::new() },
                cn: None,
                dir: None,
                sent_by: None,
                language: None,
            }
        }
    };

    // CONTACT is optional
    let contact = props.contact.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => {
                let language = get_language(&prop.parameters);
                Text {
                    content: v,
                    language,
                }
            }
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Contact.as_str().to_string(),
                    "Expected text value".to_string(),
                ));
                Text {
                    content: String::new(),
                    language: None,
                }
            }
        },
        Err(e) => {
            errors.push(e);
            Text {
                content: String::new(),
                language: None,
            }
        }
    });

    // URL is optional
    let url = props.url.map(|prop| match get_single_value(prop) {
        Ok(value) => match value_to_string(value) {
            Some(v) => Uri { uri: v },
            None => {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::Url.as_str().to_string(),
                    "Expected URI value".to_string(),
                ));
                Uri { uri: String::new() }
            }
        },
        Err(e) => {
            errors.push(e);
            Uri { uri: String::new() }
        }
    });

    // FREEBUSY can appear multiple times
    let (mut busy, mut free, mut busy_tentative, mut busy_unavailable) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for prop in props.freebusy {
        // Get the FBTYPE parameter (default is BUSY)
        let fb_type = find_parameter(&prop.parameters, TypedParameterKind::FreeBusyType).and_then(
            |p| match p {
                TypedParameter::FreeBusyType { value, .. } => Some(*value),
                _ => None,
            },
        );

        // Parse all period values
        for value in &prop.values {
            if let Some(period) = value_to_period(value) {
                match fb_type.unwrap_or(FreeBusyType::Busy) {
                    FreeBusyType::Free => free.push(period),
                    FreeBusyType::Busy => busy.push(period),
                    FreeBusyType::BusyTentative => busy_tentative.push(period),
                    FreeBusyType::BusyUnavailable => busy_unavailable.push(period),
                }
            } else {
                errors.push(SemanticError::InvalidValue(
                    PropertyKind::FreeBusy.as_str().to_string(),
                    "Expected period value".to_string(),
                ));
            }
        }
    }

    // If we have errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(VFreeBusy {
        uid,
        dt_stamp,
        dt_start,
        dt_end,
        duration,
        organizer,
        contact,
        url,
        busy,
        free,
        busy_tentative,
        busy_unavailable,
    })
}

/// Convert a Value to a Period
/// NOTE: This is a placeholder since Period values are not yet implemented in the typed phase
fn value_to_period(_value: &Value<'_>) -> Option<Period> {
    // TODO: Parse Period values when implemented in typed phase
    None
}
