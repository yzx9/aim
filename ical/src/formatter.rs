// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! iCalendar (RFC 5545) formatter module.
//!
//! This module provides functionality to format iCalendar data structures
//! to the RFC 5545 text format, writing to any `std::io::Write` implementer.
//!
//! # Example
//!
//! ```ignore
//! use aimcal_ical::{parse, formatter::format_to_string};
//!
//! let input = std::fs::read_to_string("event.ics")?;
//! let calendars = parse(&input)?;
//! let calendar = &calendars[0];
//!
//! // Format to string
//! let ics_string = format(calendar)?;
//! println!("{}", ics_string);
//! ```

mod component;
mod parameter;
mod property;
mod value;

use std::io::{self, Write};

use crate::formatter::component::write_icalendar;
use crate::semantic::ICalendar;
use crate::string_storage::StringStorage;

/// Convenience function to format an `ICalendar` to a `String` (uses default options).
///
/// # Example
///
/// ```ignore
/// use aimcal_ical::{parse, formatter::format_to_string};
///
/// let input = std::fs::read_to_string("event.ics")?;
/// let calendars = parse(&input)?;
/// let calendar = &calendars[0];
///
/// let ics_string = format(calendar)?;
/// println!("{ics_string}");
/// ```
///
/// # Errors
///
/// Returns an error if writing to the internal buffer fails or if the output
/// contains invalid UTF-8 data.
pub fn format<S: StringStorage>(calendar: &ICalendar<S>) -> io::Result<String> {
    let options: &FormatOptions = &FormatOptions::default();
    let mut buffer = Vec::new();
    let mut formatter = Formatter::new(&mut buffer, *options);
    write_icalendar(&mut formatter, calendar)?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Formatting options for the iCalendar formatter.
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Maximum line length in octets before folding.
    /// - `None`: no line folding
    /// - `Some(n)`: fold lines longer than n octets
    ///
    /// Default: `Some(75)` for RFC 5545 compliance.
    pub folding: Option<usize>,

    /// Line folding style.
    ///
    /// Default: `FoldingStyle::Space` (CRLF + SPACE).
    pub folding_style: FoldingStyle,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            folding: Some(75),
            folding_style: FoldingStyle::default(),
        }
    }
}

impl FormatOptions {
    /// Set the line folding option.
    #[must_use]
    pub fn folding(mut self, folding: Option<usize>) -> Self {
        self.folding = folding;
        self
    }

    /// Set the line folding style.
    #[must_use]
    pub const fn folding_style(mut self, style: FoldingStyle) -> Self {
        self.folding_style = style;
        self
    }

    /// Convenience method to write an `ICalendar` to any `Write` implementer.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write(
        &self,
        calendar: &ICalendar<impl StringStorage>,
        w: &mut impl Write,
    ) -> io::Result<()> {
        let mut formatter = Formatter::new(w, *self);
        write_icalendar(&mut formatter, calendar)?;
        Ok(())
    }

    /// Convenience method to write an `ICalendar` to a `String`.
    ///
    /// # Errors
    /// Returns an error if writing fails or if the output contains invalid UTF-8 data.
    pub fn write_to_string(&self, calendar: &ICalendar<impl StringStorage>) -> io::Result<String> {
        let mut buffer = Vec::new();
        self.write(calendar, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Line folding style for RFC 5545 formatting.
///
/// RFC 5545 specifies that folded lines should start with CRLF followed by
/// a whitespace character (SPACE or TAB).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FoldingStyle {
    /// CRLF + SPACE (RFC 5545 default)
    #[default]
    Space,
    /// CRLF + TAB
    Tab,
}

impl FoldingStyle {
    /// Get the folding sequence for this style.
    #[must_use]
    pub(crate) const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Space => b"\r\n ",
            Self::Tab => b"\r\n\t",
        }
    }

    /// Get the length of the continuation character after CRLF.
    #[must_use]
    pub(crate) const fn continuation_len() -> usize {
        1 // Both SPACE and TAB are 1 byte
    }
}

/// iCalendar formatter that writes to any `Write` implementer.
///
/// # Example
///
/// ```ignore
/// use aimcal_ical::formatter::Formatter;
///
/// let mut buffer = Vec::new();
/// let mut formatter = Formatter::new(&mut buffer);
/// formatter.write_begin_block("VCALENDAR")?;
/// // ... write properties and components
/// formatter.write_end_block("VCALENDAR")?;
/// ```
#[derive(Debug)]
pub struct Formatter<W: Write> {
    /// The underlying writer.
    writer: W,
    /// Formatting options.
    options: FormatOptions,
    /// Current line length in bytes (excluding the pending CRLF).
    line_length: usize,
}

