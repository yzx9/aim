// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! `RRule` expansion and computation utilities.
//!
//! This module provides extension traits for expanding recurrence rules
//! and event occurrences according to RFC 5545.

#![cfg(feature = "jiff")]

use jiff::civil::{Date, DateTime, Time};

use crate::semantic::VEvent;
use crate::string_storage::StringStorage;
use crate::value::{RecurrenceFrequency, ValueDateTime, ValueRecurrenceRule, WeekDay};

/// Maximum number of occurrences to generate to prevent infinite loops.
const MAX_OCCURRENCES: usize = 10_000;

/// A date range for querying occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    /// Start of the range (inclusive)
    pub start: Date,
    /// End of the range (inclusive)
    pub end: Date,
}

impl DateRange {
    /// Creates a new date range.
    #[must_use]
    pub const fn new(start: Date, end: Date) -> Self {
        Self { start, end }
    }
}

/// An occurrence of an event.
#[derive(Debug, Clone)]
pub struct EventOccurrence<S: StringStorage> {
    /// Reference to the original event
    pub event: VEvent<S>,
    /// Start date/time of this occurrence
    pub start: DateTime,
    /// End date/time of this occurrence
    pub end: Option<DateTime>,
}

/// Extension trait for `RRule` expansion.
pub trait RRuleExt {
    /// Expands the recurrence rule within a given date range.
    ///
    /// Returns a list of dates representing the start times of each occurrence.
    ///
    /// # Errors
    ///
    /// Returns an error if the `RRule` cannot be expanded (e.g., invalid date arithmetic).
    fn expand(&self, start: DateTime, range: DateRange) -> Result<Vec<DateTime>, RRuleError>;
}

impl RRuleExt for ValueRecurrenceRule {
    fn expand(&self, start: DateTime, range: DateRange) -> Result<Vec<DateTime>, RRuleError> {
        let mut occurrences = Vec::new();

        // Get termination conditions
        let max_count = self.count.map_or(MAX_OCCURRENCES, |c| c as usize);
        let until_date = self.until.as_ref().map(value_datetime_to_date);

        // Generate occurrences based on frequency
        match self.freq {
            RecurrenceFrequency::Yearly => {
                self.expand_yearly(start, range, max_count, until_date, &mut occurrences)?;
            }
            RecurrenceFrequency::Monthly => {
                self.expand_monthly(start, range, max_count, until_date, &mut occurrences)?;
            }
            RecurrenceFrequency::Weekly => {
                self.expand_weekly(start, range, max_count, until_date, &mut occurrences)?;
            }
            RecurrenceFrequency::Daily => {
                self.expand_daily(start, range, max_count, until_date, &mut occurrences)?;
            }
            RecurrenceFrequency::Hourly
            | RecurrenceFrequency::Minutely
            | RecurrenceFrequency::Secondly => {
                // For sub-daily frequencies, use a simpler approach
                self.expand_sub_daily(start, range, max_count, until_date, &mut occurrences)?;
            }
        }

        Ok(occurrences)
    }
}

