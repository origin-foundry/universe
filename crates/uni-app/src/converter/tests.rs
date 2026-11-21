//! The converter's commands (T020, T021) and its timing check (T024).

use serde_json::json;
use uni_time::Ratio;

use super::*;
use crate::AnchorKindForm::{Equation, Fixed};
use crate::calendars::tests::{aster, open_session};
use crate::fixtures::*;

fn values(date: &ConvertedDate) -> Vec<i64> {
    date.values.iter().map(|v| v.value).collect()
}

fn some(values: &[i64]) -> Vec<Option<i64>> {
    values.iter().copied().map(Some).collect()
}

fn date_error(session: &Session, from: i64, date: &[Option<i64>]) -> DateError {
    match session.convert(from, date) {
        Err(ConvertError::Date(e)) => e,
        other => panic!("expected a date error, got {other:?}"),
    }
}

#[test]
fn converter_calendars_lists_each_with_its_units_from_the_eras_unit_down() {
    let (session, a, t) = aster_and_tallow(Equation);
    let mut shorter = aster();
    shorter.groupings[2].members[2].count = Some(8);
    let listed = session.converter_calendars().unwrap();
    assert_eq!(
        serde_json::to_value(&listed).unwrap(),
        json!([
            { "id": a, "name": "Aster", "units": ["orbit", "span", "sol", "bell", "pulse"],
              "problem": null },
            { "id": t, "name": "Tallow", "units": ["cycle", "round", "watch"], "problem": null }
        ])
    );
    put(&session, Some(a), &shorter);
    let listed = session.converter_calendars().unwrap();
    assert!(
        listed[1]
            .problem
            .as_deref()
            .unwrap()
            .starts_with("its anchoring date in Aster")
    );
    assert_eq!(listed[1].units, ["cycle", "round", "watch"]);
}

#[test]
fn tallows_anchor_date_is_its_base_date_in_aster() {
    // Story 2, scenario 1.
    let (session, a, t) = aster_and_tallow(Fixed);
    let conversion = session.convert(t, &some(&[1, 1, 1])).unwrap();
    assert!(!conversion.rounded);
    assert!(conversion.unusable.is_empty());
    let [result] = &conversion.results[..] else {
        panic!("one result");
    };
    assert_eq!((result.id, result.name.as_str()), (a, "Aster"));
    assert_eq!(
        serde_json::to_value(&result.date).unwrap(),
        json!({
            "above": [
                { "unit": "age", "value": 5, "name": null },
                { "unit": "cohort", "value": 42, "name": null }
            ],
            "values": [
                { "unit": "orbit", "value": 412, "name": null },
                { "unit": "span", "value": 3, "name": "Third" },
                { "unit": "sol", "value": 9, "name": null },
                { "unit": "bell", "value": 0, "name": null },
                { "unit": "pulse", "value": 0, "name": null }
            ],
            "partway": null
        })
    );
}

#[test]
fn a_moment_partway_through_a_watch_gives_every_form() {
    // Story 2, scenarios 2 and 3: bell 7 of sol 9 is 7/20 of 88,775.244 s, 31,071.3354 s
    // after Tallow's epoch: 1,071.3354 s into watch 2.
    let (session, a, t) = aster_and_tallow(Fixed);
    let conversion = session.convert(a, &some(&[412, 3, 9, 7])).unwrap();
    let result = &conversion.results[0];
    assert_eq!(result.id, t);
    assert_eq!(values(&result.date), [1, 1, 2]);
    let fraction = Ratio::new(10_713_354, 300_000_000).unwrap();
    assert_eq!(
        result.date.partway,
        Some(Partway {
            decimal: "0.035".to_owned(),
            decimal_exact: false,
            exact: fraction.to_string(),
        })
    );
    assert_eq!(
        serde_json::to_value(&result.date.partway).unwrap(),
        json!({ "decimal": "0.035", "decimalExact": false, "exact": "1785559/50000000" })
    );
}

