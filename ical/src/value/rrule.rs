// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Recurrence rule type definitions for iCalendar.

use std::fmt::{self, Display};

use chumsky::extra::ParserExtra;
use chumsky::input::Input;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::keyword::{
    KW_DAY_FR, KW_DAY_MO, KW_DAY_SA, KW_DAY_SU, KW_DAY_TH, KW_DAY_TU, KW_DAY_WE, KW_RRULE_BYDAY,
    KW_RRULE_BYHOUR, KW_RRULE_BYMINUTE, KW_RRULE_BYMONTH, KW_RRULE_BYMONTHDAY, KW_RRULE_BYSECOND,
    KW_RRULE_BYSETPOS, KW_RRULE_BYWEEKNO, KW_RRULE_BYYEARDAY, KW_RRULE_COUNT, KW_RRULE_FREQ,
    KW_RRULE_FREQ_DAILY, KW_RRULE_FREQ_HOURLY, KW_RRULE_FREQ_MINUTELY, KW_RRULE_FREQ_MONTHLY,
    KW_RRULE_FREQ_SECONDLY, KW_RRULE_FREQ_WEEKLY, KW_RRULE_FREQ_YEARLY, KW_RRULE_INTERVAL,
    KW_RRULE_UNTIL, KW_RRULE_WKST,
};
use crate::value::datetime::{ValueDateTime, ValueTime, value_date, value_date_time};
use crate::value::miscellaneous::{
    ValueExpected, i8_0_1, i8_0_3, i8_0_9, i8_1_2, i8_1_4, i8_1_9, i16_0_5, i16_0_6, i16_0_9,
    i16_1_2, i16_1_9, u8_0_1, u8_0_3, u8_0_5, u8_0_9, u8_1_9,
};

/// Recurrence rule
#[derive(Debug, Clone)]
pub struct ValueRecurrenceRule {
    /// Frequency of recurrence
    pub freq: RecurrenceFrequency,
    /// Until date for recurrence
    pub until: Option<ValueDateTime>,
    /// Number of occurrences
    pub count: Option<u32>,
    /// Interval between recurrences
    pub interval: Option<u32>,
    /// Second specifier
    pub by_second: Vec<u8>,
    /// Minute specifier
    pub by_minute: Vec<u8>,
    /// Hour specifier
    pub by_hour: Vec<u8>,
    /// Day of month specifier
    pub by_month_day: Vec<i8>,
    /// Day of year specifier
    pub by_year_day: Vec<i16>,
    /// Week number specifier
    pub by_week_no: Vec<i8>,
    /// Month specifier
    pub by_month: Vec<u8>,
    /// Day of week specifier
    pub by_day: Vec<WeekDayNum>,
    /// Position in month
    pub by_set_pos: Vec<i16>,
    /// Start day of week
    pub wkst: Option<WeekDay>,
}

/// Recurrence frequency
#[derive(Debug, Clone, Copy, PartialEq)]
#[expect(missing_docs)]
pub enum RecurrenceFrequency {
    Secondly,
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Display for RecurrenceFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecurrenceFrequency::Secondly => write!(f, "{KW_RRULE_FREQ_SECONDLY}"),
            RecurrenceFrequency::Minutely => write!(f, "{KW_RRULE_FREQ_MINUTELY}"),
            RecurrenceFrequency::Hourly => write!(f, "{KW_RRULE_FREQ_HOURLY}"),
            RecurrenceFrequency::Daily => write!(f, "{KW_RRULE_FREQ_DAILY}"),
            RecurrenceFrequency::Weekly => write!(f, "{KW_RRULE_FREQ_WEEKLY}"),
            RecurrenceFrequency::Monthly => write!(f, "{KW_RRULE_FREQ_MONTHLY}"),
            RecurrenceFrequency::Yearly => write!(f, "{KW_RRULE_FREQ_YEARLY}"),
        }
    }
}

/// Day of week with optional occurrence
#[derive(Debug, Clone, Copy)]
pub struct WeekDayNum {
    /// Day of the week
    pub day: WeekDay,
    /// Occurrence in month (optional)
    pub occurrence: Option<i8>,
}

