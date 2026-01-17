// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property formatting for iCalendar properties.
//!
//! This module provides functions to format all iCalendar property types
//! as defined in RFC 5545 Section 3.8.

use std::fmt::Display;
use std::io::{self, Write};

use crate::StatusValue;
use crate::formatter::Formatter;
use crate::formatter::parameter::{
    write_param_altrep, write_param_cn, write_param_cutype, write_param_delegated_from,
    write_param_delegated_to, write_param_dir, write_param_encoding, write_param_fbtype,
    write_param_fmttype, write_param_language, write_param_member, write_param_partstat,
    write_param_related, write_param_reltype, write_param_role, write_param_rsvp,
    write_param_sent_by, write_param_tzid, write_parameters, write_syntax_parameters,
};
use crate::formatter::value::{
    format_value_text, write_date, write_duration, write_recurrence_rule, write_utc_offset,
    write_value,
};
use crate::keyword::{
    KW_ACTION, KW_ATTACH, KW_ATTENDEE, KW_CALSCALE, KW_CATEGORIES, KW_CLASS, KW_COMMENT,
    KW_COMPLETED, KW_CONTACT, KW_CREATED, KW_DESCRIPTION, KW_DTEND, KW_DTSTAMP, KW_DTSTART, KW_DUE,
    KW_DURATION, KW_EXDATE, KW_FREEBUSY, KW_GEO, KW_LAST_MODIFIED, KW_LOCATION, KW_METHOD,
    KW_ORGANIZER, KW_PERCENT_COMPLETE, KW_PRIORITY, KW_PRODID, KW_RDATE, KW_RECURRENCE_ID,
    KW_RELATED_TO, KW_REPEAT, KW_REQUEST_STATUS, KW_RESOURCES, KW_RRULE, KW_SEQUENCE, KW_STATUS,
    KW_SUMMARY, KW_TRANSP, KW_TRIGGER, KW_TZID, KW_TZNAME, KW_TZOFFSETFROM, KW_TZOFFSETTO,
    KW_TZURL, KW_UID, KW_URL, KW_VERSION,
};
use crate::parameter::Parameter;
use crate::property::{
    Action, Attachment, AttachmentValue, Attendee, CalendarScale, Categories, Classification,
    Comment, Completed, Contact, Created, DateTime, Description, DtEnd, DtStamp, DtStart, Due,
    Duration, ExDate, ExDateValue, FreeBusy, Geo, LastModified, Location, Method, Organizer,
    PercentComplete, Period, Priority, ProductId, Property, RDate, RDateValue, RRule, RecurrenceId,
    RelatedTo, Repeat, RequestStatus, Resources, Sequence, Status, Summary, Time, TimeTransparency,
    Trigger, TriggerValue, TzId, TzName, TzOffsetFrom, TzOffsetTo, TzUrl, Uid,
    UnrecognizedProperty, UriProperty, Url, Version, XNameProperty,
};
use crate::string_storage::StringStorage;
use crate::syntax::RawParameter;
use crate::value::ValueText;

