// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Post-semantic timezone validation.
//!
//! This module validates TZID parameters after semantic analysis completes,
//! ensuring they reference either VTIMEZONE components or IANA timezones.

use std::collections::HashSet;
use std::fmt::Write;

use crate::property::{DateTime, DateTimeProperty};
use crate::semantic::{CalendarComponent, ICalendar, SemanticError};
use crate::string_storage::Segments;

/// Context for TZID validation containing available timezone definitions.
pub struct TzContext<'a> {
    /// TZIDs defined in VTIMEZONE components
    pub vtimezone_tzids: &'a HashSet<String>,
}

impl TzContext<'_> {
    /// Validate a single `DateTimeProperty` and fill its jiff cache where available.
    ///
    /// Returns `Ok(())` if the TZID is valid (either in VTIMEZONE or IANA database).
    /// Returns `Err` if the TZID is not found in either location.
    pub fn validate_dt(
        &self,
        dt: &mut DateTimeProperty<Segments<'_>>,
    ) -> Result<(), SemanticError<'static>> {
        let Some(tz_id) = &dt.tz_id else {
            return Ok(());
        };

        let mut tzid_string = String::new();
        write!(tzid_string, "{tz_id}").expect("Writing to String should not fail");
        let span = dt.span;

        let in_vtimezone = self.vtimezone_tzids.contains(&tzid_string);

        // Jiff-specific: check IANA database and fill cache
        #[cfg(feature = "jiff")]
        let local_tz = jiff::tz::TimeZone::get(&tzid_string).ok();

        #[cfg(feature = "jiff")]
        let is_valid = in_vtimezone || local_tz.is_some();

        #[cfg(not(feature = "jiff"))]
        let is_valid = in_vtimezone;

        if !is_valid {
            return Err(SemanticError::TimezoneNotFound {
                tzid: tzid_string,
                span,
            });
        }

        // Fill cache only if jiff is available
        #[cfg(feature = "jiff")]
        if let Some(local_tz) = local_tz {
            Self::fill_tz_cache(dt, local_tz);
        }

        Ok(())
    }

    #[cfg(feature = "jiff")]
    fn fill_tz_cache(dt: &mut DateTimeProperty<Segments<'_>>, tz: jiff::tz::TimeZone) {
        if let DateTime::Zoned { date, time, .. } = dt.value {
            dt.value = DateTime::Zoned {
                date,
                time,
                tz_jiff: Some(tz),
            };
        }
    }

    /// Validate a `DateTime` value with an optional TZID.
    ///
    /// This is used for validating `DateTime` values within `RDate` and `ExDate` properties,
    /// where the TZID is stored at the property level rather than with each `DateTime`.
    ///
    /// Returns `Ok(())` if the TZID is valid (either in VTIMEZONE or IANA database).
    /// Returns `Err` if the TZID is not found in either location.
    pub fn validate_value_dt(
        &self,
        dt: &mut DateTime,
        tz_id: Option<&str>,
        span: <Segments<'_> as crate::string_storage::StringStorage>::Span,
    ) -> Result<(), SemanticError<'static>> {
        let Some(tz_id) = tz_id else {
            return Ok(());
        };

        let tzid_string = tz_id.to_string();

        let in_vtimezone = self.vtimezone_tzids.contains(&tzid_string);

        // Jiff-specific: check IANA database
        #[cfg(feature = "jiff")]
        let local_tz = jiff::tz::TimeZone::get(&tzid_string).ok();

        #[cfg(feature = "jiff")]
        let is_valid = in_vtimezone || local_tz.is_some();

        #[cfg(not(feature = "jiff"))]
        let is_valid = in_vtimezone;

        if !is_valid {
            return Err(SemanticError::TimezoneNotFound {
                tzid: tzid_string,
                span,
            });
        }

        // Fill cache only if jiff is available and DateTime is Zoned
        #[cfg(feature = "jiff")]
        if let (Some(local_tz), true) = (local_tz, matches!(dt, DateTime::Zoned { .. })) {
            Self::fill_dt_cache(dt, local_tz);
        }

        Ok(())
    }

    #[cfg(feature = "jiff")]
    fn fill_dt_cache(dt: &mut DateTime, tz: jiff::tz::TimeZone) {
        if let DateTime::Zoned { date, time, .. } = *dt {
            *dt = DateTime::Zoned {
                date,
                time,
                tz_jiff: Some(tz),
            };
        }
    }

    /// Validate all `RDate` properties in a slice.
    ///
    /// This helper validates timezone identifiers in `RDate` properties,
    /// returning all errors that occur.
    pub fn validate_rdates(
        &self,
        rdates: &mut [crate::property::RDate<Segments<'_>>],
    ) -> Vec<SemanticError<'static>> {
        let mut errors = Vec::new();

        for rdate in rdates {
            // Get TZID string from the property
            let mut tzid_string = String::new();
            let tz_id = if let Some(ref tz_id) = rdate.tz_id {
                write!(tzid_string, "{tz_id}").expect("Writing to String should not fail");
                Some(tzid_string.as_str())
            } else {
                None
            };
            let span = rdate.span;

            for value in &mut rdate.dates {
                // Only validate DateTime variants (Date and Period don't have timezones)
                if let crate::property::RDateValue::DateTime(dt) = value
                    && let Err(e) = self.validate_value_dt(dt, tz_id, span)
                {
                    errors.push(e);
                }
            }
        }

        errors
    }

    /// Validate all `ExDate` properties in a slice.
    ///
    /// This helper validates timezone identifiers in `ExDate` properties,
    /// returning all errors that occur.
    pub fn validate_exdates(
        &self,
        exdates: &mut [crate::property::ExDate<Segments<'_>>],
    ) -> Vec<SemanticError<'static>> {
        let mut errors = Vec::new();

        for exdate in exdates {
            // Get TZID string from the property
            let mut tzid_string = String::new();
            let tz_id = if let Some(ref tz_id) = exdate.tz_id {
                write!(tzid_string, "{tz_id}").expect("Writing to String should not fail");
                Some(tzid_string.as_str())
            } else {
                None
            };
            let span = exdate.span;

            for value in &mut exdate.dates {
                // Only validate DateTime variants (Date doesn't have timezone)
                if let crate::property::ExDateValue::DateTime(dt) = value
                    && let Err(e) = self.validate_value_dt(dt, tz_id, span)
                {
                    errors.push(e);
                }
            }
        }

        errors
    }
}

