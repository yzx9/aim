// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use crate::semantic::{ICalendarRef, SemanticError, semantic_analysis};
use crate::syntax::SyntaxError;
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
/// Parsing invalid iCalendar source will return errors:
///
/// ```
/// # use aimcal_ical::parse;
/// let invalid_ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// END:VEVENT\r\n\
/// ";
/// let result = parse(invalid_ical_src);
/// assert!(result.is_err());
/// let errors = result.unwrap_err();
/// // Each error can be converted to a string for display
/// for error in &errors {
///   eprintln!("{}", error);
/// }
/// ```
pub fn parse(src: &'_ str) -> Result<Vec<ICalendarRef<'_>>, Vec<ParseError<'_>>> {
    // Syntax analysis (includes tokenization, scanning, and tree building)
    let syntax_components = crate::syntax::syntax_analysis(src)
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
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError<'src> {
    /// Errors from syntax analysis
    #[error("{0}")]
    Syntax(SyntaxError<'src>),

    /// Errors from typed analysis
    #[error("{0}")]
    Typed(TypedError<'src>),

    /// Errors from semantic analysis
    #[error("{0}")]
    Semantic(SemanticError<'src>),
}
