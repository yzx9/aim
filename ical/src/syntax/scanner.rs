// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Scanner for iCalendar content lines.
//!
//! This module provides a linear scanner that converts token streams into
//! content lines. It serves as an experimental alternative to the chumsky-based
//! syntax parser, with simpler error recovery and more granular error reporting.
//!
//! # Architecture
//!
//! ```text
//! Source Text → Lexer → Token Stream → Scanner → Content Lines
//! ```
//!
//! # Content Line Format
//!
//! Per RFC 5545, a content line has the format:
//! ```text
//! contentline = name *(";" param) ":" value CRLF
//! ```
//!
//! # Example
//!
//! ```ignore
//! use aimcal_ical::lexer::lex_analysis;
//! use aimcal_ical::scanner::scan_content_lines;
//!
//! let src = "DTSTART;TZID=America/New_York:20250101T090000\r\n";
//! let tokens = lex_analysis(src);
//! let result = scan_content_lines(src, tokens);
//!
//! assert_eq!(result.lines.len(), 1);
//! assert!(!result.has_errors);
//! ```

use std::fmt;
use std::iter::Peekable;

use crate::string_storage::{Span, SpannedSegments};
use crate::syntax::lexer::{SpannedToken, Token};

/// A scanned iCalendar content line.
///
/// Represents a single content line per RFC 5545:
/// ```text
/// contentline = name *(";" param) ":" value CRLF
/// ```
#[derive(Debug, Clone)]
pub struct ContentLine<'src> {
    /// Property name (e.g., "DTSTART", "SUMMARY")
    pub name: SpannedSegments<'src>,

    /// Property parameters (semicolon-separated)
    pub parameters: Vec<ScannedParameter<'src>>,

    /// Property value
    pub value: SpannedSegments<'src>,

    /// Span of the entire content line (from name start to newline end)
    pub span: Span,

    /// Error information if parsing this line failed
    pub error: Option<ContentLineError>,
}

impl ContentLine<'_> {
    /// Check if this content line is valid (no errors).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.error.is_none()
    }
}

/// A scanned parameter from a content line.
///
/// Parameters have the format: `name=value` or `name=value1,value2`
#[derive(Debug, Clone)]
pub struct ScannedParameter<'src> {
    /// Parameter name (e.g., "TZID", "VALUE")
    pub name: SpannedSegments<'src>,

    /// Parameter values (comma-separated)
    pub values: Vec<ScannedParameterValue<'src>>,

    /// Span of the entire parameter
    pub span: Span,
}

/// A single scanned parameter value.
#[derive(Debug, Clone)]
pub struct ScannedParameterValue<'src> {
    /// The parameter value
    pub value: SpannedSegments<'src>,

    /// Whether the value was quoted in the source
    pub quoted: bool,

    /// Span of this value
    pub span: Span,
}

/// Errors that can occur when scanning a content line.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContentLineError {
    /// Missing colon separator.
    ///
    /// Example: `PROPNAME value` instead of `PROPNAME:value`
    MissingColon {
        /// Span where colon was expected
        expected_at: Span,
        /// Description of what was found instead
        found: Option<String>,
    },

    /// Empty content line (no name).
    EmptyLine {
        /// Span of the empty line
        span: Span,
    },

    /// Invalid parameter syntax.
    InvalidParameter {
        /// Span of the invalid parameter
        span: Span,
        /// Specific error details
        kind: ParameterErrorKind,
    },

    /// Malformed line.
    MalformedLine {
        /// Span of the malformed content
        span: Span,
        /// Description of the issue
        message: String,
    },
}

impl fmt::Display for ContentLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentLineError::MissingColon { .. } => write!(f, "missing colon in property"),
            ContentLineError::EmptyLine { .. } => write!(f, "empty content line"),
            ContentLineError::InvalidParameter { kind, .. } => {
                let msg = match kind {
                    ParameterErrorKind::MissingEquals => "missing equals in parameter",
                    ParameterErrorKind::MissingValue => "missing parameter value",
                    ParameterErrorKind::EmptyName => "empty parameter name",
                    ParameterErrorKind::UnterminatedQuote => "unterminated quoted string",
                };
                write!(f, "{msg}")
            }
            ContentLineError::MalformedLine { message, .. } => write!(f, "{message}"),
        }
    }
}

