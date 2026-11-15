use uni_time::Instant;

use crate::{Calendar, CalendarError, Date, DateStart, InstantDate};

type R<T> = Result<T, CalendarError>;

impl Calendar {
    pub fn fixed_from_instant(&self, _instant: Instant) -> R<i64> {
        todo!()
    }

    pub fn start_of_fixed(&self, _fixed: i64) -> R<DateStart> {
        todo!()
    }

    pub fn instant_to_date(&self, _instant: Instant) -> R<InstantDate> {
        todo!()
    }

    pub fn date_to_instant(&self, _date: &Date) -> R<DateStart> {
        todo!()
    }
}

/// `⌊a / b⌋` and the remainder `a − ⌊a / b⌋ × b`, in `[0, b)`, for `b > 0`.
fn floor_div(_a: i128, _b: i128) -> (i128, i128) {
    todo!()
}

/// The member that `pos` fundamental units into a grouping falls in: the last `j` with
/// `starts[j] ≤ pos`. `starts` begins at 0 and ends at the grouping's length, `pos` is inside.
fn member_at(_starts: &[i64], _pos: i64) -> usize {
    todo!()
}

/// Splits a position inside the fundamental unit, `r` in units of `1/q` ns (`0 ≤ r < p`),
/// through subdivisions of `parts`: each index, and the remainder below the last, in units
/// of `1/(q × parts…)` ns, still `< p`.
fn split(_r: i128, _p: i128, _parts: &[i64]) -> (Vec<i128>, i128) {
    todo!()
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
