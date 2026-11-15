//! Conversion between instants, fixed numbers and dates (research R4 and R5).
//!
//! Everything is checked integer arithmetic; any result that doesn't fit is
//! [`CalendarError::OutOfRange`], never a wrapped value, and nothing panics.

use uni_time::{Instant, Ratio};

use crate::calendar::{Kind, Unit};
use crate::{Calendar, CalendarError, Date, DateStart, InstantDate};

type R<T> = Result<T, CalendarError>;

const NANOS_PER_SEC: i128 = 1_000_000_000;

fn out_of_range<T>(_: T) -> CalendarError {
    CalendarError::OutOfRange
}

impl Calendar {
    /// The whole number of fundamental units from the epoch to `instant`, negative before it
    /// (FR-012).
    pub fn fixed_from_instant(&self, instant: Instant) -> R<i64> {
        Ok(self.locate(instant)?.0)
    }

    /// The fixed number and the position inside that fundamental unit, in units of `1/q` ns
    /// (`0 ≤ position < p`).
    fn locate(&self, instant: Instant) -> R<(i64, i128)> {
        // Both instants are under 2⁹⁴ ns from zero and q ≤ 10⁹, so this only fails for
        // calendars past the limits, but it's checked anyway.
        let offset = instant.as_nanos() - self.epoch.as_nanos();
        let scaled = offset
            .checked_mul(self.q)
            .ok_or(CalendarError::OutOfRange)?;
        let (fixed, position) = floor_div(scaled, self.p);
        Ok((i64::try_from(fixed).map_err(out_of_range)?, position))
    }

    /// The instant fundamental unit number `fixed` starts at (FR-012), rounded up to the next
    /// whole nanosecond if it falls between two (FR-014).
    pub fn start_of_fixed(&self, fixed: i64) -> R<DateStart> {
        self.start_at(fixed, 0)
    }

    /// The start of fundamental unit `fixed`, plus `ticks` (units of `1/(q × P)` ns, fewer
    /// than one fundamental unit's worth), as the first whole nanosecond at or after it.
    fn start_at(&self, fixed: i64, ticks: i128) -> R<DateStart> {
        // fixed × p / q ns, split into whole nanoseconds and a remainder in 1/q ns. If
        // fixed × p overflows i128, the start is over 2⁹⁷ ns away: beyond any instant.
        let scaled = i128::from(fixed)
            .checked_mul(self.p)
            .ok_or(CalendarError::OutOfRange)?;
        let (whole, rest) = floor_div(scaled, self.q);
        let q_parts = self.q * self.parts; // ≤ p < 2¹²⁰, checked in Calendar::new
        let (extra, fraction) = floor_div(rest * self.parts + ticks, q_parts);
        let mut nanos = self
            .epoch
            .as_nanos()
            .checked_add(whole)
            .and_then(|n| n.checked_add(extra))
            .ok_or(CalendarError::OutOfRange)?;
        let rounded_by = if fraction == 0 {
            None
        } else {
            nanos = nanos.checked_add(1).ok_or(CalendarError::OutOfRange)?;
            let denominator = q_parts
                .checked_mul(NANOS_PER_SEC)
                .ok_or(CalendarError::OutOfRange)?;
            Some(Ratio::new(q_parts - fraction, denominator).map_err(out_of_range)?)
        };
        Ok(DateStart {
            instant: Instant::from_nanos(nanos).map_err(out_of_range)?,
            rounded_by,
        })
    }

