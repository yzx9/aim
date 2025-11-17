// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::extra::ParserExtra;
use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;

use crate::lexer::{Token, lex};
use crate::property_value::{PropertyValue, property_value};

const KW_BEGIN: &str = "BEGIN";
const KW_END: &str = "END";

/// Parse an iCalendar component from source code
///
/// # Examples
///
/// Parsing valid iCalendar source will return the root component
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
        .map((0..src.len()).into(), |(t, s)| (t, s));

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
pub struct Component<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<Property<'src>>, // Keep the original order
    pub components: Vec<Component<'src>>,
}

fn component<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Component<'tokens>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|component| {
        let properties = property().repeated().collect::<Vec<Property>>();

        let subcomponents = component.repeated().collect::<Vec<Component<'tokens>>>();

        let body = map_ctx(|_name: &&'tokens str| (), properties.then(subcomponents));

        begin()
            .ignore_with_ctx(body.then(end()))
            .map(|((properties, components), name)| Component {
                name,
                properties,
                components,
            })
    })
}

fn begin<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, &'tokens str, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Word(KW_BEGIN))
        .ignore_then(just(Token::Colon))
        .ignore_then(select! { Token::Word(s) => s })
        .then_ignore(newline())
}

fn end<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, &'tokens str, extra::Full<Rich<'tokens, Token<'src>>, (), &'tokens str>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Word(KW_END))
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
        .then_ignore(newline().or_not())
}

#[derive(Debug, Clone)]
pub struct Property<'src> {
    pub group: Option<&'src str>,
    pub name: &'src str, // Case insensitive, keep original for writing back
    pub params: Vec<Parameter>, // Allow duplicates & multi-values
    pub value: PropertyValue<'src>, // Textual value (untyped)
}

fn property<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Property<'src>, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    let name = select! {
        Token::Word(s) if s != KW_BEGIN && s != KW_END => s
    };

    let params = just(Token::Semi)
        .ignore_then(parameter())
        .repeated()
        .collect::<Vec<_>>();

    name.then(params)
        .then_ignore(just(Token::Colon))
        .then(property_value())
        .then_ignore(newline())
        .map(|((name, params), value)| Property {
            group: None, // For now, we're not handling groups
            name,
            params,
            value,
        })
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,        // e.g. "TZID", "VALUE", "CN", "ROLE", "PARTSTAT"
    pub values: Vec<String>, // Split by commas
}

fn parameter<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Parameter, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! { Token::Word(s) => s }
        .then_ignore(just(Token::Eq))
        .then(
            select! {
                Token::Word(s) => s,
                Token::Symbol(s) => s
            }
            .repeated()
            .collect::<Vec<_>>()
            .separated_by(just(Token::Comma))
            .collect::<Vec<_>>(),
        )
        .map(|(name, values)| Parameter {
            name: name.to_string(),
            values: values.into_iter().map(|s| s.concat().to_string()).collect(),
        })
}

pub fn newline<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, (), E> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    just(Token::Control("\r"))
        .then(just(Token::Control("\n")))
        .ignored()
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

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

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

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

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
    fn test_property() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<Property<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

            property().parse(token_stream).into_result()
        }

        let src = "SUMMARY:Hello World!\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name, "SUMMARY");
        assert_eq!(prop.value, PropertyValue::Text("Hello World!".to_string()));

        let src = "DTSTART;TZID=America/New_York:20251113\r\n T100000\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name, "DTSTART");
        assert_eq!(prop.params.len(), 1);
        assert_eq!(prop.params[0].name, "TZID");
        assert_eq!(prop.params[0].values, vec!["America/New_York"]);
        assert_eq!(
            prop.value,
            PropertyValue::Text("20251113T100000".to_string())
        );
    }

    #[test]
    fn test_param() {
        fn parse(src: &str) -> Result<Parameter, Vec<Rich<'_, Token<'_>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

            parameter().parse(token_stream).into_result()
        }

        let src = "TZID=America/New_York";
        let result = parse(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let param = result.unwrap();
        assert_eq!(param.name, "TZID");
        assert_eq!(param.values, vec!["America/New_York"]);
    }
}