impl ValueRecurrenceRule {
    /// Expand YEARLY frequency.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn expand_yearly(
        &self,
        start: DateTime,
        range: DateRange,
        max_count: usize,
        until_date: Option<Date>,
        occurrences: &mut Vec<DateTime>,
    ) -> Result<(), RRuleError> {
        let interval = self.interval.unwrap_or(1) as i16;
        let base_time = start.time();
        let start_year = start.year();
        let start_day = start.day();

        let mut year_iter = (start_year..).step_by(interval as usize);

        while occurrences.len() < max_count {
            let year = year_iter.next().ok_or(RRuleError::TooManyOccurrences)?;

            // Generate candidates for this year
            let candidates =
                self.generate_yearly_candidates(year, base_time, start, Some(start_day));

            for dt in candidates {
                if occurrences.len() >= max_count {
                    break;
                }

                // Skip dates before start
                if dt < start {
                    continue;
                }

                // Check UNTIL condition
                if let Some(until) = &until_date
                    && dt.date() > *until
                {
                    return Ok(());
                }

                // Check if past range end
                if dt.date() > range.end {
                    return Ok(());
                }

                // Add if within range
                if dt.date() >= range.start {
                    occurrences.push(dt);
                }
            }

            // Stop if we've gone past the range end significantly
            if year > range.end.year() + 1 {
                break;
            }

            // Safety check
            if occurrences.len() >= MAX_OCCURRENCES {
                return Err(RRuleError::TooManyOccurrences);
            }
        }

        Ok(())
    }

    /// Expand MONTHLY frequency.
    #[allow(clippy::cast_possible_truncation)]
    fn expand_monthly(
        &self,
        start: DateTime,
        range: DateRange,
        max_count: usize,
        until_date: Option<Date>,
        occurrences: &mut Vec<DateTime>,
    ) -> Result<(), RRuleError> {
        let interval = self.interval.unwrap_or(1) as i16;
        let base_time = start.time();
        let start_day = start.day();

        let mut current = start;

        while occurrences.len() < max_count {
            // Generate candidates for this month
            // Pass start_day only if no BYMONTHDAY or BYDAY occurrence modifiers
            let use_start_day =
                self.by_month_day.is_empty() && !self.by_day.iter().any(|d| d.occurrence.is_some());
            let day_filter = if use_start_day { Some(start_day) } else { None };

            let candidates = self.generate_month_candidates(
                current.year(),
                current.month(),
                base_time,
                day_filter,
            );

            for dt in candidates {
                if occurrences.len() >= max_count {
                    break;
                }

                // Skip dates before start
                if dt < start {
                    continue;
                }

                // Check UNTIL condition
                if let Some(until) = &until_date
                    && dt.date() > *until
                {
                    return Ok(());
                }

                // Check if past range end
                if dt.date() > range.end {
                    return Ok(());
                }

                // Add if within range
                if dt.date() >= range.start {
                    occurrences.push(dt);
                }
            }

            // Advance to next month (applying interval)
            current = advance_months(current, interval)?;

            // Stop if we've gone past the range end
            if current.date() > range.end {
                break;
            }

            // Safety check
            if occurrences.len() >= MAX_OCCURRENCES {
                return Err(RRuleError::TooManyOccurrences);
            }
        }

        Ok(())
    }

    /// Expand WEEKLY frequency.
    fn expand_weekly(
        &self,
        start: DateTime,
        range: DateRange,
        max_count: usize,
        until_date: Option<Date>,
        occurrences: &mut Vec<DateTime>,
    ) -> Result<(), RRuleError> {
        let interval = self.interval.unwrap_or(1);
        let base_time = start.time();
        let wkst = self.wkst.unwrap_or(WeekDay::Monday);

        // Calculate the start of the week containing DTSTART
        let week_start = get_week_start(start.date(), wkst);

        let mut current_week = week_start;

        while occurrences.len() < max_count {
            // Generate candidates for this week
            let week_days = if self.by_day.is_empty() {
                // Use the day from DTSTART
                vec![start.date()]
            } else {
                self.get_week_day_dates(current_week)
            };

            for date in week_days {
                if occurrences.len() >= max_count {
                    break;
                }

                let dt = date.to_datetime(base_time);

                // Skip dates before start
                if dt < start {
                    continue;
                }

                // Check UNTIL condition
                if let Some(until) = &until_date
                    && dt.date() > *until
                {
                    return Ok(());
                }

                // Check if past range end
                if dt.date() > range.end {
                    return Ok(());
                }

                // Add if within range
                if dt.date() >= range.start {
                    occurrences.push(dt);
                }
            }

            // Advance to next week
            current_week = current_week
                .checked_add(jiff::Span::new().try_days(i64::from(interval) * 7).unwrap())
                .map_err(|e| RRuleError::DateArithmetic(e.to_string()))?;

            // Stop if we've gone past the range end
            if current_week > range.end {
                break;
            }

            // Safety check
            if occurrences.len() >= MAX_OCCURRENCES {
                return Err(RRuleError::TooManyOccurrences);
            }
        }

        Ok(())
    }

    /// Expand DAILY frequency.
    fn expand_daily(
        &self,
        start: DateTime,
        range: DateRange,
        max_count: usize,
        until_date: Option<Date>,
        occurrences: &mut Vec<DateTime>,
    ) -> Result<(), RRuleError> {
        let interval = self.interval.unwrap_or(1);
        let base_time = start.time();

        let mut current = start.date();

        while occurrences.len() < max_count {
            // Check UNTIL condition
            if let Some(until) = &until_date
                && current > *until
            {
                return Ok(());
            }

            // Check if past range end
            if current > range.end {
                return Ok(());
            }

            // Check BYDAY filter
            let matches_byday = self.by_day.is_empty() || self.day_matches_byday(current);

            // Check BYMONTH filter
            let matches_bymonth = self.by_month.is_empty()
                || self.by_month.contains(&current.month().cast_unsigned());

            // Check BYMONTHDAY filter
            let matches_bymonthday = if self.by_month_day.is_empty() {
                true
            } else {
                let days = days_in_month(current.year(), current.month());
                self.resolve_month_days_for_date(current.day(), days)
            };

            if matches_byday && matches_bymonth && matches_bymonthday && current >= range.start {
                occurrences.push(current.to_datetime(base_time));
            }

            // Advance by interval days
            current = current
                .checked_add(jiff::Span::new().try_days(i64::from(interval)).unwrap())
                .map_err(|e| RRuleError::DateArithmetic(e.to_string()))?;

            // Safety check
            if occurrences.len() >= MAX_OCCURRENCES {
                return Err(RRuleError::TooManyOccurrences);
            }
        }

        Ok(())
    }

    /// Expand sub-daily frequencies (HOURLY, MINUTELY, SECONDLY).
    fn expand_sub_daily(
        &self,
        start: DateTime,
        range: DateRange,
        max_count: usize,
        until_date: Option<Date>,
        occurrences: &mut Vec<DateTime>,
    ) -> Result<(), RRuleError> {
        let interval = self.interval.unwrap_or(1);
        let span = match self.freq {
            RecurrenceFrequency::Hourly => {
                jiff::Span::new().try_hours(i64::from(interval)).unwrap()
            }
            RecurrenceFrequency::Minutely => {
                jiff::Span::new().try_minutes(i64::from(interval)).unwrap()
            }
            RecurrenceFrequency::Secondly => {
                jiff::Span::new().try_seconds(i64::from(interval)).unwrap()
            }
            _ => unreachable!(),
        };

        let mut current = start;

        while occurrences.len() < max_count {
            // Check UNTIL condition
            if let Some(until) = &until_date
                && current.date() > *until
            {
                return Ok(());
            }

            // Check if past range end
            if current.date() > range.end {
                return Ok(());
            }

            // Add if within range
            if current.date() >= range.start {
                occurrences.push(current);
            }

            // Advance
            current = current
                .checked_add(span)
                .map_err(|e| RRuleError::DateArithmetic(e.to_string()))?;

            // Safety check
            if occurrences.len() >= MAX_OCCURRENCES {
                return Err(RRuleError::TooManyOccurrences);
            }
        }

        Ok(())
    }

    /// Generate candidates for YEARLY frequency.
    fn generate_yearly_candidates(
        &self,
        year: i16,
        base_time: Time,
        start: DateTime,
        start_day: Option<i8>,
    ) -> Vec<DateTime> {
        let mut candidates = Vec::new();

        // BYYEARDAY takes precedence if present
        if !self.by_year_day.is_empty() {
            return self.generate_by_year_day(year, base_time);
        }

        // BYWEEKNO takes precedence if present (YEARLY only)
        if !self.by_week_no.is_empty() {
            return self.generate_by_week_no(year, base_time);
        }

        // Determine which months to iterate
        // For YEARLY without BYMONTH, use only the month from DTSTART
        let months: Vec<i8> = if self.by_month.is_empty() {
            vec![start.month()]
        } else {
            self.by_month.iter().map(|&m| m.cast_signed()).collect()
        };

        for month in months {
            let month_candidates =
                self.generate_month_candidates(year, month, base_time, start_day);
            candidates.extend(month_candidates);
        }

        candidates
    }

    /// Generate candidates for a specific month.
    fn generate_month_candidates(
        &self,
        year: i16,
        month: i8,
        base_time: Time,
        start_day: Option<i8>,
    ) -> Vec<DateTime> {
        let mut candidates = Vec::new();

        // If BYDAY has occurrence modifiers (like 2MO, -1FR), handle specially
        if self.by_day.iter().any(|d| d.occurrence.is_some()) {
            return self.generate_byday_with_occurrence(year, month, base_time);
        }

        // Determine days in month
        let days = days_in_month(year, month);

        // Resolve BYMONTHDAY values (handle negative values)
        // If no BYMONTHDAY and no BYDAY, use the day from DTSTART (if provided)
        // If BYDAY is specified, generate all matching weekdays
        let month_days: Vec<i8> = if !self.by_month_day.is_empty() {
            self.resolve_month_days(year, month)
        } else if !self.by_day.is_empty() {
            // BYDAY is specified - generate all days, then filter by weekday
            (1..=days).collect()
        } else if let Some(day) = start_day {
            vec![day.min(days)]
        } else {
            (1..=days).collect()
        };

        // Generate candidates for each day
        for day in month_days {
            if let Ok(date) = Date::new(year, month, day) {
                // Apply BYDAY filter if present
                if !self.by_day.is_empty() && !self.day_matches_byday(date) {
                    continue;
                }

                let dt = date.to_datetime(base_time);
                candidates.push(dt);
            }
        }

        // Apply BYSETPOS filter
        self.apply_by_set_pos(candidates)
    }

    /// Generate candidates from BYYEARDAY.
    fn generate_by_year_day(&self, year: i16, base_time: Time) -> Vec<DateTime> {
        let mut result = Vec::new();
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };

        for &yearday in &self.by_year_day {
            let day_of_year = if yearday > 0 {
                yearday
            } else {
                // Negative: count from end of year
                days_in_year + yearday + 1
            };

            if day_of_year < 1 || day_of_year > days_in_year {
                continue;
            }

            if let Some(date) = date_from_year_day(year, day_of_year) {
                let dt = date.to_datetime(base_time);
                result.push(dt);
            }
        }

        result.sort();
        result.dedup();
        result
    }

    /// Generate candidates from BYWEEKNO.
    fn generate_by_week_no(&self, year: i16, base_time: Time) -> Vec<DateTime> {
        let wkst = self.wkst.unwrap_or(WeekDay::Monday);
        let mut result = Vec::new();

        for &weekno in &self.by_week_no {
            let target_week = if weekno > 0 {
                weekno
            } else {
                // Negative week number: count from end of year
                let weeks_in_year = weeks_in_year(year, wkst);
                weeks_in_year + weekno + 1
            };

            // Get the first day of the target week
            if let Some(week_start) = get_date_for_week(year, target_week, wkst) {
                // Generate days in this week
                for offset in 0i64..7 {
                    if let Ok(date) =
                        week_start.checked_add(jiff::Span::new().try_days(offset).unwrap())
                    {
                        // Apply BYDAY filter if present
                        if !self.by_day.is_empty() && !self.day_matches_byday(date) {
                            continue;
                        }

                        let dt = date.to_datetime(base_time);
                        result.push(dt);
                    }
                }
            }
        }

        result.sort();
        result.dedup();
        result
    }

    /// Resolve BYMONTHDAY values, handling negative indices.
    fn resolve_month_days(&self, year: i16, month: i8) -> Vec<i8> {
        let days = days_in_month(year, month);
        let mut result = Vec::new();

        for &day in &self.by_month_day {
            let resolved = if day > 0 {
                if day <= days { Some(day) } else { None }
            } else {
                // Negative: -1 = last day, -2 = second to last, etc.
                let from_end = -day;
                if from_end <= days {
                    Some(days - from_end + 1)
                } else {
                    None
                }
            };
            if let Some(d) = resolved {
                result.push(d);
            }
        }

        result
    }

    /// Check if a specific day matches the BYMONTHDAY list.
    fn resolve_month_days_for_date(&self, day: i8, days_in_month: i8) -> bool {
        for &md in &self.by_month_day {
            let resolved = if md > 0 {
                md
            } else {
                // Negative: -1 = last day
                days_in_month + md + 1
            };
            if resolved == day {
                return true;
            }
        }
        false
    }

    /// Check if a date matches the BYDAY constraint.
    fn day_matches_byday(&self, date: Date) -> bool {
        let weekday = date.weekday();
        for byday in &self.by_day {
            // Only check weekday, not occurrence (that's handled separately)
            if byday.occurrence.is_none() && weekday_to_weekday(byday.day) == weekday {
                return true;
            }
        }
        false
    }

    /// Generate candidates for BYDAY with occurrence modifiers.
    fn generate_byday_with_occurrence(
        &self,
        year: i16,
        month: i8,
        base_time: Time,
    ) -> Vec<DateTime> {
        let mut candidates = Vec::new();

        for byday in &self.by_day {
            if let Some(occ) = byday.occurrence {
                let weekday = weekday_to_weekday(byday.day);
                let dates = get_nth_weekday_of_month(year, month, weekday, occ);
                for date in dates {
                    let dt = date.to_datetime(base_time);
                    candidates.push(dt);
                }
            } else {
                // No occurrence modifier - add all matching weekdays in month
                let days = days_in_month(year, month);
                for day in 1..=days {
                    if let Ok(date) = Date::new(year, month, day)
                        && date.weekday() == weekday_to_weekday(byday.day)
                    {
                        let dt = date.to_datetime(base_time);
                        candidates.push(dt);
                    }
                }
            }
        }

        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Get dates for specific weekdays within a week.
    fn get_week_day_dates(&self, week_start: Date) -> Vec<Date> {
        let mut dates = Vec::new();

        for offset in 0i64..7 {
            if let Ok(date) = week_start.checked_add(jiff::Span::new().try_days(offset).unwrap()) {
                let weekday = date.weekday();

                if self.by_day.is_empty() {
                    dates.push(date);
                } else {
                    for byday in &self.by_day {
                        if byday.occurrence.is_none() && weekday_to_weekday(byday.day) == weekday {
                            dates.push(date);
                            break;
                        }
                    }
                }
            }
        }

        dates
    }

    /// Apply BYSETPOS filter to candidates.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn apply_by_set_pos(&self, mut candidates: Vec<DateTime>) -> Vec<DateTime> {
        if self.by_set_pos.is_empty() {
            candidates.sort();
            candidates.dedup();
            return candidates;
        }

        candidates.sort();
        candidates.dedup();

        let len = candidates.len() as i16;
        let mut result = Vec::new();

        for &pos in &self.by_set_pos {
            let index = if pos > 0 {
                (pos - 1) as usize
            } else {
                // Negative index from end
                (len + pos) as usize
            };

            if let Some(&dt) = candidates.get(index) {
                result.push(dt);
            }
        }

        result.sort();
        result.dedup();
        result
    }
}

