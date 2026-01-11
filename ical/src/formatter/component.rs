// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Component formatting for iCalendar components.
//!
//! This module provides functions to format all iCalendar component types
//! as defined in RFC 5545 Section 3.6.

use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::formatter::property::write_property;
use crate::keyword::{
    KW_BEGIN, KW_DAYLIGHT, KW_END, KW_STANDARD, KW_VALARM, KW_VCALENDAR, KW_VEVENT, KW_VFREEBUSY,
    KW_VJOURNAL, KW_VTIMEZONE, KW_VTODO,
};
use crate::parameter::FreeBusyType;
use crate::property::{FreeBusy, Property, Status, StatusValue};
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
        write_property(f, &Property::ProdId(calendar.prod_id.clone()))?;
        write_property(f, &Property::Version(calendar.version.clone()))?;

        // Optional properties
        if let Some(calscale) = &calendar.calscale {
            write_property(f, &Property::CalScale(calscale.clone()))?;
        }
        if let Some(method) = &calendar.method {
            write_property(f, &Property::Method(method.clone()))?;
        }

        // X-properties
        for prop in &calendar.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &calendar.unrecognized_properties {
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
        write_property(f, &Property::Uid(event.uid.clone()))?;
        write_property(f, &Property::DtStamp(event.dt_stamp.clone()))?;
        write_property(f, &Property::DtStart(event.dt_start.clone()))?;

        // Optional properties
        if let Some(dt_end) = &event.dt_end {
            write_property(f, &Property::DtEnd(dt_end.clone()))?;
        }
        if let Some(duration) = &event.duration {
            write_property(f, &Property::Duration(duration.clone()))?;
        }
        if let Some(summary) = &event.summary {
            write_property(f, &Property::Summary(summary.clone()))?;
        }
        if let Some(description) = &event.description {
            write_property(f, &Property::Description(description.clone()))?;
        }
        if let Some(location) = &event.location {
            write_property(f, &Property::Location(location.clone()))?;
        }
        if let Some(geo) = &event.geo {
            write_property(f, &Property::Geo(geo.clone()))?;
        }
        if let Some(url) = &event.url {
            write_property(f, &Property::Url(url.clone()))?;
        }
        if let Some(organizer) = &event.organizer {
            write_property(f, &Property::Organizer(organizer.clone()))?;
        }
        for attendee in &event.attendees {
            write_property(f, &Property::Attendee(attendee.clone()))?;
        }
        if let Some(last_modified) = &event.last_modified {
            write_property(f, &Property::LastModified(last_modified.clone()))?;
        }
        if let Some(status) = &event.status {
            write_property(
                f,
                &Property::Status(Status {
                    value: StatusValue::from(status.value),
                    x_parameters: status.x_parameters.clone(),
                    unrecognized_parameters: status.unrecognized_parameters.clone(),
                    span: event.uid.span,
                }),
            )?;
        }
        if let Some(transparency) = &event.transparency {
            write_property(f, &Property::Transp(transparency.clone()))?;
        }
        if let Some(sequence) = &event.sequence {
            write_property(f, &Property::Sequence(sequence.clone()))?;
        }
        if let Some(priority) = &event.priority {
            write_property(f, &Property::Priority(priority.clone()))?;
        }
        if let Some(classification) = &event.classification {
            write_property(f, &Property::Class(classification.clone()))?;
        }
        if let Some(resources) = &event.resources {
            write_property(f, &Property::Resources(resources.clone()))?;
        }
        if let Some(categories) = &event.categories {
            write_property(f, &Property::Categories(categories.clone()))?;
        }
        if let Some(rrule) = &event.rrule {
            write_property(f, &Property::RRule(rrule.clone()))?;
        }
        for rdate in &event.rdates {
            write_property(f, &Property::RDate(rdate.clone()))?;
        }
        for exdate in &event.ex_dates {
            write_property(f, &Property::ExDate(exdate.clone()))?;
        }

        // X-properties
        for prop in &event.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &event.unrecognized_properties {
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
        write_property(f, &Property::Uid(todo.uid.clone()))?;
        write_property(f, &Property::DtStamp(todo.dt_stamp.clone()))?;

        // Optional properties
        if let Some(dt_start) = &todo.dt_start {
            write_property(f, &Property::DtStart(dt_start.clone()))?;
        }
        if let Some(due) = &todo.due {
            write_property(f, &Property::Due(due.clone()))?;
        }
        if let Some(completed) = &todo.completed {
            write_property(f, &Property::Completed(completed.clone()))?;
        }
        if let Some(duration) = &todo.duration {
            write_property(f, &Property::Duration(duration.clone()))?;
        }
        if let Some(summary) = &todo.summary {
            write_property(f, &Property::Summary(summary.clone()))?;
        }
        if let Some(description) = &todo.description {
            write_property(f, &Property::Description(description.clone()))?;
        }
        if let Some(location) = &todo.location {
            write_property(f, &Property::Location(location.clone()))?;
        }
        if let Some(geo) = &todo.geo {
            write_property(f, &Property::Geo(geo.clone()))?;
        }
        if let Some(url) = &todo.url {
            write_property(f, &Property::Url(url.clone()))?;
        }
        if let Some(organizer) = &todo.organizer {
            write_property(f, &Property::Organizer(organizer.clone()))?;
        }
        for attendee in &todo.attendees {
            write_property(f, &Property::Attendee(attendee.clone()))?;
        }
        if let Some(last_modified) = &todo.last_modified {
            write_property(f, &Property::LastModified(last_modified.clone()))?;
        }
        if let Some(status) = &todo.status {
            write_property(
                f,
                &Property::Status(Status {
                    value: StatusValue::from(status.value),
                    x_parameters: status.x_parameters.clone(),
                    unrecognized_parameters: status.unrecognized_parameters.clone(),
                    span: todo.uid.span,
                }),
            )?;
        }
        if let Some(sequence) = &todo.sequence {
            write_property(f, &Property::Sequence(sequence.clone()))?;
        }
        if let Some(priority) = &todo.priority {
            write_property(f, &Property::Priority(priority.clone()))?;
        }
        if let Some(percent_complete) = &todo.percent_complete {
            write_property(f, &Property::PercentComplete(percent_complete.clone()))?;
        }
        if let Some(classification) = &todo.classification {
            write_property(f, &Property::Class(classification.clone()))?;
        }
        if let Some(resources) = &todo.resources {
            write_property(f, &Property::Resources(resources.clone()))?;
        }
        if let Some(categories) = &todo.categories {
            write_property(f, &Property::Categories(categories.clone()))?;
        }
        if let Some(rrule) = &todo.rrule {
            write_property(f, &Property::RRule(rrule.clone()))?;
        }
        for rdate in &todo.rdates {
            write_property(f, &Property::RDate(rdate.clone()))?;
        }
        for exdate in &todo.ex_dates {
            write_property(f, &Property::ExDate(exdate.clone()))?;
        }

        // X-properties
        for prop in &todo.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &todo.unrecognized_properties {
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
        write_property(f, &Property::Uid(journal.uid.clone()))?;
        write_property(f, &Property::DtStamp(journal.dt_stamp.clone()))?;
        write_property(f, &Property::DtStart(journal.dt_start.clone()))?;

        // Optional properties
        if let Some(summary) = &journal.summary {
            write_property(f, &Property::Summary(summary.clone()))?;
        }
        for description in &journal.descriptions {
            write_property(f, &Property::Description(description.clone()))?;
        }
        if let Some(organizer) = &journal.organizer {
            write_property(f, &Property::Organizer(organizer.clone()))?;
        }
        for attendee in &journal.attendees {
            write_property(f, &Property::Attendee(attendee.clone()))?;
        }
        if let Some(last_modified) = &journal.last_modified {
            write_property(f, &Property::LastModified(last_modified.clone()))?;
        }
        if let Some(status) = &journal.status {
            write_property(
                f,
                &Property::Status(Status {
                    value: StatusValue::from(status.value),
                    x_parameters: status.x_parameters.clone(),
                    unrecognized_parameters: status.unrecognized_parameters.clone(),
                    span: journal.uid.span,
                }),
            )?;
        }
        if let Some(classification) = &journal.classification {
            write_property(f, &Property::Class(classification.clone()))?;
        }
        for categories in &journal.categories {
            write_property(f, &Property::Categories(categories.clone()))?;
        }
        if let Some(rrule) = &journal.rrule {
            write_property(f, &Property::RRule(rrule.clone()))?;
        }
        for rdate in &journal.rdate {
            write_property(f, &Property::RDate(rdate.clone()))?;
        }
        for exdate in &journal.ex_date {
            write_property(f, &Property::ExDate(exdate.clone()))?;
        }
        if let Some(url) = &journal.url {
            write_property(f, &Property::Url(url.clone()))?;
        }

        // X-properties
        for prop in &journal.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &journal.unrecognized_properties {
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
        write_property(f, &Property::Uid(freebusy.uid.clone()))?;
        write_property(f, &Property::DtStamp(freebusy.dt_stamp.clone()))?;
        write_property(f, &Property::DtStart(freebusy.dt_start.clone()))?;
        write_property(f, &Property::Organizer(freebusy.organizer.clone()))?;

        // Optional properties
        if let Some(dt_end) = &freebusy.dt_end {
            write_property(f, &Property::DtEnd(dt_end.clone()))?;
        }
        if let Some(duration) = &freebusy.duration {
            write_property(f, &Property::Duration(duration.clone()))?;
        }
        if let Some(contact) = &freebusy.contact {
            write_property(f, &Property::Contact(contact.clone()))?;
        }
        if let Some(url) = &freebusy.url {
            write_property(f, &Property::Url(url.clone()))?;
        }

        // FreeBusy period collections (each with their own FBTYPE)
        for period in &freebusy.busy {
            write_property(
                f,
                &Property::FreeBusy(FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::Busy,
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                    span: freebusy.uid.span,
                }),
            )?;
        }
        for period in &freebusy.free {
            write_property(
                f,
                &Property::FreeBusy(FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::Free,
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                    span: freebusy.uid.span,
                }),
            )?;
        }
        for period in &freebusy.busy_tentative {
            write_property(
                f,
                &Property::FreeBusy(FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::BusyTentative,
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                    span: freebusy.uid.span,
                }),
            )?;
        }
        for period in &freebusy.busy_unavailable {
            write_property(
                f,
                &Property::FreeBusy(FreeBusy {
                    values: vec![period.clone()],
                    fb_type: FreeBusyType::BusyUnavailable,
                    x_parameters: Vec::new(),
                    unrecognized_parameters: Vec::new(),
                    span: freebusy.uid.span,
                }),
            )?;
        }

        // X-properties
        for prop in &freebusy.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &freebusy.unrecognized_properties {
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
        write_property(f, &Property::TzId(timezone.tz_id.clone()))?;

        // Optional properties
        if let Some(ref last_modified) = timezone.last_modified {
            write_property(f, &Property::LastModified(last_modified.clone()))?;
        }
        if let Some(ref tz_url) = timezone.tz_url {
            write_property(f, &Property::TzUrl(tz_url.clone()))?;
        }

        // X-name properties
        for prop in &timezone.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &timezone.unrecognized_properties {
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
        write_property(f, &Property::DtStart(observance.dt_start.clone()))?;
        write_property(
            f,
            &Property::TzOffsetFrom(observance.tz_offset_from.clone()),
        )?;
        write_property(f, &Property::TzOffsetTo(observance.tz_offset_to.clone()))?;

        // Optional TZNAME properties (can appear multiple times)
        for tz_name in &observance.tz_names {
            write_property(f, &Property::TzName(tz_name.clone()))?;
        }

        // Optional RRULE
        if let Some(ref rrule) = observance.rrule {
            write_property(f, &Property::RRule(rrule.clone()))?;
        }

        // X-name properties
        for prop in &observance.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &observance.unrecognized_properties {
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
        write_property(f, &Property::Action(alarm.action.clone()))?;
        write_property(f, &Property::Trigger(alarm.trigger.clone()))?;

        // Optional properties (DURATION and REPEAT must appear together)
        if let Some(ref repeat) = alarm.repeat {
            write_property(f, &Property::Repeat(repeat.clone()))?;
        }
        if let Some(ref duration) = alarm.duration {
            write_property(f, &Property::Duration(duration.clone()))?;
        }

        // Optional description (required for DISPLAY and EMAIL actions)
        if let Some(ref description) = alarm.description {
            write_property(f, &Property::Description(description.clone()))?;
        }

        // Optional summary (required for EMAIL action)
        if let Some(ref summary) = alarm.summary {
            write_property(f, &Property::Summary(summary.clone()))?;
        }

        // Optional attendees (for EMAIL action)
        for attendee in &alarm.attendees {
            write_property(f, &Property::Attendee(attendee.clone()))?;
        }

        // Optional attachment
        if let Some(ref attach) = alarm.attach {
            write_property(f, &Property::Attach(attach.clone()))?;
        }

        // X-name properties
        for prop in &alarm.x_properties {
            write_property(f, prop)?;
        }

        // Unrecognized properties
        for prop in &alarm.unrecognized_properties {
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
