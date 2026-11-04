use crate::wide::mul_div_round;
use crate::{Ratio, Rounding, TimeError};

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
        let product = self
            .0
            .checked_mul(i128::from(n))
            .ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(product)
    }

    pub fn checked_div_int(self, n: i64, rounding: Rounding) -> R<Span> {
        if n == 0 {
            return Err(TimeError::DivisionByZero);
        }
        let quotient =
            mul_div_round(self.0, 1, i128::from(n), rounding).ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(quotient)
    }

    /// This span as an exact fraction of seconds.
    pub fn as_secs_ratio(self) -> R<Ratio> {
        Ratio::new(self.0, NANOS_PER_SEC)
    }

    /// The span nearest `secs` seconds, rounding anything finer than a nanosecond as asked.
    pub fn from_secs_ratio(secs: Ratio, rounding: Rounding) -> R<Span> {
        let nanos = mul_div_round(secs.numer(), NANOS_PER_SEC, secs.denom(), rounding)
            .ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(nanos)
    }

    /// This span scaled by an exact fraction, rounding to a nanosecond as asked.
    pub fn checked_mul_ratio(self, r: Ratio, rounding: Rounding) -> R<Span> {
        let nanos =
            mul_div_round(self.0, r.numer(), r.denom(), rounding).ok_or(TimeError::OutOfRange)?;
        Span::from_nanos(nanos)
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
        assert_eq!(
            Span::from_nanos(LIMIT - 1).map(Span::as_nanos),
            Ok(LIMIT - 1)
        );
        assert_eq!(
            Span::from_nanos(-(LIMIT - 1)).map(Span::as_nanos),
            Ok(-(LIMIT - 1))
        );
        assert_eq!(Span::from_nanos(LIMIT), Err(TimeError::OutOfRange));
        assert_eq!(Span::from_nanos(-LIMIT), Err(TimeError::OutOfRange));
    }

    #[test]
    fn construction_and_arithmetic() {
        assert_eq!(Span::from_secs(-3).as_nanos(), -3_000_000_000);
        assert_eq!(
            Span::from_secs(i64::MIN).as_nanos(),
            i128::from(i64::MIN) * 1_000_000_000
        );
        assert_eq!(ns(5).checked_add(ns(-7)), Ok(ns(-2)));
        assert_eq!(ns(5).checked_sub(ns(7)), Ok(ns(-2)));
        assert_eq!(ns(5).checked_neg(), Ok(ns(-5)));
        assert_eq!(ns(LIMIT - 1).checked_add(ns(1)), Err(TimeError::OutOfRange));
        assert_eq!(
            ns(-(LIMIT - 1)).checked_sub(ns(1)),
            Err(TimeError::OutOfRange)
        );
        assert_eq!(ns(3).checked_mul_int(-4), Ok(ns(-12)));
        assert_eq!(ns(LIMIT - 1).checked_mul_int(2), Err(TimeError::OutOfRange));
        assert_eq!(
            ns(LIMIT - 1).checked_mul_int(i64::MAX),
            Err(TimeError::OutOfRange)
        );
    }

    #[test]
    fn integer_division_rounds_as_asked() {
        let cases = [
            (7, (3, 4, 4)),
            (-7, (-4, -3, -4)),
            (6, (3, 3, 3)),
            (5, (2, 3, 2)),
        ];
        for (n, (down, up, nearest)) in cases {
            assert_eq!(
                ns(n).checked_div_int(2, Rounding::Down),
                Ok(ns(down)),
                "{n} down"
            );
            assert_eq!(ns(n).checked_div_int(2, Rounding::Up), Ok(ns(up)), "{n} up");
            assert_eq!(
                ns(n).checked_div_int(2, Rounding::NearestEven),
                Ok(ns(nearest)),
                "{n} nearest"
            );
        }
        assert_eq!(ns(7).checked_div_int(-2, Rounding::Down), Ok(ns(-4)));
        assert_eq!(
            ns(7).checked_div_int(0, Rounding::Down),
            Err(TimeError::DivisionByZero)
        );
    }

    #[test]
    fn ordering() {
        assert!(ns(-1) < Span::ZERO && Span::ZERO < ns(1));
        assert!(ns(-(LIMIT - 1)) < ns(LIMIT - 1));
    }
}

