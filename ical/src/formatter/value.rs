// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Value formatting for iCalendar values.
//!
//! This module provides functions to format all iCalendar value types
//! as defined in RFC 5545 Section 3.3.

use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::keyword::{
    KW_BOOLEAN_FALSE, KW_BOOLEAN_TRUE, KW_RRULE_BYDAY, KW_RRULE_BYHOUR, KW_RRULE_BYMINUTE,
    KW_RRULE_BYMONTH, KW_RRULE_BYMONTHDAY, KW_RRULE_BYSECOND, KW_RRULE_BYSETPOS, KW_RRULE_BYWEEKNO,
    KW_RRULE_BYYEARDAY, KW_RRULE_COUNT, KW_RRULE_FREQ, KW_RRULE_INTERVAL, KW_RRULE_UNTIL,
    KW_RRULE_WKST,
};
use crate::string_storage::StringStorage;
use crate::value::{
    Value, ValueDate, ValueDateTime, ValueDuration, ValuePeriod, ValueRecurrenceRule, ValueText,
    ValueTime, ValueUtcOffset, WeekDayNum,
};

/// Format a value to the formatter.
///
/// This is the main entry point for formatting values.
pub fn write_value<W: Write, S: StringStorage>(
    f: &mut Formatter<W>,
    value: &Value<S>,
) -> io::Result<()> {
    match value {
        // Binary data is already base64 encoded from parsing
        // CalAddress / URI: no escaping per RFC 5545
        Value::Binary { value, .. }
        | Value::CalAddress { value, .. }
        | Value::Uri { value, .. } => write!(f, "{value}"),
        Value::Boolean { value, .. } => {
            let v = if *value {
                KW_BOOLEAN_TRUE
            } else {
                KW_BOOLEAN_FALSE
            };
            write!(f, "{v}",)
        }
        Value::Date { values, .. } => {
            for (i, date) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_date(f, *date)?;
            }
            Ok(())
        }
        Value::DateTime { values, .. } => {
            for (i, datetime) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_date_time(f, datetime)?;
            }
            Ok(())
        }
        Value::Duration { values, .. } => {
            for (i, duration) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_duration(f, duration)?;
            }
            Ok(())
        }
        Value::Float { values, .. } => {
            for (i, float) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{float}")?;
            }
            Ok(())
        }
        Value::Integer { values, .. } => {
            for (i, int) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{int}")?;
            }
            Ok(())
        }
        Value::Period { values, .. } => {
            for (i, period) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_period(f, period)?;
            }
            Ok(())
        }
        Value::Text { values, .. } => {
            for (i, text) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                // Use format_value_text to properly escape for iCalendar output
                write!(f, "{}", format_value_text(text))?;
            }
            Ok(())
        }
        Value::Time { values, .. } => {
            for (i, time) in values.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write_time(f, time)?;
            }
            Ok(())
        }
        Value::UtcOffset { value, .. } => write_utc_offset(f, *value),
        Value::RecurrenceRule { value, .. } => write_recurrence_rule(f, value),
        // XName / Unrecognized values are stored as raw unparsed strings
        Value::XName { raw, .. } | Value::Unrecognized { raw, .. } => write!(f, "{raw}"),
    }
}

/// Format a date value as `YYYYMMDD`.
pub fn write_date<W: Write>(f: &mut Formatter<W>, date: ValueDate) -> io::Result<()> {
    write!(f, "{:04}{:02}{:02}", date.year, date.month, date.day)
}

/// Format a date-time value as `YYYYMMDDTHHMMSS[Z]`.
fn write_date_time<W: Write>(f: &mut Formatter<W>, datetime: &ValueDateTime) -> io::Result<()> {
    write_date(f, datetime.date)?;
    write!(f, "T")?;
    write_time(f, &datetime.time)
}

