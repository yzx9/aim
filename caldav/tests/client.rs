// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Client integration tests with wiremock.

use aimcal_caldav::{AuthMethod, CalDavClient, CalDavConfig, CalendarQueryRequest, Href};
use aimcal_ical::{ICalendar, ProductId, ValueText, Version, formatter};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[ignore = "require network"]
async fn client_discover_calendars() {
    let mock_server = MockServer::start().await;

    // Mock OPTIONS request
    Mock::given(method("OPTIONS"))
        .and(path("/dav/calendars/user/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("DAV", "1, 2, access-control, calendar-access"),
        )
        .mount(&mock_server)
        .await;

    // Mock PROPFIND for calendar-home-set
    Mock::given(method("PROPFIND"))
        .and(path("/dav/calendars/user/"))
        .and(header("Content-Type", "application/xml; charset=utf-8"))
        .respond_with(ResponseTemplate::new(207).set_body_raw(
            "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/dav/calendars/user/</D:href>
    <D:propstat>
      <D:prop>
        <C:calendar-home-set>
          <D:href>/dav/calendars/user/</D:href>
        </C:calendar-home-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>",
            "application/xml",
        ))
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/dav/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");
    let result = client.discover().await.expect("Failed to discover");

    assert!(result.supports_calendars);
    assert_eq!(result.calendar_home.as_str(), "/dav/calendars/user/");
}

#[tokio::test]
#[ignore = "require network"]
async fn client_list_calendars() {
    let mock_server = MockServer::start().await;

    // Mock PROPFIND for calendar collections
    Mock::given(method("PROPFIND"))
        .and(path("/dav/calendars/user/"))
        .and(header("Content-Type", "application/xml; charset=utf-8"))
        .and(header("Depth", "1"))
        .respond_with(
            ResponseTemplate::new(207)
                .set_body_raw(
                    r#"\
<?xml version="1.0" encoding="utf-8" ?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:response>
    <D:href>/dav/calendars/user/personal/</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Personal Calendar</D:displayname>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
        <C:supported-calendar-component-set>
          <C:comp name="VEVENT"/>
          <C:comp name="VTODO"/>
        </C:supported-calendar-component-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#,
                    "application/xml",
                )
                .append_header("x-debug", "test-response"),
        )
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/dav/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");
    let calendars = client
        .list_calendars()
        .await
        .expect("Failed to list calendars");

    println!("DEBUG: calendars.len() = {}", calendars.len());
    for cal in &calendars {
        println!(
            "DEBUG: calendar: href={}, display_name={:?}",
            cal.href.as_str(),
            cal.display_name
        );
    }
    assert_eq!(calendars.len(), 1);
    assert_eq!(calendars[0].href.as_str(), "/dav/calendars/user/personal/");
    assert_eq!(
        calendars[0].display_name.as_ref().unwrap(),
        "Personal Calendar"
    );
    assert_eq!(
        calendars[0].supported_components,
        vec!["VEVENT".to_string(), "VTODO".to_string()]
    );
}

#[tokio::test]
#[ignore = "require network"]
async fn client_get_event() {
    let mock_server = MockServer::start().await;

    let ical_data = "\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//CalDAV Client//EN\r\n\
BEGIN:VEVENT\r\n\
UID:1@example.com\r\n\
DTSTAMP:20250101T000000Z\r\n\
DTSTART:20250101T120000Z\r\n\
DTEND:20250101T130000Z\r\n\
SUMMARY:Test Event\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    // Mock GET request
    Mock::given(method("GET"))
        .and(path("/calendars/user/event1.ics"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("ETag", "\"abc123\"")
                .set_body_string(ical_data),
        )
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");
    let resource = client
        .get_event(&Href::new("/calendars/user/event1.ics".to_string()))
        .await
        .expect("Failed to get event");

    assert_eq!(resource.href.as_str(), "/calendars/user/event1.ics");
    assert_eq!(resource.etag.as_str(), "\"abc123\"");
    let formatted = formatter::format(&resource.data).unwrap();
    assert!(formatted.contains("SUMMARY:Test Event"));
}

#[tokio::test]
#[ignore = "require network"]
async fn client_query_events() {
    let mock_server = MockServer::start().await;

    let ical_data = r#"\
<?xml version="1.0" encoding="utf-8" ?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:response>
    <D:href>/calendars/user/event1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>"12345"</D:getetag>
        <C:calendar-data>BEGIN:VCALENDAR&#13;&#10;VERSION:2.0&#13;&#10;PRODID:-//Example Corp.//CalDAV Client//EN&#13;&#10;BEGIN:VEVENT&#13;&#10;UID:1@example.com&#13;&#10;DTSTAMP:20250101T000000Z&#13;&#10;DTSTART:20250101T120000Z&#13;&#10;DTEND:20250101T130000Z&#13;&#10;SUMMARY:Test Event&#13;&#10;END:VEVENT&#13;&#10;END:VCALENDAR&#13;&#10;</C:calendar-data>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#;

    // Mock REPORT request
    Mock::given(method("REPORT"))
        .and(path("/calendars/user/"))
        .and(header("Content-Type", "application/xml; charset=utf-8"))
        .respond_with(ResponseTemplate::new(207).set_body_raw(ical_data, "application/xml"))
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");

    let request = CalendarQueryRequest::new()
        .component("VEVENT".to_string())
        .time_range(
            "20250101T000000Z".to_string(),
            Some("20250131T235959Z".to_string()),
        );

    let events = client
        .query(&Href::new("/calendars/user/".to_string()), &request)
        .await
        .expect("Failed to query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].href.as_str(), "/calendars/user/event1.ics");
    let formatted = formatter::format(&events[0].data).unwrap();
    assert!(formatted.contains("SUMMARY:Test Event"));
}

#[tokio::test]
#[ignore = "require network"]
async fn client_create_event() {
    let mock_server = MockServer::start().await;

    // Mock PUT request
    Mock::given(method("PUT"))
        .and(path("/calendars/user/new-event.ics"))
        .respond_with(
            ResponseTemplate::new(201)
                .insert_header("ETag", "\"new-etag\"")
                .set_body_string(""),
        )
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");

    // Create a minimal iCalendar
    let mut ical = ICalendar::new();
    ical.version = Version::default();
    ical.prod_id = ProductId {
        value: ValueText::new("-//Test//CalDAV Client//EN".to_string()),
        x_parameters: Vec::new(),
        retained_parameters: Vec::new(),
        span: (),
    };

    let etag = client
        .create_event(
            &Href::new("/calendars/user/new-event.ics".to_string()),
            &ical,
        )
        .await
        .expect("Failed to create event");

    assert_eq!(etag.as_str(), "\"new-etag\"");
}

#[tokio::test]
#[ignore = "require network"]
async fn client_update_event_with_etag() {
    let mock_server = MockServer::start().await;

    // Mock PUT request with If-Match
    Mock::given(method("PUT"))
        .and(path("/calendars/user/event1.ics"))
        .and(header("if-match", "\"old-etag\""))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("ETag", "\"new-etag\"")
                .set_body_string(""),
        )
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");

    let mut ical = aimcal_ical::ICalendar::new();
    ical.version = aimcal_ical::Version::default();
    ical.prod_id = aimcal_ical::ProductId {
        value: ValueText::new("-//Test//CalDAV Client//EN".to_string()),
        x_parameters: Vec::new(),
        retained_parameters: Vec::new(),
        span: (),
    };

    let etag = client
        .update_event(
            &Href::new("/calendars/user/event1.ics".to_string()),
            &aimcal_caldav::ETag::new("\"old-etag\"".to_string()),
            &ical,
        )
        .await
        .expect("Failed to update event");

    assert_eq!(etag.as_str(), "\"new-etag\"");
}

#[tokio::test]
#[ignore = "require network"]
async fn client_delete_event() {
    let mock_server = MockServer::start().await;

    // Mock DELETE request
    Mock::given(method("DELETE"))
        .and(path("/calendars/user/event1.ics"))
        .and(header("if-match", "\"some-etag\""))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/calendars/user/".to_string(),
        auth: AuthMethod::None,
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");

    client
        .delete_event(
            &Href::new("/calendars/user/event1.ics".to_string()),
            &aimcal_caldav::ETag::new("\"some-etag\"".to_string()),
        )
        .await
        .expect("Failed to delete event");
}

#[tokio::test]
#[ignore = "require network"]
async fn client_basic_auth_headers() {
    let mock_server = MockServer::start().await;

    // Mock OPTIONS that requires auth
    Mock::given(method("OPTIONS"))
        .and(path("/dav/calendars/user/"))
        .and(header("authorization", "Basic dXNlcjpwYXNz")) // base64 of "user:pass"
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("DAV", "1, 2, access-control, calendar-access"),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("PROPFIND"))
        .and(path("/dav/calendars/user/"))
        .and(header("authorization", "Basic dXNlcjpwYXNz"))
        .respond_with(ResponseTemplate::new(207).set_body_raw(
            "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/dav/calendars/user/</D:href>
    <D:propstat>
      <D:prop>
        <C:calendar-home-set>
          <D:href>/dav/calendars/user/</D:href>
        </C:calendar-home-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>",
            "application/xml",
        ))
        .mount(&mock_server)
        .await;

    let config = CalDavConfig {
        base_url: mock_server.uri(),
        calendar_home: "/dav/calendars/user/".to_string(),
        auth: AuthMethod::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        },
        ..Default::default()
    };

    let client = CalDavClient::new(config).expect("Failed to create client");
    let _result = client.discover().await.expect("Failed to discover");
}
