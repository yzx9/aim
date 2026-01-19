// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Request building tests.

use aimcal_caldav::{CalendarMultiGetRequest, CalendarQueryRequest, Prop, PropFindRequest};

#[test]
fn request_propfind_builds_xml() {
    let mut request = PropFindRequest::new();
    request.add_property(Prop::DisplayName);
    request.add_property(Prop::GetETag);
    request.add_property(Prop::ResourceType);

    let xml = request.build().expect("Failed to build PROPFIND XML");

    assert!(xml.contains("<D:propfind"));
    assert!(xml.contains("<D:prop>"));
    assert!(xml.contains("<D:displayname>"));
    assert!(xml.contains("</D:displayname>"));
    assert!(xml.contains("<D:getetag>"));
    assert!(xml.contains("</D:getetag>"));
    assert!(xml.contains("<D:resourcetype>"));
    assert!(xml.contains("</D:resourcetype>"));
    assert!(xml.contains("</D:prop>"));
    assert!(xml.contains("</D:propfind>"));
}

#[test]
fn request_propfind_calendav_properties_includes_namespace() {
    let mut request = PropFindRequest::new();
    request.add_property(Prop::CalendarData);
    request.add_property(Prop::CalendarHomeSet);

    let xml = request.build().expect("Failed to build PROPFIND XML");

    assert!(xml.contains("xmlns:D=\"DAV:\""));
    assert!(xml.contains("xmlns:C=\"urn:ietf:params:xml:ns:caldav\""));
    assert!(xml.contains("<C:calendar-data>"));
    assert!(xml.contains("</C:calendar-data>"));
    assert!(xml.contains("<C:calendar-home-set>"));
    assert!(xml.contains("</C:calendar-home-set>"));
}

#[test]
fn request_calendar_query_builds_xml() {
    let request = CalendarQueryRequest::new()
        .component("VEVENT".to_string())
        .time_range(
            "20250101T000000Z".to_string(),
            Some("20250131T235959Z".to_string()),
        );

    let xml = request.build().expect("Failed to build calendar-query XML");

    assert!(xml.contains("<C:calendar-query"));
    assert!(xml.contains("<D:prop>"));
    assert!(xml.contains("<D:getetag>"));
    assert!(xml.contains("</D:getetag>"));
    assert!(xml.contains("<C:calendar-data>"));
    assert!(xml.contains("</C:calendar-data>"));
    assert!(xml.contains("<C:filter>"));
    assert!(xml.contains("<C:comp-filter name=\"VCALENDAR\">"));
    assert!(xml.contains("<C:comp-filter name=\"VEVENT\">"));
    assert!(xml.contains("<C:time-range"));
    assert!(xml.contains("start=\"20250101T000000Z\""));
    assert!(xml.contains("end=\"20250131T235959Z\""));
}

#[test]
fn request_calendar_query_without_component_builds_xml() {
    let request = CalendarQueryRequest::new();

    let xml = request.build().expect("Failed to build calendar-query XML");

    assert!(xml.contains("<C:calendar-query"));
    assert!(xml.contains("<C:comp-filter name=\"VCALENDAR\">"));
    // Should not have inner component filter
    assert!(!xml.contains("<C:comp-filter name=\"VEVENT\">"));
}

#[test]
fn request_calendar_multiget_builds_xml() {
    let mut request = CalendarMultiGetRequest::new();
    request.add_href("/calendars/user/event1.ics".to_string());
    request.add_href("/calendars/user/event2.ics".to_string());

    let xml = request
        .build()
        .expect("Failed to build calendar-multiget XML");

    assert!(xml.contains("<C:calendar-multiget"));
    assert!(xml.contains("<D:prop>"));
    assert!(xml.contains("<D:getetag>"));
    assert!(xml.contains("</D:getetag>"));
    assert!(xml.contains("<C:calendar-data>"));
    assert!(xml.contains("</C:calendar-data>"));
    assert!(xml.contains("<D:href>/calendars/user/event1.ics</D:href>"));
    assert!(xml.contains("<D:href>/calendars/user/event2.ics</D:href>"));
}

#[test]
fn request_calendar_multiget_empty_builds_valid_xml() {
    let request = CalendarMultiGetRequest::new();

    let xml = request
        .build()
        .expect("Failed to build calendar-multiget XML");

    assert!(xml.contains("<C:calendar-multiget"));
    assert!(xml.contains("</C:calendar-multiget>"));
    // Should have props but no hrefs
    assert!(xml.contains("<D:prop>"));
}

#[test]
fn request_free_busy_query_builds_xml() {
    let request = aimcal_caldav::FreeBusyQueryRequest::new(
        "20250101T000000Z".to_string(),
        "20250131T235959Z".to_string(),
    );

    let xml = request
        .build()
        .expect("Failed to build free-busy-query XML");

    assert!(xml.contains("<C:free-busy-query"));
    assert!(xml.contains("<C:time-range"));
    assert!(xml.contains("start=\"20250101T000000Z\""));
    assert!(xml.contains("end=\"20250131T235959Z\""));
    assert!(xml.contains("</C:free-busy-query>"));
}