impl<W: Write> Formatter<W> {
    /// Create a new formatter with options.
    #[must_use]
    pub fn new(writer: W, options: FormatOptions) -> Self {
        Self {
            writer,
            options,
            line_length: 0,
        }
    }

    /// Get a mutable reference to the underlying writer.
    #[must_use]
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Get a reference to the underlying writer.
    #[must_use]
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Consumes this formatter, returning the underlying writer.
    #[must_use]
    pub fn into_writer(self) -> W {
        self.writer
    }

    /// Write an `ICalendar` to the underlying writer.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write(&mut self, calendar: &ICalendar<impl StringStorage>) -> io::Result<()> {
        write_icalendar(self, calendar)
    }

    /// Write a CRLF line ending.
    pub(crate) fn writeln(&mut self) -> io::Result<()> {
        write!(self.writer, "\r\n")?;
        self.line_length = 0;
        Ok(())
    }

    /// Insert line folding: CRLF + whitespace.
    ///
    /// This inserts the RFC 5545 line folding sequence and updates the
    /// line length counter (the whitespace after CRLF counts as 1 byte).
    fn insert_fold(&mut self) -> io::Result<()> {
        self.writer
            .write_all(self.options.folding_style.as_bytes())?;
        self.line_length = FoldingStyle::continuation_len();
        Ok(())
    }
}

impl<W: Write> Write for Formatter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let Some(max_len) = self.options.folding else {
            // Folding disabled, write directly
            return self.writer.write(buf);
        };

        // Track if we're in an escape sequence (backslash followed by char)
        let mut in_escape = false;

        let mut remaining = buf;
        #[expect(clippy::indexing_slicing)]
        while !remaining.is_empty() {
            // Calculate how many bytes we can write before needing to fold
            let available = max_len.saturating_sub(self.line_length);

            if available == 0 {
                // Line is full, need to fold
                // Check if we're in a safe position to fold
                let safe_to_fold = !in_escape;
                if safe_to_fold {
                    self.insert_fold()?;
                } else {
                    // Not safe to fold here, write as-is (may exceed limit)
                    // This handles edge cases like escape sequences
                }
            }

            // Determine how many bytes we can write this iteration
            let bytes_to_write = if in_escape {
                // We're in an escape sequence, write at most 1 more byte
                // to complete the escape sequence (\n, \;, \\, \,)
                1
            } else {
                // Calculate how many bytes we can write
                let available = max_len.saturating_sub(self.line_length);
                if available == 0 {
                    // Need to fold first
                    self.insert_fold()?;
                    // After folding, we can write up to max_len - 1 (continuation)
                    max_len.saturating_sub(1)
                } else {
                    available
                }
            };

            // Don't write more than we have
            let bytes_to_write = bytes_to_write.min(remaining.len());

            // Scan for UTF-8 continuation bytes to avoid breaking multi-byte sequences
            let bytes_to_write = find_safe_write_length(remaining, bytes_to_write);

            // Write the bytes
            let written = self.writer.write(&remaining[..bytes_to_write])?;
            self.line_length += written;

            // Update escape sequence tracking
            for &byte in &remaining[..written] {
                if byte == b'\\' && !in_escape {
                    in_escape = true;
                } else if in_escape {
                    // After a backslash, the next character completes the escape
                    in_escape = false;
                }
            }

            remaining = &remaining[written..];
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Find the maximum number of bytes we can write without breaking a UTF-8 sequence.
///
/// UTF-8 encoding:
/// - 0xxxxxxx: 1 byte (ASCII)
/// - 110xxxxx: 2 bytes (starts with 0b110xxxxx)
/// - 1110xxxx: 3 bytes (starts with 0b1110xxxx)
/// - 11110xxx: 4 bytes (starts with 0b11110xxx)
/// - 10xxxxxx: continuation byte (not a start byte)
fn find_safe_write_length(buf: &[u8], max_bytes: usize) -> usize {
    if max_bytes >= buf.len() {
        return buf.len();
    }

    let mut pos = max_bytes;

    // If we're at the end of the buffer or at a safe position, return max_bytes
    if pos >= buf.len() {
        return max_bytes;
    }

    // Check if the byte at position max_bytes is a UTF-8 continuation byte
    // If so, we need to move back to avoid breaking the sequence
    #[expect(clippy::indexing_slicing)]
    while pos > 0 && (buf[pos] & 0xC0) == 0x80 {
        // This is a continuation byte (10xxxxxx)
        pos -= 1;
    }

    // pos is now at the start of a UTF-8 sequence (or 0)
    // We can write up to pos bytes safely
    pos.max(max_bytes.saturating_sub(3)) // Don't go back more than 3 bytes (max UTF-8 char size - 1)
}