/// Day of the week
#[derive(Debug, Clone, Copy, PartialEq)]
#[expect(missing_docs)]
pub enum WeekDay {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Display for WeekDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeekDay::Sunday => write!(f, "{KW_DAY_SU}"),
            WeekDay::Monday => write!(f, "{KW_DAY_MO}"),
            WeekDay::Tuesday => write!(f, "{KW_DAY_TU}"),
            WeekDay::Wednesday => write!(f, "{KW_DAY_WE}"),
            WeekDay::Thursday => write!(f, "{KW_DAY_TH}"),
            WeekDay::Friday => write!(f, "{KW_DAY_FR}"),
            WeekDay::Saturday => write!(f, "{KW_DAY_SA}"),
        }
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// recur           = recur-rule-part *( ";" recur-rule-part )
///                 ;
///                 ; The rule parts are not ordered in any
///                 ; particular sequence.
///                 ;
///                 ; The FREQ rule part is REQUIRED,
///                 ; but MUST NOT occur more than once.
///                 ;
///                 ; The UNTIL or COUNT rule parts are OPTIONAL,
///                 ; but they MUST NOT occur in the same 'recur'.
///                 ;
///                 ; The other rule parts are OPTIONAL,
///                 ; but MUST NOT occur more than once.
/// ```
pub fn value_rrule<'src, I, E>() -> impl Parser<'src, I, ValueRecurrenceRule, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    recur_rrule_part()
        .separated_by(just(';'))
        .at_least(1)
        .collect()
        .try_map(build_from_parts::<I, E::Error>)
}

