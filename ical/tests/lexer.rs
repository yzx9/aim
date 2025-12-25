// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the iCalendar lexer
//!
//! These tests validate the lexer's behavior on realistic iCalendar content
//! and edge cases.

use aimcal_ical::lexer::Token;
use logos::Logos;

/// Test helper to tokenize a string and collect the tokens
fn tokenize(src: &str) -> Vec<(Token<'_>, std::ops::Range<usize>)> {
    Token::lexer(src)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => (tok, span),
            Err(()) => (Token::Error, span),
        })
        .collect()
}

/// Test helper to get just the tokens without spans
fn tokenize_tokens(src: &str) -> Vec<Token<'_>> {
    tokenize(src).into_iter().map(|(t, _)| t).collect()
}

#[test]
fn test_empty_input() {
    let tokens = tokenize_tokens("");
    assert!(tokens.is_empty());
}

#[test]
fn test_simple_ical_property() {
    let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("BEGIN"),
            Token::Colon,
            Token::Word("VCALENDAR"),
            Token::Newline,
            Token::Word("VERSION"),
            Token::Colon,
            Token::Word("2"),
            Token::Symbol("."),
            Token::Word("0"),
            Token::Newline,
            Token::Word("END"),
            Token::Colon,
            Token::Word("VCALENDAR"),
            Token::Newline,
        ]
    );
}

#[test]
fn test_summary_with_quotes() {
    let src = r#"SUMMARY:"Meeting with Team""#;
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("SUMMARY"),
            Token::Colon,
            Token::DQuote,
            Token::Word("Meeting"),
            Token::Symbol(" "),
            Token::Word("with"),
            Token::Symbol(" "),
            Token::Word("Team"),
            Token::DQuote,
        ]
    );
}

#[test]
fn test_property_with_parameters() {
    let src = "DTSTART;TZID=America/New_York:20250101T090000";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("DTSTART"),
            Token::Semicolon,
            Token::Word("TZID"),
            Token::Equal,
            Token::Word("America"),
            Token::Symbol("/"),
            Token::Word("New_York"),
            Token::Colon,
            Token::Word("20250101T090000"),
        ]
    );
}

#[test]
fn test_text_property_with_escaped_chars() {
    let src = r#"DESCRIPTION:This is a test\nWith newline\;And semicolon\,And comma"#;
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("DESCRIPTION"),
            Token::Colon,
            Token::Word("This"),
            Token::Symbol(" "),
            Token::Word("is"),
            Token::Symbol(" "),
            Token::Word("a"),
            Token::Symbol(" "),
            Token::Word("test"),
            Token::Escape(r"\n"),
            Token::Word("With"),
            Token::Symbol(" "),
            Token::Word("newline"),
            Token::Escape(r"\;"),
            Token::Word("And"),
            Token::Symbol(" "),
            Token::Word("semicolon"),
            Token::Escape(r"\,"),
            Token::Word("And"),
            Token::Symbol(" "),
            Token::Word("comma"),
        ]
    );
}

#[test]
fn test_line_folding() {
    let src = "DESCRIPTION:This is a very long description that\r\n spans multiple lines";
    let tokens = tokenize_tokens(src);
    // The folding (CRLF + space/tab) should be skipped
    assert_eq!(
        tokens,
        vec![
            Token::Word("DESCRIPTION"),
            Token::Colon,
            Token::Word("This"),
            Token::Symbol(" "),
            Token::Word("is"),
            Token::Symbol(" "),
            Token::Word("a"),
            Token::Symbol(" "),
            Token::Word("very"),
            Token::Symbol(" "),
            Token::Word("long"),
            Token::Symbol(" "),
            Token::Word("description"),
            Token::Symbol(" "),
            Token::Word("that"),
            Token::Word("spans"),
            Token::Symbol(" "),
            Token::Word("multiple"),
            Token::Symbol(" "),
            Token::Word("lines"),
        ]
    );
}

#[test]
fn test_rrule_property() {
    let src = "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("RRULE"),
            Token::Colon,
            Token::Word("FREQ"),
            Token::Equal,
            Token::Word("WEEKLY"),
            Token::Semicolon,
            Token::Word("BYDAY"),
            Token::Equal,
            Token::Word("MO"),
            Token::Comma,
            Token::Word("WE"),
            Token::Comma,
            Token::Word("FR"),
        ]
    );
}

#[test]
fn test_unicode_in_summary() {
    let src = "SUMMARY:Team会议📅 Discuss Q1 goals";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("SUMMARY"),
            Token::Colon,
            Token::Word("Team"),
            Token::UnicodeText("会议📅"),
            Token::Symbol(" "),
            Token::Word("Discuss"),
            Token::Symbol(" "),
            Token::Word("Q1"),
            Token::Symbol(" "),
            Token::Word("goals"),
        ]
    );
}

