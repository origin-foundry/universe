use std::cmp::Ordering;
use std::fmt;

use crate::wide::{Signed, gcd, mul_div_round};
use crate::{Rounding, TimeError};

type R<T> = Result<T, TimeError>;

/// An exact fraction: a 128-bit numerator over a positive 128-bit denominator, always in lowest
/// terms, so equal values are always identical.
///
/// ```
/// use uni_time::Ratio;
/// let a = Ratio::new(1, 3).unwrap();
/// let b = Ratio::new(1, 6).unwrap();
/// assert_eq!(a.checked_add(b).unwrap(), Ratio::new(1, 2).unwrap());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i128, // never i128::MIN
    den: i128, // > 0, coprime with num
}

impl Ratio {
    pub const ZERO: Ratio = Ratio { num: 0, den: 1 };
    pub const ONE: Ratio = Ratio { num: 1, den: 1 };

    /// `num / den` in lowest terms; a zero denominator is an error.
    pub fn new(num: i128, den: i128) -> R<Ratio> {
        if den == 0 {
            return Err(TimeError::DivisionByZero);
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs());
        Ratio::from_parts((num < 0) != (den < 0), num.unsigned_abs() / g, den.unsigned_abs() / g)
    }

    pub fn from_int(n: i128) -> R<Ratio> {
        Ratio::new(n, 1)
    }

    pub fn numer(self) -> i128 {
        self.num
    }

    pub fn denom(self) -> i128 {
        self.den
    }

    pub fn is_integer(self) -> bool {
        self.den == 1
    }

    pub fn checked_add(self, o: Ratio) -> R<Ratio> {
        // Knuth's method: with g = gcd(b, d), a/b + c/d = t / (b/g × d), and gcd(t, b/g × d) =
        // gcd(t, g). The numerator is formed exactly in 256 bits.
        let (b, d) = (self.den.unsigned_abs(), o.den.unsigned_abs());
        let g = gcd(b, d);
        let (b_g, d_g) = ((b / g) as i128, (d / g) as i128);
        let t = Signed::mul(self.num, d_g).add(Signed::mul(o.num, b_g));
        if t.magnitude.is_zero() {
            return Ok(Ratio::ZERO);
        }
        let g2 = gcd(g, t.magnitude.divrem(g).1);
        let num = t.magnitude.divrem(g2).0.to_u128().ok_or(TimeError::Overflow)?;
        let den = (b / g).checked_mul(d / g2).ok_or(TimeError::Overflow)?;
        Ratio::from_parts(t.negative, num, den)
    }

    pub fn checked_sub(self, o: Ratio) -> R<Ratio> {
        self.checked_add(o.checked_neg()?)
    }

    pub fn checked_mul(self, o: Ratio) -> R<Ratio> {
        // Cancel common factors first, so the product is already in lowest terms and only
        // overflows if the true answer doesn't fit.
        let (a, b) = (self.num.unsigned_abs(), self.den.unsigned_abs());
        let (c, d) = (o.num.unsigned_abs(), o.den.unsigned_abs());
        let (g1, g2) = (gcd(a, d), gcd(c, b));
        if a == 0 || c == 0 {
            return Ok(Ratio::ZERO);
        }
        let num = (a / g1).checked_mul(c / g2).ok_or(TimeError::Overflow)?;
        let den = (b / g2).checked_mul(d / g1).ok_or(TimeError::Overflow)?;
        Ratio::from_parts((self.num < 0) != (o.num < 0), num, den)
    }

    pub fn checked_div(self, o: Ratio) -> R<Ratio> {
        if o.num == 0 {
            return Err(TimeError::DivisionByZero);
        }
        let reciprocal = Ratio { num: o.den * o.num.signum(), den: o.num.abs() };
        self.checked_mul(reciprocal)
    }

    pub fn checked_neg(self) -> R<Ratio> {
        Ok(Ratio { num: -self.num, den: self.den }) // num is never i128::MIN
    }

    /// The largest whole number not above this value (towards −∞).
    pub fn floor(self) -> i128 {
        self.num.div_euclid(self.den)
    }

    /// `self − floor(self)`, always in `[0, 1)`.
    pub fn rem_floor(self) -> Ratio {
        // gcd(num mod den, den) = gcd(num, den) = 1, so this is already in lowest terms.
        Ratio { num: self.num.rem_euclid(self.den), den: self.den }
    }

    /// The nearest whole number in the chosen direction.
    pub fn round(self, rounding: Rounding) -> i128 {
        mul_div_round(self.num, 1, self.den, rounding).expect("|num / den| ≤ |num| always fits")
    }

