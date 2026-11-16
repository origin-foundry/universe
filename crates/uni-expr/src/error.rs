use std::fmt;

use crate::types::{Pos, Type};

/// Why an expression or its declarations were refused by [`Expr::check`](crate::Expr::check)
/// (FR-003, FR-013).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    /// A declared variable's name isn't a valid name, is a keyword, or is declared twice.
    InvalidVariable { name: String, reason: &'static str },
    /// A character that starts no token.
    InvalidCharacter { found: char, at: Pos },
    /// A token where something else was needed; `found` is `None` at the end of the text.
    Unexpected {
        found: Option<String>,
        expected: &'static str,
        at: Pos,
    },
    /// A whole-number literal outside the 64-bit range.
    LiteralOutOfRange { at: Pos },
    /// A second comparison operator in a row, such as the second `<` in `1 < year < 5`.
    ChainedComparison { at: Pos },
    /// The expression nests more than `limit` levels deep.
    TooDeep { limit: usize, at: Pos },
    /// A name that isn't declared.
    UnknownVariable { name: String, at: Pos },
    /// An operand, condition, branch or the whole result has the wrong type. `context` is the
    /// operator or keyword (`"+"`, `"!"`, `"if"`, `"then/else"`), or `"result"` for the whole
    /// expression; `at` is where the offending part starts.
    TypeMismatch {
        context: &'static str,
        expected: Type,
        found: Type,
        at: Pos,
    },
}

/// Why evaluating a checked expression failed (FR-006).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalError {
    /// `/` or `%` by zero, at the operator.
    DivisionByZero { at: Pos },
    /// A result outside the 64-bit range, at the operator.
    OutOfRange { at: Pos },
    /// The values given don't match the declarations in number or type (a caller bug).
    Inputs { expected: usize, given: usize },
}

/// Why no period could be given (FR-009).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeriodError {
    /// The part at `at` can't be shown to repeat as the variable counts up.
    NotRepeating { at: Pos },
    /// The period would be more than the caller's limit.
    OverLimit { limit: i64 },
    /// The variable isn't declared, or isn't a whole number.
    NotAWholeNumber { name: String },
    /// The limit is less than 1.
    BadLimit { limit: i64 },
}

/// "a whole number" or "true/false", for messages.
fn a(ty: Type) -> &'static str {
    match ty {
        Type::Int => "a whole number",
        Type::Bool => "true/false",
    }
}

/// A whole number with commas between groups of three digits.
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    if n < 0 {
        out.push('-');
    }
    for (i, d) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(d);
    }
    out
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CheckError as E;
        match self {
            E::InvalidVariable { name, reason } => {
                write!(f, "`{name}` can't be a variable name: {reason}")
            }
            E::InvalidCharacter { found, at } => {
                write!(f, "unexpected character `{found}` at {at}")
            }
            E::Unexpected {
                found: Some(found),
                expected,
                at,
            } => write!(f, "expected {expected} but found `{found}` at {at}"),
            E::Unexpected {
                found: None,
                expected,
                at,
            } => write!(f, "expected {expected} but the expression ended at {at}"),
            E::LiteralOutOfRange { at } => {
                write!(f, "number at {at} is outside the supported range")
            }
            E::ChainedComparison { at } => {
                write!(f, "comparisons can't be chained at {at}; use `&&`")
            }
            E::TooDeep { limit, at } => {
                write!(f, "expression is nested more than {limit} levels deep at {at}")
            }
            E::UnknownVariable { name, at } => write!(f, "unknown variable `{name}` at {at}"),
            E::TypeMismatch {
                context: "result",
                expected,
                found,
                at,
            } => write!(
                f,
                "the result must be {}, found {} at {at}",
                a(*expected),
                a(*found)
            ),
            E::TypeMismatch {
                context: "then/else",
                expected,
                found,
                at,
            } => write!(
                f,
                "`then` and `else` must have the same type: expected {}, found {} at {at}",
                a(*expected),
                a(*found)
            ),
            E::TypeMismatch {
                context,
                expected,
                found,
                at,
            } => write!(
                f,
                "`{context}` needs {}, found {} at {at}",
                a(*expected),
                a(*found)
            ),
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::DivisionByZero { at } => write!(f, "division by zero at {at}"),
            EvalError::OutOfRange { at } => {
                write!(f, "result at {at} is outside the supported range")
            }
            EvalError::Inputs { expected, given } => write!(
                f,
                "expected {expected} value(s) matching the declarations, got {given}"
            ),
        }
    }
}

