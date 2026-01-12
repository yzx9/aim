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
use crate::parameter::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, FreeBusyType, Parameter,
    ParticipationRole, ParticipationStatus, RelationshipType,
};
use crate::string_storage::StringStorage;
use crate::syntax::SyntaxParameter;
use crate::{RecurrenceIdRange, ValueType};

/// Format all parameters to the formatter.
///
/// This formats multiple parameters, each prefixed with a semicolon.
pub fn write_parameters<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    parameters: &[Parameter<S>],
) -> io::Result<()> {
    for param in parameters {
        write_parameter(f, param)?;
    }
    Ok(())
}

/// Format a single parameter (with semicolon prefix).
fn write_parameter<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    param: &Parameter<S>,
) -> io::Result<()> {
    match param {
        Parameter::AlternateText { value, .. } => write_param_altrep(f, value),
        Parameter::CommonName { value, .. } => write_param_cn(f, value),
        Parameter::CalendarUserType { value, .. } => write_param_cutype(f, value),
        Parameter::Delegators { values, .. } => write_param_delegated_from(f, values),
        Parameter::Delegatees { values, .. } => write_param_delegated_to(f, values),
        Parameter::Directory { value, .. } => write_param_dir(f, value),
        Parameter::Encoding { value, .. } => write_param_encoding(f, *value),
        Parameter::FormatType { value, .. } => write_param_fmttype(f, value),
        Parameter::FreeBusyType { value, .. } => write_param_fbtype(f, value),
        Parameter::Language { value, .. } => write_param_language(f, value),
        Parameter::GroupOrListMembership { values, .. } => write_param_member(f, values),
        Parameter::ParticipationStatus { value, .. } => write_param_partstat(f, value),
        Parameter::RecurrenceIdRange { value, .. } => write_param_range(f, *value),
        Parameter::AlarmTriggerRelationship { value, .. } => write_param_related(f, *value),
        Parameter::RelationshipType { value, .. } => write_param_reltype(f, value),
        Parameter::ParticipationRole { value, .. } => write_param_role(f, value),
        Parameter::SendBy { value, .. } => write_param_sent_by(f, value),
        Parameter::RsvpExpectation { value, .. } => write_param_rsvp(f, *value),
        Parameter::TimeZoneIdentifier { value, .. } => write_param_tzid(f, value),
        Parameter::ValueType { value, .. } => write_param_value(f, value),
        Parameter::XName(raw) => write_param_xname(f, raw),
        Parameter::Unrecognized(raw) => write_param_unrecognized(f, raw),
    }
}

// ============================================================================
// Specific Parameter Writer Functions
// ============================================================================
// These functions write individual parameters directly without requiring
// Parameter<S> enum construction, enabling zero-copy formatting.
// Each function writes the parameter with semicolon prefix: ";NAME=value"

/// Write an ALTREP parameter
pub fn write_param_altrep<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_ALTREP}={quoted}")
}

/// Write a CN parameter
pub fn write_param_cn<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_CN}={quoted}")
}

/// Write a CUTYPE parameter
pub fn write_param_cutype<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &CalendarUserType<S>,
) -> io::Result<()> {
    write!(f, ";{KW_CUTYPE}={value}")
}

/// Write DELEGATED-FROM parameter (multi-value, quoted)
pub fn write_param_delegated_from<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    values: &[S],
) -> io::Result<()> {
    write!(f, ";{KW_DELEGATED_FROM}=")?;
    format_quoted_list(f, values)
}

/// Write DELEGATED-TO parameter (multi-value, quoted)
pub fn write_param_delegated_to<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    values: &[S],
) -> io::Result<()> {
    write!(f, ";{KW_DELEGATED_TO}=")?;
    format_quoted_list(f, values)
}

/// Write a DIR parameter
pub fn write_param_dir<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_DIR}={quoted}")
}

/// Write an ENCODING parameter
pub fn write_param_encoding<W: Write>(f: &mut Formatter<W>, value: Encoding) -> io::Result<()> {
    write!(f, ";{KW_ENCODING}={value}")
}

/// Write an FMTTYPE parameter
pub fn write_param_fmttype<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_FMTTYPE}={quoted}")
}

/// Write an FBTYPE parameter
pub fn write_param_fbtype<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &FreeBusyType<S>,
) -> io::Result<()> {
    write!(f, ";{KW_FBTYPE}={value}")
}

/// Write a LANGUAGE parameter
pub fn write_param_language<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_LANGUAGE}={quoted}")
}

/// Write a MEMBER parameter (multi-value, quoted)
pub fn write_param_member<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    values: &[S],
) -> io::Result<()> {
    write!(f, ";{KW_MEMBER}=")?;
    format_quoted_list(f, values)
}

/// Write a PARTSTAT parameter
pub fn write_param_partstat<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &ParticipationStatus<S>,
) -> io::Result<()> {
    write!(f, ";{KW_PARTSTAT}={value}")
}

/// Write a RANGE parameter
pub fn write_param_range<W: Write>(
    f: &mut Formatter<W>,
    value: RecurrenceIdRange,
) -> io::Result<()> {
    write!(f, ";{KW_RANGE}={value}")
}

/// Write a RELATED parameter
pub fn write_param_related<W: Write>(
    f: &mut Formatter<W>,
    value: AlarmTriggerRelationship,
) -> io::Result<()> {
    write!(f, ";{KW_RELATED}={value}")
}

/// Write a RELTYPE parameter
pub fn write_param_reltype<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &RelationshipType<S>,
) -> io::Result<()> {
    write!(f, ";{KW_RELTYPE}={value}")
}

/// Write a ROLE parameter
pub fn write_param_role<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &ParticipationRole<S>,
) -> io::Result<()> {
    write!(f, ";{KW_ROLE}={value}")
}

/// Write a SENT-BY parameter
pub fn write_param_sent_by<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_SENT_BY}={quoted}")
}

/// Write an RSVP parameter
pub fn write_param_rsvp<W: Write>(f: &mut Formatter<W>, value: bool) -> io::Result<()> {
    let v = if value { KW_RSVP_TRUE } else { KW_RSVP_FALSE };
    write!(f, ";{KW_RSVP}={v}")
}

/// Write a TZID parameter
pub fn write_param_tzid<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &S,
) -> io::Result<()> {
    let quoted = quote_if_needed(value.to_string());
    write!(f, ";{KW_TZID}={quoted}")
}

/// Write a VALUE parameter
pub fn write_param_value<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &ValueType<S>,
) -> io::Result<()> {
    write!(f, ";{KW_VALUE}={value}")
}

/// Write an X-NAME parameter
pub fn write_param_xname<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    syntax: &SyntaxParameter<S>,
) -> io::Result<()> {
    write_param_syntax(f, syntax)
}

/// Write an unrecognized parameter
pub fn write_param_unrecognized<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    raw: &SyntaxParameter<S>,
) -> io::Result<()> {
    write_param_syntax(f, raw)
}

fn write_param_syntax<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    raw: &SyntaxParameter<S>,
) -> io::Result<()> {
    // Unrecognized: name=value
    write!(f, ";{}", raw.name)?;
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
        let s = s.as_ref().replace('\\', r"\\").replace('"', r#"\""#);
        format!("\"{s}\"")
    } else {
        s.as_ref().to_string()
    }
}

/// Format a quoted list for multi-value parameters (MEMBER, DELEGATED-TO, DELEGATED-FROM)
fn format_quoted_list<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    values: &[S],
) -> io::Result<()> {
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "\"{value}\"")?;
    }
    Ok(())
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