/// Specific parameter parsing errors.
#[derive(Debug, Clone, Copy)]
pub enum ParameterErrorKind {
    /// Missing equals sign.
    ///
    /// Example: `TZID America/New_York`
    MissingEquals,

    /// Missing parameter value.
    ///
    /// Example: `TZID=`
    MissingValue,

    /// Empty parameter name.
    ///
    /// Example: `;=value`
    EmptyName,

    /// Unterminated quoted string.
    ///
    /// Example: `PARAM="unclosed value`
    UnterminatedQuote,
}

/// Result of scanning content lines.
#[derive(Debug, Clone)]
pub struct ScanResult<'src> {
    /// All scanned content lines (including ones with errors)
    pub lines: Vec<ContentLine<'src>>,

    /// Whether any errors were encountered
    pub has_errors: bool,
}

/// Scan a token stream into content lines.
///
/// This function converts a token stream into a vector of content lines,
/// parsing each line's structure (name, parameters, value). Errors are
/// included in the `ContentLine` rather than causing failure, enabling
/// graceful error recovery.
///
/// # Arguments
///
/// * `src` - The original source text
/// * `tokens` - Iterator over tokens with spans
///
/// # Returns
///
/// A `ScanResult` containing all content lines (valid and invalid)
///
/// # Example
///
/// ```ignore
/// use aimcal_ical::lexer::lex_analysis;
/// use aimcal_ical::scanner::scan_content_lines;
///
/// let src = "VERSION:2.0\r\nPRODID:-//Example Corp.//CalDAV Client//EN\r\n";
/// let tokens = lex_analysis(src);
/// let result = scan_content_lines(src, tokens);
///
/// assert_eq!(result.lines.len(), 2);
/// assert!(!result.has_errors);
/// ```
pub fn scan_content_lines<'src>(
    src: &'src str,
    tokens: impl IntoIterator<Item = SpannedToken<'src>>,
) -> ScanResult<'src> {
    let mut token_iter = tokens.into_iter().peekable();
    let mut lines = Vec::new();
    let mut has_errors = false;

    while token_iter.peek().is_some() {
        match scan_one_content_line(src, &mut token_iter) {
            Some(line) => {
                if line.error.is_some() {
                    has_errors = true;
                }
                lines.push(line);
            }
            None => break, // End of input
        }
    }

    ScanResult { lines, has_errors }
}

/// Scan a single content line from tokens.
///
/// Returns `None` if we've reached end of input.
fn scan_one_content_line<'src>(
    src: &'src str,
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
) -> Option<ContentLine<'src>> {
    // Peek at first token to determine if we have content
    let first_token = *tokens.peek()?;

    // Track line start span
    let line_start = first_token.1;

    // Check if this is an empty line (just newline)
    if matches!(first_token.0, Token::Newline) {
        let newline = tokens.next()?;
        return Some(ContentLine {
            name: SpannedSegments::default(),
            parameters: Vec::new(),
            value: SpannedSegments::default(),
            span: newline.1,
            error: Some(ContentLineError::EmptyLine { span: newline.1 }),
        });
    }

    // Parse: name [;param]* : value \r\n
    let result = parse_content_line_structure(src, tokens, line_start);

    Some(result)
}

