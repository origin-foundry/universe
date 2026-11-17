//! Calendars for the UI (spec 007): the editor form's shape, checking a form with the engine,
//! and placing each refusal at the field it concerns
//! (specs/007-calendars-in-project/contracts/calendar-commands.md).

use serde::{Deserialize, Serialize};
use uni_calendar::{
    AdjustmentRuleDef, Calendar, DefinitionError, GroupingDef, MemberDef, MemberUnitDef,
    SubdivisionDef,
};
use uni_store::{CalendarRecord, EraRecord, FundamentalRecord, StoreError};

use crate::{Session, UniverseState};

const NO_UNIVERSE: &str = "No universe is open.";

/// The editor form: a [`CalendarRecord`] as JSON, with whole numbers that may be missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarForm {
    pub name: String,
    pub groupings: Vec<GroupingForm>,
    pub fundamental: FundamentalForm,
    pub subdivisions: Vec<SubdivisionForm>,
    pub era: EraForm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupingForm {
    pub name: String,
    pub first_number: Option<i64>,
    pub member_unit: Option<MemberUnitForm>,
    pub members: Vec<MemberForm>,
    pub rules: Vec<RuleForm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberUnitForm {
    pub name: String,
    pub first_number: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberForm {
    pub name: Option<String>,
    pub count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleForm {
    pub condition: String,
    pub member: String,
    pub change: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundamentalForm {
    pub name: String,
    /// In SI seconds, as typed: never turned into a floating-point number.
    pub length: String,
    pub first_number: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubdivisionForm {
    pub name: String,
    pub parts: Option<i64>,
    pub first_number: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraForm {
    pub name: String,
    pub unit: String,
}

/// One calendar in the list. `problem` is the engine's reason when a stored calendar is
/// refused (FR-015).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CalendarSummary {
    pub id: i64,
    pub name: String,
    pub problem: Option<String>,
}

/// An input of the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Field {
    Name,
    Unit,
    FirstNumber,
    MemberUnit,
    Count,
    Condition,
    Member,
    Change,
    Length,
    Parts,
}

/// Where in the form a refusal belongs. Indices count from 0; with no `field`, the message
/// goes at the top of that section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Location {
    CalendarName,
    Era {
        field: Field,
    },
    Grouping {
        index: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<Field>,
    },
    Member {
        grouping: usize,
        index: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<Field>,
    },
    Rule {
        grouping: usize,
        index: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<Field>,
    },
    Fundamental {
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<Field>,
    },
    Subdivision {
        index: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<Field>,
    },
}

/// A refused form: the engine's (or the store's) message, and where it belongs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub message: String,
    pub at: Location,
}

impl FieldError {
    fn new(message: impl Into<String>, at: Location) -> FieldError {
        FieldError {
            message: message.into(),
            at,
        }
    }
}

/// What `put_calendar` returns once the calendar is in the universe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PutResult {
    pub state: UniverseState,
    pub id: i64,
}

/// Why `put_calendar` refused: a fault in the form, or a plain store failure. The UI tells
/// them apart by shape (an object with `at`, or a string).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum PutError {
    Field(FieldError),
    Store(String),
}

impl From<CalendarRecord> for CalendarForm {
    fn from(r: CalendarRecord) -> CalendarForm {
        CalendarForm {
            name: r.name,
            groupings: r
                .groupings
                .into_iter()
                .map(|g| GroupingForm {
                    name: g.name,
                    first_number: g.first_number,
                    member_unit: g.member_unit.map(|m| MemberUnitForm {
                        name: m.name,
                        first_number: m.first_number,
                    }),
                    members: g
                        .members
                        .into_iter()
                        .map(|m| MemberForm {
                            name: m.name,
                            count: Some(m.count),
                        })
                        .collect(),
                    rules: g
                        .rules
                        .into_iter()
                        .map(|r| RuleForm {
                            condition: r.condition,
                            member: r.member,
                            change: Some(r.change),
                        })
                        .collect(),
                })
                .collect(),
            fundamental: FundamentalForm {
                name: r.fundamental.name,
                length: r.fundamental.length,
                first_number: r.fundamental.first_number,
            },
            subdivisions: r
                .subdivisions
                .into_iter()
                .map(|s| SubdivisionForm {
                    name: s.name,
                    parts: Some(s.parts),
                    first_number: s.first_number,
                })
                .collect(),
            era: EraForm {
                name: r.era.name,
                unit: r.era.unit,
            },
        }
    }
}