/// Format a single property.
///
/// This is the main entry point for formatting properties.
pub fn write_property<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    property: &Property<S>,
) -> io::Result<()> {
    match property {
        // Properties with direct text values (simple enum wrappers)
        Property::CalScale(prop) => write_prop_calscale(f, prop),
        Property::Method(prop) => write_prop_method(f, prop),
        Property::ProdId(prop) => write_prop_prodid(f, prop),
        Property::Version(prop) => write_prop_version(f, prop),
        Property::Class(prop) => write_prop_class(f, prop),
        Property::TzId(prop) => write_prop_tzid(f, prop),
        Property::Uid(prop) => write_prop_uid(f, prop),

        // Properties with Text inner type (has parameters)
        Property::Summary(prop) => write_prop_summary(f, prop),
        Property::Description(prop) => write_prop_description(f, prop),
        Property::Location(prop) => write_prop_location(f, prop),
        Property::Comment(prop) => write_prop_comment(f, prop),
        Property::TzName(prop) => write_prop_tzname(f, prop),
        Property::TzOffsetFrom(prop) => write_prop_tz_offset_from(f, prop),
        Property::TzOffsetTo(prop) => write_prop_tz_offset_to(f, prop),

        // DateTime properties
        Property::DtStart(prop) => write_prop_dtstart(f, prop),
        Property::DtEnd(prop) => write_prop_dtend(f, prop),
        Property::DtStamp(prop) => write_prop_dtstamp(f, prop),
        Property::Duration(prop) => write_prop_duration(f, prop),

        // URI properties
        Property::Url(prop) => write_prop_url(f, prop),
        Property::TzUrl(prop) => write_prop_tz_url(f, prop),

        // Integer properties
        Property::Priority(prop) => write_prop_priority(f, prop),
        Property::Sequence(prop) => write_prop_sequence(f, prop),
        Property::PercentComplete(prop) => write_prop_percent_complete(f, prop),
        Property::Repeat(prop) => write_prop_repeat(f, prop),

        // Recurrence properties
        Property::RRule(prop) => write_prop_rrule(f, prop),
        Property::ExDate(prop) => write_prop_ex_date(f, prop),
        Property::RDate(prop) => write_prop_rdate(f, prop),

        // Relationship properties
        Property::Attendee(prop) => write_prop_attendee(f, prop),
        Property::Organizer(prop) => write_prop_organizer(f, prop),

        // Attachment property
        Property::Attach(prop) => write_prop_attach(f, prop),

        // Text/enum properties
        Property::Status(prop) => write_prop_status(f, prop),
        Property::Categories(prop) => write_prop_categories(f, prop),
        Property::Resources(prop) => write_prop_resources(f, prop),
        Property::Contact(prop) => write_prop_contact(f, prop),
        Property::RelatedTo(prop) => write_prop_related_to(f, prop),

        // Additional DateTime properties
        Property::Created(prop) => write_prop_created(f, prop),
        Property::LastModified(prop) => write_prop_last_modified(f, prop),
        Property::Completed(prop) => write_prop_completed(f, prop),
        Property::Due(prop) => write_prop_due(f, prop),
        Property::RecurrenceId(prop) => write_prop_recurrence_id(f, prop),

        // Other properties
        Property::Geo(prop) => write_prop_geo(f, prop),
        Property::FreeBusy(prop) => write_prop_freebusy(f, prop),
        Property::Transp(prop) => write_prop_transp(f, prop),

        // Alarm properties
        Property::Action(prop) => write_prop_action(f, prop),
        Property::Trigger(prop) => write_prop_trigger(f, prop),

        // Miscellaneous properties
        Property::RequestStatus(prop) => write_prop_request_status(f, prop),

        // XName and Unrecognized properties - use their value directly
        Property::XName(prop) => write_prop_xname(f, prop),
        Property::Unrecognized(prop) => write_prop_unrecognized(f, prop),
    }
}

// ============================================================================
// Public property writers (for use in component.rs)
// These functions only take f and prop, avoiding unnecessary clones
// ============================================================================

/// Write an `RRule` property.
pub fn write_prop_rrule<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    rrule: &RRule<S>,
) -> io::Result<()> {
    write!(f, "{KW_RRULE}")?;
    write_syntax_parameters(f, &rrule.x_parameters)?;
    write_parameters(f, &rrule.retained_parameters)?;
    write!(f, ":")?;
    write_recurrence_rule(f, &rrule.value)?;
    f.writeln()
}

/// Write a Duration property.
pub fn write_prop_duration<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    duration: &Duration<S>,
) -> io::Result<()> {
    // Write: DURATION;params:value
    write!(f, "{KW_DURATION}")?;
    write_syntax_parameters(f, &duration.x_parameters)?;
    write_parameters(f, &duration.retained_parameters)?;
    write!(f, ":")?;
    write_duration(f, &duration.value)?;
    f.writeln()
}

/// Write an `RDate` property (can contain DATE, DATE-TIME, or PERIOD values).
pub fn write_prop_rdate<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    rdate: &RDate<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_RDATE}")?;

    // Write TZID parameter if present
    if let Some(tz) = &rdate.tz_id {
        write_param_tzid(f, tz)?;
    }

    // Write generic parameter lists
    write_syntax_parameters(f, &rdate.x_parameters)?;
    write_parameters(f, &rdate.retained_parameters)?;

    // Write property value
    write!(f, ":")?;

    for (i, date) in rdate.dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            RDateValue::Date(d) => write_date(f, *d)?,
            RDateValue::DateTime(dt) => write_datetime_value(f, dt)?,
            RDateValue::Period(p) => write_period(f, p)?,
        }
    }
    f.writeln()
}

/// Write an Organizer property.
pub fn write_prop_organizer<S: StringStorage>(
    f: &mut Formatter<impl Write>,
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
    write_syntax_parameters(f, &organizer.x_parameters)?;
    write_parameters(f, &organizer.retained_parameters)?;

    // Write property value
    write!(f, ":{}", organizer.cal_address)?;
    f.writeln()
}

