//! Anchors (spec 008): how each calendar is placed in time, how anchors are stored, checked
//! when the editor keeps a calendar, and kept right when a calendar is deleted
//! (specs/008-epoch-anchoring/research.md, R2 to R6).
//!
//! An anchor places its **anchored calendar** from a **base calendar**: its **anchor date**
//! starts at the same moment as the **base date**. A **fixed** anchor keeps the instant worked
//! out when it was confirmed (`at`); an **equation** anchor works it out from the base calendar
//! each time, so the anchored calendar **follows** its base. The one calendar with no anchor,
//! if there is one, is the **reference calendar**, at the hidden zero. Epochs are never
//! stored: [`OpenUniverse::placements`] works them out.

use std::collections::HashMap;
use std::fmt;

use rusqlite::{Connection, OptionalExtension, params};
use uni_calendar::{Calendar, CalendarError, Date, DefinitionError};
use uni_time::Instant;

use crate::{CalendarRecord, OpenUniverse, RecordError, StoreError, StoredCalendar};

/// Which way an anchor keeps its moment (FR-005).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnchorKind {
    /// Fix the moment (the default): the instant is worked out on confirm and kept.
    Fixed,
    /// Keep the equation: the instant is worked out from the base calendar each time.
    Equation,
}

/// A calendar's link into time (data-model.md, "Anchor").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub kind: AnchorKind,
    /// The base calendar's id; `None` once it has been deleted. An equation anchor has one.
    pub base: Option<i64>,
    /// The base calendar's name, written only when it was deleted.
    pub base_name: Option<String>,
    /// The anchor date, in the anchored calendar, from the era's unit down.
    pub date: Vec<i64>,
    /// The base date, in the base calendar, from the era's unit down.
    pub base_date: Vec<i64>,
    /// For a fixed anchor, the instant both dates started at when it was confirmed. Never
    /// shown: it's a value on the hidden timeline (principle IV).
    pub at: Option<Instant>,
}

/// Which date of an anchor a refusal concerns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnchorSide {
    /// The anchor date, in the anchored calendar.
    Own,
    /// The base date, in the base calendar.
    Base,
}

/// Why an anchor can't be kept (research R5). Each displays as a sentence for the editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnchorError {
    /// A calendar that must be anchored has no anchor.
    Missing,
    /// No base calendar was chosen, or it doesn't exist.
    BaseMissing,
    BaseSelf,
    /// A date with no values; `unit` is the era's unit of that side's calendar, when known.
    DateEmpty {
        side: AnchorSide,
        unit: Option<String>,
    },
    /// A value missing before a given one; `unit` names the missing one, when known.
    DateGap {
        side: AnchorSide,
        index: usize,
        unit: Option<String>,
    },
    BaseUnusable {
        name: String,
        problem: Box<PlacementProblem>,
    },
    /// The base follows the anchored calendar: `names` runs from the base, through the
    /// calendars between, to the anchored calendar.
    Loop {
        names: Vec<String>,
    },
    BaseDate(CalendarError),
    Date(CalendarError),
    OutOfRange,
}

impl fmt::Display for AnchorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorError::Missing | AnchorError::BaseMissing => {
                write!(f, "Choose a base calendar.")
            }
            AnchorError::BaseSelf => write!(f, "A calendar can't be its own base."),
            AnchorError::DateEmpty { unit, .. } => match unit {
                Some(unit) => write!(f, "Enter at least the {unit}."),
                None => write!(f, "Enter at least the first value."),
            },
            AnchorError::DateGap { unit, .. } => match unit {
                Some(unit) => write!(f, "Fill in {unit}, or leave every smaller unit empty."),
                None => write!(f, "Fill in this value, or leave every smaller unit empty."),
            },
            AnchorError::BaseUnusable { name, problem } => {
                write!(f, "{name} can't be used: {problem}")
            }
            AnchorError::Loop { names } => {
                let (base, rest) = names.split_first().expect("a loop names two calendars");
                let (anchored, between) = rest.split_last().expect("a loop names two calendars");
                write!(f, "{base} follows {anchored}")?;
                if !between.is_empty() {
                    write!(f, " through {}", list(between))?;
                }
                write!(f, ", so {anchored} can't be anchored to it.")
            }
            AnchorError::BaseDate(e) | AnchorError::Date(e) => write!(f, "{e}"),
            AnchorError::OutOfRange => write!(
                f,
                "This anchor would put the calendar beyond the timeline's range."
            ),
        }
    }
}

