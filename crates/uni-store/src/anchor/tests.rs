//! Tests for anchors (spec 008): storing them, placing calendars, checking an anchor on keep,
//! loops, deleting calendars, and a property test of random chains (SC-002).

#![allow(clippy::result_large_err)] // placements carry the engine's errors

use rusqlite::Connection;
use uni_calendar::{Calendar, CalendarError, Date, GroupingDef, MemberDef};
use uni_time::{Instant, Ratio};

use super::*;
use crate::calendar::tests::{aster, random_record, small, universe};
use crate::{EraRecord, FundamentalRecord, RecordError};

// ---- Helpers --------------------------------------------------------------------------------

fn one_member(name: &str, count: i64) -> GroupingDef {
    GroupingDef {
        name: name.to_owned(),
        member_unit: None,
        members: vec![MemberDef { name: None, count }],
        first_number: None,
        rules: vec![],
    }
}

/// The spec's Tallow: a cycle of 52 rounds, a round of 7 watches, a watch of 30,000 s.
fn tallow() -> CalendarRecord {
    CalendarRecord {
        name: "Tallow".to_owned(),
        groupings: vec![one_member("cycle", 52), one_member("round", 7)],
        fundamental: FundamentalRecord {
            name: "watch".to_owned(),
            length: "30000".to_owned(),
            first_number: None,
        },
        subdivisions: vec![],
        era: EraRecord {
            name: "Count".to_owned(),
            unit: "cycle".to_owned(),
        },
        anchor: None,
    }
}

fn anchor(kind: AnchorKind, base: i64, date: &[i64], base_date: &[i64]) -> Anchor {
    Anchor {
        kind,
        base: Some(base),
        base_name: None,
        date: date.to_vec(),
        base_date: base_date.to_vec(),
        at: None,
    }
}

fn with(record: CalendarRecord, anchor: Anchor) -> CalendarRecord {
    CalendarRecord {
        anchor: Some(anchor),
        ..record
    }
}

use AnchorKind::{Equation, Fixed};

/// Keeps `record` as the editor does: checks its anchor, stores the `at` it gives, then adds
/// or replaces it.
fn keep(
    universe: &mut OpenUniverse,
    id: Option<i64>,
    mut record: CalendarRecord,
) -> Result<i64, AnchorError> {
    let at = universe.check_anchor(id, &record)?;
    if let Some(a) = &mut record.anchor {
        a.at = at;
    }
    Ok(match id {
        Some(id) => {
            universe.replace_calendar(id, &record).unwrap();
            id
        }
        None => universe.add_calendar(&record).unwrap(),
    })
}

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

fn placed(universe: &OpenUniverse, id: i64) -> Result<Calendar, PlacementProblem> {
    universe
        .placements()
        .unwrap()
        .into_iter()
        .find(|p| p.stored.id == id)
        .unwrap()
        .placed
}

/// Where `values` starts in calendar `id`, as placed.
fn start(universe: &OpenUniverse, id: i64, values: &[i64]) -> Instant {
    placed(universe, id)
        .unwrap()
        .date_to_instant(&date(values))
        .unwrap()
        .instant
}

fn problem(universe: &OpenUniverse, id: i64) -> PlacementProblem {
    placed(universe, id).unwrap_err()
}

fn stored_anchor(universe: &OpenUniverse, id: i64) -> Option<Anchor> {
    universe.calendar(id).unwrap().record.anchor
}

fn damaged() -> StoreError {
    StoreError::NotAUniverseFile {
        reason: "it's damaged".to_owned(),
    }
}

/// Aster (the reference calendar) and Tallow, fixed with its `[1, 1, 1]` at Aster's
/// `[412, 3, 9]`.
fn aster_and_tallow(kind: AnchorKind) -> (OpenUniverse, i64, i64) {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let t = keep(
        &mut u,
        None,
        with(tallow(), anchor(kind, a, &[1, 1, 1], &[412, 3, 9])),
    )
    .unwrap();
    (u, a, t)
}

/// Aster with its span `name` lengthened to `count` sols.
fn aster_with_span(member: usize, count: i64) -> CalendarRecord {
    let mut record = aster();
    record.groupings[2].members[member].count = count;
    record
}

// ---- Storing anchors (T006) -----------------------------------------------------------------

#[test]
fn anchors_of_both_kinds_round_trip_field_for_field() {
    let mut u = universe();
    let a = u.add_calendar(&aster()).unwrap();
    let mut fixed = anchor(Fixed, a, &[1, 1, 1], &[412, 3, 9]);
    fixed.at = Some(Instant::new(-12_345, 678).unwrap());
    let t = u.add_calendar(&with(tallow(), fixed.clone())).unwrap();
    assert_eq!(stored_anchor(&u, t), Some(fixed));
    let equation = anchor(Equation, a, &[-3], &[0, 12, 55, 19, 99]);
    u.replace_calendar(t, &with(tallow(), equation.clone()))
        .unwrap();
    assert_eq!(stored_anchor(&u, t), Some(equation));
    assert_eq!(stored_anchor(&u, a), None);
}

