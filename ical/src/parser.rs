// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use chumsky::error::Rich;

use crate::lexer::{Token, lex_analysis};
use crate::semantic::{ICalendarRef, SemanticError, semantic_analysis};
use crate::syntax::syntax_analysis;
use crate::typed::{TypedError, typed_analysis};

/// Parse an iCalendar component from source code
///
/// This function performs all four phases of iCalendar parsing:
/// 1. Lexical analysis
/// 2. Syntax analysis
/// 3. Typed analysis
/// 4. Semantic analysis
///
/// ## Errors
///
/// If there are errors in any phase, a vector of error reports will be returned.
///
/// ## Examples
///
/// Parsing valid iCalendar source will return the root component:
///
/// ```
/// # use aimcal_ical::parse;
/// let ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// VERSION:2.0\r\n\
/// PRODID:-//Example Corp.//CalDAV Client//EN\r\n\
/// BEGIN:VEVENT\r\n\
/// UID:12345\r\n\
/// DTSTAMP:20250101T000000Z\r\n\
/// DTSTART:20250101T100000Z\r\n\
/// SUMMARY:Test Event\r\n\
/// END:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// ";
/// let calendars = parse(ical_src).unwrap();
/// assert_eq!(calendars[0].prod_id.value.to_string(), "-//Example Corp.//CalDAV Client//EN");
/// ```
///
/// Parsing invalid iCalendar source will return error reports:
///
/// ```
/// # use aimcal_ical::{ParseError, parse};
/// use ariadne::{Color, Label, Report, ReportKind, Source};
/// let invalid_ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// END:VEVENT\r\n\
/// ";
/// let result = parse(invalid_ical_src);
/// assert!(result.is_err());
/// let reports = result.unwrap_err().into_iter().map(|e| {
///   match e {
///     ParseError::Syntax(e) => {
///       Report::build(ReportKind::Error, e.span().into_range())
///         .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
///         .with_code(3)
///         .with_message(e.to_string())
///         .with_label(
///           Label::new(e.span().into_range())
///             .with_message(e.reason().to_string())
///             .with_color(Color::Red),
///         )
///         .finish()
///     }
///     ParseError::Typed(e) => {
///       Report::build(ReportKind::Error, e.span().into_range())
///          .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
///          .with_code(3)
///          .with_message(e.to_string())
///          .with_label(
///            Label::new(e.span().into_range())
///              .with_message(e.to_string())
///              .with_color(Color::Red),
///          )
///          .finish()
///     }
///     ParseError::Semantic(e) => {
///         Report::build(ReportKind::Error, e.span().into_range())
///             .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
///             .with_code(3)
///             .with_message(e.to_string())
///             .finish()
///     }
///     e => todo!("Other errors not implemented yet: {:?}", e),
///   }
/// }).collect::<Vec<_>>();
///
/// for report in reports {
///   report.eprint(Source::from(invalid_ical_src));
/// }
/// ```
pub fn parse(src: &'_ str) -> Result<Vec<ICalendarRef<'_>>, Vec<ParseError<'_>>> {
    let token_stream = lex_analysis(src);

    let syntax_components = syntax_analysis::<'_, '_, _, Rich<'_, _>>(src, token_stream)
        .map_err(|errs| errs.into_iter().map(ParseError::Syntax).collect::<Vec<_>>())?;

    let typed_components = typed_analysis(syntax_components)
        .map_err(|errs| errs.into_iter().map(ParseError::Typed).collect::<Vec<_>>())?;

    let icalendars = semantic_analysis(typed_components).map_err(|errs| {
        errs.into_iter()
            .map(ParseError::Semantic)
            .collect::<Vec<_>>()
    })?;

    Ok(icalendars)
}

/// Errors that can occur during parsing
// TODO: generic over error type, support different error types
#[non_exhaustive]
#[derive(Debug)]
pub enum ParseError<'src> {
    /// Errors from syntax analysis
    Syntax(Rich<'src, Token<'src>>),

    /// Errors from typed analysis
    Typed(TypedError<'src>),

    /// Errors from semantic analysis
    Semantic(SemanticError<'src>),
}