impl CalendarForm {
    /// The record the form holds, or the first required value that's missing, in form order
    /// (research R8, step 1). Nothing missing is ever filled with a default.
    pub fn to_record(&self) -> Result<CalendarRecord, FieldError> {
        let mut groupings = Vec::with_capacity(self.groupings.len());
        for (g, grouping) in self.groupings.iter().enumerate() {
            let unit = grouping.name.trim();
            let mut members = Vec::with_capacity(grouping.members.len());
            for (j, m) in grouping.members.iter().enumerate() {
                let count = m.count.ok_or_else(|| {
                    let label = m
                        .name
                        .as_deref()
                        .map(str::trim)
                        .filter(|n| !n.is_empty())
                        .map_or_else(|| (j + 1).to_string(), str::to_owned);
                    FieldError::new(
                        format!("member {label} of {unit} needs a count"),
                        Location::Member {
                            grouping: g,
                            index: j,
                            field: Some(Field::Count),
                        },
                    )
                })?;
                members.push(MemberDef {
                    name: m.name.clone(),
                    count,
                });
            }
            let mut rules = Vec::with_capacity(grouping.rules.len());
            for (k, r) in grouping.rules.iter().enumerate() {
                let change = r.change.ok_or_else(|| {
                    FieldError::new(
                        format!("rule {} of {unit} needs a change", k + 1),
                        Location::Rule {
                            grouping: g,
                            index: k,
                            field: Some(Field::Change),
                        },
                    )
                })?;
                rules.push(AdjustmentRuleDef {
                    condition: r.condition.clone(),
                    member: r.member.clone(),
                    change,
                });
            }
            groupings.push(GroupingDef {
                name: grouping.name.clone(),
                member_unit: grouping.member_unit.as_ref().map(|m| MemberUnitDef {
                    name: m.name.clone(),
                    first_number: m.first_number,
                }),
                members,
                first_number: grouping.first_number,
                rules,
            });
        }
        if self.fundamental.length.trim().is_empty() {
            return Err(FieldError::new(
                "the fundamental unit's length is missing",
                Location::Fundamental {
                    field: Some(Field::Length),
                },
            ));
        }
        let mut subdivisions = Vec::with_capacity(self.subdivisions.len());
        for (i, s) in self.subdivisions.iter().enumerate() {
            let parts = s.parts.ok_or_else(|| {
                FieldError::new(
                    format!("{} needs a number of parts", s.name.trim()),
                    Location::Subdivision {
                        index: i,
                        field: Some(Field::Parts),
                    },
                )
            })?;
            subdivisions.push(SubdivisionDef {
                name: s.name.clone(),
                parts,
                first_number: s.first_number,
            });
        }
        Ok(CalendarRecord {
            name: self.name.clone(),
            groupings,
            fundamental: FundamentalRecord {
                name: self.fundamental.name.clone(),
                length: self.fundamental.length.clone(),
                first_number: self.fundamental.first_number,
            },
            subdivisions,
            era: EraRecord {
                name: self.era.name.clone(),
                unit: self.era.unit.clone(),
            },
        })
    }
}

/// The engine's verdict on a stored record: `None` if it's a usable calendar, or the reason.
pub fn problem(record: &CalendarRecord) -> Option<String> {
    match record.to_def() {
        Err(error) => Some(error.to_string()),
        Ok(def) => Calendar::new(def).err().map(|error| error.to_string()),
    }
}

/// Where in `record` (normalised, so in the same order as the form) a refusal belongs
/// (contract, "Where each DefinitionError is placed").
///
/// The engine names units and members; this finds them in the form. A name that matches
/// nothing (which can't happen for a form the engine has just read) falls back to the
/// calendar's name rather than panicking. The match has no catch-all arm, so a new engine
/// error doesn't compile until it has a place.
pub fn locate(error: &DefinitionError, record: &CalendarRecord) -> Location {
    use DefinitionError as E;
    let at = |r: &str| record.groupings.iter().position(|g| g.name == r);
    let member = |unit: &str, label: &str, field: Field| {
        let g = at(unit)?;
        let j = (0..record.groupings[g].members.len())
            .find(|&j| member_label(&record.groupings[g], j) == label)?;
        Some(Location::Member {
            grouping: g,
            index: j,
            field: Some(field),
        })
    };
    let rule = |unit: &str, rule: usize, field: Field| {
        let g = at(unit)?;
        (1..=record.groupings[g].rules.len())
            .contains(&rule)
            .then_some(Location::Rule {
                grouping: g,
                index: rule - 1,
                field: Some(field),
            })
    };
    let grouping = |unit: &str, field: Option<Field>| {
        at(unit).map(|index| Location::Grouping { index, field })
    };
    let fundamental = |field: Field| Some(Location::Fundamental { field: Some(field) });
    let era = |field: Field| Some(Location::Era { field });

    let found = match error {
        E::EmptyName { what } => match *what {
            "calendar" => Some(Location::CalendarName),
            "era" => era(Field::Name),
            "unit" => chain(record)
                .into_iter()
                .find(|(_, name)| name.is_empty())
                .map(|(unit, _)| unit.name_location()),
            "member" => record
                .groupings
                .iter()
                .enumerate()
                .find_map(|(g, grouping)| {
                    let j = grouping
                        .members
                        .iter()
                        .position(|m| m.name.as_deref() == Some(""))?;
                    Some(Location::Member {
                        grouping: g,
                        index: j,
                        field: Some(Field::Name),
                    })
                }),
            _ => None,
        },
        E::NonPositiveLength { .. } => fundamental(Field::Length),
        E::NoMembers { unit } => grouping(unit, None),
        E::NonPositiveCount { unit, member: m }
        | E::MemberTooShort {
            unit, member: m, ..
        } => member(unit, m, Field::Count),
        E::MembersNeedUnit { unit } => grouping(unit, Some(Field::MemberUnit)),
        E::TooFewParts { unit } => record
            .subdivisions
            .iter()
            .position(|s| &s.name == unit)
            .map(|index| Location::Subdivision {
                index,
                field: Some(Field::Parts),
            }),
        E::DuplicateUnit { name } => chain(record)
            .into_iter()
            .filter(|(_, n)| n == name)
            .nth(1)
            .map(|(unit, _)| unit.name_location()),
        E::DuplicateMember { unit, name } => at(unit).and_then(|g| {
            let j = record.groupings[g]
                .members
                .iter()
                .enumerate()
                .filter(|(_, m)| m.name.as_ref() == Some(name))
                .nth(1)?
                .0;
            Some(Location::Member {
                grouping: g,
                index: j,
                field: Some(Field::Name),
            })
        }),
        E::EraUnitUnknown { .. } | E::EraUnitBelowFundamental { .. } => era(Field::Unit),
        E::FirstNumberNotAllowed { unit } => find_unit(record, unit).map(|u| match u {
            Place::Grouping(index) => Location::Grouping {
                index,
                field: Some(Field::FirstNumber),
            },
            Place::MemberUnit(index) => Location::Grouping {
                index,
                field: Some(Field::MemberUnit),
            },
            Place::Fundamental => Location::Fundamental {
                field: Some(Field::FirstNumber),
            },
            Place::Subdivision(index) => Location::Subdivision {
                index,
                field: Some(Field::FirstNumber),
            },
        }),
        E::BeyondLimits { unit, .. } => find_unit(record, unit).map(|u| match u {
            Place::Grouping(index) => Location::Grouping { index, field: None },
            Place::MemberUnit(index) => Location::Grouping {
                index,
                field: Some(Field::MemberUnit),
            },
            Place::Fundamental => Location::Fundamental {
                field: Some(Field::Length),
            },
            Place::Subdivision(index) => Location::Subdivision { index, field: None },
        }),
        E::RulesNotOnEraUnit { unit } | E::RulesPeriodOverLimit { unit, .. } => {
            grouping(unit, None)
        }
        E::RuleCondition { unit, rule: k, .. }
        | E::RulePeriod { unit, rule: k, .. }
        | E::RuleFails { unit, rule: k, .. } => rule(unit, *k, Field::Condition),
        E::RuleUnknownMember { unit, rule: k, .. } => rule(unit, *k, Field::Member),
        E::RuleZeroChange { unit, rule: k } => rule(unit, *k, Field::Change),
    };
    found.unwrap_or(Location::CalendarName)
}