impl std::error::Error for AnchorError {}

/// Why a calendar can't be placed in time (data-model.md, "Placement").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementProblem {
    Length(RecordError),
    Definition(DefinitionError),
    /// Its anchor date isn't a date of its calendar.
    Date {
        error: CalendarError,
    },
    /// Its base date isn't a date of its base calendar, `name`.
    BaseDate {
        name: String,
        error: CalendarError,
    },
    /// Its anchor keeps an equation with `name`, which can't be placed.
    BaseUnusable {
        name: String,
    },
    /// Calendars that follow each other in a loop, in order (only from a file written by
    /// hand: keeping a calendar refuses it).
    Loop {
        names: Vec<String>,
    },
    /// A fixed anchor with no moment (research R6).
    NoMoment,
    OutOfRange,
}

impl fmt::Display for PlacementProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlacementProblem::Length(e) => write!(f, "{e}"),
            PlacementProblem::Definition(e) => write!(f, "{e}"),
            PlacementProblem::Date { error } => write!(f, "its anchoring date: {error}"),
            PlacementProblem::BaseDate { name, error } => {
                write!(f, "its anchoring date in {name}: {error}")
            }
            PlacementProblem::BaseUnusable { name } => {
                write!(f, "it's anchored to {name}, which can't be placed")
            }
            PlacementProblem::Loop { names } => {
                write!(f, "{} follow each other in a loop", list(names))
            }
            PlacementProblem::NoMoment => write!(f, "its anchor has no moment: anchor it again"),
            PlacementProblem::OutOfRange => {
                write!(f, "its anchor puts it beyond the timeline's range")
            }
        }
    }
}

impl std::error::Error for PlacementProblem {}

/// "A", "A and B", "A, B and C".
fn list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// A calendar and where it is in time.
#[derive(Clone, Debug)]
pub struct Placement {
    pub stored: StoredCalendar,
    /// The engine's calendar with its epoch worked out, or why it can't be placed.
    pub placed: Result<Calendar, PlacementProblem>,
}

impl OpenUniverse {
    /// Places every calendar, in list order (research R4).
    pub fn placements(&self) -> Result<Vec<Placement>, StoreError> {
        Ok(place_all(self.calendars()?))
    }

    /// The names of the calendars whose anchor keeps an equation with calendar `id`: its
    /// followers, in list order.
    pub fn followers(&self, id: i64) -> Result<Vec<String>, StoreError> {
        Ok(self
            .calendars()?
            .into_iter()
            .filter(|c| {
                c.record
                    .anchor
                    .as_ref()
                    .is_some_and(|a| a.kind == AnchorKind::Equation && a.base == Some(id))
            })
            .map(|c| c.record.name)
            .collect())
    }

