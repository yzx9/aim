// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! XML utilities for WebDAV/CalDAV processing.

use quick_xml::events::Event;

/// XML namespaces used in `CalDAV`.
pub mod ns {
    /// `WebDAV` namespace.
    pub const DAV: &str = "DAV:";

    /// `CalDAV` namespace.
    pub const CALDAV: &str = "urn:ietf:params:xml:ns:caldav";
}

/// Reads text content of an XML element.
///
/// # Errors
///
/// Returns an error if XML parsing fails.
#[expect(dead_code)]
pub fn read_element_text<R: std::io::BufRead>(
    reader: &mut quick_xml::Reader<R>,
    event: &Event,
) -> Result<Option<String>, quick_xml::Error> {
    match event {
        Event::Start(_) => {
            let mut text = String::new();
            let mut depth = 1;
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf)? {
                    Event::Start(_) => depth += 1,
                    Event::End(_) => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    Event::Text(e) => {
                        let unescaped = e.unescape()?;
                        text.push_str(unescaped.as_ref());
                    }
                    Event::Eof => break,
                    _ => {}
                }
                buf.clear();
            }
            Ok(Some(text))
        }
        _ => Ok(None),
    }
}