/// Extension trait for event expansion.
pub trait VEventExt<S: StringStorage> {
    /// Expands an event with its recurrence rule into individual occurrences.
    ///
    /// If the event has no recurrence rule, returns a single occurrence.
    ///
    /// # Errors
    ///
    /// Returns an error if the `RRule` cannot be expanded.
    fn expand_occurrences(&self, range: DateRange) -> Result<Vec<EventOccurrence<S>>, RRuleError>;
}

impl<S: StringStorage> VEventExt<S> for VEvent<S> {
    fn expand_occurrences(&self, range: DateRange) -> Result<Vec<EventOccurrence<S>>, RRuleError> {
        // Get start datetime
        let start = self
            .dt_start
            .civil_date_time()
            .ok_or_else(|| RRuleError::InvalidRule("DTSTART must be a date-time".to_string()))?;

        // Get occurrences from RRule or just the start time
        let occurrences = if let Some(rrule) = &self.rrule {
            rrule.value.expand(start, range)?
        } else if start.date() >= range.start && start.date() <= range.end {
            vec![start]
        } else {
            vec![]
        };

        // Calculate duration for end times
        let duration = self.calculate_duration();

        // Build event occurrences
        Ok(occurrences
            .into_iter()
            .map(|start| EventOccurrence {
                event: self.clone(),
                end: duration.and_then(|d| start.checked_add(d).ok()),
                start,
            })
            .collect())
    }
}