    /// Checks the anchor of `record`, to be kept as calendar `id` (`None` for a new one),
    /// against the universe as stored (research R5, steps 5 to 9), and returns the `at` to
    /// store: `Some` for a fixed anchor, `None` for an equation or no anchor.
    ///
    /// The record's definition is assumed to have passed the engine already (the app checks
    /// it first); if it hasn't, the anchor date isn't checked against it.
    pub fn check_anchor(
        &self,
        id: Option<i64>,
        record: &CalendarRecord,
    ) -> Result<Option<Instant>, AnchorError> {
        let store = |_: StoreError| AnchorError::BaseMissing;
        let placements = self.placements().map_err(store)?;
        let stored = id.and_then(|id| placements.iter().find(|p| p.stored.id == id));
        let stored_anchor = stored.and_then(|p| p.stored.record.anchor.as_ref());

        // Step 5: completeness.
        let Some(anchor) = &record.anchor else {
            let may_stay = match id {
                None => placements.is_empty(),
                Some(_) => stored.is_some() && stored_anchor.is_none(),
            };
            return if may_stay {
                Ok(None)
            } else {
                Err(AnchorError::Missing)
            };
        };
        let unchanged = stored_anchor.is_some_and(|s| {
            s.kind == anchor.kind
                && s.base == anchor.base
                && s.date == anchor.date
                && s.base_date == anchor.base_date
        });
        if anchor.date.is_empty() {
            return Err(AnchorError::DateEmpty {
                side: AnchorSide::Own,
                unit: Some(record.era.unit.trim().to_owned()),
            });
        }
        let base = match anchor.base {
            Some(base) if Some(base) == id => return Err(AnchorError::BaseSelf),
            Some(base) => Some(
                placements
                    .iter()
                    .find(|p| p.stored.id == base)
                    .ok_or(AnchorError::BaseMissing)?,
            ),
            // A fixed anchor whose base calendar was deleted may be kept as it is.
            None if unchanged && anchor.kind == AnchorKind::Fixed => None,
            None => return Err(AnchorError::BaseMissing),
        };
        if anchor.base_date.is_empty() {
            return Err(AnchorError::DateEmpty {
                side: AnchorSide::Base,
                unit: base.map(|b| b.stored.record.era.unit.clone()),
            });
        }

        // Step 7 (before 6, so a base that can't be placed because it follows this calendar
        // is reported as the loop it is): the base must not follow the anchored calendar.
        if let (Some(id), Some(base)) = (id, base)
            && let Some(chain) = following_chain(&placements, base.stored.id, id)
        {
            let mut names = chain;
            names.push(record.name.trim().to_owned());
            return Err(AnchorError::Loop { names });
        }

        // Step 6: the base calendar is placed.
        let base_calendar = match base {
            Some(base) => Some(match &base.placed {
                Ok(calendar) => calendar,
                Err(problem) => {
                    return Err(AnchorError::BaseUnusable {
                        name: base.stored.record.name.clone(),
                        problem: Box::new(problem.clone()),
                    });
                }
            }),
            None => None,
        };

        // Step 8: the instant.
        let kept = stored_anchor.and_then(|s| s.at);
        let at = match (anchor.kind, unchanged, kept, base_calendar) {
            (AnchorKind::Fixed, true, Some(at), _) => at,
            (_, _, _, Some(base)) => {
                base.date_to_instant(&date(&anchor.base_date))
                    .map_err(AnchorError::BaseDate)?
                    .instant
            }
            (_, _, _, None) => return Err(AnchorError::BaseMissing),
        };

        // Step 9: the anchor date at that instant, in the new definition.
        if let Some(calendar) = record.to_def().ok().and_then(|d| Calendar::new(d).ok()) {
            calendar
                .epoch_for(&date(&anchor.date), at)
                .map_err(|e| match e {
                    CalendarError::OutOfRange => AnchorError::OutOfRange,
                    other => AnchorError::Date(other),
                })?;
        }
        Ok(match anchor.kind {
            AnchorKind::Fixed => Some(at),
            AnchorKind::Equation => None,
        })
    }
}

/// If calendar `from` follows calendar `target` (through equation anchors), the names from
/// `from` up to, but not including, `target`.
fn following_chain(placements: &[Placement], from: i64, target: i64) -> Option<Vec<String>> {
    let by_id: HashMap<i64, &Placement> = placements.iter().map(|p| (p.stored.id, p)).collect();
    let mut names = Vec::new();
    let mut current = from;
    // At most one step per calendar; a loop not through `target` ends the walk.
    for _ in 0..=placements.len() {
        let p = by_id.get(&current)?;
        names.push(p.stored.record.name.clone());
        let anchor = p.stored.record.anchor.as_ref()?;
        if anchor.kind != AnchorKind::Equation {
            return None;
        }
        let next = anchor.base?;
        if next == target {
            return Some(names);
        }
        current = next;
    }
    None
}

fn date(values: &[i64]) -> Date {
    Date {
        values: values.to_vec(),
    }
}

/// Where the walk in [`place_all`] has got to with each calendar.
#[allow(clippy::large_enum_variant)] // one per calendar, briefly
enum State {
    Unvisited,
    OnPath,
    Done(Result<Calendar, PlacementProblem>),
}

/// Places every calendar (research R4): the reference calendar at the hidden zero, fixed
/// anchors from their `at`, and equation anchors through their base, depth first, so a
/// calendar met again on the current path is a loop.
#[allow(clippy::result_large_err)] // the problem is as large as the engine's own errors
fn place_all(stored: Vec<StoredCalendar>) -> Vec<Placement> {
    let index: HashMap<i64, usize> = stored.iter().enumerate().map(|(i, c)| (c.id, i)).collect();
    // Each definition at the hidden zero, or why it's refused.
    let defs: Vec<Result<Calendar, PlacementProblem>> = stored
        .iter()
        .map(|c| {
            let def = c.record.to_def().map_err(PlacementProblem::Length)?;
            Calendar::new(def).map_err(PlacementProblem::Definition)
        })
        .collect();
    let mut states: Vec<State> = stored.iter().map(|_| State::Unvisited).collect();
    for i in 0..stored.len() {
        let mut path = Vec::new();
        place(i, &stored, &index, &defs, &mut states, &mut path);
    }
    stored
        .into_iter()
        .zip(states)
        .map(|(stored, state)| Placement {
            stored,
            placed: match state {
                State::Done(placed) => placed,
                State::Unvisited | State::OnPath => {
                    unreachable!("every calendar is placed by the walk")
                }
            },
        })
        .collect()
}