/// Where the form holds a unit of the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place {
    Grouping(usize),
    MemberUnit(usize),
    Fundamental,
    Subdivision(usize),
}

impl Place {
    /// Where the unit's name is entered.
    fn name_location(self) -> Location {
        match self {
            Place::Grouping(index) => Location::Grouping {
                index,
                field: Some(Field::Name),
            },
            Place::MemberUnit(index) => Location::Grouping {
                index,
                field: Some(Field::MemberUnit),
            },
            Place::Fundamental => Location::Fundamental {
                field: Some(Field::Name),
            },
            Place::Subdivision(index) => Location::Subdivision {
                index,
                field: Some(Field::Name),
            },
        }
    }
}

/// The chain, top to bottom, member units included, as the engine walks it.
fn chain(record: &CalendarRecord) -> Vec<(Place, &str)> {
    let mut units = Vec::new();
    for (i, g) in record.groupings.iter().enumerate() {
        units.push((Place::Grouping(i), g.name.as_str()));
        if let Some(m) = &g.member_unit {
            units.push((Place::MemberUnit(i), m.name.as_str()));
        }
    }
    units.push((Place::Fundamental, record.fundamental.name.as_str()));
    for (i, s) in record.subdivisions.iter().enumerate() {
        units.push((Place::Subdivision(i), s.name.as_str()));
    }
    units
}

fn find_unit(record: &CalendarRecord, name: &str) -> Option<Place> {
    chain(record)
        .into_iter()
        .find(|(_, n)| *n == name)
        .map(|(unit, _)| unit)
}

/// A member's label in the engine's messages: its name, or its number from 1.
fn member_label(grouping: &GroupingDef, j: usize) -> String {
    grouping.members[j]
        .name
        .clone()
        .unwrap_or_else(|| (j + 1).to_string())
}

fn store_error(error: StoreError) -> PutError {
    match error {
        StoreError::CalendarName | StoreError::CalendarNameTaken { .. } => {
            PutError::Field(FieldError::new(error.to_string(), Location::CalendarName))
        }
        other => PutError::Store(other.to_string()),
    }
}

impl Session {
    /// Every calendar in the open universe, in order, each with its problem if the engine
    /// refuses it.
    pub fn list_calendars(&self) -> Result<Vec<CalendarSummary>, String> {
        let slot = self.lock();
        let universe = slot.as_ref().ok_or(NO_UNIVERSE)?;
        let calendars = universe.calendars().map_err(|e| e.to_string())?;
        Ok(calendars
            .into_iter()
            .map(|c| CalendarSummary {
                id: c.id,
                problem: problem(&c.record),
                name: c.record.name,
            })
            .collect())
    }

    /// One stored calendar's form, even when the engine refuses it, so it can be fixed.
    pub fn get_calendar(&self, id: i64) -> Result<CalendarForm, String> {
        let slot = self.lock();
        let universe = slot.as_ref().ok_or(NO_UNIVERSE)?;
        let stored = universe.calendar(id).map_err(|e| e.to_string())?;
        Ok(stored.record.into())
    }