#[test]
fn test_organizer_property() {
    let src = r#"ORGANIZER;CN=John Doe:mailto:john.doe@example.com"#;
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("ORGANIZER"),
            Token::Semicolon,
            Token::Word("CN"),
            Token::Equal,
            Token::Word("John"),
            Token::Symbol(" "),
            Token::Word("Doe"),
            Token::Colon,
            Token::Word("mailto"),
            Token::Colon,
            Token::Word("john"),
            Token::Symbol("."),
            Token::Word("doe"),
            Token::Symbol("@"),
            Token::Word("example"),
            Token::Symbol("."),
            Token::Word("com"),
        ]
    );
}

#[test]
fn test_multiple_escaped_sequences() {
    let src = r#"DESCRIPTION:\\ \; \, \N \n"#;
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("DESCRIPTION"),
            Token::Colon,
            Token::Escape(r"\\"),
            Token::Symbol(" "),
            Token::Escape(r"\;"),
            Token::Symbol(" "),
            Token::Escape(r"\,"),
            Token::Symbol(" "),
            Token::Escape(r"\N"),
            Token::Symbol(" "),
            Token::Escape(r"\n"),
        ]
    );
}

#[test]
fn test_attendee_property() {
    let src = "ATTENDEE;RSVP=TRUE;CUTYPE=INDIVIDUAL:mailto:test@example.com";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("ATTENDEE"),
            Token::Semicolon,
            Token::Word("RSVP"),
            Token::Equal,
            Token::Word("TRUE"),
            Token::Semicolon,
            Token::Word("CUTYPE"),
            Token::Equal,
            Token::Word("INDIVIDUAL"),
            Token::Colon,
            Token::Word("mailto"),
            Token::Colon,
            Token::Word("test"),
            Token::Symbol("@"),
            Token::Word("example"),
            Token::Symbol("."),
            Token::Word("com"),
        ]
    );
}

#[test]
fn test_date_time_value() {
    let src = "DTSTART:20250615T133000";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("DTSTART"),
            Token::Colon,
            Token::Word("20250615T133000"),
        ]
    );
}

#[test]
fn test_exdate_property() {
    let src = "EXDATE:20250101T090000,20250108T090000,20250115T090000";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("EXDATE"),
            Token::Colon,
            Token::Word("20250101T090000"),
            Token::Comma,
            Token::Word("20250108T090000"),
            Token::Comma,
            Token::Word("20250115T090000"),
        ]
    );
}

#[test]
fn test_alarm_component() {
    let src = "BEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nEND:VALARM\r\n";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("BEGIN"),
            Token::Colon,
            Token::Word("VALARM"),
            Token::Newline,
            Token::Word("TRIGGER"),
            Token::Colon,
            Token::Word("-PT15M"),
            Token::Newline,
            Token::Word("ACTION"),
            Token::Colon,
            Token::Word("DISPLAY"),
            Token::Newline,
            Token::Word("END"),
            Token::Colon,
            Token::Word("VALARM"),
            Token::Newline,
        ]
    );
}

#[test]
fn test_geographic_position() {
    let src = "GEO:37.386013;-122.083932";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("GEO"),
            Token::Colon,
            Token::Word("37"),
            Token::Symbol("."),
            Token::Word("386013"),
            Token::Semicolon,
            Token::Word("-122"),
            Token::Symbol("."),
            Token::Word("083932"),
        ]
    );
}

#[test]
fn test_percent_complete() {
    let src = "PERCENT-COMPLETE:75";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("PERCENT-COMPLETE"),
            Token::Colon,
            Token::Word("75"),
        ]
    );
}

#[test]
fn test_priority() {
    let src = "PRIORITY:5";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![Token::Word("PRIORITY"), Token::Colon, Token::Word("5"),]
    );
}

#[test]
fn test_url_property() {
    let src = "URL:http://example.com/event.html";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("URL"),
            Token::Colon,
            Token::Word("http"),
            Token::Colon,
            Token::Symbol("/"),
            Token::Symbol("/"),
            Token::Word("example"),
            Token::Symbol("."),
            Token::Word("com"),
            Token::Symbol("/"),
            Token::Word("event"),
            Token::Symbol("."),
            Token::Word("html"),
        ]
    );
}

#[test]
fn test_uid_property() {
    let src = "UID:1234567890@example.com";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("UID"),
            Token::Colon,
            Token::Word("1234567890"),
            Token::Symbol("@"),
            Token::Word("example"),
            Token::Symbol("."),
            Token::Word("com"),
        ]
    );
}

