//! Property tests over random calendars (US4; constitution VI; research R7). 10,000 cases per
//! property (SC-002).
//!
//! Random calendars whose era's unit is a grouping also get zero to three random leap rules
//! (spec 006, research R8), so the round trips cover calendars with rules too.
//!
//! Round trips can't catch a mistake made the same way in both directions, so two properties
//! compare the engine with independent oracles that share no code with it: a deliberately
//! simple reference that uses `num_rational::BigRational` (arbitrary precision, so it never
//! overflows) and steps through units one at a time from the epoch, for calendars without
//! rules; and a year-by-year reference that walks the era's unit from the epoch, evaluating
//! every rule for every year with `uni_expr` directly (no cycle table), for calendars with
//! rules.

use num_bigint::BigInt;
use num_rational::BigRational;
use proptest::prelude::*;
use uni_calendar::{
    Calendar, CalendarDef, Clock, Date, DefinitionError, EraDef, FundamentalDef, GroupingDef,
    InstantDate, LeapRuleDef, MemberDef, MemberUnitDef, SubdivisionDef,
};
use uni_expr::{Expr, Type, Value, VarDecl};
use uni_time::{Instant, Ratio};

const CASES: u32 = 10_000;
const NANOS_PER_SEC: i128 = 1_000_000_000;
/// ±100,000 years of 365.25 days, in nanoseconds (T015).
const RANGE_NS: i128 = 3_155_760_000_000 * NANOS_PER_SEC;
/// The longest chain: four groupings, each with a member unit, the fundamental unit and three
/// subdivisions.
const MAX_UNITS: usize = 12;

// ---------------------------------------------------------------------------------------------
// Random calendars

/// One grouping: its member counts, whether each member is named, and whether a lone member
/// still gets a member unit.
type GroupingShape = (Vec<(i64, bool)>, bool);

#[derive(Clone, Debug)]
struct Case {
    def: CalendarDef,
    /// The fundamental length in nanoseconds, `p / q`.
    p: i128,
    q: i128,
}

fn grouping_shape() -> impl Strategy<Value = GroupingShape> {
    (
        prop::collection::vec((1i64..=100, any::<bool>()), 1..=20),
        any::<bool>(),
    )
}

/// A whole number of nanoseconds at least `min`, biased towards short lengths.
fn whole_ns(min: i128) -> impl Strategy<Value = i128> {
    prop_oneof![
        min..=min + 1_000,
        min..=1_000_000_000,
        min..=100_000_000_000,
    ]
}

fn denominator() -> impl Strategy<Value = i128> {
    prop_oneof![Just(1i128), 1i128..=1_000, 1i128..=1_000_000_000]
}

/// One part of a random condition, `(a * year + b) % m == c`: `a`, `b`, `m` and a seed for
/// `c` (taken modulo `m`). Small moduli are likelier, so rule periods are usually short and
/// the year-by-year reference stays quick; any `m` up to 50 can still come up.
type Term = (i64, i64, i64, u64);

/// A random rule: its first term, an optional second joined by `&&` (true) or `||` (false),
/// a seed for its member, and its change before it is kept valid.
type RuleSeed = (Term, Option<(bool, Term)>, usize, i64);

fn term() -> impl Strategy<Value = Term> {
    (
        -3i64..=3,
        -100i64..=100,
        prop_oneof![3 => 1i64..=12, 1 => 1i64..=50],
        any::<u64>(),
    )
}

fn rule_seed() -> impl Strategy<Value = RuleSeed> {
    (
        term(),
        prop::option::of((any::<bool>(), term())),
        any::<usize>(),
        -2i64..=3,
    )
}

