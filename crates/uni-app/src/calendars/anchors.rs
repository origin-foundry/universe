//! Spec 008 in the editor's commands: the anchor in the form, its refusals and where each is
//! placed (SC-003), the details the editor shows, the base list and live date checks.

use serde_json::json;

use super::tests::{aster, open_session};
use super::*;
use crate::fixtures::*;
use AnchorKindForm::{Equation, Fixed};

fn refused(session: &Session, id: Option<i64>, form: &CalendarForm) -> FieldError {
    let before = session.list_calendars().unwrap();
    let details: Vec<CalendarDetails> = before
        .iter()
        .map(|c| session.get_calendar(c.id).unwrap())
        .collect();
    let error = match session.put_calendar(id, form) {
        Err(PutError::Field(error)) => error,
        other => panic!("expected a field error, got {other:?}"),
    };
    // Nothing was stored.
    assert_eq!(session.list_calendars().unwrap(), before);
    let after: Vec<CalendarDetails> = before
        .iter()
        .map(|c| session.get_calendar(c.id).unwrap())
        .collect();
    assert_eq!(after, details);
    error
}

fn at(field: AnchorField, index: Option<usize>) -> Location {
    Location::Anchor {
        field: Some(field),
        index,
    }
}

#[test]
fn the_anchor_converts_to_and_from_a_record_in_camel_case_json() {
    let value = tallow_json(json!({
        "kind": "fixed", "base": 1, "date": [1, 1, 1], "baseDate": [412, 3, 9, null, null]
    }));
    let form = form(value);
    let record = form.to_record().unwrap();
    let anchor = record.anchor.clone().unwrap();
    assert_eq!(
        (anchor.kind, anchor.base, anchor.date, anchor.base_date),
        (AnchorKind::Fixed, Some(1), vec![1, 1, 1], vec![412, 3, 9])
    );
    // Back as the form, with the trailing empty values gone.
    let back = serde_json::to_value(CalendarForm::from(record)).unwrap();
    assert_eq!(
        back["anchor"],
        json!({ "kind": "fixed", "base": 1, "date": [1, 1, 1], "baseDate": [412, 3, 9] })
    );
    let equation = tallow(anchor_form(Equation));
    assert_eq!(
        serde_json::to_value(&equation).unwrap()["anchor"]["kind"],
        "equation"
    );
}

fn anchor_form(kind: AnchorKindForm) -> AnchorForm {
    anchor(kind, 1, &[1], &[1])
}

#[test]
fn a_new_calendar_is_anchored_to_the_reference_calendar_unless_the_universe_is_empty() {
    let session = open_session();
    assert_eq!(session.new_calendar_form().unwrap().anchor, None);
    let a = put(&session, None, &aster());
    let expected = json!({ "kind": "fixed", "base": a, "date": [1], "baseDate": [1] });
    let new = session.new_calendar_form().unwrap();
    assert_eq!(serde_json::to_value(&new).unwrap()["anchor"], expected);
    // Even when the reference calendar isn't first in the list.
    let t = put(&session, None, &tallow(anchor(Fixed, a, &[1], &[1])));
    put(
        &session,
        Some(a),
        &with_anchor(aster(), anchor(Fixed, t, &[1], &[1])),
    );
    session.delete_calendar(t).unwrap();
    let w = put(
        &session,
        None,
        &single("Wick", "beat", "1000", Some(anchor(Fixed, a, &[1], &[1]))),
    );
    // Now there's no reference calendar: the first in the list is the base.
    let _ = w;
    assert_eq!(
        session.new_calendar_form().unwrap().anchor.unwrap().base,
        Some(a)
    );
}

#[test]
fn put_stores_tallows_anchor_and_its_moment() {
    let (session, a, t) = aster_and_tallow(Fixed);
    let details = session.get_calendar(t).unwrap();
    assert_eq!(
        details.form.anchor,
        Some(anchor(Fixed, a, &[1, 1, 1], &[412, 3, 9]))
    );
    assert_eq!((details.rounded, details.note), (false, None));
    assert_eq!(session.get_calendar(a).unwrap().form.anchor, None);
    let json = serde_json::to_value(session.get_calendar(a).unwrap()).unwrap();
    assert_eq!(json["form"]["anchor"], json!(null));
    assert_eq!(json["rounded"], json!(false));
    assert_eq!(json["note"], json!(null));
    // The moment is stored: Tallow's first watch converts to Aster's base date.
    let conversion = session.convert(t, &[Some(1), Some(1), Some(1)]).unwrap();
    let values: Vec<i64> = conversion.results[0]
        .date
        .values
        .iter()
        .map(|v| v.value)
        .collect();
    assert_eq!(values, [412, 3, 9, 0, 0]);
}

