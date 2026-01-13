// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Component formatting for iCalendar components.
//!
//! This module provides functions to format all iCalendar component types
//! as defined in RFC 5545 Section 3.6.

use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::formatter::property::{
    write_prop_action, write_prop_attach, write_prop_attendee, write_prop_calscale,
    write_prop_categories, write_prop_class, write_prop_completed, write_prop_contact,
    write_prop_description, write_prop_dtend, write_prop_dtstamp, write_prop_dtstart,
    write_prop_due, write_prop_duration, write_prop_ex_date, write_prop_freebusy, write_prop_geo,
    write_prop_last_modified, write_prop_location, write_prop_method, write_prop_organizer,
    write_prop_percent_complete, write_prop_priority, write_prop_prodid, write_prop_rdate,
    write_prop_repeat, write_prop_resources, write_prop_rrule, write_prop_sequence,
    write_prop_status_value, write_prop_summary, write_prop_transp, write_prop_trigger,
    write_prop_tz_offset_from, write_prop_tz_offset_to, write_prop_tz_url, write_prop_tzid,
    write_prop_tzname, write_prop_uid, write_prop_url, write_prop_version, write_prop_xname,
    write_property,
};
use crate::keyword::{
    KW_BEGIN, KW_DAYLIGHT, KW_END, KW_STANDARD, KW_VALARM, KW_VCALENDAR, KW_VEVENT, KW_VFREEBUSY,
    KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::parameter::FreeBusyType;
use crate::property::FreeBusy;
use crate::semantic::{
    CalendarComponent, CustomComponent, ICalendar, TimeZoneObservance, VAlarm, VEvent, VFreeBusy,
    VJournal, VTimeZone, VTodo,
};
use crate::string_storage::StringStorage;

/// Format an `ICalendar` component.
pub fn write_icalendar<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    calendar: &ICalendar<S>,
) -> io::Result<()> {
    with_block(f, KW_VCALENDAR, |f| {
        // Required properties
        write_prop_prodid(f, &calendar.prod_id)?;
        write_prop_version(f, &calendar.version)?;

        // Optional properties
        if let Some(calscale) = &calendar.calscale {
            write_prop_calscale(f, calscale)?;
        }
        if let Some(method) = &calendar.method {
            write_prop_method(f, method)?;
        }

        // X-properties
        for prop in &calendar.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &calendar.retained_properties {
            write_property(f, prop)?;
        }

        // Components
        for component in &calendar.components {
            write_calendar_component(f, component)?;
        }

        Ok(())
    })
}

/// Format a calendar component (handles all component types).
///
/// This handles `Property::XName` and `Property::Unrecognized` variants.
fn write_calendar_component<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    component: &CalendarComponent<S>,
) -> io::Result<()> {
    match component {
        CalendarComponent::Event(v) => write_vevent(f, v),
        CalendarComponent::Todo(v) => write_vtodo(f, v),
        CalendarComponent::VJournal(v) => write_vjournal(f, v),
        CalendarComponent::VFreeBusy(v) => write_vfreebusy(f, v),
        CalendarComponent::VTimeZone(v) => write_vtimezone(f, v),
        CalendarComponent::VAlarm(v) => write_valarm(f, v),
        CalendarComponent::Custom(v) => write_custom_component(f, v),
    }
}

