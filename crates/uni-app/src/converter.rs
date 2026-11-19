//! The converter (spec 008, research R9 and R10), and the date shapes it shares with the
//! anchor's live checks: dates entered value by value, read back with unit and member names,
//! and shown in every other calendar with how far into the smallest unit the moment falls.
//!
//! Nothing here returns a timeline value or seconds (FR-012): only whole numbers and text.

use serde::Serialize;
use uni_calendar::{Calendar, CalendarError, Date};
use uni_store::{CalendarRecord, Placement};
use uni_time::Instant;

use crate::Session;
use crate::calendars::{NO_UNIVERSE, unit_chain};

/// One value of a date, with its unit's name and, for a member, the member's name if it has
/// one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DateValue {
    pub unit: String,
    pub value: i64,
    pub name: Option<String>,
}

/// A date checked by its calendar: every value from the era's unit to the smallest unit, the
/// ones left empty filled in with their first values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DateReading {
    pub values: Vec<DateValue>,
}

/// A date its calendar refuses: the engine's message and the position, from 0, of the value
/// it concerns, when it concerns one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DateError {
    pub message: String,
    pub index: Option<usize>,
}

/// How far into its smallest unit a moment falls, in every form the precision switch shows
/// (research R10): `decimal` truncated to three places, and `exact` as a fraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Partway {
    pub decimal: String,
    pub decimal_exact: bool,
    pub exact: String,
}

/// A moment as a date in one calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConvertedDate {
    /// The units above the era's unit (Aster's age and cohort).
    pub above: Vec<DateValue>,
    /// From the era's unit to the smallest unit.
    pub values: Vec<DateValue>,
    /// `None` when the moment is exactly at the smallest unit's start.
    pub partway: Option<Partway>,
}

/// A calendar the converter can take a date in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConverterCalendar {
    pub id: i64,
    pub name: String,
    /// The units from the era's unit down: the fields to fill in.
    pub units: Vec<String>,
    pub problem: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConvertedCalendar {
    pub id: i64,
    pub name: String,
    pub date: ConvertedDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unusable {
    pub name: String,
    pub problem: String,
}

/// A date shown in every other calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Conversion {
    /// The date's exact start lay between two whole nanoseconds of the timeline.
    pub rounded: bool,
    /// Every other calendar that can be placed, in list order.
    pub results: Vec<ConvertedCalendar>,
    /// The calendars that can't be placed, and why.
    pub unusable: Vec<Unusable>,
}

/// Why `convert` gave no conversion: the date, or a plain failure. The UI tells them apart by
/// shape (an object with `message`, or a string).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ConvertError {
    Date(DateError),
    Store(String),
}

/// The index of the value a `CalendarError` concerns, among `units` (from the era's unit
/// down). The match is total, with no catch-all, so a new engine error needs a place.
pub(crate) fn error_index(error: &CalendarError, units: &[String]) -> Option<usize> {
    match error {
        CalendarError::InvalidValue { unit, .. } => units.iter().position(|u| u == unit),
        CalendarError::EmptyDate => Some(0),
        CalendarError::OutOfRange
        | CalendarError::TooManyValues { .. }
        | CalendarError::UnknownUnit { .. }
        | CalendarError::NotFixed { .. }
        | CalendarError::NotAGrouping { .. } => None,
    }
}

/// Where a gap is in `values`: the first missing value followed by a given one.
pub(crate) fn gap(values: &[Option<i64>]) -> Option<usize> {
    let last = values.iter().rposition(Option::is_some)?;
    values[..last].iter().position(Option::is_none)
}

/// The given values, trailing empty ones dropped; `None` if there's a gap.
pub(crate) fn given(values: &[Option<i64>]) -> Option<Vec<i64>> {
    if gap(values).is_some() {
        return None;
    }
    Some(values.iter().map_while(|v| *v).collect())
}

/// The units of `calendar` from the era's unit down.
pub(crate) fn units_from_era(calendar: &Calendar) -> Vec<String> {
    let all = calendar.units();
    let era = all
        .iter()
        .position(|u| *u == calendar.era_unit())
        .expect("the era's unit is in the chain");
    all[era..].iter().map(|u| (*u).to_owned()).collect()
}

/// Checks a date entered value by value, as the converter and the anchor's fields do: no
/// gap, at least one value, and the calendar accepts it.
pub(crate) fn check_date(
    calendar: &Calendar,
    values: &[Option<i64>],
) -> Result<(Vec<i64>, uni_calendar::DateStart), DateError> {
    let units = units_from_era(calendar);
    if let Some(index) = gap(values) {
        return Err(DateError {
            message: format!(
                "Fill in {}, or leave every smaller unit empty.",
                units.get(index).map_or("this value", String::as_str)
            ),
            index: Some(index),
        });
    }
    let values = given(values).expect("no gap");
    if values.is_empty() {
        return Err(DateError {
            message: format!("Enter at least the {}.", units[0]),
            index: Some(0),
        });
    }
    let start = calendar
        .date_to_instant(&Date {
            values: values.clone(),
        })
        .map_err(|e| DateError {
            message: e.to_string(),
            index: error_index(&e, &units),
        })?;
    Ok((values, start))
}

