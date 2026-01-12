// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property formatting for iCalendar properties.
//!
//! This module provides functions to format all iCalendar property types
//! as defined in RFC 5545 Section 3.8.

use std::fmt::Display;
use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::formatter::parameter::{
    write_param_altrep, write_param_cn, write_param_cutype, write_param_delegated_from,
    write_param_delegated_to, write_param_dir, write_param_encoding, write_param_fbtype,
    write_param_fmttype, write_param_language, write_param_member, write_param_partstat,
    write_param_related, write_param_reltype, write_param_role, write_param_rsvp,
    write_param_sent_by, write_param_tzid, write_parameters,
};
use crate::formatter::value::{write_date, write_duration, write_recurrence_rule, write_value};
use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS, KW_SUMMARY,
    KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::parameter::Parameter;
use crate::property::{
    Attachment, AttachmentValue, Attendee, Categories, DateTime, Duration, ExDate, ExDateValue,
    Geo, Organizer, Period, Property, RDate, RDateValue, RRule, RelatedTo, Resources, Time,
    Trigger, TriggerValue, UriProperty,
};
use crate::string_storage::StringStorage;

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
        Property::Duration(prop) => write_prop_duration(f, prop),

        // URI properties
        Property::Url(prop) => write_uri_prop(f, KW_URL, prop),
        Property::TzUrl(prop) => write_uri_prop(f, KW_TZURL, prop),

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
            write_value(f, value)
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
        Property::ExDate(prop) => write_multi_date_prop(f, KW_EXDATE, prop),
        Property::RDate(prop) => write_rdate_prop(f, prop),

        // Relationship properties
        Property::Attendee(prop) => write_cal_address_prop(f, KW_ATTENDEE, prop),
        Property::Organizer(prop) => write_organizer_prop(f, prop),

        // Attachment property
        Property::Attach(prop) => write_attach_prop(f, prop),

        // Text/enum properties
        Property::Status(prop) => write_text_with_language(
            f,
            KW_STATUS,
            &prop.value,
            None,
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::Categories(prop) => write_categories_prop(f, prop),
        Property::Resources(prop) => write_resources_prop(f, prop),
        Property::Contact(prop) => write_text_with_language(
            f,
            KW_CONTACT,
            &prop.inner.content,
            prop.inner.language.as_ref(),
            &prop.x_parameters,
            &prop.unrecognized_parameters,
        ),
        Property::RelatedTo(prop) => write_related_to_prop(f, prop),

        // Additional DateTime properties
        Property::Created(prop) => write_datetime_prop(f, KW_CREATED, &prop.inner),
        Property::LastModified(prop) => write_datetime_prop(f, KW_LAST_MODIFIED, &prop.inner),
        Property::Completed(prop) => write_datetime_prop(f, KW_COMPLETED, &prop.inner),
        Property::Due(prop) => write_datetime_prop(f, KW_DUE, &prop.inner),
        Property::RecurrenceId(prop) => write_datetime_prop(f, KW_RECURRENCE_ID, &prop.inner),

        // Other properties
        Property::Geo(prop) => write_geo_prop(f, prop),
        Property::FreeBusy(prop) => write_freebusy_prop(f, prop),
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
            write_prop_trigger(f, prop, &prop.x_parameters, &prop.unrecognized_parameters)
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
    content: &impl Display,
    language: Option<&S>,
    altrep: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = language {
        write_param_language(f, lang)?;
    }

    // Write ALTREP parameter if present
    if let Some(uri) = altrep {
        write_param_altrep(f, uri)?;
    }

    // Write generic parameter lists
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    // Write property value
    write!(f, ":{content}")
}

/// Write a text property with LANGUAGE parameter only (`TextWithLanguage` type).
///
/// This handles properties like `Comment`, `TzName` that only support LANGUAGE.
fn write_text_with_language<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    content: &impl Display,
    language: Option<&S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = language {
        write_param_language(f, lang)?;
    }

    // Write generic parameter lists
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    // Write property value
    write!(f, ":{content}")
}

/// Write a URI property (`Url` or `TzUrl`).
fn write_uri_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    prop: &UriProperty<S>,
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.unrecognized_parameters)?;
    write!(f, ":{}", prop.uri)
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
    write!(f, ":")?;
    write_datetime(f, datetime)
}

/// Write a Duration property.
fn write_prop_duration<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    duration: &Duration<S>,
) -> io::Result<()> {
    // Write: DURATION;params:value
    write!(f, "{KW_DURATION}")?;
    write_parameters(f, &duration.x_parameters)?;
    write_parameters(f, &duration.unrecognized_parameters)?;
    write!(f, ":")?;
    write_duration(f, &duration.value)
}