#[test]
fn a_third_of_a_unit_is_inexact_as_a_decimal() {
    let (session, _, t) = aster_and_tallow(Fixed);
    put(
        &session,
        None,
        &single(
            "Thirds",
            "span",
            "90000",
            Some(anchor(Equation, t, &[1], &[1])),
        ),
    );
    let conversion = session.convert(t, &some(&[1, 1, 2])).unwrap();
    let thirds = conversion
        .results
        .iter()
        .find(|r| r.name == "Thirds")
        .unwrap();
    assert_eq!(values(&thirds.date), [1]);
    assert_eq!(
        thirds.date.partway,
        Some(Partway {
            decimal: "0.333".to_owned(),
            decimal_exact: false,
            exact: "1/3".to_owned(),
        })
    );
    // At the start of a unit, there is no mark.
    let conversion = session.convert(t, &some(&[1, 1, 1])).unwrap();
    let thirds = conversion
        .results
        .iter()
        .find(|r| r.name == "Thirds")
        .unwrap();
    assert_eq!(thirds.date.partway, None);
}

#[test]
fn dates_before_the_epochs_use_astronomical_numbers() {
    // Story 2, scenario 5.
    let (session, a, t) = aster_and_tallow(Fixed);
    let conversion = session.convert(t, &some(&[-3])).unwrap();
    assert_eq!(conversion.results[0].id, a);
    let conversion = session.convert(a, &some(&[-5, 2])).unwrap();
    let tallow = &conversion.results[0];
    assert!(tallow.date.values[0].value < 1, "{:?}", tallow.date);
}

#[test]
fn refused_dates_give_the_engines_reason_at_the_value() {
    // Story 2, scenario 4; FR-011.
    let (session, a, t) = aster_and_tallow(Fixed);
    assert_eq!(
        date_error(&session, a, &some(&[412, 13])),
        DateError {
            message: "span 13 is outside 1 to 12 in orbit 412".to_owned(),
            index: Some(1)
        }
    );
    assert_eq!(
        date_error(&session, t, &some(&[1, 1, 8])),
        DateError {
            message: "watch 8 is outside 1 to 7".to_owned(),
            index: Some(2)
        }
    );
    assert_eq!(
        date_error(&session, a, &[Some(412), None, Some(9)]),
        DateError {
            message: "Fill in span, or leave every smaller unit empty.".to_owned(),
            index: Some(1)
        }
    );
    assert_eq!(
        date_error(&session, a, &[None, None]),
        DateError {
            message: "Enter at least the orbit.".to_owned(),
            index: Some(0)
        }
    );
    assert_eq!(
        serde_json::to_value(ConvertError::Date(DateError {
            message: "m".to_owned(),
            index: None
        }))
        .unwrap(),
        json!({ "message": "m", "index": null })
    );
    // Trailing empty values are fine.
    assert!(session.convert(a, &[Some(412), None, None]).is_ok());
}

#[test]
fn unusable_calendars_are_listed_with_their_problems_not_converted_into() {
    // FR-006.
    let (session, a, t) = aster_and_tallow(Equation);
    let w = put(
        &session,
        None,
        &single("Wick", "beat", "1000", Some(anchor(Fixed, a, &[1], &[1]))),
    );
    let mut shorter = aster();
    shorter.groupings[2].members[2].count = Some(8);
    put(&session, Some(a), &shorter);
    let conversion = session.convert(w, &some(&[5])).unwrap();
    assert_eq!(
        conversion.results.iter().map(|r| r.id).collect::<Vec<_>>(),
        [a]
    );
    assert_eq!(conversion.unusable.len(), 1);
    assert_eq!(conversion.unusable[0].name, "Tallow");
    // And Tallow itself can't be converted from.
    assert!(matches!(
        session.convert(t, &some(&[1])),
        Err(ConvertError::Store(e)) if e.starts_with("Tallow can't be used: ")
    ));
}