#[expect(clippy::too_many_lines)]
fn build_from_parts<'src, I, Err>(
    parts: Vec<Part>,
    span: I::Span,
) -> Result<ValueRecurrenceRule, Err>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    Err: LabelError<'src, I, ValueExpected>,
{
    let mut freq = None;
    let mut until = None;
    let mut count = None;
    let mut interval = None;
    let mut by_second = Vec::new();
    let mut by_minute = Vec::new();
    let mut by_hour = Vec::new();
    let mut by_month_day = Vec::new();
    let mut by_year_day = Vec::new();
    let mut by_week_no = Vec::new();
    let mut by_month = Vec::new();
    let mut by_day = Vec::new();
    let mut by_set_pos = Vec::new();
    let mut wkst = None;

    for part in parts {
        match part {
            Part::Freq(f) => match freq {
                Some(_) => {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                None => freq = Some(f),
            },
            Part::Until(u) => {
                if until.is_some() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                until = Some(u);
            }
            Part::Count(c) => {
                if count.is_some() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                count = Some(c);
            }
            Part::Interval(i) => {
                if interval.is_some() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                interval = Some(i);
            }
            Part::BySecond(v) => {
                if !by_second.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_second = v;
            }
            Part::ByMinute(v) => {
                if !by_minute.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_minute = v;
            }
            Part::ByHour(v) => {
                if !by_hour.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_hour = v;
            }
            Part::ByMonthDay(v) => {
                if !by_month_day.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_month_day = v;
            }
            Part::ByYearDay(v) => {
                if !by_year_day.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_year_day = v;
            }
            Part::ByWeekNo(v) => {
                if !by_week_no.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_week_no = v;
            }
            Part::ByMonth(v) => {
                if !by_month.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_month = v;
            }
            Part::ByDay(v) => {
                if !by_day.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_day = v;
            }
            Part::BySetPos(v) => {
                if !by_set_pos.is_empty() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                by_set_pos = v;
            }
            Part::Wkst(w) => {
                if wkst.is_some() {
                    return Err(Err::expected_found(
                        [ValueExpected::RRuleDuplicatePart],
                        None,
                        span,
                    ));
                }
                wkst = Some(w);
            }
        }
    }

    // Validate required FREQ
    let freq =
        freq.ok_or_else(|| Err::expected_found([ValueExpected::RRuleRequiredFreq], None, span))?;

    // Validate UNTIL and COUNT are mutually exclusive
    if until.is_some() && count.is_some() {
        return Err(Err::expected_found(
            [ValueExpected::RRuleCountUntilExclusion],
            None,
            span,
        ));
    }

    Ok(ValueRecurrenceRule {
        freq,
        until,
        count,
        interval,
        by_second,
        by_minute,
        by_hour,
        by_month_day,
        by_year_day,
        by_week_no,
        by_month,
        by_day,
        by_set_pos,
        wkst,
    })
}

#[derive(Debug, Clone)]
enum Part {
    Freq(RecurrenceFrequency),
    Until(ValueDateTime),
    Count(u32),
    Interval(u32),
    BySecond(Vec<u8>),
    ByMinute(Vec<u8>),
    ByHour(Vec<u8>),
    ByMonthDay(Vec<i8>),
    ByYearDay(Vec<i16>),
    ByWeekNo(Vec<i8>),
    ByMonth(Vec<u8>),
    ByDay(Vec<WeekDayNum>),
    BySetPos(Vec<i16>),
    Wkst(WeekDay),
}

/// ```txt
/// recur-rule-part = ( "FREQ" "=" freq )
///                 / ( "UNTIL" "=" enddate )
///                 / ( "COUNT" "=" 1*DIGIT )
///                 / ( "INTERVAL" "=" 1*DIGIT )
///                 / ( "BYSECOND" "=" byseclist )
///                 / ( "BYMINUTE" "=" byminlist )
///                 / ( "BYHOUR" "=" byhrlist )
///                 / ( "BYDAY" "=" bywdaylist )
///                 / ( "BYMONTHDAY" "=" bymodaylist )
///                 / ( "BYYEARDAY" "=" byyrdaylist )
///                 / ( "BYWEEKNO" "=" bywknolist )
///                 / ( "BYMONTH" "=" bymolist )
///                 / ( "BYSETPOS" "=" bysplist )
///                 / ( "WKST" "=" weekday )
/// ```
fn recur_rrule_part<'src, I, E>() -> impl Parser<'src, I, Part, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    let kw = |kw| just(kw).ignore_then(just('='));

    // Frequency parser
    let freq = kw(KW_RRULE_FREQ).ignore_then(freq()).map(Part::Freq);

    // UNTIL can be a date or date-time
    let until = kw(KW_RRULE_UNTIL).ignore_then(enddate()).map(Part::Until);

    // COUNT - positive integer
    let count = kw(KW_RRULE_COUNT)
        .ignore_then(u32_non_zero())
        .map(Part::Count);

    // INTERVAL - positive integer
    let interval = kw(KW_RRULE_INTERVAL)
        .ignore_then(u32_non_zero())
        .map(Part::Interval);

    // BYSECOND - 0 to 60
    let by_second = kw(KW_RRULE_BYSECOND)
        .ignore_then(byseclist())
        .map(Part::BySecond);

    // BYMINUTE - 0 to 59
    let by_minute = kw(KW_RRULE_BYMINUTE)
        .ignore_then(byminlist())
        .map(Part::ByMinute);

    // BYHOUR - 0 to 23
    let by_hour = kw(KW_RRULE_BYHOUR)
        .ignore_then(byhrlist())
        .map(Part::ByHour);

    // BYDAY - weekday with optional occurrence
    let by_day = kw(KW_RRULE_BYDAY)
        .ignore_then(bywdaylist())
        .map(Part::ByDay);

    // BYMONTHDAY - -31 to -1 and 1 to 31
    let by_month_day = kw(KW_RRULE_BYMONTHDAY)
        .ignore_then(bymodaylist())
        .map(Part::ByMonthDay);

    // BYYEARDAY - -366 to -1 and 1 to 366
    let by_year_day = kw(KW_RRULE_BYYEARDAY)
        .ignore_then(byyrdaylist())
        .map(Part::ByYearDay);

    // BYWEEKNO - -53 to -1 and 1 to 53
    let by_week_no = kw(KW_RRULE_BYWEEKNO)
        .ignore_then(bywknolist())
        .map(Part::ByWeekNo);

    // BYMONTH - 1 to 12
    let by_month = kw(KW_RRULE_BYMONTH)
        .ignore_then(bymolist())
        .map(Part::ByMonth);

    // BYSETPOS - -366 to -1 and 1 to 366
    let by_set_pos = kw(KW_RRULE_BYSETPOS)
        .ignore_then(bysplist())
        .map(Part::BySetPos);

    // WKST - single weekday
    let wkst = kw(KW_RRULE_WKST).ignore_then(weekday()).map(Part::Wkst);

    choice((
        freq,
        until,
        count,
        interval,
        by_second,
        by_minute,
        by_hour,
        by_day,
        by_month_day,
        by_year_day,
        by_week_no,
        by_month,
        by_set_pos,
        wkst,
    ))
}

