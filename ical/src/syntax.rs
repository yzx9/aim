// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parser for iCalendar syntax as defined in RFC 5545, built on top of the lexer, no type.

use crate::string_storage::StringStorage;

use chumsky::DefaultExpected;
use chumsky::container::Container;
use chumsky::error::Error;
use chumsky::extra::ParserExtra;
use chumsky::input::ValueInput;
use chumsky::inspector::Inspector;
use chumsky::prelude::*;

use crate::keyword::{KW_BEGIN, KW_END};
use crate::lexer::{SpannedToken, Token};
use crate::string_storage::{Span, SpannedSegments};

/// Parse raw iCalendar components from token stream
///
/// ## Errors
/// If there are parsing errors, a vector of errors will be returned.
pub fn syntax_analysis<'tokens, 'src: 'tokens, I, Err>(
    src: &'src str,
    token_stream: I,
) -> Result<Vec<SyntaxComponent<'src>>, Vec<Err>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    Err: Error<'tokens, I> + 'tokens,
{
    let parser = component().repeated().at_least(1).collect::<Vec<_>>();
    let components = parser.parse(token_stream).into_result()?;
    Ok(components.into_iter().map(|comp| comp.build(src)).collect())
}

/// A parsed iCalendar component (e.g., VCALENDAR, VEVENT, VTODO)
#[derive(Debug, Clone)]
pub struct SyntaxComponent<'src> {
    /// Component name (e.g., "VCALENDAR", "VEVENT", "VTIMEZONE", "VALARM")
    pub name: &'src str,
    /// Properties in original order
    pub properties: Vec<SyntaxProperty<'src>>,
    /// Nested child components
    pub children: Vec<SyntaxComponent<'src>>,
    /// Span of the entire component (from BEGIN to END)
    pub span: Span,
}

/// A parsed iCalendar property (name, optional parameters, and value)
#[derive(Debug, Clone)]
pub struct SyntaxProperty<'src> {
    /// Property name (case-insensitive, original casing preserved)
    pub name: SpannedSegments<'src>,
    /// Property parameters (allow duplicates & multi-values)
    pub parameters: Vec<SyntaxParameterRef<'src>>,
    /// Raw property value (may need further parsing by typed analysis)
    pub value: SpannedSegments<'src>,
}

/// A parsed iCalendar parameter (e.g., `TZID=America/New_York`)
#[derive(Debug, Clone)]
pub struct SyntaxParameter<S: StringStorage> {
    /// Parameter name (e.g., "TZID", "VALUE", "CN", "ROLE", "PARTSTAT")
    pub name: S,
    /// Parameter values split by commas
    pub values: Vec<SyntaxParameterValue<S>>,
}

impl SyntaxParameter<SpannedSegments<'_>> {
    /// Get the full span of this parameter (from name to last value)
    #[must_use]
    pub fn span(&self) -> Span {
        match self.values.last() {
            Some(v) => Span {
                start: self.name.span().start,
                end: v.value.span().end,
            },
            None => self.name.span(),
        }
    }
}

/// A single parameter value with optional quoting
#[derive(Debug, Clone)]
pub struct SyntaxParameterValue<S: StringStorage> {
    /// The parameter value
    pub value: S,
    /// Whether the value was quoted in the source
    pub quoted: bool,
}

/// Type alias for borrowed syntax parameter
pub type SyntaxParameterRef<'src> = SyntaxParameter<SpannedSegments<'src>>;

/// Type alias for owned syntax parameter
pub type SyntaxParameterOwned = SyntaxParameter<String>;

/// Type alias for borrowed syntax parameter value
pub type SyntaxParameterValueRef<'src> = SyntaxParameterValue<SpannedSegments<'src>>;

/// Type alias for owned syntax parameter value
pub type SyntaxParameterValueOwned = SyntaxParameterValue<String>;

impl SyntaxParameterRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> SyntaxParameterOwned {
        SyntaxParameterOwned {
            name: self.name.to_owned(),
            values: self
                .values
                .iter()
                .map(SyntaxParameterValue::to_owned)
                .collect(),
        }
    }
}

impl SyntaxParameterValueRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> SyntaxParameterValueOwned {
        SyntaxParameterValueOwned {
            value: self.value.to_owned(),
            quoted: self.quoted,
        }
    }
}

struct RawComponent<'src> {
    pub name: &'src str,
    pub properties: Vec<RawProperty>,
    pub children: Vec<RawComponent<'src>>,
    pub span: Span,
}