    /// Builds from a sign and coprime magnitudes, rejecting anything that doesn't fit.
    fn from_parts(negative: bool, num: u128, den: u128) -> R<Ratio> {
        let num = i128::try_from(num).map_err(|_| TimeError::Overflow)?;
        let den = i128::try_from(den).map_err(|_| TimeError::Overflow)?;
        Ok(Ratio { num: if negative { -num } else { num }, den })
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, o: &Ratio) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Ratio {
    /// By value: a/b against c/d compares a×d with c×b, exactly in 256 bits.
    fn cmp(&self, o: &Ratio) -> Ordering {
        Signed::mul(self.num, o.den).cmp(Signed::mul(o.num, self.den))
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rounding, TimeError};

    fn r(n: i128, d: i128) -> Ratio {
        Ratio::new(n, d).unwrap()
    }

    #[test]
    fn new_reduces_and_normalises_sign() {
        assert_eq!((r(2, 4).numer(), r(2, 4).denom()), (1, 2));
        assert_eq!(r(-2, -4), r(1, 2));
        assert_eq!((r(2, -4).numer(), r(2, -4).denom()), (-1, 2));
        assert_eq!((r(0, -7).numer(), r(0, -7).denom()), (0, 1));
        assert_eq!(Ratio::ZERO, r(0, 5));
        assert_eq!(Ratio::ONE, r(3, 3));
    }

    #[test]
    fn zero_denominator_and_min_are_errors() {
        assert_eq!(Ratio::new(1, 0), Err(TimeError::DivisionByZero));
        assert_eq!(r(1, 2).checked_div(Ratio::ZERO), Err(TimeError::DivisionByZero));
        assert_eq!(Ratio::new(i128::MIN, 1), Err(TimeError::Overflow));
        assert_eq!(Ratio::from_int(i128::MIN), Err(TimeError::Overflow));
        assert_eq!(Ratio::new(1, i128::MIN), Err(TimeError::Overflow));
    }

    #[test]
    fn known_answers() {
        assert_eq!(r(1, 3).checked_add(r(1, 6)), Ok(r(1, 2)));
        assert_eq!(r(146097, 400).checked_sub(r(365, 1)), Ok(r(97, 400)));
        assert_eq!(r(2, 3).checked_mul(r(3, 4)), Ok(r(1, 2)));
        assert_eq!(r(1, 2).checked_div(r(1, 4)), Ok(r(2, 1)));
        assert_eq!(r(1, 2).checked_neg(), Ok(r(-1, 2)));
        assert!(r(6, 3).is_integer() && !r(1, 2).is_integer());
    }

    #[test]
    fn overflow_is_an_error() {
        assert_eq!(r(i128::MAX, 1).checked_add(Ratio::ONE), Err(TimeError::Overflow));
        assert_eq!(r(i128::MAX, 1).checked_mul(r(2, 1)), Err(TimeError::Overflow));
        assert_eq!(r(1, i128::MAX).checked_add(r(1, i128::MAX - 1)), Err(TimeError::Overflow));
    }

    #[test]
    fn factors_cancel_before_multiplying() {
        let big = i128::MAX; // odd, so it shares no factor with 2
        assert_eq!(r(big, 2).checked_mul(r(2, big)), Ok(Ratio::ONE));
        assert_eq!(r(big, 7).checked_div(r(big, 7)), Ok(Ratio::ONE));
    }

    #[test]
    fn floor_and_remainder() {
        assert_eq!((r(7, 2).floor(), r(7, 2).rem_floor()), (3, r(1, 2)));
        assert_eq!((r(-7, 2).floor(), r(-7, 2).rem_floor()), (-4, r(1, 2)));
        assert_eq!((r(-4, 1).floor(), r(-4, 1).rem_floor()), (-4, Ratio::ZERO));
    }

    #[test]
    fn rounding_in_each_direction() {
        let cases = [
            (r(5, 2), 2, 3, 2),
            (r(-5, 2), -3, -2, -2),
            (r(7, 2), 3, 4, 4),
            (r(1, 3), 0, 1, 0),
            (r(-1, 3), -1, 0, 0),
            (r(2, 3), 0, 1, 1),
            (r(4, 1), 4, 4, 4),
        ];
        for (x, down, up, nearest) in cases {
            assert_eq!(x.round(Rounding::Down), down, "{x} down");
            assert_eq!(x.round(Rounding::Up), up, "{x} up");
            assert_eq!(x.round(Rounding::NearestEven), nearest, "{x} nearest");
        }
    }

    #[test]
    fn ordering_by_value() {
        assert!(r(-1, 2) < r(1, 3) && r(1, 3) < r(1, 2));
        assert!(r(i128::MAX, i128::MAX - 1) < r(i128::MAX - 1, i128::MAX - 2));
        assert!(r(i128::MAX - 1, i128::MAX) < r(i128::MAX, i128::MAX - 1));
        assert!(r(-i128::MAX, 1) < r(i128::MAX, 1));
        assert_eq!(r(2, 4).cmp(&r(1, 2)), std::cmp::Ordering::Equal);
    }

    #[test]
    fn display() {
        assert_eq!(r(1, 2).to_string(), "1/2");
        assert_eq!(r(3, 1).to_string(), "3");
        assert_eq!(r(-7, 2).to_string(), "-7/2");
    }
}