#[test]
fn every_anchor_refusal_is_placed_at_its_field() {
    let (session, a, t) = aster_and_tallow(Equation);
    let base = |field| at(field, None);
    // Missing: a second calendar without an anchor.
    let mut bare = tallow(anchor(Fixed, a, &[1], &[1]));
    bare.anchor = None;
    bare.name = "Bare".to_owned();
    assert_eq!(
        refused(&session, None, &bare),
        FieldError::new("Choose a base calendar.", base(AnchorField::Base))
    );
    // BaseMissing.
    let mut none = anchor(Fixed, a, &[1], &[1]);
    none.base = None;
    assert_eq!(
        refused(&session, Some(t), &tallow(none)).at,
        base(AnchorField::Base)
    );
    assert_eq!(
        refused(&session, Some(t), &tallow(anchor(Fixed, 999, &[1], &[1]))).at,
        base(AnchorField::Base)
    );
    // BaseSelf.
    assert_eq!(
        refused(&session, Some(t), &tallow(anchor(Fixed, t, &[1], &[1]))),
        FieldError::new("A calendar can't be its own base.", base(AnchorField::Base))
    );
    // DateEmpty on each side.
    assert_eq!(
        refused(&session, Some(t), &tallow(anchor(Fixed, a, &[], &[1]))),
        FieldError::new("Enter at least the cycle.", at(AnchorField::Date, Some(0)))
    );
    assert_eq!(
        refused(&session, Some(t), &tallow(anchor(Fixed, a, &[1], &[]))),
        FieldError::new(
            "Enter at least the orbit.",
            at(AnchorField::BaseDate, Some(0))
        )
    );
    // DateGap on each side.
    let mut gap = anchor(Fixed, a, &[1], &[1]);
    gap.date = vec![Some(1), None, Some(1)];
    assert_eq!(
        refused(&session, Some(t), &tallow(gap)),
        FieldError::new(
            "Fill in round, or leave every smaller unit empty.",
            at(AnchorField::Date, Some(1))
        )
    );
    let mut gap = anchor(Fixed, a, &[1], &[1]);
    gap.base_date = vec![Some(412), None, Some(9)];
    assert_eq!(
        refused(&session, Some(t), &tallow(gap)),
        FieldError::new(
            "Fill in span, or leave every smaller unit empty.",
            at(AnchorField::BaseDate, Some(1))
        )
    );
    // The engine's InvalidValue on each side, at the value's index.
    assert_eq!(
        refused(
            &session,
            Some(t),
            &tallow(anchor(Fixed, a, &[1], &[412, 13]))
        ),
        FieldError::new(
            "span 13 is outside 1 to 12 in orbit 412",
            at(AnchorField::BaseDate, Some(1))
        )
    );
    assert_eq!(
        refused(
            &session,
            Some(t),
            &tallow(anchor(Fixed, a, &[1, 1, 8], &[1]))
        ),
        FieldError::new("watch 8 is outside 1 to 7", at(AnchorField::Date, Some(2)))
    );
    // OutOfRange: the epoch would be beyond the timeline.
    let huge = put(
        &session,
        None,
        &single("Huge", "aeon", "1000", Some(anchor(Fixed, a, &[1], &[1]))),
    );
    assert_eq!(
        refused(
            &session,
            Some(t),
            &tallow(anchor(
                Fixed,
                huge,
                &[-10_000_000_000_000],
                &[9_000_000_000_000_000]
            ))
        ),
        FieldError::new(
            "This anchor would put the calendar beyond the timeline's range.",
            Location::Anchor {
                field: None,
                index: None
            }
        )
    );
    // Loop: Wick follows Tallow.
    let w = put(
        &session,
        None,
        &single(
            "Wick",
            "beat",
            "1000",
            Some(anchor(Equation, t, &[1], &[1])),
        ),
    );
    assert_eq!(
        refused(
            &session,
            Some(t),
            &tallow(anchor(Fixed, w, &[1, 1, 1], &[1]))
        ),
        FieldError::new(
            "Wick follows Tallow, so Tallow can't be anchored to it.",
            base(AnchorField::Base)
        )
    );
}