impl<S: StringStorage> VEvent<S> {
    /// Calculate the duration of the event.
    fn calculate_duration(&self) -> Option<jiff::Span> {
        if let Some(ref dt_end) = self.dt_end
            && let (Some(start), Some(end)) =
                (self.dt_start.civil_date_time(), dt_end.civil_date_time())
        {
            return end.since(start).ok();
        }

        if let Some(ref duration) = self.duration {
            return value_duration_to_span(&duration.value);
        }

        None
    }
}

/// Convert `ValueDateTime` to `Date`.
fn value_datetime_to_date(dt: &ValueDateTime) -> Date {
    jiff::civil::date(dt.date.year, dt.date.month, dt.date.day)
}

/// Convert `ValueDuration` to `jiff::Span`.
fn value_duration_to_span(duration: &crate::value::ValueDuration) -> Option<jiff::Span> {
    use crate::value::ValueDuration as VDur;

    match duration {
        VDur::DateTime {
            positive,
            day,
            hour,
            minute,
            second,
        } => {
            let span = jiff::Span::new()
                .try_days(i64::from(*day))
                .ok()?
                .try_hours(i64::from(*hour))
                .ok()?
                .try_minutes(i64::from(*minute))
                .ok()?
                .try_seconds(i64::from(*second))
                .ok()?;

            Some(if *positive { span } else { span.negate() })
        }
        VDur::Week { positive, week } => {
            let span = jiff::Span::new().try_weeks(i64::from(*week)).ok()?;
            Some(if *positive { span } else { span.negate() })
        }
    }
}

