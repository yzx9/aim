// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Event component (VEVENT) for iCalendar semantic components.

use crate::RecurrenceRule;
use crate::keyword::{KW_VALARM, KW_VEVENT};
use crate::semantic::SemanticError;
use crate::semantic::analysis::{
    find_parameter, find_properties, find_property_by_kind, get_language, get_single_value,
    get_tzid, parse_cal_address, value_to_date_time, value_to_duration, value_to_int,
    value_to_string,
};
use crate::semantic::enums::{Classification, Period};
use crate::semantic::properties::{Attendee, DateTime, Duration, Geo, Organizer, Text, Uri};
use crate::semantic::valarm::{VAlarm, parse_valarm};
use crate::typed::parameter_types::{CalendarUserType, ParticipationRole, ParticipationStatus};
use crate::typed::{
    PropertyKind, TypedComponent, TypedParameter, TypedParameterKind, TypedProperty, Value,
};

/// Event component (VEVENT)
#[derive(Debug, Clone)]
pub struct VEvent {
    /// Unique identifier for the event
    pub uid: String,

    /// Date/time the event was created
    pub dt_stamp: DateTime,

    /// Date/time the event starts
    pub dt_start: DateTime,

    /// Date/time the event ends
    pub dt_end: Option<DateTime>,

    /// Duration of the event (alternative to `dt_end`)
    pub duration: Option<Duration>,

    /// Summary/title of the event
    pub summary: Option<Text>,

    /// Description of the event
    pub description: Option<Text>,

    /// Location of the event
    pub location: Option<Text>,

    /// Geographic position
    pub geo: Option<Geo>,

    /// URL associated with the event
    pub url: Option<Uri>,

    /// Organizer of the event
    pub organizer: Option<Organizer>,

    /// Attendees of the event
    pub attendees: Vec<Attendee>,

    /// Last modification date/time
    pub last_modified: Option<DateTime>,

    /// Status of the event
    pub status: Option<EventStatus>,

    /// Time transparency
    pub transparency: Option<TimeTransparency>,

    /// Sequence number for revisions
    pub sequence: Option<u32>,

    /// Priority (1-9, 1 is highest)
    pub priority: Option<u8>,

    /// Classification
    pub classification: Option<Classification>,

    /// Resources
    pub resources: Vec<Text>,

    /// Categories
    pub categories: Vec<Text>,

    /// Recurrence rule
    pub rrule: Option<RecurrenceRule>,

    /// Recurrence dates
    pub rdate: Vec<Period>,

    /// Exception dates
    pub ex_date: Vec<DateTime>,

    /// Timezone identifier
    pub tz_id: Option<String>,

    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
    /// Sub-components (like alarms)
    pub alarms: Vec<VAlarm>,
}

/// Event status
#[derive(Debug, Clone, Copy)]
pub enum EventStatus {
    /// Event is tentative
    Tentative,

    /// Event is confirmed
    Confirmed,

    /// Event is cancelled
    Cancelled,
    // /// Custom status
    // Custom(String),
}

/// Time transparency for events
#[derive(Debug, Clone, Copy)]
pub enum TimeTransparency {
    /// Event blocks time
    Opaque,

    /// Event does not block time
    Transparent,
    // /// Custom transparency
    // Custom(String),
}