    /// The form a new calendar starts from (research R7, FR-002a): the smallest calendar the
    /// engine accepts once its length is entered, named "Calendar" (or "Calendar 2", … when
    /// that's taken).
    pub fn new_calendar_form(&self) -> Result<CalendarForm, String> {
        let slot = self.lock();
        let taken: Vec<String> = match slot.as_ref() {
            None => vec![],
            Some(u) => u
                .calendars()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|c| c.record.name)
                .collect(),
        };
        let name = (1..)
            .map(|n| match n {
                1 => "Calendar".to_owned(),
                n => format!("Calendar {n}"),
            })
            .find(|name| !taken.contains(name))
            .expect("there is always a free name");
        Ok(CalendarForm {
            name,
            groupings: vec![],
            fundamental: FundamentalForm {
                name: "unit".to_owned(),
                length: String::new(),
                first_number: None,
            },
            subdivisions: vec![],
            era: EraForm {
                name: "Era".to_owned(),
                unit: "unit".to_owned(),
            },
        })
    }

    /// Checks `form` (research R8) and adds it as a new calendar (`id` = `None`) or replaces
    /// calendar `id`. Nothing is stored unless the engine accepts it.
    pub fn put_calendar(
        &self,
        id: Option<i64>,
        form: &CalendarForm,
    ) -> Result<PutResult, PutError> {
        let mut slot = self.lock();
        let universe = slot
            .as_mut()
            .ok_or_else(|| PutError::Store(NO_UNIVERSE.to_owned()))?;
        // 1. Every required value is there.
        let record = form.to_record().map_err(PutError::Field)?.normalised();
        // 2. The calendar's name.
        universe
            .check_calendar_name(&record.name, id)
            .map_err(store_error)?;
        // 3. The length's text.
        let def = record.to_def().map_err(|error| {
            PutError::Field(FieldError::new(
                error.to_string(),
                Location::Fundamental {
                    field: Some(Field::Length),
                },
            ))
        })?;
        // 4. The engine.
        Calendar::new(def).map_err(|error| {
            PutError::Field(FieldError::new(error.to_string(), locate(&error, &record)))
        })?;
        let id = match id {
            None => universe.add_calendar(&record),
            Some(id) => universe.replace_calendar(id, &record).map(|()| id),
        }
        .map_err(store_error)?;
        Ok(PutResult {
            state: Session::describe(&slot),
            id,
        })
    }

    /// Deletes calendar `id`.
    pub fn delete_calendar(&self, id: i64) -> Result<UniverseState, String> {
        let mut slot = self.lock();
        let universe = slot.as_mut().ok_or(NO_UNIVERSE)?;
        universe.delete_calendar(id).map_err(|e| e.to_string())?;
        Ok(Session::describe(&slot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uni_store::UniverseName;

    const ASTER_RULE: &str = "orbit % 5 == 0 && (orbit % 100 != 0 || orbit % 500 == 0)";

    /// Quickstart step 4's Aster, as the UI sends it.
    fn aster_json() -> serde_json::Value {
        let spans = [
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
        let one = |name: &str| {
            json!({ "name": name, "firstNumber": null, "memberUnit": null,
                    "members": [{ "name": null, "count": 10 }], "rules": [] })
        };
        json!({
            "name": "Aster",
            "groupings": [
                one("age"),
                one("cohort"),
                {
                    "name": "orbit", "firstNumber": null,
                    "memberUnit": { "name": "span", "firstNumber": null },
                    "members": spans.iter().map(|(n, c)| json!({ "name": n, "count": c }))
                        .collect::<Vec<_>>(),
                    "rules": [{ "condition": ASTER_RULE, "member": "Twelfth", "change": 1 }]
                }
            ],
            "fundamental": { "name": "sol", "length": "88775.244", "firstNumber": null },
            "subdivisions": [
                { "name": "bell", "parts": 20, "firstNumber": null },
                { "name": "pulse", "parts": 100, "firstNumber": null }
            ],
            "era": { "name": "After Landing", "unit": "orbit" }
        })
    }

    pub(crate) fn aster() -> CalendarForm {
        serde_json::from_value(aster_json()).unwrap()
    }

    pub(crate) fn open_session() -> Session {
        let session = Session::default();
        session.new_universe("Test").unwrap();
        session
    }

    pub(crate) fn refusal(session: &Session, form: &CalendarForm) -> FieldError {
        match session.put_calendar(None, form) {
            Err(PutError::Field(error)) => error,
            other => panic!("expected a field error, got {other:?}"),
        }
    }

    #[test]
    fn the_form_converts_to_and_from_a_record_in_camel_case_json() {
        let form = aster();
        assert_eq!(serde_json::to_value(&form).unwrap(), aster_json());
        let record = form.to_record().unwrap();
        assert_eq!(record.fundamental.length, "88775.244");
        assert_eq!(record.groupings[2].rules[0].condition, ASTER_RULE);
        assert_eq!(record.groupings[2].members[11].count, 55);
        assert_eq!(CalendarForm::from(record), form);
    }

    #[test]
    fn a_new_calendar_starts_as_the_smallest_one_the_engine_accepts() {
        let session = open_session();
        let form = session.new_calendar_form().unwrap();
        assert_eq!(
            serde_json::to_value(&form).unwrap(),
            json!({
                "name": "Calendar",
                "groupings": [],
                "fundamental": { "name": "unit", "length": "", "firstNumber": null },
                "subdivisions": [],
                "era": { "name": "Era", "unit": "unit" }
            })
        );
        let mut filled = form.clone();
        filled.fundamental.length = "1000".to_owned();
        session.put_calendar(None, &filled).unwrap();
        assert_eq!(session.new_calendar_form().unwrap().name, "Calendar 2");
        filled.name = "Calendar 2".to_owned();
        session.put_calendar(None, &filled).unwrap();
        assert_eq!(session.new_calendar_form().unwrap().name, "Calendar 3");
    }

    #[test]
    fn put_adds_a_calendar_and_marks_the_universe_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let session = open_session();
        session
            .save_as(dir.path().join("t.universe").to_str().unwrap())
            .unwrap();
        let put = session.put_calendar(None, &aster()).unwrap();
        assert!(put.state.dirty);
        assert_eq!(
            session.list_calendars().unwrap(),
            [CalendarSummary {
                id: put.id,
                name: "Aster".to_owned(),
                problem: None
            }]
        );
        assert_eq!(session.get_calendar(put.id).unwrap(), aster());
        assert_eq!(
            serde_json::to_value(&put).unwrap()["id"],
            json!(put.id),
            "{{ state, id }}"
        );
    }

    #[test]
    fn missing_required_values_are_refused_at_their_field() {
        let session = open_session();
        let mut form = aster();
        form.groupings[2].members[3].count = None;
        assert_eq!(
            refusal(&session, &form),
            FieldError::new(
                "member Fourth of orbit needs a count",
                Location::Member {
                    grouping: 2,
                    index: 3,
                    field: Some(Field::Count)
                }
            )
        );
        let mut form = aster();
        form.groupings[0].members[0].count = None;
        assert_eq!(
            refusal(&session, &form).message,
            "member 1 of age needs a count"
        );
        let mut form = aster();
        form.groupings[2].rules[0].change = None;
        assert_eq!(
            refusal(&session, &form),
            FieldError::new(
                "rule 1 of orbit needs a change",
                Location::Rule {
                    grouping: 2,
                    index: 0,
                    field: Some(Field::Change)
                }
            )
        );
        let mut form = aster();
        form.fundamental.length = "  ".to_owned();
        assert_eq!(
            refusal(&session, &form),
            FieldError::new(
                "the fundamental unit's length is missing",
                Location::Fundamental {
                    field: Some(Field::Length)
                }
            )
        );
        let mut form = aster();
        form.subdivisions[1].parts = None;
        assert_eq!(
            refusal(&session, &form),
            FieldError::new(
                "pulse needs a number of parts",
                Location::Subdivision {
                    index: 1,
                    field: Some(Field::Parts)
                }
            )
        );
        assert!(session.list_calendars().unwrap().is_empty());
    }

    #[test]
    fn field_errors_and_store_errors_serialise_as_the_contract_says() {
        let error = PutError::Field(FieldError::new(
            "m",
            Location::Rule {
                grouping: 2,
                index: 0,
                field: Some(Field::Condition),
            },
        ));
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            json!({ "message": "m",
                    "at": { "kind": "rule", "grouping": 2, "index": 0, "field": "condition" } })
        );
        let at = |l: Location| serde_json::to_value(l).unwrap();
        assert_eq!(
            at(Location::CalendarName),
            json!({ "kind": "calendarName" })
        );
        assert_eq!(
            at(Location::Grouping {
                index: 1,
                field: Some(Field::MemberUnit)
            }),
            json!({ "kind": "grouping", "index": 1, "field": "memberUnit" })
        );
        assert_eq!(
            at(Location::Fundamental { field: None }),
            json!({ "kind": "fundamental" })
        );
        assert_eq!(
            at(Location::Era { field: Field::Unit }),
            json!({ "kind": "era", "field": "unit" })
        );
        assert_eq!(
            serde_json::to_value(PutError::Store("No universe is open.".into())).unwrap(),
            json!("No universe is open.")
        );
    }

    #[test]
    fn every_command_but_the_new_form_needs_an_open_universe() {
        let session = Session::default();
        assert_eq!(session.list_calendars().unwrap_err(), NO_UNIVERSE);
        assert_eq!(session.get_calendar(1).unwrap_err(), NO_UNIVERSE);
        assert_eq!(session.delete_calendar(1).unwrap_err(), NO_UNIVERSE);
        assert_eq!(
            session.put_calendar(None, &aster()),
            Err(PutError::Store(NO_UNIVERSE.to_owned()))
        );
        assert_eq!(session.new_calendar_form().unwrap().name, "Calendar");
    }

    #[test]
    fn stored_calendars_the_engine_refuses_are_listed_with_their_problem() {
        // The store keeps records without checking them, so it can write calendars the
        // engine refuses, as a hand-edited file or a future bug might.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("odd.universe");
        let mut universe =
            uni_store::OpenUniverse::new(UniverseName::new("Odd").unwrap(), "0.1.0").unwrap();
        let mut bad_length = aster().to_record().unwrap();
        bad_length.name = "Bad length".to_owned();
        bad_length.fundamental.length = "88775,244".to_owned();
        let mut refused = aster().to_record().unwrap();
        refused.name = "Refused".to_owned();
        refused.groupings[2].rules[0].member = "Thirteenth".to_owned();
        let bad_length_id = universe.add_calendar(&bad_length).unwrap();
        let refused_id = universe.add_calendar(&refused).unwrap();
        let good = universe
            .add_calendar(&aster().to_record().unwrap())
            .unwrap();
        universe.save_as(&path).unwrap();
        drop(universe);

        let session = Session::default();
        session.open(path.to_str().unwrap()).unwrap();
        assert_eq!(
            session.list_calendars().unwrap(),
            [
                CalendarSummary {
                    id: bad_length_id,
                    name: "Bad length".to_owned(),
                    problem: Some(
                        "the fundamental unit's length \"88775,244\" is not a whole number, \
                         decimal or fraction"
                            .to_owned()
                    ),
                },
                CalendarSummary {
                    id: refused_id,
                    name: "Refused".to_owned(),
                    problem: Some(
                        "rule 1 of orbit changes Thirteenth, which orbit doesn't have".to_owned()
                    ),
                },
                CalendarSummary {
                    id: good,
                    name: "Aster".to_owned(),
                    problem: None,
                },
            ]
        );
        let form = session.get_calendar(refused_id).unwrap();
        assert_eq!(form.groupings[2].rules[0].member, "Thirteenth");
        assert_eq!(
            session
                .get_calendar(bad_length_id)
                .unwrap()
                .fundamental
                .length,
            "88775,244"
        );
        // Fixing it through the editor makes it usable again.
        let mut fixed = form;
        fixed.groupings[2].rules[0].member = "Twelfth".to_owned();
        session.put_calendar(Some(refused_id), &fixed).unwrap();
        assert_eq!(session.list_calendars().unwrap()[1].problem, None);
    }

    // ---- Editing and deleting (T014) ----

    fn saved_with_aster(dir: &std::path::Path) -> (Session, i64) {
        let session = open_session();
        let id = session.put_calendar(None, &aster()).unwrap().id;
        session
            .save_as(dir.join("t.universe").to_str().unwrap())
            .unwrap();
        (session, id)
    }

    #[test]
    fn put_with_an_id_replaces_that_calendar() {
        let dir = tempfile::tempdir().unwrap();
        let (session, id) = saved_with_aster(dir.path());
        let mut form = aster();
        form.groupings[2].members[0].count = Some(57);
        form.groupings[2].members[0].name = Some("First".to_owned());
        let put = session.put_calendar(Some(id), &form).unwrap();
        assert_eq!(put.id, id);
        assert!(put.state.dirty);
        assert_eq!(session.get_calendar(id).unwrap(), form);
        assert_eq!(session.list_calendars().unwrap().len(), 1);
    }

    #[test]
    fn a_refused_replacement_leaves_the_stored_calendar_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let (session, id) = saved_with_aster(dir.path());
        let mut form = aster();
        form.groupings[2].members[0].count = Some(0);
        assert!(matches!(
            session.put_calendar(Some(id), &form),
            Err(PutError::Field(_))
        ));
        assert_eq!(session.get_calendar(id).unwrap(), aster());
        assert!(!session.state().dirty);
    }

    #[test]
    fn replacing_or_deleting_a_missing_calendar_is_a_store_error() {
        let session = open_session();
        assert_eq!(
            session.put_calendar(Some(9), &aster()),
            Err(PutError::Store(
                "That calendar no longer exists.".to_owned()
            ))
        );
        assert_eq!(
            session.delete_calendar(9).unwrap_err(),
            "That calendar no longer exists."
        );
    }

    #[test]
    fn delete_removes_the_calendar_and_marks_the_universe_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let (session, id) = saved_with_aster(dir.path());
        let state = session.delete_calendar(id).unwrap();
        assert!(state.dirty);
        assert!(session.list_calendars().unwrap().is_empty());
        assert_eq!(
            session.get_calendar(id).unwrap_err(),
            "That calendar no longer exists."
        );
    }
}