/// Write an Attach property.
pub fn write_prop_attach<S: StringStorage>(
    f: &mut Formatter<impl Write>,
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
    write_syntax_parameters(f, &attach.x_parameters)?;
    write_parameters(f, &attach.retained_parameters)?;

    // Write property value
    write!(f, ":")?;
    match &attach.value {
        AttachmentValue::Uri(uri) => write!(f, "{uri}")?,
        AttachmentValue::Binary(data) => write!(f, "{data}")?,
    }
    f.writeln()
}

/// Write a Categories property.
pub fn write_prop_categories<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    categories: &Categories<S>,
) -> io::Result<()> {
    write!(f, "{KW_CATEGORIES}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = &categories.language {
        write_param_language(f, lang)?;
    }

    write_syntax_parameters(f, &categories.x_parameters)?;
    write_parameters(f, &categories.retained_parameters)?;

    write!(f, ":")?;
    for (i, value) in categories.values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "{value}")?;
    }
    f.writeln()
}

/// Write a Resources property.
pub fn write_prop_resources<S: StringStorage>(
    f: &mut Formatter<impl Write>,
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

    write_syntax_parameters(f, &resources.x_parameters)?;
    write_parameters(f, &resources.retained_parameters)?;

    write!(f, ":")?;
    for (i, value) in resources.values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write!(f, "{value}")?;
    }
    f.writeln()
}

/// Write a `RelatedTo` property.
pub fn write_prop_related_to<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    related_to: &RelatedTo<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_RELATED_TO}")?;

    // Write RELTYPE parameter using centralized formatter
    write_param_reltype(f, &related_to.reltype)?;

    // Write generic parameter lists
    write_syntax_parameters(f, &related_to.x_parameters)?;
    write_parameters(f, &related_to.retained_parameters)?;

    // Write property value
    write!(f, ":{}", related_to.content)?;
    f.writeln()
}

/// Write a `Geo` property.
pub fn write_prop_geo<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    geo: &Geo<S>,
) -> io::Result<()> {
    write!(f, "{KW_GEO}")?;
    write_syntax_parameters(f, &geo.x_parameters)?;
    write_parameters(f, &geo.retained_parameters)?;
    write!(f, ":{};{}", geo.lat, geo.lon)?;
    f.writeln()
}

/// Write a `FreeBusy` property.
pub fn write_prop_freebusy<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    freebusy: &FreeBusy<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_FREEBUSY}")?;

    // Write FBTYPE parameter using centralized formatter
    write_param_fbtype(f, &freebusy.fb_type)?;

    // Write generic parameter lists
    write_syntax_parameters(f, &freebusy.x_parameters)?;
    write_parameters(f, &freebusy.retained_parameters)?;

    // Write property value
    write!(f, ":")?;
    for (i, period) in freebusy.values.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        write_period(f, period)?;
    }
    f.writeln()
}

/// Write a `CalScale` property.
pub fn write_prop_calscale<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &CalendarScale<S>,
) -> io::Result<()> {
    write!(f, "{KW_CALSCALE}:{}", prop.value)?;
    f.writeln()
}

/// Write a `Method` property.
pub fn write_prop_method<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Method<S>,
) -> io::Result<()> {
    write!(f, "{KW_METHOD}:{}", prop.value)?;
    f.writeln()
}

/// Write a `ProdId` property.
pub fn write_prop_prodid<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &ProductId<S>,
) -> io::Result<()> {
    write!(f, "{KW_PRODID}:{}", prop.value)?;
    f.writeln()
}

/// Write a `Version` property.
pub fn write_prop_version<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Version<S>,
) -> io::Result<()> {
    write!(f, "{KW_VERSION}:{}", prop.value)?;
    f.writeln()
}

/// Write a `Class` property.
pub fn write_prop_class<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Classification<S>,
) -> io::Result<()> {
    write!(f, "{KW_CLASS}:{}", prop.value)?;
    f.writeln()
}

/// Write a `TzId` property.
pub fn write_prop_tzid<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TzId<S>,
) -> io::Result<()> {
    write!(f, "{KW_TZID}:{}", prop.content)?;
    f.writeln()
}

/// Write a `Uid` property.
pub fn write_prop_uid<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Uid<S>,
) -> io::Result<()> {
    write!(f, "{KW_UID}:{}", prop.content)?;
    f.writeln()
}

