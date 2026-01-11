// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property formatting for iCalendar properties.
//!
//! This module provides functions to format all iCalendar property types
//! as defined in RFC 5545 Section 3.8.

use std::fmt;
use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::formatter::parameter::write_parameters;
use crate::formatter::value::{format_date, format_duration, format_recurrence_rule, format_value};
use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FBTYPE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION,
    KW_METHOD, KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE,
    KW_RECURRENCE_ID, KW_RELATED_TO, KW_REPEAT, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::parameter::{
    CalendarUserType, Encoding, FreeBusyType, Parameter, ParticipationRole, ParticipationStatus,
    RelationshipType,
};
use crate::property::{
    AttachmentValue, DateTime, Duration, ExDateValue, Period, Property, RDateValue, RRule, Time,
    Trigger, TriggerValue,
};
use crate::string_storage::StringStorage;
use crate::value::ValueText;

/// Format a single property.
///
/// This is the main entry point for formatting properties.
#[expect(clippy::too_many_lines)]
pub fn write_property<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    property: &Property<S>,
) -> io::Result<()> {
    match property {
        // Properties with direct text values (simple enum wrappers)
        Property::CalScale(prop) => write!(f, "{KW_CALSCALE}:{}", prop.value),
        Property::Method(prop) => write!(f, "{KW_METHOD}:{}", prop.value),
        Property::ProdId(prop) => write!(f, "{KW_PRODID}:{}", prop.value),
        Property::Version(prop) => write!(f, "{KW_VERSION}:{}", prop.value),
        Property::Class(prop) => write!(f, "{KW_CLASS}:{}", prop.value),
        Property::TzId(prop) => write!(f, "{KW_TZID}:{}", prop.inner.content),
        Property::Uid(prop) => write!(f, "{KW_UID}:{}", prop.inner.content),

        // Properties with Text inner type (has parameters)
        Property::Summary(prop) => write_text_with_params(
            f,
            KW_SUMMARY,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            prop.inner.altrep.as_ref(),
            &prop.inner.x_parameters,
            &prop.inner.unrecognized_parameters,
        ),
        Property::Description(prop) => write_text_with_params(
            f,
            KW_DESCRIPTION,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            prop.inner.altrep.as_ref(),
            &prop.inner.x_parameters,
            &prop.inner.unrecognized_parameters,
        ),
        Property::Location(prop) => write_text_with_params(
            f,
            KW_LOCATION,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            prop.inner.altrep.as_ref(),
            &prop.inner.x_parameters,
            &prop.inner.unrecognized_parameters,
        ),
        Property::Comment(prop) => write_text_with_language(
            f,
            KW_COMMENT,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            &prop.inner.x_parameters,
            &prop.inner.unrecognized_parameters,
        ),
        Property::TzName(prop) => write_text_with_language(
            f,
            KW_TZNAME,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            &prop.inner.x_parameters,
            &prop.inner.unrecognized_parameters,
        ),

        // DateTime properties
        Property::DtStart(prop) => write_datetime_prop(f, KW_DTSTART, &prop.inner),
        Property::DtEnd(prop) => write_datetime_prop(f, KW_DTEND, &prop.inner),
        Property::DtStamp(prop) => write_datetime_prop(f, KW_DTSTAMP, &prop.inner),
        Property::Duration(prop) => write_duration_prop(f, prop),

        // URI properties
        Property::Url(prop) => write_uri_prop(
            f,
            KW_URL,
            &prop.uri,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::TzUrl(prop) => write_uri_prop(
            f,
            KW_TZURL,
            &prop.uri,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // XName and Unrecognized properties - use their value directly
        Property::XName {
            name,
            parameters,
            value,
            ..
        }
        | Property::Unrecognized {
            name,
            parameters,
            value,
            ..
        } => {
            write!(f, "{name}")?;
            write_parameters(f, parameters)?;
            write!(f, ":")?;
            format_value(f, value)
        }

        // Integer properties
        Property::Priority(prop) => write_integer_prop(
            f,
            KW_PRIORITY,
            prop.value,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Sequence(prop) => write_integer_prop(
            f,
            KW_SEQUENCE,
            prop.value,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::PercentComplete(prop) => write_integer_prop(
            f,
            KW_PERCENT_COMPLETE,
            prop.value,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Repeat(prop) => write_integer_prop(
            f,
            KW_REPEAT,
            prop.value,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Recurrence properties
        Property::RRule(prop) => write_rrule_prop(f, prop),
        Property::ExDate(prop) => write_multi_date_prop(
            f,
            KW_EXDATE,
            &prop.dates,
            prop.tz_id.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::RDate(prop) => write_rdate_prop(
            f,
            &prop.dates,
            prop.tz_id.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Relationship properties
        Property::Attendee(prop) => write_cal_address_prop(
            f,
            KW_ATTENDEE,
            &prop.cal_address,
            prop.cn.as_ref(),
            &prop.role,
            &prop.part_stat,
            prop.rsvp,
            &prop.cutype,
            prop.member.as_deref(),
            prop.delegated_to.as_deref(),
            prop.delegated_from.as_deref(),
            prop.dir.as_ref(),
            prop.sent_by.as_ref(),
            prop.language.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Organizer(prop) => write_organizer_prop(
            f,
            &prop.cal_address,
            prop.cn.as_ref(),
            prop.dir.as_ref(),
            prop.sent_by.as_ref(),
            prop.language.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Attachment property
        Property::Attach(prop) => write_attach_prop(
            f,
            &prop.value,
            prop.fmt_type.as_ref(),
            prop.encoding.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Text/enum properties
        Property::Status(prop) => write_text_with_language(
            f,
            KW_STATUS,
            &prop.value,
            None,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Categories(prop) => write_multi_text_prop(
            f,
            KW_CATEGORIES,
            &prop.values,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Resources(prop) => write_multi_text_prop(
            f,
            KW_RESOURCES,
            &prop.values,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Contact(prop) => write_text_with_language(
            f,
            KW_CONTACT,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::RelatedTo(prop) => write_related_to_prop(
            f,
            &prop.content,
            &prop.reltype,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Additional DateTime properties
        Property::Created(prop) => write_datetime_prop(f, KW_CREATED, &prop.inner),
        Property::LastModified(prop) => write_datetime_prop(f, KW_LAST_MODIFIED, &prop.inner),
        Property::Completed(prop) => write_datetime_prop(f, KW_COMPLETED, &prop.inner),
        Property::Due(prop) => write_datetime_prop(f, KW_DUE, &prop.inner),
        Property::RecurrenceId(prop) => write_datetime_prop(f, KW_RECURRENCE_ID, &prop.inner),

        // Other properties
        Property::Geo(prop) => write_geo_prop(
            f,
            prop.lat,
            prop.lon,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::FreeBusy(prop) => write_freebusy_prop(
            f,
            &prop.values,
            &prop.fb_type,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Transp(prop) => write_text_with_language(
            f,
            KW_TRANSP,
            &prop.value,
            None,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),

        // Alarm properties
        Property::Action(prop) => write_text_with_language(
            f,
            KW_ACTION,
            &prop.value,
            None,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Trigger(prop) => {
            write_trigger_prop(f, prop, &prop.x_parameters, &prop.unrecognized_parameters)
        }

        _ => {
            // TODO: Implement proper formatting for all property types
            let name = property.kind().to_string();
            todo!("{name}:IMPLEMENTED");
        }
    }?;
    f.writeln()
}

/// Write a text property with parameters (Text type).
///
/// This handles properties like Summary, Description, Location, etc.
fn write_text_with_params<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    content: &impl fmt::Display,
    language: Option<&S>,
    altrep: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    // Add LANGUAGE parameter if present
    if let Some(lang) = language {
        // Need to format LANGUAGE parameter directly
        write!(f, "{name};LANGUAGE={lang}")?;
        write_parameters(f, x_params)?;
        write_parameters(f, unrecognized_params)?;
        write!(f, ":{content}")?;
        return Ok(());
    }

    // Add ALTREP parameter if present
    if let Some(uri) = altrep {
        write!(f, "{name};ALTREP={uri}")?;
        write_parameters(f, x_params)?;
        write_parameters(f, unrecognized_params)?;
        write!(f, ":{content}")?;
        return Ok(());
    }

    // Write: NAME;params:value
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{content}")
}

/// Write a text property with LANGUAGE parameter only (`TextWithLanguage` type).
///
/// This handles properties like `Comment`, `TzName` that only support LANGUAGE.
fn write_text_with_language<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    content: &impl fmt::Display,
    language: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    // Add LANGUAGE parameter if present
    if let Some(lang) = language {
        write!(f, "{name};LANGUAGE={lang}")?;
        write_parameters(f, x_params)?;
        write_parameters(f, unrecognized_params)?;
        write!(f, ":{content}")?;
        return Ok(());
    }

    // Write: NAME;params:value
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{content}")
}

/// Write a URI property.
fn write_uri_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    uri: &impl fmt::Display,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{uri}")
}

/// Write a `DateTime` property.
fn write_datetime_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    datetime: &DateTime<S>,
) -> io::Result<()> {
    // Collect all parameters using a helper
    let (x_params, unrecognized_params) = match datetime {
        DateTime::Floating {
            x_parameters,
            unrecognized_parameters,
            ..
        }
        | DateTime::Zoned {
            x_parameters,
            unrecognized_parameters,
            ..
        }
        | DateTime::Utc {
            x_parameters,
            unrecognized_parameters,
            ..
        }
        | DateTime::Date {
            x_parameters,
            unrecognized_parameters,
            ..
        } => (x_parameters, unrecognized_parameters),
    };

    // Write: NAME;params:value
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    // Write the value
    match datetime {
        DateTime::Floating { date, time, .. } => {
            write!(f, ":")?;
            format_date(f, *date)?;
            write!(f, "T")?;
            write_property_time(f, time, false)?;
        }
        DateTime::Zoned { date, time, .. } => {
            write!(f, ":")?;
            format_date(f, *date)?;
            write!(f, "T")?;
            write_property_time(f, time, false)?;
            // Note: TZID parameter should already be in the params list
        }
        DateTime::Utc { date, time, .. } => {
            write!(f, ":")?;
            format_date(f, *date)?;
            write!(f, "T")?;
            write_property_time(f, time, true)?;
        }
        DateTime::Date { date, .. } => {
            write!(f, ":")?;
            format_date(f, *date)?;
        }
    }
    Ok(())
}

/// Write a Duration property.
fn write_duration_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    duration: &Duration<S>,
) -> io::Result<()> {
    // Write: DURATION;params:value
    write!(f, "{KW_DURATION}")?;
    write_parameters(f, &duration.x_parameters)?;
    write_parameters(f, &duration.unrecognized_parameters)?;
    write!(f, ":")?;
    format_duration(f, &duration.value)
}

/// Write an integer property.
fn write_integer_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    value: impl fmt::Display,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{value}")
}

/// Write an `RRule` property.
fn write_rrule_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    rrule: &RRule<S>,
) -> io::Result<()> {
    write!(f, "{KW_RRULE}")?;
    write_parameters(f, &rrule.x_parameters)?;
    write_parameters(f, &rrule.unrecognized_parameters)?;
    write!(f, ":")?;
    format_recurrence_rule(f, &rrule.value)
}

/// Write a multi-valued date property (ExDate/RDate).
fn write_multi_date_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    dates: &[ExDateValue<S>],
    tz_id: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;

    // Add TZID parameter if present
    if let Some(tz) = tz_id {
        write!(f, ";TZID={tz}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":")?;

    for (i, date) in dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            ExDateValue::Date(d) => format_date(f, *d)?,
            ExDateValue::DateTime(dt) => match dt {
                DateTime::Floating { date, time, .. } | DateTime::Zoned { date, time, .. } => {
                    format_date(f, *date)?;
                    write!(f, "T")?;
                    write_property_time(f, time, false)?;
                }
                DateTime::Utc { date, time, .. } => {
                    format_date(f, *date)?;
                    write!(f, "T")?;
                    write_property_time(f, time, true)?;
                }
                DateTime::Date { date, .. } => {
                    format_date(f, *date)?;
                }
            },
        }
    }
    Ok(())
}

/// Write an `RDate` property (can contain DATE, DATE-TIME, or PERIOD values).
fn write_rdate_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    dates: &[RDateValue<S>],
    tz_id: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_RDATE}")?;

    // Add TZID parameter if present
    if let Some(tz) = tz_id {
        write!(f, ";TZID={tz}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":")?;

    for (i, date) in dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            RDateValue::Date(d) => format_date(f, *d)?,
            RDateValue::DateTime(dt) => match dt {
                DateTime::Floating { date, time, .. } | DateTime::Zoned { date, time, .. } => {
                    format_date(f, *date)?;
                    write!(f, "T")?;
                    write_property_time(f, time, false)?;
                }
                DateTime::Utc { date, time, .. } => {
                    format_date(f, *date)?;
                    write!(f, "T")?;
                    write_property_time(f, time, true)?;
                }
                DateTime::Date { date, .. } => {
                    format_date(f, *date)?;
                }
            },
            RDateValue::Period(p) => write_period(f, p)?,
        }
    }
    Ok(())
}

/// Write a cal-address property (Attendee).
#[expect(clippy::too_many_arguments)]
fn write_cal_address_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    cal_address: &S,
    cn: Option<&S>,
    role: &ParticipationRole<S>,
    part_stat: &ParticipationStatus<S>,
    rsvp: Option<bool>,
    cutype: &CalendarUserType<S>,
    member: Option<&[S]>,
    delegated_to: Option<&[S]>,
    delegated_from: Option<&[S]>,
    dir: Option<&S>,
    sent_by: Option<&S>,
    language: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;

    // Write all parameters
    if let Some(v) = cn {
        write!(f, ";CN={v}")?;
    }
    write!(f, ";CUTYPE={cutype}")?;
    write!(f, ";ROLE={role}")?;
    write!(f, ";PARTSTAT={part_stat}")?;
    if let Some(v) = rsvp {
        write!(f, ";RSVP={}", if v { "TRUE" } else { "FALSE" })?;
    }
    if let Some(values) = member {
        write!(f, ";MEMBER=")?;
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "\"{v}\"")?;
        }
    }
    if let Some(values) = delegated_to {
        write!(f, ";DELEGATED-TO=")?;
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "\"{v}\"")?;
        }
    }
    if let Some(values) = delegated_from {
        write!(f, ";DELEGATED-FROM=")?;
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "\"{v}\"")?;
        }
    }
    if let Some(v) = dir {
        write!(f, ";DIR={v}")?;
    }
    if let Some(v) = sent_by {
        write!(f, ";SENT-BY={v}")?;
    }
    if let Some(v) = language {
        write!(f, ";LANGUAGE={v}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    write!(f, ":{cal_address}")
}

/// Write an Organizer property.
#[expect(clippy::too_many_arguments)]
fn write_organizer_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    cal_address: &S,
    cn: Option<&S>,
    dir: Option<&S>,
    sent_by: Option<&S>,
    language: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_ORGANIZER}")?;

    if let Some(v) = cn {
        write!(f, ";CN={v}")?;
    }
    if let Some(v) = dir {
        write!(f, ";DIR={v}")?;
    }
    if let Some(v) = sent_by {
        write!(f, ";SENT-BY={v}")?;
    }
    if let Some(v) = language {
        write!(f, ";LANGUAGE={v}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{cal_address}")
}

/// Write an Attach property.
fn write_attach_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    value: &AttachmentValue<S>,
    fmt_type: Option<&S>,
    encoding: Option<&Encoding>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_ATTACH}")?;

    if let Some(v) = fmt_type {
        write!(f, ";FMTTYPE={v}")?;
    }
    if let Some(v) = encoding {
        write!(f, ";ENCODING={v}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    write!(f, ":")?;
    match value {
        AttachmentValue::Uri(uri) => write!(f, "{uri}")?,
        AttachmentValue::Binary(data) => write!(f, "{data}")?,
    }
    Ok(())
}

/// Write a multi-valued text property (`Categories`, `Resources`).
fn write_multi_text_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    values: &[ValueText<S>],
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    write!(f, ":")?;
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

/// Write a `RelatedTo` property.
fn write_related_to_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    content: &ValueText<S>,
    reltype: &RelationshipType<S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_RELATED_TO};RELTYPE={reltype}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{content}")
}

/// Write a `Geo` property.
fn write_geo_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    lat: f64,
    lon: f64,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_GEO}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;
    write!(f, ":{lat};{lon}")
}

