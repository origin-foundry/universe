# Data Model: Exact Time Core

All types live in `uni-time`. No floating point anywhere.

## Instant

| Field | Type | Rule |
|---|---|---|
| `secs` | `i64` | Whole seconds from the hidden zero; negative before it |
| `nanos` | `u32` | `0 ≤ nanos < 1_000_000_000`, always |

- Ordered by `(secs, nanos)`. Equal only if both fields are equal.
- Constructed by `Instant::new(secs, nanos)`, which rejects `nanos ≥ 10⁹` with `OutOfRange`, or `Instant::from_nanos(i128)`.

## Span

| Field | Type | Rule |
|---|---|---|
| (one value) | `i128` | Nanoseconds; `|value| < 2⁶⁴ × 10⁹` (strictly under 2⁶⁴ seconds, so any gap between two instants fits) |

- Ordered by value. Zero, positive and negative are all valid.

## Ratio

| Field | Type | Rule |
|---|---|---|
| `num` | `i128` | Any value except `i128::MIN`, so negation can't overflow |
| `den` | `i128` | `> 0`; `gcd(|num|, den) == 1`; zero is `0/1` |

- Equal values are identical, because the stored form is unique.
- Ordered by cross-multiplication, using a wider intermediate so comparison never overflows.

## Rounding

`Down` (towards −∞) · `Up` (towards +∞) · `NearestEven` (to nearest; ties to even).

## TimeError

| Variant | When | Message |
|---|---|---|
| `OutOfRange` | An instant or span would leave its range; `nanos ≥ 10⁹` | "time is out of range" |
| `DivisionByZero` | Denominator or divisor is zero | "division by zero" |
| `Overflow` | An exact fraction result doesn't fit in 128 bits | "fraction too large to represent exactly" |
