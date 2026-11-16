//! Checked whole-number arithmetic with floored `/` and `%` (research R5).
//!
//! Each function gives `None` when the exact result doesn't fit in `i64`, or for a division by
//! zero; the evaluator tells the two apart by the divisor.

pub(crate) fn add(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

pub(crate) fn sub(a: i64, b: i64) -> Option<i64> {
    a.checked_sub(b)
}

pub(crate) fn mul(a: i64, b: i64) -> Option<i64> {
    a.checked_mul(b)
}

pub(crate) fn neg(a: i64) -> Option<i64> {
    a.checked_neg()
}

/// Floored division: rounds towards minus infinity. `None` for `b = 0` and for
/// `i64::MIN / -1`, whose result doesn't fit.
pub(crate) fn div(a: i64, b: i64) -> Option<i64> {
    let q = a.checked_div(b)?;
    // `checked_div` succeeded, so `b` isn't 0 and `a % b` can't overflow.
    if a % b != 0 && ((a < 0) != (b < 0)) {
        // The truncated quotient is above the floored one by exactly one. It is at most 0
        // here and not `i64::MIN` (that needs `b = 1`, which leaves no remainder).
        Some(q - 1)
    } else {
        Some(q)
    }
}

/// Floored remainder: takes the sign of `b`, or is 0. `None` for `b = 0`. `i64::MIN % -1` is 0,
/// which fits.
pub(crate) fn rem(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        return None;
    }
    if b == -1 {
        return Some(0);
    }
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        // `r` and `b` have opposite signs, so this can't overflow.
        Some(r + b)
    } else {
        Some(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floored_division_and_remainder() {
        assert_eq!(div(-7, 4), Some(-2));
        assert_eq!(rem(-7, 4), Some(1));
        assert_eq!(div(7, -4), Some(-2));
        assert_eq!(rem(7, -4), Some(-1));
        assert_eq!(div(-7, -4), Some(1));
        assert_eq!(rem(-7, -4), Some(-3));
        assert_eq!(div(7, 4), Some(1));
        assert_eq!(rem(7, 4), Some(3));
        assert_eq!(rem(0, 5), Some(0));
        assert_eq!(div(-8, 4), Some(-2));
        assert_eq!(rem(-8, 4), Some(0));
    }

    #[test]
    fn division_failures() {
        assert_eq!(div(i64::MIN, -1), None);
        assert_eq!(rem(i64::MIN, -1), Some(0));
        assert_eq!(div(5, 0), None);
        assert_eq!(rem(5, 0), None);
        assert_eq!(div(i64::MIN, 1), Some(i64::MIN));
        assert_eq!(div(i64::MIN, 2), Some(i64::MIN / 2));
        assert_eq!(div(i64::MAX, -1), Some(-i64::MAX));
        assert_eq!(div(i64::MIN + 1, i64::MAX), Some(-1));
        assert_eq!(div(1, i64::MIN), Some(-1));
        assert_eq!(rem(1, i64::MIN), Some(i64::MIN + 1));
    }

    #[test]
    fn checked_operations() {
        assert_eq!(add(i64::MAX, 1), None);
        assert_eq!(add(2, 3), Some(5));
        assert_eq!(sub(i64::MIN, 1), None);
        assert_eq!(sub(2, 3), Some(-1));
        assert_eq!(mul(4_611_686_018_427_387_904, 2), None);
        assert_eq!(mul(-4_611_686_018_427_387_904, 2), Some(i64::MIN));
        assert_eq!(neg(i64::MIN), None);
        assert_eq!(neg(i64::MAX), Some(-i64::MAX));
    }

    #[test]
    fn division_identity_holds() {
        let samples = [
            i64::MIN,
            i64::MIN + 1,
            -1_000_003,
            -100,
            -7,
            -4,
            -1,
            0,
            1,
            3,
            4,
            7,
            100,
            999_983,
            i64::MAX - 1,
            i64::MAX,
        ];
        for a in samples {
            for b in samples {
                let (Some(q), Some(r)) = (div(a, b), rem(a, b)) else {
                    assert!(b == 0 || (a == i64::MIN && b == -1), "{a} / {b}");
                    continue;
                };
                // a = q·b + r exactly, computed in i128 so nothing wraps.
                assert_eq!(
                    i128::from(q) * i128::from(b) + i128::from(r),
                    i128::from(a),
                    "{a} / {b}"
                );
                assert!(r == 0 || (r < 0) == (b < 0), "{a} % {b} = {r}");
                assert!(r.unsigned_abs() < b.unsigned_abs(), "{a} % {b} = {r}");
            }
        }
    }
}