#[test]
fn with_one_calendar_there_is_nothing_to_convert_into() {
    // Story 2, scenario 6.
    let session = open_session();
    let a = put(&session, None, &aster());
    let conversion = session.convert(a, &some(&[1])).unwrap();
    assert!(conversion.results.is_empty() && conversion.unusable.is_empty());
}

#[test]
fn a_date_starting_between_whole_nanoseconds_is_reported_as_rounded() {
    let session = open_session();
    let a = put(&session, None, &aster());
    let awkward = put(
        &session,
        None,
        &single(
            "Awkward",
            "beat",
            "355/113",
            Some(anchor(Fixed, a, &[1], &[1])),
        ),
    );
    assert!(session.convert(awkward, &some(&[8])).unwrap().rounded);
    assert!(!session.convert(awkward, &some(&[1])).unwrap().rounded);
}

#[test]
fn no_timeline_value_is_returned() {
    // FR-012: only dates, names, whole numbers and fractions of a unit.
    let (session, a, t) = aster_and_tallow(Fixed);
    let mut texts = vec![
        serde_json::to_string(&session.convert(a, &some(&[412, 3, 9, 7])).unwrap()).unwrap(),
        serde_json::to_string(&session.converter_calendars().unwrap()).unwrap(),
        serde_json::to_string(&session.get_calendar(t).unwrap()).unwrap(),
        serde_json::to_string(&session.anchor_bases(Some(t)).unwrap()).unwrap(),
    ];
    let mut longer = aster();
    longer.groupings[2].members[0].count = Some(57);
    put(&session, Some(a), &longer);
    texts.push(serde_json::to_string(&session.get_calendar(t).unwrap()).unwrap());
    for text in texts {
        for word in ["instant", "secs", "nanos", "epoch", "\"at\""] {
            assert!(!text.contains(word), "{word} in {text}");
        }
    }
}

#[test]
fn convert_needs_an_open_universe_and_an_existing_calendar() {
    let session = Session::default();
    assert_eq!(
        session.convert(1, &some(&[1])),
        Err(ConvertError::Store(NO_UNIVERSE.to_owned()))
    );
    assert_eq!(session.converter_calendars().unwrap_err(), NO_UNIVERSE);
    let session = open_session();
    assert_eq!(
        session.convert(9, &some(&[1])),
        Err(ConvertError::Store(
            "That calendar no longer exists.".to_owned()
        ))
    );
}

/// SC-005, run by hand in release mode:
/// `cargo test -p uni-app --release -- --ignored convert_twenty_calendars_timing --nocapture`.
#[test]
#[ignore = "timing check; run by hand with --release -- --ignored --nocapture"]
fn convert_twenty_calendars_timing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("twenty.universe");
    let session = open_session();
    let mut previous = put(&session, None, &aster());
    // A calendar whose orbit has 1,000 spans.
    let mut big = aster();
    big.name = "Big".to_owned();
    big.groupings[2].members = (1..=1000)
        .map(|j| crate::calendars::MemberForm {
            name: (j % 3 != 0).then(|| format!("Member {j}")),
            count: Some(1 + j % 7),
        })
        .collect();
    big.groupings[2].rules.clear();
    big.anchor = Some(anchor(Equation, previous, &[1], &[3]));
    previous = put(&session, None, &big);
    for n in 3..=20 {
        let mut next = aster();
        next.name = format!("Calendar {n}");
        next.anchor = Some(anchor(Equation, previous, &[1], &[2, 1, 1]));
        previous = put(&session, None, &next);
    }
    session.save_as(path.to_str().unwrap()).unwrap();
    session.open(path.to_str().unwrap()).unwrap();
    let start = std::time::Instant::now();
    let conversion = session
        .convert(previous, &some(&[412, 3, 9, 7, 41]))
        .unwrap();
    let took = start.elapsed();
    assert_eq!(conversion.results.len(), 19);
    println!("convert across 20 calendars took {took:?}");
    assert!(took.as_secs_f64() < 1.0, "SC-005: {took:?}");
}
