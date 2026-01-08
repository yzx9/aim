// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Miscellaneous Component Properties (RFC 5545 Section 3.8.8)
//!
//! This module contains properties that describe the content of calendar components,
//! including textual descriptions, locations, categories, and resources.
//!
//! - 3.8.8.3: `RequestStatus` - Status code for request processing

use crate::property::util::Text;

simple_property_wrapper!(
    /// Simple text property wrapper (RFC 5545 Section 3.8.8.3)
    pub RequestStatus<S> => Text

    ref   = pub type RequestStatusRef;
    owned = pub type RequestStatusOwned;
);