fn case() -> impl Strategy<Value = Case> {
    (
        prop::collection::vec(grouping_shape(), 1..=4),
        prop::collection::vec(1i64..=100, 0..=3),
    )
        .prop_flat_map(|(groupings, parts)| {
            let product: i128 = parts.iter().map(|&n| i128::from(n)).product();
            // At least 1,000 ns keeps ±100,000 years of fixed numbers inside i64, and at least
            // `product` ns keeps the smallest unit at least 1 ns long (research R3).
            (
                Just(groupings),
                Just(parts),
                whole_ns(product.max(1_000)),
                denominator(),
                any::<u128>(),
                any::<usize>(),
                prop::collection::vec(prop::option::of(-5i64..=5), MAX_UNITS),
                -1_000_000_000_000_000i64..=1_000_000_000_000_000,
                0u32..1_000_000_000,
                prop::collection::vec(rule_seed(), 0..=3),
            )
        })
        .prop_map(
            |(groupings, parts, whole, q, fraction, era, firsts, secs, nanos, rules)| {
                let fraction = i128::try_from(fraction % q.unsigned_abs()).unwrap();
                let mut case = build(
                    &groupings,
                    &parts,
                    whole * q + fraction,
                    q,
                    era,
                    &firsts,
                    secs,
                    nanos,
                );
                add_rules(&mut case.def, &rules);
                case
            },
        )
}

#[allow(clippy::too_many_arguments)]
fn build(
    groupings: &[GroupingShape],
    parts: &[i64],
    p_unreduced: i128,
    q_unreduced: i128,
    era: usize,
    firsts: &[Option<i64>],
    secs: i64,
    nanos: u32,
) -> Case {
    let length_ns = Ratio::new(p_unreduced, q_unreduced).unwrap();
    let mut names = Vec::new(); // chain order
    let mut grouping_defs = Vec::new();
    for (g, (members, lone_unit)) in groupings.iter().enumerate() {
        let name = format!("g{g}");
        names.push(name.clone());
        let member_unit = (members.len() > 1 || *lone_unit).then(|| {
            names.push(format!("g{g}m"));
            MemberUnitDef {
                name: format!("g{g}m"),
                first_number: None,
            }
        });
        grouping_defs.push(GroupingDef {
            name,
            member_unit,
            members: members
                .iter()
                .enumerate()
                .map(|(j, &(count, named))| MemberDef {
                    name: named.then(|| format!("m{j}")),
                    count,
                })
                .collect(),
            first_number: None,
            rules: vec![],
        });
    }
    names.push("f".to_string());
    let at_or_above = names.len();
    let mut def = CalendarDef {
        name: "Random".to_string(),
        clock: Clock::Gct,
        groupings: grouping_defs,
        fundamental: FundamentalDef {
            name: "f".to_string(),
            length: length_ns
                .checked_div(Ratio::from_int(NANOS_PER_SEC).unwrap())
                .unwrap(),
            first_number: None,
        },
        subdivisions: parts
            .iter()
            .enumerate()
            .map(|(s, &parts)| SubdivisionDef {
                name: format!("s{s}"),
                parts,
                first_number: None,
            })
            .collect(),
        era: EraDef {
            name: "Era".to_string(),
            unit: names[era % at_or_above].clone(),
            epoch: Instant::new(secs, nanos).unwrap(),
        },
    };
    // First numbers only below the era's unit.
    let era = era % at_or_above;
    let mut k = 0;
    for g in &mut def.groupings {
        if k > era {
            g.first_number = firsts[k];
        }
        k += 1;
        if let Some(m) = &mut g.member_unit {
            if k > era {
                m.first_number = firsts[k];
            }
            k += 1;
        }
    }
    if k > era {
        def.fundamental.first_number = firsts[k];
    }
    for (s, sub) in def.subdivisions.iter_mut().enumerate() {
        sub.first_number = firsts[k + 1 + s];
    }
    Case {
        def,
        p: length_ns.numer(),
        q: length_ns.denom(),
    }
}