/// Convert `WeekDay` to `jiff::civil::Weekday`.
fn weekday_to_weekday(day: WeekDay) -> jiff::civil::Weekday {
    match day {
        WeekDay::Sunday => jiff::civil::Weekday::Sunday,
        WeekDay::Monday => jiff::civil::Weekday::Monday,
        WeekDay::Tuesday => jiff::civil::Weekday::Tuesday,
        WeekDay::Wednesday => jiff::civil::Weekday::Wednesday,
        WeekDay::Thursday => jiff::civil::Weekday::Thursday,
        WeekDay::Friday => jiff::civil::Weekday::Friday,
        WeekDay::Saturday => jiff::civil::Weekday::Saturday,
    }
}

/// Advance a datetime by a number of months.
#[allow(clippy::cast_possible_truncation)]
fn advance_months(dt: DateTime, months: i16) -> Result<DateTime, RRuleError> {
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    let time = dt.time();

    // Calculate new year and month
    let total_months = (year * 12) + i16::from(month) - 1 + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12) + 1;

    // Handle day overflow (e.g., Jan 31 + 1 month = Feb 28/29)
    let max_day = days_in_month(new_year, new_month as i8);
    let new_day = day.min(max_day);

    DateTime::new(
        new_year,
        new_month as i8,
        new_day,
        time.hour(),
        time.minute(),
        time.second(),
        i32::from(time.millisecond()),
    )
    .map_err(|e| RRuleError::DateArithmetic(e.to_string()))
}

