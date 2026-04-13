// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Syntax analysis for iCalendar files as defined in RFC 5545
//!
//! This module provides the syntax analysis phase of the iCalendar parser,
//! which includes:
//!
//! - **Lexer**: Tokenizes raw iCalendar text into structured tokens
//! - **Scanner**: Scans token streams into a vector of content lines
//! - **Tree Builder**: Builds a component tree from content lines using a stack-based algorithm
//! - **Syntax Parser**: Combines all phases to produce a hierarchical component structure
//!
//! # Architecture
//!
//! ```text
//! Source Text → Lexer → Token Stream → Scanner → Content Lines → Tree Builder → Component Tree
//! ```
//!
//! # Example
//!
//! ```rust
//! use aimcal_ical::syntax::{syntax_analysis, RawComponent};
//!
//! let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
//! let components = syntax_analysis(src).unwrap();
//! assert_eq!(components.len(), 1);
//! assert_eq!(components[0].name.resolve().as_ref(), "VCALENDAR");
//! ```

mod lexer;
mod scanner;
mod tree_builder;

pub use lexer::{SpannedToken, Token, tokenize};
pub use scanner::{ContentLine, ContentLineError, ScanResult, scan_content_lines};
pub use tree_builder::{
    RawComponent, RawParameter, RawParameterValue, RawProperty, TreeBuildError, TreeBuilderResult,
    build_tree,
};

use std::fmt;

/// Options for controlling syntax analysis behavior.
///
/// # Example
///
/// ```rust
/// use aimcal_ical::syntax::{ParseOptions, syntax_analysis};
///
/// let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
///
/// // Default options (lenient): bare LF is accepted
/// let components = syntax_analysis(src).unwrap();
///
/// // Strict options: bare LF (without preceding CR) is rejected
/// let opts = ParseOptions::new().strict_line_endings(true);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// When `true`, bare LF (without preceding CR) is reported as an error.
    ///
    /// RFC 5545 specifies CRLF (`\r\n`) as the line ending, but many real-world
    /// iCalendar files use bare LF (`\n`). Default is `false` for compatibility.
    pub strict_line_endings: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseOptions {
    /// Create new parse options with default (lenient) settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            strict_line_endings: false, // Default to lenient line ending handling
        }
    }

    /// Create new parse options with strict settings.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            strict_line_endings: true,
        }
    }

    /// Set whether to enforce strict CRLF line endings.
    ///
    /// When `true`, bare LF (without preceding CR) will be reported as a
    /// scanning error. Default is `false`.
    #[must_use]
    pub const fn strict_line_endings(mut self, strict: bool) -> Self {
        self.strict_line_endings = strict;
        self
    }
}

/// Parse raw iCalendar components from source text
///
/// This function performs tokenization, scanning, and tree building to produce
/// a hierarchical component tree.
///
/// # Arguments
///
/// * `src` - The iCalendar source text
///
/// # Returns
///
/// A result containing either:
/// - `Ok(Vec<SyntaxComponent>)` - Parsed components
/// - `Err(Vec<SyntaxError>)` - Syntax errors
///
/// # Example
///
/// ```rust
/// use aimcal_ical::syntax::syntax_analysis;
///
/// let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
/// let components = syntax_analysis(src).unwrap();
/// assert_eq!(components.len(), 1);
/// ```
///
/// ## Errors
/// If there are parsing errors, a vector of errors will be returned.
pub fn syntax_analysis(src: &str) -> Result<Vec<RawComponent<'_>>, Vec<SyntaxError<'_>>> {
    syntax_analysis_with_options(src, ParseOptions::default())
}

/// Parse raw iCalendar components from source text with custom options
///
/// This function performs tokenization, scanning, and tree building to produce
/// a hierarchical component tree, using the provided [`ParseOptions`].
///
/// # Arguments
///
/// * `src` - The iCalendar source text
/// * `options` - Parse options controlling analysis behavior
///
/// # Returns
///
/// A result containing either:
/// - `Ok(Vec<SyntaxComponent>)` - Parsed components
/// - `Err(Vec<SyntaxError>)` - Syntax errors
///
/// # Errors
///
/// Returns a vector of [`SyntaxError`] if tokenization, scanning, or tree building fails.
pub fn syntax_analysis_with_options<'src>(
    src: &'src str,
    options: ParseOptions,
) -> Result<Vec<RawComponent<'src>>, Vec<SyntaxError<'src>>> {
    // Tokenize
    let tokens = tokenize(src);

    // Scan tokens into content lines
    let scan_result = scan_content_lines(src, tokens, options);

    // Collect scanning errors
    let mut errors: Vec<SyntaxError<'src>> = Vec::new();
    for line in &scan_result.lines {
        if let Some(ref error) = line.error {
            errors.push(error.clone().into());
        }
    }

    // Phase 2: Build component tree from content lines
    let tree_result = build_tree(&scan_result.lines);

    // Collect tree builder errors
    for err in tree_result.errors {
        errors.push(err.into());
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(tree_result.roots)
}

/// Unified error type for syntax analysis
///
/// This enum represents all possible errors that can occur during syntax analysis,
/// encompassing both scanning and tree building phases.
#[derive(Debug, Clone)]
pub enum SyntaxError<'src> {
    /// Errors from scanning content lines
    Scanner(ContentLineError),

    /// Errors from building the component tree
    TreeBuilder(TreeBuildError<'src>),
}

impl fmt::Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::Scanner(err) => write!(f, "{err}"),
            SyntaxError::TreeBuilder(err) => write!(f, "{err}"),
        }
    }
}