#[test]
fn base_name_round_trips_and_survives_a_replace_without_it() {
    let mut u = universe();
    let a = u.add_calendar(&aster()).unwrap();
    let mut gone = anchor(Fixed, a, &[1], &[1]);
    gone.base = None;
    gone.base_name = Some("Old Aster".to_owned());
    gone.at = Some(Instant::ZERO);
    let t = u.add_calendar(&with(tallow(), gone.clone())).unwrap();
    assert_eq!(stored_anchor(&u, t), Some(gone.clone()));
    // The editor doesn't send base_name back.
    let mut sent = gone.clone();
    sent.base_name = None;
    u.replace_calendar(t, &with(tallow(), sent)).unwrap();
    assert_eq!(stored_anchor(&u, t), Some(gone));
}

#[test]
fn dates_are_stored_as_compact_arrays() {
    let mut u = universe();
    let a = u.add_calendar(&aster()).unwrap();
    let t = u
        .add_calendar(&with(
            tallow(),
            anchor(Equation, a, &[1, -1, 1], &[412, 3, 9]),
        ))
        .unwrap();
    let (own, base): (String, String) =
        u.db.query_row(
            "SELECT date, base_date FROM anchor WHERE calendar_id = ?1",
            [t],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((own.as_str(), base.as_str()), ("[1,-1,1]", "[412,3,9]"));
}

#[test]
fn date_text_is_read_strictly() {
    assert_eq!(parse_date_text("[412,3,9]"), Some(vec![412, 3, 9]));
    assert_eq!(parse_date_text("[-5]"), Some(vec![-5]));
    for text in [
        "[]", "[1,]", "[ 1]", "1,2", "[1.5]", "[1", "1]", "[,1]", "[-]", "[1e3]", "[+1]",
    ] {
        assert_eq!(parse_date_text(text), None, "{text}");
    }
}

#[test]
fn stored_dates_in_the_wrong_shape_read_as_damaged() {
    for bad in ["[]", "[1,]", "[ 1]", "1,2", "[1.5]"] {
        for column in ["date", "base_date"] {
            let mut u = universe();
            let a = u.add_calendar(&aster()).unwrap();
            let t = u
                .add_calendar(&with(tallow(), anchor(Equation, a, &[1], &[1])))
                .unwrap();
            u.db.execute(&format!("UPDATE anchor SET {column} = ?1"), [bad])
                .unwrap();
            assert_eq!(u.calendar(t), Err(damaged()), "{column} {bad}");
            assert_eq!(u.calendars(), Err(damaged()), "{column} {bad}");
        }
    }
}

#[test]
fn two_calendars_without_an_anchor_read_as_damaged() {
    let mut u = universe();
    let a = u.add_calendar(&aster()).unwrap();
    u.add_calendar(&tallow()).unwrap();
    assert_eq!(u.calendars(), Err(damaged()));
    assert_eq!(u.calendar(a), Err(damaged()));
}

// ---- Placing calendars (T007, T014) ----------------------------------------------------------

#[test]
fn aster_alone_is_at_the_hidden_zero() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    assert_eq!(placed(&u, a).unwrap().epoch(), Instant::ZERO);
}

#[test]
fn tallow_starts_at_its_base_date() {
    for kind in [Fixed, Equation] {
        let (u, a, t) = aster_and_tallow(kind);
        assert_eq!(
            start(&u, t, &[1, 1, 1]),
            start(&u, a, &[412, 3, 9]),
            "{kind:?}"
        );
    }
}

#[test]
fn a_third_calendar_is_placed_through_tallow() {
    for kind in [Fixed, Equation] {
        let (mut u, _, t) = aster_and_tallow(Equation);
        let w = keep(
            &mut u,
            None,
            with(small("Wick"), anchor(kind, t, &[7], &[2, 5, 3])),
        )
        .unwrap();
        assert_eq!(start(&u, w, &[7]), start(&u, t, &[2, 5, 3]), "{kind:?}");
    }
}

#[test]
fn an_equation_chain_places_in_any_list_order() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Fixed, a, &[1], &[1])),
    )
    .unwrap();
    let t = keep(
        &mut u,
        None,
        with(tallow(), anchor(Equation, a, &[1, 1, 1], &[412, 3, 9])),
    )
    .unwrap();
    // Wick, earlier in the list, now follows Tallow, which comes after it.
    keep(
        &mut u,
        Some(w),
        with(small("Wick"), anchor(Equation, t, &[10], &[3])),
    )
    .unwrap();
    assert_eq!(start(&u, w, &[10]), start(&u, t, &[3]));
    assert_eq!(start(&u, t, &[1, 1, 1]), start(&u, a, &[412, 3, 9]));
}

