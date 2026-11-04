use std::fmt;

/// Why a time operation couldn't give an exact answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    /// An instant or span would leave its supported range, or a nanosecond field is 10⁹ or more.
    OutOfRange,
    /// A denominator or divisor is zero.
    DivisionByZero,
    /// An exact fraction result doesn't fit in 128 bits.
    Overflow,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeError::OutOfRange => "time is out of range",
            TimeError::DivisionByZero => "division by zero",
            TimeError::Overflow => "fraction too large to represent exactly",
        })
    }
}

impl std::error::Error for TimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_the_problem() {
        assert_eq!(TimeError::OutOfRange.to_string(), "time is out of range");
        assert_eq!(TimeError::DivisionByZero.to_string(), "division by zero");
        assert_eq!(
            TimeError::Overflow.to_string(),
            "fraction too large to represent exactly"
        );
    }
}
