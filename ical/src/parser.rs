// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;

use crate::lexer::{Token, lex};

/// Parse an iCalendar component from source code
///
/// # Examples
///
/// ```
/// # use aimcal_ical::parse;
/// let ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// ";
/// assert!(parse(ical_src).is_ok());
/// ```
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
///   report.print(Source::from(invalid_ical_src));
/// }
/// ```
pub fn parse<'src>(src: &'src str) -> Result<Component<'src>, Vec<Report<'src>>> {
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
        .map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

    // Parse the token stream with our chumsky parser
    component()
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
                .collect()
        })
}

#[derive(Debug, Clone)]
pub struct Component<'a> {
    pub name: &'a str,        // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub props: Vec<Property>, // Keep the original order
    pub subcomponents: Vec<Component<'a>>,
}

fn component<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Component<'tokens>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|component| {
        let components = map_ctx(
            |_name: &&'tokens str| (),
            component.repeated().collect::<Vec<Component<'tokens>>>(),
        );

        begin()
            .ignore_with_ctx(components.then(end()))
            .map(|(subcomponents, name)| Component {
                name,
                props: vec![],
                subcomponents,
            })
    })
}

fn begin<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, &'tokens str, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Word("BEGIN"))
        .ignore_then(just(Token::Colon))
        .ignore_then(select! { Token::Word(s) => s })
        .then_ignore(just(Token::Newline))
}

fn end<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, &'tokens str, extra::Full<Rich<'tokens, Token<'src>>, (), &'tokens str>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Word("END"))
        .ignore_then(just(Token::Colon))
        .ignore_then(select! { Token::Word(s) => s })
        .try_map_with(|got, e| {
            if &got == e.ctx() {
                Ok(got)
            } else {
                Err(Rich::custom(
                    e.span(),
                    format!("END mismatch: got {got}, expected {}", e.ctx()),
                ))
            }
        })
        .then_ignore(just(Token::Newline))
}

#[derive(Debug, Clone)]
pub struct Property {
    pub group: Option<String>,
    pub name: String,       // Case insensitive, keep original for writing back
    pub params: Vec<Param>, // Allow duplicates & multi-values
    pub value: RawValue,    // Textual value (untyped)
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,        // e.g. "TZID", "VALUE", "CN", "ROLE", "PARTSTAT"
    pub values: Vec<String>, // Split by commas
}

#[derive(Debug, Clone)]
pub struct RawValue {
    pub text: String, // Unfolded and unescaped raw value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component() {
        fn parse(src: &str) -> Result<&str, Vec<Rich<'_, Token<'_>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream =
                Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

            begin()
                .ignore_with_ctx(end())
                .parse(token_stream)
                .into_result()
        }

        let matched = parse(
            "\
BEGIN:VCALENDAR\r\n\
END:VCALENDAR\r\n\
",
        );
        assert_eq!(matched, Ok("VCALENDAR"));

        let mismatched = "\
BEGIN:VCALENDAR\r\n\
END:VEVENT\r\n\
";
        let mismatched = parse(mismatched);
        let expected = Err(vec![Rich::custom(
            (17..27).into(),
            "END mismatch: got VEVENT, expected VCALENDAR",
        )]);
        assert_eq!(mismatched, expected);
    }

    #[test]
    fn test_begin_end_match() {
        fn parse(src: &str) -> Result<&str, Vec<Rich<'_, Token<'_>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream =
                Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

            begin()
                .ignore_with_ctx(end())
                .parse(token_stream)
                .into_result()
        }

        let matched = parse(
            "\
BEGIN:VCALENDAR\r\n\
END:VCALENDAR\r\n\
",
        );
        assert_eq!(matched, Ok("VCALENDAR"));

        let mismatched = "\
BEGIN:VCALENDAR\r\n\
END:VEVENT\r\n\
";
        let mismatched = parse(mismatched);
        let expected = Err(vec![Rich::custom(
            (17..27).into(),
            "END mismatch: got VEVENT, expected VCALENDAR",
        )]);
        assert_eq!(mismatched, expected);
    }
}