    /// The date `instant` falls on: the numbers of the units above the era's unit, every
    /// value from the era's unit down, and the exact time since the smallest unit started
    /// (FR-010). Never rounds.
    ///
    /// ```
    /// # use uni_calendar::{Calendar, CalendarDef, Clock, EraDef, FundamentalDef, GroupingDef, MemberDef, SubdivisionDef};
    /// # use uni_time::{Instant, Ratio, Span};
    /// # let def = CalendarDef {
    /// #     name: "Plain".to_string(),
    /// #     clock: Clock::Gct,
    /// #     groupings: vec![GroupingDef { name: "year".to_string(), member_unit: None,
    /// #         members: vec![MemberDef { name: None, count: 400 }], first_number: None }],
    /// #     fundamental: FundamentalDef { name: "sol".to_string(),
    /// #         length: Ratio::from_int(90_000).unwrap(), first_number: None },
    /// #     subdivisions: vec![SubdivisionDef { name: "hour".to_string(), parts: 10, first_number: None }],
    /// #     era: EraDef { name: "Plain Era".to_string(), unit: "year".to_string(), epoch: Instant::ZERO },
    /// # };
    /// // A 400-sol year of 90,000 s sols, 10 hours in each, epoch at the timeline's zero.
    /// let cal = Calendar::new(def).unwrap();
    /// let t = Instant::ZERO.checked_add(Span::from_secs(-1)).unwrap();
    /// let got = cal.instant_to_date(t).unwrap();
    /// assert_eq!(got.date.values, [0, 400, 9]); // year 0, sol 400, hour 9
    /// assert_eq!(got.remainder, Ratio::from_int(8_999).unwrap()); // seconds into hour 9
    /// ```
    pub fn instant_to_date(&self, instant: Instant) -> R<InstantDate> {
        let (fixed, position) = self.locate(instant)?;
        let mut numbers = Vec::with_capacity(self.units.len());
        let mut pos = fixed; // position within the current parent, in fundamental units
        let mut parent_count: i64 = 0; // the parent's count from the epoch
        for (k, unit) in self.units[..=self.fundamental].iter().enumerate() {
            let (index, count) = match &unit.kind {
                Kind::Grouping { length, .. } => {
                    let index = pos.div_euclid(*length);
                    pos = pos.rem_euclid(*length);
                    (index, fixed.div_euclid(*length))
                }
                Kind::Member { starts, .. } => {
                    let j = member_at(starts, pos);
                    pos -= starts[j];
                    let j = i64::try_from(j).map_err(out_of_range)?;
                    let members = i64::try_from(starts.len() - 1).map_err(out_of_range)?;
                    let count = parent_count
                        .checked_mul(members)
                        .and_then(|c| c.checked_add(j))
                        .ok_or(CalendarError::OutOfRange)?;
                    (j, count)
                }
                Kind::Fundamental | Kind::Subdivision { .. } => (pos, fixed),
            };
            parent_count = count;
            numbers.push(self.number(k, index, count)?);
        }
        let parts: Vec<i64> = self.subdivision_parts().collect();
        let (indices, remainder) = split(position, self.p, &parts);
        for (k, index) in (self.fundamental + 1..).zip(indices) {
            let index = i64::try_from(index).map_err(out_of_range)?;
            numbers.push(self.number(k, index, 0)?);
        }
        // The remainder is in units of 1/(q × P) ns.
        let denominator = self
            .q
            .checked_mul(self.parts)
            .and_then(|d| d.checked_mul(NANOS_PER_SEC))
            .ok_or(CalendarError::OutOfRange)?;
        let remainder = Ratio::new(remainder, denominator).map_err(out_of_range)?;
        let date = Date {
            values: numbers.split_off(self.era),
        };
        Ok(InstantDate {
            above: numbers,
            date,
            remainder,
        })
    }

    /// A unit's number: by count from the epoch at or above the era's unit (astronomical,
    /// so the one starting at the epoch is 1), by position plus first number below it.
    fn number(&self, k: usize, index: i64, count: i64) -> R<i64> {
        let n = if k <= self.era {
            count.checked_add(1)
        } else {
            index.checked_add(self.units[k].first)
        };
        n.ok_or(CalendarError::OutOfRange)
    }

