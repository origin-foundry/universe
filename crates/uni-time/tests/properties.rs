//! Property tests against an independent oracle (constitution VI; spec 002, FR-013).
//!
//! Every `Ratio` result is compared with `num_rational::BigRational`, an arbitrary-precision
//! implementation that never overflows: where the exact answer fits in 128 bits ours must equal
//! it, and where it doesn't ours must return `Overflow`. 10,000 cases per property (SC-002).

use num_bigint::BigInt;
use num_rational::BigRational;
use proptest::prelude::*;
use uni_time::{Instant, Ratio, Rounding, Span, TimeError};

const CASES: u32 = 10_000;
const SPAN_LIMIT: i128 = (1 << 64) * 1_000_000_000;

/// Whole numbers biased towards small values, the range limits and powers of two.
fn int() -> impl Strategy<Value = i128> {
    prop_oneof![
        -1_000i128..=1_000,
        any::<i64>().prop_map(i128::from),
        any::<i128>().prop_filter("never i128::MIN", |&n| n != i128::MIN),
        (0u32..127).prop_map(|k| 1i128 << k),
        (0i128..1_000).prop_map(|k| i128::MAX - k),
        (0i128..1_000).prop_map(|k| -(i128::MAX - k)),
    ]
}

fn ratio() -> impl Strategy<Value = Ratio> {
    (int(), int().prop_filter("non-zero denominator", |&d| d != 0))
        .prop_filter_map("representable", |(n, d)| Ratio::new(n, d).ok())
}

fn rounding() -> impl Strategy<Value = Rounding> {
    prop_oneof![Just(Rounding::Down), Just(Rounding::Up), Just(Rounding::NearestEven)]
}

fn big(r: Ratio) -> BigRational {
    BigRational::new(BigInt::from(r.numer()), BigInt::from(r.denom()))
}

/// Our expected answer for an exact oracle value: the same value if it fits, else `Overflow`.
fn expect(oracle: &BigRational) -> Result<Ratio, TimeError> {
    let (n, d) = (i128::try_from(oracle.numer()), i128::try_from(oracle.denom()));
    match (n, d) {
        (Ok(n), Ok(d)) if n != i128::MIN => Ok(Ratio::new(n, d).expect("oracle is reduced and fits")),
        _ => Err(TimeError::Overflow),
    }
}

fn round_oracle(x: &BigRational, rounding: Rounding) -> BigInt {
    let floor = x.floor().to_integer();
    let frac = x - BigRational::from_integer(floor.clone());
    let zero = BigRational::from_integer(BigInt::from(0));
    let half = BigRational::new(BigInt::from(1), BigInt::from(2));
    let up = floor.clone() + 1;
    match rounding {
        Rounding::Down => floor,
        Rounding::Up if frac == zero => floor,
        Rounding::Up => up,
        Rounding::NearestEven if frac < half => floor,
        Rounding::NearestEven if frac > half => up,
        Rounding::NearestEven if &floor % 2 == BigInt::from(0) => floor,
        Rounding::NearestEven => up,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn add_matches_oracle(a in ratio(), b in ratio()) {
        prop_assert_eq!(a.checked_add(b), expect(&(big(a) + big(b))));
    }

    #[test]
    fn sub_matches_oracle(a in ratio(), b in ratio()) {
        prop_assert_eq!(a.checked_sub(b), expect(&(big(a) - big(b))));
    }

    #[test]
    fn mul_matches_oracle(a in ratio(), b in ratio()) {
        prop_assert_eq!(a.checked_mul(b), expect(&(big(a) * big(b))));
    }

    #[test]
    fn div_matches_oracle(a in ratio(), b in ratio()) {
        if b == Ratio::ZERO {
            prop_assert_eq!(a.checked_div(b), Err(TimeError::DivisionByZero));
        } else {
            prop_assert_eq!(a.checked_div(b), expect(&(big(a) / big(b))));
        }
    }

    #[test]
    fn ordering_matches_oracle(a in ratio(), b in ratio()) {
        prop_assert_eq!(a.cmp(&b), big(a).cmp(&big(b)));
    }

    #[test]
    fn floor_and_remainder_match_oracle(a in ratio()) {
        let x = big(a);
        prop_assert_eq!(BigInt::from(a.floor()), x.floor().to_integer());
        prop_assert_eq!(big(a.rem_floor()), &x - x.floor());
    }

    #[test]
    fn rounding_matches_oracle(a in ratio(), r in rounding()) {
        prop_assert_eq!(BigInt::from(a.round(r)), round_oracle(&big(a), r));
    }

    #[test]
    fn instant_add_then_sub_is_identity(t in any::<i64>(), n in 0u32..1_000_000_000, s in -SPAN_LIMIT + 1..SPAN_LIMIT) {
        let t = Instant::new(t, n).unwrap();
        let s = Span::from_nanos(s).unwrap();
        if let Ok(later) = t.checked_add(s) {
            prop_assert_eq!(later.checked_sub(s), Ok(t));
            prop_assert_eq!(later.since(t), s);
        } else {
            let exact = t.as_nanos() + s.as_nanos();
            prop_assert!(Instant::from_nanos(exact).is_err());
        }
    }

    #[test]
    fn since_then_add_is_identity(a in any::<i64>(), an in 0u32..1_000_000_000, b in any::<i64>(), bn in 0u32..1_000_000_000) {
        let (a, b) = (Instant::new(a, an).unwrap(), Instant::new(b, bn).unwrap());
        prop_assert_eq!(b.checked_add(a.since(b)), Ok(a));
    }

    #[test]
    fn span_ratio_round_trip(s in -SPAN_LIMIT + 1..SPAN_LIMIT, r in rounding()) {
        let s = Span::from_nanos(s).unwrap();
        prop_assert_eq!(Span::from_secs_ratio(s.as_secs_ratio().unwrap(), r), Ok(s));
    }

    #[test]
    fn scaling_matches_oracle(s in -SPAN_LIMIT + 1..SPAN_LIMIT, f in ratio(), r in rounding()) {
        let span = Span::from_nanos(s).unwrap();
        let exact = round_oracle(&(BigRational::from_integer(BigInt::from(s)) * big(f)), r);
        let expected = i128::try_from(exact).ok().and_then(|n| Span::from_nanos(n).ok()).ok_or(TimeError::OutOfRange);
        prop_assert_eq!(span.checked_mul_ratio(f, r), expected);
    }
}