/// Format a `VEvent` component.
fn write_vevent<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    event: &VEvent<S>,
) -> io::Result<()> {
    with_block(f, KW_VEVENT, |f| {
        // Required properties
        write_prop_uid(f, &event.uid)?;
        write_prop_dtstamp(f, &event.dt_stamp)?;
        write_prop_dtstart(f, &event.dt_start)?;

        // Optional properties
        if let Some(dt_end) = &event.dt_end {
            write_prop_dtend(f, dt_end)?;
        }
        if let Some(duration) = &event.duration {
            write_prop_duration(f, duration)?;
        }
        if let Some(summary) = &event.summary {
            write_prop_summary(f, summary)?;
        }
        if let Some(description) = &event.description {
            write_prop_description(f, description)?;
        }
        if let Some(location) = &event.location {
            write_prop_location(f, location)?;
        }
        if let Some(geo) = &event.geo {
            write_prop_geo(f, geo)?;
        }
        if let Some(url) = &event.url {
            write_prop_url(f, url)?;
        }
        if let Some(organizer) = &event.organizer {
            write_prop_organizer(f, organizer)?;
        }
        for attendee in &event.attendees {
            write_prop_attendee(f, attendee)?;
        }
        if let Some(last_modified) = &event.last_modified {
            write_prop_last_modified(f, last_modified)?;
        }
        if let Some(status) = &event.status {
            write_prop_status_value(
                f,
                status.value,
                &status.x_parameters,
                &status.retained_parameters,
            )?;
        }
        if let Some(transparency) = &event.transparency {
            write_prop_transp(f, transparency)?;
        }
        if let Some(sequence) = &event.sequence {
            write_prop_sequence(f, sequence)?;
        }
        if let Some(priority) = &event.priority {
            write_prop_priority(f, priority)?;
        }
        if let Some(classification) = &event.classification {
            write_prop_class(f, classification)?;
        }
        if let Some(resources) = &event.resources {
            write_prop_resources(f, resources)?;
        }
        if let Some(categories) = &event.categories {
            write_prop_categories(f, categories)?;
        }
        if let Some(rrule) = &event.rrule {
            write_prop_rrule(f, rrule)?;
        }
        for rdate in &event.rdates {
            write_prop_rdate(f, rdate)?;
        }
        for exdate in &event.ex_dates {
            write_prop_ex_date(f, exdate)?;
        }

        // X-properties
        for prop in &event.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &event.retained_properties {
            write_property(f, prop)?;
        }

        // Alarms
        for alarm in &event.alarms {
            write_valarm(f, alarm)?;
        }

        Ok(())
    })
}

/// Format a `VTodo` component.
fn write_vtodo<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    todo: &VTodo<S>,
) -> io::Result<()> {
    with_block(f, KW_VTODO, |f| {
        // Required properties
        write_prop_uid(f, &todo.uid)?;
        write_prop_dtstamp(f, &todo.dt_stamp)?;

        // Optional properties
        if let Some(dt_start) = &todo.dt_start {
            write_prop_dtstart(f, dt_start)?;
        }
        if let Some(due) = &todo.due {
            write_prop_due(f, due)?;
        }
        if let Some(completed) = &todo.completed {
            write_prop_completed(f, completed)?;
        }
        if let Some(duration) = &todo.duration {
            write_prop_duration(f, duration)?;
        }
        if let Some(summary) = &todo.summary {
            write_prop_summary(f, summary)?;
        }
        if let Some(description) = &todo.description {
            write_prop_description(f, description)?;
        }
        if let Some(location) = &todo.location {
            write_prop_location(f, location)?;
        }
        if let Some(geo) = &todo.geo {
            write_prop_geo(f, geo)?;
        }
        if let Some(url) = &todo.url {
            write_prop_url(f, url)?;
        }
        if let Some(organizer) = &todo.organizer {
            write_prop_organizer(f, organizer)?;
        }
        for attendee in &todo.attendees {
            write_prop_attendee(f, attendee)?;
        }
        if let Some(last_modified) = &todo.last_modified {
            write_prop_last_modified(f, last_modified)?;
        }
        if let Some(status) = &todo.status {
            write_prop_status_value(
                f,
                status.value,
                &status.x_parameters,
                &status.retained_parameters,
            )?;
        }
        if let Some(sequence) = &todo.sequence {
            write_prop_sequence(f, sequence)?;
        }
        if let Some(priority) = &todo.priority {
            write_prop_priority(f, priority)?;
        }
        if let Some(percent_complete) = &todo.percent_complete {
            write_prop_percent_complete(f, percent_complete)?;
        }
        if let Some(classification) = &todo.classification {
            write_prop_class(f, classification)?;
        }
        if let Some(resources) = &todo.resources {
            write_prop_resources(f, resources)?;
        }
        if let Some(categories) = &todo.categories {
            write_prop_categories(f, categories)?;
        }
        if let Some(rrule) = &todo.rrule {
            write_prop_rrule(f, rrule)?;
        }
        for rdate in &todo.rdates {
            write_prop_rdate(f, rdate)?;
        }
        for exdate in &todo.ex_dates {
            write_prop_ex_date(f, exdate)?;
        }

        // X-properties
        for prop in &todo.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &todo.retained_properties {
            write_property(f, prop)?;
        }

        // Alarms
        for alarm in &todo.alarms {
            write_valarm(f, alarm)?;
        }

        Ok(())
    })
}