impl<'src> RawComponent<'src> {
    fn build(self, src: &'src str) -> SyntaxComponent<'src> {
        SyntaxComponent {
            name: self.name,
            properties: self.properties.into_iter().map(|p| p.build(src)).collect(),
            children: self.children.into_iter().map(|c| c.build(src)).collect(),
            span: self.span,
        }
    }
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
            .map_with(|((properties, children), name), extra| RawComponent {
                name,
                properties,
                children,
                span: Span::from(extra.span()),
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
                    #[expect(clippy::explicit_auto_deref)]
                    [DefaultExpected::Token(Token::Word(*expected).into())],
                    Some(Token::Word(got).into()),
                    e.span(),
                ));
            }
            got
        })
        .then_ignore(just(Token::Newline).or_not())
}

struct RawProperty {
    name: SpanCollector,
    parameters: Vec<RawParameter>,
    value: SpanCollector,
}

impl RawProperty {
    fn build(self, src: &'_ str) -> SyntaxProperty<'_> {
        SyntaxProperty {
            name: self.name.build(src),
            parameters: self.parameters.into_iter().map(|p| p.build(src)).collect(),
            value: self.value.build(src),
        }
    }
}

fn property<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, RawProperty, E> + Clone
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

    name.then(params)
        .then_ignore(just(Token::Colon))
        .then(value())
        .then_ignore(just(Token::Newline))
        .map(|((name, params), value)| RawProperty {
            name,
            parameters: params,
            value,
        })
}

struct RawParameter {
    pub name: SpanCollector,
    pub values: Vec<RawParameterValue>,
}

impl RawParameter {
    fn build(self, src: &'_ str) -> SyntaxParameterRef<'_> {
        SyntaxParameter {
            name: self.name.build(src),
            values: self.values.into_iter().map(|v| v.build(src)).collect(),
        }
    }
}

struct RawParameterValue {
    pub value: SpanCollector,
    pub quoted: bool,
}

impl RawParameterValue {
    fn build(self, src: &'_ str) -> SyntaxParameterValueRef<'_> {
        SyntaxParameterValue {
            value: self.value.build(src),
            quoted: self.quoted,
        }
    }
}

fn parameter<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, RawParameter, E> + Clone
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
                   | Token::Word(_)
                   | Token::UnicodeText(_)
                ) => t,
            }
            .map_with(SpannedToken::from_map_extra)
            .repeated()
            .collect(),
        )
        .then_ignore(just(Token::DQuote))
        .map(|value| RawParameterValue {
            value,
            quoted: true,
        });

    // safe characters
    let paramtext = select! {
        t @ (
            Token::Symbol(_)
            | Token::Word(_)
            | Token::UnicodeText(_)
        ) => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .at_least(1)
    .collect()
    .map(|value| RawParameterValue {
        value,
        quoted: false,
    });

    let values = choice((quoted_string, paramtext))
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>();

    name.then_ignore(just(Token::Equal))
        .then(values)
        .map(|(name, values)| RawParameter { name, values })
}

fn value<'tokens, 'src: 'tokens, I, E>() -> impl Parser<'tokens, I, SpanCollector, E> + Clone
where
    I: Input<'tokens, Token = Token<'src>, Span = SimpleSpan>,
    E: ParserExtra<'tokens, I>,
{
    select! {
        t @ (
            Token::DQuote
            | Token::Comma
            | Token::Colon
            | Token::Semicolon
            | Token::Equal
            | Token::Symbol(_)
            | Token::Word(_)
            | Token::UnicodeText(_)
        ) => t,
    }
    .map_with(SpannedToken::from_map_extra)
    .repeated()
    .at_least(1)
    .collect::<SpanCollector>()
}

#[derive(Default)]
struct SpanCollector(Vec<Span>);

impl SpanCollector {
    pub fn build(self, src: &'_ str) -> SpannedSegments<'_> {
        let segments: Vec<_> = self
            .0
            .into_iter()
            .map(|s| {
                let segment_str = &src[s.into_range()];
                (segment_str, s)
            })
            .collect();
        SpannedSegments::new(segments)
    }
}

impl<'src> Container<SpannedToken<'src>> for SpanCollector {
    fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }

    fn push(&mut self, spanned_token: SpannedToken<'src>) {
        match self.0.last_mut() {
            Some(last) if last.end == spanned_token.1.start => {
                last.end = spanned_token.1.end;
            }
            _ => self.0.push(spanned_token.1),
        }
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
        (lefts, rights)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex_analysis;
    use crate::string_storage::Span;

    use super::*;

    #[test]
    fn parses_component() {
        fn parse(src: &str) -> Result<&str, Vec<Rich<'_, Token<'_>>>> {
            begin::<'_, '_, _, extra::Err<_>>()
                .ignore_with_ctx(end())
                .parse(lex_analysis(src))
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
        assert_eq!(
            &errs.first().map(ToString::to_string).unwrap_or_default(),
            expected_msg
        );
    }

    #[test]
    fn matches_begin_end_tags() {
        fn parse(src: &str) -> Result<&str, Vec<Rich<'_, Token<'_>>>> {
            begin::<'_, '_, _, extra::Err<_>>()
                .ignore_with_ctx(end())
                .parse(lex_analysis(src))
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
        assert_eq!(&errs.first().unwrap().to_string(), expected_msg);
    }

    #[test]
    fn parses_property() {
        fn parse<'tokens, 'src: 'tokens>(
            src: &'src str,
        ) -> Result<SyntaxProperty<'src>, Vec<Rich<'src, Token<'tokens>>>> {
            property::<'_, '_, _, extra::Err<_>>()
                .parse(lex_analysis(src))
                .into_result()
                .map(|p| p.build(src))
        }

        let src = "SUMMARY:Hello World!\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name.to_owned(), "SUMMARY");
        assert_eq!(prop.value.to_owned(), "Hello World!");

        let src = "DTSTART;TZID=America/New_York:20251113\r\n T100000\r\n";
        let result = parse(src);
        assert!(result.is_ok(), "Parse '{src}' error: {:?}", result.err());
        let prop = result.unwrap();
        assert_eq!(prop.name.to_owned(), "DTSTART");
        assert_eq!(prop.parameters.len(), 1);
        assert_eq!(prop.parameters.first().unwrap().name.to_owned(), "TZID");
        assert_eq!(
            prop.parameters
                .first()
                .unwrap()
                .values
                .iter()
                .map(|a| a.value.to_owned())
                .collect::<Vec<_>>(),
            ["America/New_York"]
        );
        assert_eq!(prop.value.to_owned(), "20251113T100000");
    }

    #[test]
    fn parses_parameter() {
        fn parse(src: &str) -> Result<SyntaxParameterRef<'_>, Vec<Rich<'_, Token<'_>>>> {
            parameter::<'_, '_, _, extra::Err<_>>()
                .parse(lex_analysis(src))
                .into_result()
                .map(|p| p.build(src))
        }

        let src = "TZID=America/New_York";
        let result = parse(src);
        assert!(result.is_ok(), "Parse {src} error: {:?}", result.err());
        let param = result.unwrap();
        assert_eq!(param.name.to_owned(), "TZID");
        assert_eq!(
            param
                .values
                .iter()
                .map(|a| a.value.to_owned())
                .collect::<Vec<_>>(),
            ["America/New_York"]
        );
    }

    #[test]
    fn spanned_segments_starts_with_str_ignore_ascii_case() {
        fn make_segments<'a>(parts: &[(&'a str, Span)]) -> SpannedSegments<'a> {
            let segments = parts.iter().map(|&(s, span)| (s, span)).collect();
            SpannedSegments::new(segments)
        }

        // Test X- properties (case-insensitive)
        let segments = make_segments(&[("X-CUSTOM-PROP", Span::new(0, 12))]);
        assert!(segments.starts_with_str_ignore_ascii_case("X-"));
        assert!(segments.starts_with_str_ignore_ascii_case("x-"));

        // Test non-X- properties
        let segments = make_segments(&[("NONSTANDARD-PROP", Span::new(0, 15))]);
        assert!(!segments.starts_with_str_ignore_ascii_case("X-"));
        assert!(!segments.starts_with_str_ignore_ascii_case("x-"));

        // Test mixed case
        let segments = make_segments(&[("x-custom", Span::new(0, 7))]);
        assert!(segments.starts_with_str_ignore_ascii_case("X-"));
        assert!(segments.starts_with_str_ignore_ascii_case("x-"));

        // Test multi-segment
        let segments = make_segments(&[("X-", Span::new(0, 2)), ("CUSTOM", Span::new(2, 7))]);
        assert!(segments.starts_with_str_ignore_ascii_case("x-"));
        assert!(segments.starts_with_str_ignore_ascii_case("X-C"));
    }
}