/// Parse the structure of a content line.
fn parse_content_line_structure<'src>(
    src: &'src str,
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
    line_start: Span,
) -> ContentLine<'src> {
    // Parse name (sequence of Word tokens)
    let (name, _name_span) = parse_property_name(tokens);

    // Parse parameters (semicolon-separated)
    let mut parameters = Vec::new();
    while let Some(&SpannedToken(Token::Semicolon, semi_span)) = tokens.peek() {
        tokens.next(); // consume semicolon

        match parse_parameter(src, tokens) {
            Ok(param) => parameters.push(param),
            Err(err) => {
                // Create error line but consume remaining tokens until newline to avoid infinite loop
                consume_until_newline(tokens);
                let line_end = get_current_end(tokens);
                return ContentLine {
                    name,
                    parameters,
                    value: SpannedSegments::default(),
                    span: Span::new(line_start.start, line_end),
                    error: Some(ContentLineError::InvalidParameter {
                        span: semi_span,
                        kind: err,
                    }),
                };
            }
        }
    }

    // Expect colon
    let colon_span = match tokens.peek() {
        Some(&SpannedToken(Token::Colon, span)) => {
            tokens.next(); // consume colon
            span
        }
        Some(&SpannedToken(token, span)) => {
            // Missing colon error - consume remaining tokens until newline to avoid infinite loop
            consume_until_newline(tokens);
            return ContentLine {
                name,
                parameters,
                value: SpannedSegments::default(),
                span: Span::new(line_start.start, span.end),
                error: Some(ContentLineError::MissingColon {
                    expected_at: span,
                    found: Some(format!("{token:?}")),
                }),
            };
        }
        // Unexpected end of input
        None => {
            return ContentLine {
                name,
                parameters,
                value: SpannedSegments::default(),
                span: line_start,
                error: Some(ContentLineError::MissingColon {
                    expected_at: Span::new(line_start.end, line_start.end + 1),
                    found: None,
                }),
            };
        }
    };

    // Parse value (everything until newline)
    let (value, value_end) = parse_value(tokens);

    // Expect and consume newline
    match tokens.next() {
        Some(SpannedToken(Token::Newline, newline_span)) => {
            let line_end = newline_span.end;

            ContentLine {
                name,
                parameters,
                value,
                span: Span::new(line_start.start, line_end),
                error: None,
            }
        }
        // Missing newline - still return content line with what we have
        Some(token) => ContentLine {
            name,
            parameters,
            value,
            span: Span::new(line_start.start, token.1.end),
            error: None, // Not a fatal error, just malformed
        },
        None => {
            // End of input without newline
            let end = if value_end > 0 {
                value_end
            } else {
                colon_span.end
            };
            ContentLine {
                name,
                parameters,
                value,
                span: Span::new(line_start.start, end),
                error: None,
            }
        }
    }
}

/// Parse property name from tokens.
///
/// Returns [`SpannedSegments`] and [`Span`] of the property name.
fn parse_property_name<'src>(
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
) -> (SpannedSegments<'src>, Span) {
    let mut segments = Vec::new();
    let mut start = None;
    let mut end = None;

    // Collect consecutive Word tokens (property names can be hyphenated like "PERCENT-COMPLETE")
    while let Some(&SpannedToken(Token::Word(text), span)) = tokens.peek() {
        if start.is_none() {
            start = Some(span.start);
        }
        end = Some(span.end);
        segments.push((text, span));
        tokens.next(); // consume the token
    }

    if segments.is_empty() {
        (SpannedSegments::default(), Span::new(0, 0))
    } else {
        let name_span = Span::new(start.unwrap(), end.unwrap());
        (SpannedSegments::new(segments), name_span)
    }
}

/// Parse a single parameter.
///
/// Format: `name=value` or `name=value1,value2`
fn parse_parameter<'src>(
    _src: &'src str,
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
) -> Result<ScannedParameter<'src>, ParameterErrorKind> {
    let param_start = match tokens.peek() {
        Some(&SpannedToken(_, span)) => span.start,
        None => return Err(ParameterErrorKind::EmptyName),
    };

    // Parse parameter name
    let mut name_segments = Vec::new();
    let mut name_end = None;

    while let Some(&SpannedToken(Token::Word(text), span)) = tokens.peek() {
        name_end = Some(span.end);
        name_segments.push((text, span));
        tokens.next(); // consume the token
    }

    if name_segments.is_empty() {
        return Err(ParameterErrorKind::EmptyName);
    }

    let name = SpannedSegments::new(name_segments);

    // Expect equals sign
    match tokens.next() {
        Some(SpannedToken(Token::Equal, _)) => {}
        Some(_) | None => return Err(ParameterErrorKind::MissingEquals),
    }

    // Parse parameter values (comma-separated)
    let mut values = Vec::new();
    loop {
        match parse_parameter_value(tokens) {
            Ok(Some(value)) => values.push(value),
            Ok(None) => break,
            Err(err) => return Err(err),
        }

        // Check for comma separator
        match tokens.peek() {
            Some(&SpannedToken(Token::Comma, _)) => {
                tokens.next(); // consume comma
            }
            _ => break,
        }
    }

    if values.is_empty() {
        return Err(ParameterErrorKind::MissingValue);
    }

    let param_end = values
        .last()
        .map(|v| v.span.end)
        .or(name_end)
        .unwrap_or(param_start);

    Ok(ScannedParameter {
        name,
        values,
        span: Span::new(param_start, param_end),
    })
}

