// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Calendar Properties (RFC 5545 Section 3.7)
//!
//! This module contains property types for the "Calendar Properties"
//! section of RFC 5545. All types implement `kind()` methods and validate
//! their property kind during conversion from `ParsedProperty`:
//!
//! - 3.7.1: `CalendarScale` - Calendar scale (GREGORIAN)
//! - 3.7.2: `Method` - iTIP method (PUBLISH, REQUEST, etc.)
//! - 3.7.3: `ProductId` - Product identifier (vendor/product info)
//! - 3.7.4: `Version` - iCalendar version (2.0)

use std::convert::TryFrom;

use crate::keyword::{
    KW_CALSCALE_GREGORIAN, KW_METHOD_ADD, KW_METHOD_CANCEL, KW_METHOD_COUNTER,
    KW_METHOD_DECLINECOUNTER, KW_METHOD_PUBLISH, KW_METHOD_REFRESH, KW_METHOD_REPLY,
    KW_METHOD_REQUEST, KW_VERSION_2_0,
};
use crate::property::PropertyKind;
use crate::property::util::take_single_string;
use crate::typed::{ParsedProperty, TypedError};

/// Calendar scale specification (RFC 5545 Section 3.7.1)
#[derive(Debug, Clone, Copy, Default)]
pub enum CalendarScale {
    /// Gregorian calendar
    #[default]
    Gregorian,
}

impl CalendarScale {
    /// Get the property kind for `CalendarScale`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::CalScale
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for CalendarScale {
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
        match text.to_uppercase().as_str() {
            KW_CALSCALE_GREGORIAN => Ok(CalendarScale::Gregorian),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::CalScale,
                value: format!("Unsupported calendar scale: {text}"),
                span: prop.span,
            }]),
        }
    }
}

/// Method types for iCalendar objects (RFC 5545 Section 3.7.2)
#[derive(Debug, Clone, Copy)]
pub enum Method {
    /// Publish an event
    Publish,

    /// Request an event
    Request,

    /// Reply to an event
    Reply,

    /// Add an event
    Add,

    /// Cancel an event
    Cancel,

    /// Refresh an event
    Refresh,

    /// Counter an event
    Counter,

    /// Decline counter
    DeclineCounter,
}

impl Method {
    /// Get the property kind for `Method`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Method
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Method {
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
        match text.to_uppercase().as_str() {
            KW_METHOD_PUBLISH => Ok(Method::Publish),
            KW_METHOD_REQUEST => Ok(Method::Request),
            KW_METHOD_REPLY => Ok(Method::Reply),
            KW_METHOD_ADD => Ok(Method::Add),
            KW_METHOD_CANCEL => Ok(Method::Cancel),
            KW_METHOD_REFRESH => Ok(Method::Refresh),
            KW_METHOD_COUNTER => Ok(Method::Counter),
            KW_METHOD_DECLINECOUNTER => Ok(Method::DeclineCounter),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Method,
                value: format!("Unsupported method type: {text}"),
                span: prop.span,
            }]),
        }
    }
}

/// Product identifier that identifies the software that created the iCalendar data (RFC 5545 Section 3.7.3)
#[derive(Debug, Clone, Default)]
pub struct ProductId {
    /// Company identifier
    pub company: String,

    /// Product identifier
    pub product: String,

    /// Language of the text (optional)
    pub language: Option<String>,
}

impl ProductId {
    /// Get the property kind for `ProductId`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::ProdId
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for ProductId {
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

        // PRODID format: company//product//language
        // e.g., "-//Mozilla.org/NONSGML Mozilla Calendar V1.0//EN"
        let parts: Vec<_> = text.split("//").collect();
        if parts.len() >= 2 {
            Ok(ProductId {
                company: parts.first().map(|s| (*s).to_string()).unwrap_or_default(),
                product: parts.get(1).map(|s| (*s).to_string()).unwrap_or_default(),
                language: parts.get(2).map(|s| (*s).to_string()),
            })
        } else {
            // If not in the expected format, use the whole string as product
            Ok(ProductId {
                company: String::new(),
                product: text,
                language: None,
            })
        }
    }
}

/// iCalendar version specification (RFC 5545 Section 3.7.4)
#[derive(Debug, Clone, Copy)]
pub enum Version {
    /// Version 2.0 (most common)
    V2_0,
}

impl Version {
    /// Get the property kind for `Version`
    #[must_use]
    pub const fn kind() -> PropertyKind {
        PropertyKind::Version
    }
}

impl<'src> TryFrom<ParsedProperty<'src>> for Version {
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
        match text.as_str() {
            KW_VERSION_2_0 => Ok(Version::V2_0),
            _ => Err(vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Version,
                value: format!("Unsupported iCalendar version: {text}"),
                span: prop.span,
            }]),
        }
    }
}
