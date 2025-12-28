// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Attendee property for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::{SemanticError, Uri};
use crate::typed::{
    CalendarUserType, ParticipationRole, ParticipationStatus, PropertyKind, TypedParameter,
    TypedParameterKind, TypedProperty,
};

/// Attendee information
#[derive(Debug, Clone)]
pub struct Attendee {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: Uri,

    /// Common name (optional)
    pub cn: Option<String>,

    /// Participation role
    pub role: ParticipationRole,

    /// Participation status
    pub part_stat: ParticipationStatus,

    /// RSVP expectation
    pub rsvp: Option<bool>,

    /// Whether the attendee is required
    pub cutype: CalendarUserType,

    /// Member of a group (optional)
    pub member: Option<Uri>,

    /// Delegated to (optional)
    pub delegated_to: Option<Uri>,

    /// Delegated from (optional)
    pub delegated_from: Option<Uri>,

    /// Directory entry reference (optional)
    pub dir: Option<Uri>,

    /// Sent by (optional)
    pub sent_by: Option<Uri>,

    /// Language (optional)
    pub language: Option<String>,
}

impl TryFrom<&TypedProperty<'_>> for Attendee {
    type Error = SemanticError;

    #[allow(clippy::too_many_lines)]
    fn try_from(prop: &TypedProperty<'_>) -> Result<Self, Self::Error> {
        let value = prop.values.first().ok_or(SemanticError::MissingValue {
            property: PropertyKind::Attendee,
        })?;

        let cal_address = Uri::try_from(value).map_err(|_| SemanticError::InvalidValue {
            property: PropertyKind::Attendee,
            value: "Expected calendar user address".to_string(),
        })?;

        // Collect all optional parameters in a single pass
        let mut cn = None;
        let mut role = None;
        let mut part_stat = None;
        let mut rsvp = None;
        let mut cutype = None;
        let mut member = None;
        let mut delegated_to = None;
        let mut delegated_from = None;
        let mut dir = None;
        let mut sent_by = None;
        let mut language = None;

        for param in &prop.parameters {
            match param.kind() {
                TypedParameterKind::CommonName => {
                    if let TypedParameter::CommonName { value, .. } = param {
                        cn = Some(value.resolve().to_string());
                    }
                }
                TypedParameterKind::ParticipationRole => {
                    if let TypedParameter::ParticipationRole { value, .. } = param {
                        role = Some(*value);
                    }
                }
                TypedParameterKind::ParticipationStatus => {
                    if let TypedParameter::ParticipationStatus { value, .. } = param {
                        part_stat = Some(*value);
                    }
                }
                TypedParameterKind::RsvpExpectation => {
                    if let TypedParameter::RsvpExpectation { value, .. } = param {
                        rsvp = Some(*value);
                    }
                }
                TypedParameterKind::CalendarUserType => {
                    if let TypedParameter::CalendarUserType { value, .. } = param {
                        cutype = Some(*value);
                    }
                }
                TypedParameterKind::GroupOrListMembership => {
                    if let TypedParameter::GroupOrListMembership { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        member = Some(Uri {
                            uri: v.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::Delegatees => {
                    if let TypedParameter::Delegatees { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        delegated_to = Some(Uri {
                            uri: v.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::Delegators => {
                    if let TypedParameter::Delegators { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        delegated_from = Some(Uri {
                            uri: v.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::Directory => {
                    if let TypedParameter::Directory { value, .. } = param {
                        dir = Some(Uri {
                            uri: value.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::SendBy => {
                    if let TypedParameter::SendBy { value, .. } = param {
                        sent_by = Some(Uri {
                            uri: value.resolve().to_string(),
                        });
                    }
                }
                TypedParameterKind::Language => {
                    if let TypedParameter::Language { value, .. } = param {
                        language = Some(value.resolve().to_string());
                    }
                }
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Apply defaults as per RFC 5545
        let role = role.unwrap_or(ParticipationRole::ReqParticipant);
        let part_stat = part_stat.unwrap_or(ParticipationStatus::NeedsAction);
        let cutype = cutype.unwrap_or(CalendarUserType::Individual);

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
}
