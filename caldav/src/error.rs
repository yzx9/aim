// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::types::Href;

/// `CalDAV` client errors.
#[non_exhaustive]
#[derive(Debug)]
pub enum CalDavError {
    /// HTTP layer error.
    Http(String),

    /// XML parsing/writing error.
    Xml(String),

    /// iCalendar parsing error.
    Ical(String),

    /// Authentication error.
    Auth(String),

    /// Resource not found.
    NotFound(Href),

    /// Precondition failed (`ETag` mismatch).
    PreconditionFailed(String),

    /// Server doesn't support `CalDAV`.
    NotACalDavServer,

    /// Invalid response from server.
    InvalidResponse(String),

    /// Configuration error.
    Config(String),

    /// Server doesn't support required capability.
    UnsupportedCapability(String),
}

impl fmt::Display for CalDavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::Ical(e) => write!(f, "iCalendar parsing error: {e}"),
            Self::Auth(e) => write!(f, "Authentication failed: {e}"),
            Self::NotFound(href) => write!(f, "Resource not found: {href}"),
            Self::PreconditionFailed(e) => write!(f, "Precondition failed: {e}"),
            Self::NotACalDavServer => write!(f, "Server doesn't support CalDAV"),
            Self::InvalidResponse(e) => write!(f, "Invalid server response: {e}"),
            Self::Config(e) => write!(f, "Configuration error: {e}"),
            Self::UnsupportedCapability(cap) => {
                write!(f, "Server doesn't support required capability: {cap}")
            }
        }
    }
}

impl std::error::Error for CalDavError {}

impl From<reqwest::Error> for CalDavError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<quick_xml::Error> for CalDavError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e.to_string())
    }
}

impl From<std::io::Error> for CalDavError {
    fn from(e: std::io::Error) -> Self {
        Self::Xml(format!("IO error: {e}"))
    }
}