impl From<ContentLineError> for SyntaxError<'_> {
    fn from(err: ContentLineError) -> Self {
        SyntaxError::Scanner(err)
    }
}

impl<'src> From<TreeBuildError<'src>> for SyntaxError<'src> {
    fn from(err: TreeBuildError<'src>) -> Self {
        SyntaxError::TreeBuilder(err)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::indexing_slicing)]

    use super::*;

    #[test]
    fn parses_component_with_lf_endings() {
        // Default (lenient): bare LF should be accepted
        let src = "\
BEGIN:VCALENDAR\n\
VERSION:2.0\n\
END:VCALENDAR\n\
";
        let result = syntax_analysis(src);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let components = result.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name.resolve().as_ref(), "VCALENDAR");
    }

    #[test]
    fn strict_mode_rejects_bare_lf() {
        let src = "\
BEGIN:VCALENDAR\n\
END:VCALENDAR\n\
";
        let opts = ParseOptions::new().strict_line_endings(true);
        let result = syntax_analysis_with_options(src, opts);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(!errs.is_empty());
        assert!(errs.iter().any(|e| e.to_string().contains("bare LF")));
    }

    #[test]
    fn strict_mode_accepts_crlf() {
        let src = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
END:VCALENDAR\r\n\
";
        let opts = ParseOptions::new().strict_line_endings(true);
        let result = syntax_analysis_with_options(src, opts);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let components = result.unwrap();
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn parses_component() {
        // Test with the new scanner + tree builder pipeline
        let src = "\
BEGIN:VCALENDAR\r\n\
END:VCALENDAR\r\n\
";

        let result = syntax_analysis(src);

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let components = result.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name.resolve().as_ref(), "VCALENDAR");
    }

    #[test]
    fn matches_begin_end_tags() {
        // Test matched BEGIN/END
        let src = "\
BEGIN:VCALENDAR\r\n\
END:VCALENDAR\r\n\
";

        let result = syntax_analysis(src);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let components = result.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name.resolve().as_ref(), "VCALENDAR");

        // Test mismatched BEGIN/END
        let src = "\
BEGIN:VCALENDAR\r\n\
END:VEVENT\r\n\
";

        let result = syntax_analysis(src);
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(errs[0].to_string().contains("mismatched nesting"));
    }

    #[test]
    fn parses_property() {
        let src = "SUMMARY:Hello World!\r\n";
        let tokens = tokenize(src);
        let scan_result = scan_content_lines(src, tokens, ParseOptions::default());

        assert_eq!(scan_result.lines.len(), 1);
        let line = &scan_result.lines[0];
        assert!(line.error.is_none());
        assert_eq!(line.name.resolve().as_ref(), "SUMMARY");
        assert_eq!(line.value.resolve().as_ref(), "Hello World!");

        // Test with parameters
        let src = "DTSTART;TZID=America/New_York:20251113\r\n T100000\r\n";
        let tokens = tokenize(src);
        let scan_result = scan_content_lines(src, tokens, ParseOptions::default());

        assert_eq!(scan_result.lines.len(), 1);
        let line = &scan_result.lines[0];
        assert!(line.error.is_none());
        assert_eq!(line.name.resolve().as_ref(), "DTSTART");
        assert_eq!(line.parameters.len(), 1);
        assert_eq!(line.parameters[0].name.resolve().as_ref(), "TZID");
        assert_eq!(
            line.parameters[0].values[0].value.resolve().as_ref(),
            "America/New_York"
        );
    }

    #[test]
    fn parses_parameter() {
        let src = "TZID=America/New_York";
        let tokens = tokenize(src);
        let scan_result = scan_content_lines(src, tokens, ParseOptions::default());

        assert_eq!(scan_result.lines.len(), 1);
        let line = &scan_result.lines[0];
        // This is not a complete property (no colon), so it will have an error
        // but we can still check the parameter parsing
        assert_eq!(line.name.resolve().as_ref(), "TZID");
    }
}