/// Parse a `TypedComponent` into a `VEvent`
#[allow(clippy::too_many_lines)]
pub fn parse_vevent(comp: TypedComponent) -> Result<VEvent, SemanticError> {
    if comp.name != KW_VEVENT {
        return Err(SemanticError::InvalidStructure(format!(
            "Expected VEVENT component, got '{}'",
            comp.name
        )));
    }

    // UID is required
    let uid_prop = find_property_by_kind(&comp.properties, PropertyKind::Uid)
        .ok_or_else(|| SemanticError::MissingProperty(PropertyKind::Uid.as_str().to_string()))?;
    let uid = value_to_string(get_single_value(uid_prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::Uid.as_str().to_string(),
            "Expected text value".to_string(),
        )
    })?;

    // DTSTAMP is required
    let dt_stamp_prop =
        find_property_by_kind(&comp.properties, PropertyKind::DtStamp).ok_or_else(|| {
            SemanticError::MissingProperty(PropertyKind::DtStamp.as_str().to_string())
        })?;
    let dt_stamp = value_to_date_time(get_single_value(dt_stamp_prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::DtStamp.as_str().to_string(),
            "Expected date-time value".to_string(),
        )
    })?;

    // DTSTART is required
    let dt_start_prop =
        find_property_by_kind(&comp.properties, PropertyKind::DtStart).ok_or_else(|| {
            SemanticError::MissingProperty(PropertyKind::DtStart.as_str().to_string())
        })?;
    let mut dt_start = value_to_date_time(get_single_value(dt_start_prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::DtStart.as_str().to_string(),
            "Expected date-time value".to_string(),
        )
    })?;
    // Add timezone if specified
    if let Some(tz_id) = get_tzid(&dt_start_prop.parameters) {
        dt_start.tz_id = Some(tz_id);
    }

    // DTEND is optional
    let dt_end =
        if let Some(dt_end_prop) = find_property_by_kind(&comp.properties, PropertyKind::DtEnd) {
            let mut dt_end_value =
                value_to_date_time(get_single_value(dt_end_prop)?).ok_or_else(|| {
                    SemanticError::InvalidValue(
                        PropertyKind::DtEnd.as_str().to_string(),
                        "Expected date-time value".to_string(),
                    )
                })?;
            if let Some(tz_id) = get_tzid(&dt_end_prop.parameters) {
                dt_end_value.tz_id = Some(tz_id);
            }
            Some(dt_end_value)
        } else {
            None
        };

    // DURATION is optional (alternative to DTEND)
    let duration = if let Some(duration_prop) =
        find_property_by_kind(&comp.properties, PropertyKind::Duration)
    {
        Some(
            value_to_duration(get_single_value(duration_prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Duration.as_str().to_string(),
                    "Expected duration value".to_string(),
                )
            })?,
        )
    } else {
        None
    };

    // SUMMARY is optional
    let summary = if let Some(summary_prop) =
        find_property_by_kind(&comp.properties, PropertyKind::Summary)
    {
        let text = value_to_string(get_single_value(summary_prop)?).ok_or_else(|| {
            SemanticError::InvalidValue(
                PropertyKind::Summary.as_str().to_string(),
                "Expected text value".to_string(),
            )
        })?;
        let language = get_language(&summary_prop.parameters);
        Some(Text {
            content: text,
            language,
        })
    } else {
        None
    };

    // DESCRIPTION is optional
    let description = if let Some(desc_prop) =
        find_property_by_kind(&comp.properties, PropertyKind::Description)
    {
        let text = value_to_string(get_single_value(desc_prop)?).ok_or_else(|| {
            SemanticError::InvalidValue(
                PropertyKind::Description.as_str().to_string(),
                "Expected text value".to_string(),
            )
        })?;
        let language = get_language(&desc_prop.parameters);
        Some(Text {
            content: text,
            language,
        })
    } else {
        None
    };

    // LOCATION is optional
    let location =
        if let Some(loc_prop) = find_property_by_kind(&comp.properties, PropertyKind::Location) {
            let text = value_to_string(get_single_value(loc_prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Location.as_str().to_string(),
                    "Expected text value".to_string(),
                )
            })?;
            let language = get_language(&loc_prop.parameters);
            Some(Text {
                content: text,
                language,
            })
        } else {
            None
        };

    // GEO is optional
    let geo = if let Some(geo_prop) = find_property_by_kind(&comp.properties, PropertyKind::Geo) {
        let values = &geo_prop.values;
        if values.len() == 2 {
            let Some(lat_val) = values.first() else {
                return Err(SemanticError::InvalidValue(
                    PropertyKind::Geo.as_str().to_string(),
                    "Expected float value for latitude".to_string(),
                ));
            };
            let Some(lon_val) = values.get(1) else {
                return Err(SemanticError::InvalidValue(
                    PropertyKind::Geo.as_str().to_string(),
                    "Expected float value for longitude".to_string(),
                ));
            };
            let lat = match lat_val {
                Value::Float(f) => *f,
                _ => {
                    return Err(SemanticError::InvalidValue(
                        PropertyKind::Geo.as_str().to_string(),
                        "Expected float value for latitude".to_string(),
                    ));
                }
            };
            let lon = match lon_val {
                Value::Float(f) => *f,
                _ => {
                    return Err(SemanticError::InvalidValue(
                        PropertyKind::Geo.as_str().to_string(),
                        "Expected float value for longitude".to_string(),
                    ));
                }
            };
            Some(Geo { lat, lon })
        } else {
            return Err(SemanticError::InvalidValue(
                PropertyKind::Geo.as_str().to_string(),
                "Expected exactly 2 float values".to_string(),
            ));
        }
    } else {
        None
    };

    // URL is optional
    let url = if let Some(url_prop) = find_property_by_kind(&comp.properties, PropertyKind::Url) {
        Some(Uri {
            uri: value_to_string(get_single_value(url_prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Url.as_str().to_string(),
                    "Expected URI value".to_string(),
                )
            })?,
        })
    } else {
        None
    };

    // ORGANIZER is optional
    let organizer = find_property_by_kind(&comp.properties, PropertyKind::Organizer)
        .map(parse_organizer)
        .transpose()?;

    // ATTENDEE can appear multiple times
    let attendees = find_properties(&comp.properties, PropertyKind::Attendee)
        .into_iter()
        .map(parse_attendee)
        .collect::<Result<Vec<_>, _>>()?;

    // LAST-MODIFIED is optional
    let last_modified =
        if let Some(prop) = find_property_by_kind(&comp.properties, PropertyKind::LastModified) {
            Some(value_to_date_time(get_single_value(prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::LastModified.as_str().to_string(),
                    "Expected date-time value".to_string(),
                )
            })?)
        } else {
            None
        };

    // STATUS is optional
    let status = find_property_by_kind(&comp.properties, PropertyKind::Status)
        .map(|p| {
            let text = value_to_string(get_single_value(p)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Status.as_str().to_string(),
                    "Expected text value".to_string(),
                )
            })?;
            match text.to_uppercase().as_str() {
                "TENTATIVE" => Ok(EventStatus::Tentative),
                "CONFIRMED" => Ok(EventStatus::Confirmed),
                "CANCELLED" => Ok(EventStatus::Cancelled),
                _ => Err(SemanticError::InvalidValue(
                    PropertyKind::Status.as_str().to_string(),
                    format!("Invalid status: {text}"),
                )),
            }
        })
        .transpose()?;

    // TRANSP is optional
    let transparency = find_property_by_kind(&comp.properties, PropertyKind::Transp)
        .map(|p| {
            let text = value_to_string(get_single_value(p)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Transp.as_str().to_string(),
                    "Expected text value".to_string(),
                )
            })?;
            match text.to_uppercase().as_str() {
                "OPAQUE" => Ok(TimeTransparency::Opaque),
                "TRANSPARENT" => Ok(TimeTransparency::Transparent),
                _ => Err(SemanticError::InvalidValue(
                    PropertyKind::Transp.as_str().to_string(),
                    format!("Invalid transparency: {text}"),
                )),
            }
        })
        .transpose()?;

    // SEQUENCE is optional
    let sequence =
        if let Some(prop) = find_property_by_kind(&comp.properties, PropertyKind::Sequence) {
            Some(value_to_int::<u32>(get_single_value(prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Sequence.as_str().to_string(),
                    "Expected integer value".to_string(),
                )
            })?)
        } else {
            None
        };

    // PRIORITY is optional
    let priority =
        if let Some(prop) = find_property_by_kind(&comp.properties, PropertyKind::Priority) {
            Some(value_to_int::<u8>(get_single_value(prop)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Priority.as_str().to_string(),
                    "Expected integer value".to_string(),
                )
            })?)
        } else {
            None
        };

    // CLASS is optional
    let classification = find_property_by_kind(&comp.properties, PropertyKind::Class)
        .map(|p| {
            let text = value_to_string(get_single_value(p)?).ok_or_else(|| {
                SemanticError::InvalidValue(
                    PropertyKind::Class.as_str().to_string(),
                    "Expected text value".to_string(),
                )
            })?;
            match text.to_uppercase().as_str() {
                "PUBLIC" => Ok(Classification::Public),
                "PRIVATE" => Ok(Classification::Private),
                "CONFIDENTIAL" => Ok(Classification::Confidential),
                _ => Err(SemanticError::InvalidValue(
                    PropertyKind::Class.as_str().to_string(),
                    format!("Invalid classification: {text}"),
                )),
            }
        })
        .transpose()?;

    // RESOURCES can appear multiple times (comma-separated values)
    let resources = find_property_by_kind(&comp.properties, PropertyKind::Resources)
        .map(|p| {
            p.values
                .iter()
                .filter_map(|v| {
                    value_to_string(v).map(|s| Text {
                        content: s,
                        language: get_language(&p.parameters),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // CATEGORIES can appear multiple times (comma-separated values)
    let categories = find_property_by_kind(&comp.properties, PropertyKind::Categories)
        .map(|p| {
            p.values
                .iter()
                .filter_map(|v| {
                    value_to_string(v).map(|s| Text {
                        content: s,
                        language: get_language(&p.parameters),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // RRULE is optional
    let rrule = match find_property_by_kind(&comp.properties, PropertyKind::RRule) {
        Some(prop) => {
            match get_single_value(prop)? {
                Value::Text(_text) => {
                    // TODO: Parse RRULE from text format
                    // For now, skip RRULE parsing
                    None
                }
                _ => {
                    return Err(SemanticError::InvalidValue(
                        PropertyKind::RRule.as_str().to_string(),
                        "Expected text value".to_string(),
                    ));
                }
            }
        }
        None => None,
    };

    // RDATE is optional (periods)
    let rdate = vec![]; // TODO: implement RDATE parsing

    // EXDATE is optional
    let ex_date = find_properties(&comp.properties, PropertyKind::ExDate)
        .into_iter()
        .flat_map(|p| {
            p.values
                .iter()
                .filter_map(|v| value_to_date_time(v))
                .collect::<Vec<_>>()
        })
        .collect();

    // Parse sub-components (alarms)
    let alarms = comp
        .children
        .into_iter()
        .filter_map(|child| {
            if child.name == KW_VALARM {
                Some(parse_valarm(child))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(VEvent {
        uid,
        dt_stamp,
        dt_start,
        dt_end,
        duration,
        summary,
        description,
        location,
        geo,
        url,
        organizer,
        attendees,
        last_modified,
        status,
        transparency,
        sequence,
        priority,
        classification,
        resources,
        categories,
        rrule,
        rdate,
        ex_date,
        tz_id: get_tzid(&dt_start_prop.parameters),
        alarms,
    })
}

/// Parse an ORGANIZER property into an Organizer
fn parse_organizer(prop: &TypedProperty<'_>) -> Result<Organizer, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::Organizer.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Organizer {
        cal_address,
        cn,
        dir,
        sent_by,
        language,
    })
}

/// Parse an ATTENDEE property into an Attendee
fn parse_attendee(prop: &TypedProperty<'_>) -> Result<Attendee, SemanticError> {
    let cal_address = parse_cal_address(get_single_value(prop)?).ok_or_else(|| {
        SemanticError::InvalidValue(
            PropertyKind::Attendee.as_str().to_string(),
            "Expected calendar user address".to_string(),
        )
    })?;

    // Extract CN parameter
    let cn =
        find_parameter(&prop.parameters, TypedParameterKind::CommonName).and_then(|p| match p {
            TypedParameter::CommonName { value, .. } => Some(value.resolve().to_string()),
            _ => None,
        });

    // Extract ROLE parameter (default: REQ-PARTICIPANT)
    let role = find_parameter(&prop.parameters, TypedParameterKind::ParticipationRole)
        .and_then(|p| match p {
            TypedParameter::ParticipationRole { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationRole::ReqParticipant);

    // Extract PARTSTAT parameter (default: NEEDS-ACTION)
    let part_stat = find_parameter(&prop.parameters, TypedParameterKind::ParticipationStatus)
        .and_then(|p| match p {
            TypedParameter::ParticipationStatus { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(ParticipationStatus::NeedsAction);

    // Extract RSVP parameter
    let rsvp = find_parameter(&prop.parameters, TypedParameterKind::RsvpExpectation).and_then(
        |p| match p {
            TypedParameter::RsvpExpectation { value, .. } => Some(*value),
            _ => None,
        },
    );

    // Extract CUTYPE parameter (default: INDIVIDUAL)
    let cutype = find_parameter(&prop.parameters, TypedParameterKind::CalendarUserType)
        .and_then(|p| match p {
            TypedParameter::CalendarUserType { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap_or(CalendarUserType::Individual);

    // Extract MEMBER parameter
    let member = find_parameter(&prop.parameters, TypedParameterKind::GroupOrListMembership)
        .and_then(|p| match p {
            TypedParameter::GroupOrListMembership { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-TO parameter
    let delegated_to =
        find_parameter(&prop.parameters, TypedParameterKind::Delegatees).and_then(|p| match p {
            TypedParameter::Delegatees { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DELEGATED-FROM parameter
    let delegated_from =
        find_parameter(&prop.parameters, TypedParameterKind::Delegators).and_then(|p| match p {
            TypedParameter::Delegators { values, .. } => values.first().map(|v| Uri {
                uri: v.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract DIR parameter
    let dir =
        find_parameter(&prop.parameters, TypedParameterKind::Directory).and_then(|p| match p {
            TypedParameter::Directory { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract SENT-BY parameter
    let sent_by =
        find_parameter(&prop.parameters, TypedParameterKind::SendBy).and_then(|p| match p {
            TypedParameter::SendBy { value, .. } => Some(Uri {
                uri: value.resolve().to_string(),
            }),
            _ => None,
        });

    // Extract LANGUAGE parameter
    let language = get_language(&prop.parameters);

    Ok(Attendee {
        cal_address,
        cn,
        role,
        part_stat,
        rsvp,
        cutype,
        member,
        delegated_to,
        delegated_from,
        dir,
        sent_by,
        language,
    })
}
