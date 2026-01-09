// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;
use std::fmt;

use chumsky::Parser;
use chumsky::container::Container;
use chumsky::extra::ParserExtra;
use chumsky::prelude::*;

use crate::string_storage::{SpannedSegments, StringStorage};

/// Text value type defined in RFC 5545 Section 3.3.11.
#[derive(Default, Debug, Clone)]
pub struct ValueText<S: StringStorage> {
    tokens: Vec<ValueTextToken<S>>,
}

/// Type alias for borrowed text value
pub type ValueTextRef<'src> = ValueText<SpannedSegments<'src>>;

/// Type alias for owned text value
pub type ValueTextOwned = ValueText<String>;

impl<'src> ValueTextRef<'src> {
    /// Resolve the text value into a single string, processing escapes.
    ///
    /// This version tries to avoid allocation when there's only a single string token.
    #[must_use]
    pub fn resolve(&self) -> Cow<'src, str> {
        #[expect(clippy::indexing_slicing)]
        if self.tokens.len() == 1
            && let ValueTextToken::Str(part) = &self.tokens[0]
        {
            part.resolve()
        } else {
            Cow::Owned(self.to_string())
        }
    }

    /// Compare the text value with a string, ignoring ASCII case.
    ///
    /// This method iterates through tokens without allocating a new string,
    /// using string slice comparison for efficiency.
    #[must_use]
    pub(crate) fn eq_str_ignore_ascii_case(&self, other: &str) -> bool {
        let mut remaining = other;

        for token in &self.tokens {
            if remaining.is_empty() {
                return false;
            }

            match token {
                ValueTextToken::Str(part) => {
                    if part.len() > remaining.len() {
                        return false;
                    }
                    let Some((head, tail)) = remaining.split_at_checked(part.len()) else {
                        return false;
                    };
                    if !part.eq_str_ignore_ascii_case(head) {
                        return false;
                    }
                    remaining = tail;
                }
                ValueTextToken::Escape(escape_char) => {
                    // Escape token is exactly 1 character
                    let Some((first, rest)) = remaining.split_at_checked(1) else {
                        return false;
                    };
                    // Compare first with expected escape character
                    if !first.eq_ignore_ascii_case(escape_char.as_ref()) {
                        return false;
                    }
                    remaining = rest;
                }
            }
        }

        // Check if we've consumed all characters from other
        remaining.is_empty()
    }

    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> ValueTextOwned {
        ValueTextOwned {
            tokens: self
                .tokens
                .iter()
                .map(|token| match token {
                    ValueTextToken::Str(s) => ValueTextToken::Str(s.to_owned()),
                    ValueTextToken::Escape(c) => ValueTextToken::Escape(*c),
                })
                .collect(),
        }
    }
}