#[test]
fn refused_definitions_are_problems_and_their_followers_unusable() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let mut bad_length = small("Bad length");
    bad_length.fundamental.length = "about a thousand".to_owned();
    let l = u
        .add_calendar(&with(bad_length, anchor(Fixed, a, &[1], &[1])))
        .unwrap();
    let mut refused = tallow();
    refused.groupings[0].members[0].count = 0;
    let r = u
        .add_calendar(&with(refused, anchor(Equation, a, &[1], &[1])))
        .unwrap();
    let f = u
        .add_calendar(&with(small("Follower"), anchor(Equation, r, &[1], &[1])))
        .unwrap();
    assert!(matches!(
        problem(&u, l),
        PlacementProblem::Length(RecordError::Length { .. })
    ));
    assert!(matches!(problem(&u, r), PlacementProblem::Definition(_)));
    assert_eq!(
        problem(&u, f),
        PlacementProblem::BaseUnusable {
            name: "Tallow".to_owned()
        }
    );
    assert_eq!(
        problem(&u, f).to_string(),
        "it's anchored to Tallow, which can't be placed"
    );
}

#[test]
fn a_fixed_anchor_stays_put_when_its_base_is_edited() {
    let (mut u, a, t) = aster_and_tallow(Fixed);
    let before = start(&u, t, &[1, 1, 1]);
    keep(&mut u, Some(a), aster_with_span(0, 57)).unwrap();
    assert_eq!(start(&u, t, &[1, 1, 1]), before, "FR-005");
    assert_ne!(
        start(&u, a, &[412, 3, 9]),
        before,
        "Aster has moved under it"
    );
}

#[test]
fn an_equation_follows_its_base_when_it_is_edited() {
    let (mut u, a, t) = aster_and_tallow(Equation);
    let before = start(&u, t, &[1, 1, 1]);
    keep(&mut u, Some(a), aster_with_span(1, 57)).unwrap();
    let now = start(&u, a, &[412, 3, 9]);
    assert_ne!(now, before);
    assert_eq!(start(&u, t, &[1, 1, 1]), now, "Story 3, scenario 3");
}

#[test]
fn an_equation_whose_base_date_is_gone_is_unusable_and_so_are_its_followers() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let t = keep(
        &mut u,
        None,
        with(tallow(), anchor(Equation, a, &[1], &[412, 2, 56])),
    )
    .unwrap();
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Equation, t, &[1], &[1])),
    )
    .unwrap();
    keep(&mut u, Some(a), aster_with_span(1, 55)).unwrap();
    let PlacementProblem::BaseDate { name, error } = problem(&u, t) else {
        panic!("expected BaseDate")
    };
    assert_eq!(name, "Aster");
    assert!(matches!(error, CalendarError::InvalidValue { .. }));
    assert_eq!(
        problem(&u, t).to_string(),
        "its anchoring date in Aster: sol 56 is outside 1 to 55 in span Second"
            .replace(" in span Second", &format!(" {}", in_second(&error)))
    );
    assert_eq!(
        problem(&u, w),
        PlacementProblem::BaseUnusable {
            name: "Tallow".to_owned()
        }
    );
}

/// The engine's own wording for where the value was checked, so this test doesn't depend on it.
fn in_second(error: &CalendarError) -> String {
    let text = error.to_string();
    text.strip_prefix("sol 56 is outside 1 to 55")
        .unwrap_or(&text)
        .trim()
        .to_owned()
}

/// Rewrites anchors directly, as a file written by hand could have them.
fn set_raw_anchor(u: &OpenUniverse, id: i64, kind: &str, base: i64) {
    u.db.execute(
        "UPDATE anchor SET kind = ?1, base_calendar_id = ?2, at_secs = NULL, at_nanos = NULL
         WHERE calendar_id = ?3",
        params![kind, base, id],
    )
    .unwrap();
}

#[test]
fn a_loop_from_a_file_gives_every_calendar_on_it_the_loop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("loop.universe");
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let t = keep(
        &mut u,
        None,
        with(tallow(), anchor(Equation, a, &[1], &[1])),
    )
    .unwrap();
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Equation, t, &[1], &[1])),
    )
    .unwrap();
    let m = keep(
        &mut u,
        None,
        with(small("Moss"), anchor(Equation, w, &[1], &[1])),
    )
    .unwrap();
    let f = keep(
        &mut u,
        None,
        with(small("Fern"), anchor(Equation, m, &[1], &[1])),
    )
    .unwrap();
    set_raw_anchor(&u, t, "equation", m);
    let expected = PlacementProblem::Loop {
        // Each follows the next: Tallow follows Moss, which follows Wick, which follows Tallow.
        names: vec!["Tallow".to_owned(), "Moss".to_owned(), "Wick".to_owned()],
    };
    for id in [t, w, m] {
        assert_eq!(problem(&u, id), expected);
    }
    assert_eq!(
        expected.to_string(),
        "Tallow, Moss and Wick follow each other in a loop"
    );
    assert_eq!(
        problem(&u, f),
        PlacementProblem::BaseUnusable {
            name: "Moss".to_owned()
        }
    );
    assert!(placed(&u, a).is_ok());
    // FR-015: the file still opens.
    u.save_as(&path).unwrap();
    drop(u);
    let reopened = OpenUniverse::open(&path, "0.1.0").unwrap();
    assert_eq!(problem(&reopened, w), expected);
}