#[test]
fn test_classification() {
    let src = "CLASS:PUBLIC";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![Token::Word("CLASS"), Token::Colon, Token::Word("PUBLIC"),]
    );
}

#[test]
fn test_created_last_modified() {
    let src = "CREATED:20250101T000000Z\r\nLAST-MODIFIED:20250102T120000Z";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("CREATED"),
            Token::Colon,
            Token::Word("20250101T000000Z"),
            Token::Newline,
            Token::Word("LAST-MODIFIED"),
            Token::Colon,
            Token::Word("20250102T120000Z"),
        ]
    );
}

#[test]
fn test_sequence() {
    let src = "SEQUENCE:2";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![Token::Word("SEQUENCE"), Token::Colon, Token::Word("2"),]
    );
}

#[test]
fn test_status_values() {
    let src = "STATUS:TENTATIVE\r\nSTATUS:CONFIRMED\r\nSTATUS:CANCELLED";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("STATUS"),
            Token::Colon,
            Token::Word("TENTATIVE"),
            Token::Newline,
            Token::Word("STATUS"),
            Token::Colon,
            Token::Word("CONFIRMED"),
            Token::Newline,
            Token::Word("STATUS"),
            Token::Colon,
            Token::Word("CANCELLED"),
        ]
    );
}

#[test]
fn test_transp() {
    let src = "TRANSP:OPAQUE";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![Token::Word("TRANSP"), Token::Colon, Token::Word("OPAQUE"),]
    );
}

#[test]
fn test_location_property() {
    let src = "LOCATION:Conference Room B\\, Building 1";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("LOCATION"),
            Token::Colon,
            Token::Word("Conference"),
            Token::Symbol(" "),
            Token::Word("Room"),
            Token::Symbol(" "),
            Token::Word("B"),
            Token::Escape(r"\,"),
            Token::Symbol(" "),
            Token::Word("Building"),
            Token::Symbol(" "),
            Token::Word("1"),
        ]
    );
}

#[test]
fn test_complete_icalendar_minimal() {
    let src = "\
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:12345@example.com
DTSTAMP:20250101T120000Z
DTSTART:20250615T133000Z
DTEND:20250615T143000Z
SUMMARY:Team Meeting
END:VEVENT
END:VCALENDAR
";
    let tokens = tokenize_tokens(src);

    // Verify key tokens are present
    assert!(tokens.contains(&Token::Word("BEGIN")));
    assert!(tokens.contains(&Token::Word("VCALENDAR")));
    assert!(tokens.contains(&Token::Word("VERSION")));
    assert!(tokens.contains(&Token::Word("VEVENT")));
    assert!(tokens.contains(&Token::Word("SUMMARY")));
    assert!(tokens.contains(&Token::Word("Team")));
    assert!(tokens.contains(&Token::Word("Meeting")));
}

#[test]
fn test_token_positions() {
    let src = "BEGIN:VCALENDAR";
    let tokens = tokenize(src);

    // Check that we get proper span information
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].0, Token::Word("BEGIN"));
    assert_eq!(tokens[0].1, 0..5);
    assert_eq!(tokens[1].0, Token::Colon);
    assert_eq!(tokens[1].1, 5..6);
    assert_eq!(tokens[2].0, Token::Word("VCALENDAR"));
    assert_eq!(tokens[2].1, 6..15);
}

#[test]
fn test_multiline_unicode_description() {
    let src = "\
DESCRIPTION:Important meeting with team members from 中国🇨🇳 and Japan 🇯🇵\
\r\n to discuss Q1 2025 strategy and planning.\r\n Please prepare your reports.\
";
    let tokens = tokenize_tokens(src);

    // Verify unicode tokens are properly recognized
    assert!(tokens.iter().any(|t| matches!(t, Token::UnicodeText(_))));
    assert!(tokens.contains(&Token::Word("DESCRIPTION")));
    assert!(tokens.contains(&Token::Word("Important")));
}

#[test]
fn test_complex_categories() {
    let src = "CATEGORIES:MEETING,TEAM,STRATEGY,IMPORTANT";
    let tokens = tokenize_tokens(src);
    assert_eq!(
        tokens,
        vec![
            Token::Word("CATEGORIES"),
            Token::Colon,
            Token::Word("MEETING"),
            Token::Comma,
            Token::Word("TEAM"),
            Token::Comma,
            Token::Word("STRATEGY"),
            Token::Comma,
            Token::Word("IMPORTANT"),
        ]
    );
}
