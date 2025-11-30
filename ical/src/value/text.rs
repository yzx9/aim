// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;

use chumsky::Parser;
use chumsky::container::Container;
use chumsky::extra::ParserExtra;
use chumsky::prelude::*;

use crate::syntax::SpannedSegments;

/// Text value type defined in RFC 5545 Section 3.3.11.
#[derive(Debug, Clone)]
pub struct ValueText<'src> {
    tokens: Vec<ValueTextToken<'src>>,
}

impl ValueText<'_> {
    /// Resolve the text value into a single string, processing escapes.
    #[must_use]
    pub fn resolve(&self) -> Cow<'_, str> {
        #[allow(clippy::indexing_slicing)]
        if self.tokens.len() == 1
            && let ValueTextToken::Str(parts) = &self.tokens[0]
            && parts.len() == 1
        {
            return Cow::Borrowed(parts[0]);
        }

        let mut s = String::new();
        for token in &self.tokens {
            match token {
                ValueTextToken::Str(parts) => {
                    s.reserve(parts.iter().map(|p| p.len()).sum());
                    for part in parts {
                        s.push_str(part);
                    }
                }
                ValueTextToken::Escape(ValueTextEscape::Backslash) => s.push('\\'),
                ValueTextToken::Escape(ValueTextEscape::Semicolon) => s.push(';'),
                ValueTextToken::Escape(ValueTextEscape::Comma) => s.push(','),
                ValueTextToken::Escape(ValueTextEscape::Newline) => s.push('\n'),
            }
        }
        Cow::Owned(s)
    }
}

#[derive(Debug, Clone)]
enum ValueTextToken<'src> {
    Str(Vec<&'src str>),
    Escape(ValueTextEscape),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueTextEscape {
    Backslash,
    Semicolon,
    Comma,
    Newline,
}

#[derive(Debug)]
pub struct RawValueText(Vec<Either<SpanCollector, ValueTextEscape>>);

impl RawValueText {
    pub fn build<'src>(self, src: &SpannedSegments<'src>) -> ValueText<'src> {
        let tokens = self
            .0
            .into_iter()
            .map(|t| match t {
                Either::Left(collector) => ValueTextToken::Str(collector.build(src)),
                Either::Right(v) => ValueTextToken::Escape(v),
            })
            .collect();

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
    fn build<'src>(self, src: &SpannedSegments<'src>) -> Vec<&'src str> {
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
                    vec.push(s);
                } else {
                    // within this segment
                    flag = false;
                    let i = span.start - item.1.start;
                    let j = span.end - item.1.start;
                    vec.push(item.0.get(i..j).unwrap()); // SAFETY: since i,j are in range
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
            Some(last) if last.end == span.start => last.end = span.end,
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
            (Some(first), Some(last)) => first.1.start..last.1.end, // is it ..?
            _ => 0..0,
        };
        Stream::from_iter(segs.into_spanned_chars()).map(eoi.into(), |(t, s)| (t, s.into()))
    }

    #[test]
    fn test_text() {
        fn parse(src: &str) -> ValueText<'_> {
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
            let src = format!("BEGIN:VEVENT\r\nTEST_PROP:{src}\r\nEND:VEVENT");
            let result = &parse(&src);
            assert_eq!(result.resolve(), expected);
        }
    }
}