/// Names `numbers`, the values of `calendar`'s units from the `skip`-th unit of its chain
/// down, with member names from `record`.
fn named(
    record: &CalendarRecord,
    calendar: &Calendar,
    numbers: &[i64],
    skip: usize,
) -> Vec<DateValue> {
    let chain = unit_chain(record);
    let era = chain
        .iter()
        .position(|u| u.name == calendar.era_unit())
        .unwrap_or(0);
    numbers
        .iter()
        .zip(skip..)
        .map(|(&value, k)| {
            let unit = chain.get(k);
            let name = unit.and_then(|u| {
                let members = u.members.as_ref()?;
                let len = i64::try_from(members.len()).ok()?;
                let index = if k <= era {
                    (value - 1).rem_euclid(len)
                } else {
                    value - u.first
                };
                members.get(usize::try_from(index).ok()?)?.clone()
            });
            DateValue {
                unit: unit.map_or_else(String::new, |u| u.name.clone()),
                value,
                name,
            }
        })
        .collect()
}

/// The values of a date from the era's unit down, named.
pub(crate) fn reading(record: &CalendarRecord, calendar: &Calendar, values: &[i64]) -> DateReading {
    let era = units_from_era(calendar);
    let skip = calendar.units().len() - era.len();
    DateReading {
        values: named(record, calendar, values, skip),
    }
}

/// `instant` as a date in `calendar`, named, with how far into the smallest unit it falls.
pub(crate) fn converted_date(
    record: &CalendarRecord,
    calendar: &Calendar,
    instant: Instant,
) -> Result<ConvertedDate, CalendarError> {
    let found = calendar.instant_to_date(instant)?;
    let partway = if found.remainder == uni_time::Ratio::ZERO {
        None
    } else {
        let fraction = found
            .remainder
            .checked_div(calendar.smallest_unit_length()?)
            .map_err(|_| CalendarError::OutOfRange)?;
        let (decimal, decimal_exact) = fraction.to_decimal(3);
        Some(Partway {
            decimal,
            decimal_exact,
            exact: fraction.to_string(),
        })
    };
    let above = named(record, calendar, &found.above, 0);
    Ok(ConvertedDate {
        values: named(record, calendar, &found.date.values, found.above.len()),
        above,
        partway,
    })
}

fn find(placements: &[Placement], id: i64) -> Result<&Placement, ConvertError> {
    placements
        .iter()
        .find(|p| p.stored.id == id)
        .ok_or_else(|| ConvertError::Store("That calendar no longer exists.".to_owned()))
}

impl Session {
    /// Every calendar, with the units to fill in and its problem if it can't be placed.
    pub fn converter_calendars(&self) -> Result<Vec<ConverterCalendar>, String> {
        let slot = self.lock();
        let universe = slot.as_ref().ok_or(NO_UNIVERSE)?;
        let placements = universe.placements().map_err(|e| e.to_string())?;
        Ok(placements
            .into_iter()
            .map(|p| {
                let units = p
                    .stored
                    .record
                    .to_def()
                    .ok()
                    .and_then(|d| Calendar::new(d).ok())
                    .map_or_else(Vec::new, |c| units_from_era(&c));
                ConverterCalendar {
                    id: p.stored.id,
                    name: p.stored.record.name.clone(),
                    units,
                    problem: p.placed.err().map(|e| e.to_string()),
                }
            })
            .collect())
    }

    /// The moment `date` starts at in calendar `from`, as a date in every other calendar
    /// (research R9).
    pub fn convert(&self, from: i64, date: &[Option<i64>]) -> Result<Conversion, ConvertError> {
        let slot = self.lock();
        let universe = slot
            .as_ref()
            .ok_or_else(|| ConvertError::Store(NO_UNIVERSE.to_owned()))?;
        let placements = universe
            .placements()
            .map_err(|e| ConvertError::Store(e.to_string()))?;
        let source = find(&placements, from)?;
        let calendar = source.placed.as_ref().map_err(|problem| {
            ConvertError::Store(format!(
                "{} can't be used: {problem}",
                source.stored.record.name
            ))
        })?;
        let (_, start) = check_date(calendar, date).map_err(ConvertError::Date)?;
        let mut results = Vec::new();
        let mut unusable = Vec::new();
        for p in &placements {
            if p.stored.id == from {
                continue;
            }
            let name = p.stored.record.name.clone();
            match &p.placed {
                Ok(target) => match converted_date(&p.stored.record, target, start.instant) {
                    Ok(date) => results.push(ConvertedCalendar {
                        id: p.stored.id,
                        name,
                        date,
                    }),
                    Err(e) => unusable.push(Unusable {
                        name,
                        problem: e.to_string(),
                    }),
                },
                Err(problem) => unusable.push(Unusable {
                    name,
                    problem: problem.to_string(),
                }),
            }
        }
        Ok(Conversion {
            rounded: start.rounded_by.is_some(),
            results,
            unusable,
        })
    }
}

#[cfg(test)]
mod tests;