/// Get the start of the week containing a date.
fn get_week_start(date: Date, wkst: WeekDay) -> Date {
    let weekday = date.weekday();
    let wkst_weekday = weekday_to_weekday(wkst);

    // Calculate days since week start (0-6)
    let weekday_num = i64::from(weekday.to_monday_one_offset());
    let wkst_num = i64::from(wkst_weekday.to_monday_one_offset());

    let days_since_week_start = (weekday_num - wkst_num + 7) % 7;

    date.checked_sub(jiff::Span::new().try_days(days_since_week_start).unwrap())
        .unwrap_or(date)
}

/// Get the number of weeks in a year.
#[allow(clippy::cast_possible_truncation)]
fn weeks_in_year(year: i16, wkst: WeekDay) -> i8 {
    let jan1 = jiff::civil::date(year, 1, 1);
    let dec31 = jiff::civil::date(year, 12, 31);

    let first_week_start = get_week_start(jan1, wkst);
    let last_week_start = get_week_start(dec31, wkst);

    let days = (last_week_start - first_week_start).get_days();
    ((days / 7) + 1) as i8
}

/// Get the date for a specific week number in a year.
fn get_date_for_week(year: i16, week: i8, wkst: WeekDay) -> Option<Date> {
    let jan1 = jiff::civil::date(year, 1, 1);
    let first_week_start = get_week_start(jan1, wkst);

    first_week_start
        .checked_add(jiff::Span::new().try_weeks(i64::from(week - 1)).unwrap())
        .ok()
}