#[test]
fn an_unusable_base_is_refused_at_the_base() {
    let session = open_session();
    let a = put(&session, None, &aster());
    let broken = {
        let slot = &mut *session.lock();
        let universe = slot.as_mut().unwrap();
        let mut record = aster().to_record().unwrap();
        record.name = "Broken".to_owned();
        record.fundamental.length = "88775,244".to_owned();
        record.anchor = Some(uni_store::Anchor {
            kind: AnchorKind::Fixed,
            base: Some(a),
            base_name: None,
            date: vec![1],
            base_date: vec![1],
            at: Some(uni_time::Instant::ZERO),
        });
        universe.add_calendar(&record).unwrap()
    };
    let error = refused(&session, None, &tallow(anchor(Fixed, broken, &[1], &[1])));
    assert_eq!(error.at, at(AnchorField::Base, None));
    assert!(
        error.message.starts_with("Broken can't be used: "),
        "{}",
        error.message
    );
}

#[test]
fn list_calendars_gives_placement_problems() {
    let (session, a, t) = aster_and_tallow(Equation);
    let mut shorter = aster();
    shorter.groupings[2].members[2].count = Some(8);
    put(&session, Some(a), &shorter);
    let listed = session.list_calendars().unwrap();
    assert_eq!(listed[0].problem, None);
    assert_eq!(listed[1].id, t);
    assert_eq!(
        listed[1].problem.as_deref(),
        Some("its anchoring date in Aster: sol 9 is outside 1 to 8 in Third in orbit 412")
    );
}

#[test]
fn the_base_list_greys_out_followers_and_unplaceable_calendars() {
    let (session, a, t) = aster_and_tallow(Equation);
    let w = put(
        &session,
        None,
        &single(
            "Wick",
            "beat",
            "1000",
            Some(anchor(Equation, t, &[1], &[1])),
        ),
    );
    let bases = session.anchor_bases(Some(t)).unwrap();
    assert_eq!(
        serde_json::to_value(&bases).unwrap(),
        json!([
            { "id": a, "name": "Aster", "ineligible": null },
            { "id": w, "name": "Wick", "ineligible": "follows" }
        ])
    );
    // Once Wick is fixed instead, it can be chosen.
    put(
        &session,
        Some(w),
        &single("Wick", "beat", "1000", Some(anchor(Fixed, t, &[1], &[1]))),
    );
    assert_eq!(session.anchor_bases(Some(t)).unwrap()[1].ineligible, None);
    // A calendar that can't be placed.
    let mut shorter = aster();
    shorter.groupings[2].members[2].count = Some(8);
    put(&session, Some(a), &shorter);
    assert_eq!(
        session.anchor_bases(Some(w)).unwrap()[1].ineligible,
        Some(Ineligible::Unplaceable),
        "Tallow follows Aster, at a date Aster no longer has"
    );
    // For a new calendar, every calendar is listed.
    let all: Vec<i64> = session
        .anchor_bases(None)
        .unwrap()
        .iter()
        .map(|b| b.id)
        .collect();
    assert_eq!(all, [a, t, w]);
}