/// ```txt
/// freq        = "SECONDLY" / "MINUTELY" / "HOURLY" / "DAILY"
///             / "WEEKLY" / "MONTHLY" / "YEARLY"
/// ```
fn freq<'src, I, E>() -> impl Parser<'src, I, RecurrenceFrequency, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        just(KW_RRULE_FREQ_SECONDLY).to(RecurrenceFrequency::Secondly),
        just(KW_RRULE_FREQ_MINUTELY).to(RecurrenceFrequency::Minutely),
        just(KW_RRULE_FREQ_HOURLY).to(RecurrenceFrequency::Hourly),
        just(KW_RRULE_FREQ_DAILY).to(RecurrenceFrequency::Daily),
        just(KW_RRULE_FREQ_WEEKLY).to(RecurrenceFrequency::Weekly),
        just(KW_RRULE_FREQ_MONTHLY).to(RecurrenceFrequency::Monthly),
        just(KW_RRULE_FREQ_YEARLY).to(RecurrenceFrequency::Yearly),
    ))
}

/// ```txt
/// enddate     = date / date-time
/// ```
// TODO: According to RFC 5545, the UNTIL value MUST be a date or date-time
// that matches the type of the DTSTART property.
fn enddate<'src, I, E>() -> impl Parser<'src, I, ValueDateTime, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    // Try date-time first, then fall back to date
    // PERF: Could be optimized to avoid backtracking
    choice((
        value_date_time(),
        value_date().try_map(|date, span| {
            // Should always succeed as date is already validated
            let time = ValueTime::new(0, 0, 0, false)
                .map_err(|_| E::Error::expected_found([ValueExpected::Time], None, span))?;
            Ok(ValueDateTime::new(date, time))
        }),
    ))
}

/// ```txt
/// byseclist   = ( seconds *("," seconds) )
/// ```
fn byseclist<'src, I, E>() -> impl Parser<'src, I, Vec<u8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    seconds().separated_by(just(',')).collect()
}

/// ```txt
/// seconds     = 1*2DIGIT       ;0 to 60
/// ```
fn seconds<'src, I, E>() -> impl Parser<'src, I, u8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        u8_0_5().then(u8_0_9()).map(|(a, b)| a * 10 + b), // 00-59
        just("60").to(60),                                // 60
        u8_0_9(),                                         // 0-9
    ))
}

/// ```txt
/// byminlist   = ( minutes *("," minutes) )
/// ```
fn byminlist<'src, I, E>() -> impl Parser<'src, I, Vec<u8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    minutes().separated_by(just(',')).collect()
}

/// ```txt
/// minutes     = 1*2DIGIT       ;0 to 59
/// ```
fn minutes<'src, I, E>() -> impl Parser<'src, I, u8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        u8_0_5().then(u8_0_9()).map(|(a, b)| a * 10 + b), // 00-59
        u8_0_9(),                                         // 0-9
    ))
}

/// ```txt
/// byhrlist    = ( hour *("," hour) )
/// ```
fn byhrlist<'src, I, E>() -> impl Parser<'src, I, Vec<u8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    hour().separated_by(just(',')).collect()
}

