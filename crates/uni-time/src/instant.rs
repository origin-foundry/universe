use crate::{Span, TimeError};

type R<T> = Result<T, TimeError>;

const NANOS_PER_SEC: i128 = 1_000_000_000;

/// A moment on the absolute timeline: whole seconds from the hidden zero, plus nanoseconds.
/// Moments before the zero have negative seconds; nanoseconds are always `0..1_000_000_000`,
/// so half a second before zero is `secs: -1, nanos: 500_000_000`.
///
/// ```
/// use uni_time::{Instant, Span};
/// let t = Instant::new(-1, 500_000_000).unwrap();
/// let later = t.checked_add(Span::from_secs(1)).unwrap();
/// assert_eq!((later.secs(), later.nanos()), (0, 500_000_000));
/// assert_eq!(later.since(t), Span::from_secs(1));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    secs: i64,
    nanos: u32,
}

impl Instant {
    pub const ZERO: Instant = Instant { secs: 0, nanos: 0 };

    pub fn new(secs: i64, nanos: u32) -> R<Instant> {
        if i128::from(nanos) < NANOS_PER_SEC {
            Ok(Instant { secs, nanos })
        } else {
            Err(TimeError::OutOfRange)
        }
    }

    pub fn from_nanos(nanos: i128) -> R<Instant> {
        let secs = i64::try_from(nanos.div_euclid(NANOS_PER_SEC)).map_err(|_| TimeError::OutOfRange)?;
        let nanos = u32::try_from(nanos.rem_euclid(NANOS_PER_SEC)).expect("0 ≤ remainder < 10⁹");
        Ok(Instant { secs, nanos })
    }

    pub fn secs(self) -> i64 {
        self.secs
    }

    pub fn nanos(self) -> u32 {
        self.nanos
    }

    /// Nanoseconds from the zero; always fits.
    pub fn as_nanos(self) -> i128 {
        i128::from(self.secs) * NANOS_PER_SEC + i128::from(self.nanos)
    }

    pub fn checked_add(self, span: Span) -> R<Instant> {
        Instant::from_nanos(self.as_nanos() + span.as_nanos())
    }

    pub fn checked_sub(self, span: Span) -> R<Instant> {
        Instant::from_nanos(self.as_nanos() - span.as_nanos())
    }

    /// The span from `earlier` to `self` (negative if `self` is earlier). Always fits, because
    /// spans reach just past the widest gap between two instants.
    pub fn since(self, earlier: Instant) -> Span {
        Span::from_nanos(self.as_nanos() - earlier.as_nanos()).expect("any gap between instants fits in a span")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Span, TimeError};

    const HUNDRED_THOUSAND_YEARS: i64 = 3_155_695_200_000; // 100,000 Gregorian years of seconds
    const ONE_NS: i128 = 1;

    fn at(secs: i64, nanos: u32) -> Instant {
        Instant::new(secs, nanos).unwrap()
    }

    #[test]
    fn construction() {
        assert_eq!(Instant::new(0, 1_000_000_000), Err(TimeError::OutOfRange));
        let half_before = Instant::from_nanos(-500_000_000).unwrap();
        assert_eq!((half_before.secs(), half_before.nanos()), (-1, 500_000_000));
        assert_eq!(half_before.as_nanos(), -500_000_000);
        assert_eq!(Instant::ZERO, at(0, 0));
        let top = i128::from(i64::MAX) * 1_000_000_000 + 999_999_999;
        assert_eq!(Instant::from_nanos(top).map(Instant::as_nanos), Ok(top));
        assert_eq!(Instant::from_nanos(top + 1), Err(TimeError::OutOfRange));
    }

    #[test]
    fn ordering_across_zero_and_one_nanosecond_apart() {
        assert!(at(-1, 500_000_000) < Instant::ZERO);
        assert!(at(0, 0) < at(0, 1));
        assert!(at(-1, 999_999_999) < at(0, 0));
    }

    #[test]
    fn exact_at_a_hundred_thousand_years_either_way() {
        for secs in [HUNDRED_THOUSAND_YEARS, -HUNDRED_THOUSAND_YEARS] {
            let t = at(secs, 0);
            let later = t.checked_add(Span::from_nanos(ONE_NS).unwrap()).unwrap();
            assert_eq!(later.since(t).as_nanos(), 1);
            assert_eq!(later.checked_sub(Span::from_nanos(ONE_NS).unwrap()), Ok(t));
        }
    }

    #[test]
    fn add_then_subtract_returns_the_original() {
        let t = at(-12_345, 678);
        let span = Span::from_nanos(-987_654_321_012).unwrap();
        assert_eq!(t.checked_add(span).and_then(|x| x.checked_sub(span)), Ok(t));
    }

    #[test]
    fn range_edges_are_errors() {
        let one = Span::from_nanos(1).unwrap();
        assert_eq!(at(i64::MAX, 999_999_999).checked_add(one), Err(TimeError::OutOfRange));
        assert_eq!(at(i64::MIN, 0).checked_sub(one), Err(TimeError::OutOfRange));
    }

    #[test]
    fn since_between_the_extremes_never_overflows() {
        let (first, last) = (at(i64::MIN, 0), at(i64::MAX, 999_999_999));
        let gap = last.since(first);
        assert_eq!(gap.as_nanos(), last.as_nanos() - first.as_nanos());
        assert_eq!(first.since(last).as_nanos(), -gap.as_nanos());
        assert_eq!(first.checked_add(gap), Ok(last));
    }
}
