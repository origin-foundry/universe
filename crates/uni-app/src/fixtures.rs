//! Test calendars for spec 008's app tests: Aster (from spec 007's tests), Tallow and small
//! calendars, entered as the UI sends them.

use serde_json::{Value, json};

use crate::calendars::tests::{aster, open_session};
use crate::{AnchorForm, AnchorKindForm, CalendarForm, PutError, Session};

/// The spec's Tallow: a cycle of 52 rounds, a round of 7 watches, a watch of 30,000 s.
pub(crate) fn tallow_json(anchor: Value) -> Value {
    let one = |name: &str, count: i64| {
        json!({ "name": name, "firstNumber": null, "memberUnit": null,
                "members": [{ "name": null, "count": count }], "rules": [] })
    };
    json!({
        "name": "Tallow",
        "groupings": [one("cycle", 52), one("round", 7)],
        "fundamental": { "name": "watch", "length": "30000", "firstNumber": null },
        "subdivisions": [],
        "era": { "name": "Count", "unit": "cycle" },
        "anchor": anchor
    })
}

pub(crate) fn form(value: Value) -> CalendarForm {
    serde_json::from_value(value).unwrap()
}

pub(crate) fn anchor(
    kind: AnchorKindForm,
    base: i64,
    date: &[i64],
    base_date: &[i64],
) -> AnchorForm {
    AnchorForm {
        kind,
        base: Some(base),
        date: date.iter().copied().map(Some).collect(),
        base_date: base_date.iter().copied().map(Some).collect(),
    }
}

/// A calendar of one unit called `unit`, `length` seconds long.
pub(crate) fn single(
    name: &str,
    unit: &str,
    length: &str,
    anchor: Option<AnchorForm>,
) -> CalendarForm {
    form(json!({
        "name": name,
        "groupings": [],
        "fundamental": { "name": unit, "length": length, "firstNumber": null },
        "subdivisions": [],
        "era": { "name": "Era", "unit": unit },
        "anchor": anchor
    }))
}

pub(crate) fn with_anchor(mut form: CalendarForm, anchor: AnchorForm) -> CalendarForm {
    form.anchor = Some(anchor);
    form
}

pub(crate) fn tallow(anchor: AnchorForm) -> CalendarForm {
    with_anchor(form(tallow_json(Value::Null)), anchor)
}

/// A session with Aster, and Tallow anchored to it with `kind`: Tallow's cycle 1, round 1,
/// watch 1 at the start of Aster's orbit 412, span 3, sol 9.
pub(crate) fn aster_and_tallow(kind: AnchorKindForm) -> (Session, i64, i64) {
    let session = open_session();
    let a = put(&session, None, &aster());
    let t = put(
        &session,
        None,
        &tallow(anchor(kind, a, &[1, 1, 1], &[412, 3, 9])),
    );
    (session, a, t)
}

pub(crate) fn put(session: &Session, id: Option<i64>, form: &CalendarForm) -> i64 {
    match session.put_calendar(id, form) {
        Ok(put) => put.id,
        Err(PutError::Field(e)) => panic!("refused: {e:?}"),
        Err(PutError::Store(e)) => panic!("store: {e}"),
    }
}