/// ```txt
/// hour        = 1*2DIGIT       ;0 to 23
/// ```
fn hour<'src, I, E>() -> impl Parser<'src, I, u8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        u8_0_1().then(u8_0_9()).map(|(a, b)| a * 10 + b), // 00-19
        just('2').ignore_then(u8_0_3()).map(|b| 20 + b),  // 20-23
        u8_0_9(),                                         // 0-9
    ))
}

/// ```txt
/// bywdaylist  = ( weekdaynum *("," weekdaynum) )
/// ```
fn bywdaylist<'src, I, E>() -> impl Parser<'src, I, Vec<WeekDayNum>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    weekdaynum().separated_by(just(',')).collect()
}

/// ```txt
/// weekdaynum  = [[plus / minus] ordwk] weekday
/// plus        = "+"
/// minus       = "-"
/// ```
fn weekdaynum<'src, I, E>() -> impl Parser<'src, I, WeekDayNum, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    is_positive()
        .then(ordwk())
        .map(|(positive, n)| if positive { n } else { -n })
        .or_not()
        .then(weekday())
        .map(|(occurrence, day)| WeekDayNum { day, occurrence })
}

/// ```txt
/// ordwk       = 1*2DIGIT       ;1 to 53
/// ```
fn ordwk<'src, I, E>() -> impl Parser<'src, I, i8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        i8_1_4().then(i8_0_9()).map(|(a, b)| a * 10 + b), // 10-49
        just('5').ignore_then(i8_0_3()).map(|a| 50 + a),  // 50-53
        just('0').ignore_then(i8_1_9()),                  // 01-09
        i8_1_9(),                                         // 1-9
    ))
}

/// ```txt
/// weekday     = "SU" / "MO" / "TU" / "WE" / "TH" / "FR" / "SA"
/// ```
fn weekday<'src, I, E>() -> impl Parser<'src, I, WeekDay, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        just(KW_DAY_SU).to(WeekDay::Sunday),
        just(KW_DAY_MO).to(WeekDay::Monday),
        just(KW_DAY_TU).to(WeekDay::Tuesday),
        just(KW_DAY_WE).to(WeekDay::Wednesday),
        just(KW_DAY_TH).to(WeekDay::Thursday),
        just(KW_DAY_FR).to(WeekDay::Friday),
        just(KW_DAY_SA).to(WeekDay::Saturday),
    ))
}

/// ```txt
/// bymodaylist = ( monthdaynum *("," monthdaynum) )
/// ```
fn bymodaylist<'src, I, E>() -> impl Parser<'src, I, Vec<i8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    monthdaynum().separated_by(just(',')).collect()
}

/// ```txt
/// monthdaynum = [plus / minus] ordmoday
/// ```
fn monthdaynum<'src, I, E>() -> impl Parser<'src, I, i8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    is_positive()
        .then(ordmoday())
        .map(|(positive, n)| if positive { n } else { -n })
}

/// ```txt
/// ordmoday    = 1*2DIGIT       ;1 to 31
/// ```
fn ordmoday<'src, I, E>() -> impl Parser<'src, I, i8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        i8_1_2().then(i8_0_9()).map(|(a, b)| a * 10 + b), // 10-29
        just('3').ignore_then(i8_0_1()).map(|a| 30 + a),  // 30-31
        just('0').or_not().ignore_then(i8_1_9()),         // 1-9 / 01-09
    ))
}

/// ```txt
/// byyrdaylist = ( yeardaynum *("," yeardaynum) )
/// ```
fn byyrdaylist<'src, I, E>() -> impl Parser<'src, I, Vec<i16>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    yeardaynum().separated_by(just(',')).collect()
}

/// ```txt
/// yeardaynum  = [plus / minus] ordyrday
/// ```
fn yeardaynum<'src, I, E>() -> impl Parser<'src, I, i16, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    is_positive()
        .then(ordyrday())
        .map(|(positive, n)| if positive { n } else { -n })
}