/// Adds random rules to the era's unit when it is a grouping (research R8). Each change is
/// kept valid by construction: a shortening that could take its member below 1 in a year where
/// every shortening rule on it holds is turned into a lengthening. Rules whose combined period
/// passes the engine's limit are dropped from the end until the calendar is accepted.
fn add_rules(def: &mut CalendarDef, seeds: &[RuleSeed]) {
    let era = def.era.unit.clone();
    let Some(g) = def.groupings.iter_mut().find(|g| g.name == era) else {
        return;
    };
    let mut lowest: Vec<i64> = g.members.iter().map(|m| m.count).collect();
    let text = |&(a, b, m, c): &Term| {
        let c = i64::try_from(c % m.unsigned_abs()).unwrap();
        format!("({a} * {era} + {b}) % {m} == {c}")
    };
    for (first, second, member, change) in seeds {
        let j = member % g.members.len();
        let mut change = if *change == 0 { 1 } else { *change };
        if change < 0 && lowest[j] + change < 1 {
            change = -change;
        }
        if change < 0 {
            lowest[j] += change;
        }
        let condition = match second {
            None => text(first),
            Some((and, second)) => {
                let join = if *and { "&&" } else { "||" };
                format!("{} {join} {}", text(first), text(second))
            }
        };
        g.rules.push(LeapRuleDef {
            condition,
            member: g.members[j]
                .name
                .clone()
                .unwrap_or_else(|| (j + 1).to_string()),
            change,
        });
    }
    while let Err(DefinitionError::RulesPeriodOverLimit { .. }) = Calendar::new(def.clone()) {
        let g = def.groupings.iter_mut().find(|g| g.name == era).unwrap();
        g.rules.pop();
    }
}

/// The same calendar without its rules.
fn without_rules(def: &CalendarDef) -> CalendarDef {
    let mut def = def.clone();
    for g in &mut def.groupings {
        g.rules.clear();
    }
    def
}

fn offset_ns() -> impl Strategy<Value = i128> {
    prop_oneof![
        -RANGE_NS..=RANGE_NS,
        -1_000_000_000_000_000i128..=1_000_000_000_000_000,
        -1_000i128..=1_000,
    ]
}

fn at(case: &Case, offset: i128) -> Instant {
    Instant::from_nanos(case.def.era.epoch.as_nanos() + offset).unwrap()
}

fn engine(case: &Case) -> Calendar {
    Calendar::new(case.def.clone()).expect("the strategy builds valid calendars")
}

// ---------------------------------------------------------------------------------------------
// Exact arithmetic helpers, all BigRational

fn big(n: i128) -> BigRational {
    BigRational::from_integer(BigInt::from(n))
}

fn ratio(r: Ratio) -> BigRational {
    BigRational::new(BigInt::from(r.numer()), BigInt::from(r.denom()))
}

fn ns_of(t: Instant) -> BigRational {
    big(t.as_nanos())
}

/// The smallest unit's length in seconds.
fn smallest_secs(case: &Case) -> BigRational {
    let product: i128 = case
        .def
        .subdivisions
        .iter()
        .map(|s| i128::from(s.parts))
        .product();
    big(case.p) / big(case.q) / big(product) / big(NANOS_PER_SEC)
}

// ---------------------------------------------------------------------------------------------
// The test-side model of the chain, used to generate valid dates and by the oracle

#[derive(Clone, Debug)]
enum Node {
    /// A grouping: its member counts (of the unit below it) and whether it has a member unit.
    Grouping {
        counts: Vec<i64>,
        has_unit: bool,
    },
    /// A grouping's member unit, holding `counts[j]` of the unit below the grouping.
    Member {
        counts: Vec<i64>,
    },
    Fundamental,
    Subdivision {
        parts: i64,
    },
}

struct Model {
    nodes: Vec<Node>,
    firsts: Vec<i64>,
    /// Length in ns of each node that has a fixed length (groupings, fundamental, subdivisions).
    lengths: Vec<Option<BigRational>>,
    /// Length in ns of the unit directly below each grouping (for groupings and member units).
    below: Vec<Option<BigRational>>,
    era: usize,
}