#[test]
fn a_two_calendar_loop_reads_as_following_each_other() {
    let names = vec!["Tallow".to_owned(), "Wick".to_owned()];
    assert_eq!(
        PlacementProblem::Loop { names }.to_string(),
        "Tallow and Wick follow each other in a loop"
    );
}

#[test]
fn problem_messages() {
    assert_eq!(
        PlacementProblem::NoMoment.to_string(),
        "its anchor has no moment: anchor it again"
    );
    assert_eq!(
        PlacementProblem::OutOfRange.to_string(),
        "its anchor puts it beyond the timeline's range"
    );
    let error = CalendarError::OutOfRange;
    assert_eq!(
        PlacementProblem::Date {
            error: error.clone()
        }
        .to_string(),
        format!("its anchoring date: {error}")
    );
}

// ---- Checking an anchor on keep (T008, T019) -------------------------------------------------

#[test]
fn a_calendar_in_an_empty_universe_needs_no_anchor_and_later_ones_do() {
    let mut u = universe();
    assert_eq!(u.check_anchor(None, &aster()), Ok(None));
    let a = keep(&mut u, None, aster()).unwrap();
    assert_eq!(u.check_anchor(None, &tallow()), Err(AnchorError::Missing));
    // The reference calendar may stay without one.
    assert_eq!(u.check_anchor(Some(a), &aster()), Ok(None));
}

#[test]
fn tallows_fixed_anchor_gives_the_instant_of_its_base_date() {
    let (u, a, _) = aster_and_tallow(Fixed);
    let record = with(tallow(), anchor(Fixed, a, &[1, 1, 1], &[412, 3, 9]));
    assert_eq!(
        u.check_anchor(
            None,
            &CalendarRecord {
                name: "Tallow 2".to_owned(),
                ..record
            }
        ),
        Ok(Some(start(&u, a, &[412, 3, 9])))
    );
    let equation = with(small("Wick"), anchor(Equation, a, &[1], &[412]));
    assert_eq!(u.check_anchor(None, &equation), Ok(None));
}

#[test]
fn the_reference_calendar_can_be_anchored_to_a_later_fixed_calendar() {
    let (mut u, a, t) = aster_and_tallow(Fixed);
    let tallow_before = start(&u, t, &[1, 1, 1]);
    let record = with(aster(), anchor(Fixed, t, &[1], &[3, 1, 1]));
    keep(&mut u, Some(a), record).unwrap();
    assert_eq!(start(&u, a, &[1]), start(&u, t, &[3, 1, 1]));
    assert_eq!(start(&u, t, &[1, 1, 1]), tallow_before, "Tallow stays put");
    // No calendar is without an anchor now, and that reads back fine.
    assert!(
        u.calendars()
            .unwrap()
            .iter()
            .all(|c| c.record.anchor.is_some())
    );
}

#[test]
fn anchors_are_refused_with_their_reasons() {
    let (u, a, t) = aster_and_tallow(Fixed);
    let check = |id, anchor: Anchor| u.check_anchor(id, &with(tallow(), anchor));
    // No base, or one that doesn't exist.
    let mut none = anchor(Fixed, a, &[1], &[1]);
    none.base = None;
    assert_eq!(check(None, none), Err(AnchorError::BaseMissing));
    assert_eq!(
        check(None, anchor(Fixed, 999, &[1], &[1])),
        Err(AnchorError::BaseMissing)
    );
    assert_eq!(
        check(Some(t), anchor(Fixed, t, &[1], &[1])),
        Err(AnchorError::BaseSelf)
    );
    assert_eq!(
        check(None, anchor(Fixed, a, &[], &[1])),
        Err(AnchorError::DateEmpty {
            side: AnchorSide::Own,
            unit: Some("cycle".to_owned())
        })
    );
    assert_eq!(
        check(None, anchor(Fixed, a, &[1], &[])),
        Err(AnchorError::DateEmpty {
            side: AnchorSide::Base,
            unit: Some("orbit".to_owned())
        })
    );
    let Err(AnchorError::BaseDate(e)) = check(None, anchor(Fixed, a, &[1], &[412, 13])) else {
        panic!("expected BaseDate");
    };
    assert_eq!(e.to_string(), "span 13 is outside 1 to 12 in orbit 412");
    let Err(AnchorError::Date(e)) = check(None, anchor(Fixed, a, &[1, 53], &[412])) else {
        panic!("expected Date");
    };
    assert!(matches!(e, CalendarError::InvalidValue { ref unit, .. } if unit == "round"));
}