impl fmt::Display for PeriodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodError::NotRepeating { at } => write!(
                f,
                "the part at {at} stops the expression repeating as the variable counts up"
            ),
            PeriodError::OverLimit { limit } => write!(
                f,
                "the expression repeats only after more than {} steps",
                grouped(*limit)
            ),
            PeriodError::NotAWholeNumber { name } => {
                write!(f, "`{name}` isn't a whole-number variable of this expression")
            }
            PeriodError::BadLimit { .. } => f.write_str("the period limit must be at least 1"),
        }
    }
}

impl std::error::Error for CheckError {}
impl std::error::Error for EvalError {}
impl std::error::Error for PeriodError {}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: Pos = Pos { line: 1, column: 8 };

    #[test]
    fn check_error_messages() {
        let cases = [
            (
                CheckError::InvalidVariable {
                    name: "if".into(),
                    reason: "it's a keyword",
                },
                "`if` can't be a variable name: it's a keyword",
            ),
            (
                CheckError::InvalidCharacter {
                    found: '#',
                    at: Pos { line: 1, column: 6 },
                },
                "unexpected character `#` at 1:6",
            ),
            (
                CheckError::Unexpected {
                    found: None,
                    expected: "a value",
                    at: Pos {
                        line: 1,
                        column: 17,
                    },
                },
                "expected a value but the expression ended at 1:17",
            ),
            (
                CheckError::Unexpected {
                    found: Some(")".into()),
                    expected: "an operator or the end",
                    at: Pos { line: 1, column: 5 },
                },
                "expected an operator or the end but found `)` at 1:5",
            ),
            (
                CheckError::LiteralOutOfRange { at: Pos::START },
                "number at 1:1 is outside the supported range",
            ),
            (
                CheckError::ChainedComparison {
                    at: Pos {
                        line: 1,
                        column: 10,
                    },
                },
                "comparisons can't be chained at 1:10; use `&&`",
            ),
            (
                CheckError::TooDeep {
                    limit: 256,
                    at: Pos {
                        line: 3,
                        column: 40,
                    },
                },
                "expression is nested more than 256 levels deep at 3:40",
            ),
            (
                CheckError::UnknownVariable {
                    name: "month".into(),
                    at: Pos::START,
                },
                "unknown variable `month` at 1:1",
            ),
            (
                CheckError::TypeMismatch {
                    context: "+",
                    expected: Type::Int,
                    found: Type::Bool,
                    at: AT,
                },
                "`+` needs a whole number, found true/false at 1:8",
            ),
            (
                CheckError::TypeMismatch {
                    context: "result",
                    expected: Type::Bool,
                    found: Type::Int,
                    at: Pos::START,
                },
                "the result must be true/false, found a whole number at 1:1",
            ),
            (
                CheckError::TypeMismatch {
                    context: "then/else",
                    expected: Type::Int,
                    found: Type::Bool,
                    at: AT,
                },
                "`then` and `else` must have the same type: expected a whole number, \
                 found true/false at 1:8",
            ),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }

    #[test]
    fn eval_error_messages() {
        assert_eq!(
            EvalError::DivisionByZero {
                at: Pos { line: 1, column: 5 }
            }
            .to_string(),
            "division by zero at 1:5"
        );
        assert_eq!(
            EvalError::OutOfRange {
                at: Pos { line: 1, column: 6 }
            }
            .to_string(),
            "result at 1:6 is outside the supported range"
        );
        assert_eq!(
            EvalError::Inputs {
                expected: 1,
                given: 2
            }
            .to_string(),
            "expected 1 value(s) matching the declarations, got 2"
        );
    }

    #[test]
    fn period_error_messages() {
        assert_eq!(
            PeriodError::NotRepeating {
                at: Pos { line: 1, column: 6 }
            }
            .to_string(),
            "the part at 1:6 stops the expression repeating as the variable counts up"
        );
        assert_eq!(
            PeriodError::OverLimit { limit: 1000 }.to_string(),
            "the expression repeats only after more than 1,000 steps"
        );
        assert_eq!(
            PeriodError::OverLimit { limit: 1_234_567 }.to_string(),
            "the expression repeats only after more than 1,234,567 steps"
        );
        assert_eq!(
            PeriodError::NotAWholeNumber {
                name: "year".into()
            }
            .to_string(),
            "`year` isn't a whole-number variable of this expression"
        );
        assert_eq!(
            PeriodError::BadLimit { limit: 0 }.to_string(),
            "the period limit must be at least 1"
        );
    }

    #[test]
    fn grouping_digits() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(-100_000), "-100,000");
        assert_eq!(grouped(i64::MIN), "-9,223,372,036,854,775,808");
    }
}
