// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Parsers for property values as defined in RFC 5545 Section 3.3.

use chumsky::Parser;
use chumsky::extra::ParserExtra;
use chumsky::label::LabelError;
use chumsky::prelude::*;

use crate::value::miscellaneous::ValueExpected;

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// float      = (["+"] / "-") 1*DIGIT ["." 1*DIGIT]
/// ```
pub fn value_float<'src, I, E>() -> impl Parser<'src, I, f64, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    let sign = sign().or_not();
    let integer_part = select! { c @ '0'..='9' => c }
        .repeated()
        .at_least(1)
        .collect::<String>();

    let fractional_part = just('.').ignore_then(integer_part);

    sign.then(integer_part)
        .then(fractional_part.or_not())
        .try_map_with(|((sign, int_part), frac_part), e| {
            let capacity = sign.map_or(0, |_| 1)
                + int_part.len()
                + frac_part.as_ref().map_or(0, |f| 1 + f.len());

            let mut s = String::with_capacity(capacity);
            if let Some(sign) = sign {
                s.push(sign);
            }
            s.push_str(&int_part);
            if let Some(frac) = frac_part {
                s.push('.');
                s.push_str(&frac);
            }

            let n = match lexical::parse_partial::<f64, _>(&s) {
                Ok((f, n)) => {
                    if n < s.len() {
                        n
                    } else if f.is_infinite() || f.is_nan() {
                        0
                    } else {
                        return Ok(f);
                    }
                }
                Err(_) => 0,
            };

            Err(E::Error::expected_found(
                [ValueExpected::F64],
                Some(s.chars().nth(n).unwrap().into()), // SAFETY: since n < len
                e.span(),
            ))
        })
}

/// Float multiple values parser.
///
/// If the property permits, multiple "float" values are specified by a
/// COMMA-separated list of values.
pub fn values_float<'src, I, E>() -> impl Parser<'src, I, Vec<f64>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    values_float_impl(',')
}

/// Float multiple values parser with semicolon separator.
///
/// This parser is used for properties like GEO that use semicolon-separated
/// float values instead of the standard comma-separated format.
///
/// Format Definition:
/// ```txt
/// geovalue = float ";" float
/// ;Latitude and Longitude components
/// ```
///
/// # Example
///
/// ```text
/// GEO:37.386013;-122.083932
/// ```
#[must_use]
pub fn values_float_semicolon<'src, I, E>() -> impl Parser<'src, I, Vec<f64>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    values_float_impl(';')
}

fn values_float_impl<'src, I, E>(separator: char) -> impl Parser<'src, I, Vec<f64>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_float().separated_by(just(separator)).collect()
}

/// Format Definition:  This value type is defined by the following notation:
///
/// ```txt
/// integer    = (["+"] / "-") 1*DIGIT
/// ```
fn value_integer<'src, I, E>() -> impl Parser<'src, I, i32, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    sign()
        .or_not()
        .then(
            select! { c @ '0'..='9' => c }
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .try_map_with(|(sign, digits), e| {
            let capacity = sign.map_or(0, |_| 1) + digits.len();
            let mut int_str = String::with_capacity(capacity);
            if let Some(s) = sign {
                int_str.push(s);
            }
            int_str.push_str(&digits);

            match lexical::parse_partial::<i32, _>(&int_str) {
                Ok((v, n)) if n == int_str.len() => Ok(v),
                Ok((_, n)) => Err(E::Error::expected_found(
                    [ValueExpected::I32],
                    Some(int_str.chars().nth(n).unwrap().into()), // SAFETY: since n < len
                    e.span(),
                )),
                Err(_) => Err(E::Error::expected_found(
                    [ValueExpected::I32],
                    Some(int_str.chars().next().unwrap().into()), // SAFETY: since at least 1 digit
                    e.span(),
                )),
            }
        })
}

/// Integer multiple values parser.
///
/// If the property permits, multiple "integer" values are specified by a
/// COMMA-separated list of values.
pub fn values_integer<'src, I, E>() -> impl Parser<'src, I, Vec<i32>, E>
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, ValueExpected>,
{
    value_integer().separated_by(just(',')).collect()
}

