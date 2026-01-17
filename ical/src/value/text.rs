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

use crate::string_storage::{SegmentedSpannedChars, Segments, Span, StringStorage};

/// Text value type defined in RFC 5545 Section 3.3.11.
#[derive(Default, Debug, Clone)]
pub struct ValueText<S: StringStorage> {
    tokens: Vec<(ValueTextToken<S>, S::Span)>,
}

impl<'a> ValueText<Segments<'a>> {
    /// Resolve the text value into a single string, processing escapes.
    ///
    /// This version tries to avoid allocation when there's only a single string token.
    #[must_use]
    pub fn resolve(&self) -> Cow<'a, str> {
        #[expect(clippy::indexing_slicing)]
        if self.tokens.len() == 1
            && let (ValueTextToken::Str(part), _) = &self.tokens[0]
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

        for (token, _) in &self.tokens {
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
    pub fn to_owned(&self) -> ValueText<String> {
        ValueText {
            tokens: self
                .tokens
                .iter()
                .map(|(token, _)| match token {
                    ValueTextToken::Str(s) => ValueTextToken::Str(s.to_owned()),
                    ValueTextToken::Escape(c) => ValueTextToken::Escape(*c),
                })
                .map(|token| (token, ()))
                .collect(),
        }
    }

    /// Get the full span from the first to the last token.
    ///
    /// This method provides O(1) access to the span that covers all tokens
    /// in the `ValueText`, from the first character to the last.
    #[must_use]
    pub fn span(&self) -> Span {
        if self.tokens.is_empty() {
            Span { start: 0, end: 0 }
        } else {
            #[expect(clippy::indexing_slicing)]
            let first = &self.tokens[0].1;
            #[expect(clippy::indexing_slicing)]
            let last = &self.tokens[self.tokens.len() - 1].1;
            Span {
                start: first.start,
                end: last.end,
            }
        }
    }

    /// Create an iterator over characters with their spans.
    ///
    /// This method provides a zero-copy iterator that yields each character
    /// along with its source position, enabling accurate error reporting.
    #[must_use]
    pub fn into_spanned_chars(self) -> ValueTextSpannedChars<'a> {
        ValueTextSpannedChars {
            tokens: self.tokens.into_iter(),
            current_segments: None,
            current_escape: None,
        }
    }
}

/// Iterator over characters in a `ValueText` with their spans.
///
/// This struct is created by `ValueText::into_spanned_chars()` and yields
/// characters along with their source positions.
///
/// # Lifetime
///
/// The lifetime parameter `'a` represents the lifetime of the underlying
/// string data in the original `ValueText`.
#[derive(Debug)]
pub struct ValueTextSpannedChars<'a> {
    /// Remaining tokens to process
    tokens: std::vec::IntoIter<(ValueTextToken<Segments<'a>>, Span)>,
    /// Current segment spanned chars iterator (if processing a Str token)
    current_segments: Option<SegmentedSpannedChars<'a>>,
    /// Current escape char (if processing an Escape token)
    current_escape: Option<(char, Span)>,
}

impl Iterator for ValueTextSpannedChars<'_> {
    type Item = (char, Span);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get next char from current segments iterator
            if let Some(ref mut iter) = self.current_segments {
                if let Some(item) = iter.next() {
                    return Some(item);
                }
                self.current_segments = None;
            }

            // Try to get next char from current escape
            if let Some(item) = self.current_escape.take() {
                return Some(item);
            }

            // Get next token
            let (token, span) = self.tokens.next()?;

            match token {
                ValueTextToken::Str(segments) => {
                    self.current_segments = Some(segments.into_spanned_chars());
                }
                ValueTextToken::Escape(escape_char) => {
                    let c = escape_char.as_ref().chars().next().unwrap();
                    self.current_escape = Some((c, span));
                }
            }
        }
    }
}

impl<S: StringStorage> fmt::Display for ValueText<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (token, _) in &self.tokens {
            match token {
                ValueTextToken::Str(part) => write!(f, "{part}")?,
                ValueTextToken::Escape(c) => write!(f, "{c}")?,
            }
        }
        Ok(())
    }
}

impl ValueText<String> {
    /// Create a new `ValueText<String>` from a string.
    ///
    /// This constructor is provided for convenient construction of owned text values.
    /// The input string is treated as a single unescaped text token.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self {
            tokens: vec![(ValueTextToken::Str(value), ())],
        }
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
pub struct RawValueText(Vec<Either<SpanCollector, (ValueTextEscape, SimpleSpan)>>);

impl RawValueText {
    pub fn build<'src>(self, src: &Segments<'src>) -> ValueText<Segments<'src>> {
        let size = self.0.iter().fold(0, |acc, t| match t {
            Either::Left(collector) => acc + collector.0.len(),
            Either::Right(_) => acc + 1,
        });

        let mut tokens = Vec::with_capacity(size);
        for t in self.0 {
            match t {
                Either::Left(collector) => tokens.extend(
                    collector
                        .build(src)
                        .into_iter()
                        .map(|(s, span)| (ValueTextToken::Str(s), span.into())),
                ),
                Either::Right((v, span)) => tokens.push((ValueTextToken::Escape(v), span.into())),
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
        .map_with(|v, e| (v, e.span()))
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
    fn build<'src>(self, src: &Segments<'src>) -> Vec<(Segments<'src>, SimpleSpan)> {
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
                    let i = span.start.saturating_sub(item.1.start);
                    let s = item.0.get(i..).unwrap(); // SAFETY: since in range
                    match iter.next() {
                        Some(a) => item = a,
                        None => flag = false, // no more segments
                    }
                    vec.push((Segments::new(vec![(s, span.into())]), span));
                } else {
                    // within this segment
                    flag = false;
                    let i = span.start.saturating_sub(item.1.start);
                    let j = span.end.saturating_sub(item.1.start);
                    let s = item.0.get(i..j).unwrap(); // SAFETY: since i,j are in range
                    vec.push((Segments::new(vec![(s, span.into())]), span));
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

    use crate::syntax::syntax_analysis;

    use super::*;

    fn make_input(segs: Segments<'_>) -> impl Input<'_, Token = char, Span = SimpleSpan> {
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

    fn parse(src: &str) -> ValueText<Segments<'_>> {
        let comps = syntax_analysis(src).unwrap();
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
        let segs = Segments::default();
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
