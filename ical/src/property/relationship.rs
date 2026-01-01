// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Component Relationship Properties (RFC 5545 Section 3.8.4)
//!
//! This module contains property types for the "Component Relationship Properties"
//! section of RFC 5545. All types implement `kind()` methods and validate their
//! property kind during conversion from `ParsedProperty`:
//!
//! - 3.8.4.1: `Attendee` - Event participant with calendar user address and
//!   participation parameters (CUType, Role, PartStat, etc.)
//! - 3.8.4.3: `Organizer` - Event organizer with calendar user address
//!   and sent-by parameter

use std::convert::TryFrom;

use crate::parameter::{CalendarUserType, Parameter, ParticipationRole, ParticipationStatus};
use crate::property::PropertyKind;
use crate::property::util::take_single_text;
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::ValueText;

/// Attendee information (RFC 5545 Section 3.8.4.1)
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

impl Attendee<'_> {
    /// Get the property kind for `Attendee`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Attendee
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Attendee<'src> {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

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
            let kind_name = param.kind().name();
            let param_span = param.span();

            match param {
                Parameter::CommonName { value, .. } => match cn {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => cn = Some(value),
                },
                Parameter::ParticipationRole { value, .. } => match role {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => role = Some(value),
                },
                Parameter::ParticipationStatus { value, .. } => match part_stat {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => part_stat = Some(value),
                },
                Parameter::RsvpExpectation { value, .. } => match rsvp {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => rsvp = Some(value),
                },
                Parameter::CalendarUserType { value, .. } => match cutype {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => cutype = Some(value),
                },
                Parameter::GroupOrListMembership { mut values, .. } => match member {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => {
                        // RFC 5545: MEMBER parameter is single-valued for Attendee
                        if values.len() == 1 {
                            member = values.pop();
                        } else {
                            errors.push(TypedError::PropertyInvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "MEMBER parameter expects 1 value, got {}",
                                    values.len()
                                ),
                                span: param_span,
                            });
                        }
                    }
                },
                Parameter::Delegatees { mut values, .. } => match delegated_to {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => {
                        // RFC 5545: DELEGATED-TO parameter is single-valued for Attendee
                        if values.len() == 1 {
                            delegated_to = values.pop();
                        } else {
                            errors.push(TypedError::PropertyInvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "DELEGATED-TO parameter expects 1 value, got {}",
                                    values.len()
                                ),
                                span: param_span,
                            });
                        }
                    }
                },
                Parameter::Delegators { mut values, .. } => match delegated_from {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => {
                        // RFC 5545: DELEGATED-FROM parameter is single-valued for Attendee
                        if values.len() == 1 {
                            delegated_from = values.pop();
                        } else {
                            errors.push(TypedError::PropertyInvalidValue {
                                property: PropertyKind::Attendee,
                                value: format!(
                                    "DELEGATED-FROM parameter expects 1 value, got {}",
                                    values.len()
                                ),
                                span: param_span,
                            });
                        }
                    }
                },
                Parameter::Directory { value, .. } => match dir {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => dir = Some(value),
                },
                Parameter::SendBy { value, .. } => match sent_by {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => sent_by = Some(value),
                },
                Parameter::Language { value, .. } => match language {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => language = Some(value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Get cal_address value
        let cal_address = match take_single_text(prop.kind, prop.values) {
            Ok(text) => text,
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