/// Get the date from a year and day-of-year.
#[allow(clippy::cast_possible_truncation)]
fn date_from_year_day(year: i16, day_of_year: i16) -> Option<Date> {
    let is_leap = is_leap_year(year);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining_days = day_of_year;
    for (month, &days) in days_in_months.iter().enumerate() {
        if remaining_days <= days as i16 {
            return Date::new(year, (month + 1) as i8, remaining_days as i8).ok();
        }
        remaining_days -= days as i16;
    }

    None
}

/// Get the nth occurrence of a weekday in a month.
#[allow(clippy::cast_sign_loss)]
fn get_nth_weekday_of_month(
    year: i16,
    month: i8,
    weekday: jiff::civil::Weekday,
    n: i8,
) -> Vec<Date> {
    let days = days_in_month(year, month);

    // Find all occurrences of the weekday in the month
    let mut occurrences = Vec::new();
    for day in 1..=days {
        if let Ok(date) = Date::new(year, month, day)
            && date.weekday() == weekday
        {
            occurrences.push(date);
        }
    }

    if occurrences.is_empty() {
        return Vec::new();
    }

    // Get the nth occurrence (1-indexed) or -n from end
    let index = if n > 0 {
        (n as usize).checked_sub(1)
    } else {
        // Negative: count from end
        let from_end = (-n) as usize;
        if from_end <= occurrences.len() {
            Some(occurrences.len() - from_end)
        } else {
            None
        }
    };

    index
        .and_then(|i| occurrences.get(i).copied())
        .map(|date| vec![date])
        .unwrap_or_default()
}

/// Get the number of days in a month.
fn days_in_month(year: i16, month: i8) -> i8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30, // April, June, September, November
    }
}