#[test]
fn read_date_reads_a_date_back_with_member_names() {
    let session = open_session();
    let a = put(&session, None, &aster());
    let source = DateSource::Id { id: a };
    let reading = session
        .read_date(&source, &[Some(412), Some(3), Some(9)])
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reading).unwrap(),
        json!({ "values": [
            { "unit": "orbit", "value": 412, "name": null },
            { "unit": "span", "value": 3, "name": "Third" },
            { "unit": "sol", "value": 9, "name": null },
            { "unit": "bell", "value": 0, "name": null },
            { "unit": "pulse", "value": 0, "name": null }
        ] })
    );
    let span_13 = session.read_date(&source, &[Some(412), Some(13)]).unwrap();
    assert_eq!(
        serde_json::to_value(&span_13).unwrap(),
        json!({ "message": "span 13 is outside 1 to 12 in orbit 412", "index": 1 })
    );
    // A draft, checked against its own definition.
    let draft = DateSource::Form {
        form: Box::new(tallow(anchor(Fixed, a, &[1], &[1]))),
    };
    let reading = session
        .read_date(&draft, &[Some(1), Some(1), Some(1)])
        .unwrap();
    assert!(matches!(reading, DateCheck::Reading(_)));
    let mut bad = tallow(anchor(Fixed, a, &[1], &[1]));
    bad.groupings[0].members[0].count = Some(0);
    let refused = session
        .read_date(
            &DateSource::Form {
                form: Box::new(bad),
            },
            &[Some(1)],
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&refused).unwrap(),
        json!({ "definition": "member 1 of cycle must have a positive count" })
    );
    // The source arrives from the UI as JSON.
    let parsed: DateSource = serde_json::from_value(json!({ "id": a })).unwrap();
    assert_eq!(parsed, source);
}

// ---- What the editor says about an anchor (T016) ----

#[test]
fn a_fixed_anchor_notes_where_its_moment_now_falls() {
    let (session, a, t) = aster_and_tallow(Fixed);
    let mut longer = aster();
    longer.groupings[2].members[0].count = Some(57);
    put(&session, Some(a), &longer);
    let details = session.get_calendar(t).unwrap();
    let Some(Note::Moved { now }) = &details.note else {
        panic!("expected a moved note, got {:?}", details.note);
    };
    let values: Vec<i64> = now.values.iter().map(|v| v.value).collect();
    // 411 orbits each a sol longer: 411 sols earlier in the calendar's dates.
    assert_ne!(values[..3], [412, 3, 9]);
    assert_eq!(
        serde_json::to_value(&details.note).unwrap()["kind"],
        json!("moved")
    );
    // An equation anchor follows instead, so has no note.
    let (session, a, t) = aster_and_tallow(Equation);
    put(&session, Some(a), &longer);
    assert_eq!(session.get_calendar(t).unwrap().note, None);
}

#[test]
fn an_anchor_whose_base_was_deleted_says_so() {
    let (session, a, t) = aster_and_tallow(Equation);
    assert_eq!(session.calendar_followers(a).unwrap(), ["Tallow"]);
    session.delete_calendar(a).unwrap();
    let details = session.get_calendar(t).unwrap();
    assert_eq!(
        serde_json::to_value(&details.note).unwrap(),
        json!({ "kind": "gone", "name": "Aster" })
    );
    assert_eq!(details.form.anchor.as_ref().unwrap().kind, Fixed);
    assert_eq!(details.form.anchor.unwrap().base, None);
    // Kept unchanged, it stays as it is.
    let form = session.get_calendar(t).unwrap().form;
    put(&session, Some(t), &form);
    assert!(matches!(
        session.get_calendar(t).unwrap().note,
        Some(Note::Gone { .. })
    ));
}

#[test]
fn rounded_says_whether_the_base_date_starts_between_two_whole_nanoseconds() {
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
    let t = put(&session, None, &tallow(anchor(Fixed, awkward, &[1], &[8])));
    assert!(session.get_calendar(t).unwrap().rounded);
    let (session, _, t) = aster_and_tallow(Fixed);
    assert!(!session.get_calendar(t).unwrap().rounded);
}

#[test]
fn the_anchor_location_serialises_as_the_contract_says() {
    assert_eq!(
        serde_json::to_value(at(AnchorField::BaseDate, Some(1))).unwrap(),
        json!({ "kind": "anchor", "field": "baseDate", "index": 1 })
    );
    assert_eq!(
        serde_json::to_value(Location::Anchor {
            field: None,
            index: None
        })
        .unwrap(),
        json!({ "kind": "anchor" })
    );
}

#[test]
fn the_new_commands_need_an_open_universe() {
    let session = Session::default();
    assert_eq!(session.anchor_bases(None).unwrap_err(), NO_UNIVERSE);
    assert_eq!(session.calendar_followers(1).unwrap_err(), NO_UNIVERSE);
    assert_eq!(
        session
            .read_date(&DateSource::Id { id: 1 }, &[Some(1)])
            .unwrap_err(),
        NO_UNIVERSE
    );
}
