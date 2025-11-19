// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parser for iCalendar syntax as defined in RFC 5545, built on top of the lexer, no type.

use std::{fmt::Display, ops::Range, str::Chars};

use ariadne::{Color, Label, Report, ReportKind};
use chumsky::DefaultExpected;
use chumsky::container::Container;
use chumsky::error::Error;
use chumsky::extra::ParserExtra;
use chumsky::input::Stream;
use chumsky::inspector::Inspector;
use chumsky::prelude::*;
use chumsky::span::Span;

use crate::lexer::{Token, lex};

const KW_BEGIN: &str = "BEGIN";
const KW_END: &str = "END";

/// Parse an iCalendar component from source code
///
/// ## Examples
///
/// Parsing valid iCalendar source will return the root component
///
/// ```
/// # use aimcal_ical::syntax;
/// let ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// ";
/// assert!(syntax(ical_src).is_ok());
/// ```
///
/// Parsing invalid iCalendar source will return error reports
///
/// ```
/// # use aimcal_ical::syntax;
/// use ariadne::Source;
/// let invalid_ical_src = "\
/// BEGIN:VCALENDAR\r\n\
/// BEGIN:VEVENT\r\n\
/// END:VCALENDAR\r\n\
/// END:VEVENT\r\n\
/// ";
/// let result = syntax(invalid_ical_src);
/// assert!(result.is_err());
/// for report in result.unwrap_err().iter() {
///   report.eprint(Source::from(invalid_ical_src));
/// }
/// ```
pub fn syntax<'src>(src: &'src str) -> Result<RawComponent<'src>, Vec<Report<'src>>> {
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
    component::<'_, '_, _, Rich<'src, Token<'_>>>()
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
pub struct RawComponent<'src> {
    pub name: &'src str, // "VCALENDAR" / "VEVENT" / "VTIMEZONE" / "VALARM" / ...
    pub properties: Vec<RawProperty<'src>>, // Keep the original order
    pub children: Vec<RawComponent<'src>>,
}

fn component<'tokens, 'src: 'tokens, I, Err>()
-> impl Parser<'tokens, I, RawComponent<'src>, extra::Err<Err>> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    Err: Error<'tokens, I> + 'tokens,
{
    recursive(|component| {
        let body = choice((property().map(Either::Left), component.map(Either::Right)))
            .repeated()
            .collect::<Vec<_>>()
            .map(|a| a.into_iter().partition_either());

        begin()
            .ignore_with_ctx(map_ctx(|_| (), body).then(end()))
            .map(|((properties, children), name)| RawComponent {
                name,
                properties,
                children,
            })
    })
}

fn begin<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, &'src str, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    just(Token::Word(KW_BEGIN))
        .ignore_then(just(Token::Colon))
        .ignore_then(select! { Token::Word(s) => s })
        .then_ignore(just(Token::Newline))
}

fn end<'tokens, 'src: 'tokens, I, Err, State>()
-> impl Parser<'tokens, I, &'src str, extra::Full<Err, State, &'src str>> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    Err: Error<'tokens, I>,
    State: Inspector<'tokens, I>,
{
    just(Token::Word(KW_END))
        .ignore_then(just(Token::Colon))
        .ignore_then(select! { Token::Word(s) => s })
        .validate(|got, e, emitter| {
            let expected = e.ctx();
            if &got != expected {
                emitter.emit(Err::expected_found(
                    #[allow(clippy::explicit_auto_deref)]
                    [DefaultExpected::Token(Token::Word(*expected).into())],
                    Some(Token::Word(got).into()),
                    e.span(),
                ));
            }
            got
        })
        .then_ignore(just(Token::Newline).or_not())
}

#[derive(Debug, Clone)]
pub struct RawProperty<'src> {
    pub name: StrSegments<'src>, // Case insensitive, keep original for writing back
    pub params: Vec<RawParameter<'src>>, // Allow duplicates & multi-values
    pub value: Vec<StrSegments<'src>>, // Textual value (untyped)
}

