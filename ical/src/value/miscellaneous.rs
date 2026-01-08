// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use std::borrow::Cow;

use chumsky::Parser;
use chumsky::error::RichPattern;
use chumsky::extra::ParserExtra;
use chumsky::input::Input;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

/// Failure reasons when a specific value type was expected but not found.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueExpected {
    /// A date value was expected
    Date,
    /// A 64-bit floating-point value was expected
    F64,
    /// A 32-bit signed integer value was expected
    I32,
    /// A 32-bit unsigned integer value was expected
    U32,
    /// Period date-times must have consistent timezone (both UTC or both floating)
    MismatchedTimezone,
}

impl From<ValueExpected> for RichPattern<'_, char> {
    fn from(expected: ValueExpected) -> Self {
        match expected {
            ValueExpected::Date => Self::Label(Cow::Borrowed("invalid date")),
            ValueExpected::F64 => Self::Label(Cow::Borrowed("f64 out of range")),
            ValueExpected::I32 => Self::Label(Cow::Borrowed("i32 out of range")),
            ValueExpected::U32 => Self::Label(Cow::Borrowed("u32 out of range")),
            ValueExpected::MismatchedTimezone => Self::Label(Cow::Borrowed(
                "period date-times must have consistent timezone",
            )),
        }
    }
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// binary     = *(4b-char) [b-end]
/// ; A "BASE64" encoded character string, as defined by [RFC4648].
///
/// b-end      = (2b-char "==") / (3b-char "=")
///
/// b-char = ALPHA / DIGIT / "+" / "/"
/// ```
pub fn value_binary<'src, I, E>() -> impl Parser<'src, I, (), E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // b-char = ALPHA / DIGIT / "+" / "/"
    let b_char = select! {
        'A'..='Z' => (),
        'a'..='z' => (),
        '0'..='9' => (),
        '+' => (),
        '/' => (),
    };

    // 4b-char
    let quartet = b_char.repeated().exactly(4).ignored();

    // b-end
    let two_eq = just('=').then_ignore(just('='));
    let one_eq = just('=');

    // b-end = (2b-char "==") / (3b-char "=")
    let b_end = b_char
        .repeated()
        .exactly(2)
        .ignored()
        .then_ignore(two_eq)
        .or(b_char.repeated().exactly(3).ignored().then_ignore(one_eq))
        .ignored();

    // *(4b-char) [b-end]
    quartet
        .repeated() // allow zero quartets
        .ignore_then(b_end.or_not())
        .ignored()
        .then_ignore(end())
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// boolean    = "TRUE" / "FALSE"
/// ```
///
/// Description:  These values are case-insensitive text.  No additional
///    content value encoding (i.e., BACKSLASH character encoding, see
///    Section 3.3.11) is defined for this value type.
pub fn value_boolean<'src, I, E>() -> impl Parser<'src, I, bool, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    // case-insensitive
    let t = choice((just('T'), just('t')))
        .ignore_then(choice((just('R'), just('r'))))
        .ignore_then(choice((just('U'), just('u'))))
        .ignore_then(choice((just('E'), just('e'))))
        .ignored()
        .to(true);

    let f = choice((just('F'), just('f')))
        .ignore_then(choice((just('A'), just('a'))))
        .ignore_then(choice((just('L'), just('l'))))
        .ignore_then(choice((just('S'), just('s'))))
        .ignore_then(choice((just('E'), just('e'))))
        .ignored()
        .to(false);

    choice((t, f))
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use super::*;

    #[test]
    fn parses_binary() {
        fn check(src: &str) -> Result<(), Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_binary::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }
        let success_cases = [
            // examples from RFC 5545 Section 3.1.3
            // Original text include a typo (ignore the padding): https://www.rfc-editor.org/errata/eid5602
            "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4=",
            // examples from RFC 5545 Section 3.3.1
            "\
AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAA\
ACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAERE\
AAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAA\
",
            // extra tests
            "TWFu",     // "Man"
            "QUJDREVG", // "ABCDEF"
            "AAAA",     // all zero bytes
            "+/9a",     // bytes with high bits set
            "ZgZg",     // "ff"
            "TQ==",     // "M"
            "TWE=",     // "Ma"
            "SGVsbG8=", // "Hello"
        ];
        for src in success_cases {
            assert!(check(src).is_ok(), "Parse {src} should succeed");
        }

        let fail_cases = [
            "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4",
            "TQ===",   // invalid length
            "TWFu=",   // invalid length
            "TWFuA",   // invalid length
            "TWFu===", // invalid length
            "T@Fu",    // invalid character
        ];
        for src in fail_cases {
            assert!(check(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn parses_boolean() {
        fn parse(src: &str) -> Result<bool, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_boolean::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        for (src, expected) in [
            ("TRUE", true),
            ("True", true),
            ("true", true),
            ("FALSE", false),
            ("False", false),
            ("false", false),
        ] {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            "True ", " FALSE", "T RUE", "FA LSE", "1", "0", "YES", "NO", "",
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }
}