/// Format a duration value as `P[n]DT[n]H[n]M[n]S` (RFC 5545 Section 3.3.6).
pub fn write_duration<W: Write>(f: &mut Formatter<W>, duration: &ValueDuration) -> io::Result<()> {
    // Get positive flag and write sign
    match duration {
        ValueDuration::Week { positive, .. } | ValueDuration::DateTime { positive, .. }
            if !positive =>
        {
            write!(f, "-")?;
        }
        _ => { /* positive, no sign */ }
    }
    write!(f, "P")?;

    match duration {
        ValueDuration::Week { week, .. } => write!(f, "{week}W")?,
        ValueDuration::DateTime {
            day,
            hour,
            minute,
            second,
            ..
        } => {
            // Only include components that are non-zero
            let mut has_time = false;
            if *day > 0 {
                write!(f, "{day}D")?;
            }
            if *hour > 0 || *minute > 0 || *second > 0 {
                write!(f, "T")?;
                has_time = true;
            }
            if *hour > 0 {
                write!(f, "{hour}H")?;
            }
            if *minute > 0 {
                write!(f, "{minute}M")?;
            }
            if *second > 0 {
                write!(f, "{second}S")?;
            }
            // If all are zero, we still need to output something like PT0S
            if !has_time && *day == 0 {
                write!(f, "T0S")?;
            }
        }
    }
    Ok(())
}

/// Format a period value as `start/end` or `start/duration`.
pub fn write_period<W: Write>(f: &mut Formatter<W>, period: &ValuePeriod) -> io::Result<()> {
    // Format: start/end OR start/duration
    match period {
        ValuePeriod::Explicit { start, end } => {
            write_date_time(f, start)?;
            write!(f, "/")?;
            write_date_time(f, end)?;
        }
        ValuePeriod::Duration { start, duration } => {
            write_date_time(f, start)?;
            write!(f, "/")?;
            write_duration(f, duration)?;
        }
    }
    Ok(())
}

/// Format a time value as `HHMMSS[Z]`.
pub fn write_time<W: Write>(w: &mut W, time: &ValueTime) -> io::Result<()> {
    let utc = if time.utc { "Z" } else { "" };
    write!(
        w,
        "{:02}{:02}{:02}{}",
        time.hour, time.minute, time.second, utc
    )
}

/// Format a UTC offset value as `+HHMM` or `-HHMM` (with optional seconds).
fn write_utc_offset<W: Write>(f: &mut Formatter<W>, offset: ValueUtcOffset) -> io::Result<()> {
    let sign = if offset.positive { "+" } else { "-" };
    write!(f, "{sign}{:02}{:02}", offset.hour, offset.minute)?;
    if let Some(second) = offset.second {
        write!(f, "{second:02}")?;
    }
    Ok(())
}

