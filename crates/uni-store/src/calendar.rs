//! Calendars in the project file (spec 007): a calendar as the owner entered it, the rows it's
//! stored in, and the one path from stored text to a [`CalendarDef`] that the editor and the
//! file share (specs/007-calendars-in-project/research.md, R1, R2 and R9).

use std::fmt;

use rusqlite::{Connection, OptionalExtension, params};
use uni_calendar::{
    AdjustmentRuleDef, CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, MemberDef,
    MemberUnitDef, SubdivisionDef,
};
use uni_time::{Instant, Ratio, TimeError};

use crate::universe::not_ours;
use crate::{MAX_NAME_CHARS, OpenUniverse, StoreError};

/// A calendar exactly as the owner entered it: the engine's [`CalendarDef`], except that the
/// fundamental unit's length is the text typed and there's no epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarRecord {
    /// Trimmed; 1 to 200 characters; unique in the universe.
    pub name: String,
    /// Top to bottom; may be empty.
    pub groupings: Vec<GroupingDef>,
    pub fundamental: FundamentalRecord,
    /// Top to bottom; may be empty.
    pub subdivisions: Vec<SubdivisionDef>,
    pub era: EraRecord,
}

/// The fundamental unit, with its length in SI seconds as the text typed (`"88775.244"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundamentalRecord {
    pub name: String,
    /// A whole number, decimal or fraction, read exactly by [`Ratio::parse`].
    pub length: String,
    pub first_number: Option<i64>,
}

/// The calendar's one era. Its epoch is the fixed hidden point until spec 008.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EraRecord {
    pub name: String,
    /// The unit it numbers.
    pub unit: String,
}

/// Why a record can't become a [`CalendarDef`] at all. Everything else is the engine's to
/// refuse, with a [`uni_calendar::DefinitionError`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// The fundamental unit's length isn't an exact number that fits.
    Length { text: String, error: TimeError },
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::Length { text, error } => {
                write!(f, "the fundamental unit's length \"{text}\" is {error}")
            }
        }
    }
}

impl std::error::Error for RecordError {}

/// A calendar read from a universe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredCalendar {
    /// Stable for the calendar's life, across edits.
    pub id: i64,
    pub record: CalendarRecord,
}

impl CalendarRecord {
    /// The engine's definition: the length read exactly, and the epoch at the fixed hidden
    /// point (`Instant::ZERO`) until spec 008 adds anchoring. [`uni_calendar::Calendar::new`]
    /// does the rest of the checking.
    pub fn to_def(&self) -> Result<CalendarDef, RecordError> {
        let length =
            Ratio::parse(&self.fundamental.length).map_err(|error| RecordError::Length {
                text: self.fundamental.length.clone(),
                error,
            })?;
        Ok(CalendarDef {
            name: self.name.clone(),
            clock: Clock::Gct,
            groupings: self.groupings.clone(),
            fundamental: FundamentalDef {
                name: self.fundamental.name.clone(),
                length,
                first_number: self.fundamental.first_number,
            },
            subdivisions: self.subdivisions.clone(),
            era: EraDef {
                name: self.era.name.clone(),
                unit: self.era.unit.clone(),
                epoch: Instant::ZERO,
            },
        })
    }

    /// The record with every name, the rule members and the length trimmed, and member names
    /// that are empty after trimming left out (research R4). Rule conditions are kept exactly.
    pub fn normalised(&self) -> CalendarRecord {
        let trim = |s: &str| s.trim().to_owned();
        let member_name = |name: &Option<String>| {
            name.as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(str::to_owned)
        };
        CalendarRecord {
            name: trim(&self.name),
            groupings: self
                .groupings
                .iter()
                .map(|g| GroupingDef {
                    name: trim(&g.name),
                    member_unit: g.member_unit.as_ref().map(|m| MemberUnitDef {
                        name: trim(&m.name),
                        first_number: m.first_number,
                    }),
                    members: g
                        .members
                        .iter()
                        .map(|m| MemberDef {
                            name: member_name(&m.name),
                            count: m.count,
                        })
                        .collect(),
                    first_number: g.first_number,
                    rules: g
                        .rules
                        .iter()
                        .map(|r| AdjustmentRuleDef {
                            condition: r.condition.clone(),
                            member: trim(&r.member),
                            change: r.change,
                        })
                        .collect(),
                })
                .collect(),
            fundamental: FundamentalRecord {
                name: trim(&self.fundamental.name),
                length: trim(&self.fundamental.length),
                first_number: self.fundamental.first_number,
            },
            subdivisions: self
                .subdivisions
                .iter()
                .map(|s| SubdivisionDef {
                    name: trim(&s.name),
                    parts: s.parts,
                    first_number: s.first_number,
                })
                .collect(),
            era: EraRecord {
                name: trim(&self.era.name),
                unit: trim(&self.era.unit),
            },
        }
    }
}

impl OpenUniverse {
    /// Every calendar, in the universe's order.
    pub fn calendars(&self) -> Result<Vec<StoredCalendar>, StoreError> {
        let mut stmt = self
            .db
            .prepare_cached("SELECT id, name FROM calendar ORDER BY seq")?;
        let heads = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(shape)?;
        heads
            .into_iter()
            .map(|(id, name)| {
                Ok(StoredCalendar {
                    id,
                    record: read_record(&self.db, id, name)?,
                })
            })
            .collect()
    }