/// Write an integer property.
fn write_integer_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    value: impl Display,
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
    write_recurrence_rule(f, &rrule.value)
}

/// Write a multi-valued date property (`ExDate`).
fn write_multi_date_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    exdate: &ExDate<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write TZID parameter if present
    if let Some(tz) = &exdate.tz_id {
        write_param_tzid(f, tz)?;
    }

    // Write generic parameter lists
    write_parameters(f, &exdate.x_parameters)?;
    write_parameters(f, &exdate.unrecognized_parameters)?;

    // Write property value
    write!(f, ":")?;

    for (i, date) in exdate.dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            ExDateValue::Date(d) => write_date(f, *d)?,
            ExDateValue::DateTime(dt) => write_datetime(f, dt)?,
        }
    }
    Ok(())
}

/// Write an `RDate` property (can contain DATE, DATE-TIME, or PERIOD values).
fn write_rdate_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    rdate: &RDate<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_RDATE}")?;

    // Write TZID parameter if present
    if let Some(tz) = &rdate.tz_id {
        write_param_tzid(f, tz)?;
    }

    // Write generic parameter lists
    write_parameters(f, &rdate.x_parameters)?;
    write_parameters(f, &rdate.unrecognized_parameters)?;

    // Write property value
    write!(f, ":")?;

    for (i, date) in rdate.dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            RDateValue::Date(d) => write_date(f, *d)?,
            RDateValue::DateTime(dt) => write_datetime(f, dt)?,
            RDateValue::Period(p) => write_period(f, p)?,
        }
    }
    Ok(())
}

/// Write a cal-address property (Attendee).
fn write_cal_address_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    name: &str,
    attendee: &Attendee<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write all parameters using centralized formatters
    if let Some(v) = &attendee.cn {
        write_param_cn(f, v)?;
    }
    write_param_cutype(f, &attendee.cutype)?;
    write_param_role(f, &attendee.role)?;
    write_param_partstat(f, &attendee.part_stat)?;
    if let Some(v) = attendee.rsvp {
        write_param_rsvp(f, v)?;
    }
    if let Some(values) = attendee.member.as_deref() {
        write_param_member(f, values)?;
    }
    if let Some(values) = attendee.delegated_to.as_deref() {
        write_param_delegated_to(f, values)?;
    }
    if let Some(values) = attendee.delegated_from.as_deref() {
        write_param_delegated_from(f, values)?;
    }
    if let Some(v) = &attendee.dir {
        write_param_dir(f, v)?;
    }
    if let Some(v) = &attendee.sent_by {
        write_param_sent_by(f, v)?;
    }
    if let Some(v) = &attendee.language {
        write_param_language(f, v)?;
    }

    // Write generic parameter lists
    write_parameters(f, &attendee.x_parameters)?;
    write_parameters(f, &attendee.unrecognized_parameters)?;

    // Write property value
    write!(f, ":{}", attendee.cal_address)
}

/// Write an Organizer property.
fn write_organizer_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    organizer: &Organizer<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_ORGANIZER}")?;

    // Write all parameters using centralized formatters
    if let Some(v) = &organizer.cn {
        write_param_cn(f, v)?;
    }
    if let Some(v) = &organizer.dir {
        write_param_dir(f, v)?;
    }
    if let Some(v) = &organizer.sent_by {
        write_param_sent_by(f, v)?;
    }
    if let Some(v) = &organizer.language {
        write_param_language(f, v)?;
    }

    // Write generic parameter lists
    write_parameters(f, &organizer.x_parameters)?;
    write_parameters(f, &organizer.unrecognized_parameters)?;

    // Write property value
    write!(f, ":{}", organizer.cal_address)
}

/// Write an Attach property.
fn write_attach_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    attach: &Attachment<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_ATTACH}")?;

    // Write parameters using centralized formatters
    if let Some(v) = &attach.fmt_type {
        write_param_fmttype(f, v)?;
    }
    if let Some(v) = attach.encoding {
        write_param_encoding(f, v)?;
    }

    // Write generic parameter lists
    write_parameters(f, &attach.x_parameters)?;
    write_parameters(f, &attach.unrecognized_parameters)?;

    // Write property value
    write!(f, ":")?;
    match &attach.value {
        AttachmentValue::Uri(uri) => write!(f, "{uri}")?,
        AttachmentValue::Binary(data) => write!(f, "{data}")?,
    }
    Ok(())
}

