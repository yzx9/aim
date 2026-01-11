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

/// Formatting options for the iCalendar formatter.
///
/// Currently all options are defaults, but this struct provides
/// future extensibility for features like line folding, etc.
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Line ending style (CRLF per RFC 5545).
    /// This field is currently always true but may become configurable.
    _crlf: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { _crlf: true }
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
    _options: FormatOptions,
}

impl<W: Write> Formatter<W> {
    /// Create a new formatter with default options.
    #[must_use]
    pub fn new(writer: W) -> Self {
        Self::with_options(writer, FormatOptions::default())
    }

    /// Create a new formatter with custom options.
    #[must_use]
    pub fn with_options(writer: W, options: FormatOptions) -> Self {
        Self {
            writer,
            _options: options,
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

    /// Write a CRLF line ending.
    pub(crate) fn writeln(&mut self) -> io::Result<()> {
        write!(self.writer, "\r\n")
    }
}

impl<W: Write> Write for Formatter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf) // TODO: handle line folding
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

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
/// let ics_string = format_to_string(calendar)?;
/// println!("{}", ics_string);
/// ```
///
/// # Errors
///
/// Returns an error if writing to the internal buffer fails or if the output
/// contains invalid UTF-8 data.
pub fn format<S: StringStorage>(calendar: &ICalendar<S>) -> io::Result<String> {
    let options = FormatOptions::default();
    let mut buffer = Vec::new();
    let mut formatter = Formatter::with_options(&mut buffer, options);
    write_icalendar(&mut formatter, calendar)?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
