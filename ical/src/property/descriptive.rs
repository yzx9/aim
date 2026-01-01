// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Descriptive Component Properties (RFC 5545 Section 3.8.1)
//!
//! This module contains property types for the "Descriptive Component Properties"
//! section of RFC 5545. Properties are organized into three categories:
//!
//! ## Complex Property Types
//!
//! - 3.8.1.1: `Attachment` - Attached documents or resources
//! - 3.8.1.3: `Classification` - Access classification (PUBLIC, PRIVATE, CONFIDENTIAL)
//! - 3.8.1.6: `Geo` - Geographic position (latitude/longitude)
//! - 3.8.4.3: `Organizer` - Event organizer
//!
//! ## Text Property Wrapper Types
//!
//! Each text wrapper implements `Deref` and `DerefMut` to the `Text` type,
//! which provides content, language, and altrep parameters. All wrappers
//! validate their property kind during conversion:
//!
//! - 3.8.1.2: `Categories` - Categories or tags (multi-valued)
//! - 3.8.1.4: `Comment` - Non-processing comments
//! - 3.8.1.5: `Description` - Detailed description
//! - 3.8.1.7: `Location` - Venue location
//! - 3.8.1.10: `Resources` - Resources (multi-valued)
//! - 3.8.1.12: `Summary` - Summary/subject
//! - 3.8.4.2: `Contact` - Contact information
//! - 3.8.4.5: `RelatedTo` - Related to another component
//! - 3.8.8.3: `RequestStatus` - Status code for request processing
//!
//! ## URI/Identifier Wrapper Types
//!
//! These wrappers also implement `Deref`/`DerefMut` to `Text` and validate
//! their property kind:
//!
//! - 3.8.4.6: `Url` - Uniform Resource Locator
//! - 3.8.4.7: `Uid` - Unique identifier
//! - 3.8.3.1: `TzId` - Time zone identifier
//! - 3.8.3.2: `TzName` - Time zone name
//! - 3.8.3.5: `TzUrl` - Time zone URL
//!
//! ## Implementation Notes
//!
//! All wrapper types use the `Text` or `Texts` utility types from the `util` module,
//! which provide common functionality for text properties including language
//! and alternate text representation (altrep) parameters.

use std::convert::TryFrom;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use chumsky::{Parser, error::Rich, extra, input::Stream};

use crate::keyword::{KW_CLASS_CONFIDENTIAL, KW_CLASS_PRIVATE, KW_CLASS_PUBLIC};
use crate::parameter::{Encoding, Parameter};
use crate::property::PropertyKind;
use crate::property::util::{Text, Texts, take_single_string, take_single_text, take_single_value};
use crate::syntax::SpannedSegments;
use crate::typed::{ParsedProperty, TypedError};
use crate::value::{Value, ValueText, values_float_semicolon};

/// Simple text property wrapper (RFC 5545 Section 3.8.1.4)
///
/// This is a wrapper type for simple text properties that only contain
/// a single Text value with optional language and altrep parameters.
#[derive(Debug, Clone)]
pub struct Comment<'src>(pub Text<'src>);

impl<'src> Deref for Comment<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Comment<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Comment<'_> {
    /// Get the property kind for Comment
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Comment
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Comment<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Comment)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.1.5)
#[derive(Debug, Clone)]
pub struct Description<'src>(pub Text<'src>);

impl<'src> Deref for Description<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Description<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Description<'_> {
    /// Get the property kind for `Description`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Description
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Description<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Description)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.1.7)
#[derive(Debug, Clone)]
pub struct Location<'src>(pub Text<'src>);

impl<'src> Deref for Location<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Location<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Location<'_> {
    /// Get the property kind for Location
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Location
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Location<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Location)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.1.12)
#[derive(Debug, Clone)]
pub struct Summary<'src>(pub Text<'src>);

impl<'src> Deref for Summary<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Summary<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Summary<'_> {
    /// Get the property kind for Summary
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Summary
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Summary<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Summary)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.4.2)
#[derive(Debug, Clone)]
pub struct Contact<'src>(pub Text<'src>);

