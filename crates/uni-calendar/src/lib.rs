//! The calendar engine: calendars defined over the hidden timeline, and exact conversion
//! between instants and dates.
//!
//! This is an exact crate: integers and exact fractions only, never floating point
//! (constitution principle III).
//!
//! # What it covers (spec 004)
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
//! # Leap rules (spec 006)
//!
//! - The era's unit, when it is a grouping, may carry [`LeapRuleDef`]s: a true/false
//!   expression (spec 005's `uni-expr`) of the unit's own number, under its own name, and a
//!   change that lengthens or shortens one member. Rules that hold together add up.
//! - [`Calendar::new`] checks the rules with the rest of the definition, works out how often
//!   they repeat ([`Calendar::rule_period`], at most [`RULE_PERIOD_LIMIT`] instances), and
//!   evaluates them once for one period, so every failure is found then and conversion never
//!   meets an expression error.
//! - Both conversion directions find any year through that one period (the supercycle), in
//!   time independent of the distance from the epoch. [`Calendar::era_instance`] reports a
//!   year's member lengths and which rules hold.
//!
//! # What comes later
//!
//! Rules on other units and rules that read more than the era's unit's number; inserting or
//! removing members, intercalary members, length and start expressions; rules that don't
//! repeat (postponements, observational calendars); parallel cycles, multiple eras (and
//! counting-down pre-epoch numbering), event-delimited units, other clocks, formats, presets
//! and rule suggestion.

#![deny(clippy::float_arithmetic)]

mod calendar;
mod convert;
mod date;
mod def;
mod error;
mod rules;

pub use calendar::{Calendar, RULE_PERIOD_LIMIT};
pub use date::{Date, DateStart, EraInstance, InstantDate};
pub use def::{
    CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, LeapRuleDef, MemberDef, MemberUnitDef,
    SubdivisionDef,
};
pub use error::{CalendarError, DefinitionError, EraNumber, Limit};
