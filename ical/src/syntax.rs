// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parser for iCalendar syntax as defined in RFC 5545, built on top of the lexer, no type.

use chumsky::DefaultExpected;
use chumsky::error::Error;
use chumsky::extra::ParserExtra;
use chumsky::inspector::Inspector;
use chumsky::prelude::*;

use crate::keyword::{KW_BEGIN, KW_END};
use crate::lexer::{SpannedToken, SpannedTokens, Token};

pub fn syntax_analysis<'tokens, 'src: 'tokens, I, Err>()
-> impl Parser<'tokens, I, Vec<RawComponent<'src>>, extra::Err<Err>>
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    Err: Error<'tokens, I> + 'tokens,
{
    component().repeated().at_least(1).collect()
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
        .ignore_then(select! { Token::Word(s) => s }) // FIXME: folding may break words?
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
    pub name: SpannedTokens<'src>, // Case insensitive, keep original for writing back
    pub params: Vec<RawParameter<'src>>, // Allow duplicates & multi-values
    pub value: Vec<SpannedTokens<'src>>, // Textual value (untyped)
}

fn property<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, RawProperty<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    let name = select! {
        t @ Token::Word(s) if s != KW_BEGIN && s != KW_END => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .at_least(1)
    .collect();

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
    pub name: SpannedTokens<'src>, // e.g. "TZID", "VALUE", "CN", "ROLE", "PARTSTAT"
    pub values: Vec<RawParameterValue<'src>>, // Split by commas
}

#[derive(Debug, Clone)]
pub struct RawParameterValue<'src> {
    pub value: SpannedTokens<'src>,
    pub quoted: bool,
}

fn parameter<'tokens, 'src: 'tokens, I, E>()
-> impl Parser<'tokens, I, RawParameter<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    let name = select! {
        t @ Token::Word(_) => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .at_least(1)
    .collect();

    let quoted_string = just(Token::DQuote)
        .ignore_then(
            select! {
               t @ (
                   Token::Comma
                   | Token::Colon
                   | Token::Semicolon
                   | Token::Symbol(_)
                   | Token::Escape(_)
                   | Token::Word(_)
                   | Token::UnicodeText(_)
                ) => t,
            }
            .map_with(SpannedToken::from_map_extra)
            .repeated()
            .collect(),
        )
        .then_ignore(just(Token::DQuote))
        .map(|s| RawParameterValue {
            value: s,
            quoted: true,
        });

    // safe characters
    let paramtext = select! {
        t @ (
            Token::Symbol(_)
            | Token::Escape(_)
            | Token::Word(_)
            | Token::UnicodeText(_)
        ) => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .collect()
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

fn value<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, SpannedTokens<'src>, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    select! {
        t @ (
            Token::DQuote
            | Token::Symbol(_)
            | Token::Escape(_)
            | Token::Word(_)
            | Token::UnicodeText(_)
        ) => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .at_least(1)
    .collect()
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
    use chumsky::input::Stream;

    use crate::lexer::lex;

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