#[cfg(test)]
mod ratio_conversions {
    use super::*;

    fn q(n: i128, d: i128) -> Ratio {
        Ratio::new(n, d).unwrap()
    }

    #[test]
    fn span_to_seconds_is_exact() {
        assert_eq!(
            Span::from_nanos(1_500_000_000).unwrap().as_secs_ratio(),
            Ok(q(3, 2))
        );
        assert_eq!(
            Span::from_nanos(-1).unwrap().as_secs_ratio(),
            Ok(q(-1, 1_000_000_000))
        );
    }

    #[test]
    fn whole_nanoseconds_round_trip_exactly() {
        for n in [
            0,
            1,
            -1,
            999_999_999,
            -123_456_789_012_345,
            (1i128 << 64) * 1_000_000_000 - 1,
        ] {
            let span = Span::from_nanos(n).unwrap();
            for rounding in [Rounding::Down, Rounding::Up, Rounding::NearestEven] {
                assert_eq!(
                    Span::from_secs_ratio(span.as_secs_ratio().unwrap(), rounding),
                    Ok(span)
                );
            }
        }
    }

    #[test]
    fn sub_nanosecond_results_round_only_as_asked() {
        let third = q(1, 3);
        assert_eq!(
            Span::from_secs_ratio(third, Rounding::Down)
                .unwrap()
                .as_nanos(),
            333_333_333
        );
        assert_eq!(
            Span::from_secs_ratio(third, Rounding::Up)
                .unwrap()
                .as_nanos(),
            333_333_334
        );
        assert_eq!(
            Span::from_secs_ratio(third, Rounding::NearestEven)
                .unwrap()
                .as_nanos(),
            333_333_333
        );
        let minus_third = q(-1, 3);
        assert_eq!(
            Span::from_secs_ratio(minus_third, Rounding::Down)
                .unwrap()
                .as_nanos(),
            -333_333_334
        );
        assert_eq!(
            Span::from_secs_ratio(minus_third, Rounding::Up)
                .unwrap()
                .as_nanos(),
            -333_333_333
        );
    }

    #[test]
    fn ties_round_to_even() {
        let two_and_a_half_ns = q(5, 2_000_000_000);
        let three_and_a_half_ns = q(7, 2_000_000_000);
        assert_eq!(
            Span::from_secs_ratio(two_and_a_half_ns, Rounding::NearestEven)
                .unwrap()
                .as_nanos(),
            2
        );
        assert_eq!(
            Span::from_secs_ratio(three_and_a_half_ns, Rounding::NearestEven)
                .unwrap()
                .as_nanos(),
            4
        );
    }

    #[test]
    fn scaling_by_a_fraction() {
        let one_second = Span::from_secs(1);
        assert_eq!(
            one_second
                .checked_mul_ratio(q(1, 7), Rounding::Down)
                .unwrap()
                .as_nanos(),
            142_857_142
        );
        assert_eq!(
            one_second
                .checked_mul_ratio(q(-1, 7), Rounding::Down)
                .unwrap()
                .as_nanos(),
            -142_857_143
        );
        let huge = q(i128::MAX, i128::MAX - 1); // just over 1, with a huge numerator
        assert_eq!(
            one_second.checked_mul_ratio(huge, Rounding::Down),
            Ok(one_second)
        );
    }

    #[test]
    fn out_of_range_is_an_error_never_clipped() {
        let big = Span::from_nanos((1i128 << 64) * 1_000_000_000 - 1).unwrap();
        assert_eq!(
            big.checked_mul_ratio(q(2, 1), Rounding::Down),
            Err(TimeError::OutOfRange)
        );
        assert_eq!(
            Span::from_secs_ratio(q(i128::MAX, 1), Rounding::Down),
            Err(TimeError::OutOfRange)
        );
    }
}