/// Parse a single parameter value.
///
/// Returns `Ok(None)` if there's no value to parse.
fn parse_parameter_value<'src>(
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
) -> Result<Option<ScannedParameterValue<'src>>, ParameterErrorKind> {
    let start = match tokens.peek() {
        Some(&SpannedToken(_, span)) => span.start,
        None => return Ok(None),
    };

    // Check if quoted
    let quoted = match tokens.peek() {
        Some(&SpannedToken(Token::DQuote, _)) => {
            tokens.next(); // consume opening quote
            true
        }
        _ => false,
    };

    // Collect value tokens
    let mut segments = Vec::new();

    if quoted {
        // Collect until closing quote
        loop {
            match tokens.next() {
                // End of quoted string
                Some(SpannedToken(Token::DQuote, span)) => {
                    return Ok(Some(ScannedParameterValue {
                        value: SpannedSegments::new(segments),
                        quoted: true,
                        span: Span::new(start, span.end),
                    }));
                }
                Some(token) => {
                    let text = token_to_text(token.0);
                    if !text.is_empty() {
                        segments.push((text, token.1));
                    }
                }
                None => return Err(ParameterErrorKind::UnterminatedQuote),
            }
        }
    } else {
        // Collect until separator (semicolon, colon, comma, equals, newline)
        while let Some(token) = tokens.peek() {
            match token.0 {
                Token::Semicolon | Token::Colon | Token::Comma | Token::Equal | Token::Newline => {
                    break;
                }
                _ => {
                    let token = tokens.next().unwrap();
                    let text = token_to_text(token.0);
                    if !text.is_empty() {
                        segments.push((text, token.1));
                    }
                }
            }
        }

        if segments.is_empty() {
            return Ok(None);
        }

        let end = segments.last().map_or(start, |(_, s)| s.end);
        Ok(Some(ScannedParameterValue {
            value: SpannedSegments::new(segments),
            quoted: false,
            span: Span::new(start, end),
        }))
    }
}

/// Parse value content (everything until newline).
///
/// Returns ([`SpannedSegments`], `usize`) - value segments and end position.
fn parse_value<'src>(
    tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'src>>>,
) -> (SpannedSegments<'src>, usize) {
    let mut segments = Vec::new();
    let mut end = 0;

    // Collect all tokens until newline
    while let Some(token) = tokens.peek() {
        if matches!(token.0, Token::Newline) {
            break;
        }

        let token = tokens.next().unwrap();
        end = token.1.end;

        let text = token_to_text(token.0);
        if !text.is_empty() {
            segments.push((text, token.1));
        }
    }

    if segments.is_empty() {
        return (SpannedSegments::default(), 0);
    }

    (SpannedSegments::new(segments), end)
}

/// Get the current end position from token iterator.
fn get_current_end<'a>(tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'a>>>) -> usize {
    tokens.peek().map_or(0, |SpannedToken(_, span)| span.end)
}

/// Consume all tokens until a newline is found (including the newline).
///
/// This is used in error recovery to avoid infinite loops when errors occur.
fn consume_until_newline<'a>(tokens: &mut Peekable<impl Iterator<Item = SpannedToken<'a>>>) {
    for SpannedToken(token, _) in tokens.by_ref() {
        if matches!(token, Token::Newline) {
            break;
        }
    }
}

