// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Miscellaneous Component Properties (RFC 5545 Section 3.8.8)
//!
//! This module contains properties that describe the content of calendar components,
//! including textual descriptions, locations, categories, and resources.
//!
//! - 3.8.8.3: `RequestStatus` - Status code for request processing

use crate::property::common::TextWithLanguage;
use crate::string_storage::StringStorage;

simple_property_wrapper!(
    /// Text property wrapper (RFC 5545 Section 3.8.8.3)
    ///
    /// Per RFC 5545, REQUEST-STATUS supports the LANGUAGE parameter but not ALTREP.
    pub RequestStatus<S> => TextWithLanguage

    ref   = pub type RequestStatusRef;
    owned = pub type RequestStatusOwned;
);