/// ```txt
/// ordyrday    = 1*3DIGIT      ;1 to 366
/// ```
fn ordyrday<'src, I, E>() -> impl Parser<'src, I, i16, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    let i16_1_99 = i16_1_9().then(i16_0_9().or_not()).map(|(a, b)| match b {
        Some(b) => a * 10 + b, // 10-99
        None => a,             // 1-9
    });

    choice((
        just('3').ignore_then(choice((
            just('6').ignore_then(i16_0_6()).map(|a| 360 + a), // 360- 366
            i16_0_5().then(i16_0_9()).map(|(a, b)| 300 + a * 10 + b), // 300-359
        ))),
        i16_1_2()
            .then(i16_0_9())
            .then(i16_0_9())
            .map(|((a, b), c)| a * 100 + b * 10 + c), // 100-299
        just('0').or_not().ignore_then(choice((
            just('0').ignore_then(i16_0_9()), // 01-09 / 001-009
            i16_1_99,                         // 1-9 / 10-99 / 01-09 / 010-099
        ))),
    ))
}

/// ```txt
/// bywknolist  = ( weeknum *("," weeknum) )
/// ```
fn bywknolist<'src, I, E>() -> impl Parser<'src, I, Vec<i8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    weeknum().separated_by(just(',')).collect()
}

/// ```txt
/// weeknum     = [plus / minus] ordwk
/// ```
fn weeknum<'src, I, E>() -> impl Parser<'src, I, i8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    is_positive()
        .then(ordwk())
        .map(|(positive, n)| if positive { n } else { -n })
}

/// ```txt
/// bymolist    = ( monthnum *("," monthnum) )
/// ```
fn bymolist<'src, I, E>() -> impl Parser<'src, I, Vec<u8>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    monthnum().separated_by(just(',')).collect()
}

/// ```txt
/// monthnum    = 1*2DIGIT       ;1 to 12
/// ```
fn monthnum<'src, I, E>() -> impl Parser<'src, I, u8, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    choice((
        just('0').ignore_then(u8_1_9()),                 // 01-09
        just('1').ignore_then(u8_0_9()).map(|a| 10 + a), // 10-12
        u8_1_9(),                                        // 1-9
    ))
}

/// ```txt
/// bysplist    = ( setposday *("," setposday) )
/// ```
fn bysplist<'src, I, E>() -> impl Parser<'src, I, Vec<i16>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    setposday().separated_by(just(',')).collect()
}

/// ```txt
/// setposday   = yeardaynum
/// ```
fn setposday<'src, I, E>() -> impl Parser<'src, I, i16, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    yeardaynum()
}

// Helper parsers

fn is_positive<'src, I, E>() -> impl Parser<'src, I, bool, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    select! { c @ ('+' | '-') => c }
        .or_not()
        .map(|c| !matches!(c, Some('-')))
}

/// Parse u32 (1 or more digits)
fn u32_non_zero<'src, I, E>() -> impl Parser<'src, I, u32, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    select! { c @ '0'..='9' => c }
        .repeated()
        .at_least(1)
        .at_most(10) // u32 max is 10 digits
        .collect::<String>()
        .try_map_with(|str, e| {
            lexical::parse_partial::<u32, _>(&str)
                .map_err(|_| E::Error::expected_found([ValueExpected::U32], None, e.span()))
                .and_then(|(v, _)| match v {
                    0 => Err(E::Error::expected_found(
                        [ValueExpected::PositiveU32],
                        None,
                        e.span(),
                    )),
                    v => Ok(v),
                })
        })
}

#[cfg(test)]
mod tests {
    use chumsky::extra;
    use chumsky::input::Stream;

    use super::*;

