// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parameter formatting for iCalendar parameters.
//!
//! This module provides functions to format all iCalendar parameter types
//! as defined in RFC 5545 Section 3.2.

use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::keyword::{
    KW_ALTREP, KW_CN, KW_CUTYPE, KW_DELEGATED_FROM, KW_DELEGATED_TO, KW_DIR, KW_ENCODING,
    KW_FBTYPE, KW_FMTTYPE, KW_LANGUAGE, KW_MEMBER, KW_PARTSTAT, KW_RANGE, KW_RELATED, KW_RELTYPE,
    KW_ROLE, KW_RSVP, KW_RSVP_FALSE, KW_RSVP_TRUE, KW_SENT_BY, KW_TZID, KW_VALUE,
};
use crate::parameter::Parameter;
use crate::string_storage::StringStorage;

/// Format all parameters to the formatter.
///
/// This formats multiple parameters, each prefixed with a semicolon.
pub fn write_parameters<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    parameters: &[Parameter<S>],
) -> io::Result<()> {
    for param in parameters {
        write!(f, ";")?;
        write_parameter(f, param)?;
    }
    Ok(())
}

/// Format a single parameter.
fn write_parameter<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    param: &Parameter<S>,
) -> io::Result<()> {
    match param {
        Parameter::AlternateText { value, .. } => {
            write!(f, "{KW_ALTREP}={}", quote_if_needed(value.to_string()))
        }
        Parameter::CommonName { value, .. } => {
            write!(f, "{KW_CN}={}", quote_if_needed(value.to_string()))
        }
        Parameter::CalendarUserType { value, .. } => write!(f, "{KW_CUTYPE}={value}"),
        Parameter::Delegators { values, .. } => {
            write!(f, "{KW_DELEGATED_FROM}=")?;
            format_quoted_list_direct(f, values)
        }
        Parameter::Delegatees { values, .. } => {
            write!(f, "{KW_DELEGATED_TO}=")?;
            format_quoted_list_direct(f, values)
        }
        Parameter::Directory { value, .. } => {
            write!(f, "{KW_DIR}={}", quote_if_needed(value.to_string()))
        }
        Parameter::Encoding { value, .. } => write!(f, "{KW_ENCODING}={value}"),
        Parameter::FormatType { value, .. } => {
            write!(f, "{KW_FMTTYPE}={}", quote_if_needed(value.to_string()))
        }
        Parameter::FreeBusyType { value, .. } => write!(f, "{KW_FBTYPE}={value}"),
        Parameter::Language { value, .. } => {
            write!(f, "{KW_LANGUAGE}={}", quote_if_needed(value.to_string()))
        }
        Parameter::GroupOrListMembership { values, .. } => {
            write!(f, "{KW_MEMBER}=")?;
            format_quoted_list_direct(f, values)
        }
        Parameter::ParticipationStatus { value, .. } => write!(f, "{KW_PARTSTAT}={value}"),
        Parameter::RecurrenceIdRange { value, .. } => write!(f, "{KW_RANGE}={value}"),
        Parameter::AlarmTriggerRelationship { value, .. } => write!(f, "{KW_RELATED}={value}"),
        Parameter::RelationshipType { value, .. } => write!(f, "{KW_RELTYPE}={value}"),
        Parameter::ParticipationRole { value, .. } => write!(f, "{KW_ROLE}={value}"),
        Parameter::SendBy { value, .. } => {
            write!(f, "{KW_SENT_BY}={}", quote_if_needed(value.to_string()))
        }
        Parameter::RsvpExpectation { value, .. } => write!(
            f,
            "{KW_RSVP}={}",
            if *value { KW_RSVP_TRUE } else { KW_RSVP_FALSE }
        ),
        Parameter::TimeZoneIdentifier { value, .. } => {
            write!(f, "{KW_TZID}={}", quote_if_needed(value.to_string()))
        }
        Parameter::ValueType { value, .. } => write!(f, "{KW_VALUE}={value}"),
        Parameter::XName(raw) => {
            // XName: name=value
            let name = &raw.name.to_string();
            write!(f, "{name}")?;
            if !raw.values.is_empty() {
                write!(f, "=")?;
                // Format values as comma-separated list (quoted if needed)
                for (i, v) in raw.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    let s = v.value.to_string();
                    write!(f, "{}", quote_if_needed(&s))?;
                }
            }
            Ok(())
        }
        Parameter::Unrecognized(raw) => {
            // Unrecognized: name=value
            let name = &raw.name.to_string();
            write!(f, "{name}")?;
            if !raw.values.is_empty() {
                write!(f, "=")?;
                // Format values as comma-separated list (quoted if needed)
                for (i, v) in raw.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    let s = v.value.to_string();
                    write!(f, "{}", quote_if_needed(&s))?;
                }
            }
            Ok(())
        }
    }
}

/// Format a list of values, quoted if needed (direct helper function).
fn format_quoted_list_direct<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    values: &[S],
) -> io::Result<()> {
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        let s = value.to_string();
        write!(f, "\"{s}\"")?;
    }
    Ok(())
}

/// Quote a string if it contains special characters.
///
/// Per RFC 5545, parameter values containing these characters MUST be quoted:
/// - Control characters
/// - DQUOTE (")
/// - Semicolon (;)
/// - Colon (:)
/// - Backslash (\)
/// - Comma (,)
fn quote_if_needed<S: AsRef<str>>(s: S) -> String {
    // Check if string needs quoting
    let needs_quoting = s
        .as_ref()
        .chars()
        .any(|c| c.is_ascii_control() || c == '"' || c == ';' || c == ':' || c == '\\' || c == ',');

    if needs_quoting {
        format!(
            "\"{}\"",
            s.as_ref().replace('\\', r"\\").replace('"', r#"\""#)
        )
    } else {
        s.as_ref().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_if_needed() {
        assert_eq!(quote_if_needed("simple"), "simple");
        assert_eq!(quote_if_needed("with;semicolon"), "\"with;semicolon\"");
        assert_eq!(quote_if_needed("with:colon"), "\"with:colon\"");
        assert_eq!(quote_if_needed("with,comma"), "\"with,comma\"");
        assert_eq!(quote_if_needed("with\\backslash"), r#""with\\backslash""#);
        assert_eq!(quote_if_needed("with\"quote"), r#""with\"quote""#);
    }
}
