/// Which way to round a result that falls between two representable values.
///
/// There's no default: every operation that may need rounding asks for one, so nothing is
/// ever rounded without the caller saying so (constitution principle VII).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// Towards minus infinity (floor), the same either side of zero.
    Down,
    /// Towards plus infinity (ceiling), the same either side of zero.
    Up,
    /// To the nearest value; exact ties go to the even one, so repeated rounding doesn't drift.
    NearestEven,
}