/// Trait for components that can validate their own TZIDs.
pub trait ValidateTzids {
    /// Validate TZID parameters and fill the jiff cache where available.
    fn validate_tzids(&mut self, ctx: &TzContext<'_>) -> Result<(), Vec<SemanticError<'static>>>;
}

/// Validate all TZID parameters in an `ICalendar`.
///
/// Returns errors for TZIDs that reference neither:
/// 1. A VTIMEZONE component in the calendar
/// 2. An IANA timezone in the local database (when jiff feature is enabled)
///
/// # Errors
///
/// Returns a vector of `SemanticError::TimezoneNotFound` for each TZID that
/// references neither a VTIMEZONE component nor an IANA timezone.
pub fn validate_tzids(
    cal: &mut ICalendar<Segments<'_>>,
) -> Result<(), Vec<SemanticError<'static>>> {
    use std::fmt::Write;

    // Collect VTIMEZONE TZIDs
    let mut vtimezone_tzids = Vec::new();
    for comp in &cal.components {
        if let CalendarComponent::VTimeZone(vtz) = comp {
            let mut tzid_string = String::new();
            write!(tzid_string, "{}", vtz.tz_id.content)
                .expect("Writing to String should not fail");
            vtimezone_tzids.push(tzid_string);
        }
    }

    let vtimezone_set: HashSet<String> = vtimezone_tzids.into_iter().collect();

    let ctx = TzContext {
        vtimezone_tzids: &vtimezone_set,
    };

    let mut errors = Vec::new();

    // Validate each component
    for component in &mut cal.components {
        if let Err(mut e) = component.validate_tzids(&ctx) {
            errors.append(&mut e);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