    /// One calendar, or [`StoreError::NoSuchCalendar`].
    pub fn calendar(&self, id: i64) -> Result<StoredCalendar, StoreError> {
        let name: String = self
            .db
            .query_row("SELECT name FROM calendar WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(shape)?
            .ok_or(StoreError::NoSuchCalendar { id })?;
        Ok(StoredCalendar {
            id,
            record: read_record(&self.db, id, name)?,
        })
    }

    /// `name` trimmed, if it can name a calendar here: 1 to 200 characters
    /// ([`StoreError::CalendarName`]) and not used by another calendar than `except`
    /// ([`StoreError::CalendarNameTaken`]).
    pub fn check_calendar_name(
        &self,
        name: &str,
        except: Option<i64>,
    ) -> Result<String, StoreError> {
        let name = name.trim();
        if !(1..=MAX_NAME_CHARS).contains(&name.chars().count()) {
            return Err(StoreError::CalendarName);
        }
        let taken: Option<i64> = self
            .db
            .query_row(
                "SELECT id FROM calendar WHERE name = ?1 AND id IS NOT ?2",
                params![name, except],
                |r| r.get(0),
            )
            .optional()?;
        match taken {
            Some(_) => Err(StoreError::CalendarNameTaken {
                name: name.to_owned(),
            }),
            None => Ok(name.to_owned()),
        }
    }

    /// Adds a calendar at the end of the list and returns its id. The record is
    /// [normalised](CalendarRecord::normalised) and its name checked; the definition isn't
    /// (the caller checks it with the engine first).
    pub fn add_calendar(&mut self, record: &CalendarRecord) -> Result<i64, StoreError> {
        let record = record.normalised();
        self.check_calendar_name(&record.name, None)?;
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO calendar (seq, name)
             VALUES ((SELECT COALESCE(MAX(seq), 0) + 1 FROM calendar), ?1)",
            [&record.name],
        )?;
        let id = tx.last_insert_rowid();
        write_rows(&tx, id, &record)?;
        tx.commit()?;
        self.dirty = true;
        Ok(id)
    }

    /// Replaces a calendar's definition in one transaction, keeping its id and place in the
    /// list. The same checks as [`add_calendar`](OpenUniverse::add_calendar), except that it
    /// may keep its own name.
    pub fn replace_calendar(&mut self, id: i64, record: &CalendarRecord) -> Result<(), StoreError> {
        let record = record.normalised();
        self.calendar_exists(id)?;
        self.check_calendar_name(&record.name, Some(id))?;
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE calendar SET name = ?1 WHERE id = ?2",
            params![record.name, id],
        )?;
        // Members and rules go with their units, by cascade.
        tx.execute("DELETE FROM unit WHERE calendar_id = ?1", [id])?;
        tx.execute("DELETE FROM era WHERE calendar_id = ?1", [id])?;
        write_rows(&tx, id, &record)?;
        tx.commit()?;
        self.dirty = true;
        Ok(())
    }

    /// Deletes a calendar and, by cascade, everything stored with it.
    pub fn delete_calendar(&mut self, id: i64) -> Result<(), StoreError> {
        if self
            .db
            .execute("DELETE FROM calendar WHERE id = ?1", [id])?
            == 0
        {
            return Err(StoreError::NoSuchCalendar { id });
        }
        self.dirty = true;
        Ok(())
    }

    fn calendar_exists(&self, id: i64) -> Result<(), StoreError> {
        self.db
            .query_row("SELECT 1 FROM calendar WHERE id = ?1", [id], |_| Ok(()))
            .optional()?
            .ok_or(StoreError::NoSuchCalendar { id })
    }
}

