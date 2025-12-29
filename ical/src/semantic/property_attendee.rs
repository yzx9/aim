// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Attendee property for iCalendar semantic components.

use std::convert::TryFrom;

use crate::semantic::SemanticError;
use crate::semantic::property_common::take_single_value;
use crate::syntax::SpannedSegments;
use crate::typed::{
    CalendarUserType, ParticipationRole, ParticipationStatus, PropertyKind, TypedParameter,
    TypedParameterKind, TypedProperty, Value, ValueText,
};

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
    type Error = SemanticError;

    #[allow(clippy::too_many_lines)]
    fn try_from(prop: TypedProperty<'src>) -> Result<Self, Self::Error> {
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
            match param.kind() {
                TypedParameterKind::CommonName => {
                    if let TypedParameter::CommonName { value, .. } = param {
                        cn = Some(value);
                    }
                }
                TypedParameterKind::ParticipationRole => {
                    if let TypedParameter::ParticipationRole { value, .. } = param {
                        role = Some(value);
                    }
                }
                TypedParameterKind::ParticipationStatus => {
                    if let TypedParameter::ParticipationStatus { value, .. } = param {
                        part_stat = Some(value);
                    }
                }
                TypedParameterKind::RsvpExpectation => {
                    if let TypedParameter::RsvpExpectation { value, .. } = param {
                        rsvp = Some(value);
                    }
                }
                TypedParameterKind::CalendarUserType => {
                    if let TypedParameter::CalendarUserType { value, .. } = param {
                        cutype = Some(value);
                    }
                }
                TypedParameterKind::GroupOrListMembership => {
                    if let TypedParameter::GroupOrListMembership { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        member = Some(v.clone()); // PERF: avoid allocation
                    }
                }
                TypedParameterKind::Delegatees => {
                    if let TypedParameter::Delegatees { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        delegated_to = Some(v.clone()); // PERF: avoid allocation
                    }
                }
                TypedParameterKind::Delegators => {
                    if let TypedParameter::Delegators { values, .. } = param
                        && let Some(v) = values.first()
                    {
                        delegated_from = Some(v.clone()); // PERF: avoid allocation
                    }
                }
                TypedParameterKind::Directory => {
                    if let TypedParameter::Directory { value, .. } = param {
                        dir = Some(value);
                    }
                }
                TypedParameterKind::SendBy => {
                    if let TypedParameter::SendBy { value, .. } = param {
                        sent_by = Some(value);
                    }
                }
                TypedParameterKind::Language => {
                    if let TypedParameter::Language { value, .. } = param {
                        language = Some(value);
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

        let Ok(Value::Text(cal_address)) = take_single_value(prop.kind, prop.values) else {
            return Err(SemanticError::InvalidValue {
                property: PropertyKind::Attendee,
                value: "Expected calendar user address".to_string(),
            });
        };

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
