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
//!   participation parameters
//! - 3.8.4.2: `Contact` - Contact information
//! - 3.8.4.3: `Organizer` - Event organizer
//! - 3.8.4.3: `Organizer` - Event organizer with calendar user address and
//!   sent-by parameter
//! - 3.8.4.4: `RecurrenceId` - Recurrence ID
//! - 3.8.4.5: `RelatedTo` - Related to another component
//! - 3.8.4.6: `Url` - Uniform Resource Locator
//! - 3.8.4.7: `Uid` - Unique identifier

use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use crate::DateTime;
use crate::parameter::{CalendarUserType, Parameter, ParticipationRole, ParticipationStatus};
use crate::property::PropertyKind;
use crate::property::util::{Text, take_single_text};
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
    pub role: ParticipationRole<'src>,

    /// Participation status
    pub part_stat: ParticipationStatus<'src>,

    /// RSVP expectation
    pub rsvp: Option<bool>,

    /// Whether the attendee is required
    pub cutype: CalendarUserType<'src>,

    /// Member of a group (optional, multi-valued)
    pub member: Option<Vec<SpannedSegments<'src>>>,

    /// Delegated to (optional, multi-valued)
    pub delegated_to: Option<Vec<SpannedSegments<'src>>>,

    /// Delegated from (optional, multi-valued)
    pub delegated_from: Option<Vec<SpannedSegments<'src>>>,

    /// Directory entry reference (optional)
    pub dir: Option<SpannedSegments<'src>>,

    /// Sent by (optional)
    pub sent_by: Option<SpannedSegments<'src>>,

    /// Language (optional)
    pub language: Option<SpannedSegments<'src>>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Attendee<'src> {
    type Error = Vec<TypedError<'src>>;

    #[expect(clippy::too_many_lines)]
    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Attendee) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Attendee,
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
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::CommonName { .. } if cn.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::CommonName { value, .. } => cn = Some(value),

                p @ Parameter::ParticipationRole { .. } if role.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::ParticipationRole { value, .. } => role = Some(value),

                p @ Parameter::ParticipationStatus { .. } if part_stat.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::ParticipationStatus { value, .. } => part_stat = Some(value),

                p @ Parameter::RsvpExpectation { .. } if rsvp.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::RsvpExpectation { value, .. } => rsvp = Some(value),

                p @ Parameter::CalendarUserType { .. } if cutype.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::CalendarUserType { value, .. } => cutype = Some(value),

                p @ Parameter::GroupOrListMembership { .. } if member.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::GroupOrListMembership { values, .. } => member = Some(values),

                p @ Parameter::Delegatees { .. } if delegated_to.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Delegatees { values, .. } => delegated_to = Some(values),

                p @ Parameter::Delegators { .. } if delegated_from.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Delegators { values, .. } => delegated_from = Some(values),

                p @ Parameter::Directory { .. } if dir.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Directory { value, .. } => dir = Some(value),

                p @ Parameter::SendBy { .. } if sent_by.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::SendBy { value, .. } => sent_by = Some(value),

                p @ Parameter::Language { .. } if language.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Language { value, .. } => language = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        // Get cal_address value
        let cal_address = match take_single_text(&prop.kind, prop.values) {
            Ok(text) => Some(text),
            Err(e) => {
                errors.extend(e);
                None
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
            cal_address: cal_address.unwrap(), // SAFETY: ensured above
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
            x_parameters,
            unrecognized_parameters,
        })
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.4.2)
    Contact<'src>: Text<'src> => Contact
);

/// Organizer information (RFC 5545 Section 3.8.4.3)
#[derive(Debug, Clone)]
pub struct Organizer<'src> {
    /// Calendar user address (mailto: or other URI)
    pub cal_address: ValueText<'src>, // TODO: parse mailto:

    /// Common name (optional)
    pub cn: Option<SpannedSegments<'src>>,

    /// Directory entry reference (optional)
    pub dir: Option<SpannedSegments<'src>>,

    /// Sent by (optional)
    pub sent_by: Option<SpannedSegments<'src>>,

    /// Language (optional)
    pub language: Option<SpannedSegments<'src>>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Organizer<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Organizer) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Organizer,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut errors = Vec::new();

        // Collect all optional parameters in a single pass
        let mut cn = None;
        let mut dir = None;
        let mut sent_by = None;
        let mut language = None;
        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::CommonName { .. } if cn.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::CommonName { value, .. } => cn = Some(value),

                p @ Parameter::Directory { .. } if dir.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Directory { value, .. } => dir = Some(value),

                p @ Parameter::SendBy { .. } if sent_by.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::SendBy { value, .. } => sent_by = Some(value),

                p @ Parameter::Language { .. } if language.is_some() => {
                    errors.push(TypedError::ParameterDuplicated {
                        span: p.span(),
                        parameter: p.into_kind(),
                    });
                }
                Parameter::Language { value, .. } => language = Some(value),

                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        // Get cal_address value
        let cal_address = match take_single_text(&prop.kind, prop.values) {
            Ok(text) => Some(text),
            Err(e) => {
                errors.extend(e);
                None
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Organizer {
            cal_address: cal_address.unwrap(), // SAFETY: ensured above
            cn,
            dir,
            sent_by,
            language,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// Recurrence ID property wrapper (RFC 5545 Section 3.8.4.4)
#[derive(Debug, Clone)]
pub struct RecurrenceId<'src>(pub DateTime<'src>);

impl<'src> Deref for RecurrenceId<'src> {
    type Target = DateTime<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RecurrenceId<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for RecurrenceId<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::RecurrenceId) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::RecurrenceId,
                found: prop.kind,
                span: prop.span,
            }]);
        }
        DateTime::try_from(prop).map(RecurrenceId)
    }
}

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.4.5)
    RelatedTo<'src>: Text<'src> => RelatedTo
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.4.6)
    Url<'src>: Text<'src> => Url
);

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.4.7)
    Uid<'src>: Text<'src> => Uid
);
