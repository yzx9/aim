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
use crate::property::util::take_single_text;
use crate::string_storage::{SpannedSegments, StringStorage};
use crate::typed::{ParsedProperty, TypedError};
use crate::value::ValueText;

define_prop_value_enum! {
    /// Calendar scale value (RFC 5545 Section 3.7.1)
    #[derive(Default)]
    pub enum CalendarScaleValue {
        /// Gregorian calendar
        #[default]
        Gregorian => KW_CALSCALE_GREGORIAN
    }
}

/// Calendar scale specification (RFC 5545 Section 3.7.1)
#[derive(Debug, Clone, Default)]
pub struct CalendarScale<S: StringStorage> {
    /// Calendar scale value
    pub value: CalendarScaleValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for CalendarScale<SpannedSegments<'src>> {
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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::CalScale, prop.value)?;
        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::CalScale,
                value: format!("Unsupported calendar scale: {text}"),
                span: value_span,
            }]
        })?;

        Ok(CalendarScale {
            value,
            x_parameters,
            unrecognized_parameters,
            span: prop.span,
        })
    }
}

impl CalendarScale<SpannedSegments<'_>> {
    /// Convert borrowed `CalendarScale` to owned `CalendarScale`
    #[must_use]
    pub fn to_owned(&self) -> CalendarScale<String> {
        CalendarScale {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

define_prop_value_enum! {
    /// Method value for iCalendar objects (RFC 5545 Section 3.7.2)
    #[derive(Default)]
    pub enum MethodValue {
        /// Publish an event (most common)
        #[default]
        Publish         => KW_METHOD_PUBLISH,

        /// Request an event
        Request         => KW_METHOD_REQUEST,

        /// Reply to an event
        Reply           => KW_METHOD_REPLY,

        /// Add an event
        Add             => KW_METHOD_ADD,

        /// Cancel an event
        Cancel          => KW_METHOD_CANCEL,

        /// Refresh an event
        Refresh         => KW_METHOD_REFRESH,

        /// Counter an event
        Counter         => KW_METHOD_COUNTER,

        /// Decline counter
        DeclineCounter  => KW_METHOD_DECLINECOUNTER,
    }
}

/// Method type for iCalendar objects (RFC 5545 Section 3.7.2)
#[derive(Debug, Clone, Default)]
pub struct Method<S: StringStorage> {
    /// Method value
    pub value: MethodValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Method<SpannedSegments<'src>> {
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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Method, prop.value)?;
        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Method,
                value: format!("Unsupported method type: {text}"),
                span: value_span,
            }]
        })?;

        Ok(Method {
            value,
            x_parameters,
            unrecognized_parameters,
            span: prop.span,
        })
    }
}

impl Method<SpannedSegments<'_>> {
    /// Convert borrowed `Method` to owned `Method`
    #[must_use]
    pub fn to_owned(&self) -> Method<String> {
        Method {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

/// Product identifier that identifies the software that created the iCalendar data (RFC 5545 Section 3.7.3)
#[derive(Debug, Clone, Default)]
pub struct ProductId<S: StringStorage> {
    /// The vendor of the implementation SHOULD assure that this is a globally
    /// unique identifier; using some technique such as an FPI value, as
    /// defined in [ISO.9070.1991].
    pub value: ValueText<S>,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for ProductId<SpannedSegments<'src>> {
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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value = take_single_text(&PropertyKind::ProdId, prop.value)?;
        Ok(ProductId {
            value,
            x_parameters,
            unrecognized_parameters,
            span: prop.span,
        })
    }
}

impl ProductId<SpannedSegments<'_>> {
    /// Convert borrowed `ProductId` to owned `ProductId`
    #[must_use]
    pub fn to_owned(&self) -> ProductId<String> {
        ProductId {
            value: self.value.to_owned(),
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}

define_prop_value_enum! {
    /// iCalendar version value (RFC 5545 Section 3.7.4)
    #[derive(Default)]
    pub enum VersionValue {
        /// Version 2.0 (most common)
        #[default]
        V2_0 => KW_VERSION_2_0,
    }
}

/// iCalendar version specification (RFC 5545 Section 3.7.4)
#[derive(Debug, Clone, Default)]
pub struct Version<S: StringStorage> {
    /// Version value
    pub value: VersionValue,

    /// X-name parameters (custom experimental parameters)
    pub x_parameters: Vec<Parameter<S>>,

    /// Unrecognized parameters (IANA tokens not recognized by this implementation)
    pub unrecognized_parameters: Vec<Parameter<S>>,

    /// Span of the property in the source
    pub span: S::Span,
}

impl<'src> TryFrom<ParsedProperty<'src>> for Version<SpannedSegments<'src>> {
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
                p => {
                    // Preserve other parameters not used by this property for round-trip
                    unrecognized_parameters.push(p);
                }
            }
        }

        let value_span = prop.value.span();
        let text = take_single_text(&PropertyKind::Version, prop.value)?;
        let value = text.try_into().map_err(|text| {
            vec![TypedError::PropertyInvalidValue {
                property: PropertyKind::Version,
                value: format!("Unsupported iCalendar version: {text}"),
                span: value_span,
            }]
        })?;

        Ok(Version {
            value,
            x_parameters,
            unrecognized_parameters,
            span: prop.span,
        })
    }
}

impl Version<SpannedSegments<'_>> {
    /// Convert borrowed `Version` to owned `Version`
    #[must_use]
    pub fn to_owned(&self) -> Version<String> {
        Version {
            value: self.value,
            x_parameters: self.x_parameters.iter().map(Parameter::to_owned).collect(),
            unrecognized_parameters: self
                .unrecognized_parameters
                .iter()
                .map(Parameter::to_owned)
                .collect(),
            span: (),
        }
    }
}
