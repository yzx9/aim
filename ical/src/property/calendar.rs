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
use crate::parameter::Parameter;
use crate::property::PropertyKind;
use crate::property::util::take_single_string;
use crate::typed::{ParsedProperty, TypedError};

/// Calendar scale value (RFC 5545 Section 3.7.1)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CalendarScaleValue {
    /// Gregorian calendar
    #[default]
    Gregorian,
}

/// Calendar scale specification (RFC 5545 Section 3.7.1)
#[derive(Debug, Clone, Default)]
pub struct CalendarScale<'src> {
    /// Calendar scale value
    pub value: CalendarScaleValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for CalendarScale<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::CalScale) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::CalScale,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_string(&PropertyKind::CalScale, prop.value)?;
        let value = match text.to_uppercase().as_str() {
            KW_CALSCALE_GREGORIAN => CalendarScaleValue::Gregorian,
            _ => {
                return Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::CalScale,
                    value: format!("Unsupported calendar scale: {text}"),
                    span: value_span,
                }]);
            }
        };

        Ok(CalendarScale {
            value,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// Method value for iCalendar objects (RFC 5545 Section 3.7.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MethodValue {
    /// Publish an event (most common)
    #[default]
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

/// Method type for iCalendar objects (RFC 5545 Section 3.7.2)
#[derive(Debug, Clone, Default)]
pub struct Method<'src> {
    /// Method value
    pub value: MethodValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Method<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Method) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Method,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_string(&PropertyKind::Method, prop.value)?;
        let value = match text.to_uppercase().as_str() {
            KW_METHOD_PUBLISH => MethodValue::Publish,
            KW_METHOD_REQUEST => MethodValue::Request,
            KW_METHOD_REPLY => MethodValue::Reply,
            KW_METHOD_ADD => MethodValue::Add,
            KW_METHOD_CANCEL => MethodValue::Cancel,
            KW_METHOD_REFRESH => MethodValue::Refresh,
            KW_METHOD_COUNTER => MethodValue::Counter,
            KW_METHOD_DECLINECOUNTER => MethodValue::DeclineCounter,
            _ => {
                return Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::Method,
                    value: format!("Unsupported method type: {text}"),
                    span: value_span,
                }]);
            }
        };

        Ok(Method {
            value,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// Product identifier that identifies the software that created the iCalendar data (RFC 5545 Section 3.7.3)
#[derive(Debug, Clone, Default)]
pub struct ProductId<'src> {
    /// Company identifier
    pub company: String,

    /// Product identifier
    pub product: String,

    /// Language of the text (optional)
    pub language: Option<String>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for ProductId<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::ProdId) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::ProdId,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let text = take_single_string(&PropertyKind::ProdId, prop.value)?;

        // PRODID format: company//product//language
        // e.g., "-//Mozilla.org/NONSGML Mozilla Calendar V1.0//EN"
        let parts: Vec<_> = text.split("//").collect();
        let (company, product, language) = if parts.len() >= 2 {
            (
                parts.first().map(|s| (*s).to_string()).unwrap_or_default(),
                parts.get(1).map(|s| (*s).to_string()).unwrap_or_default(),
                parts.get(2).map(|s| (*s).to_string()),
            )
        } else {
            // If not in the expected format, use the whole string as product
            (String::new(), text, None)
        };

        Ok(ProductId {
            company,
            product,
            language,
            x_parameters,
            unrecognized_parameters,
        })
    }
}

/// iCalendar version value (RFC 5545 Section 3.7.4)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VersionValue {
    /// Version 2.0 (most common)
    #[default]
    V2_0,
}

/// iCalendar version specification (RFC 5545 Section 3.7.4)
#[derive(Debug, Clone, Default)]
pub struct Version<'src> {
    /// Version value
    pub value: VersionValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<'src>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<'src>>,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Version<'src> {
    type Error = Vec<TypedError<'src>>;

    fn try_from(prop: ParsedProperty<'src>) -> Result<Self, Self::Error> {
        if !matches!(prop.kind, PropertyKind::Version) {
            return Err(vec![TypedError::PropertyUnexpectedKind {
                expected: PropertyKind::Version,
                found: prop.kind,
                span: prop.span,
            }]);
        }

        let mut x_parameters = Vec::new();
        let mut unrecognized_parameters = Vec::new();

        for param in prop.parameters {
            match param {
                p @ Parameter::XName { .. } => x_parameters.push(p),
                p @ Parameter::Unrecognized { .. } => unrecognized_parameters.push(p),
                _ => {}
            }
        }

        let value_span = prop.value.span();
        let text = take_single_string(&PropertyKind::Version, prop.value)?;
        let value = match text.as_str() {
            KW_VERSION_2_0 => VersionValue::V2_0,
            _ => {
                return Err(vec![TypedError::PropertyInvalidValue {
                    property: PropertyKind::Version,
                    value: format!("Unsupported iCalendar version: {text}"),
                    span: value_span,
                }]);
            }
        };

        Ok(Version {
            value,
            x_parameters,
            unrecognized_parameters,
        })
    }
}