#[allow(clippy::result_large_err)]
fn place(
    i: usize,
    stored: &[StoredCalendar],
    index: &HashMap<i64, usize>,
    defs: &[Result<Calendar, PlacementProblem>],
    states: &mut [State],
    path: &mut Vec<usize>,
) {
    if !matches!(states[i], State::Unvisited) {
        return;
    }
    let calendar = match &defs[i] {
        Ok(calendar) => calendar,
        Err(problem) => {
            states[i] = State::Done(Err(problem.clone()));
            return;
        }
    };
    let Some(anchor) = &stored[i].record.anchor else {
        states[i] = State::Done(Ok(calendar.clone()));
        return;
    };
    let at = match anchor.kind {
        AnchorKind::Fixed => anchor.at.ok_or(PlacementProblem::NoMoment),
        AnchorKind::Equation => {
            let Some(&b) = anchor.base.and_then(|base| index.get(&base)) else {
                let name = anchor
                    .base_name
                    .clone()
                    .unwrap_or_else(|| "a calendar that no longer exists".to_owned());
                states[i] = State::Done(Err(PlacementProblem::BaseUnusable { name }));
                return;
            };
            if matches!(states[b], State::OnPath) {
                // A loop: every calendar from b to here is on it.
                let start = path.iter().position(|&k| k == b).expect("b is on the path");
                let mut members: Vec<usize> = path[start..].to_vec();
                members.push(i);
                // Name them from the one earliest in the list, so the message doesn't depend
                // on where the walk started.
                let first = members
                    .iter()
                    .enumerate()
                    .min_by_key(|&(_, &k)| k)
                    .map(|(n, _)| n)
                    .expect("a loop has members");
                members.rotate_left(first);
                let names: Vec<String> = members
                    .iter()
                    .map(|&k| stored[k].record.name.clone())
                    .collect();
                for &k in &members {
                    states[k] = State::Done(Err(PlacementProblem::Loop {
                        names: names.clone(),
                    }));
                }
                return;
            }
            states[i] = State::OnPath;
            path.push(i);
            place(b, stored, index, defs, states, path);
            path.pop();
            if let State::Done(_) = states[i] {
                return; // it was on a loop
            }
            let base_name = || stored[b].record.name.clone();
            match &states[b] {
                State::Done(Ok(base)) => base
                    .date_to_instant(&date(&anchor.base_date))
                    .map(|start| start.instant)
                    .map_err(|error| match error {
                        CalendarError::OutOfRange => PlacementProblem::OutOfRange,
                        error => PlacementProblem::BaseDate {
                            name: base_name(),
                            error,
                        },
                    }),
                _ => Err(PlacementProblem::BaseUnusable { name: base_name() }),
            }
        }
    };
    let placed = at.and_then(|at| {
        let epoch = calendar
            .epoch_for(&date(&anchor.date), at)
            .map_err(|error| match error {
                CalendarError::OutOfRange => PlacementProblem::OutOfRange,
                error => PlacementProblem::Date { error },
            })?;
        let mut def = stored[i]
            .record
            .to_def()
            .map_err(PlacementProblem::Length)?;
        def.era.epoch = epoch;
        Calendar::new(def).map_err(PlacementProblem::Definition)
    });
    states[i] = State::Done(placed);
}

// ---- Rows ----------------------------------------------------------------------------------

/// A date as stored: `[412,3,9]`, whole numbers with no spaces (research R3).
pub(crate) fn date_text(values: &[i64]) -> String {
    let inner: Vec<String> = values.iter().map(i64::to_string).collect();
    format!("[{}]", inner.join(","))
}

/// Reads a stored date, strictly: `None` for anything [`date_text`] doesn't write, or an
/// empty date.
pub(crate) fn parse_date_text(text: &str) -> Option<Vec<i64>> {
    let inner = text.strip_prefix('[')?.strip_suffix(']')?;
    inner
        .split(',')
        .map(|v| {
            let digits = v.strip_prefix('-').unwrap_or(v);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            v.parse().ok()
        })
        .collect()
}

