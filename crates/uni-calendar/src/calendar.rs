//! Checking a definition and the checked calendar it becomes.

use std::collections::HashSet;

use uni_time::{Instant, Ratio};

use crate::rules::Cycle;
use crate::{CalendarDef, CalendarError, Clock, DefinitionError, EraInstance, Limit};

type R<T> = Result<T, CalendarError>;

const NANOS_PER_SEC: i128 = 1_000_000_000;
/// The largest denominator a fundamental length in nanoseconds may have (research R3).
const MAX_DENOMINATOR: i128 = 1_000_000_000;
/// The fundamental unit's length in ticks must stay below this (research R3).
const MAX_TICKS: i128 = 1 << 120;

/// The most instances of the era's unit its leap rules may take to repeat (spec 006,
/// research R3).
pub const RULE_PERIOD_LIMIT: i64 = 100_000;

/// A checked calendar, ready to convert between instants and dates. Immutable; made only by
/// [`Calendar::new`].
///
/// Inside, the definition is flattened into one chain of units, top to bottom (data-model.md,
/// "The unit chain"), with each grouping's length in fundamental units, each member unit's
/// prefix sums, and the fundamental length in nanoseconds as a reduced fraction `p/q`
/// precomputed. A **tick** is `1/(q × P)` ns, where `P` is the product of the subdivisions'
/// part counts; every unit boundary is a whole number of ticks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calendar {
    pub(crate) name: String,
    pub(crate) clock: Clock,
    pub(crate) era_name: String,
    pub(crate) epoch: Instant,
    /// The chain, top to bottom.
    pub(crate) units: Vec<Unit>,
    /// Index of the era's unit in `units`.
    pub(crate) era: usize,
    /// Index of the fundamental unit in `units`.
    pub(crate) fundamental: usize,
    /// The fundamental length in nanoseconds is `p / q`, in lowest terms.
    pub(crate) p: i128,
    pub(crate) q: i128,
    /// The product of the subdivisions' part counts.
    pub(crate) parts: i128,
    /// The era's unit's cycle when it is a grouping (spec 006, research R4); `None` keeps the
    /// spec 004 path for an era that numbers a member unit or the fundamental unit.
    pub(crate) cycle: Option<Cycle>,
}

/// One unit of the chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Unit {
    pub(crate) name: String,
    /// The number of its first instance within its parent; unused at or above the era's unit.
    pub(crate) first: i64,
    pub(crate) kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `length` fundamental units, holding `children` of the next unit in the chain (its one
    /// member's count, or the number of members when it has a member unit).
    Grouping {
        length: i64,
        children: i64,
    },
    /// A grouping's member unit. Member `j` holds `counts[j]` of the next unit in the chain
    /// and starts `starts[j]` fundamental units into the grouping (`starts` has one more
    /// entry, the grouping's length).
    Member {
        names: Vec<Option<String>>,
        counts: Vec<i64>,
        starts: Vec<i64>,
    },
    Fundamental,
    Subdivision {
        parts: i64,
    },
}

impl Unit {
    /// How a member of a member unit is named in errors: its name, or its number from 1.
    pub(crate) fn member_label(names: &[Option<String>], j: usize) -> String {
        names[j].clone().unwrap_or_else(|| (j + 1).to_string())
    }
}