/// Write a `FreeBusy` property.
fn write_freebusy_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    values: &[Period<S>],
    fb_type: &FreeBusyType<S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_FREEBUSY};{KW_FBTYPE}={fb_type}")?;
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    write!(f, ":")?;
    for (i, period) in values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write_period(f, period)?;
    }
    Ok(())
}

/// Format a property Period value as date-time/date-time or date-time/duration.
fn write_period<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    period: &Period<S>,
) -> io::Result<()> {
    match period {
        Period::ExplicitUtc {
            start_date,
            start_time,
            end_date,
            end_time,
        } => {
            format_date(f, *start_date)?;
            write!(f, "T")?;
            write_property_time(f, start_time, true)?;
            write!(f, "/")?;
            format_date(f, *end_date)?;
            write!(f, "T")?;
            write_property_time(f, end_time, true)?;
        }
        Period::ExplicitFloating {
            start_date,
            start_time,
            end_date,
            end_time,
        }
        | Period::ExplicitZoned {
            start_date,
            start_time,
            end_date,
            end_time,
            ..
        } => {
            format_date(f, *start_date)?;
            write!(f, "T")?;
            write_property_time(f, start_time, false)?;
            write!(f, "/")?;
            format_date(f, *end_date)?;
            write!(f, "T")?;
            write_property_time(f, end_time, false)?;
        }
        Period::DurationUtc {
            start_date,
            start_time,
            duration,
        } => {
            format_date(f, *start_date)?;
            write!(f, "T")?;
            write_property_time(f, start_time, true)?;
            write!(f, "/")?;
            format_duration(f, duration)?;
        }
        Period::DurationFloating {
            start_date,
            start_time,
            duration,
        }
        | Period::DurationZoned {
            start_date,
            start_time,
            duration,
            ..
        } => {
            format_date(f, *start_date)?;
            write!(f, "T")?;
            write_property_time(f, start_time, false)?;
            write!(f, "/")?;
            format_duration(f, duration)?;
        }
    }
    Ok(())
}

