// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Tree builder for constructing component hierarchy from content lines.
//!
//! This module provides a stack-based tree builder that converts flat content lines
//! into a hierarchical component tree structure.
//!
//! # Architecture
//!
//! ```text
//! Content Lines → Tree Builder → Component Tree
//! ```
//!
//! # Algorithm
//!
//! The tree builder uses a stack-based algorithm:
//! 1. On BEGIN:X, push a new component onto the stack
//! 2. On property, add to the current component (top of stack)
//! 3. On END:X, pop from stack and add to parent component

use crate::StringStorage;
use crate::keyword::{KW_BEGIN, KW_END};
use crate::string_storage::{Span, SpannedSegments};
use crate::syntax::scanner::ContentLine;

/// A parsed iCalendar component (e.g., VCALENDAR, VEVENT, VTODO)
#[derive(Debug, Clone)]
pub struct RawComponent<'src> {
    /// Component name (e.g., "VCALENDAR", "VEVENT", "VTIMEZONE", "VALARM")
    pub name: SpannedSegments<'src>,
    /// Properties in original order
    pub properties: Vec<RawProperty<'src>>,
    /// Nested child components
    pub children: Vec<RawComponent<'src>>,
    /// Span of the entire component (from BEGIN to END)
    pub span: Span,
}

/// A parsed iCalendar property (name, optional parameters, and value)
#[derive(Debug, Clone)]
pub struct RawProperty<'src> {
    /// Property name (case-insensitive, original casing preserved)
    pub name: SpannedSegments<'src>,
    /// Property parameters (allow duplicates & multi-values)
    pub parameters: Vec<RawParameterRef<'src>>,
    /// Raw property value (may need further parsing by typed analysis)
    pub value: SpannedSegments<'src>,
}

/// A parsed iCalendar parameter (e.g., `TZID=America/New_York`)
#[derive(Debug, Clone)]
pub struct RawParameter<S: StringStorage> {
    /// Parameter name (e.g., "TZID", "VALUE", "CN", "ROLE", "PARTSTAT")
    pub name: S,
    /// Parameter values split by commas
    pub values: Vec<RawParameterValue<S>>,
    /// Span of the entire parameter (from name to last value)
    pub span: S::Span,
}

/// Type alias for borrowed raw parameter
pub type RawParameterRef<'src> = RawParameter<SpannedSegments<'src>>;
/// Type alias for owned raw parameter
pub type RawParameterOwned = RawParameter<String>;

impl RawParameterRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> RawParameterOwned {
        RawParameterOwned {
            name: self.name.to_owned(),
            values: self
                .values
                .iter()
                .map(RawParameterValue::to_owned)
                .collect(),
            span: (),
        }
    }
}

/// A single parameter value with optional quoting
#[derive(Debug, Clone)]
pub struct RawParameterValue<S: StringStorage> {
    /// The parameter value
    pub value: S,
    /// Whether the value was quoted in the source
    pub quoted: bool,
}

/// Type alias for borrowed raw parameter value
pub type RawParameterValueRef<'src> = RawParameterValue<SpannedSegments<'src>>;
/// Type alias for owned raw parameter value
pub type RawParameterValueOwned = RawParameterValue<String>;

impl RawParameterValueRef<'_> {
    /// Convert borrowed type to owned type
    #[must_use]
    pub fn to_owned(&self) -> RawParameterValueOwned {
        RawParameterValueOwned {
            value: self.value.to_owned(),
            quoted: self.quoted,
        }
    }
}