impl<S: StringStorage> fmt::Display for ValueText<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for token in &self.tokens {
            match token {
                ValueTextToken::Str(part) => write!(f, "{part}")?,
                ValueTextToken::Escape(c) => write!(f, "{c}")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum ValueTextToken<S: StringStorage> {
    Str(S),
    Escape(ValueTextEscape),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueTextEscape {
    Backslash,
    Semicolon,
    Comma,
    Newline,
}

impl AsRef<str> for ValueTextEscape {
    fn as_ref(&self) -> &str {
        match self {
            ValueTextEscape::Backslash => "\\",
            ValueTextEscape::Semicolon => ";",
            ValueTextEscape::Comma => ",",
            ValueTextEscape::Newline => "\n",
        }
    }
}

impl fmt::Display for ValueTextEscape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[derive(Debug)]
pub struct RawValueText(Vec<Either<SpanCollector, ValueTextEscape>>);

impl RawValueText {
    pub fn build<'src>(self, src: &SpannedSegments<'src>) -> ValueTextRef<'src> {
        let size = self.0.iter().fold(0, |acc, t| match t {
            Either::Left(collector) => acc + collector.0.len(),
            Either::Right(_) => acc + 1,
        });

        let mut tokens = Vec::with_capacity(size);
        for t in self.0 {
            match t {
                Either::Left(collector) => {
                    tokens.extend(collector.build(src).into_iter().map(ValueTextToken::Str));
                }
                Either::Right(v) => tokens.push(ValueTextToken::Escape(v)),
            }
        }

        ValueText { tokens }
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// text       = *(TSAFE-CHAR / ":" / DQUOTE / ESCAPED-CHAR)
/// ; Folded according to description above
///
/// ESCAPED-CHAR = ("\\" / "\;" / "\," / "\N" / "\n")
/// ; \\ encodes \, \N or \n encodes newline
/// ; \; encodes ;, \, encodes ,
///
/// TSAFE-CHAR = WSP / %x21 / %x23-2B / %x2D-39 / %x3C-5B / %x5D-7E / NON-US-
/// ASCII
/// ; Any character except CONTROLs not needed by the current
/// ; character set, DQUOTE, ";", ":", "\", ","
/// ```
fn value_text<'src, I, E>() -> impl Parser<'src, I, RawValueText, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    let s = select! { c if c != '\\' => c }
        .ignored()
        .repeated()
        .at_least(1)
        .map_with(|(), e| e.span())
        .collect::<SpanCollector>()
        .map(Either::Left);

    let escape = just('\\')
        .ignore_then(select! {
            ';' => ValueTextEscape::Semicolon,
            ',' => ValueTextEscape::Comma,
            'N' | 'n' => ValueTextEscape::Newline,
            '\\' => ValueTextEscape::Backslash,
        })
        .map(Either::Right);

    choice((s, escape)).repeated().collect().map(RawValueText)
}

/// Text multiple values parser.
///
/// If the property permits, multiple TEXT values are specified by a
/// COMMA-separated list of values.
pub fn values_text<'src, I, E>() -> impl Parser<'src, I, Vec<RawValueText>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    value_text().separated_by(just(',')).collect()
}

#[derive(Debug, Default)]
struct SpanCollector(Vec<SimpleSpan>);

impl SpanCollector {
    fn build<'src>(self, src: &SpannedSegments<'src>) -> Vec<SpannedSegments<'src>> {
        // assume src segments are non-overlapping and sorted
        let mut iter = src.segments.iter();
        let Some(mut item) = iter.next() else {
            return Vec::new(); // no segments
        };

        let mut vec = Vec::with_capacity(self.0.len());
        for span in self.0 {
            let mut flag = true;
            while flag {
                if span.start > item.1.end {
                    // need next segment, and skip this one
                    match iter.next() {
                        Some(a) => item = a,
                        None => flag = false, // no more segments
                    }
                } else if span.end > item.1.end {
                    // need next segment
                    let i = span.start - item.1.start;
                    let s = item.0.get(i..).unwrap(); // SAFETY: since in range
                    match iter.next() {
                        Some(a) => item = a,
                        None => flag = false, // no more segments
                    }
                    vec.push(SpannedSegments::new(vec![(s, span.into())]));
                } else {
                    // within this segment
                    flag = false;
                    let i = span.start - item.1.start;
                    let j = span.end - item.1.start;
                    let s = item.0.get(i..j).unwrap(); // SAFETY: since i,j are in range
                    vec.push(SpannedSegments::new(vec![(s, span.into())]));
                }
            }
        }
        vec
    }
}

impl Container<SimpleSpan> for SpanCollector {
    fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }

    fn push(&mut self, span: SimpleSpan) {
        match self.0.last_mut() {
            Some(last) if last.end() == span.start() => {
                *last = SimpleSpan::new(last.context(), last.start()..span.end());
            }
            _ => self.0.push(span),
        }
    }
}