impl Calendar {
    /// Checks a definition and builds the calendar, or says what's wrong with it (FR-006).
    ///
    /// The rules are checked in the order data-model.md lists them, so the first fault found
    /// is the one reported.
    ///
    /// ```
    /// use uni_calendar::{Calendar, CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef,
    ///     MemberDef, SubdivisionDef};
    /// use uni_time::{Instant, Ratio};
    ///
    /// // A 400-sol year with 10 hours in each sol.
    /// let def = CalendarDef {
    ///     name: "Plain".to_string(),
    ///     clock: Clock::Gct,
    ///     groupings: vec![GroupingDef {
    ///         name: "year".to_string(),
    ///         member_unit: None,
    ///         members: vec![MemberDef { name: None, count: 400 }],
    ///         first_number: None,
    ///         rules: vec![],
    ///     }],
    ///     fundamental: FundamentalDef {
    ///         name: "sol".to_string(),
    ///         length: Ratio::from_int(90_000).unwrap(),
    ///         first_number: None,
    ///     },
    ///     subdivisions: vec![SubdivisionDef { name: "hour".to_string(), parts: 10, first_number: None }],
    ///     era: EraDef { name: "Plain Era".to_string(), unit: "year".to_string(), epoch: Instant::ZERO },
    /// };
    /// let cal = Calendar::new(def).unwrap();
    /// assert_eq!(cal.units(), ["year", "sol", "hour"]);
    /// assert_eq!(cal.units_per("year", "hour"), Ok(4_000));
    /// ```
    pub fn new(def: CalendarDef) -> Result<Calendar, DefinitionError> {
        check_names(&def)?;
        check_shape(&def)?;
        let units = flatten(&def);
        check_unique(&def, &units)?;
        let era = era_index(&def, &units)?;
        check_first_numbers(&def, era)?;
        let (p, q, parts) = check_fundamental_limits(&def)?;
        let units = with_lengths(&def, units)?;
        let cycle = Cycle::build(&def, &units, era)?;
        Ok(Calendar {
            name: def.name,
            clock: def.clock,
            era_name: def.era.name,
            epoch: def.era.epoch,
            fundamental: def.groupings.len()
                + def
                    .groupings
                    .iter()
                    .filter(|g| g.member_unit.is_some())
                    .count(),
            units,
            era,
            p,
            q,
            parts,
            cycle,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }

    pub fn era_name(&self) -> &str {
        &self.era_name
    }

    /// Where fixed number 0 starts on the hidden timeline.
    pub fn epoch(&self) -> Instant {
        self.epoch
    }

    /// The chain of units, top to bottom, member units included.
    pub fn units(&self) -> Vec<&str> {
        self.units.iter().map(|u| u.name.as_str()).collect()
    }

    pub fn era_unit(&self) -> &str {
        &self.units[self.era].name
    }

    /// The number of instances of the era's unit after which its rules repeat (1 without
    /// rules).
    pub fn rule_period(&self) -> i64 {
        self.cycle.as_ref().map_or(1, |c| c.period)
    }

    /// One instance of the era's unit after its rules (FR-010).
    pub fn era_instance(&self, _number: i64) -> R<EraInstance> {
        todo!()
    }

    /// How many `inner` there are in one `outer`, when that's one fixed whole number: a year
    /// in sols is 668, a sol in minutes 2,000, a year in months 12. A count that varies (a
    /// month in sols), or isn't whole, is `NotFixed`.
    pub fn units_per(&self, outer: &str, inner: &str) -> R<i64> {
        let o = self.index_of(outer)?;
        let i = self.index_of(inner)?;
        self.count_between(o, i)
            .ok_or_else(|| CalendarError::NotFixed {
                unit: outer.to_string(),
                of: inner.to_string(),
            })?
    }

    fn index_of(&self, name: &str) -> R<usize> {
        self.units
            .iter()
            .position(|u| u.name == name)
            .ok_or_else(|| CalendarError::UnknownUnit {
                name: name.to_string(),
            })
    }

    /// `None` when the count isn't fixed; `Some(Err)` when it is but doesn't fit in `i64`.
    fn count_between(&self, o: usize, i: usize) -> Option<R<i64>> {
        if o == i {
            return Some(Ok(1));
        }
        if o > i {
            return None;
        }
        if let Kind::Member { counts, .. } = &self.units[i].kind {
            // A member unit's instances vary, but every instance of its grouping (just above
            // it) holds all of them.
            let members = i64::try_from(counts.len()).expect("a Vec's length fits");
            return Some(match self.count_between(o, i - 1)? {
                Ok(n) => n.checked_mul(members).ok_or(CalendarError::OutOfRange),
                Err(e) => Err(e),
            });
        }
        let ratio = self
            .length_in_fundamentals(o)?
            .checked_div(self.length_in_fundamentals(i)?)
            .ok()?;
        if !ratio.is_integer() {
            return None;
        }
        Some(i64::try_from(ratio.numer()).map_err(|_| CalendarError::OutOfRange))
    }

    /// The unit's length in fundamental units, if it's always the same.
    fn length_in_fundamentals(&self, index: usize) -> Option<Ratio> {
        match &self.units[index].kind {
            Kind::Grouping { length, .. } => Ratio::from_int((*length).into()).ok(),
            Kind::Member { starts, .. } => {
                let mut lengths = starts.windows(2).map(|w| w[1] - w[0]);
                let first = lengths.next()?;
                lengths
                    .all(|l| l == first)
                    .then(|| Ratio::from_int(first.into()).ok())?
            }
            Kind::Fundamental => Some(Ratio::ONE),
            Kind::Subdivision { .. } => {
                let mut parts: i128 = 1;
                for unit in &self.units[self.fundamental + 1..=index] {
                    if let Kind::Subdivision { parts: n } = unit.kind {
                        parts *= i128::from(n); // ≤ the product checked in `new`
                    }
                }
                Ratio::new(1, parts).ok()
            }
        }
    }
}

fn check_names(def: &CalendarDef) -> Result<(), DefinitionError> {
    let empty = |what| Err(DefinitionError::EmptyName { what });
    if def.name.is_empty() {
        return empty("calendar");
    }
    if def.era.name.is_empty() {
        return empty("era");
    }
    for g in &def.groupings {
        if g.name.is_empty() || g.member_unit.as_ref().is_some_and(|m| m.name.is_empty()) {
            return empty("unit");
        }
        if g.members.iter().any(|m| m.name.as_deref() == Some("")) {
            return empty("member");
        }
    }
    if def.fundamental.name.is_empty() || def.subdivisions.iter().any(|s| s.name.is_empty()) {
        return empty("unit");
    }
    Ok(())
}

fn check_shape(def: &CalendarDef) -> Result<(), DefinitionError> {
    if def.fundamental.length <= Ratio::ZERO {
        return Err(DefinitionError::NonPositiveLength {
            unit: def.fundamental.name.clone(),
        });
    }
    if let Some(g) = def.groupings.iter().find(|g| g.members.is_empty()) {
        return Err(DefinitionError::NoMembers {
            unit: g.name.clone(),
        });
    }
    for g in &def.groupings {
        if let Some((j, m)) = g.members.iter().enumerate().find(|(_, m)| m.count <= 0) {
            return Err(DefinitionError::NonPositiveCount {
                unit: g.name.clone(),
                member: m.name.clone().unwrap_or_else(|| (j + 1).to_string()),
            });
        }
    }
    if let Some(g) = def
        .groupings
        .iter()
        .find(|g| g.members.len() > 1 && g.member_unit.is_none())
    {
        return Err(DefinitionError::MembersNeedUnit {
            unit: g.name.clone(),
        });
    }
    if let Some(s) = def.subdivisions.iter().find(|s| s.parts < 1) {
        return Err(DefinitionError::TooFewParts {
            unit: s.name.clone(),
        });
    }
    Ok(())
}

/// The chain, top to bottom, with first numbers defaulted ("Default 1" for groupings, member
/// units and the fundamental unit; "Default 0" for subdivisions). Lengths come later.
fn flatten(def: &CalendarDef) -> Vec<Unit> {
    let mut units = Vec::new();
    let placeholder = || Kind::Grouping {
        length: 0,
        children: 0,
    };
    for g in &def.groupings {
        units.push(Unit {
            name: g.name.clone(),
            first: g.first_number.unwrap_or(1),
            kind: placeholder(),
        });
        if let Some(m) = &g.member_unit {
            units.push(Unit {
                name: m.name.clone(),
                first: m.first_number.unwrap_or(1),
                kind: placeholder(),
            });
        }
    }
    units.push(Unit {
        name: def.fundamental.name.clone(),
        first: def.fundamental.first_number.unwrap_or(1),
        kind: Kind::Fundamental,
    });
    for s in &def.subdivisions {
        units.push(Unit {
            name: s.name.clone(),
            first: s.first_number.unwrap_or(0),
            kind: Kind::Subdivision { parts: s.parts },
        });
    }
    units
}

fn check_unique(def: &CalendarDef, units: &[Unit]) -> Result<(), DefinitionError> {
    let mut seen = HashSet::new();
    if let Some(u) = units.iter().find(|u| !seen.insert(u.name.as_str())) {
        return Err(DefinitionError::DuplicateUnit {
            name: u.name.clone(),
        });
    }
    for g in &def.groupings {
        let mut seen = HashSet::new();
        if let Some(name) = g
            .members
            .iter()
            .filter_map(|m| m.name.as_deref())
            .find(|name| !seen.insert(*name))
        {
            return Err(DefinitionError::DuplicateMember {
                unit: g.name.clone(),
                name: name.to_string(),
            });
        }
    }
    Ok(())
}

fn era_index(def: &CalendarDef, units: &[Unit]) -> Result<usize, DefinitionError> {
    let unit = &def.era.unit;
    let index = units
        .iter()
        .position(|u| &u.name == unit)
        .ok_or_else(|| DefinitionError::EraUnitUnknown { unit: unit.clone() })?;
    if matches!(units[index].kind, Kind::Subdivision { .. }) {
        return Err(DefinitionError::EraUnitBelowFundamental { unit: unit.clone() });
    }
    Ok(index)
}

/// A first number may only be given below the era's unit: the era numbers the rest.
fn check_first_numbers(def: &CalendarDef, era: usize) -> Result<(), DefinitionError> {
    let mut given = Vec::new(); // (name, given?) in chain order
    for g in &def.groupings {
        given.push((&g.name, g.first_number.is_some()));
        if let Some(m) = &g.member_unit {
            given.push((&m.name, m.first_number.is_some()));
        }
    }
    given.push((
        &def.fundamental.name,
        def.fundamental.first_number.is_some(),
    ));
    match given.iter().take(era + 1).find(|(_, is_given)| *is_given) {
        Some((name, _)) => Err(DefinitionError::FirstNumberNotAllowed {
            unit: (*name).clone(),
        }),
        None => Ok(()),
    }
}

/// The fundamental length in nanoseconds as `(p, q)`, and the product of the part counts,
/// checked against the limits in research R3.
fn check_fundamental_limits(def: &CalendarDef) -> Result<(i128, i128, i128), DefinitionError> {
    let fundamental = &def.fundamental.name;
    let beyond = |unit: &String, limit| DefinitionError::BeyondLimits {
        unit: unit.clone(),
        limit,
    };
    let in_ns = def
        .fundamental
        .length
        .checked_mul(Ratio::from_int(NANOS_PER_SEC).expect("fits"))
        .map_err(|_| beyond(fundamental, Limit::TooManyTicks))?;
    let (p, q) = (in_ns.numer(), in_ns.denom());
    if q > MAX_DENOMINATOR {
        return Err(beyond(fundamental, Limit::LengthTooFine));
    }
    let parts = def
        .subdivisions
        .iter()
        .try_fold(1i128, |acc, s| acc.checked_mul(i128::from(s.parts)));
    // The smallest unit is p / (q × P) ns; it must be at least 1 ns.
    let smallest = def.subdivisions.last().map_or(fundamental, |s| &s.name);
    match parts.and_then(|n| n.checked_mul(q)) {
        Some(q_parts) if p >= q_parts => {}
        _ => return Err(beyond(smallest, Limit::ShorterThanNanosecond)),
    }
    let parts = parts.expect("checked just above");
    match p.checked_mul(parts) {
        Some(ticks) if ticks < MAX_TICKS => {}
        _ => return Err(beyond(fundamental, Limit::TooManyTicks)),
    }
    Ok((p, q, parts))
}

/// Fills in each grouping's length in fundamental units and each member unit's prefix sums,
/// bottom up, refusing any that don't fit in `i64`.
fn with_lengths(def: &CalendarDef, mut units: Vec<Unit>) -> Result<Vec<Unit>, DefinitionError> {
    let too_long = |g: &crate::GroupingDef| DefinitionError::BeyondLimits {
        unit: g.name.clone(),
        limit: Limit::GroupingTooLong,
    };
    // Walk the groupings bottom up; `index` is each grouping's place in the chain.
    let mut index = def.groupings.len()
        + def
            .groupings
            .iter()
            .filter(|g| g.member_unit.is_some())
            .count();
    let mut below: i64 = 1; // length of the unit directly below, in fundamental units
    for g in def.groupings.iter().rev() {
        let mut starts = Vec::with_capacity(g.members.len() + 1);
        let mut total: i64 = 0;
        starts.push(0);
        for m in &g.members {
            total = m
                .count
                .checked_mul(below)
                .and_then(|l| total.checked_add(l))
                .ok_or_else(|| too_long(g))?;
            starts.push(total);
        }
        if let Some(_member_unit) = &g.member_unit {
            index -= 1;
            units[index].kind = Kind::Member {
                names: g.members.iter().map(|m| m.name.clone()).collect(),
                counts: g.members.iter().map(|m| m.count).collect(),
                starts,
            };
        }
        index -= 1;
        let children = if g.member_unit.is_some() {
            i64::try_from(g.members.len()).map_err(|_| too_long(g))?
        } else {
            g.members[0].count
        };
        units[index].kind = Kind::Grouping {
            length: total,
            children,
        };
        below = total;
    }
    Ok(units)
}

#[cfg(test)]
pub(crate) mod tests {
    use uni_time::{Instant, Ratio};

    use super::*;
    use crate::{
        EraDef, FundamentalDef, GroupingDef, Limit, MemberDef, MemberUnitDef, SubdivisionDef,
    };

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn one(name: &str, count: i64) -> GroupingDef {
        GroupingDef {
            name: s(name),
            member_unit: None,
            members: vec![MemberDef { name: None, count }],
            first_number: None,
            rules: vec![],
        }
    }

    fn sub(name: &str, parts: i64) -> SubdivisionDef {
        SubdivisionDef {
            name: s(name),
            parts,
            first_number: None,
        }
    }

    /// Aster, as in the spec, built here because unit tests can't see `tests/common`.
    pub(crate) fn aster() -> CalendarDef {
        let counts = [56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55];
        let names = [
            "Prime", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth", "Ninth",
            "Tenth", "Eleventh", "Twelfth",
        ];
        CalendarDef {
            name: s("Aster"),
            clock: Clock::Gct,
            groupings: vec![
                one("age", 10),
                one("decade", 10),
                GroupingDef {
                    name: s("year"),
                    member_unit: Some(MemberUnitDef {
                        name: s("month"),
                        first_number: None,
                    }),
                    members: names
                        .iter()
                        .zip(counts)
                        .map(|(name, count)| MemberDef {
                            name: Some(s(name)),
                            count,
                        })
                        .collect(),
                    first_number: None,
                    rules: vec![],
                },
            ],
            fundamental: FundamentalDef {
                name: s("sol"),
                length: Ratio::new(88_775_244, 1000).unwrap(),
                first_number: None,
            },
            subdivisions: vec![sub("hour", 20), sub("minute", 100)],
            era: EraDef {
                name: s("After Landing"),
                unit: s("year"),
                epoch: Instant::ZERO,
            },
        }
    }

    /// Only a fundamental unit, numbered by the era.
    fn bare(length: Ratio) -> CalendarDef {
        CalendarDef {
            name: s("Bare"),
            clock: Clock::Gct,
            groupings: vec![],
            fundamental: FundamentalDef {
                name: s("tick"),
                length,
                first_number: None,
            },
            subdivisions: vec![],
            era: EraDef {
                name: s("Count"),
                unit: s("tick"),
                epoch: Instant::ZERO,
            },
        }
    }

    fn refused(def: CalendarDef) -> DefinitionError {
        Calendar::new(def).expect_err("should be refused")
    }

    fn secs(num: i128, den: i128) -> Ratio {
        Ratio::new(num, den).unwrap()
    }

    fn beyond(unit: &str, limit: Limit) -> DefinitionError {
        DefinitionError::BeyondLimits {
            unit: s(unit),
            limit,
        }
    }

    #[test]
    fn non_positive_lengths() {
        for length in [Ratio::ZERO, secs(-1, 1)] {
            let mut def = aster();
            def.fundamental.length = length;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveLength { unit: s("sol") }
            );
        }
    }

    #[test]
    fn grouping_without_members() {
        let mut def = aster();
        def.groupings[1].members.clear();
        assert_eq!(
            refused(def),
            DefinitionError::NoMembers { unit: s("decade") }
        );
    }

    #[test]
    fn non_positive_counts_name_the_member() {
        for count in [0, -3] {
            let mut def = aster();
            def.groupings[2].members[2].count = count;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveCount {
                    unit: s("year"),
                    member: s("Third")
                }
            );
            let mut def = aster();
            def.groupings[0].members[0].count = count;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveCount {
                    unit: s("age"),
                    member: s("1")
                }
            );
        }
    }

    #[test]
    fn several_members_need_a_member_unit() {
        let mut def = aster();
        def.groupings[1].members.push(MemberDef {
            name: None,
            count: 5,
        });
        assert_eq!(
            refused(def),
            DefinitionError::MembersNeedUnit { unit: s("decade") }
        );
    }

    #[test]
    fn subdivision_into_no_parts() {
        let mut def = aster();
        def.subdivisions[1].parts = 0;
        assert_eq!(
            refused(def),
            DefinitionError::TooFewParts { unit: s("minute") }
        );
    }

    #[test]
    fn duplicate_unit_names() {
        let mut def = aster();
        def.subdivisions[1].name = s("hour");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("hour") }
        );
        let mut def = aster();
        def.groupings[2].member_unit.as_mut().unwrap().name = s("decade");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("decade") }
        );
        let mut def = aster();
        def.fundamental.name = s("age");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("age") }
        );
    }

    #[test]
    fn duplicate_member_names() {
        let mut def = aster();
        def.groupings[2].members[1].name = Some(s("Prime"));
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateMember {
                unit: s("year"),
                name: s("Prime")
            }
        );
    }

    #[test]
    fn era_unit_must_be_at_or_above_the_fundamental_unit() {
        let mut def = aster();
        def.era.unit = s("week");
        assert_eq!(
            refused(def),
            DefinitionError::EraUnitUnknown { unit: s("week") }
        );
        let mut def = aster();
        def.era.unit = s("hour");
        assert_eq!(
            refused(def),
            DefinitionError::EraUnitBelowFundamental { unit: s("hour") }
        );
    }

    #[test]
    fn first_numbers_only_below_the_era_unit() {
        let mut def = aster();
        def.groupings[2].first_number = Some(0);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("year") }
        );
        let mut def = aster();
        def.groupings[1].first_number = Some(1);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("decade") }
        );
        let mut def = bare(Ratio::ONE);
        def.fundamental.first_number = Some(0);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("tick") }
        );
        // Below the era's unit, any whole number is fine.
        let mut def = aster();
        def.groupings[2].member_unit.as_mut().unwrap().first_number = Some(-4);
        def.fundamental.first_number = Some(0);
        def.subdivisions[0].first_number = Some(1);
        assert!(Calendar::new(def).is_ok());
    }

    #[test]
    fn empty_names() {
        let mut def = aster();
        def.name.clear();
        assert_eq!(
            refused(def),
            DefinitionError::EmptyName { what: "calendar" }
        );
        let mut def = aster();
        def.era.name.clear();
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "era" });
        let mut def = aster();
        def.subdivisions[0].name.clear();
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "unit" });
        let mut def = aster();
        def.groupings[2].members[0].name = Some(String::new());
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "member" });
    }

    #[test]
    fn length_too_fine() {
        // 1 + 1/10¹⁹ s is 10⁹ + 1/10¹⁰ ns: a denominator of 10¹⁰.
        let ten_19 = 10_000_000_000_000_000_000;
        let def = bare(secs(ten_19 + 1, ten_19));
        assert_eq!(refused(def), beyond("tick", Limit::LengthTooFine));
        // A denominator of exactly 10⁹ is fine: 1/10¹⁸ s more than a second.
        let ten_18 = 1_000_000_000_000_000_000;
        assert!(Calendar::new(bare(secs(ten_18 + 1, ten_18))).is_ok());
    }

    #[test]
    fn smallest_unit_shorter_than_a_nanosecond() {
        let mut def = bare(secs(1, 1_000_000_000));
        def.subdivisions = vec![sub("half", 2)];
        assert_eq!(refused(def), beyond("half", Limit::ShorterThanNanosecond));
        let def = bare(secs(1, 2_000_000_000));
        assert_eq!(refused(def), beyond("tick", Limit::ShorterThanNanosecond));
    }

    #[test]
    fn too_many_ticks() {
        let mut def = bare(secs(1_000_000 * 31_557_600, 1));
        def.subdivisions = vec![sub("minute", 1 << 60)];
        assert_eq!(refused(def), beyond("tick", Limit::TooManyTicks));
    }

    #[test]
    fn grouping_too_long() {
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![one("big", 1 << 32), one("bigger", 1 << 32)];
        def.era.unit = s("big");
        assert_eq!(refused(def), beyond("big", Limit::GroupingTooLong));
        // Members that add up past i64 overflow too.
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![GroupingDef {
            name: s("wide"),
            member_unit: Some(MemberUnitDef {
                name: s("part"),
                first_number: None,
            }),
            members: vec![
                MemberDef {
                    name: None,
                    count: i64::MAX,
                },
                MemberDef {
                    name: None,
                    count: 1,
                },
            ],
            first_number: None,
            rules: vec![],
        }];
        def.era.unit = s("wide");
        assert_eq!(refused(def), beyond("wide", Limit::GroupingTooLong));
    }

    #[test]
    fn accepted_edge_shapes() {
        // Only a fundamental unit, which its era numbers.
        let cal = Calendar::new(bare(Ratio::ONE)).unwrap();
        assert_eq!(cal.units(), ["tick"]);
        assert_eq!(cal.era_unit(), "tick");
        // A year of one 400-sol month.
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![GroupingDef {
            name: s("year"),
            member_unit: Some(MemberUnitDef {
                name: s("month"),
                first_number: None,
            }),
            members: vec![MemberDef {
                name: Some(s("Only")),
                count: 400,
            }],
            first_number: None,
            rules: vec![],
        }];
        def.era.unit = s("year");
        let cal = Calendar::new(def).unwrap();
        assert_eq!(cal.units(), ["year", "month", "tick"]);
        assert_eq!(cal.units_per("year", "tick"), Ok(400));
        // A 1 ns fundamental unit, and one of a million years.
        assert!(Calendar::new(bare(secs(1, 1_000_000_000))).is_ok());
        assert!(Calendar::new(bare(secs(31_557_600_000_000, 1))).is_ok());
    }

    #[test]
    fn aster_is_accepted() {
        let cal = Calendar::new(aster()).unwrap();
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
    }
}