/// Writes a record's units, members, rules and era as rows of calendar `id` (research R2).
fn write_rows(db: &Connection, id: i64, record: &CalendarRecord) -> rusqlite::Result<()> {
    let mut unit = db.prepare_cached(
        "INSERT INTO unit (calendar_id, seq, kind, name, first_number, member_unit_name,
                           member_unit_first_number, length, parts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    let mut member = db.prepare_cached(
        "INSERT INTO unit_member (unit_id, seq, name, count) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut rule = db.prepare_cached(
        "INSERT INTO rule (unit_id, seq, condition_src, member, change)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let none_text = None::<&str>;
    let none_int = None::<i64>;
    let mut seq: i64 = 0;
    for g in &record.groupings {
        seq += 1;
        let unit_id = unit.insert(params![
            id,
            seq,
            "grouping",
            g.name,
            g.first_number,
            g.member_unit.as_ref().map(|m| &m.name),
            g.member_unit.as_ref().and_then(|m| m.first_number),
            none_text,
            none_int,
        ])?;
        for (j, m) in (1i64..).zip(&g.members) {
            member.execute(params![unit_id, j, m.name, m.count])?;
        }
        for (k, r) in (1i64..).zip(&g.rules) {
            rule.execute(params![unit_id, k, r.condition, r.member, r.change])?;
        }
    }
    let f = &record.fundamental;
    seq += 1;
    unit.execute(params![
        id,
        seq,
        "fundamental",
        f.name,
        f.first_number,
        none_text,
        none_int,
        f.length,
        none_int,
    ])?;
    for s in &record.subdivisions {
        seq += 1;
        unit.execute(params![
            id,
            seq,
            "subdivision",
            s.name,
            s.first_number,
            none_text,
            none_int,
            none_text,
            s.parts,
        ])?;
    }
    db.execute(
        "INSERT INTO era (calendar_id, seq, name, unit_name) VALUES (?1, 1, ?2, ?3)",
        params![id, record.era.name, record.era.unit],
    )?;
    Ok(())
}

/// One stored unit row.
struct UnitRow {
    id: i64,
    kind: String,
    name: String,
    first_number: Option<i64>,
    member_unit_name: Option<String>,
    member_unit_first_number: Option<i64>,
    length: Option<String>,
    parts: Option<i64>,
}

/// Reads calendar `id`'s rows back into a record, or reports the file as damaged when they
/// aren't in the shape [`write_rows`] leaves them (data model, "Tables").
fn read_record(db: &Connection, id: i64, name: String) -> Result<CalendarRecord, StoreError> {
    let mut stmt = db.prepare_cached(
        "SELECT id, kind, name, first_number, member_unit_name, member_unit_first_number,
                length, parts
         FROM unit WHERE calendar_id = ?1 ORDER BY seq",
    )?;
    let units = stmt
        .query_map([id], |r| {
            Ok(UnitRow {
                id: r.get(0)?,
                kind: r.get(1)?,
                name: r.get(2)?,
                first_number: r.get(3)?,
                member_unit_name: r.get(4)?,
                member_unit_first_number: r.get(5)?,
                length: r.get(6)?,
                parts: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(shape)?;

    // Groupings, then exactly one fundamental unit, then subdivisions; only groupings have
    // members or rules.
    let mut groupings = Vec::new();
    let mut fundamental = None;
    let mut subdivisions = Vec::new();
    for u in units {
        let members = read_members(db, u.id)?;
        let rules = read_rules(db, u.id)?;
        let has_children = !members.is_empty() || !rules.is_empty();
        match (u.kind.as_str(), fundamental.is_some()) {
            ("grouping", false) => {
                let member_unit = match (u.member_unit_name, u.member_unit_first_number) {
                    (Some(name), first_number) => Some(MemberUnitDef { name, first_number }),
                    (None, None) => None,
                    (None, Some(_)) => return Err(damaged()),
                };
                groupings.push(GroupingDef {
                    name: u.name,
                    member_unit,
                    members,
                    first_number: u.first_number,
                    rules,
                });
            }
            ("fundamental", false) if !has_children => {
                fundamental = Some(FundamentalRecord {
                    name: u.name,
                    length: u.length.ok_or_else(damaged)?,
                    first_number: u.first_number,
                });
            }
            ("subdivision", true) if !has_children => subdivisions.push(SubdivisionDef {
                name: u.name,
                parts: u.parts.ok_or_else(damaged)?,
                first_number: u.first_number,
            }),
            _ => return Err(damaged()),
        }
    }

    let mut stmt =
        db.prepare_cached("SELECT name, unit_name FROM era WHERE calendar_id = ?1 ORDER BY seq")?;
    let mut eras = stmt
        .query_map([id], |r| {
            Ok(EraRecord {
                name: r.get(0)?,
                unit: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(shape)?;
    if eras.len() != 1 {
        return Err(damaged());
    }
    Ok(CalendarRecord {
        name,
        groupings,
        fundamental: fundamental.ok_or_else(damaged)?,
        subdivisions,
        era: eras.remove(0),
    })
}

fn read_members(db: &Connection, unit_id: i64) -> Result<Vec<MemberDef>, StoreError> {
    let mut stmt =
        db.prepare_cached("SELECT name, count FROM unit_member WHERE unit_id = ?1 ORDER BY seq")?;
    stmt.query_map([unit_id], |r| {
        Ok(MemberDef {
            name: r.get(0)?,
            count: r.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()
    .map_err(shape)
}

fn read_rules(db: &Connection, unit_id: i64) -> Result<Vec<AdjustmentRuleDef>, StoreError> {
    let mut stmt = db.prepare_cached(
        "SELECT condition_src, member, change FROM rule WHERE unit_id = ?1 ORDER BY seq",
    )?;
    stmt.query_map([unit_id], |r| {
        Ok(AdjustmentRuleDef {
            condition: r.get(0)?,
            member: r.get(1)?,
            change: r.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()
    .map_err(shape)
}

fn damaged() -> StoreError {
    not_ours("it's damaged")
}

/// A value of the wrong type in a calendar row means the file is damaged; anything else is
/// an ordinary database error.
fn shape(error: rusqlite::Error) -> StoreError {
    use rusqlite::Error as E;
    match error {
        E::InvalidColumnType(..)
        | E::FromSqlConversionFailure(..)
        | E::IntegralValueOutOfRange(..) => damaged(),
        other => other.into(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::UniverseName;
    use uni_calendar::Calendar;

    const SPANS: [(&str, i64); 12] = [
        ("Prime", 56),
        ("Second", 56),
        ("Third", 55),
        ("Fourth", 56),
        ("Fifth", 56),
        ("Sixth", 55),
        ("Seventh", 56),
        ("Eighth", 56),
        ("Ninth", 55),
        ("Tenth", 56),
        ("Eleventh", 56),
        ("Twelfth", 55),
    ];
    pub(crate) const ASTER_RULE: &str = "orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)";

    fn one_member(name: &str, count: i64) -> GroupingDef {
        GroupingDef {
            name: name.to_owned(),
            member_unit: None,
            members: vec![MemberDef { name: None, count }],
            first_number: None,
            rules: vec![],
        }
    }

    fn subdivision(name: &str, parts: i64) -> SubdivisionDef {
        SubdivisionDef {
            name: name.to_owned(),
            parts,
            first_number: None,
        }
    }

    /// The quickstart's Aster, with its adjustment rule.
    pub(crate) fn aster() -> CalendarRecord {
        CalendarRecord {
            name: "Aster".to_owned(),
            groupings: vec![
                one_member("age", 10),
                one_member("cohort", 10),
                GroupingDef {
                    name: "orbit".to_owned(),
                    member_unit: Some(MemberUnitDef {
                        name: "span".to_owned(),
                        first_number: None,
                    }),
                    members: SPANS
                        .iter()
                        .map(|&(name, count)| MemberDef {
                            name: Some(name.to_owned()),
                            count,
                        })
                        .collect(),
                    first_number: None,
                    rules: vec![AdjustmentRuleDef {
                        condition: ASTER_RULE.to_owned(),
                        member: "Twelfth".to_owned(),
                        change: 1,
                    }],
                },
            ],
            fundamental: FundamentalRecord {
                name: "sol".to_owned(),
                length: "88775.244".to_owned(),
                first_number: None,
            },
            subdivisions: vec![subdivision("bell", 20), subdivision("pulse", 100)],
            era: EraRecord {
                name: "After Landing".to_owned(),
                unit: "orbit".to_owned(),
            },
        }
    }

    /// A small valid calendar called `name`.
    pub(crate) fn small(name: &str) -> CalendarRecord {
        CalendarRecord {
            name: name.to_owned(),
            groupings: vec![],
            fundamental: FundamentalRecord {
                name: "unit".to_owned(),
                length: "1000".to_owned(),
                first_number: None,
            },
            subdivisions: vec![],
            era: EraRecord {
                name: "Era".to_owned(),
                unit: "unit".to_owned(),
            },
        }
    }

    pub(crate) fn universe() -> OpenUniverse {
        OpenUniverse::new(UniverseName::new("Test").unwrap(), "0.1.0").unwrap()
    }

    fn count(universe: &OpenUniverse, table: &str) -> i64 {
        universe
            .db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    fn damaged() -> StoreError {
        StoreError::NotAUniverseFile {
            reason: "it's damaged".to_owned(),
        }
    }

    // ---- to_def (T005) ----

    #[test]
    fn aster_converts_to_the_engines_aster_at_the_hidden_epoch() {
        let def = aster().to_def().unwrap();
        assert_eq!(def.name, "Aster");
        assert_eq!(def.clock, Clock::Gct);
        assert_eq!(def.groupings, aster().groupings);
        assert_eq!(
            def.fundamental,
            FundamentalDef {
                name: "sol".to_owned(),
                length: Ratio::new(88_775_244, 1000).unwrap(),
                first_number: None,
            }
        );
        assert_eq!(def.subdivisions, aster().subdivisions);
        assert_eq!(
            def.era,
            EraDef {
                name: "After Landing".to_owned(),
                unit: "orbit".to_owned(),
                epoch: Instant::ZERO,
            }
        );
        let calendar = Calendar::new(def).unwrap();
        assert_eq!(
            calendar.units(),
            ["age", "cohort", "orbit", "span", "sol", "bell", "pulse"]
        );
    }

    #[test]
    fn a_bad_length_is_a_record_error_with_its_text() {
        let mut record = aster();
        record.fundamental.length = "88775,244".to_owned();
        let error = record.to_def().unwrap_err();
        assert_eq!(
            error,
            RecordError::Length {
                text: "88775,244".to_owned(),
                error: TimeError::NotANumber,
            }
        );
        assert_eq!(
            error.to_string(),
            "the fundamental unit's length \"88775,244\" is not a whole number, decimal or fraction"
        );
        record.fundamental.length = "1/0".to_owned();
        assert!(matches!(
            record.to_def(),
            Err(RecordError::Length {
                error: TimeError::DivisionByZero,
                ..
            })
        ));
    }

    #[test]
    fn optional_fields_stay_empty_and_set_ones_are_kept() {
        let def = small("Plain").to_def().unwrap();
        assert_eq!(def.fundamental.first_number, None);
        let mut record = aster();
        record.era.unit = "age".to_owned();
        record.groupings[2].first_number = Some(0);
        record.subdivisions[0].first_number = Some(3);
        let def = record.to_def().unwrap();
        assert_eq!(def.groupings[2].first_number, Some(0));
        assert_eq!(def.subdivisions[0].first_number, Some(3));
        assert_eq!(def.groupings[0].first_number, None);
        assert_eq!(
            def.fundamental.length,
            Ratio::new(88_775_244, 1000).unwrap()
        );
    }

    #[test]
    fn normalising_trims_names_and_drops_blank_member_names() {
        let mut record = aster();
        record.name = "  Aster ".to_owned();
        record.groupings[0].name = " age".to_owned();
        record.groupings[2].members[0].name = Some(" Prime ".to_owned());
        record.groupings[2].members[1].name = Some("   ".to_owned());
        record.groupings[2].member_unit.as_mut().unwrap().name = "span ".to_owned();
        record.groupings[2].rules[0].member = " Twelfth".to_owned();
        record.groupings[2].rules[0].condition = " orbit % 4 == 0\n".to_owned();
        record.fundamental.name = "\tsol".to_owned();
        record.fundamental.length = " 88775.244 ".to_owned();
        record.subdivisions[1].name = "pulse  ".to_owned();
        record.era = EraRecord {
            name: " After Landing ".to_owned(),
            unit: " orbit".to_owned(),
        };
        let mut expected = aster();
        expected.groupings[2].members[1].name = None;
        expected.groupings[2].rules[0].condition = " orbit % 4 == 0\n".to_owned();
        assert_eq!(record.normalised(), expected);
    }

    // ---- Storing records (T006) ----

    #[test]
    fn aster_round_trips_through_the_store_field_for_field() {
        let mut universe = universe();
        let id = universe.add_calendar(&aster()).unwrap();
        assert_eq!(
            universe.calendar(id).unwrap(),
            StoredCalendar {
                id,
                record: aster()
            }
        );
    }

    #[test]
    fn optional_fields_round_trip_empty_and_set() {
        let mut universe = universe();
        let mut all_set = aster();
        all_set.name = "Set".to_owned();
        all_set.era.unit = "age".to_owned();
        all_set.groupings[1].first_number = Some(-4);
        all_set.groupings[2].first_number = Some(0);
        all_set.groupings[2]
            .member_unit
            .as_mut()
            .unwrap()
            .first_number = Some(7);
        all_set.fundamental.first_number = Some(0);
        all_set.subdivisions[0].first_number = Some(1);
        all_set.subdivisions[1].first_number = Some(-2);
        all_set.groupings[2].members[3].name = None;
        let empty = small("Empty");
        for record in [all_set, empty] {
            let id = universe.add_calendar(&record).unwrap();
            assert_eq!(universe.calendar(id).unwrap().record, record);
        }
    }

    #[test]
    fn rule_conditions_come_back_exactly_as_typed() {
        let mut universe = universe();
        let mut record = aster();
        let condition = "orbit  %  5 == 0\n  && (orbit % 100 != 0 || orbit % 500 == 0)";
        record.groupings[2].rules[0].condition = condition.to_owned();
        let id = universe.add_calendar(&record).unwrap();
        let back = universe.calendar(id).unwrap().record;
        assert_eq!(back.groupings[2].rules[0].condition, condition);
    }

    #[test]
    fn calendars_come_back_in_insertion_order() {
        let mut universe = universe();
        let ids: Vec<i64> = ["Cairn", "Aster", "Bell"]
            .iter()
            .map(|name| universe.add_calendar(&small(name)).unwrap())
            .collect();
        let listed = universe.calendars().unwrap();
        assert_eq!(listed.iter().map(|c| c.id).collect::<Vec<_>>(), ids);
        assert_eq!(
            listed
                .iter()
                .map(|c| c.record.name.as_str())
                .collect::<Vec<_>>(),
            ["Cairn", "Aster", "Bell"]
        );
    }

    #[test]
    fn adding_marks_the_universe_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let mut universe = universe();
        universe.save_as(&dir.path().join("t.universe")).unwrap();
        assert!(!universe.is_dirty());
        universe.add_calendar(&small("Aster")).unwrap();
        assert!(universe.is_dirty());
    }

    #[test]
    fn calendar_names_are_checked() {
        let mut universe = universe();
        for bad in ["", "   ", &"x".repeat(201)] {
            assert_eq!(
                universe.add_calendar(&small(bad)),
                Err(StoreError::CalendarName),
                "{bad:?}"
            );
        }
        universe.add_calendar(&small(&"é".repeat(200))).unwrap();
        universe.add_calendar(&small("Aster")).unwrap();
        assert_eq!(
            universe.add_calendar(&small(" Aster ")),
            Err(StoreError::CalendarNameTaken {
                name: "Aster".to_owned()
            })
        );
        // Exact match only: the engine's rule, with no case folding of its own.
        universe.add_calendar(&small("aster")).unwrap();
        assert_eq!(count(&universe, "calendar"), 3);
    }

    #[test]
    fn names_are_trimmed_when_stored() {
        let mut universe = universe();
        let mut record = aster();
        record.name = " Aster ".to_owned();
        record.groupings[2].members[0].name = Some("  ".to_owned());
        let id = universe.add_calendar(&record).unwrap();
        let back = universe.calendar(id).unwrap().record;
        assert_eq!(back.name, "Aster");
        assert_eq!(back.groupings[2].members[0].name, None);
    }

    #[test]
    fn a_missing_calendar_is_no_such_calendar() {
        let universe = universe();
        assert_eq!(
            universe.calendar(42),
            Err(StoreError::NoSuchCalendar { id: 42 })
        );
    }

    #[test]
    fn rows_in_the_wrong_shape_read_as_damaged() {
        let cases = [
            "DELETE FROM unit WHERE kind = 'fundamental'",
            "INSERT INTO era (calendar_id, seq, name, unit_name) VALUES (1, 2, 'Second', 'orbit')",
            "DELETE FROM era",
            "UPDATE unit SET seq = 100 WHERE name = 'age'",
            "UPDATE unit SET seq = -1 WHERE name = 'pulse'",
            "INSERT INTO unit_member (unit_id, seq, count)
                 VALUES ((SELECT id FROM unit WHERE name = 'sol'), 1, 3)",
            "INSERT INTO rule (unit_id, seq, condition_src, member, change)
                 VALUES ((SELECT id FROM unit WHERE name = 'bell'), 1, 'true', '1', 1)",
            "UPDATE unit_member SET count = 'many' WHERE seq = 1",
        ];
        for sql in cases {
            let mut universe = universe();
            let id = universe.add_calendar(&aster()).unwrap();
            assert_eq!(id, 1);
            universe.db.execute(sql, []).unwrap();
            assert_eq!(universe.calendar(id), Err(damaged()), "{sql}");
            assert_eq!(universe.calendars(), Err(damaged()), "{sql}");
        }
    }

    // ---- Saving and reopening files (T007) ----

    const APP: &str = "0.1.0";

    fn reopen(universe: OpenUniverse, path: &std::path::Path) -> OpenUniverse {
        drop(universe);
        OpenUniverse::open(path, APP).unwrap()
    }

    fn records(universe: &OpenUniverse) -> Vec<CalendarRecord> {
        universe
            .calendars()
            .unwrap()
            .into_iter()
            .map(|c| c.record)
            .collect()
    }

    #[test]
    fn calendars_survive_save_and_reopen_unchanged_and_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aster.universe");
        let mut universe = universe();
        let expected = vec![small("Cairn"), aster(), small("Bell")];
        for record in &expected {
            universe.add_calendar(record).unwrap();
        }
        let ids: Vec<i64> = universe.calendars().unwrap().iter().map(|c| c.id).collect();
        universe.save_as(&path).unwrap();
        let reopened = reopen(universe, &path);
        assert_eq!(records(&reopened), expected);
        let reopened_ids: Vec<i64> = reopened.calendars().unwrap().iter().map(|c| c.id).collect();
        assert_eq!(reopened_ids, ids, "ids are stable");
        assert!(!reopened.is_dirty());
    }

    #[test]
    fn a_version_1_file_opens_with_no_calendars_after_keeping_a_copy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.universe");
        {
            let conn = Connection::open(&path).unwrap();
            crate::schema::create_v0(&conn, "0.0.1", 1_700_000_000).unwrap();
            conn.execute_batch(crate::MIGRATIONS[0]).unwrap();
            conn.execute_batch(
                "UPDATE universe SET name = 'Old Aster'; UPDATE meta SET schema_version = 1;",
            )
            .unwrap();
        }
        let original = std::fs::read(&path).unwrap();
        let universe = OpenUniverse::open(&path, APP).unwrap();
        assert_eq!(universe.name().unwrap().as_str(), "Old Aster");
        assert_eq!(universe.created_at_unix().unwrap(), 1_700_000_000);
        assert!(universe.calendars().unwrap().is_empty());
        assert!(universe.is_dirty());
        drop(universe);
        assert_eq!(
            std::fs::read(dir.path().join("old.v1.universe")).unwrap(),
            original
        );
    }

    #[test]
    fn a_file_with_calendar_rows_in_the_wrong_shape_is_refused_as_damaged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aster.universe");
        let mut universe = universe();
        universe.add_calendar(&aster()).unwrap();
        universe.save_as(&path).unwrap();
        drop(universe);
        Connection::open(&path)
            .unwrap()
            .execute("DELETE FROM era", [])
            .unwrap();
        assert_eq!(OpenUniverse::open(&path, APP).err(), Some(damaged()));
    }

    #[test]
    fn a_thousand_member_grouping_survives_save_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.universe");
        let mut universe = universe();
        let record = big("Big");
        Calendar::new(record.to_def().unwrap()).unwrap();
        universe.add_calendar(&record).unwrap();
        universe.save_as(&path).unwrap();
        assert_eq!(records(&reopen(universe, &path)), [record]);
    }

    /// A calendar whose one grouping has 1,000 members.
    pub(crate) fn big(name: &str) -> CalendarRecord {
        let mut record = small(name);
        record.groupings = vec![GroupingDef {
            name: "orbit".to_owned(),
            member_unit: Some(MemberUnitDef {
                name: "span".to_owned(),
                first_number: None,
            }),
            members: (1..=1000)
                .map(|j| MemberDef {
                    name: (j % 3 != 0).then(|| format!("Member {j}")),
                    count: 1 + j % 7,
                })
                .collect(),
            first_number: None,
            rules: vec![],
        }];
        record.era.unit = "orbit".to_owned();
        record
    }

    /// A small deterministic generator, so one proptest seed makes one whole record.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            // xorshift64*
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        fn range(&mut self, lo: i64, hi: i64) -> i64 {
            lo + (self.next() % (hi - lo + 1) as u64) as i64
        }
        fn coin(&mut self) -> bool {
            self.next().is_multiple_of(2)
        }
        /// Up to `max` letters from a small alphabet, Unicode included.
        fn word(&mut self, alphabet: &[char], max: i64) -> String {
            (0..self.range(0, max))
                .map(|_| alphabet[self.range(0, alphabet.len() as i64 - 1) as usize])
                .collect()
        }
        fn maybe(&mut self, lo: i64, hi: i64) -> Option<i64> {
            self.coin().then(|| self.range(lo, hi))
        }
    }

    const IDENT: [char; 6] = ['a', 'b', 'q', 'x', 'z', '_'];
    const TEXT: [char; 8] = ['a', 'k', 'r', 'é', '✶', 'Ñ', ' ', '-'];

    /// A random record the engine accepts (research R11): 1–4 groupings of 1–20 members,
    /// optional names and first numbers, 0–3 subdivisions, an era on a random grouping, member
    /// unit or the fundamental unit, 0–2 rules on the era's unit when it's a grouping, and a
    /// length written as a whole number, decimal or fraction. Names are already trimmed.
    fn random_record(seed: u64) -> CalendarRecord {
        let r = &mut Rng(seed | 1);
        let text = |r: &mut Rng, prefix: String| {
            let word = r.word(&TEXT, 6);
            format!("{prefix}{}", word.trim_end())
        };
        let mut groupings: Vec<GroupingDef> = (0..r.range(1, 4))
            .map(|i| {
                let members: Vec<MemberDef> = (0..r.range(1, 20))
                    .map(|j| MemberDef {
                        name: r.coin().then(|| text(r, format!("M{j}"))),
                        count: r.range(1, 200),
                    })
                    .collect();
                let member_unit = (members.len() > 1 || r.coin()).then(|| MemberUnitDef {
                    name: format!("m{i}{}", r.word(&IDENT, 4)),
                    first_number: None,
                });
                GroupingDef {
                    name: format!("g{i}{}", r.word(&IDENT, 4)),
                    member_unit,
                    members,
                    first_number: None,
                    rules: vec![],
                }
            })
            .collect();
        let length = match r.range(0, 2) {
            0 => r.range(1, 100_000).to_string(),
            1 => {
                let digits = r.range(1, 3) as usize;
                let frac = r.range(0, 10_i64.pow(digits as u32) - 1);
                format!("{}.{frac:0digits$}", r.range(1, 100_000))
            }
            _ => {
                let den = r.range(1, 1000);
                format!("{}/{den}", r.range(den, 100_000 * den))
            }
        };
        let subdivisions = (0..r.range(0, 3))
            .map(|i| SubdivisionDef {
                name: format!("s{i}{}", r.word(&IDENT, 4)),
                parts: r.range(1, 100),
                first_number: r.maybe(-100, 100),
            })
            .collect();

        // The chain above and at the fundamental unit, as (grouping, is member unit).
        let mut chain: Vec<(Option<usize>, bool)> = Vec::new();
        for (i, g) in groupings.iter().enumerate() {
            chain.push((Some(i), false));
            if g.member_unit.is_some() {
                chain.push((Some(i), true));
            }
        }
        chain.push((None, false));
        let era = r.range(0, chain.len() as i64 - 1) as usize;
        let mut fundamental_first = None;
        // First numbers only below the era's unit, which numbers everything at or above it.
        for &(at, is_member_unit) in &chain[era + 1..] {
            let first = r.maybe(-100, 100);
            match (at, is_member_unit) {
                (Some(i), false) => groupings[i].first_number = first,
                (Some(i), true) => {
                    groupings[i].member_unit.as_mut().unwrap().first_number = first;
                }
                (None, _) => fundamental_first = first,
            }
        }
        let fundamental_name = format!("f{}", r.word(&IDENT, 4));
        let era_unit = match chain[era] {
            (Some(i), false) => {
                let g = &mut groupings[i];
                for _ in 0..r.range(0, 2) {
                    let k = r.range(2, 10);
                    let j = r.range(0, g.members.len() as i64 - 1) as usize;
                    let member = g.members[j]
                        .name
                        .clone()
                        .unwrap_or_else(|| (j + 1).to_string());
                    let condition = format!("{} % {k} == {}", g.name, r.range(0, k - 1));
                    let change = r.range(1, 3);
                    g.rules.push(AdjustmentRuleDef {
                        condition,
                        member,
                        change,
                    });
                }
                g.name.clone()
            }
            (Some(i), true) => groupings[i].member_unit.as_ref().unwrap().name.clone(),
            (None, _) => fundamental_name.clone(),
        };
        CalendarRecord {
            name: text(r, "C".to_owned()),
            groupings,
            fundamental: FundamentalRecord {
                name: fundamental_name,
                length,
                first_number: fundamental_first,
            },
            subdivisions,
            era: EraRecord {
                name: text(r, "E".to_owned()),
                unit: era_unit,
            },
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(1000))]

        /// SC-002: store, save to a file, reopen: the same record, field for field.
        #[test]
        fn random_valid_calendars_survive_save_and_reopen(seed in proptest::prelude::any::<u64>()) {
            let record = random_record(seed);
            let def = record.to_def().unwrap();
            proptest::prop_assert!(Calendar::new(def).is_ok(), "{record:?}");
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("p.universe");
            let mut universe = universe();
            universe.add_calendar(&record).unwrap();
            universe.save_as(&path).unwrap();
            proptest::prop_assert_eq!(records(&reopen(universe, &path)), vec![record]);
        }
    }

    // ---- Calendars the engine refuses (T012, FR-015) ----

    #[test]
    fn stored_calendars_the_engine_refuses_still_read_back_as_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("odd.universe");
        let mut universe = universe();
        let bad_length = universe.add_calendar(&small("Bad length")).unwrap();
        let refused = universe.add_calendar(&aster()).unwrap();
        universe.save_as(&path).unwrap();
        drop(universe);
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute(
                "UPDATE unit SET length = 'about a thousand' WHERE calendar_id = ?1",
                [bad_length],
            )
            .unwrap();
            conn.execute("UPDATE unit_member SET count = 0 WHERE name = 'Third'", [])
                .unwrap();
        }
        let universe = OpenUniverse::open(&path, APP).unwrap();
        let first = universe.calendar(bad_length).unwrap().record;
        assert_eq!(first.fundamental.length, "about a thousand");
        assert!(matches!(first.to_def(), Err(RecordError::Length { .. })));
        let second = universe.calendar(refused).unwrap().record;
        assert_eq!(second.groupings[2].members[2].count, 0);
        assert!(Calendar::new(second.to_def().unwrap()).is_err());
        assert_eq!(universe.calendars().unwrap().len(), 2);
    }

    // ---- Replacing and deleting (T013) ----

    fn row_counts(universe: &OpenUniverse) -> [i64; 5] {
        ["calendar", "unit", "unit_member", "rule", "era"].map(|t| count(universe, t))
    }

    #[test]
    fn replacing_keeps_the_id_and_place_and_every_row_is_replaced() {
        let mut universe = universe();
        let first = universe.add_calendar(&small("First")).unwrap();
        let id = universe.add_calendar(&aster()).unwrap();
        let last = universe.add_calendar(&small("Last")).unwrap();
        // Aster has 6 unit rows (a member unit is part of its grouping's row), 1 + 1 + 12
        // members and 1 rule; each small calendar has 1 unit row.
        assert_eq!(row_counts(&universe), [3, 6 + 2, 14, 1, 3]);

        let mut changed = small("Aster renamed");
        changed.subdivisions = vec![subdivision("tick", 10)];
        universe.replace_calendar(id, &changed).unwrap();
        assert_eq!(row_counts(&universe), [3, 2 + 2, 0, 0, 3], "no orphan rows");
        let listed = universe.calendars().unwrap();
        assert_eq!(
            listed.iter().map(|c| c.id).collect::<Vec<_>>(),
            [first, id, last]
        );
        assert_eq!(listed[1].record, changed);
    }

    #[test]
    fn a_calendar_may_keep_its_own_name_but_not_take_anothers() {
        let mut universe = universe();
        let id = universe.add_calendar(&aster()).unwrap();
        universe.add_calendar(&small("Bell")).unwrap();
        let mut record = aster();
        record.groupings[2].members[0].count = 57;
        universe.replace_calendar(id, &record).unwrap();
        assert_eq!(universe.calendar(id).unwrap().record, record);
        record.name = " Bell".to_owned();
        assert_eq!(
            universe.replace_calendar(id, &record),
            Err(StoreError::CalendarNameTaken {
                name: "Bell".to_owned()
            })
        );
        record.name = String::new();
        assert_eq!(
            universe.replace_calendar(id, &record),
            Err(StoreError::CalendarName)
        );
        assert_eq!(universe.calendar(id).unwrap().record.name, "Aster");
    }

    #[test]
    fn deleting_removes_the_calendar_and_all_its_rows() {
        let mut universe = universe();
        let id = universe.add_calendar(&aster()).unwrap();
        let other = universe.add_calendar(&small("Other")).unwrap();
        universe.delete_calendar(id).unwrap();
        assert_eq!(row_counts(&universe), [1, 1, 0, 0, 1]);
        assert_eq!(
            universe.calendar(id),
            Err(StoreError::NoSuchCalendar { id })
        );
        assert_eq!(universe.calendars().unwrap()[0].id, other);
        assert_eq!(
            universe.delete_calendar(id),
            Err(StoreError::NoSuchCalendar { id })
        );
        assert_eq!(
            universe.replace_calendar(id, &aster()),
            Err(StoreError::NoSuchCalendar { id })
        );
    }

    #[test]
    fn replacing_and_deleting_mark_the_universe_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let mut universe = universe();
        let id = universe.add_calendar(&aster()).unwrap();
        universe.save_as(&dir.path().join("t.universe")).unwrap();
        universe.replace_calendar(id, &aster()).unwrap();
        assert!(universe.is_dirty());
        universe.save().unwrap();
        universe.delete_calendar(id).unwrap();
        assert!(universe.is_dirty());
    }

    #[test]
    fn a_deletion_reaches_the_file_only_when_saved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aster.universe");
        let mut universe = universe();
        let id = universe.add_calendar(&aster()).unwrap();
        universe.save_as(&path).unwrap();
        universe.delete_calendar(id).unwrap();
        // Closing without saving (Story 2, scenario 5).
        let reopened = reopen(universe, &path);
        assert_eq!(records(&reopened), [aster()]);
    }
}