impl<'src> Deref for Contact<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Contact<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Contact<'_> {
    /// Get the property kind for Contact
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Contact
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Contact<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Contact)
    }
}

/// Multi-valued text property wrapper (RFC 5545 Section 3.8.1.2)
#[derive(Debug, Clone)]
pub struct Categories<'src>(pub Texts<'src>);

impl<'src> Deref for Categories<'src> {
    type Target = Texts<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Categories<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Categories<'_> {
    /// Get the property kind for Categories
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Categories
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Categories<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Texts::try_from(prop).map(Categories)
    }
}

/// Multi-valued text property wrapper (RFC 5545 Section 3.8.1.10)
#[derive(Debug, Clone)]
pub struct Resources<'src>(pub Texts<'src>);

impl<'src> Deref for Resources<'src> {
    type Target = Texts<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Resources<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Resources<'_> {
    /// Get the property kind for Resources
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Resources
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Resources<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Texts::try_from(prop).map(Resources)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.8.3)
#[derive(Debug, Clone)]
pub struct RequestStatus<'src>(pub Text<'src>);

impl<'src> Deref for RequestStatus<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RequestStatus<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RequestStatus<'_> {
    /// Get the property kind for `RequestStatus`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::RequestStatus
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for RequestStatus<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(RequestStatus)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.4.5)
#[derive(Debug, Clone)]
pub struct RelatedTo<'src>(pub Text<'src>);

impl<'src> Deref for RelatedTo<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelatedTo<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RelatedTo<'_> {
    /// Get the property kind for `RelatedTo`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::RelatedTo
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for RelatedTo<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(RelatedTo)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.4.7)
#[derive(Debug, Clone)]
pub struct Url<'src>(pub Text<'src>);

impl<'src> Deref for Url<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Url<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Url<'_> {
    /// Get the property kind for `Url`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Url
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Url<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Url)
    }
}

/// Simple text property wrapper (RFC 5545 Section 3.8.4.9)
#[derive(Debug, Clone)]
pub struct Uid<'src>(pub Text<'src>);

impl Uid<'_> {
    /// Get the property kind for `Uid`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Uid
    }
}

impl<'src> Deref for Uid<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uid<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Uid<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(Uid)
    }
}

/// Simple text property wrapper for `TzId` (RFC 5545 Section 3.8.3.1)
#[derive(Debug, Clone)]
pub struct TzId<'src>(pub Text<'src>);

impl<'src> Deref for TzId<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzId<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzId<'_> {
    /// Get the property kind for `TzId`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzId
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzId<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzId)
    }
}

/// Simple text property wrapper for `TzName` (RFC 5545 Section 3.8.3.2)
#[derive(Debug, Clone)]
pub struct TzName<'src>(pub Text<'src>);

impl<'src> Deref for TzName<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzName<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzName<'_> {
    /// Get the property kind for `TzName`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzName
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzName<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzName)
    }
}

/// Simple text property wrapper for `TzUrl` (RFC 5545 Section 3.8.3.5)
#[derive(Debug, Clone)]
pub struct TzUrl<'src>(pub Text<'src>);

impl<'src> Deref for TzUrl<'src> {
    type Target = Text<'src>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TzUrl<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TzUrl<'_> {
    /// Get the property kind for `TzUrl`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::TzUrl
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for TzUrl<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        Text::try_from(prop).map(TzUrl)
    }
}

/// Geographic position (RFC 5545 Section 3.8.1.6)
#[derive(Debug, Clone, Copy)]
pub struct Geo {
    /// Latitude
    pub lat: f64,

    /// Longitude
    pub lon: f64,
}