/// Format a `VJournal` component.
fn write_vjournal<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    journal: &VJournal<S>,
) -> io::Result<()> {
    with_block(f, KW_VJOURNAL, |f| {
        // Required properties
        write_prop_uid(f, &journal.uid)?;
        write_prop_dtstamp(f, &journal.dt_stamp)?;
        write_prop_dtstart(f, &journal.dt_start)?;

        // Optional properties
        if let Some(summary) = &journal.summary {
            write_prop_summary(f, summary)?;
        }
        for description in &journal.descriptions {
            write_prop_description(f, description)?;
        }
        if let Some(organizer) = &journal.organizer {
            write_prop_organizer(f, organizer)?;
        }
        for attendee in &journal.attendees {
            write_prop_attendee(f, attendee)?;
        }
        if let Some(last_modified) = &journal.last_modified {
            write_prop_last_modified(f, last_modified)?;
        }
        if let Some(status) = &journal.status {
            write_prop_status_value(
                f,
                status.value,
                &status.x_parameters,
                &status.retained_parameters,
            )?;
        }
        if let Some(classification) = &journal.classification {
            write_prop_class(f, classification)?;
        }
        for categories in &journal.categories {
            write_prop_categories(f, categories)?;
        }
        if let Some(rrule) = &journal.rrule {
            write_prop_rrule(f, rrule)?;
        }
        for rdate in &journal.rdate {
            write_prop_rdate(f, rdate)?;
        }
        for exdate in &journal.ex_date {
            write_prop_ex_date(f, exdate)?;
        }
        if let Some(url) = &journal.url {
            write_prop_url(f, url)?;
        }

        // X-properties
        for prop in &journal.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &journal.retained_properties {
            write_property(f, prop)?;
        }

        Ok(())
    })
}

/// Format a `VFreeBusy` component.
fn write_vfreebusy<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    freebusy: &VFreeBusy<S>,
) -> io::Result<()> {
    with_block(f, KW_VFREEBUSY, |f| {
        // Required properties
        write_prop_uid(f, &freebusy.uid)?;
        write_prop_dtstamp(f, &freebusy.dt_stamp)?;
        write_prop_dtstart(f, &freebusy.dt_start)?;
        write_prop_organizer(f, &freebusy.organizer)?;

        // Optional properties
        if let Some(dt_end) = &freebusy.dt_end {
            write_prop_dtend(f, dt_end)?;
        }
        if let Some(duration) = &freebusy.duration {
            write_prop_duration(f, duration)?;
        }
        if let Some(contact) = &freebusy.contact {
            write_prop_contact(f, contact)?;
        }
        if let Some(url) = &freebusy.url {
            write_prop_url(f, url)?;
        }

        // FreeBusy period collections (each with their own FBTYPE)
        for period in &freebusy.busy {
            write_prop_freebusy(
                f,
                &FreeBusy {
                    values: vec![period.clone()], // TODO: avoid clone
                    fb_type: FreeBusyType::Busy,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                    span: freebusy.uid.span, // placeholder span
                },
            )?;
        }
        for period in &freebusy.free {
            write_prop_freebusy(
                f,
                &FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::Free,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                    span: freebusy.uid.span, // placeholder span
                },
            )?;
        }
        for period in &freebusy.busy_tentative {
            write_prop_freebusy(
                f,
                &FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::BusyTentative,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                    span: freebusy.uid.span, // placeholder span
                },
            )?;
        }
        for period in &freebusy.busy_unavailable {
            write_prop_freebusy(
                f,
                &FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::BusyUnavailable,
                    x_parameters: Vec::new(),
                    retained_parameters: Vec::new(),
                    span: freebusy.uid.span, // placeholder span
                },
            )?;
        }

        // X-properties
        for prop in &freebusy.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &freebusy.retained_properties {
            write_property(f, prop)?;
        }

        Ok(())
    })
}