/// One test per `DefinitionError` variant: entered through the form, refused with the
/// engine's own message, placed as the contract's table says, and nothing stored (SC-003).
#[cfg(test)]
mod locations {
    use super::tests::{aster, open_session, refusal};
    use super::*;
    use uni_calendar::Limit;

    fn grouping(index: usize, field: Option<Field>) -> Location {
        Location::Grouping { index, field }
    }
    fn member(grouping: usize, index: usize, field: Field) -> Location {
        Location::Member {
            grouping,
            index,
            field: Some(field),
        }
    }
    fn rule(index: usize, field: Field) -> Location {
        Location::Rule {
            grouping: 2,
            index,
            field: Some(field),
        }
    }
    fn fundamental(field: Field) -> Location {
        Location::Fundamental { field: Some(field) }
    }
    fn subdivision(index: usize, field: Option<Field>) -> Location {
        Location::Subdivision { index, field }
    }

    /// Puts `form`, checks the refusal is the engine's message at `at` and that nothing was
    /// stored, and returns the engine's error so the test can check its variant.
    fn refused_at(form: &CalendarForm, at: Location) -> DefinitionError {
        let session = open_session();
        let record = form.to_record().unwrap().normalised();
        let engine = Calendar::new(record.to_def().unwrap()).unwrap_err();
        assert_eq!(
            refusal(&session, form),
            FieldError {
                message: engine.to_string(),
                at
            },
            "{engine:?}"
        );
        assert!(session.list_calendars().unwrap().is_empty());
        engine
    }