/// Write a `Summary` property.
pub fn write_prop_summary<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Summary<S>,
) -> io::Result<()> {
    write_text_with_params(
        f,
        KW_SUMMARY,
        &prop.content,
        prop.language.as_ref(),
        prop.altrep.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Description` property.
pub fn write_prop_description<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Description<S>,
) -> io::Result<()> {
    write_text_with_params(
        f,
        KW_DESCRIPTION,
        &prop.content,
        prop.language.as_ref(),
        prop.altrep.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Location` property.
pub fn write_prop_location<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Location<S>,
) -> io::Result<()> {
    write_text_with_params(
        f,
        KW_LOCATION,
        &prop.content,
        prop.language.as_ref(),
        prop.altrep.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Comment` property.
pub fn write_prop_comment<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Comment<S>,
) -> io::Result<()> {
    write_text_with_language(
        f,
        KW_COMMENT,
        &prop.content,
        prop.language.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `TzName` property.
pub fn write_prop_tzname<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TzName<S>,
) -> io::Result<()> {
    write_text_with_language(
        f,
        KW_TZNAME,
        &prop.content,
        prop.language.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `DtStart` property.
pub fn write_prop_dtstart<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &DtStart<S>,
) -> io::Result<()> {
    write_datetime(f, KW_DTSTART, &prop.inner)?;
    f.writeln()
}

/// Write a `DtEnd` property.
pub fn write_prop_dtend<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &DtEnd<S>,
) -> io::Result<()> {
    write_datetime(f, KW_DTEND, &prop.inner)?;
    f.writeln()
}

/// Write a `DtStamp` property.
pub fn write_prop_dtstamp<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &DtStamp<S>,
) -> io::Result<()> {
    write_datetime(f, KW_DTSTAMP, &prop.inner)?;
    f.writeln()
}

/// Write a `Created` property.
pub fn write_prop_created<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Created<S>,
) -> io::Result<()> {
    write_datetime(f, KW_CREATED, &prop.inner)?;
    f.writeln()
}

/// Write a `LastModified` property.
pub fn write_prop_last_modified<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &LastModified<S>,
) -> io::Result<()> {
    write_datetime(f, KW_LAST_MODIFIED, &prop.inner)?;
    f.writeln()
}

/// Write a `Completed` property.
pub fn write_prop_completed<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Completed<S>,
) -> io::Result<()> {
    write_datetime(f, KW_COMPLETED, &prop.inner)?;
    f.writeln()
}

/// Write a `Due` property.
pub fn write_prop_due<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Due<S>,
) -> io::Result<()> {
    write_datetime(f, KW_DUE, &prop.inner)?;
    f.writeln()
}

/// Write a `RecurrenceId` property.
pub fn write_prop_recurrence_id<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &RecurrenceId<S>,
) -> io::Result<()> {
    write_datetime(f, KW_RECURRENCE_ID, &prop.inner)?;
    f.writeln()
}

/// Write a `Status` property.
pub fn write_prop_status<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Status<S>,
) -> io::Result<()> {
    write_escaped_text(
        f,
        KW_STATUS,
        &prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Status` property from a `StatusValue` directly.
pub fn write_prop_status_value<T: Into<StatusValue>, S: StringStorage>(
    f: &mut Formatter<impl Write>,
    status: T,
    x_parameters: &[RawParameter<S>],
    retained_parameters: &[Parameter<S>],
) -> io::Result<()> {
    let s: StatusValue = status.into();
    write_escaped_text(f, KW_STATUS, &s, x_parameters, retained_parameters)?;
    f.writeln()
}

/// Write a `Transp` property.
pub fn write_prop_transp<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TimeTransparency<S>,
) -> io::Result<()> {
    write_escaped_text(
        f,
        KW_TRANSP,
        &prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Priority` property.
pub fn write_prop_priority<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Priority<S>,
) -> io::Result<()> {
    write_integer(
        f,
        KW_PRIORITY,
        prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Sequence` property.
pub fn write_prop_sequence<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Sequence<S>,
) -> io::Result<()> {
    write_integer(
        f,
        KW_SEQUENCE,
        prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `PercentComplete` property.
pub fn write_prop_percent_complete<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &PercentComplete<S>,
) -> io::Result<()> {
    write_integer(
        f,
        KW_PERCENT_COMPLETE,
        prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Repeat` property.
pub fn write_prop_repeat<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Repeat<S>,
) -> io::Result<()> {
    write_integer(
        f,
        KW_REPEAT,
        prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write an `Action` property.
pub fn write_prop_action<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Action<S>,
) -> io::Result<()> {
    write_escaped_text(
        f,
        KW_ACTION,
        &prop.value,
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Contact` property.
pub fn write_prop_contact<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Contact<S>,
) -> io::Result<()> {
    write_text_with_language(
        f,
        KW_CONTACT,
        &prop.content,
        prop.language.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `RequestStatus` property.
pub fn write_prop_request_status<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &RequestStatus<S>,
) -> io::Result<()> {
    write_text_with_language(
        f,
        KW_REQUEST_STATUS,
        &prop.content,
        prop.language.as_ref(),
        &prop.x_parameters,
        &prop.retained_parameters,
    )?;
    f.writeln()
}

/// Write a `Trigger` property.
pub fn write_prop_trigger<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Trigger<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_TRIGGER}")?;

    // Write RELATED parameter if present, using centralized formatter
    if let Some(related) = &prop.related {
        write_param_related(f, *related)?;
    }

    // Write generic parameter lists
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;

    // Write property value
    write!(f, ":")?;
    match &prop.value {
        TriggerValue::Duration(d) => write_duration(f, d),
        TriggerValue::DateTime(dt) => write_datetime_value(f, dt),
    }?;
    f.writeln()
}

/// Write a `Url` property.
pub fn write_prop_url<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Url<S>,
) -> io::Result<()> {
    write_uri(f, KW_URL, prop)?;
    f.writeln()
}

/// Write a `TzUrl` property.
pub fn write_prop_tz_url<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TzUrl<S>,
) -> io::Result<()> {
    write_uri(f, KW_TZURL, prop)?;
    f.writeln()
}

/// Write an `ExDate` property.
pub fn write_prop_ex_date<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &ExDate<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_EXDATE}")?;

    // Write TZID parameter if present
    if let Some(tz) = &prop.tz_id {
        write_param_tzid(f, tz)?;
    }

    // Write generic parameter lists
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;

    // Write property value
    write!(f, ":")?;

    for (i, date) in prop.dates.iter().enumerate() {
        if i > 0 {
            write!(f, ",")?;
        }
        match date {
            ExDateValue::Date(d) => write_date(f, *d)?,
            ExDateValue::DateTime(dt) => write_datetime_value(f, dt)?,
        }
    }
    f.writeln()
}