    fn parse(src: &'_ str) -> Result<ValueRecurrenceRule, Vec<Rich<'_, char>>> {
        let stream = Stream::from_iter(src.chars());
        value_rrule::<'_, _, extra::Err<_>>()
            .parse(stream)
            .into_result()
    }

    #[test]
    fn parses_rrule_freq_only() {
        // Test all frequency values
        let freqs = [
            ("FREQ=SECONDLY", RecurrenceFrequency::Secondly),
            ("FREQ=MINUTELY", RecurrenceFrequency::Minutely),
            ("FREQ=HOURLY", RecurrenceFrequency::Hourly),
            ("FREQ=DAILY", RecurrenceFrequency::Daily),
            ("FREQ=WEEKLY", RecurrenceFrequency::Weekly),
            ("FREQ=MONTHLY", RecurrenceFrequency::Monthly),
            ("FREQ=YEARLY", RecurrenceFrequency::Yearly),
        ];

        for (src, expected_freq) in freqs {
            let result = parse(src).unwrap();
            assert_eq!(result.freq, expected_freq, "Failed for {src}");
            assert!(result.until.is_none());
            assert!(result.count.is_none());
            assert!(result.interval.is_none());
        }
    }

    #[test]
    fn parses_rrule_with_interval() {
        let src = "FREQ=DAILY;INTERVAL=2";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Daily);
        assert_eq!(result.interval, Some(2));
    }

    #[test]
    fn parses_rrule_with_until_datetime() {
        let src = "FREQ=DAILY;UNTIL=19971224T000000Z";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Daily);
        assert!(result.until.is_some());

