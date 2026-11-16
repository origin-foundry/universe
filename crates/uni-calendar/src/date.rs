//! Dates: the values conversions take and give.

use uni_time::{Instant, Ratio};

/// A date, from the era's unit downwards: at least one value, at most one per unit from the
/// era's unit to the smallest unit. Missing smaller units take their first value (FR-011).
/// Units above the era's unit are never entered (FR-010).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Date {
    pub values: Vec<i64>,
}

/// What an instant converts to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstantDate {
    /// Numbers of the units above the era's unit, top to bottom (Aster: age, cohort).
    pub above: Vec<i64>,
    /// Every value from the era's unit to the smallest unit.
    pub date: Date,
    /// Exact seconds from the start of the smallest unit; `0 ≤ remainder <` that unit's length.
    pub remainder: Ratio,
}

/// One instance of the era's unit after its adjustment rules (FR-010).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EraInstance {
    /// Each member's count after the rules, in the unit the members count.
    pub members: Vec<i64>,
    /// The instance's length in fundamental units.
    pub length: i64,
    /// The numbers (from 1) of the rules that hold, in order.
    pub holding: Vec<usize>,
}

/// Where a date starts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DateStart {
    /// The first whole nanosecond at or after the date's exact start.
    pub instant: Instant,
    /// `None` if the exact start is a whole nanosecond; otherwise the exact seconds added to
    /// reach it, `0 < rounded_by < 1 ns` (FR-014).
    pub rounded_by: Option<Ratio>,
}