/// Write an `Attendee` property.
pub fn write_prop_attendee<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &Attendee<S>,
) -> io::Result<()> {
    // Write property name
    write!(f, "{KW_ATTENDEE}")?;

    // Write all parameters using centralized formatters
    if let Some(v) = &prop.cn {
        write_param_cn(f, v)?;
    }
    write_param_cutype(f, &prop.cutype)?;
    write_param_role(f, &prop.role)?;
    write_param_partstat(f, &prop.part_stat)?;
    if let Some(v) = prop.rsvp {
        write_param_rsvp(f, v)?;
    }
    if let Some(values) = prop.member.as_deref() {
        write_param_member(f, values)?;
    }
    if let Some(values) = prop.delegated_to.as_deref() {
        write_param_delegated_to(f, values)?;
    }
    if let Some(values) = prop.delegated_from.as_deref() {
        write_param_delegated_from(f, values)?;
    }
    if let Some(v) = &prop.dir {
        write_param_dir(f, v)?;
    }
    if let Some(v) = &prop.sent_by {
        write_param_sent_by(f, v)?;
    }
    if let Some(v) = &prop.language {
        write_param_language(f, v)?;
    }

    // Write generic parameter lists
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;

    // Write property value
    write!(f, ":{}", prop.cal_address)?;
    f.writeln()
}

/// Write a `TzOffsetFrom` property.
pub fn write_prop_tz_offset_from<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TzOffsetFrom<S>,
) -> io::Result<()> {
    write!(f, "{KW_TZOFFSETFROM}")?;
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;
    write!(f, ":")?;
    write_utc_offset(f, prop.value)?;
    f.writeln()
}

/// Write a `TzOffsetTo` property.
pub fn write_prop_tz_offset_to<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &TzOffsetTo<S>,
) -> io::Result<()> {
    write!(f, "{KW_TZOFFSETTO}")?;
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;
    write!(f, ":")?;
    write_utc_offset(f, prop.value)?;
    f.writeln()
}

