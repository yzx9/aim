// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! `CalDAV` client for accessing and managing calendars on `CalDAV` servers (RFC 4791).

#![warn(
    trivial_casts,
    trivial_numeric_casts,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    clippy::dbg_macro,
    clippy::indexing_slicing,
    clippy::pedantic
)]
// Allow certain clippy lints that are too restrictive for this crate
#![allow(
    clippy::option_option,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::match_bool
)]

mod client;
mod config;
mod error;
mod http;
mod request;
mod response;
mod sync;
mod types;
mod xml;

pub use crate::client::{CalDavClient, DiscoverResult, FreeBusyData};
pub use crate::config::{AuthMethod, CalDavConfig};
pub use crate::error::CalDavError;
pub use crate::request::{
    CalendarMultiGetRequest, CalendarQueryRequest, FreeBusyQueryRequest, Prop, PropFindRequest,
    TextMatch, TimeRange,
};
pub use crate::response::MultiStatusResponse;
pub use crate::types::{CalendarCollection, CalendarResource, ETag, Href};
