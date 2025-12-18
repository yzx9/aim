// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

mod analysis;
mod parameter;
mod property_spec;
mod value;
mod value_datetime;
mod value_numeric;
mod value_text;

pub use crate::typed::analysis::{TypedAnalysisError, TypedComponent, typed_analysis};
pub use crate::typed::value::ValueDuration;
pub use crate::typed::value_datetime::{ValueDate, ValueDateTime, ValueTime, ValueUtcOffset};
pub use crate::typed::value_text::ValueText;