#[test]
fn messages_name_the_unit_and_the_calendars() {
    let empty = AnchorError::DateEmpty {
        side: AnchorSide::Own,
        unit: Some("cycle".to_owned()),
    };
    assert_eq!(empty.to_string(), "Enter at least the cycle.");
    let gap = AnchorError::DateGap {
        side: AnchorSide::Base,
        index: 1,
        unit: Some("span".to_owned()),
    };
    assert_eq!(
        gap.to_string(),
        "Fill in span, or leave every smaller unit empty."
    );
    assert_eq!(AnchorError::Missing.to_string(), "Choose a base calendar.");
    assert_eq!(
        AnchorError::BaseSelf.to_string(),
        "A calendar can't be its own base."
    );
    assert_eq!(
        AnchorError::OutOfRange.to_string(),
        "This anchor would put the calendar beyond the timeline's range."
    );
    let names = |n: &[&str]| n.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        AnchorError::Loop {
            names: names(&["Wick", "Tallow"])
        }
        .to_string(),
        "Wick follows Tallow, so Tallow can't be anchored to it."
    );
    assert_eq!(
        AnchorError::Loop {
            names: names(&["Wick", "Moss", "Tallow"])
        }
        .to_string(),
        "Wick follows Tallow through Moss, so Tallow can't be anchored to it."
    );
    assert_eq!(
        AnchorError::BaseUnusable {
            name: "Aster".to_owned(),
            problem: Box::new(PlacementProblem::NoMoment)
        }
        .to_string(),
        "Aster can't be used: its anchor has no moment: anchor it again"
    );
}

#[test]
fn an_unusable_base_is_refused() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let mut refused = tallow();
    refused.groupings[0].members[0].count = 0;
    let r = u
        .add_calendar(&with(refused, anchor(Fixed, a, &[1], &[1])))
        .unwrap();
    let result = u.check_anchor(None, &with(small("Wick"), anchor(Fixed, r, &[1], &[1])));
    assert!(matches!(
        result,
        Err(AnchorError::BaseUnusable { ref name, ref problem })
            if name == "Tallow" && matches!(**problem, PlacementProblem::Definition(_))
    ));
}

#[test]
fn editing_the_anchored_calendar_keeps_its_fixed_moment() {
    // FR-005b, even after Aster changed.
    let (mut u, a, t) = aster_and_tallow(Fixed);
    let moment = start(&u, t, &[1, 1, 1]);
    keep(&mut u, Some(a), aster_with_span(0, 57)).unwrap();
    let mut longer = with(tallow(), anchor(Fixed, a, &[1, 1, 1], &[412, 3, 9]));
    longer.fundamental.length = "31000".to_owned();
    assert_eq!(u.check_anchor(Some(t), &longer), Ok(Some(moment)));
    keep(&mut u, Some(t), longer).unwrap();
    assert_eq!(start(&u, t, &[1, 1, 1]), moment);
    // An edit that removes the anchor date is refused.
    keep(
        &mut u,
        Some(t),
        with(tallow(), anchor(Fixed, a, &[1, 1, 7], &[412, 3, 9])),
    )
    .unwrap();
    let mut fewer = with(tallow(), anchor(Fixed, a, &[1, 1, 7], &[412, 3, 9]));
    fewer.groupings[1].members[0].count = 6;
    assert!(matches!(
        u.check_anchor(Some(t), &fewer),
        Err(AnchorError::Date(CalendarError::InvalidValue { .. }))
    ));
}

#[test]
fn an_unchanged_fixed_anchor_is_kept_even_when_its_base_cannot_be_placed() {
    // A fixed anchor doesn't read its base (FR-005b), so editing the calendar still works.
    let (mut u, a, t) = aster_and_tallow(Fixed);
    let moment = start(&u, t, &[1, 1, 1]);
    let mut refused = aster();
    refused.groupings[0].members[0].count = 0;
    u.replace_calendar(a, &refused).unwrap();
    let mut edited = with(tallow(), anchor(Fixed, a, &[1, 1, 1], &[412, 3, 9]));
    edited.fundamental.length = "31000".to_owned();
    assert_eq!(u.check_anchor(Some(t), &edited), Ok(Some(moment)));
    // A changed one needs a usable base.
    let changed = with(tallow(), anchor(Fixed, a, &[1, 1, 2], &[412, 3, 9]));
    assert!(matches!(
        u.check_anchor(Some(t), &changed),
        Err(AnchorError::BaseUnusable { .. })
    ));
}

#[test]
fn changing_the_anchor_works_the_moment_out_again_from_the_base_as_it_is_now() {
    let (mut u, a, t) = aster_and_tallow(Fixed);
    keep(&mut u, Some(a), aster_with_span(0, 57)).unwrap();
    let now = start(&u, a, &[412, 3, 9]);
    let changes = [
        anchor(Equation, a, &[1, 1, 1], &[412, 3, 9]),
        anchor(Fixed, a, &[1, 1, 2], &[412, 3, 9]),
        anchor(Fixed, a, &[1, 1, 1], &[412, 3, 10]),
    ];
    for changed in changes {
        let kind = changed.kind;
        let at = u.check_anchor(Some(t), &with(tallow(), changed.clone()));
        let expected_base = start(&u, a, &changed.base_date);
        match kind {
            Fixed => assert_eq!(at, Ok(Some(expected_base))),
            Equation => assert_eq!(at, Ok(None)),
        }
        let _ = now;
    }
    // A different base.
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Fixed, a, &[1], &[1])),
    )
    .unwrap();
    assert_eq!(
        u.check_anchor(Some(t), &with(tallow(), anchor(Fixed, w, &[1, 1, 1], &[5]))),
        Ok(Some(start(&u, w, &[5])))
    );
}