/// Format a recurrence rule value (RFC 5545 Section 3.3.10).
pub fn write_recurrence_rule<W: Write>(
    f: &mut Formatter<W>,
    rule: &ValueRecurrenceRule,
) -> io::Result<()> {
    // Format: FREQ=DAILY;INTERVAL=1;...

    // FREQ is required
    write!(f, "{KW_RRULE_FREQ}={}", rule.freq)?;

    // UNTIL or COUNT (optional, mutually exclusive)
    if let Some(until) = &rule.until {
        write!(f, ";{KW_RRULE_UNTIL}=")?;
        write_date_time(f, until)?;
    } else if let Some(count) = rule.count {
        write!(f, ";{KW_RRULE_COUNT}={count}")?;
    }

    // INTERVAL (optional)
    if let Some(interval) = rule.interval {
        write!(f, ";{KW_RRULE_INTERVAL}={interval}")?;
    }

    // BYSECOND (optional)
    if !rule.by_second.is_empty() {
        write!(f, ";{KW_RRULE_BYSECOND}=")?;
        for (i, sec) in rule.by_second.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{sec}")?;
        }
    }

    // BYMINUTE (optional)
    if !rule.by_minute.is_empty() {
        write!(f, ";{KW_RRULE_BYMINUTE}=")?;
        for (i, min) in rule.by_minute.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{min}")?;
        }
    }

    // BYHOUR (optional)
    if !rule.by_hour.is_empty() {
        write!(f, ";{KW_RRULE_BYHOUR}=")?;
        for (i, hour) in rule.by_hour.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{hour}")?;
        }
    }

    // BYMONTHDAY (optional)
    if !rule.by_month_day.is_empty() {
        write!(f, ";{KW_RRULE_BYMONTHDAY}=")?;
        for (i, day) in rule.by_month_day.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{day}")?;
        }
    }

    // BYYEARDAY (optional)
    if !rule.by_year_day.is_empty() {
        write!(f, ";{KW_RRULE_BYYEARDAY}=")?;
        for (i, day) in rule.by_year_day.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{day}")?;
        }
    }

    // BYWEEKNO (optional)
    if !rule.by_week_no.is_empty() {
        write!(f, ";{KW_RRULE_BYWEEKNO}=")?;
        for (i, week) in rule.by_week_no.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{week}")?;
        }
    }

    // BYMONTH (optional)
    if !rule.by_month.is_empty() {
        write!(f, ";{KW_RRULE_BYMONTH}=")?;
        for (i, month) in rule.by_month.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{month}")?;
        }
    }

    // BYDAY (optional)
    if !rule.by_day.is_empty() {
        write!(f, ";{KW_RRULE_BYDAY}=")?;
        for (i, weekday) in rule.by_day.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", format_weekday_num(*weekday))?;
        }
    }

    // BYSETPOS (optional)
    if !rule.by_set_pos.is_empty() {
        write!(f, ";{KW_RRULE_BYSETPOS}=")?;
        for (i, pos) in rule.by_set_pos.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{pos}")?;
        }
    }

    // WKST (optional)
    if let Some(wkst) = rule.wkst {
        write!(f, ";{KW_RRULE_WKST}={wkst}")?;
    }

    Ok(())
}

/// Format a weekday number (e.g., "1MO", "-1FR", "SU").
fn format_weekday_num(wn: WeekDayNum) -> String {
    match wn.occurrence {
        Some(occ) => format!("{occ}{}", wn.day),
        None => wn.day.to_string(),
    }
}

/// Format a `ValueText` as an iCalendar escaped string.
///
/// This function properly escapes special characters per RFC 5545:
/// - Backslash → \\
/// - Semicolon → \;
/// - Comma → \,
/// - Newline → \n
///
/// # Arguments
///
/// * `text` - The text value to format
///
/// # Returns
///
/// A string with proper iCalendar escape sequences
pub fn format_value_text<S: StringStorage>(text: &ValueText<S>) -> String {
    let mut result = String::new();

    // Use Display to get the resolved (unescaped) text
    let resolved = text.to_string();

    // Re-escape special characters for iCalendar format
    for c in resolved.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            ';' => result.push_str("\\;"),
            ',' => result.push_str("\\,"),
            '\n' => result.push_str("\\n"),
            '\r' => {} // Skip CR characters
            _ => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::formatter::Formatter;

    #[test]
    fn test_format_date() {
        let date = ValueDate {
            year: 1997,
            month: 7,
            day: 14,
        };
        let mut buffer = Vec::new();
        let mut f = Formatter::new(&mut buffer);
        write_date(&mut f, date).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "19970714");
    }

    #[test]
    fn test_format_time() {
        let time = ValueTime::new(13, 30, 0, false);
        let mut buffer = Vec::new();
        write_time(&mut buffer, &time).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "133000");

        let time_utc = ValueTime::new(7, 0, 0, true);
        let mut buffer = Vec::new();
        write_time(&mut buffer, &time_utc).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "070000Z");
    }

    #[test]
    fn test_format_utc_offset() {
        let offset = ValueUtcOffset {
            positive: false,
            hour: 5,
            minute: 0,
            second: None,
        };
        let mut buffer = Vec::new();
        let mut f = Formatter::new(&mut buffer);
        write_utc_offset(&mut f, offset).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "-0500");

        let offset = ValueUtcOffset {
            positive: true,
            hour: 1,
            minute: 0,
            second: None,
        };
        let mut buffer = Vec::new();
        let mut f = Formatter::new(&mut buffer);
        write_utc_offset(&mut f, offset).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "+0100");
    }
}