        let until = result.until.unwrap();
        assert_eq!(until.date.year, 1997);
        assert_eq!(until.date.month, 12);
        assert_eq!(until.date.day, 24);
        assert!(until.time.utc);
    }

    #[test]
    fn parses_rrule_with_until_date() {
        let src = "FREQ=DAILY;UNTIL=19971224";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Daily);
        assert!(result.until.is_some());
        let until = result.until.unwrap();
        assert_eq!(until.date.year, 1997);
        assert_eq!(until.date.month, 12);
        assert_eq!(until.date.day, 24);
        assert!(!until.time.utc);
        assert_eq!(until.time.hour, 0);
        assert_eq!(until.time.minute, 0);
        assert_eq!(until.time.second, 0);
    }

    #[test]
    fn parses_rrule_with_count() {
        let src = "FREQ=DAILY;COUNT=10";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Daily);
        assert_eq!(result.count, Some(10));
    }

    #[test]
    fn parses_rrule_with_byday() {
        // Simple days
        let src = "FREQ=WEEKLY;BYDAY=MO,WE,FR";
        let result = parse(src).unwrap();
        assert_eq!(result.by_day.len(), 3);

        let first = result.by_day.first().unwrap();
        assert_eq!(first.day, WeekDay::Monday);
        assert_eq!(first.occurrence, None);
        assert_eq!(result.by_day.get(1).unwrap().day, WeekDay::Wednesday);
        assert_eq!(result.by_day.get(2).unwrap().day, WeekDay::Friday);

        // With occurrence
        let src = "FREQ=MONTHLY;BYDAY=1MO,-1MO";
        let result = parse(src).unwrap();
        assert_eq!(result.by_day.len(), 2);

        let first = result.by_day.first().unwrap();
        assert_eq!(first.day, WeekDay::Monday);
        assert_eq!(first.occurrence, Some(1));

        let second = result.by_day.get(1).unwrap();
        assert_eq!(second.day, WeekDay::Monday);
        assert_eq!(second.occurrence, Some(-1));
    }

    #[test]
    fn parses_rrule_with_byhour() {
        let src = "FREQ=DAILY;BYHOUR=9,10,11,12,13,14,15,16";
        let result = parse(src).unwrap();
        assert_eq!(result.by_hour, vec![9, 10, 11, 12, 13, 14, 15, 16]);
    }

    #[test]
    fn parses_rrule_with_byminute() {
        let src = "FREQ=DAILY;BYMINUTE=0,20,40";
        let result = parse(src).unwrap();
        assert_eq!(result.by_minute, vec![0, 20, 40]);
    }

    #[test]
    fn parses_rrule_with_bysecond() {
        let src = "FREQ=HOURLY;BYSECOND=0,15,30,45";
        let result = parse(src).unwrap();
        assert_eq!(result.by_second, vec![0, 15, 30, 45]);
    }

    #[test]
    fn parses_rrule_with_bymonthday() {
        let src = "FREQ=MONTHLY;BYMONTHDAY=1,15,-1";
        let result = parse(src).unwrap();
        assert_eq!(result.by_month_day, vec![1, 15, -1]);
    }

    #[test]
    fn parses_rrule_with_byyearday() {
        let src = "FREQ=YEARLY;BYYEARDAY=1,100,200,-1";
        let result = parse(src).unwrap();
        assert_eq!(result.by_year_day, vec![1, 100, 200, -1]);
    }

    #[test]
    fn parses_rrule_with_byweekno() {
        let src = "FREQ=YEARLY;BYWEEKNO=20,21,-1";
        let result = parse(src).unwrap();
        assert_eq!(result.by_week_no, vec![20, 21, -1]);
    }

    #[test]
    fn parses_rrule_with_bymonth() {
        let src = "FREQ=YEARLY;BYMONTH=1,2,3";
        let result = parse(src).unwrap();
        assert_eq!(result.by_month, vec![1, 2, 3]);
    }

    #[test]
    fn parses_rrule_with_bysetpos() {
        let src = "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1";
        let result = parse(src).unwrap();
        assert_eq!(result.by_set_pos, vec![-1]);
    }

    #[test]
    fn parses_rrule_with_wkst() {
        let src = "FREQ=WEEKLY;WKST=SU";
        let result = parse(src).unwrap();
        assert_eq!(result.wkst.unwrap(), WeekDay::Sunday);
    }

    #[test]
    fn parses_rrule_complex() {
        // Example from RFC 5545
        let src = "FREQ=YEARLY;INTERVAL=2;BYMONTH=1;BYDAY=SU;BYHOUR=8,9;BYMINUTE=30";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Yearly);
        assert_eq!(result.interval, Some(2));
        assert_eq!(result.by_month, vec![1]);
        assert_eq!(result.by_day.len(), 1);
        assert_eq!(result.by_day.first().unwrap().day, WeekDay::Sunday);
        assert_eq!(result.by_hour, vec![8, 9]);
        assert_eq!(result.by_minute, vec![30]);
    }

    #[test]
    fn parses_rrule_rejects_missing_freq() {
        // Missing FREQ should fail
        let src = "INTERVAL=2;COUNT=10";
        assert!(parse(src).is_err(), "Missing FREQ should fail");
    }

    #[test]
    fn parses_rrule_rejects_until_and_count_together() {
        // UNTIL and COUNT together should fail
        let src = "FREQ=DAILY;UNTIL=19971224T000000Z;COUNT=10";
        assert!(parse(src).is_err(), "UNTIL and COUNT together should fail");
    }

    #[test]
    fn parses_rrule_handles_reordered_parts() {
        // Parts in different order
        let src = "COUNT=10;INTERVAL=2;FREQ=DAILY";
        let result = parse(src).unwrap();
        assert_eq!(result.freq, RecurrenceFrequency::Daily);
        assert_eq!(result.count, Some(10));
        assert_eq!(result.interval, Some(2));
    }

    #[test]
    fn parses_rrule_rejects_duplicate_parts() {
        let test_cases = [
            ("FREQ=DAILY;FREQ=WEEKLY", "FREQ"),
            (
                "FREQ=DAILY;UNTIL=19971224T000000Z;UNTIL=19971225T000000Z",
                "UNTIL",
            ),
            ("FREQ=DAILY;COUNT=10;COUNT=20", "COUNT"),
            ("FREQ=DAILY;INTERVAL=1;INTERVAL=2", "INTERVAL"),
            ("FREQ=WEEKLY;BYDAY=MO;BYDAY=FR", "BYDAY"),
            ("FREQ=DAILY;BYHOUR=9;BYHOUR=10", "BYHOUR"),
        ];

        for (src, part_name) in test_cases {
            assert!(
                parse(src).is_err(),
                "Duplicate {part_name} should fail for input: {src}"
            );
        }
    }
}