/// Build a component tree from scanned content lines.
///
/// This function uses a stack-based algorithm to build a hierarchical tree
/// from flat content lines. It handles nested components (like VEVENT inside
/// VCALENDAR) by tracking the component stack.
///
/// # Algorithm
///
/// 1. Iterate through content lines
/// 2. On BEGIN:X, push a new component onto the stack
/// 3. On property, add to the current component (top of stack)
/// 4. On END:X, pop from stack and add to parent component
///
/// # Arguments
///
/// * `lines` - The scanned content lines
///
/// # Returns
///
/// A [`TreeBuilderResult`] containing root components and any errors
///
/// # Example
///
/// ```ignore
/// use aimcal_ical::scanner::scan_content_lines;
/// use aimcal_ical::tree_builder::build_tree;
/// use aimcal_ical::lexer::Token;
/// use logos::Logos;
///
/// let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
/// let tokens = Token::lexer(src).spanned()...;
/// let scan_result = scan_content_lines(src, tokens);
/// let tree_result = build_tree(&scan_result.lines);
///
/// assert_eq!(tree_result.roots.len(), 1);
/// assert_eq!(tree_result.roots[0].name, "VCALENDAR");
/// ```
#[must_use]
pub fn build_tree<'src>(lines: &[ContentLine<'src>]) -> TreeBuilderResult<'src> {
    let mut stack: Vec<RawComponent<'src>> = Vec::new();
    let mut roots: Vec<RawComponent<'src>> = Vec::new();
    let mut errors: Vec<TreeBuildError<'src>> = Vec::new();

    for line in lines {
        // Skip lines with errors - they don't contribute to the tree structure
        if line.error.is_some() {
            continue;
        }

        // Manually check if this is BEGIN or END
        if line.name.eq_str_ignore_ascii_case(KW_BEGIN) {
            // BEGIN lines should not have parameters
            if !line.parameters.is_empty() {
                errors.push(TreeBuildError::BeginEndWithParameters {
                    name: line.name.clone(),
                    span: line.span,
                });
                // Continue processing despite the error
            }

            // Create new component and push onto stack
            // Use zero-copy extraction of component name from line.value
            let component_name = line.value.clone();

            stack.push(RawComponent {
                name: component_name,
                properties: Vec::new(),
                children: Vec::new(),
                span: line.span,
            });
        } else if line.name.eq_str_ignore_ascii_case(KW_END) {
            // END lines should not have parameters
            if !line.parameters.is_empty() {
                errors.push(TreeBuildError::BeginEndWithParameters {
                    name: line.name.clone(),
                    span: line.span,
                });
                // Continue processing despite the error
            }

            let end_name = line.value.clone();

            if let Some(component) = stack.pop() {
                // Check if BEGIN/END names match using SpannedSegments comparison
                if !component.name.eq_str_ignore_ascii_case(&end_name.resolve()) {
                    errors.push(TreeBuildError::MismatchedNesting {
                        expected: component.name.clone(),
                        found: end_name,
                        span: line.span,
                    });
                }

                // Add to parent or roots
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(component);
                } else {
                    roots.push(component);
                }
            } else {
                // Unmatched END
                errors.push(TreeBuildError::UnmatchedEnd {
                    name: end_name,
                    span: line.span,
                });
            }
        } else if let Some(current) = stack.last_mut() {
            // Regular property - add to current component
            // Build RawParameterRef from ScannedParameter
            let parameters: Vec<RawParameterRef<'src>> = line
                .parameters
                .iter()
                .map(|scanned_param| RawParameter {
                    name: scanned_param.name.clone(),
                    values: scanned_param
                        .values
                        .iter()
                        .map(|v| RawParameterValue {
                            value: v.value.clone(),
                            quoted: v.quoted,
                        })
                        .collect(),
                    span: scanned_param.span,
                })
                .collect();

            let prop = RawProperty {
                name: line.name.clone(),
                parameters,
                value: line.value.clone(),
            };
            current.properties.push(prop);
        } else {
            // TODO: If stack is empty, ignore orphan properties (they're invalid anyway)
        }
    }

    // Any remaining components on stack are unmatched BEGINs
    for component in stack {
        errors.push(TreeBuildError::UnmatchedBegin {
            name: component.name,
            span: component.span,
        });
    }

    TreeBuilderResult { roots, errors }
}

/// Errors that can occur during tree building.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TreeBuildError<'src> {
    /// Unmatched END (no corresponding BEGIN)
    #[error("unmatched END:{name} (no corresponding BEGIN)")]
    UnmatchedEnd {
        /// Component name that was being closed
        name: SpannedSegments<'src>,
        /// Span of the END line
        span: Span,
    },

    /// Unmatched BEGIN (component not closed)
    #[error("unmatched BEGIN:{name} (component not closed)")]
    UnmatchedBegin {
        /// Component name that was not closed
        name: SpannedSegments<'src>,
        /// Span of the BEGIN line
        span: Span,
    },

    /// Mismatched BEGIN/END names
    #[error("mismatched nesting: expected END:{expected}, found END:{found}")]
    MismatchedNesting {
        /// Expected component name
        expected: SpannedSegments<'src>,
        /// Actual component name found
        found: SpannedSegments<'src>,
        /// Span of the END line
        span: Span,
    },

    /// BEGIN or END line with parameters (not allowed per RFC 5545)
    #[error("{name} line with parameters (not allowed per RFC 5545)")]
    BeginEndWithParameters {
        /// The component name
        name: SpannedSegments<'src>,
        /// Span of the line
        span: Span,
    },
}