#[test]
fn an_unchanged_anchor_to_a_deleted_base_is_kept_but_a_changed_one_needs_a_base() {
    let (mut u, a, t) = aster_and_tallow(Fixed);
    let moment = start(&u, t, &[1, 1, 1]);
    u.delete_calendar(a).unwrap();
    let kept = stored_anchor(&u, t).unwrap();
    assert_eq!(kept.base, None);
    assert_eq!(
        u.check_anchor(Some(t), &with(tallow(), kept.clone())),
        Ok(Some(moment))
    );
    let mut changed = kept;
    changed.date = vec![1, 1, 2];
    assert_eq!(
        u.check_anchor(Some(t), &with(tallow(), changed)),
        Err(AnchorError::BaseMissing)
    );
}

#[test]
fn an_anchor_beyond_the_timeline_is_out_of_range() {
    let mut u = universe();
    let a = keep(&mut u, None, small("Huge")).unwrap();
    // Huge's unit is 1,000 s: unit 9 × 10¹⁵ starts 9 × 10¹⁸ s after the zero, inside the
    // timeline; Tallow's cycle −10¹³ is about 10²⁰ s before its epoch, so its epoch would be
    // beyond the end.
    let record = with(
        tallow(),
        anchor(Fixed, a, &[-10_000_000_000_000], &[9_000_000_000_000_000]),
    );
    assert_eq!(u.check_anchor(None, &record), Err(AnchorError::OutOfRange));
}

#[test]
fn a_base_that_follows_the_anchored_calendar_is_a_loop() {
    // T019: Wick follows Tallow directly.
    let (mut u, a, t) = aster_and_tallow(Equation);
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Equation, t, &[1], &[1])),
    )
    .unwrap();
    for kind in [Fixed, Equation] {
        assert_eq!(
            u.check_anchor(Some(t), &with(tallow(), anchor(kind, w, &[1], &[1]))),
            Err(AnchorError::Loop {
                names: vec!["Wick".to_owned(), "Tallow".to_owned()]
            }),
            "{kind:?}"
        );
    }
    // And through a third calendar.
    let m = keep(
        &mut u,
        None,
        with(small("Moss"), anchor(Equation, w, &[1], &[1])),
    )
    .unwrap();
    assert_eq!(
        u.check_anchor(Some(t), &with(tallow(), anchor(Fixed, m, &[1], &[1]))),
        Err(AnchorError::Loop {
            names: vec!["Moss".to_owned(), "Wick".to_owned(), "Tallow".to_owned()]
        })
    );
    // The reference calendar can't take a base that follows it: Tallow follows Aster.
    assert!(matches!(
        u.check_anchor(Some(a), &with(aster(), anchor(Fixed, t, &[1], &[1]))),
        Err(AnchorError::Loop { .. })
    ));
}

#[test]
fn a_base_merely_fixed_to_the_anchored_calendar_is_accepted() {
    let (mut u, _, t) = aster_and_tallow(Fixed);
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Fixed, t, &[1], &[1])),
    )
    .unwrap();
    assert!(
        u.check_anchor(Some(t), &with(tallow(), anchor(Fixed, w, &[1, 1, 1], &[5])))
            .is_ok()
    );
}

// ---- Deleting calendars (T015) ----------------------------------------------------------------

#[test]
fn followers_are_the_calendars_keeping_an_equation_with_it() {
    let (mut u, a, _) = aster_and_tallow(Equation);
    keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Fixed, a, &[1], &[3])),
    )
    .unwrap();
    assert_eq!(u.followers(a).unwrap(), ["Tallow"]);
}

#[test]
fn deleting_a_base_fixes_its_followers_where_they_stand() {
    let (mut u, a, t) = aster_and_tallow(Equation);
    let w = keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Fixed, a, &[1], &[3])),
    )
    .unwrap();
    let m = keep(
        &mut u,
        None,
        with(small("Moss"), anchor(Equation, t, &[2], &[1, 4])),
    )
    .unwrap();
    let dates: [(i64, &[i64]); 4] = [(t, &[1, 1, 1]), (t, &[-4, 9, 2]), (w, &[17]), (m, &[-3])];
    let before: Vec<Instant> = dates.iter().map(|&(id, d)| start(&u, id, d)).collect();
    u.delete_calendar(a).unwrap();
    let after: Vec<Instant> = dates.iter().map(|&(id, d)| start(&u, id, d)).collect();
    assert_eq!(after, before, "FR-006a: nothing moves");
    let tallow_anchor = stored_anchor(&u, t).unwrap();
    assert_eq!(tallow_anchor.kind, Fixed);
    assert_eq!(tallow_anchor.at, Some(before[0]));
    for id in [t, w] {
        let kept = stored_anchor(&u, id).unwrap();
        assert_eq!(
            (kept.base, kept.base_name.as_deref()),
            (None, Some("Aster"))
        );
    }
    // Moss still follows Tallow.
    assert_eq!(stored_anchor(&u, m).unwrap().kind, Equation);
}