    fn subdivision_parts(&self) -> impl Iterator<Item = i64> + '_ {
        self.units[self.fundamental + 1..]
            .iter()
            .filter_map(|u| match u.kind {
                Kind::Subdivision { parts } => Some(parts),
                _ => None,
            })
    }

    /// The instant a date starts at (FR-011): the first whole nanosecond at or after its
    /// exact start, with `rounded_by` giving the exact amount added when that wasn't already
    /// whole (FR-014). Missing smaller units take their first value. A value outside its
    /// unit's range is refused, naming the unit, the value and the range (FR-013).
    ///
    /// ```
    /// # use uni_calendar::{Calendar, CalendarDef, CalendarError, Clock, Date, EraDef, FundamentalDef, GroupingDef, MemberDef, SubdivisionDef};
    /// # use uni_time::{Instant, Ratio, Span};
    /// # let def = CalendarDef {
    /// #     name: "Plain".to_string(),
    /// #     clock: Clock::Gct,
    /// #     groupings: vec![GroupingDef { name: "year".to_string(), member_unit: None,
    /// #         members: vec![MemberDef { name: None, count: 400 }], first_number: None }],
    /// #     fundamental: FundamentalDef { name: "sol".to_string(),
    /// #         length: Ratio::from_int(90_000).unwrap(), first_number: None },
    /// #     subdivisions: vec![SubdivisionDef { name: "hour".to_string(), parts: 10, first_number: None }],
    /// #     era: EraDef { name: "Plain Era".to_string(), unit: "year".to_string(), epoch: Instant::ZERO },
    /// # };
    /// // A 400-sol year of 90,000 s sols, 10 hours in each, epoch at the timeline's zero.
    /// let cal = Calendar::new(def).unwrap();
    /// let start = cal.date_to_instant(&Date { values: vec![1, 2] }).unwrap(); // sol 2
    /// assert_eq!(start.instant, Instant::ZERO.checked_add(Span::from_secs(90_000)).unwrap());
    /// assert_eq!(start.rounded_by, None);
    /// let hour_ten = cal.date_to_instant(&Date { values: vec![1, 1, 10] });
    /// assert_eq!(hour_ten.unwrap_err().to_string(), "hour 10 is outside 0 to 9");
    /// ```
    pub fn date_to_instant(&self, date: &Date) -> R<DateStart> {
        let values = &date.values;
        let max = self.units.len() - self.era;
        if values.is_empty() {
            return Err(CalendarError::EmptyDate);
        }
        if values.len() > max {
            return Err(CalendarError::TooManyValues {
                given: values.len(),
                max,
            });
        }
        // The era's unit is numbered by count from the epoch: the one starting there is 1.
        let count = values[0].checked_sub(1).ok_or(CalendarError::OutOfRange)?;
        // Check every value against its range, before any other arithmetic. `indices[i]` is
        // the position of unit `era + 1 + i` within its parent; missing ones are 0.
        let mut member = match &self.units[self.era].kind {
            Kind::Member { counts, .. } => {
                let members = i64::try_from(counts.len()).map_err(out_of_range)?;
                usize::try_from(count.rem_euclid(members)).map_err(out_of_range)?
            }
            _ => 0,
        };
        let mut indices = Vec::with_capacity(max - 1);
        for k in self.era + 1..self.units.len() {
            let value = values.get(k - self.era).copied();
            let index = match value {
                Some(value) => self.check_value(k, member, value)?,
                None => 0,
            };
            if let Kind::Member { .. } = self.units[k].kind {
                member = usize::try_from(index).map_err(out_of_range)?;
            }
            indices.push(index);
        }
        // Now the start: fundamental units first, then ticks within the last one.
        let mut fixed = match &self.units[self.era].kind {
            Kind::Grouping { length, .. } => count.checked_mul(*length),
            Kind::Member { starts, .. } => {
                let members = i64::try_from(starts.len() - 1).map_err(out_of_range)?;
                let length = starts[starts.len() - 1];
                let start =
                    starts[usize::try_from(count.rem_euclid(members)).map_err(out_of_range)?];
                count
                    .div_euclid(members)
                    .checked_mul(length)
                    .and_then(|f| f.checked_add(start))
            }
            Kind::Fundamental | Kind::Subdivision { .. } => Some(count),
        }
        .ok_or(CalendarError::OutOfRange)?;
        let mut ticks: i128 = 0;
        let mut tick_length = self.p * self.parts; // one fundamental unit, < 2¹²⁰
        for (k, &index) in (self.era + 1..).zip(&indices) {
            let offset = match &self.units[k].kind {
                Kind::Grouping { length, .. } => index.checked_mul(*length),
                Kind::Member { starts, .. } => {
                    Some(starts[usize::try_from(index).map_err(out_of_range)?])
                }
                Kind::Fundamental => Some(index),
                Kind::Subdivision { parts } => {
                    tick_length /= i128::from(*parts);
                    ticks += i128::from(index) * tick_length; // < one fundamental unit
                    Some(0)
                }
            };
            fixed = offset
                .and_then(|o| fixed.checked_add(o))
                .ok_or(CalendarError::OutOfRange)?;
        }
        self.start_at(fixed, ticks)
    }

    /// Checks `value` for unit `k` (below the era's unit), where `member` is the member of
    /// the member unit just above, if there is one. Returns its position within its parent.
    fn check_value(&self, k: usize, member: usize, value: i64) -> R<i64> {
        let unit = &self.units[k];
        let (count, within) = match (&unit.kind, &self.units[k - 1].kind) {
            (Kind::Subdivision { parts }, _) => (*parts, None),
            (_, Kind::Grouping { children, .. }) => (*children, None),
            (_, Kind::Member { names, counts, .. }) => {
                (counts[member], Some(Unit::member_label(names, member)))
            }
            // The chain puts only subdivisions below the fundamental unit, so this can't
            // happen; it's an error rather than a panic all the same.
            (_, Kind::Fundamental | Kind::Subdivision { .. }) => {
                return Err(CalendarError::OutOfRange);
            }
        };
        let first = unit.first;
        let last = first
            .checked_add(count - 1)
            .ok_or(CalendarError::OutOfRange)?;
        if value < first || value > last {
            return Err(CalendarError::InvalidValue {
                unit: unit.name.clone(),
                value,
                first,
                last,
                within,
            });
        }
        Ok(value - first)
    }
}

