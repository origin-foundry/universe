//! Calendar definitions: plain values that [`Calendar::new`](crate::Calendar::new) checks and
//! turns into a ready-to-use calendar. Field rules are in
//! `specs/004-calendar-engine-basics/data-model.md`.

use uni_time::{Instant, Ratio};

/// The clock a calendar counts. Only the absolute timeline (GCT) exists until Phase 4
/// (FR-007); the clock stays an explicit part of the definition so later features can change
/// it without reshaping calendars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    /// The absolute timeline itself.
    Gct,
}

/// A whole calendar, before checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDef {
    /// Not empty.
    pub name: String,
    /// Only `Clock::Gct` for now (FR-007).
    pub clock: Clock,
    /// Top to bottom; may be empty. The last one's members count the fundamental unit; each
    /// other one's count the grouping below it.
    pub groupings: Vec<GroupingDef>,
    /// Exactly one.
    pub fundamental: FundamentalDef,
    /// Top to bottom; the first splits the fundamental unit; may be empty.
    pub subdivisions: Vec<SubdivisionDef>,
    /// Exactly one.
    pub era: EraDef,
}

/// The smallest unit counted whole, such as the sol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundamentalDef {
    /// Unique among unit names.
    pub name: String,
    /// In seconds, `> 0`: any length the definer chooses, never checked against physics
    /// (FR-002a), within the engine's limits (research R3).
    pub length: Ratio,
    /// Default 1. Refused if the fundamental unit is the era's unit.
    pub first_number: Option<i64>,
}

/// A unit made of an ordered list of members, each a run of the unit directly below it, such
/// as Aster's orbit of twelve spans counted in sols, or its cohort of ten orbits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupingDef {
    /// Unique among unit names.
    pub name: String,
    /// Required when there's more than one member: the unit the members are instances of.
    pub member_unit: Option<MemberUnitDef>,
    /// At least one; each a run of the unit directly below this grouping.
    pub members: Vec<MemberDef>,
    /// Default 1. Refused if this grouping is the era's unit or above it.
    pub first_number: Option<i64>,
    /// Adjustment rules; may be empty. Only the grouping that is the era's unit may have any
    /// (spec 006, research R1).
    pub rules: Vec<AdjustmentRuleDef>,
}

/// An adjustment rule on the era's unit: in each instance whose number makes `condition` true,
/// `member` is `change` units longer (or shorter, when negative). Rules that hold in the same
/// instance add up. Field rules are in spec 006's `data-model.md`.
///
/// ```
/// # use uni_calendar::{Calendar, CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, AdjustmentRuleDef, MemberDef, MemberUnitDef};
/// # use uni_time::{Instant, Ratio};
/// # let def = CalendarDef {
/// #     name: "Adjustment".to_string(),
/// #     clock: Clock::Gct,
/// #     groupings: vec![GroupingDef {
/// #         name: "orbit".to_string(),
/// #         member_unit: Some(MemberUnitDef { name: "span".to_string(), first_number: None }),
/// #         members: vec![MemberDef { name: Some("Long".to_string()), count: 200 },
/// #             MemberDef { name: Some("Short".to_string()), count: 165 }],
/// #         first_number: None,
/// #         rules: vec![AdjustmentRuleDef { condition: "orbit % 4 == 0".to_string(),
/// #             member: "Short".to_string(), change: 1 }],
/// #     }],
/// #     fundamental: FundamentalDef { name: "sol".to_string(),
/// #         length: Ratio::from_int(86_400).unwrap(), first_number: None },
/// #     subdivisions: vec![],
/// #     era: EraDef { name: "Era".to_string(), unit: "orbit".to_string(), epoch: Instant::ZERO },
/// # };
/// // A 365-sol orbit of two spans; Short is a sol longer when `orbit % 4 == 0`.
/// let cal = Calendar::new(def).unwrap();
/// use uni_calendar::{CalendarError, Date};
///
/// // Sol 166 of Short exists only in long orbits, and the error says which orbit was checked.
/// assert!(cal.date_to_instant(&Date { values: vec![4, 2, 166] }).is_ok());
/// let refused = cal.date_to_instant(&Date { values: vec![5, 2, 166] }).unwrap_err();
/// assert_eq!(refused.to_string(), "sol 166 is outside 1 to 165 in Short in orbit 5");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdjustmentRuleDef {
    /// A true/false expression (spec 005) reading one whole-number variable named after the
    /// era's unit, such as `orbit % 4 == 0`.
    pub condition: String,
    /// The member it changes: its name, or its number from 1 when it has none.
    pub member: String,
    /// Not 0. Positive lengthens, negative shortens, in the unit the grouping's members count.
    pub change: i64,
}

/// The unit a grouping's members are instances of, such as Aster's span; its lengths come from
/// the grouping's members.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberUnitDef {
    /// Unique among unit names (e.g. "span").
    pub name: String,
    /// Default 1. Refused if the member unit is the era's unit or above it.
    pub first_number: Option<i64>,
}

/// One member of a grouping, such as Prime, 56 sols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberDef {
    /// Unique within its grouping when given (e.g. "Prime").
    pub name: Option<String>,
    /// `> 0`: how many of the unit directly below the grouping.
    pub count: i64,
}

/// A unit split into equal children, such as a sol into 20 bells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubdivisionDef {
    /// Unique among unit names (e.g. "bell").
    pub name: String,
    /// `≥ 1`: equal children of the unit above.
    pub parts: i64,
    /// Default 0.
    pub first_number: Option<i64>,
}

/// The calendar's one era, which numbers a chosen unit and places the epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EraDef {
    /// Not empty (e.g. "After Landing").
    pub name: String,
    /// Names a grouping, member unit or the fundamental unit; never a subdivision.
    pub unit: String,
    /// Where fixed number 0 starts on the hidden timeline (FR-008).
    pub epoch: Instant,
}