fn property<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, RawProperty<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    let name = select! {
        Token::Word(s) if s != KW_BEGIN && s != KW_END => s,
    }
    .map_with(|s, e| (s, e.span()))
    .repeated()
    .at_least(1)
    .collect::<StrSegments>();

    let params = just(Token::Semicolon)
        .ignore_then(parameter())
        .repeated()
        .collect();

    let values = value().separated_by(just(Token::Comma)).collect();

    name.then(params)
        .then_ignore(just(Token::Colon))
        .then(values)
        .then_ignore(just(Token::Newline))
        .map(|((name, params), value)| RawProperty {
            name,
            params,
            value,
        })
}

#[derive(Debug, Clone)]
pub struct RawParameter<'src> {
    pub name: StrSegments<'src>, // e.g. "TZID", "VALUE", "CN", "ROLE", "PARTSTAT"
    pub values: Vec<RawParameterValue<'src>>, // Split by commas
}

#[derive(Debug, Clone)]
pub struct RawParameterValue<'src> {
    #[allow(dead_code)]
    value: StrSegments<'src>,

    #[allow(dead_code)]
    quoted: bool,
}

fn parameter<'tokens, 'src: 'tokens, I, E>()
-> impl Parser<'tokens, I, RawParameter<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    let name = select! {
        Token::Word(s) => s
    }
    .map_with(|s, e| (s, e.span()))
    .repeated()
    .at_least(1)
    .collect::<StrSegments<'_>>();

    let quoted_string = just(Token::DQuote)
        .ignore_then(
            select! {
                Token::Comma => ",",
                Token::Colon => ":",
                Token::Semicolon => ";",
                Token::Symbol(s) => s,
                Token::Escape(s) => s,
                Token::Word(s) => s,
                Token::UnicodeText(s) => s,
            }
            .map_with(|s, e| (s, e.span()))
            .repeated()
            .collect::<StrSegments>(),
        )
        .then_ignore(just(Token::DQuote))
        .map(|s| RawParameterValue {
            value: s,
            quoted: true,
        });

    // safe characters
    let paramtext = select! {
        Token::Symbol(s) => s,
        Token::Escape(s) => s,
        Token::Word(s) => s,
        Token::UnicodeText(s) => s,
    }
    .map_with(|s, e| (s, e.span()))
    .repeated()
    .collect::<StrSegments>()
    .map(|s| RawParameterValue {
        value: s,
        quoted: false,
    });

    let value = choice((paramtext, quoted_string))
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>();

    name.then_ignore(just(Token::Equal))
        .then(value)
        .map(|(name, values)| RawParameter { name, values })
}

fn value<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, StrSegments<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    select! {
        Token::DQuote => r#"""#,
        Token::Symbol(s) => s,
        Token::Escape(s) => match s { // TODO: should unescape in later stage?
            r"\n" | r"\N" => "\n",
            r"\;" => ";",
            r"\," => ",",
            r"\\" => r"\",
            _ => unreachable!(),
        },
        Token::Word(s) => s,
        Token::UnicodeText(s) => s,
    }
    .map_with(|s, e| (s, e.span()))
    .repeated()
    .at_least(1)
    .collect::<StrSegments>()
}

#[derive(Debug, Clone)]
struct StrSegment<'src> {
    segment: &'src str,

    #[allow(dead_code)]
    span: Range<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct StrSegments<'src>(Vec<StrSegment<'src>>);

impl<'src> StrSegments<'src> {
    fn iter_chars<'segs: 'src>(&'segs self) -> StrSegmentsCharsIter<'src, 'segs> {
        StrSegmentsCharsIter {
            segments: &self.0,
            seg_idx: 0,
            chars: None,
        }
    }
}

impl<'src, S> Container<(&'src str, S)> for StrSegments<'src>
where
    S: Span<Offset = usize>,
{
    fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }

    // TODO: maybe we can expand last segment if possible
    fn push(&mut self, (segment, span): (&'src str, S)) {
        self.0.push(StrSegment {
            segment,
            span: span.start()..span.end(),
        });
    }
}

