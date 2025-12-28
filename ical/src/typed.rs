// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

//! Typed representation of iCalendar components and properties.

mod analysis;
mod parameter;
/// Parameter type definitions and parsing functions.
pub mod parameter_type;
mod property_spec;
mod rrule;
mod value;
mod value_datetime;
mod value_duration;
mod value_numeric;
mod value_period;
mod value_text;

pub use crate::typed::analysis::{
    TypedAnalysisError, TypedComponent, TypedProperty, typed_analysis,
};
pub use crate::typed::parameter::{TypedParameter, TypedParameterKind};
pub use crate::typed::parameter_type::{
    AlarmTriggerRelationship, CalendarUserType, Encoding, ParticipationRole, ParticipationStatus,
};
pub use crate::typed::property_spec::{
    PropertyCardinality, PropertyKind, PropertySpec, ValueCardinality,
};
pub use crate::typed::rrule::{Day, RecurrenceFrequency, RecurrenceRule, WeekDay};
pub use crate::typed::value::{Value, ValueExpected};
pub use crate::typed::value_datetime::{ValueDate, ValueDateTime, ValueTime, ValueUtcOffset};
pub use crate::typed::value_duration::ValueDuration;
pub(crate) use crate::typed::value_numeric::values_float_semicolon;
pub use crate::typed::value_period::ValuePeriod;
pub use crate::typed::value_text::ValueText;