impl Model {
    fn new(def: &CalendarDef) -> Model {
        let mut nodes = Vec::new();
        let mut firsts = Vec::new();
        for g in &def.groupings {
            let counts: Vec<i64> = g.members.iter().map(|m| m.count).collect();
            nodes.push(Node::Grouping {
                counts: counts.clone(),
                has_unit: g.member_unit.is_some(),
            });
            firsts.push(g.first_number.unwrap_or(1));
            if let Some(m) = &g.member_unit {
                nodes.push(Node::Member { counts });
                firsts.push(m.first_number.unwrap_or(1));
            }
        }
        nodes.push(Node::Fundamental);
        firsts.push(def.fundamental.first_number.unwrap_or(1));
        for s in &def.subdivisions {
            nodes.push(Node::Subdivision { parts: s.parts });
            firsts.push(s.first_number.unwrap_or(0));
        }
        let fundamental = nodes
            .iter()
            .position(|n| matches!(n, Node::Fundamental))
            .unwrap();
        let mut lengths = vec![None; nodes.len()];
        let mut below = vec![None; nodes.len()];
        let fund_ns = ratio(def.fundamental.length) * big(NANOS_PER_SEC);
        lengths[fundamental] = Some(fund_ns.clone());
        let mut length = fund_ns.clone();
        for k in fundamental + 1..nodes.len() {
            if let Node::Subdivision { parts } = nodes[k] {
                length /= big(i128::from(parts));
                lengths[k] = Some(length.clone());
            }
        }
        // Bottom up through the groupings.
        let mut under = fund_ns;
        for k in (0..fundamental).rev() {
            match &nodes[k] {
                Node::Member { .. } => below[k] = Some(under.clone()),
                Node::Grouping { counts, .. } => {
                    below[k] = Some(under.clone());
                    let total: i128 = counts.iter().map(|&c| i128::from(c)).sum();
                    under *= big(total);
                    lengths[k] = Some(under.clone());
                }
                _ => unreachable!(),
            }
        }
        let era = def_names(def)
            .iter()
            .position(|n| *n == def.era.unit)
            .unwrap();
        Model {
            nodes,
            firsts,
            lengths,
            below,
            era,
        }
    }

    /// The lengths (ns) of the children of node `k`'s instance, where `index` is that
    /// instance's position within its own parent (it picks the member for a member unit).
    fn children(&self, k: usize, index: usize) -> Vec<BigRational> {
        match &self.nodes[k] {
            Node::Grouping { counts, has_unit } => {
                let under = self.below[k].clone().unwrap();
                if *has_unit {
                    counts
                        .iter()
                        .map(|&c| &under * big(i128::from(c)))
                        .collect()
                } else {
                    vec![under; usize::try_from(counts[0]).unwrap()]
                }
            }
            Node::Member { counts, .. } => {
                vec![self.below[k].clone().unwrap(); usize::try_from(counts[index]).unwrap()]
            }
            Node::Fundamental | Node::Subdivision { .. } => match self.nodes.get(k + 1) {
                Some(Node::Subdivision { parts }) => {
                    vec![self.lengths[k + 1].clone().unwrap(); usize::try_from(*parts).unwrap()]
                }
                _ => vec![],
            },
        }
    }
}

fn def_names(def: &CalendarDef) -> Vec<String> {
    let mut names = Vec::new();
    for g in &def.groupings {
        names.push(g.name.clone());
        if let Some(m) = &g.member_unit {
            names.push(m.name.clone());
        }
    }
    names.push(def.fundamental.name.clone());
    names.extend(def.subdivisions.iter().map(|s| s.name.clone()));
    names
}

/// A valid date: `era_value` for the era's unit, then a value in range for each unit below it,
/// picked by `seeds`, truncated to `len` values. Also returns the full date with the missing
/// units at their first values.
fn valid_date(model: &Model, era_value: i64, seeds: &[u64], len: usize) -> (Date, Vec<i64>) {
    let mut values = vec![era_value];
    // The era's unit's position within its parent, for the member it picks: by count.
    let mut index = match &model.nodes[model.era] {
        Node::Member { counts, .. } => {
            usize::try_from((era_value - 1).rem_euclid(i64::try_from(counts.len()).unwrap()))
                .unwrap()
        }
        _ => 0,
    };
    for (k, seed) in seeds
        .iter()
        .enumerate()
        .take(model.nodes.len())
        .skip(model.era + 1)
    {
        let count = model.children(k - 1, index).len() as u64;
        let chosen = seed % count;
        index = usize::try_from(chosen).unwrap();
        values.push(model.firsts[k] + i64::try_from(chosen).unwrap());
    }
    let len = 1 + len % values.len();
    let mut full = values[..len].to_vec();
    // Missing units take their first value: position 0 everywhere below.
    full.extend((model.era + len..model.nodes.len()).map(|k| model.firsts[k]));
    (
        Date {
            values: values[..len].to_vec(),
        },
        full,
    )
}

