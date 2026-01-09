// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! String storage abstraction for zero-copy and owned string representations.

use std::borrow::Cow;
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;

use chumsky::span::SimpleSpan;

/// Trait for string storage types.
///
/// This trait abstracts over different string storage strategies, enabling
/// both zero-copy parsing (with borrowed data) and owned data representations.
///
/// # Implementors
///
/// - `String` - Owned string data
/// - `SpannedSegments<'src>` - Zero-copy borrowed segments
pub trait StringStorage: Clone + Display {}

impl StringStorage for String {}

/// A span representing a range in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start position of the span
    pub start: usize,
    /// End position of the span
    pub end: usize,
}

impl Span {
    /// Create a new span from start and end positions
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Convert to a standard range
    #[must_use]
    pub const fn into_range(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<SimpleSpan<usize>> for Span {
    fn from(span: SimpleSpan<usize>) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for SimpleSpan<usize> {
    fn from(span: Span) -> Self {
        use chumsky::span::Span as _;
        SimpleSpan::new((), span.start..span.end)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A spanned text segment (text with its position in the source)
pub type SpannedSegment<'src> = (&'src str, Span);

/// A collection of spanned text segments (multi-segment value with positions)
#[derive(Default, Clone, Debug)]
pub struct SpannedSegments<'src> {
    pub(crate) segments: Vec<SpannedSegment<'src>>,
    len: usize,
}

impl<'src> SpannedSegments<'src> {
    /// Create a new `SpannedSegments` from a vector of segments
    #[must_use]
    pub(crate) fn new(segments: Vec<SpannedSegment<'src>>) -> Self {
        let len = segments.iter().map(|(s, _)| s.len()).sum();
        Self { segments, len }
    }

    /// Get the total length in bytes of all segments
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the segments contain no elements
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the full span from first to last segment
    #[must_use]
    pub fn span(&self) -> Span {
        match (self.segments.first(), self.segments.last()) {
            (Some((_, first_span)), Some((_, last_span))) => Span {
                start: first_span.start,
                end: last_span.end,
            },
            _ => Span { start: 0, end: 0 },
        }
    }

    /// Resolve segments into a single string (borrowed if single segment, owned otherwise)
    ///
    /// # Panics
    ///
    /// Panics if there are no segments. This should never happen in practice
    /// as `SpannedSegments` is always created with at least one segment.
    #[must_use]
    pub fn resolve(&self) -> Cow<'src, str> {
        if self.segments.len() == 1 {
            let s = self.segments.first().unwrap().0; // SAFETY: due to len() == 1
            Cow::Borrowed(s)
        } else {
            let mut s = String::with_capacity(self.len);
            for (seg, _) in &self.segments {
                s.push_str(seg);
            }
            Cow::Owned(s)
        }
    }

    /// Convert to owned String efficiently
    ///
    /// This is more explicit and slightly more efficient than using the
    /// `Display` trait's `to_string()` method, as it uses the known capacity.
    #[must_use]
    pub fn to_owned(&self) -> String {
        let mut s = String::with_capacity(self.len);
        for (seg, _) in &self.segments {
            s.push_str(seg);
        }
        s
    }

    /// Check if segments start with the given prefix, ignoring ASCII case
    #[must_use]
    pub(crate) fn starts_with_str_ignore_ascii_case(&self, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        } else if prefix.len() > self.len {
            return false;
        }

        let mut remaining = prefix;
        for (seg, _) in &self.segments {
            if remaining.is_empty() {
                return true;
            } else if seg.len() >= remaining.len() {
                // This segment is long enough to contain the rest of the prefix
                return seg[..remaining.len()].eq_ignore_ascii_case(remaining);
            } else if !seg.eq_ignore_ascii_case(&remaining[..seg.len()]) {
                return false;
            }
            // This segment is shorter than the remaining prefix
            remaining = &remaining[seg.len()..];
        }

        remaining.is_empty()
    }

    /// Compare segments to a string ignoring ASCII case
    #[must_use]
    pub fn eq_str_ignore_ascii_case(&self, mut other: &str) -> bool {
        if other.len() != self.len {
            return false;
        }

        for (seg, _) in &self.segments {
            let Some((head, tail)) = other.split_at_checked(seg.len()) else {
                return false;
            };
            if !head.eq_ignore_ascii_case(seg) {
                return false;
            }
            other = tail;
        }

        true
    }

    pub(crate) fn into_spanned_chars(self) -> SegmentedSpannedChars<'src> {
        SegmentedSpannedChars {
            segments: self.segments,
            seg_idx: 0,
            chars: None,
        }
    }
}

impl Display for SpannedSegments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (seg, _) in &self.segments {
            seg.fmt(f)?;
        }
        Ok(())
    }
}

impl StringStorage for SpannedSegments<'_> {}

/// Iterator over characters in spanned segments
#[derive(Debug, Clone)]
pub struct SegmentedSpannedChars<'src> {
    segments: Vec<SpannedSegment<'src>>,
    seg_idx: usize,
    chars: Option<(Span, Peekable<CharIndices<'src>>)>,
}

impl Iterator for SegmentedSpannedChars<'_> {
    type Item = (char, Span);

    fn next(&mut self) -> Option<Self::Item> {
        while self.seg_idx < self.segments.len() {
            match self.chars {
                Some((ref span, ref mut chars)) => match chars.next() {
                    Some((start, c)) => {
                        let char_span = match chars.peek() {
                            Some((end, _)) => Span::new(span.start + start, span.start + end),
                            None => Span::new(span.start + start, span.end),
                        };
                        return Some((c, char_span));
                    }
                    None => {
                        self.seg_idx += 1;
                        self.chars = None;
                    }
                },
                None => {
                    let (s, span) = self.segments.get(self.seg_idx).unwrap(); // SAFETY: due to while condition
                    self.chars = Some((*span, s.char_indices().peekable()));
                }
            }
        }

        None
    }
}
