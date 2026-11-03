use crate::wide::mul_div_round;
use crate::{Rounding, TimeError};

type R<T> = Result<T, TimeError>;

const NANOS_PER_SEC: i128 = 1_000_000_000;
/// Spans are strictly under 2⁶⁴ seconds either way, so the gap between any two instants fits.
const LIMIT: i128 = (1 << 64) * NANOS_PER_SEC;

/// A signed length of time, to the nanosecond.
///
/// ```
/// use uni_time::Span;
/// let a = Span::from_secs(2);
/// let b = Span::from_nanos(-500_000_000).unwrap();
/// assert_eq!(a.checked_add(b).unwrap().as_nanos(), 1_500_000_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span(i128);

impl Span {
    pub const ZERO: Span = Span(0);

    pub fn from_nanos(nanos: i128) -> R<Span> {
        if nanos.unsigned_abs() < LIMIT as u128 {
            Ok(Span(nanos))
        } else {
            Err(TimeError::OutOfRange)
        }
    }

    pub fn from_secs(secs: i64) -> Span {
        Span(i128::from(secs) * NANOS_PER_SEC) // |secs| ≤ 2⁶³ < 2⁶⁴
    }

    pub fn as_nanos(self) -> i128 {
        self.0
    }

    pub fn checked_add(self, other: Span) -> R<Span> {
        Span::from_nanos(self.0 + other.0) // both under 2⁹⁵, so no i128 overflow
    }

    pub fn checked_sub(self, other: Span) -> R<Span> {
        Span::from_nanos(self.0 - other.0)
    }

    pub fn checked_neg(self) -> R<Span> {
        Ok(Span(-self.0)) // the range is symmetric
    }

    pub fn checked_mul_int(self, n: i64) -> R<Span> {
        let product = self.0.checked_mul(i128::from(n)).ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(product)
    }

    pub fn checked_div_int(self, n: i64, rounding: Rounding) -> R<Span> {
        if n == 0 {
            return Err(TimeError::DivisionByZero);
        }
        let quotient = mul_div_round(self.0, 1, i128::from(n), rounding).ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(quotient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rounding, TimeError};

    const LIMIT: i128 = (1i128 << 64) * 1_000_000_000;

    fn ns(n: i128) -> Span {
        Span::from_nanos(n).unwrap()
    }

    #[test]
    fn range_limits() {
        assert_eq!(Span::from_nanos(LIMIT - 1).map(Span::as_nanos), Ok(LIMIT - 1));
        assert_eq!(Span::from_nanos(-(LIMIT - 1)).map(Span::as_nanos), Ok(-(LIMIT - 1)));
        assert_eq!(Span::from_nanos(LIMIT), Err(TimeError::OutOfRange));
        assert_eq!(Span::from_nanos(-LIMIT), Err(TimeError::OutOfRange));
    }

    #[test]
    fn construction_and_arithmetic() {
        assert_eq!(Span::from_secs(-3).as_nanos(), -3_000_000_000);
        assert_eq!(Span::from_secs(i64::MIN).as_nanos(), i128::from(i64::MIN) * 1_000_000_000);
        assert_eq!(ns(5).checked_add(ns(-7)), Ok(ns(-2)));
        assert_eq!(ns(5).checked_sub(ns(7)), Ok(ns(-2)));
        assert_eq!(ns(5).checked_neg(), Ok(ns(-5)));
        assert_eq!(ns(LIMIT - 1).checked_add(ns(1)), Err(TimeError::OutOfRange));
        assert_eq!(ns(-(LIMIT - 1)).checked_sub(ns(1)), Err(TimeError::OutOfRange));
        assert_eq!(ns(3).checked_mul_int(-4), Ok(ns(-12)));
        assert_eq!(ns(LIMIT - 1).checked_mul_int(2), Err(TimeError::OutOfRange));
        assert_eq!(ns(LIMIT - 1).checked_mul_int(i64::MAX), Err(TimeError::OutOfRange));
    }

    #[test]
    fn integer_division_rounds_as_asked() {
        let cases = [(7, (3, 4, 4)), (-7, (-4, -3, -4)), (6, (3, 3, 3)), (5, (2, 3, 2))];
        for (n, (down, up, nearest)) in cases {
            assert_eq!(ns(n).checked_div_int(2, Rounding::Down), Ok(ns(down)), "{n} down");
            assert_eq!(ns(n).checked_div_int(2, Rounding::Up), Ok(ns(up)), "{n} up");
            assert_eq!(ns(n).checked_div_int(2, Rounding::NearestEven), Ok(ns(nearest)), "{n} nearest");
        }
        assert_eq!(ns(7).checked_div_int(-2, Rounding::Down), Ok(ns(-4)));
        assert_eq!(ns(7).checked_div_int(0, Rounding::Down), Err(TimeError::DivisionByZero));
    }

    #[test]
    fn ordering() {
        assert!(ns(-1) < Span::ZERO && Span::ZERO < ns(1));
        assert!(ns(-(LIMIT - 1)) < ns(LIMIT - 1));
    }
}
