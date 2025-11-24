// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::error::Rich;

use crate::lexer::lex_analysis;
use crate::syntax::syntax_analysis;
use crate::typed::{TypedComponent, typed_analysis};

/// Parse an iCalendar component from source code
///
/// ## Errors
///
/// If there are lexing or parsing errors, a vector of error reports will be returned.
///
/// ## Examples
///
/// Parsing valid iCalendar source will return the root component
///
/// ```
/// # use aimcal_ical::parse;
/// let ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// SUMMARY:Test Event\r\n\
/// END:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// ";
/// assert!(parse(ical_src).is_ok());
/// ```
///
/// Parsing invalid iCalendar source will return error reports
///
/// ```
/// # use aimcal_ical::parse;
/// use ariadne::Source;
/// let invalid_ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// END:VEVENT\r\n\
/// ";
/// let result = parse(invalid_ical_src);
/// assert!(result.is_err());
/// for report in result.unwrap_err().iter() {
///   report.eprint(Source::from(invalid_ical_src));
/// }
/// ```
pub fn parse(src: &'_ str) -> Result<Vec<TypedComponent<'_>>, Vec<Report<'_>>> {
    let token_stream = lex_analysis(src);

    let raw_components =
        syntax_analysis::<'_, '_, _, Rich<'_, _>>(token_stream).map_err(|errs| {
            errs.iter()
                .map(|err| {
                    Report::build(ReportKind::Error, err.span().into_range())
                        .with_config(
                            ariadne::Config::new().with_index_type(ariadne::IndexType::Byte),
                        )
                        .with_code(3)
                        .with_message(err.to_string())
                        .with_label(
                            Label::new(err.span().into_range())
                                .with_message(err.reason().to_string())
                                .with_color(Color::Red),
                        )
                        .finish()
                })
                .collect::<Vec<_>>()
        })?;

    typed_analysis(raw_components).map_err(|errs| {
        errs.iter()
            .map(|err: &Rich<_>| {
                Report::build(ReportKind::Error, err.span().into_range())
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_code(4)
                    .with_message(err.to_string())
                    .with_label(
                        Label::new(err.span().into_range())
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
            })
            .collect()
    })
}
