//! Exact time values for Universe: moments on the absolute timeline, spans between them, and
//! exact fractions.
//!
//! Everything here is whole numbers and exact fractions; there is no floating point anywhere
//! (constitution principle III). Operations that can fail return [`TimeError`] rather than
//! wrapping, rounding or panicking, and anything that would need rounding takes a
//! [`Rounding`] chosen by the caller. Reference frames, clocks and calendars build on these
//! types in later phases. See `specs/002-exact-time-core/contracts/uni-time-api.md`.

#![deny(clippy::float_arithmetic)]

mod error;
mod instant;
mod ratio;
mod rounding;
mod span;
mod wide;

pub use error::TimeError;
pub use instant::Instant;
pub use ratio::Ratio;
pub use rounding::Rounding;
pub use span::Span;
