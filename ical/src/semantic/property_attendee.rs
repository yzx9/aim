// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Attendee property for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::SemanticError;
use crate::semantic::property_common::take_single_value;
use crate::syntax::SpannedSegments;
use crate::parameter::{CalendarUserType, ParticipationRole, ParticipationStatus, TypedParameter, TypedParameterKind};
use crate::typed::{PropertyKind, TypedProperty, Value};
use crate::value::ValueText;

/// Attendee information
#[derive(Debug, Clone)]
pub struct Attendee<'src> {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: ValueText<'src>,

    /// Common name (optional)
    pub cn: Option<SpannedSegments<'src>>,

    /// Participation role
    pub role: ParticipationRole,

    /// Participation status
    pub part_stat: ParticipationStatus,

    /// RSVP expectation
    pub rsvp: Option<bool>,

    /// Whether the attendee is required
    pub cutype: CalendarUserType,

    /// Member of a group (optional)
    pub member: Option<SpannedSegments<'src>>,

    /// Delegated to (optional)
    pub delegated_to: Option<SpannedSegments<'src>>,

    /// Delegated from (optional)
    pub delegated_from: Option<SpannedSegments<'src>>,

    /// Directory entry reference (optional)
    pub dir: Option<SpannedSegments<'src>>,

    /// Sent by (optional)
    pub sent_by: Option<SpannedSegments<'src>>,

    /// Language (optional)
    pub language: Option<SpannedSegments<'src>>,
}

impl<'src> TryFrom<TypedProperty<'src>> for Attendee<'src> {
    type Error = Vec<SemanticError>;

    #[allow(clippy::too_many_lines)]
    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

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

        for param in prop.parameters {
            match param {
                TypedParameter::CommonName { value, .. } => match cn {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::CommonName,
                    }),
                    None => cn = Some(value),
                },
                TypedParameter::ParticipationRole { value, .. } => match role {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::ParticipationRole,
                    }),
                    None => role = Some(value),
                },
                TypedParameter::ParticipationStatus { value, .. } => match part_stat {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::ParticipationStatus,
                    }),
                    None => part_stat = Some(value),
                },
                TypedParameter::RsvpExpectation { value, .. } => match rsvp {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::RsvpExpectation,
                    }),
                    None => rsvp = Some(value),
                },
                TypedParameter::CalendarUserType { value, .. } => match cutype {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::CalendarUserType,
                    }),
                    None => cutype = Some(value),
                },
                TypedParameter::GroupOrListMembership { mut values, .. } => match member {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::GroupOrListMembership,
                    }),
                    None => {
                        // RFC 5545: MEMBER parameter is single-valued for Attendee
                        if values.len() == 1 {
                            member = values.pop();
                        } else {
                            errors.push(SemanticError::InvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "MEMBER parameter expects 1 value, got {}",
                                    values.len()
                                ),
                            });
                        }
                    }
                },
                TypedParameter::Delegatees { mut values, .. } => match delegated_to {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::Delegatees,
                    }),
                    None => {
                        // RFC 5545: DELEGATED-TO parameter is single-valued for Attendee
                        if values.len() == 1 {
                            delegated_to = values.pop();
                        } else {
                            errors.push(SemanticError::InvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "DELEGATED-TO parameter expects 1 value, got {}",
                                    values.len()
                                ),
                            });
                        }
                    }
                },
                TypedParameter::Delegators { mut values, .. } => match delegated_from {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::Delegators,
                    }),
                    None => {
                        // RFC 5545: DELEGATED-FROM parameter is single-valued for Attendee
                        if values.len() == 1 {
                            delegated_from = values.pop();
                        } else {
                            errors.push(SemanticError::InvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "DELEGATED-FROM parameter expects 1 value, got {}",
                                    values.len()
                                ),
                            });
                        }
                    }
                },
                TypedParameter::Directory { value, .. } => match dir {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::Directory,
                    }),
                    None => dir = Some(value),
                },
                TypedParameter::SendBy { value, .. } => match sent_by {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::SendBy,
                    }),
                    None => sent_by = Some(value),
                },
                TypedParameter::Language { value, .. } => match language {
                    Some(_) => errors.push(SemanticError::DuplicateParameter {
                        parameter: TypedParameterKind::Language,
                    }),
                    None => language = Some(value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Get cal_address value
        let cal_address = match take_single_value(prop.kind, prop.values) {
            Ok(Value::Text(text)) => text,
            Ok(_) => {
                errors.push(SemanticError::InvalidValue {
                    property: PropertyKind::Attendee,
                    value: "Expected calendar user address".to_string(),
                });
                return Err(errors);
            }
            Err(e) => {
                errors.push(e);
                return Err(errors);
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
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
