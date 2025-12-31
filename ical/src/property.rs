// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Property specification module for iCalendar properties.
//!
//! This module provides property specifications and metadata as defined in
//! RFC 5545, including property kinds, cardinality rules, and allowed
//! parameters and value types.

mod spec;

pub use spec::{PropertyCardinality, PropertyKind, PropertySpec, ValueCardinality};