/// Write a Trigger property.
fn write_trigger_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    trigger: &Trigger<S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{KW_TRIGGER}")?;

    // Add RELATED parameter if present
    if let Some(related) = &trigger.related {
        write!(f, ";RELATED={related}")?;
    }

    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    write!(f, ":")?;
    match &trigger.value {
        TriggerValue::Duration(d) => {
            format_duration(f, d)?;
        }
        TriggerValue::DateTime(dt) => match dt {
            DateTime::Floating { date, time, .. } | DateTime::Zoned { date, time, .. } => {
                format_date(f, *date)?;
                write!(f, "T")?;
                write_property_time(f, time, false)?;
            }
            DateTime::Utc { date, time, .. } => {
                format_date(f, *date)?;
                write!(f, "T")?;
                write_property_time(f, time, true)?;
            }
            DateTime::Date { date, .. } => format_date(f, *date)?,
        },
    }
    Ok(())
}

/// Format a `property::datetime::Time` value as `HHMMSS[Z]`.
fn write_property_time<W: Write>(f: &mut Formatter<W>, time: &Time, utc: bool) -> io::Result<()> {
    write!(
        f,
        "{:02}{:02}{:02}{}",
        time.hour,
        time.minute,
        time.second,
        if utc { "Z" } else { "" }
    )
}