pub fn write_prop_xname<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &XNameProperty<S>,
) -> io::Result<()> {
    write!(f, "{}", prop.name)?;
    write_parameters(f, &prop.parameters)?;
    write!(f, ":")?;
    write_value(f, &prop.value)?;
    f.writeln()
}

pub fn write_prop_unrecognized<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    prop: &UnrecognizedProperty<S>,
) -> io::Result<()> {
    write!(f, "{}", prop.name)?;
    write_parameters(f, &prop.parameters)?;
    write!(f, ":")?;
    write_value(f, &prop.value)?;
    f.writeln()
}

// ============================================================================
// Helper functions for common patterns
// ============================================================================

/// Write a text property with parameters (Text type).
///
/// This handles properties like Summary, Description, Location, etc.
fn write_text_with_params<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    content: &ValueText<S>,
    language: Option<&S>,
    altrep: Option<&S>,
    x_params: &[RawParameter<S>],
    retained_params: &[Parameter<S>],
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
    write_syntax_parameters(f, x_params)?;
    write_parameters(f, retained_params)?;

    // Write property value with proper iCalendar escaping
    write!(f, ":{}", format_value_text(content))
}

/// Write a text property.
///
/// This is a generalized version for any Display content, and assumes no special escaping is needed.
fn write_escaped_text<D: Display, S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    content: &D,
    x_params: &[RawParameter<S>],
    retained_params: &[Parameter<S>],
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write generic parameter lists
    write_syntax_parameters(f, x_params)?;
    write_parameters(f, retained_params)?;

    // Write property value
    write!(f, ":{content}")
}

/// Write a text property with LANGUAGE parameter only for `ValueText` content.
///
/// This is a specialized version for `ValueText` that properly escapes special characters.
fn write_text_with_language<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    content: &ValueText<S>,
    language: Option<&S>,
    x_params: &[RawParameter<S>],
    retained_params: &[Parameter<S>],
) -> io::Result<()> {
    // Write property name
    write!(f, "{name}")?;

    // Write LANGUAGE parameter if present
    if let Some(lang) = language {
        write_param_language(f, lang)?;
    }

    // Write generic parameter lists
    write_syntax_parameters(f, x_params)?;
    write_parameters(f, retained_params)?;

    // Write property value with proper iCalendar escaping
    write!(f, ":{}", format_value_text(content))
}

/// Write a URI property (`Url` or `TzUrl`).
fn write_uri<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    prop: &UriProperty<S>,
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_syntax_parameters(f, &prop.x_parameters)?;
    write_parameters(f, &prop.retained_parameters)?;
    write!(f, ":{}", prop.uri)
}

/// Write an integer property.
fn write_integer<D: Display, S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    value: D,
    x_params: &[RawParameter<S>],
    retainedzed_params: &[Parameter<S>],
) -> io::Result<()> {
    write!(f, "{name}")?;
    write_syntax_parameters(f, x_params)?;
    write_parameters(f, retainedzed_params)?;
    write!(f, ":{value}")
}

/// Write a `DateTime` property.
fn write_datetime<S: StringStorage>(
    f: &mut Formatter<impl Write>,
    name: &str,
    datetime: &DateTime<S>,
) -> io::Result<()> {
    // Collect all parameters using a helper
    let (x_params, retained_params) = match datetime {
        DateTime::Floating {
            x_parameters,
            retained_parameters,
            ..
        }
        | DateTime::Zoned {
            x_parameters,
            retained_parameters,
            ..
        }
        | DateTime::Utc {
            x_parameters,
            retained_parameters,
            ..
        }
        | DateTime::Date {
            x_parameters,
            retained_parameters,
            ..
        } => (x_parameters, retained_parameters),
    };

    // Write: NAME;params:value
    write!(f, "{name}")?;
    write_syntax_parameters(f, x_params)?;
    write_parameters(f, retained_params)?;

    // Write the value
    write!(f, ":")?;
    write_datetime_value(f, datetime)
}

/// Write a `DateTime` value (without property name or params).
fn write_datetime_value<S: StringStorage>(
    f: &mut Formatter<impl Write>,
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
fn write_time(f: &mut Formatter<impl Write>, time: &Time, utc: bool) -> io::Result<()> {
    let utc = if utc { "Z" } else { "" };
    write!(
        f,
        "{:02}{:02}{:02}{}",
        time.hour, time.minute, time.second, utc
    )
}

/// Format a property Period value as date-time/date-time or date-time/duration.
fn write_period<S: StringStorage>(
    f: &mut Formatter<impl Write>,
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