fn kind_text(kind: AnchorKind) -> &'static str {
    match kind {
        AnchorKind::Fixed => "fixed",
        AnchorKind::Equation => "equation",
    }
}

/// Writes calendar `id`'s anchor row, replacing any there.
pub(crate) fn write_anchor(
    db: &Connection,
    id: i64,
    anchor: Option<&Anchor>,
) -> rusqlite::Result<()> {
    db.execute("DELETE FROM anchor WHERE calendar_id = ?1", [id])?;
    if let Some(a) = anchor {
        db.execute(
            "INSERT INTO anchor (calendar_id, kind, base_calendar_id, base_name, date, base_date,
                                 at_secs, at_nanos)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                kind_text(a.kind),
                a.base,
                a.base_name,
                date_text(&a.date),
                date_text(&a.base_date),
                a.at.map(Instant::secs),
                a.at.map(Instant::nanos),
            ],
        )?;
    }
    Ok(())
}

/// The stored `base_name` of calendar `id`'s anchor, if any.
pub(crate) fn stored_base_name(db: &Connection, id: i64) -> rusqlite::Result<Option<String>> {
    Ok(db
        .query_row(
            "SELECT base_name FROM anchor WHERE calendar_id = ?1",
            [id],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}

/// Reads calendar `id`'s anchor, or `None` for the reference calendar. A row in the wrong
/// shape is `Err(())`: the file is damaged.
pub(crate) fn read_anchor(db: &Connection, id: i64) -> Result<Option<Anchor>, StoreError> {
    type Row = (
        String,
        Option<i64>,
        Option<String>,
        String,
        String,
        Option<i64>,
        Option<i64>,
    );
    let mut stmt = db.prepare_cached(
        "SELECT kind, base_calendar_id, base_name, date, base_date, at_secs, at_nanos
         FROM anchor WHERE calendar_id = ?1",
    )?;
    let row: Option<Row> = stmt
        .query_row([id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .optional()
        .map_err(|_| damaged())?;
    let Some((kind, base, base_name, date, base_date, secs, nanos)) = row else {
        return Ok(None);
    };
    let kind = match kind.as_str() {
        "fixed" => AnchorKind::Fixed,
        "equation" => AnchorKind::Equation,
        _ => return Err(damaged()),
    };
    let at = match (secs, nanos) {
        (None, None) => None,
        (Some(secs), Some(nanos)) => Some(
            u32::try_from(nanos)
                .ok()
                .and_then(|n| Instant::new(secs, n).ok())
                .ok_or_else(damaged)?,
        ),
        _ => return Err(damaged()),
    };
    Ok(Some(Anchor {
        kind,
        base,
        base_name,
        date: parse_date_text(&date).ok_or_else(damaged)?,
        base_date: parse_date_text(&base_date).ok_or_else(damaged)?,
        at,
    }))
}

/// More than one calendar without an anchor means the file is damaged (research R3).
pub(crate) fn check_one_reference(db: &Connection) -> Result<(), StoreError> {
    let unanchored: i64 = db.query_row(
        "SELECT COUNT(*) FROM calendar WHERE id NOT IN (SELECT calendar_id FROM anchor)",
        [],
        |r| r.get(0),
    )?;
    if unanchored > 1 {
        return Err(damaged());
    }
    Ok(())
}

/// Before calendar `id` is deleted (research R6): its followers' anchors become fixed at the
/// instant their anchor date starts at now (or no moment, if they couldn't be placed), and
/// every anchor based on it keeps its name.
pub(crate) fn before_delete(
    db: &Connection,
    placements: &[Placement],
    id: i64,
) -> rusqlite::Result<()> {
    let Some(deleted) = placements.iter().find(|p| p.stored.id == id) else {
        return Ok(());
    };
    for p in placements {
        let Some(anchor) = &p.stored.record.anchor else {
            continue;
        };
        if anchor.base != Some(id) {
            continue;
        }
        let mut anchor = anchor.clone();
        if anchor.kind == AnchorKind::Equation {
            anchor.kind = AnchorKind::Fixed;
            anchor.at = p
                .placed
                .as_ref()
                .ok()
                .and_then(|c| c.date_to_instant(&date(&anchor.date)).ok())
                .map(|start| start.instant);
        }
        anchor.base_name = Some(deleted.stored.record.name.clone());
        write_anchor(db, p.stored.id, Some(&anchor))?;
    }
    Ok(())
}

fn damaged() -> StoreError {
    crate::universe::not_ours("it's damaged")
}

#[cfg(test)]
mod tests;
