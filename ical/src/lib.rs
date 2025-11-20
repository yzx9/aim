// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod keyword;
mod lexer;
mod property_spec;
mod property_value;
mod syntax;
mod typed;

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::Parser;
use chumsky::error::Rich;
use chumsky::input::{Input, Stream};
use chumsky::span::SimpleSpan;

use crate::lexer::{Token, lex};
use crate::typed::TypedComponent;

pub use crate::syntax::syntax_analysis;
pub use crate::typed::typed_analysis;

/// Parse an iCalendar component from source code
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
pub fn parse<'src>(src: &'src str) -> Result<Vec<TypedComponent<'src>>, Vec<Report<'src>>> {
    // Create a logos lexer over the source code
    let token_iter = lex(src)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier
            // to work with
            Ok(tok) => (tok, SimpleSpan::from(span)),
            Err(()) => unimplemented!(),
        });

    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    let token_stream = Stream::from_iter(token_iter)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..src.len()).into(), |(t, s)| (t, s));

    let raw_components = syntax_analysis::<'_, '_, _, Rich<'src, Token<'_>>>()
        .parse(token_stream)
        .into_result()
        .map_err(|errs| {
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
            .map(|err| {
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
