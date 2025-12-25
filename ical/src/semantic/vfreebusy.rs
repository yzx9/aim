// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Free/busy time component (VFREEBUSY) for iCalendar semantic components.

use crate::SemanticError;
use crate::semantic::enums::Period;
use crate::semantic::properties::{DateTime, Duration, Organizer, Text, Uri};
use crate::typed::TypedComponent;

/// Free/busy time component (VFREEBUSY)
#[derive(Debug, Clone)]
pub struct VFreeBusy {
    /// Unique identifier for the free/busy info
    pub uid: String,

    /// Date/time the free/busy info was created
    pub dt_stamp: DateTime,

    /// Start of the free/busy period
    pub dt_start: DateTime,

    /// End of the free/busy period
    pub dt_end: DateTime,

    /// Organizer of the free/busy info
    pub organizer: Organizer,

    /// Contact information
    pub contact: Option<Text>,

    /// URL for additional free/busy info
    pub url: Option<Uri>,

    /// Busy periods
    pub busy: Vec<Period>,

    /// Free periods
    pub free: Vec<Period>,

    /// Busy-tentative periods
    pub busy_tentative: Vec<Period>,

    /// Unavailable periods
    pub busy_unavailable: Vec<Period>,

    /// Duration of the free/busy info
    pub duration: Option<Duration>,
    // /// Custom properties
    // pub custom_properties: HashMap<String, Vec<String>>,
}

/// Parse a `TypedComponent` into a `VFreeBusy`
pub fn parse_vfreebusy(_comp: TypedComponent) -> Result<VFreeBusy, SemanticError> {
    todo!("Implement parse_vfreebusy")
}