#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use crate::lexer::lex_analysis;
    use crate::syntax::syntax_analysis;

    use super::*;

    fn make_input(segs: SpannedSegments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
        let eoi = match (segs.segments.first(), segs.segments.last()) {
            (Some(first), Some(last)) => SimpleSpan::new((), first.1.start..last.1.end),
            _ => SimpleSpan::new((), 0..0),
        };
        Stream::from_iter(segs.into_spanned_chars()).map(eoi, |(t, s)| {
            // Convert our custom Span to SimpleSpan
            let simple = SimpleSpan::new((), s.start..s.end);
            (t, simple)
        })
    }

    fn parse(src: &str) -> ValueTextRef<'_> {
        let token_stream = lex_analysis(src);
        let comps = syntax_analysis::<'_, '_, _, Rich<'_, _>>(src, token_stream).unwrap();
        assert_eq!(comps.len(), 1);
        let syntax_component = comps.first().unwrap();
        assert_eq!(syntax_component.properties.len(), 1);

        let segs = syntax_component.properties.first().unwrap().value.clone();
        let stream = make_input(segs.clone());
        value_text::<'_, _, extra::Err<Rich<_>>>()
            .parse(stream)
            .into_result()
            .map(|raw_text| raw_text.build(&segs))
            .unwrap()
    }

    fn with_component(src: &str) -> String {
        format!("BEGIN:VEVENT\r\nTEST_PROP:{src}\r\nEND:VEVENT")
    }

    #[test]
    fn parses_text() {
        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.11
            (r"Project XYZ Final Review\nConference Room - 3B\nCome Prepared.",
              "Project XYZ Final Review\nConference Room - 3B\nCome Prepared."),
            // extra tests
            (r"Hello\, World\; \N", "Hello, World; \n"),
            ( r#""Quoted Text" and more text"#, r#""Quoted Text" and more text"#,),
            ("Unicode 字符串 🎉", "Unicode 字符串 🎉"),
            ("123\r\n 456\r\n\t789", "123456789"),
        ];
        for (src, expected) in success_cases {
            let src = with_component(src);
            let result = &parse(&src);
            assert_eq!(result.to_string(), expected);
        }
    }

    #[test]
    fn value_text_eq_str_ignore_ascii_case() {
        // Test basic case insensitive matching
        {
            let src = with_component("ABC");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("abc"));
            assert!(result.eq_str_ignore_ascii_case("ABC"));
            assert!(!result.eq_str_ignore_ascii_case("xyz"));
        }

        // Test with space
        {
            let src = with_component("ABC DEF");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("abc def"));
            assert!(result.eq_str_ignore_ascii_case("ABC DEF"));
        }

        // Test with mixed case
        {
            let src = with_component("Hello World");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("hello world"));
            assert!(result.eq_str_ignore_ascii_case("HELLO WORLD"));
            assert!(result.eq_str_ignore_ascii_case("HeLlO WoRlD"));
            assert!(result.eq_str_ignore_ascii_case("Hello World"));
        }

        // Test with escaped comma
        {
            let src = with_component(r"Hello\, World");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("hello, world"));
            assert!(result.eq_str_ignore_ascii_case("HELLO, WORLD"));
        }

        // Test with escaped semicolon
        {
            let src = with_component(r"Hello\; World");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("hello; world"));
            assert!(result.eq_str_ignore_ascii_case("HELLO; WORLD"));
        }

        // Test with escaped backslash
        {
            let src = with_component(r"C:\\Path");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("c:\\path"));
            assert!(result.eq_str_ignore_ascii_case("C:\\PATH"));
        }

        // Test length difference
        {
            let src = with_component("abc");
            let result = parse(&src);
            assert!(!result.eq_str_ignore_ascii_case("abcd"));
            assert!(!result.eq_str_ignore_ascii_case("ab"));
        }

        // Test with escaped newline (using \N per RFC 5545)
        {
            let src = with_component(r"Hello\NWorld");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("hello\nworld"));
            assert!(result.eq_str_ignore_ascii_case("HELLO\nWORLD"));
        }

        // Test with multiple escape sequences
        {
            let src = with_component(r"Text\, with\; \\escapes\Nand more");
            let result = parse(&src);
            assert!(result.eq_str_ignore_ascii_case("text, with; \\escapes\nand more"));
            assert!(result.eq_str_ignore_ascii_case("TEXT, WITH; \\ESCAPES\nAND MORE"));
        }
    }

    #[test]
    fn value_text_eq_str_ignore_ascii_case_empty() {
        let segs = SpannedSegments::default();
        let stream = make_input(segs.clone());
        let result = value_text::<'_, _, extra::Err<Rich<_>>>()
            .parse(stream)
            .into_result()
            .map(|raw_text| raw_text.build(&segs))
            .unwrap();

        assert!(result.eq_str_ignore_ascii_case(""));
        assert!(!result.eq_str_ignore_ascii_case("a"));
    }
}