const fn sign<'src, I, E>() -> impl Parser<'src, I, char, E> + Copy
where
    I: Input<'src, Token = char, Span = SimpleSpan>,
    E: ParserExtra<'src, I>,
{
    select! { c @ ('+' | '-') => c }
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;

    use super::*;

    #[test]
    #[expect(clippy::approx_constant)]
    fn parses_float() {
        fn parse(src: &str) -> Result<f64, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_float::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        let success_cases = [
            // examples from RFC 5545 Section 3.3.7
            ("1000000.0000001", 1_000_000.000_000_1),
            ("1.333", 1.333),
            ("-3.14", -3.14),
            // extra tests
            ("123.456", 123.456),
            ("-987.654", -987.654),
            ("+0.001", 0.001),
            ("42", 42.0),
            ("+3.14", 3.14),
            ("-0.0", -0.0),
            ("0", 0.0),
            ("+0", 0.0),
            ("-1234567890.0987654321", -1_234_567_890.098_765_4), // precision limit, last digit rounded
        ];
        for (src, expected) in success_cases {
            let f = parse(src).unwrap();
            assert!((f - expected).abs() < 1e-5);
        }

        let infinity = (0..=f64::MAX_10_EXP).map(|_| '9').collect::<String>();
        let fail_cases = [
            &infinity,  // infinity
            "nan",      // RFC5545 does not allow non-numeric values
            "infinity", // RFC5545 does not allow non-numeric values
            "+.",       // missing digits
            "-.",       // missing digits
            ".",        // missing digits
            "",         // empty string
            "12a34",    // invalid character
            // Scientific notation is NOT allowed by RFC 5545 Section 3.3.7
            // Format: float = (["+"] / "-") 1*DIGIT ["." 1*DIGIT]
            "1e10",    // scientific notation with 'e'
            "1.5e10",  // scientific notation with 'e' and decimal
            "2E-3",    // scientific notation with uppercase 'E' and negative exponent
            "1.23e+5", // scientific notation with 'e' and explicit plus
            "3E",      // scientific notation with 'E' but no exponent
            "1.5e",    // scientific notation with 'e' but no exponent
            "-2.5e3",  // scientific notation with negative sign
            "+1.2E10", // scientific notation with plus sign and uppercase 'E'
            "0e0",     // scientific notation with zero values
            ".5e10",   // scientific notation with leading dot
            "1.e10",   // scientific notation with trailing dot
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }

    #[test]
    fn parses_integer() {
        fn parse(src: &str) -> Result<i32, Vec<Rich<'_, char>>> {
            let stream = Stream::from_iter(src.chars());
            value_integer::<'_, _, extra::Err<_>>()
                .parse(stream)
                .into_result()
        }

        #[rustfmt::skip]
        let success_cases = [
            // examples from RFC 5545 Section 3.3.8
            ("1234567890", 1_234_567_890),
            ("-1234567890", -1_234_567_890),
            ("+1234567890", 1_234_567_890),
            ("432109876", 432_109_876),
            // extra tests
            ( "0", 0),
            ("+0", 0),
            ("-0", 0),
            ("+0000000000000000000000", 0), // long zero
            ("12345", 12345),
            ("-6789", -6789),
            ("+2147483647",  2_147_483_647), // i32 max
            ("-2147483648", -2_147_483_648), // i32 min
        ];
        for (src, expected) in success_cases {
            assert_eq!(parse(src).unwrap(), expected);
        }

        let fail_cases = [
            "nan",                   // RFC5545 does not allow non-numeric values
            "infinity",              // RFC5545 does not allow non-numeric values
            "+2147483648",           // i32 max + 1
            "-2147483649",           // i32 min - 1
            "12345678901234567890",  // overflow, too long
            "-12345678901234567890", // underflow, too long
            "+",                     // missing digits
            "-",                     // missing digits
            "",                      // empty string
            "12a34",                 // invalid character
        ];
        for src in fail_cases {
            assert!(parse(src).is_err(), "Parse {src} should fail");
        }
    }
}