// ---------------------------------------------------------------------------------------------
// The year-by-year reference (research R8)

/// The era's unit's rules, evaluated directly for each year with `uni_expr`, never through a
/// cycle.
struct Reference {
    /// Each rule: its condition, member index and change.
    rules: Vec<(Expr, usize, i64)>,
    /// The members' listed counts.
    base: Vec<i64>,
    /// The length of the unit the members count, in fundamental units.
    below: i128,
}

impl Reference {
    /// `None` when the era's unit isn't a grouping.
    fn new(def: &CalendarDef) -> Option<Reference> {
        let at = def.groupings.iter().position(|g| g.name == def.era.unit)?;
        let g = &def.groupings[at];
        let vars = [VarDecl {
            name: g.name.clone(),
            ty: Type::Int,
        }];
        let label = |j: usize| {
            g.members[j]
                .name
                .clone()
                .unwrap_or_else(|| (j + 1).to_string())
        };
        let rules = g
            .rules
            .iter()
            .map(|rule| {
                let expr = Expr::check(&rule.condition, &vars, Type::Bool).unwrap();
                let j = (0..g.members.len())
                    .find(|&j| label(j) == rule.member)
                    .unwrap();
                (expr, j, rule.change)
            })
            .collect();
        let below = def.groupings[at + 1..]
            .iter()
            .map(|g| g.members.iter().map(|m| i128::from(m.count)).sum::<i128>())
            .product();
        Some(Reference {
            rules,
            base: g.members.iter().map(|m| m.count).collect(),
            below,
        })
    }

    /// The members' counts in year `year`.
    fn counts(&self, year: i64) -> Vec<i64> {
        let mut counts = self.base.clone();
        for (expr, j, change) in &self.rules {
            if expr.eval(&[Value::Int(year)]) == Ok(Value::Bool(true)) {
                counts[*j] += change;
            }
        }
        counts
    }

    /// Year `year`'s length in fundamental units.
    fn length(&self, year: i64) -> i128 {
        self.counts(year)
            .iter()
            .map(|&c| i128::from(c))
            .sum::<i128>()
            * self.below
    }

    /// Where year `year` starts, in fundamental units from the epoch, by adding up every
    /// year between it and the epoch.
    fn start(&self, year: i64) -> i128 {
        if year >= 1 {
            (1..year).map(|y| self.length(y)).sum()
        } else {
            -(year..=0).map(|y| self.length(y)).sum::<i128>()
        }
    }
}

/// The calendar as it is in year `year`: the era's unit's members at that year's counts, and
/// no rules, so the test-side model can describe the units below the year.
fn year_def(def: &CalendarDef, year: i64) -> CalendarDef {
    let mut out = without_rules(def);
    if let Some(reference) = Reference::new(def) {
        let g = out
            .groupings
            .iter_mut()
            .find(|g| g.name == def.era.unit)
            .unwrap();
        for (m, count) in g.members.iter_mut().zip(reference.counts(year)) {
            m.count = count;
        }
    }
    out
}

/// The date values, from the era's unit down, of the fundamental unit `offset` units into
/// year `year`, worked out from that year's model alone.
fn reference_date(def: &CalendarDef, year: i64, offset: i128) -> Vec<i64> {
    let model = Model::new(&year_def(def, year));
    let fundamental = model
        .nodes
        .iter()
        .position(|n| matches!(n, Node::Fundamental))
        .unwrap();
    let mut rem = big(offset) * model.lengths[fundamental].clone().unwrap();
    let mut values = vec![year];
    let mut index = 0usize;
    for k in model.era + 1..model.nodes.len() {
        if k > fundamental {
            values.push(model.firsts[k]); // a fundamental unit's start is in its first parts
            continue;
        }
        let children = model.children(k - 1, index);
        let mut pos = 0usize;
        while rem >= children[pos] {
            rem -= &children[pos];
            pos += 1;
        }
        index = pos;
        values.push(model.firsts[k] + i64::try_from(pos).unwrap());
    }
    values
}