    fn with_rule(condition: &str, member: &str, change: i64) -> CalendarForm {
        let mut form = aster();
        form.groupings[2].rules = vec![RuleForm {
            condition: condition.to_owned(),
            member: member.to_owned(),
            change: Some(change),
        }];
        form
    }

    #[test]
    fn empty_names() {
        let mut form = aster();
        form.era.name = "  ".to_owned();
        let e = refused_at(&form, Location::Era { field: Field::Name });
        assert_eq!(e, DefinitionError::EmptyName { what: "era" });

        let mut form = aster();
        form.groupings[1].name = " ".to_owned();
        let e = refused_at(&form, grouping(1, Some(Field::Name)));
        assert_eq!(e, DefinitionError::EmptyName { what: "unit" });

        let mut form = aster();
        form.groupings[2].member_unit.as_mut().unwrap().name = String::new();
        refused_at(&form, grouping(2, Some(Field::MemberUnit)));

        let mut form = aster();
        form.fundamental.name = String::new();
        refused_at(&form, fundamental(Field::Name));

        let mut form = aster();
        form.subdivisions[1].name = String::new();
        refused_at(&form, subdivision(1, Some(Field::Name)));
    }

    #[test]
    fn empty_calendar_and_member_names_are_placed_too() {
        // The form never reaches the engine with these (the name is checked first, and blank
        // member names become none), but the placement is still total.
        let mut record = aster().to_record().unwrap();
        let calendar = DefinitionError::EmptyName { what: "calendar" };
        assert_eq!(locate(&calendar, &record), Location::CalendarName);
        record.groupings[2].members[4].name = Some(String::new());
        let empty_member = DefinitionError::EmptyName { what: "member" };
        assert_eq!(locate(&empty_member, &record), member(2, 4, Field::Name));
    }