impl Display for StrSegments<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.iter_chars() {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

struct StrSegmentsCharsIter<'src, 'segs: 'src> {
    segments: &'segs [StrSegment<'src>],
    seg_idx: usize,
    chars: Option<Chars<'src>>,
}

impl<'src, 'segs> Iterator for StrSegmentsCharsIter<'src, 'segs> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        while self.seg_idx < self.segments.len() {
            match self.chars {
                Some(ref mut chars) => match chars.next() {
                    Some(c) => return Some(c),
                    None => {
                        self.seg_idx += 1;
                        self.chars = None;
                    }
                },
                None => {
                    let seg = self.segments.get(self.seg_idx).unwrap();
                    self.chars = Some(seg.segment.chars());
                }
            }
        }

        None
    }
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

trait EitherIterExt<L, R> {
    fn partition_either(self) -> (Vec<L>, Vec<R>);
}

impl<L, R, I> EitherIterExt<L, R> for I
where
    I: Iterator<Item = Either<L, R>>,
{
    fn partition_either(self) -> (Vec<L>, Vec<R>) {
        let mut lefts = Vec::new();
        let mut rights = Vec::new();
        for v in self {
            match v {
                Either::Left(a) => lefts.push(a),
                Either::Right(b) => rights.push(b),
            }
        }
        lefts.shrink_to_fit();
        rights.shrink_to_fit();
        (lefts, rights)
    }
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

            begin::<'_, '_, _, extra::Err<_>>()
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
        assert!(mismatched.is_err());
        let errs = mismatched.unwrap_err();
        assert_eq!(errs.len(), 1);
        let expected_msg = "found 'Word(VEVENT)' expected 'Word(VCALENDAR)'";
        assert_eq!(&errs[0].to_string(), expected_msg);
    }

    #[test]
    fn test_begin_end_match() {
        fn parse(src: &str) -> Result<&str, Vec<Rich<'_, Token<'_>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

            begin::<'_, '_, _, extra::Err<_>>()
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
        assert!(mismatched.is_err());
        let errs = mismatched.unwrap_err();
        assert_eq!(errs.len(), 1);
        let expected_msg = "found 'Word(VEVENT)' expected 'Word(VCALENDAR)'";
        assert_eq!(&errs[0].to_string(), expected_msg);
    }

    #[test]
    fn test_property() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<RawProperty<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

            property::<'_, '_, _, extra::Err<_>>()
                .parse(token_stream)
                .into_result()
        }

        let src = "SUMMARY:Hello World!\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name.to_string(), "SUMMARY");
        assert_eq!(
            prop.value.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            ["Hello World!"]
        );

        let src = "DTSTART;TZID=America/New_York:20251113\r\n T100000\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name.to_string(), "DTSTART");
        assert_eq!(prop.params.len(), 1);
        assert_eq!(prop.params[0].name.to_string(), "TZID");
        assert_eq!(
            prop.params[0]
                .values
                .iter()
                .map(|a| a.value.to_string())
                .collect::<Vec<_>>(),
            ["America/New_York"]
        );
        assert_eq!(
            prop.value.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            ["20251113T100000"]
        );
    }

    #[test]
    fn test_param() {
        fn parse<'src>(src: &'src str) -> Result<RawParameter<'src>, Vec<Rich<'src, Token<'src>>>> {
            let lexer = lex(src).spanned().map(|(token, span)| match token {
                Ok(tok) => (tok, span.into()),
                Err(()) => panic!("lex error"),
            });

            let token_stream = Stream::from_iter(lexer).map((0..src.len()).into(), |(t, s)| (t, s));

            parameter::<'_, '_, _, extra::Err<_>>()
                .parse(token_stream)
                .into_result()
        }

        let src = "TZID=America/New_York";
        let result = parse(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let param = result.unwrap();
        assert_eq!(param.name.to_string(), "TZID");
        assert_eq!(
            param
                .values
                .iter()
                .map(|a| a.value.to_string())
                .collect::<Vec<_>>(),
            ["America/New_York"]
        );
    }
}