#[test]
fn after_deleting_the_reference_calendar_there_is_none() {
    let (mut u, a, t) = aster_and_tallow(Fixed);
    u.delete_calendar(a).unwrap();
    assert!(stored_anchor(&u, t).is_some());
    // A new calendar must now be anchored.
    assert_eq!(
        u.check_anchor(None, &small("Wick")),
        Err(AnchorError::Missing)
    );
}

#[test]
fn a_follower_that_could_not_be_placed_has_no_moment() {
    let mut u = universe();
    let a = keep(&mut u, None, aster()).unwrap();
    let t = keep(
        &mut u,
        None,
        with(tallow(), anchor(Equation, a, &[1], &[412, 2, 56])),
    )
    .unwrap();
    keep(&mut u, Some(a), aster_with_span(1, 55)).unwrap();
    u.delete_calendar(a).unwrap();
    let kept = stored_anchor(&u, t).unwrap();
    assert_eq!((kept.kind, kept.at), (Fixed, None));
    assert_eq!(problem(&u, t), PlacementProblem::NoMoment);
}

// ---- Files and the property test (T009) ------------------------------------------------------

#[test]
fn anchors_of_both_kinds_survive_save_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("anchors.universe");
    let (mut u, _, t) = aster_and_tallow(Fixed);
    keep(
        &mut u,
        None,
        with(small("Wick"), anchor(Equation, t, &[3], &[1, 2])),
    )
    .unwrap();
    let before = u.calendars().unwrap();
    u.save_as(&path).unwrap();
    drop(u);
    let reopened = OpenUniverse::open(&path, "0.1.0").unwrap();
    assert_eq!(reopened.calendars().unwrap(), before);
}

#[test]
fn a_version_2_file_keeps_every_date_at_the_same_instant() {
    // SC-004: in a version-2 file every calendar was at the hidden zero.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.universe");
    {
        let mut conn = Connection::open(&path).unwrap();
        crate::schema::create_v0(&conn, "0.0.1", 0).unwrap();
        let tx = conn.transaction().unwrap();
        for step in &crate::MIGRATIONS[..2] {
            tx.execute_batch(step).unwrap();
        }
        tx.execute("UPDATE meta SET schema_version = 2", [])
            .unwrap();
        tx.commit().unwrap();
    }
    // Add the calendars through a current universe's rows, written into the old file.
    let mut current = universe();
    let a = current.add_calendar(&aster()).unwrap();
    let t = current
        .add_calendar(&with(tallow(), anchor(Fixed, a, &[1], &[1])))
        .unwrap();
    {
        let conn = Connection::open(&path).unwrap();
        for table in ["calendar", "unit", "unit_member", "rule", "era"] {
            let rows: Vec<Vec<rusqlite::types::Value>> = {
                let mut stmt = current
                    .db
                    .prepare(&format!("SELECT * FROM {table}"))
                    .unwrap();
                let n = stmt.column_count();
                stmt.query_map([], |r| {
                    (0..n)
                        .map(|i| r.get::<_, rusqlite::types::Value>(i))
                        .collect()
                })
                .unwrap()
                .map(Result::unwrap)
                .collect()
            };
            for row in rows {
                let marks = vec!["?"; row.len()].join(", ");
                conn.execute(
                    &format!("INSERT INTO {table} VALUES ({marks})"),
                    rusqlite::params_from_iter(row),
                )
                .unwrap();
            }
        }
    }
    let original = std::fs::read(&path).unwrap();
    let opened = OpenUniverse::open(&path, "0.1.0").unwrap();
    assert!(opened.is_dirty());
    assert_eq!(stored_anchor(&opened, a), None);
    let migrated = stored_anchor(&opened, t).unwrap();
    assert_eq!(
        migrated,
        Anchor {
            at: Some(Instant::ZERO),
            ..anchor(Fixed, a, &[1], &[1])
        }
    );
    // Every date is where it was with both epochs at the hidden zero.
    for values in [&[1][..], &[412, 3, 9], &[-7, 12, 55, 19, 99]] {
        let old = Calendar::new(aster().to_def().unwrap()).unwrap();
        assert_eq!(
            start(&opened, a, values),
            old.date_to_instant(&date(values)).unwrap().instant
        );
    }
    for values in [&[1][..], &[2, 3, 4], &[-5, 52, 7]] {
        let old = Calendar::new(tallow().to_def().unwrap()).unwrap();
        assert_eq!(
            start(&opened, t, values),
            old.date_to_instant(&date(values)).unwrap().instant
        );
    }
    drop(opened);
    assert_eq!(
        std::fs::read(dir.path().join("old.v2.universe")).unwrap(),
        original
    );
}

