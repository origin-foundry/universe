//! The calendar engine: calendars with a fixed-length fundamental unit, and exact conversion
//! between instants on the hidden timeline and dates.
//!
//! This is an exact crate: no floating point (constitution principle III).

#![deny(clippy::float_arithmetic)]

mod calendar;
mod convert;
mod date;
mod def;
mod error;

pub use calendar::Calendar;
pub use date::{Date, DateStart, InstantDate};
pub use def::{
    CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, MemberDef, MemberUnitDef,
    SubdivisionDef,
};
pub use error::{CalendarError, DefinitionError, Limit};