/// Format a `VTimeZone` component.
fn write_vtimezone<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    timezone: &VTimeZone<S>,
) -> io::Result<()> {
    with_block(f, KW_VTIMEZONE, |f| {
        // Required properties
        write_prop_tzid(f, &timezone.tz_id)?;

        // Optional properties
        if let Some(ref last_modified) = timezone.last_modified {
            write_prop_last_modified(f, last_modified)?;
        }
        if let Some(ref tz_url) = timezone.tz_url {
            write_prop_tz_url(f, tz_url)?;
        }

        // X-name properties
        for prop in &timezone.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &timezone.retained_properties {
            write_property(f, prop)?;
        }

        // Standard time observances
        for standard in &timezone.standard {
            format_tz_observance(f, KW_STANDARD, standard)?;
        }

        // Daylight saving time observances
        for daylight in &timezone.daylight {
            format_tz_observance(f, KW_DAYLIGHT, daylight)?;
        }

        Ok(())
    })
}

/// Format a timezone observance (STANDARD or DAYLIGHT) component.
fn format_tz_observance<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    name: &str,
    observance: &TimeZoneObservance<S>,
) -> io::Result<()> {
    with_block(f, name, |f| {
        // Required properties
        write_prop_dtstart(f, &observance.dt_start)?;
        write_prop_tz_offset_from(f, &observance.tz_offset_from)?;
        write_prop_tz_offset_to(f, &observance.tz_offset_to)?;

        // Optional TZNAME properties (can appear multiple times)
        for tz_name in &observance.tz_names {
            write_prop_tzname(f, tz_name)?;
        }

        // Optional RRULE
        if let Some(ref rrule) = observance.rrule {
            write_prop_rrule(f, rrule)?;
        }

        // X-name properties
        for prop in &observance.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &observance.retained_properties {
            write_property(f, prop)?;
        }

        Ok(())
    })
}

/// Format a `VAlarm` component.
fn write_valarm<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    alarm: &VAlarm<S>,
) -> io::Result<()> {
    with_block(f, KW_VALARM, |f| {
        // Required properties
        write_prop_action(f, &alarm.action)?;
        write_prop_trigger(f, &alarm.trigger)?;

        // Optional properties (DURATION and REPEAT must appear together)
        if let Some(ref repeat) = alarm.repeat {
            write_prop_repeat(f, repeat)?;
        }
        if let Some(ref duration) = alarm.duration {
            write_prop_duration(f, duration)?;
        }

        // Optional description (required for DISPLAY and EMAIL actions)
        if let Some(ref description) = alarm.description {
            write_prop_description(f, description)?;
        }

        // Optional summary (required for EMAIL action)
        if let Some(ref summary) = alarm.summary {
            write_prop_summary(f, summary)?;
        }

        // Optional attendees (for EMAIL action)
        for attendee in &alarm.attendees {
            write_prop_attendee(f, attendee)?;
        }

        // Optional attachment
        if let Some(ref attach) = alarm.attach {
            write_prop_attach(f, attach)?;
        }

        // X-name properties
        for prop in &alarm.x_properties {
            write_prop_xname(f, prop)?;
        }

        // Unrecognized properties
        for prop in &alarm.retained_properties {
            write_property(f, prop)?;
        }

        Ok(())
    })
}

/// Format a custom component (x-comp).
fn write_custom_component<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    component: &CustomComponent<S>,
) -> io::Result<()> {
    with_block(f, &component.name, |f| {
        // Properties
        for prop in &component.properties {
            write_property(f, prop)?;
        }

        // Children
        for child in &component.children {
            write_calendar_component(f, child)?;
        }

        Ok(())
    })
}

/// Write a block with BEGIN and END.
fn with_block<W: Write, F: FnOnce(&mut Formatter<W>) -> io::Result<()>>(
    f: &mut Formatter<W>,
    name: &str,
    write_content: F,
) -> io::Result<()> {
    write!(f, "{KW_BEGIN}:{name}")?;
    f.writeln()?;

    write_content(f)?;

    write!(f, "{KW_END}:{name}")?;
    f.writeln()
}
