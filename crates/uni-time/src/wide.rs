//! Exact 256-bit intermediates, so 128-bit results are only reported as too large when the true
//! answer really doesn't fit (spec 002, FR-009). Private to the crate.

use std::cmp::Ordering;

use crate::Rounding;

/// An unsigned 256-bit magnitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct U256 {
    hi: u128,
    lo: u128,
}

const LOW: u128 = u64::MAX as u128;

impl U256 {
    pub(crate) const ZERO: U256 = U256 { hi: 0, lo: 0 };

    pub(crate) fn from_u128(x: u128) -> U256 {
        U256 { hi: 0, lo: x }
    }

    /// The exact product of two 128-bit magnitudes.
    pub(crate) fn mul(a: u128, b: u128) -> U256 {
        let (a1, a0) = (a >> 64, a & LOW);
        let (b1, b0) = (b >> 64, b & LOW);
        let (p00, p01, p10, p11) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
        let mid = (p00 >> 64) + (p01 & LOW) + (p10 & LOW);
        U256 {
            lo: (p00 & LOW) | (mid << 64),
            hi: p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64),
        }
    }

    pub(crate) fn is_zero(self) -> bool {
        self.hi == 0 && self.lo == 0
    }

    /// The value as a u128, if it fits.
    pub(crate) fn to_u128(self) -> Option<u128> {
        (self.hi == 0).then_some(self.lo)
    }

    /// Sum of two magnitudes whose sum is known to fit (both are below 2²⁵⁵ here).
    pub(crate) fn add(self, o: U256) -> U256 {
        let (lo, carry) = self.lo.overflowing_add(o.lo);
        U256 { lo, hi: self.hi + o.hi + u128::from(carry) }
    }

    /// Difference of two magnitudes, `self ≥ o`.
    pub(crate) fn sub(self, o: U256) -> U256 {
        let (lo, borrow) = self.lo.overflowing_sub(o.lo);
        U256 { lo, hi: self.hi - o.hi - u128::from(borrow) }
    }

    /// Quotient and remainder by a divisor of at most 2¹²⁷.
    pub(crate) fn divrem(self, d: u128) -> (U256, u128) {
        debug_assert!(d != 0 && d <= 1u128 << 127);
        if self.hi == 0 {
            return (U256::from_u128(self.lo / d), self.lo % d);
        }
        let (mut q, mut r) = (U256::ZERO, 0u128);
        for i in (0..256).rev() {
            let word = if i >= 128 { self.hi } else { self.lo };
            r = (r << 1) | ((word >> (i % 128)) & 1);
            if r >= d {
                r -= d;
                if i >= 128 {
                    q.hi |= 1 << (i - 128);
                } else {
                    q.lo |= 1 << i;
                }
            }
        }
        (q, r)
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, o: &U256) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for U256 {
    fn cmp(&self, o: &U256) -> Ordering {
        (self.hi, self.lo).cmp(&(o.hi, o.lo))
    }
}

/// A signed 256-bit value as a sign and a magnitude; zero is never negative.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Signed {
    pub(crate) negative: bool,
    pub(crate) magnitude: U256,
}

impl Signed {
    /// The exact product of two i128s.
    pub(crate) fn mul(a: i128, b: i128) -> Signed {
        let magnitude = U256::mul(a.unsigned_abs(), b.unsigned_abs());
        Signed { negative: (a < 0) != (b < 0) && !magnitude.is_zero(), magnitude }
    }

    pub(crate) fn add(self, o: Signed) -> Signed {
        if self.negative == o.negative {
            return Signed { negative: self.negative, magnitude: self.magnitude.add(o.magnitude) };
        }
        let (big, small) = if self.magnitude >= o.magnitude { (self, o) } else { (o, self) };
        let magnitude = big.magnitude.sub(small.magnitude);
        Signed { negative: big.negative && !magnitude.is_zero(), magnitude }
    }

    pub(crate) fn cmp(self, o: Signed) -> Ordering {
        match (self.negative, o.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => self.magnitude.cmp(&o.magnitude),
            (true, true) => o.magnitude.cmp(&self.magnitude),
        }
    }
}

/// `round(a × b ÷ d)` exactly, for `d != 0`; `None` if the result doesn't fit in an i128
/// (excluding `i128::MIN`).
pub(crate) fn mul_div_round(a: i128, b: i128, d: i128, rounding: Rounding) -> Option<i128> {
    debug_assert!(d != 0);
    let product = Signed::mul(a, b);
    let negative = product.negative && d > 0 || !product.magnitude.is_zero() && !product.negative && d < 0;
    let divisor = d.unsigned_abs();
    let (q, r) = product.magnitude.divrem(divisor);
    let q = q.to_u128()?;
    let away = match rounding {
        Rounding::Down => negative && r != 0,
        Rounding::Up => !negative && r != 0,
        Rounding::NearestEven => match (2 * r).cmp(&divisor) {
            Ordering::Less => false,
            Ordering::Greater => true,
            Ordering::Equal => q % 2 == 1,
        },
    };
    let magnitude = if away { q.checked_add(1)? } else { q };
    let magnitude = i128::try_from(magnitude).ok()?;
    Some(if negative { -magnitude } else { magnitude })
}

pub(crate) fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn products_and_division_are_exact() {
        let p = U256::mul(u128::MAX, u128::MAX); // (2¹²⁸ − 1)² = 2²⁵⁶ − 2¹²⁹ + 1
        assert_eq!(p, U256 { hi: u128::MAX - 1, lo: 1 });
        let x = U256::mul(i128::MAX as u128, 1_000_000_007);
        assert_eq!(x.divrem(1_000_000_007), (U256::from_u128(i128::MAX as u128), 0));
        assert_eq!(x.divrem(i128::MAX as u128), (U256::from_u128(1_000_000_007), 0));
    }

    #[test]
    fn rounding_signs() {
        assert_eq!(mul_div_round(-7, 1, 2, Rounding::Down), Some(-4));
        assert_eq!(mul_div_round(7, 1, -2, Rounding::Down), Some(-4));
        assert_eq!(mul_div_round(-7, -1, 2, Rounding::Up), Some(4));
        assert_eq!(mul_div_round(-5, 1, 2, Rounding::NearestEven), Some(-2));
        assert_eq!(mul_div_round(0, 5, -3, Rounding::Down), Some(0));
        assert_eq!(mul_div_round(i128::MAX, 2, 1, Rounding::Down), None);
    }
}