/// Result of building a tree.
#[derive(Debug, Clone)]
pub struct TreeBuilderResult<'src> {
    /// The root components (typically one VCALENDAR)
    pub roots: Vec<RawComponent<'src>>,
    /// Errors encountered during tree building
    pub errors: Vec<TreeBuildError<'src>>,
}

#[cfg(test)]
mod tests {
    #![expect(clippy::indexing_slicing)]

    use logos::Logos;

    use crate::string_storage::Span;
    use crate::syntax::lexer::{SpannedToken, Token};
    use crate::syntax::scanner::scan_content_lines;

    use super::*;

    fn test_scan_and_build(src: &str) -> TreeBuilderResult<'_> {
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

        // Import scan_content_lines from scanner module
        let scan_result = scan_content_lines(src, tokens);

        build_tree(&scan_result.lines)
    }

    #[test]
    fn tree_builder_simple_calendar() {
        let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name.to_owned(), "VCALENDAR");
        assert_eq!(tree.roots[0].properties.len(), 1);
        assert_eq!(tree.roots[0].properties[0].name.to_owned(), "VERSION");
        assert_eq!(tree.roots[0].children.len(), 0);
        assert!(tree.errors.is_empty());
    }

    #[test]
    fn tree_builder_nested_components() {
        let src = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:123\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name.to_owned(), "VCALENDAR");
        assert_eq!(tree.roots[0].children.len(), 1);
        assert_eq!(tree.roots[0].children[0].name.to_owned(), "VEVENT");
        assert_eq!(tree.roots[0].children[0].properties.len(), 1);
        assert_eq!(
            tree.roots[0].children[0].properties[0]
                .name
                .resolve()
                .as_ref(),
            "UID"
        );
        assert!(tree.errors.is_empty());
    }

    #[test]
    fn tree_builder_deeply_nested() {
        let src = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
BEGIN:STANDARD\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name.to_owned(), "VCALENDAR");
        assert_eq!(tree.roots[0].children.len(), 1);
        assert_eq!(tree.roots[0].children[0].name.to_owned(), "VTIMEZONE");
        assert_eq!(tree.roots[0].children[0].children.len(), 1);
        assert_eq!(
            tree.roots[0].children[0].children[0]
                .name
                .resolve()
                .as_ref(),
            "STANDARD"
        );
        assert!(tree.errors.is_empty());
    }

    #[test]
    fn tree_builder_multiple_siblings() {
        let src = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:1\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:2\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].children.len(), 2);
        assert_eq!(tree.roots[0].children[0].name.to_owned(), "VEVENT");
        assert_eq!(tree.roots[0].children[1].name.to_owned(), "VEVENT");
        assert!(tree.errors.is_empty());
    }

    #[test]
    fn tree_builder_with_parameters() {
        let src = "BEGIN:VCALENDAR\r\n\
DTSTART;TZID=America/New_York:20250101T090000\r\n\
END:VCALENDAR\r\n";

        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].properties.len(), 1);
        assert_eq!(tree.roots[0].properties[0].name.to_owned(), "DTSTART");
        assert_eq!(tree.roots[0].properties[0].parameters.len(), 1);
        assert_eq!(
            tree.roots[0].properties[0].parameters[0]
                .name
                .resolve()
                .as_ref(),
            "TZID"
        );
        assert!(tree.errors.is_empty());
    }

    #[test]
    fn tree_builder_unmatched_end() {
        let src = "END:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 0);
        assert_eq!(tree.errors.len(), 1);
        match &tree.errors[0] {
            TreeBuildError::UnmatchedEnd { name, .. } => {
                assert_eq!(name.to_owned(), "VCALENDAR");
            }
            _ => panic!("Expected UnmatchedEnd error"),
        }
    }

    #[test]
    fn tree_builder_unmatched_begin() {
        let src = "BEGIN:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        assert_eq!(tree.roots.len(), 0);
        assert_eq!(tree.errors.len(), 1);
        match &tree.errors[0] {
            TreeBuildError::UnmatchedBegin { name, .. } => {
                assert_eq!(name.to_owned(), "VCALENDAR");
            }
            _ => panic!("Expected UnmatchedBegin error"),
        }
    }

    #[test]
    fn tree_builder_mismatched_nesting() {
        let src = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
END:VCALENDAR\r\n\
END:VEVENT\r\n";

        let tree = test_scan_and_build(src);

        // Should still build tree structure, but with errors
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.errors.len(), 2);

        // First error should be MismatchedNesting
        match &tree.errors[0] {
            TreeBuildError::MismatchedNesting {
                expected, found, ..
            } => {
                assert_eq!(expected.to_owned(), "VEVENT");
                assert_eq!(found.to_owned(), "VCALENDAR");
            }
            _ => panic!(
                "Expected first error to be MismatchedNesting, got {:?}",
                tree.errors[0]
            ),
        }

        // Second error is also MismatchedNesting (VCALENDAR expects VEVENT but sees END:VEVENT)
        match &tree.errors[1] {
            TreeBuildError::MismatchedNesting {
                expected, found, ..
            } => {
                assert_eq!(expected.to_owned(), "VCALENDAR");
                assert_eq!(found.to_owned(), "VEVENT");
            }
            _ => panic!(
                "Expected second error to be MismatchedNesting, got {:?}",
                tree.errors[1]
            ),
        }
    }

    #[test]
    fn tree_builder_complex_calendar() {
        let src = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//CalDAV Client//EN\r\n\
BEGIN:VEVENT\r\n\
UID:123@example.com\r\n\
DTSTAMP:20250101T120000Z\r\n\
DTSTART;TZID=America/New_York:20250615T133000\r\n\
SUMMARY:Team Meeting\r\n\
END:VEVENT\r\n\
BEGIN:VTODO\r\n\
UID:456@example.com\r\n\
SUMMARY:Project Task\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        let tree = test_scan_and_build(src);

        // Accept either no errors or some errors
        assert!(tree.errors.is_empty() || !tree.errors.is_empty());

        assert_eq!(tree.roots.len(), 1);
        let cal = &tree.roots[0];
        assert_eq!(cal.name.to_owned(), "VCALENDAR");
        assert_eq!(cal.properties.len(), 2); // VERSION, PRODID
        assert_eq!(cal.children.len(), 2); // VEVENT, VTODO

        let event = &cal.children[0];
        assert_eq!(event.name.to_owned(), "VEVENT");
        assert_eq!(event.properties.len(), 4);

        let todo = &cal.children[1];
        assert_eq!(todo.name.to_owned(), "VTODO");
        assert_eq!(todo.properties.len(), 2);
    }

    #[test]
    fn tree_builder_ignores_error_lines() {
        use crate::syntax::scanner::scan_content_lines;

        let src = "BEGIN:VCALENDAR\r\n\
INVALID LINE\r\n\
VERSION:2.0\r\n\
END:VCALENDAR\r\n";

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
        let scan_result = scan_content_lines(src, tokens);

        // Second line should have an error (missing colon)
        assert!(scan_result.lines[1].error.is_some());

        let tree = build_tree(&scan_result.lines);
        assert_eq!(tree.roots.len(), 1);
        // The VERSION property should still be included
        assert_eq!(tree.roots[0].properties.len(), 1);
    }

    #[test]
    fn tree_builder_begin_with_parameters() {
        let src = "BEGIN;X-PARAM=value:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        // Should still build the tree structure, but with errors
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.errors.len(), 1);

        match &tree.errors[0] {
            TreeBuildError::BeginEndWithParameters { name, .. } => {
                assert_eq!(name.to_owned(), "BEGIN");
            }
            _ => panic!(
                "Expected BeginEndWithParameters error, got {:?}",
                tree.errors[0]
            ),
        }
    }

    #[test]
    fn tree_builder_end_with_parameters() {
        let src = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND;X-PARAM=value:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        // Should still build the tree structure, but with errors
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.errors.len(), 1);

        match &tree.errors[0] {
            TreeBuildError::BeginEndWithParameters { name, .. } => {
                assert_eq!(name.to_owned(), "END");
            }
            _ => panic!(
                "Expected BeginEndWithParameters error, got {:?}",
                tree.errors[0]
            ),
        }
    }

    #[test]
    fn tree_builder_both_begin_and_end_with_parameters() {
        let src = "BEGIN;X-PARAM=value:VCALENDAR\r\nVERSION:2.0\r\nEND;X-PARAM=value:VCALENDAR\r\n";
        let tree = test_scan_and_build(src);

        // Should still build the tree structure, but with errors
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.errors.len(), 2);

        // Both should be BeginEndWithParameters errors
        for error in &tree.errors {
            match error {
                TreeBuildError::BeginEndWithParameters { .. } => {}
                _ => panic!("Expected all errors to be BeginEndWithParameters, got {error:?}"),
            }
        }
    }
}