impl Geo {
    /// Get the property kind for `Geo`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Geo
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Geo {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;

        // Use the typed phase's float parser with semicolon separator
        let stream = Stream::from_iter(text.chars());
        let parser = values_float_semicolon::<_, extra::Err<Rich<char, _>>>();

        match parser.parse(stream).into_result() {
            Ok(result) => match (result.first(), result.get(1)) {
                (Some(&lat), Some(&lon)) => Ok(Geo { lat, lon }),
                (_, _) => Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::Geo,
                    value: format!(
                        "Expected exactly 2 float values (lat;long), got {}",
                        result.len()
                    ),
                    span: prop.span,
                }]),
            },
            Err(_) => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Geo,
                value: format!("Expected 'lat;long' format with semicolon separator, got {text}"),
                span: prop.span,
            }]),
        }
    }
}

/// Classification of calendar data (RFC 5545 Section 3.8.1.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Classification {
    /// Public classification
    #[default]
    Public,

    /// Private classification
    Private,

    /// Confidential classification
    Confidential,
    // /// Custom classification
    // Custom(String),
}

impl Classification {
    /// Get the property kind for `Classification`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Class
    }
}

impl FromStr for Classification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            KW_CLASS_PUBLIC => Ok(Self::Public),
            KW_CLASS_PRIVATE => Ok(Self::Private),
            KW_CLASS_CONFIDENTIAL => Ok(Self::Confidential),
            _ => Err(format!("Invalid classification: {s}")),
        }
    }
}

impl AsRef<str> for Classification {
    fn as_ref(&self) -> &str {
        match self {
            Self::Public => KW_CLASS_PUBLIC,
            Self::Private => KW_CLASS_PRIVATE,
            Self::Confidential => KW_CLASS_CONFIDENTIAL,
        }
    }
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Classification {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if prop.kind != Self::kind() {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: Self::kind(),
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let text = take_single_string(Self::kind(), prop.values).map_err(|e| vec![e])?;
        text.parse().map_err(|e| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Class,
                value: e,
                span: prop.span,
            }]
        })
    }
}

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
}

impl Organizer<'_> {
    /// Get the property kind for `Organizer`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Organizer
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Organizer<'src> {
    type Error = Vec<TypedError<'src>>;

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

        Ok(Organizer {
            cal_address,
            cn,
            dir,
            sent_by,
            language,
        })
    }
}

/// Attachment information (RFC 5545 Section 3.8.1.1)
#[derive(Debug, Clone)]
pub struct Attachment<'src> {
    /// URI or binary data
    pub value: AttachmentValue<'src>,

    /// Format type (optional)
    pub fmt_type: Option<SpannedSegments<'src>>,

    /// Encoding (optional)
    pub encoding: Option<Encoding>,
}

/// Attachment value (URI or binary)
#[derive(Debug, Clone)]
pub enum AttachmentValue<'src> {
    /// URI reference
    Uri(ValueText<'src>),

    /// Binary data
    Binary(SpannedSegments<'src>),
}

impl Attachment<'_> {
    /// Get the property kind for `Attachment`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Attach
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Attachment<'src> {
    type Error = Vec<TypedError<'src>>;

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
        let mut fmt_type = None;
        let mut encoding = None;

        for param in prop.parameters {
            let kind_name = param.kind().name();
            let param_span = param.span();

            match param {
                Parameter::FormatType { value, .. } => match fmt_type {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => fmt_type = Some(value),
                },
                Parameter::Encoding { value, .. } => match encoding {
                    Some(_) => errors.push(TypedError::ParameterDuplicated {
                        parameter: kind_name,
                        span: param_span,
                    }),
                    None => encoding = Some(value),
                },
                // Ignore unknown parameters
                _ => {}
            }
        }

        // Get value
        let value = match take_single_value(prop.kind, prop.values) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                return Err(errors);
            }
        };

        // Return all errors if any occurred
        if !errors.is_empty() {
            return Err(errors);
        }

        match value {
            Value::Text(uri) => Ok(Attachment {
                value: AttachmentValue::Uri(uri.clone()),
                fmt_type,
                encoding,
            }),
            Value::Binary(data) => Ok(Attachment {
                value: AttachmentValue::Binary(data.clone()),
                fmt_type,
                encoding,
            }),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Attach,
                value: "Expected URI or binary value".to_string(),
                span: prop.span,
            }]),
        }
    }
}
