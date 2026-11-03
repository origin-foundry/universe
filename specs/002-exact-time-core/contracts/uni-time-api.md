# Contract: `uni-time` public API

The Rust API that later features build on. `R<T>` means `Result<T, TimeError>`. Names are fixed by this contract; doc comments and private helpers are up to the build.

## Instant

```rust
impl Instant {
    pub const ZERO: Instant;
    pub fn new(secs: i64, nanos: u32) -> R<Instant>;
    pub fn from_nanos(nanos: i128) -> R<Instant>;
    pub fn secs(self) -> i64;
    pub fn nanos(self) -> u32;
    pub fn as_nanos(self) -> i128;                       // always fits
    pub fn checked_add(self, span: Span) -> R<Instant>;
    pub fn checked_sub(self, span: Span) -> R<Instant>;
    pub fn since(self, earlier: Instant) -> Span;        // self − earlier; always fits
}
// derives: Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash
```

## Span

```rust
impl Span {
    pub const ZERO: Span;
    pub fn from_nanos(nanos: i128) -> R<Span>;
    pub fn from_secs(secs: i64) -> Span;                 // always fits
    pub fn as_nanos(self) -> i128;
    pub fn checked_add(self, other: Span) -> R<Span>;
    pub fn checked_sub(self, other: Span) -> R<Span>;
    pub fn checked_neg(self) -> R<Span>;
    pub fn checked_mul_int(self, n: i64) -> R<Span>;
    pub fn checked_div_int(self, n: i64, rounding: Rounding) -> R<Span>;
    pub fn as_secs_ratio(self) -> R<Ratio>;              // exact (FR-010)
    pub fn from_secs_ratio(secs: Ratio, rounding: Rounding) -> R<Span>;
    pub fn checked_mul_ratio(self, r: Ratio, rounding: Rounding) -> R<Span>;  // FR-011
}
// derives: Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash
```

## Ratio

```rust
impl Ratio {
    pub const ZERO: Ratio;
    pub const ONE: Ratio;
    pub fn new(num: i128, den: i128) -> R<Ratio>;        // reduces; den 0 -> DivisionByZero
    pub fn from_int(n: i128) -> R<Ratio>;                // i128::MIN -> Overflow
    pub fn numer(self) -> i128;
    pub fn denom(self) -> i128;
    pub fn is_integer(self) -> bool;
    pub fn checked_add(self, o: Ratio) -> R<Ratio>;
    pub fn checked_sub(self, o: Ratio) -> R<Ratio>;
    pub fn checked_mul(self, o: Ratio) -> R<Ratio>;
    pub fn checked_div(self, o: Ratio) -> R<Ratio>;
    pub fn checked_neg(self) -> R<Ratio>;
    pub fn floor(self) -> i128;                          // towards −∞
    pub fn rem_floor(self) -> Ratio;                     // self − floor, in [0, 1)
    pub fn round(self, rounding: Rounding) -> i128;
}
// derives: Copy, Clone, Debug, PartialEq, Eq, Hash; Ord implemented by value
// Display: "num/den", or "num" when den == 1
```

## Rounding and TimeError

```rust
pub enum Rounding { Down, Up, NearestEven }
pub enum TimeError { OutOfRange, DivisionByZero, Overflow }  // Display + std::error::Error
```