    #[test]
    fn non_positive_length() {
        let mut form = aster();
        form.fundamental.length = "-1".to_owned();
        let e = refused_at(&form, fundamental(Field::Length));
        assert!(matches!(e, DefinitionError::NonPositiveLength { .. }));
    }

    #[test]
    fn no_members() {
        let mut form = aster();
        form.groupings[1].members.clear();
        let e = refused_at(&form, grouping(1, None));
        assert!(matches!(e, DefinitionError::NoMembers { .. }));
    }

    #[test]
    fn non_positive_count() {
        let mut form = aster();
        form.groupings[2].members[2].count = Some(0);
        let e = refused_at(&form, member(2, 2, Field::Count));
        assert!(matches!(e, DefinitionError::NonPositiveCount { .. }));
        // An unnamed member is named by its number.
        let mut form = aster();
        form.groupings[0].members[0].count = Some(-3);
        refused_at(&form, member(0, 0, Field::Count));
    }

    #[test]
    fn members_need_a_unit() {
        let mut form = aster();
        form.groupings[2].member_unit = None;
        let e = refused_at(&form, grouping(2, Some(Field::MemberUnit)));
        assert!(matches!(e, DefinitionError::MembersNeedUnit { .. }));
    }

    #[test]
    fn too_few_parts() {
        let mut form = aster();
        form.subdivisions[0].parts = Some(0);
        let e = refused_at(&form, subdivision(0, Some(Field::Parts)));
        assert!(matches!(e, DefinitionError::TooFewParts { .. }));
    }

    #[test]
    fn duplicate_units_are_placed_at_the_second() {
        let mut form = aster();
        form.subdivisions[1].name = "bell".to_owned();
        let e = refused_at(&form, subdivision(1, Some(Field::Name)));
        assert!(matches!(e, DefinitionError::DuplicateUnit { .. }));

        let mut form = aster();
        form.groupings[1].name = "age".to_owned();
        refused_at(&form, grouping(1, Some(Field::Name)));

        // A member unit named like the grouping above it.
        let mut form = aster();
        form.groupings[2].member_unit.as_mut().unwrap().name = "cohort".to_owned();
        refused_at(&form, grouping(2, Some(Field::MemberUnit)));

        let mut form = aster();
        form.fundamental.name = "span".to_owned();
        refused_at(&form, fundamental(Field::Name));
    }

    #[test]
    fn duplicate_members_are_placed_at_the_second() {
        let mut form = aster();
        form.groupings[2].members[6].name = Some("Third".to_owned());
        let e = refused_at(&form, member(2, 6, Field::Name));
        assert!(matches!(e, DefinitionError::DuplicateMember { .. }));
    }

    #[test]
    fn era_units_that_dont_exist_or_are_subdivisions() {
        let mut form = aster();
        form.era.unit = "nothing".to_owned();
        let e = refused_at(&form, Location::Era { field: Field::Unit });
        assert!(matches!(e, DefinitionError::EraUnitUnknown { .. }));
        form.era.unit = "bell".to_owned();
        let e = refused_at(&form, Location::Era { field: Field::Unit });
        assert!(matches!(e, DefinitionError::EraUnitBelowFundamental { .. }));
    }

    #[test]
    fn first_numbers_the_era_numbers() {
        let mut form = aster();
        form.groupings[0].first_number = Some(0);
        let e = refused_at(&form, grouping(0, Some(Field::FirstNumber)));
        assert!(matches!(e, DefinitionError::FirstNumberNotAllowed { .. }));

        let mut form = aster();
        form.groupings[2].rules.clear();
        form.era.unit = "span".to_owned();
        form.groupings[2].member_unit.as_mut().unwrap().first_number = Some(0);
        refused_at(&form, grouping(2, Some(Field::MemberUnit)));

        let mut form = aster();
        form.groupings[2].rules.clear();
        form.era.unit = "sol".to_owned();
        form.fundamental.first_number = Some(0);
        refused_at(&form, fundamental(Field::FirstNumber));
    }

    #[test]
    fn beyond_the_engines_limits() {
        // The fundamental unit: too many ticks, and a length finer than a nanosecond allows.
        let mut form = aster();
        form.fundamental.length = "1000000000000000000000000000000".to_owned();
        let e = refused_at(&form, fundamental(Field::Length));
        assert!(matches!(
            e,
            DefinitionError::BeyondLimits {
                limit: Limit::TooManyTicks,
                ..
            }
        ));
        form.fundamental.length = "1/3000000000000000000".to_owned();
        refused_at(&form, fundamental(Field::Length));

        // The smallest subdivision is shorter than a nanosecond.
        let mut form = aster();
        form.fundamental.length = "1".to_owned();
        form.subdivisions[1].parts = Some(1_000_000_000);
        let e = refused_at(&form, subdivision(1, None));
        assert!(matches!(
            e,
            DefinitionError::BeyondLimits {
                limit: Limit::ShorterThanNanosecond,
                ..
            }
        ));

        // A grouping too long to count in fundamental units.
        let mut form = aster();
        form.groupings[0].members[0].count = Some(i64::MAX / 2);
        let e = refused_at(&form, grouping(0, None));
        assert!(matches!(
            e,
            DefinitionError::BeyondLimits {
                limit: Limit::GroupingTooLong,
                ..
            }
        ));
    }