// ---------------------------------------------------------------------------------------------
// The oracle

struct OracleDate {
    above: Vec<BigInt>,
    date: Vec<BigInt>,
    remainder: BigRational, // seconds
}

/// Finds the date of `instant` by stepping through units one at a time from the epoch.
fn oracle(def: &CalendarDef, instant: Instant) -> OracleDate {
    let model = Model::new(def);
    let offset = ns_of(instant) - ns_of(def.era.epoch);
    let zero = big(0);

    // The top unit: step whole instances from the epoch.
    let top = model.lengths[0].clone().unwrap();
    let mut count = BigInt::from(0);
    let mut start = zero.clone();
    if offset >= zero {
        while &start + &top <= offset {
            start += &top;
            count += 1;
        }
    } else {
        while start > offset {
            start -= &top;
            count -= 1;
        }
    }
    let mut rem = &offset - &start;
    let mut positions: Vec<BigInt> = vec![count.clone()];
    let mut counts: Vec<BigInt> = vec![count];
    let mut index = 0usize;

    for k in 1..model.nodes.len() {
        let children = model.children(k - 1, index);
        let mut pos = 0usize;
        while rem >= children[pos] {
            rem -= &children[pos];
            pos += 1;
        }
        index = pos;
        positions.push(BigInt::from(pos));
        // Count from the epoch, needed at or above the era's unit only.
        let from_epoch = match &model.nodes[k] {
            Node::Member {
                counts: members, ..
            } => &counts[k - 1] * BigInt::from(members.len()) + BigInt::from(pos),
            _ => match &model.lengths[k] {
                Some(length) => {
                    let instance_start = &offset - &rem;
                    let n = instance_start / length;
                    assert!(n.is_integer(), "fixed-length units tile from the epoch");
                    n.to_integer()
                }
                None => BigInt::from(0),
            },
        };
        counts.push(from_epoch);
    }

    let numbers: Vec<BigInt> = (0..model.nodes.len())
        .map(|k| {
            if k <= model.era {
                &counts[k] + 1
            } else {
                &positions[k] + model.firsts[k]
            }
        })
        .collect();
    OracleDate {
        above: numbers[..model.era].to_vec(),
        date: numbers[model.era..].to_vec(),
        remainder: rem / big(NANOS_PER_SEC),
    }
}

fn top_length_ns(case: &Case) -> i128 {
    let mut fundamentals: i128 = 1;
    for g in case.def.groupings.iter().rev() {
        fundamentals *= g.members.iter().map(|m| i128::from(m.count)).sum::<i128>();
    }
    fundamentals * case.p / case.q
}

// ---------------------------------------------------------------------------------------------
// Properties

