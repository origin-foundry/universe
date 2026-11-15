//! Conversion between instants, fixed numbers and dates (research R4 and R5).
//!
//! Everything is checked integer arithmetic; any result that doesn't fit is
//! [`CalendarError::OutOfRange`], never a wrapped value, and nothing panics.

use uni_time::{Instant, Ratio};

use crate::calendar::Kind;
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

    pub fn start_of_fixed(&self, _fixed: i64) -> R<DateStart> {
        todo!()
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

    pub fn date_to_instant(&self, _date: &Date) -> R<DateStart> {
        todo!()
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