    #[test]
    fn rules_off_the_eras_unit() {
        let mut form = aster();
        form.groupings[1].rules = vec![RuleForm {
            condition: "cohort % 2 == 0".to_owned(),
            member: "1".to_owned(),
            change: Some(1),
        }];
        let e = refused_at(&form, grouping(1, None));
        assert!(matches!(e, DefinitionError::RulesNotOnEraUnit { .. }));
    }

    #[test]
    fn rule_conditions_keep_their_position() {
        let e = refused_at(
            &with_rule("orbit %", "Twelfth", 1),
            rule(0, Field::Condition),
        );
        assert!(matches!(e, DefinitionError::RuleCondition { .. }));
        assert!(e.to_string().contains("1:8"), "{e}");

        let e = refused_at(
            &with_rule("orbit > 100", "Twelfth", 1),
            rule(0, Field::Condition),
        );
        assert!(matches!(e, DefinitionError::RulePeriod { .. }));

        let e = refused_at(
            &with_rule("100 / (orbit % 7) == 3", "Twelfth", 1),
            rule(0, Field::Condition),
        );
        assert!(matches!(e, DefinitionError::RuleFails { .. }));
    }

    #[test]
    fn rule_members_and_changes() {
        let e = refused_at(
            &with_rule("orbit % 4 == 0", "Thirteenth", 1),
            rule(0, Field::Member),
        );
        assert!(matches!(e, DefinitionError::RuleUnknownMember { .. }));
        let e = refused_at(
            &with_rule("orbit % 4 == 0", "Twelfth", 0),
            rule(0, Field::Change),
        );
        assert!(matches!(e, DefinitionError::RuleZeroChange { .. }));
    }

    #[test]
    fn the_second_rule_is_placed_at_index_1() {
        let mut form = aster();
        form.groupings[2].rules.push(RuleForm {
            condition: "orbit %".to_owned(),
            member: "Prime".to_owned(),
            change: Some(1),
        });
        refused_at(&form, rule(1, Field::Condition));
    }

    #[test]
    fn rules_that_repeat_too_rarely_together() {
        let mut form = aster();
        form.groupings[2].rules = ["97", "89", "83"]
            .iter()
            .map(|k| RuleForm {
                condition: format!("orbit % {k} == 0"),
                member: "Twelfth".to_owned(),
                change: Some(1),
            })
            .collect();
        let e = refused_at(&form, grouping(2, None));
        assert!(matches!(e, DefinitionError::RulesPeriodOverLimit { .. }));
    }

    #[test]
    fn members_made_too_short() {
        let e = refused_at(
            &with_rule("orbit % 4 == 0", "Third", -55),
            member(2, 2, Field::Count),
        );
        assert!(matches!(e, DefinitionError::MemberTooShort { .. }));
    }

    #[test]
    fn a_name_matching_no_unit_falls_back_to_the_calendar_name() {
        let record = aster().to_record().unwrap();
        let ghost = |s: &str| s.to_owned();
        for error in [
            DefinitionError::NoMembers { unit: ghost("x") },
            DefinitionError::DuplicateUnit { name: ghost("x") },
            DefinitionError::DuplicateMember {
                unit: ghost("orbit"),
                name: ghost("x"),
            },
            DefinitionError::NonPositiveCount {
                unit: ghost("orbit"),
                member: ghost("x"),
            },
            DefinitionError::RuleZeroChange {
                unit: ghost("x"),
                rule: 1,
            },
            DefinitionError::RuleZeroChange {
                unit: ghost("orbit"),
                rule: 9,
            },
            DefinitionError::EmptyName { what: "other" },
        ] {
            assert_eq!(locate(&error, &record), Location::CalendarName, "{error:?}");
        }
    }

    #[test]
    fn calendar_name_and_length_faults_come_before_the_engine() {
        let session = open_session();
        session.put_calendar(None, &aster()).unwrap();
        let mut form = aster();
        assert_eq!(
            refusal(&session, &form),
            FieldError {
                message: "This universe already has a calendar called Aster.".to_owned(),
                at: Location::CalendarName
            }
        );
        for name in ["", "   ", &"x".repeat(201)] {
            form.name = name.to_owned();
            assert_eq!(
                refusal(&session, &form),
                FieldError {
                    message: "A calendar needs a name of 1 to 200 characters.".to_owned(),
                    at: Location::CalendarName
                }
            );
        }
        form.name = "Other".to_owned();
        form.fundamental.length = "88775.244s".to_owned();
        form.groupings[0].name = String::new(); // the engine would refuse this too, later
        assert_eq!(
            refusal(&session, &form),
            FieldError {
                message: "the fundamental unit's length \"88775.244s\" is not a whole number, \
                          decimal or fraction"
                    .to_owned(),
                at: fundamental(Field::Length)
            }
        );
        assert_eq!(session.list_calendars().unwrap().len(), 1);
    }
}