/// `⌊a / b⌋` and the remainder `a − ⌊a / b⌋ × b`, in `[0, b)`, for `b > 0`.
fn floor_div(a: i128, b: i128) -> (i128, i128) {
    (a.div_euclid(b), a.rem_euclid(b))
}

/// The member that `pos` fundamental units into a grouping falls in: the last `j` with
/// `starts[j] ≤ pos`. `starts` begins at 0 and ends at the grouping's length, `pos` is inside.
fn member_at(starts: &[i64], pos: i64) -> usize {
    starts.partition_point(|&s| s <= pos) - 1
}

/// Splits a position inside the fundamental unit, `r` in units of `1/q` ns (`0 ≤ r < p`),
/// through subdivisions of `parts`: each index, and the remainder below the last, in units
/// of `1/(q × parts…)` ns, still `< p`.
///
/// Every step stays below `p × P < 2¹²⁰` (research R3), so nothing here can overflow.
fn split(r: i128, p: i128, parts: &[i64]) -> (Vec<i128>, i128) {
    let mut r = r;
    let indices = parts
        .iter()
        .map(|&n| {
            let scaled = r * i128::from(n);
            let index = scaled / p;
            r = scaled - index * p;
            index
        })
        .collect();
    (indices, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_division_rounds_towards_minus_infinity() {
        assert_eq!(floor_div(7, 2), (3, 1));
        assert_eq!(floor_div(-7, 2), (-4, 1));
        assert_eq!(floor_div(-1, 668), (-1, 667));
        assert_eq!(floor_div(-668, 668), (-1, 0));
        assert_eq!(floor_div(0, 5), (0, 0));
    }

    #[test]
    fn member_lookup_at_each_months_first_and_last_sol() {
        let counts = [56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55];
        let mut starts = vec![0];
        for c in counts {
            starts.push(starts.last().unwrap() + c);
        }
        assert_eq!(*starts.last().unwrap(), 668);
        for (j, c) in counts.iter().enumerate() {
            assert_eq!(member_at(&starts, starts[j]), j, "first sol of month {j}");
            assert_eq!(
                member_at(&starts, starts[j] + c - 1),
                j,
                "last sol of month {j}"
            );
        }
        assert_eq!(member_at(&[0, 400], 399), 0);
    }

    #[test]
    fn a_sol_split_into_seven_leaves_an_exact_fraction() {
        // A 100,000 s turn, p = 10¹⁴ ns, q = 1, split into 7 watches. 20,000 s into the turn
        // is in watch 1 (which starts at 14,285.714… s), 5,714.285… s in: 4 × 10¹³ / 7 ns.
        let p = 100_000_000_000_000;
        let (indices, rem) = split(20_000_000_000_000, p, &[7]);
        assert_eq!(indices, [1]);
        // In units of 1/7 ns: 20,000 s × 7 − 1 × 100,000 s = 40,000 s, i.e. 4 × 10¹³ / 7 ns.
        assert_eq!(rem, 40_000_000_000_000);
        // Two levels, Aster-style: a sol of 88,775.244 s into 20 hours of 100 minutes.
        let p = 88_775_244_000_000;
        let (indices, rem) = split(p - 1, p, &[20, 100]);
        assert_eq!(indices, [19, 99]);
        assert_eq!(rem, (p - 1) * 2_000 - (19 * 100 + 99) * p);
        let (indices, rem) = split(0, p, &[]);
        assert!(indices.is_empty() && rem == 0);
    }
}