fn key(d: &InstantDate) -> (Vec<i64>, Ratio) {
    let mut numbers = d.above.clone();
    numbers.extend(&d.date.values);
    (numbers, d.remainder)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// US4-1: instant → date → start of date + remainder = the instant, exactly.
    #[test]
    fn instant_to_date_and_back(case in case(), offset in offset_ns()) {
        let cal = engine(&case);
        let t = at(&case, offset);
        let got = cal.instant_to_date(t).unwrap();
        prop_assert!(ratio(got.remainder) >= big(0));
        prop_assert!(ratio(got.remainder) < smallest_secs(&case));
        let start = cal.date_to_instant(&got.date).unwrap();
        let exact_start = ns_of(start.instant)
            - start.rounded_by.map_or(big(0), |r| ratio(r) * big(NANOS_PER_SEC));
        prop_assert_eq!(exact_start + ratio(got.remainder) * big(NANOS_PER_SEC), ns_of(t));
    }

    /// US4-2 and US4-3: date → instant → the same date; a rounded start is less than 1 ns
    /// after the exact start and still in the same date.
    #[test]
    fn date_to_instant_and_back(
        case in case(),
        era_value in -1_000i64..=1_000,
        seeds in prop::collection::vec(any::<u64>(), MAX_UNITS),
        len in any::<usize>(),
    ) {
        let cal = engine(&case);
        let model = Model::new(&year_def(&case.def, era_value));
        let (date, full) = valid_date(&model, era_value, &seeds, len);
        let start = cal.date_to_instant(&date).unwrap();
        let back = cal.instant_to_date(start.instant).unwrap();
        prop_assert_eq!(&back.date.values, &full);
        match start.rounded_by {
            None => prop_assert_eq!(back.remainder, Ratio::ZERO),
            Some(r) => {
                prop_assert!(ratio(r) > big(0));
                prop_assert!(ratio(r) < big(1) / big(NANOS_PER_SEC));
                prop_assert_eq!(back.remainder, r);
            }
        }
    }

    /// FR-012: fixed number → start → the same fixed number.
    #[test]
    fn fixed_numbers_round_trip(case in case(), fixed in -1_000_000_000_000_000i64..=1_000_000_000_000_000) {
        let cal = engine(&case);
        let start = cal.start_of_fixed(fixed).unwrap();
        prop_assert_eq!(cal.fixed_from_instant(start.instant), Ok(fixed));
    }

    /// A later instant never gives an earlier date.
    #[test]
    fn dates_keep_their_order(case in case(), a in offset_ns(), b in offset_ns()) {
        let cal = engine(&case);
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        let da = cal.instant_to_date(at(&case, a)).unwrap();
        let db = cal.instant_to_date(at(&case, b)).unwrap();
        prop_assert!(key(&da) <= key(&db), "{:?} then {:?}", da, db);
    }

    /// T016: the engine agrees with the stepping oracle within ±50 top-level units, for
    /// calendars without rules (the oracle's units all have fixed lengths).
    #[test]
    fn engine_matches_the_oracle(case in case(), fraction in -50_000_000i128..=50_000_000) {
        let def = without_rules(&case.def);
        // An offset of fraction / 10⁶ top-level units, to the nearest nanosecond below.
        let offset = top_length_ns(&case) * fraction / 1_000_000;
        let t = at(&case, offset);
        let got = Calendar::new(def.clone()).unwrap().instant_to_date(t).unwrap();
        let want = oracle(&def, t);
        let to_big = |v: &[i64]| v.iter().map(|&n| BigInt::from(n)).collect::<Vec<_>>();
        prop_assert_eq!(to_big(&got.above), want.above);
        prop_assert_eq!(to_big(&got.date.values), want.date);
        prop_assert_eq!(ratio(got.remainder), want.remainder);
    }

    /// Spec 006 Story 3 scenario 3: with random rules, years agree with the year-by-year
    /// reference within ±(2 × rule period + 50) years of the epoch: where each starts, its
    /// members' lengths, and the date of a fundamental unit inside it.
    #[test]
    fn engine_matches_the_year_by_year_reference(
        case in case(),
        near in -50i64..=50,
        far in -1_000_000i64..=1_000_000,
        pick_far in any::<bool>(),
        seed in any::<u64>(),
    ) {
        let Some(reference) = Reference::new(&case.def) else {
            return Ok(()); // the era's unit isn't a grouping: no rules to check
        };
        let cal = engine(&case);
        let span = 2 * cal.rule_period() + 50;
        let year = 1 + if pick_far { far * span / 1_000_000 } else { near };
        let start = reference.start(year);
        let start_fixed = i64::try_from(start).unwrap();
        // Where the year starts.
        let got = cal.date_to_instant(&Date { values: vec![year] }).unwrap();
        prop_assert_eq!(cal.fixed_from_instant(got.instant), Ok(start_fixed));
        // Its members' lengths.
        prop_assert_eq!(cal.era_instance(year).unwrap().members, reference.counts(year));
        // The date of one fundamental unit inside it.
        let offset = i128::from(seed) % reference.length(year);
        let fixed = i64::try_from(start + offset).unwrap();
        let got = cal.instant_to_date(cal.start_of_fixed(fixed).unwrap().instant).unwrap();
        prop_assert_eq!(got.date.values, reference_date(&case.def, year, offset));
    }
}