/// Write a Categories property.
fn write_categories_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    categories: &Categories<S>,
) -> io::Result<()> {
    write!(f, "{KW_CATEGORIES}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = &categories.language {
        write_param_language(f, lang)?;
    }

    write_parameters(f, &categories.x_parameters)?;
    write_parameters(f, &categories.unrecognized_parameters)?;

    write!(f, ":")?;
    for (i, value) in categories.values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

/// Write a Resources property.
fn write_resources_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    resources: &Resources<S>,
) -> io::Result<()> {
    write!(f, "{KW_RESOURCES}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = &resources.language {
        write_param_language(f, lang)?;
    }

    // Write ALTREP parameter if present
    if let Some(uri) = &resources.altrep {
        write_param_altrep(f, uri)?;
    }

    write_parameters(f, &resources.x_parameters)?;
    write_parameters(f, &resources.unrecognized_parameters)?;

    write!(f, ":")?;
    for (i, value) in resources.values.iter().enumerate() {
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
    related_to: &RelatedTo<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_RELATED_TO}")?;

    // Write RELTYPE parameter using centralized formatter
    write_param_reltype(f, &related_to.reltype)?;

    // Write generic parameter lists
    write_parameters(f, &related_to.x_parameters)?;
    write_parameters(f, &related_to.unrecognized_parameters)?;

    // Write property value
    write!(f, ":{}", related_to.content)
}

/// Write a `Geo` property.
fn write_geo_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    geo: &Geo<S>,
) -> io::Result<()> {
    write!(f, "{KW_GEO}")?;
    write_parameters(f, &geo.x_parameters)?;
    write_parameters(f, &geo.unrecognized_parameters)?;
    write!(f, ":{};{}", geo.lat, geo.lon)
}

/// Write a `FreeBusy` property.
fn write_freebusy_prop<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    freebusy: &crate::FreeBusy<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_FREEBUSY}")?;

    // Write FBTYPE parameter using centralized formatter
    write_param_fbtype(f, &freebusy.fb_type)?;

    // Write generic parameter lists
    write_parameters(f, &freebusy.x_parameters)?;
    write_parameters(f, &freebusy.unrecognized_parameters)?;

    // Write property value
    write!(f, ":")?;
    for (i, period) in freebusy.values.iter().enumerate() {
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
            write_date(f, *start_date)?;
            write!(f, "T")?;
            write_time(f, start_time, true)?;
            write!(f, "/")?;
            write_date(f, *end_date)?;
            write!(f, "T")?;
            write_time(f, end_time, true)?;
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
            write_date(f, *start_date)?;
            write!(f, "T")?;
            write_time(f, start_time, false)?;
            write!(f, "/")?;
            write_date(f, *end_date)?;
            write!(f, "T")?;
            write_time(f, end_time, false)?;
        }
        Period::DurationUtc {
            start_date,
            start_time,
            duration,
        } => {
            write_date(f, *start_date)?;
            write!(f, "T")?;
            write_time(f, start_time, true)?;
            write!(f, "/")?;
            write_duration(f, duration)?;
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
            write_date(f, *start_date)?;
            write!(f, "T")?;
            write_time(f, start_time, false)?;
            write!(f, "/")?;
            write_duration(f, duration)?;
        }
    }
    Ok(())
}

/// Write a Trigger property.
fn write_prop_trigger<S: StringStorage, W: Write>(
    f: &mut Formatter<W>,
    trigger: &Trigger<S>,
    x_params: &[Parameter<S>],
    unrecognized_params: &[Parameter<S>],
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_TRIGGER}")?;

    // Write RELATED parameter if present, using centralized formatter
    if let Some(related) = &trigger.related {
        write_param_related(f, *related)?;
    }

    // Write generic parameter lists
    write_parameters(f, x_params)?;
    write_parameters(f, unrecognized_params)?;

    // Write property value
    write!(f, ":")?;
    match &trigger.value {
        TriggerValue::Duration(d) => write_duration(f, d),
        TriggerValue::DateTime(dt) => write_datetime(f, dt),
    }
}

fn write_datetime<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    datetime: &DateTime<S>,
) -> io::Result<()> {
    match datetime {
        DateTime::Floating { date, time, .. }
            // NOTE: TZID parameter should already be in the params list
           | DateTime::Zoned { date, time, .. } => {
            write_date(f, *date)?;
            write!(f, "T")?;
            write_time(f, time, false)
        }
        DateTime::Utc { date, time, .. } => {
            write_date(f, *date)?;
            write!(f, "T")?;
            write_time(f, time, true)
        }
        DateTime::Date { date, .. } => write_date(f, *date),
    }
}

/// Format a `property::datetime::Time` value as `HHMMSS[Z]`.
fn write_time<W: Write>(f: &mut Formatter<W>, time: &Time, utc: bool) -> io::Result<()> {
    let utc = if utc { "Z" } else { "" };
    write!(
        f,
        "{:02}{:02}{:02}{}",
        time.hour, time.minute, time.second, utc
    )
}
