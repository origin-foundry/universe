//! The calendar engine: calendars defined over the hidden timeline, and exact conversion
//! between instants and dates.
//!
//! This is an exact crate: integers and exact fractions only, never floating point
//! (constitution principle III).
//!
//! # What it covers now (spec 004)
//!
//! - A [`CalendarDef`] describes a calendar: one fundamental unit of fixed, exact length (the
//!   definer's choice, never checked against physics), a chain of fixed-count groupings above
//!   it (optionally with a member unit, such as a year's months), a chain of equal
//!   subdivisions below it, and one era that numbers a chosen unit and places the epoch.
//! - [`Calendar::new`] checks it and refuses anything that can't work with a
//!   [`DefinitionError`] naming the unit and reason.
//! - A [`Calendar`] converts an instant to its fixed number and its date, with the exact
//!   remainder below the smallest unit ([`Calendar::instant_to_date`]), and a date back to the
//!   instant it starts at ([`Calendar::date_to_instant`]). The only rounding is to the next
//!   whole nanosecond when a date starts between two, and it's reported with the exact amount.
//!   Impossible dates and results out of range are a [`CalendarError`], never wrapped.
//!
//! # What comes later
//!
//! Leap rules and the expression language, parallel cycles, multiple eras (and counting-down
//! pre-epoch numbering), intercalary members, event-delimited units, other clocks, formats,
//! presets and rule suggestion.

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