/// Convert a token to its text representation.
fn token_to_text(token: Token<'_>) -> &str {
    match token {
        Token::Word(s) | Token::Symbol(s) | Token::UnicodeText(s) => s,
        Token::Comma => ",",
        Token::Colon => ":",
        Token::Semicolon => ";",
        Token::Equal => "=",
        Token::DQuote => "\"",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::indexing_slicing)]

    use std::borrow::Cow;

    use logos::Logos;

    use super::*;
    use crate::string_storage::Span;
    use crate::syntax::lexer::{SpannedToken, Token};

    fn test_scan(src: &str) -> ScanResult<'_> {
        let tokens: Vec<_> = Token::lexer(src)
            .spanned()
            .map(|(tok, span)| {
                let span = Span {
                    start: span.start,
                    end: span.end,
                };
                match tok {
                    Ok(tok) => SpannedToken(tok, span),
                    Err(()) => SpannedToken(Token::Error, span),
                }
            })
            .collect();
        scan_content_lines(src, tokens)
    }

    fn component_name<'src>(line: &ContentLine<'src>) -> Option<Cow<'src, str>> {
        let is_begin = line.name.eq_str_ignore_ascii_case("BEGIN");
        let is_end = line.name.eq_str_ignore_ascii_case("END");
        if (is_begin || is_end) && !line.value.is_empty() {
            Some(line.value.resolve())
        } else {
            None
        }
    }

    #[test]
    fn scanner_valid_simple_property() {
        let src = "SUMMARY:Team Meeting\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(!result.has_errors);

        let line = &result.lines[0];
        assert_eq!(line.name.to_owned(), "SUMMARY");
        assert!(line.parameters.is_empty());
        assert_eq!(line.value.to_owned(), "Team Meeting");
        assert!(line.is_valid());
    }

    #[test]
    fn scanner_property_with_single_parameter() {
        let src = "DTSTART;TZID=America/New_York:20250101T090000\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(!result.has_errors);

        let line = &result.lines[0];
        assert_eq!(line.name.to_owned(), "DTSTART");
        assert_eq!(line.parameters.len(), 1);
        assert_eq!(line.parameters[0].name.to_owned(), "TZID");
        assert_eq!(line.parameters[0].values.len(), 1);
        assert_eq!(
            line.parameters[0].values[0].value.to_owned(),
            "America/New_York"
        );
        assert!(!line.parameters[0].values[0].quoted);
        assert_eq!(line.value.to_owned(), "20250101T090000");
    }

    #[test]
    fn scanner_property_with_multiple_parameters() {
        let src =
            "ATTENDEE;RSVP=TRUE;CUTYPE=INDIVIDUAL;ROLE=REQ-PARTICIPANT:mailto:test@example.com\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(!result.has_errors);

        let line = &result.lines[0];
        assert_eq!(line.name.to_owned(), "ATTENDEE");
        assert_eq!(line.parameters.len(), 3);

        assert_eq!(line.parameters[0].name.to_owned(), "RSVP");
        assert_eq!(line.parameters[1].name.to_owned(), "CUTYPE");
        assert_eq!(line.parameters[2].name.to_owned(), "ROLE");
    }

    #[test]
    fn scanner_quoted_parameter_value() {
        let src = "X-CUSTOM;PARAM=\"value with spaces\":test\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(!result.has_errors);

        let param = &result.lines[0].parameters[0];
        assert!(param.values[0].quoted);
        assert_eq!(param.values[0].value.to_owned(), "value with spaces");
    }

    #[test]
    fn scanner_begin_end_lines() {
        let src = "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 2);
        assert!(!result.has_errors);

        assert!(result.lines[0].name.eq_str_ignore_ascii_case("BEGIN"));
        assert_eq!(
            component_name(&result.lines[0]).as_deref(),
            Some("VCALENDAR")
        );

        assert!(result.lines[1].name.eq_str_ignore_ascii_case("END"));
        assert_eq!(
            component_name(&result.lines[1]).as_deref(),
            Some("VCALENDAR")
        );
    }

    #[test]
    fn scanner_multiple_content_lines() {
        let src =
            "VERSION:2.0\r\nPRODID:-//Example Corp.//CalDAV Client//EN\r\nCALSCALE:GREGORIAN\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 3);
        assert!(!result.has_errors);

        assert_eq!(result.lines[0].name.to_owned(), "VERSION");
        assert_eq!(result.lines[1].name.to_owned(), "PRODID");
        assert_eq!(result.lines[2].name.to_owned(), "CALSCALE");
    }

    #[test]
    fn scanner_missing_colon() {
        let src = "VERSION 2.0\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(result.has_errors);

        let line = &result.lines[0];
        assert!(line.error.is_some());

        match &line.error {
            Some(ContentLineError::MissingColon { .. }) => {}
            _ => panic!("Expected MissingColon error, got {:?}", line.error),
        }
    }

    #[test]
    fn scanner_empty_value() {
        let src = "SUMMARY:\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        // Empty value is valid (not an error)
        assert!(result.lines[0].is_valid());
        assert_eq!(result.lines[0].value.to_owned(), "");
    }

    #[test]
    fn scanner_invalid_parameter_missing_equals() {
        let src = "DTSTART;TZID:20250101\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(result.has_errors);

        let line = &result.lines[0];
        match &line.error {
            Some(ContentLineError::InvalidParameter { kind, .. }) => {
                assert!(matches!(kind, ParameterErrorKind::MissingEquals));
            }
            _ => panic!("Expected InvalidParameter error, got {:?}", line.error),
        }
    }

    #[test]
    fn scanner_parameter_empty_value() {
        let src = "DTSTART;TZID=:20250101\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(result.has_errors);

        match &result.lines[0].error {
            Some(ContentLineError::InvalidParameter {
                kind: ParameterErrorKind::MissingValue,
                ..
            }) => {}
            _ => panic!(
                "Expected MissingValue error, got {:?}",
                result.lines[0].error
            ),
        }
    }

    #[test]
    fn scanner_unterminated_quote() {
        let src = "X-CUSTOM;PARAM=\"unclosed:value\r\n";
        let result = test_scan(src);

        assert_eq!(result.lines.len(), 1);
        assert!(result.has_errors);

        match &result.lines[0].error {
            Some(ContentLineError::InvalidParameter {
                kind: ParameterErrorKind::UnterminatedQuote,
                ..
            }) => {}
            _ => panic!(
                "Expected UnterminatedQuote error, got {:?}",
                result.lines[0].error
            ),
        }
    }

    #[test]
    fn scanner_unicode_in_value() {
        let src = "SUMMARY:Team会议📅\r\n";
        let result = test_scan(src);

        assert!(!result.has_errors);
        assert_eq!(result.lines[0].value.to_owned(), "Team会议📅");
    }

    #[test]
    fn scanner_hyphenated_property_name() {
        let src = "PERCENT-COMPLETE:75\r\n";
        let result = test_scan(src);

        assert!(!result.has_errors);
        assert_eq!(result.lines[0].name.to_owned(), "PERCENT-COMPLETE");
    }

    #[test]
    fn scanner_x_property() {
        let src = "X-CUSTOM-PROP;X-PARAM=x-value:value\r\n";
        let result = test_scan(src);

        assert!(!result.has_errors);
        assert_eq!(result.lines[0].name.to_owned(), "X-CUSTOM-PROP");
        assert_eq!(result.lines[0].parameters[0].name.to_owned(), "X-PARAM");
    }

    #[test]
    fn scanner_complex_calendar() {
        let src = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//CalDAV Client//EN\r\n\
BEGIN:VEVENT\r\n\
UID:123@example.com\r\n\
DTSTAMP:20250101T120000Z\r\n\
DTSTART;TZID=America/New_York:20250615T133000\r\n\
DTEND;TZID=America/New_York:20250615T143000\r\n\
SUMMARY:Team Meeting\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let result = test_scan(src);

        assert!(!result.has_errors);
        assert_eq!(result.lines.len(), 11);

        assert!(result.lines[0].name.eq_str_ignore_ascii_case("BEGIN"));
        assert_eq!(
            component_name(&result.lines[0]).as_deref(),
            Some("VCALENDAR")
        );

        assert!(result.lines[3].name.eq_str_ignore_ascii_case("BEGIN"));
        assert_eq!(component_name(&result.lines[3]).as_deref(), Some("VEVENT"));

        // Check property with parameter
        assert_eq!(result.lines[6].name.to_owned(), "DTSTART");
        assert_eq!(result.lines[6].parameters.len(), 1);
        assert_eq!(result.lines[6].parameters[0].name.to_owned(), "TZID");
    }

    #[test]
    fn scanner_multi_value_parameter() {
        let src = "X-CUSTOM;PARAM=value1,value2,value3:test\r\n";
        let result = test_scan(src);

        assert!(!result.has_errors);
        assert_eq!(result.lines[0].parameters[0].values.len(), 3);
        assert_eq!(
            result.lines[0].parameters[0].values[0]
                .value
                .resolve()
                .as_ref(),
            "value1"
        );
        assert_eq!(
            result.lines[0].parameters[0].values[1]
                .value
                .resolve()
                .as_ref(),
            "value2"
        );
        assert_eq!(
            result.lines[0].parameters[0].values[2]
                .value
                .resolve()
                .as_ref(),
            "value3"
        );
    }
}