/// A deterministic generator for the property test's choices.
struct Picks(u64);

impl Picks {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    /// An instant within about ±30,000 Earth years of the zero.
    fn instant(&mut self) -> Instant {
        let range: i128 = 1_000_000_000_000_000_000;
        let offset =
            (self.next() as i128 % range) * if self.next().is_multiple_of(2) { 1 } else { -1 };
        Instant::from_nanos(offset).unwrap()
    }
    /// A valid date of `calendar`: the date of a random instant, cut to a random length.
    fn date(&mut self, calendar: &Calendar) -> Vec<i64> {
        let mut values = calendar
            .instant_to_date(self.instant())
            .unwrap()
            .date
            .values;
        values.truncate(1 + self.below(values.len()));
        values
    }
}

/// The exact moment `t` falls at in `calendar` and back: the start of its date plus the
/// remainder, in nanoseconds, as an exact fraction.
fn there_and_back(calendar: &Calendar, t: Instant) -> Ratio {
    let found = calendar.instant_to_date(t).unwrap();
    let start = calendar.date_to_instant(&found.date).unwrap();
    let billion = Ratio::from_int(1_000_000_000).unwrap();
    let rounded = start
        .rounded_by
        .unwrap_or(Ratio::ZERO)
        .checked_mul(billion)
        .unwrap();
    Ratio::from_int(start.instant.as_nanos())
        .unwrap()
        .checked_sub(rounded)
        .unwrap()
        .checked_add(found.remainder.checked_mul(billion).unwrap())
        .unwrap()
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(1000))]

    /// SC-002: random universes of anchored calendars, saved and reopened: every anchor's two
    /// dates start at the same instant (or, for a fixed anchor, at its kept moment), and a
    /// moment taken from one calendar to another and back is the same moment.
    #[test]
    fn random_chains_of_anchors_hold_through_save_and_reopen(seed in proptest::prelude::any::<u64>()) {
        let picks = &mut Picks(seed | 1);
        let mut u = universe();
        let n = 2 + picks.below(4);
        let mut ids = Vec::new();
        for i in 0..n {
            let mut record = random_record(picks.next());
            record.name = format!("{i} {}", record.name);
            if i > 0 {
                let base = ids[picks.below(i)];
                let base_calendar = placed(&u, base).unwrap();
                let own = Calendar::new(record.to_def().unwrap()).unwrap();
                let kind = if picks.next().is_multiple_of(2) { Fixed } else { Equation };
                record.anchor = Some(anchor(kind, base, &picks.date(&own), &picks.date(&base_calendar)));
            }
            ids.push(keep(&mut u, None, record).unwrap());
        }
        let first_anchored = picks.below(4) == 0;
        if first_anchored {
            // Anchor the first calendar, fixing the moment, to a later one that doesn't follow it.
            let mut record = u.calendar(ids[0]).unwrap().record;
            let own = Calendar::new(record.to_def().unwrap()).unwrap();
            let candidates: Vec<i64> = ids[1..]
                .iter()
                .copied()
                .filter(|&b| following_chain(&u.placements().unwrap(), b, ids[0]).is_none())
                .collect();
            if !candidates.is_empty() {
                let base = candidates[picks.below(candidates.len())];
                let base_calendar = placed(&u, base).unwrap();
                record.anchor = Some(anchor(Fixed, base, &picks.date(&own), &picks.date(&base_calendar)));
                keep(&mut u, Some(ids[0]), record).unwrap();
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chain.universe");
        u.save_as(&path).unwrap();
        drop(u);
        let u = OpenUniverse::open(&path, "0.1.0").unwrap();
        let placements = u.placements().unwrap();
        let by_id: HashMap<i64, &Calendar> = placements
            .iter()
            .map(|p| (p.stored.id, p.placed.as_ref().unwrap()))
            .collect();
        for p in &placements {
            let Some(a) = &p.stored.record.anchor else { continue };
            let own = by_id[&p.stored.id].date_to_instant(&date(&a.date)).unwrap().instant;
            let base = by_id[&a.base.unwrap()].date_to_instant(&date(&a.base_date)).unwrap().instant;
            match a.kind {
                Fixed => {
                    proptest::prop_assert_eq!(Some(own), a.at);
                    if !first_anchored {
                        proptest::prop_assert_eq!(own, base);
                    }
                }
                Equation => proptest::prop_assert_eq!(own, base),
            }
        }
        // A date in one calendar, as a moment, through another calendar and back.
        let from = by_id[&ids[picks.below(n)]];
        let to = by_id[&ids[picks.below(n)]];
        let t = from.date_to_instant(&date(&picks.date(from))).unwrap().instant;
        let back = there_and_back(to, t);
        proptest::prop_assert_eq!(back, Ratio::from_int(t.as_nanos()).unwrap());
    }
}