/// Check if a year is a leap year.
fn is_leap_year(year: i16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Error type for `RRule` operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RRuleError {
    /// The recurrence rule is invalid
    #[error("Invalid recurrence rule: {0}")]
    InvalidRule(String),
    /// Date arithmetic failed
    #[error("Date arithmetic error: {0}")]
    DateArithmetic(String),
    /// The expansion would generate too many occurrences
    #[error("Too many occurrences generated")]
    TooManyOccurrences,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{RecurrenceFrequency, WeekDay, WeekDayNum};

    fn create_rrule(freq: RecurrenceFrequency) -> ValueRecurrenceRule {
        ValueRecurrenceRule {
            freq,
            until: None,
            count: None,
            interval: None,
            by_second: Vec::new(),
            by_minute: Vec::new(),
            by_hour: Vec::new(),
            by_month_day: Vec::new(),
            by_year_day: Vec::new(),
            by_week_no: Vec::new(),
            by_month: Vec::new(),
            by_day: Vec::new(),
            by_set_pos: Vec::new(),
            wkst: None,
        }
    }

    fn create_date(year: i16, month: i8, day: i8) -> Date {
        jiff::civil::date(year, month, day)
    }

    fn create_datetime(
        year: i16,
        month: i8,
        day: i8,
        hour: i8,
        minute: i8,
        second: i8,
    ) -> DateTime {
        DateTime::new(year, month, day, hour, minute, second, 0).unwrap()
    }

    #[test]
    fn rrule_expand_daily_simple() {
        let rrule = create_rrule(RecurrenceFrequency::Daily);
        let start = create_datetime(2024, 1, 1, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 1, 5));

        let result = rrule.expand(start, range).unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(*result.first().unwrap(), start);
        assert_eq!(
            *result.get(4).unwrap(),
            create_datetime(2024, 1, 5, 10, 0, 0)
        );
    }

    #[test]
    fn rrule_expand_daily_with_interval() {
        let mut rrule = create_rrule(RecurrenceFrequency::Daily);
        rrule.interval = Some(2);
        let start = create_datetime(2024, 1, 1, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 1, 10));

        let result = rrule.expand(start, range).unwrap();

        assert_eq!(result.len(), 5); // 1, 3, 5, 7, 9
        assert_eq!(*result.first().unwrap(), start);
        assert_eq!(
            *result.get(1).unwrap(),
            create_datetime(2024, 1, 3, 10, 0, 0)
        );
    }

    #[test]
    fn rrule_expand_daily_with_count() {
        let mut rrule = create_rrule(RecurrenceFrequency::Daily);
        rrule.count = Some(3);
        let start = create_datetime(2024, 1, 1, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 12, 31));

        let result = rrule.expand(start, range).unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn rrule_expand_weekly_simple() {
        let rrule = create_rrule(RecurrenceFrequency::Weekly);
        let start = create_datetime(2024, 1, 1, 10, 0, 0); // Monday
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 1, 22));

        let result = rrule.expand(start, range).unwrap();

        // Should get 4 Mondays: Jan 1, 8, 15, 22
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn rrule_expand_weekly_with_byday() {
        let mut rrule = create_rrule(RecurrenceFrequency::Weekly);
        rrule.by_day = vec![
            WeekDayNum {
                day: WeekDay::Monday,
                occurrence: None,
            },
            WeekDayNum {
                day: WeekDay::Friday,
                occurrence: None,
            },
        ];
        let start = create_datetime(2024, 1, 1, 10, 0, 0); // Monday
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 1, 15));

        let result = rrule.expand(start, range).unwrap();

        // Week 1: Mon Jan 1, Fri Jan 5
        // Week 2: Mon Jan 8, Fri Jan 12
        // Week 3: Mon Jan 15 (Fri Jan 19 is past range)
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn rrule_expand_monthly_simple() {
        let rrule = create_rrule(RecurrenceFrequency::Monthly);
        let start = create_datetime(2024, 1, 15, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 6, 30));

        let result = rrule.expand(start, range).unwrap();

        assert_eq!(result.len(), 6);
    }

    #[test]
    fn rrule_expand_monthly_with_bymonthday() {
        let mut rrule = create_rrule(RecurrenceFrequency::Monthly);
        rrule.by_month_day = vec![15, -1]; // 15th and last day
        let start = create_datetime(2024, 1, 15, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 3, 31));

        let result = rrule.expand(start, range).unwrap();

        // Jan 15, Jan 31, Feb 15, Feb 29 (leap year), Mar 15, Mar 31
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn rrule_expand_yearly_simple() {
        let rrule = create_rrule(RecurrenceFrequency::Yearly);
        let start = create_datetime(2024, 6, 15, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2027, 12, 31));

        let result = rrule.expand(start, range).unwrap();

        assert_eq!(result.len(), 4); // 2024, 2025, 2026, 2027
    }

    #[test]
    fn rrule_expand_with_until() {
        let mut rrule = create_rrule(RecurrenceFrequency::Daily);
        let until = ValueDateTime::new(
            crate::value::ValueDate::new(2024, 1, 5).unwrap(),
            crate::value::ValueTime::new(23, 59, 59, false).unwrap(),
        );
        rrule.until = Some(until);
        let start = create_datetime(2024, 1, 1, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 12, 31));

        let result = rrule.expand(start, range).unwrap();

        // Should stop at Jan 5
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn rrule_expand_with_byday_occurrence() {
        let mut rrule = create_rrule(RecurrenceFrequency::Monthly);
        rrule.by_day = vec![WeekDayNum {
            day: WeekDay::Friday,
            occurrence: Some(-1), // Last Friday
        }];
        let start = create_datetime(2024, 1, 26, 10, 0, 0); // Last Friday of Jan 2024
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 3, 31));

        let result = rrule.expand(start, range).unwrap();

        // Should get last Friday of Jan, Feb, Mar
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn rrule_expand_with_bysetpos() {
        let mut rrule = create_rrule(RecurrenceFrequency::Monthly);
        rrule.by_day = vec![
            WeekDayNum {
                day: WeekDay::Monday,
                occurrence: None,
            },
            WeekDayNum {
                day: WeekDay::Wednesday,
                occurrence: None,
            },
            WeekDayNum {
                day: WeekDay::Friday,
                occurrence: None,
            },
        ];
        rrule.by_set_pos = vec![-1]; // Last occurrence
        let start = create_datetime(2024, 1, 1, 10, 0, 0);
        let range = DateRange::new(create_date(2024, 1, 1), create_date(2024, 2, 29));

        let result = rrule.expand(start, range).unwrap();

        // Should get last Mon/Wed/Fri of each month
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn days_in_month_february_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn is_leap_year_test() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }
}
