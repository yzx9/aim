// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Response parsing tests.

use aimcal_caldav::MultiStatusResponse;

#[test]
fn response_parse_simple_namespace_test() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\">
  <D:response>
    <D:href>/test/</D:href>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml);
    println!("DEBUG: parse result = {:?}", response);
    assert!(response.is_ok());
    let response = response.unwrap();
    println!(
        "DEBUG: response.responses.len() = {}",
        response.responses.len()
    );
}

#[test]
fn response_parse_multistatus_basic() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\">
  <D:response>
    <D:href>/calendars/user/event1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>\"12345\"</D:getetag>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    assert_eq!(
        response.responses[0].href.as_str(),
        "/calendars/user/event1.ics"
    );
    assert_eq!(response.responses[0].prop_stats.len(), 1);
    assert_eq!(
        response.responses[0].prop_stats[0].status,
        "HTTP/1.1 200 OK"
    );
    assert_eq!(
        response.responses[0].prop_stats[0]
            .props
            .get_etag
            .as_ref()
            .unwrap()
            .as_str(),
        "\"12345\""
    );
}

#[test]
fn response_parse_calendar_collection() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/calendars/user/</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Personal Calendar</D:displayname>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
        <C:supported-calendar-component-set>
          <C:comp name=\"VEVENT\"/>
          <C:comp name=\"VTODO\"/>
        </C:supported-calendar-component-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    let prop = &response.responses[0].prop_stats[0].props;
    assert_eq!(prop.display_name.as_ref().unwrap(), "Personal Calendar");
    assert!(prop.is_calendar);
    assert!(prop.is_collection);
    assert_eq!(
        prop.supported_calendar_components.as_ref().unwrap(),
        &vec!["VEVENT".to_string(), "VTODO".to_string()]
    );
}

#[test]
fn response_into_collections_filters_calendars() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/calendars/user/personal/</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Personal</D:displayname>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
  <D:response>
    <D:href>/calendars/user/work/</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Work</D:displayname>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");
    let collections = response.into_collections();

    assert_eq!(collections.len(), 2);
    assert_eq!(collections[0].href.as_str(), "/calendars/user/personal/");
    assert_eq!(collections[0].display_name.as_ref().unwrap(), "Personal");
    assert_eq!(collections[1].href.as_str(), "/calendars/user/work/");
    assert_eq!(collections[1].display_name.as_ref().unwrap(), "Work");
}

#[test]
fn response_parse_multiple_propstats() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\">
  <D:response>
    <D:href>/calendars/user/event1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Event 1</D:displayname>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
    <D:propstat>
      <D:prop>
        <D:getetag>\"12345\"</D:getetag>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    assert_eq!(response.responses[0].prop_stats.len(), 2);
    assert_eq!(
        response.responses[0].prop_stats[0]
            .props
            .display_name
            .as_ref()
            .unwrap(),
        "Event 1"
    );
    assert_eq!(
        response.responses[0].prop_stats[1]
            .props
            .get_etag
            .as_ref()
            .unwrap()
            .as_str(),
        "\"12345\""
    );
}

#[test]
fn response_parse_calendar_data() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/calendars/user/event1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>\"12345\"</D:getetag>
        <C:calendar-data>\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example Corp.//CalDAV Client//EN\r\n\
BEGIN:VEVENT\r\n\
UID:1@example.com\r\n\
SUMMARY:Test Event\r\n\
DTSTART:20250101T120000Z\r\n\
DTEND:20250101T130000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
</C:calendar-data>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    let calendar_data = response.responses[0].prop_stats[0]
        .props
        .calendar_data
        .as_ref()
        .expect("Missing calendar data");
    assert!(calendar_data.contains("BEGIN:VCALENDAR"));
    assert!(calendar_data.contains("SUMMARY:Test Event"));
}

#[test]
fn response_parse_calendar_home_set() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/dav/principals/user/</D:href>
    <D:propstat>
      <D:prop>
        <C:calendar-home-set>
          <D:href>/dav/calendars/user/</D:href>
        </C:calendar-home-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    let calendar_home = response.responses[0].prop_stats[0]
        .props
        .calendar_home_set
        .as_ref()
        .expect("Missing calendar home set");
    assert_eq!(calendar_home.as_str(), "/dav/calendars/user/");
}

#[test]
fn response_parse_with_error_status() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\">
  <D:response>
    <D:href>/calendars/user/event1.ics</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>\"12345\"</D:getetag>
      </D:prop>
      <D:status>HTTP/1.1 404 Not Found</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");

    assert_eq!(response.responses.len(), 1);
    assert_eq!(
        response.responses[0].prop_stats[0].status,
        "HTTP/1.1 404 Not Found"
    );
}

#[test]
fn response_parse_list_calendars_xml() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
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
          <C:comp name=\"VEVENT\"/>
          <C:comp name=\"VTODO\"/>
        </C:supported-calendar-component-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");
    println!(
        "DEBUG: response.responses.len() = {}",
        response.responses.len()
    );
    for (i, resp) in response.responses.iter().enumerate() {
        println!("DEBUG: response[{}]: href={}", i, resp.href.as_str());
        for (j, prop_stat) in resp.prop_stats.iter().enumerate() {
            println!(
                "DEBUG:   prop_stat[{}]: status={}, is_calendar={}, is_collection={}, display_name={:?}",
                j,
                prop_stat.status,
                prop_stat.props.is_calendar,
                prop_stat.props.is_collection,
                prop_stat.props.display_name
            );
        }
    }

    let collections = response.into_collections();
    println!("DEBUG: collections.len() = {}", collections.len());
    assert_eq!(collections.len(), 1);
    assert_eq!(
        collections[0].href.as_str(),
        "/dav/calendars/user/personal/"
    );
    assert_eq!(
        collections[0].display_name.as_ref().unwrap(),
        "Personal Calendar"
    );
}

#[test]
fn response_parse_resourcetype_with_calendar() {
    let xml = "\
<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<D:multistatus xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:response>
    <D:href>/test/</D:href>
    <D:propstat>
      <D:prop>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>";

    let response = MultiStatusResponse::from_xml(xml).expect("Failed to parse multistatus");
    println!(
        "DEBUG: is_calendar = {}, is_collection = {}",
        response.responses[0].prop_stats[0].props.is_calendar,
        response.responses[0].prop_stats[0].props.is_collection
    );
    assert!(response.responses[0].prop_stats[0].props.is_calendar);
    assert!(response.responses[0].prop_stats[0].props.is_collection);
}
