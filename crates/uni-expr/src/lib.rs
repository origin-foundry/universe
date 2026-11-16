//! The expression language, first subset (spec 005).
//!
//! Calendars describe their irregular parts, such as adjustment rules, with small expressions. This
//! crate checks such an expression before it is used, evaluates it exactly, and works out how
//! it repeats as one of its variables counts up.
//!
//! # What this subset covers
//!
//! - **Values**: whole numbers (64-bit, signed) and true/false.
//! - **Operators**: `+`, `-` (binary and unary), `*`, `/` and `%` (both floored: `/` rounds
//!   towards minus infinity and `%` takes the sign of the divisor), the comparisons `==`, `!=`,
//!   `<`, `<=`, `>` and `>=` (which don't chain), `&&`, `||` and `!`, `if … then … else …` and
//!   parentheses. Precedence, loosest first: `if`; `||`; `&&`; comparisons; `+ -`; `* / %`;
//!   unary `-` and `!`. Operators of one level group left to right.
//! - **Variables** the caller declares, each a whole number or true/false ([`VarDecl`]).
//! - **Checking** ([`Expr::check`]): syntax, names, types and nesting (at most [`MAX_DEPTH`]
//!   levels) are checked before use, and a refusal says why and where, as a line and column
//!   counted in characters ([`Pos`]).
//! - **Evaluation** ([`Expr::eval`]): exact; a division by zero or a result outside 64 bits is
//!   an error at the operator, never a wrapped value. `&&`, `||` and `if` evaluate only what
//!   they need.
//! - **Periods** ([`Expr::period`]): a number of steps after which the expression's value
//!   repeats as one whole-number variable counts up, worked out from the expression itself.
//!   It is always safe, though not always the smallest.
//!
//! Nothing here panics on any input, and nothing uses floating point (constitution principle
//! III: this is an exact crate).
//!
//! # What comes later
//!
//! `let`, user-defined functions, other types (fractions, durations, instants, dates), sky
//! functions, and storing an expression's tree alongside its source text all arrive with later
//! features (`/language/expression-language.md`).
//!
//! # Example
//!
//! ```
//! use uni_expr::{Expr, Type, Value, VarDecl};
//!
//! let vars = [VarDecl { name: "turn".into(), ty: Type::Int }];
//! let rule = Expr::check("turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)", &vars, Type::Bool)?;
//! assert_eq!(rule.eval(&[Value::Int(2000)])?, Value::Bool(true));
//! assert_eq!(rule.eval(&[Value::Int(1900)])?, Value::Bool(false));
//! assert_eq!(rule.period("turn", 100_000)?, 400);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::float_arithmetic)]

mod arith;
mod check;
mod error;
mod eval;
mod lex;
mod parse;
mod period;
mod types;

pub use check::Expr;
pub use error::{CheckError, EvalError, PeriodError};
pub use types::{Pos, Type, Value, VarDecl};

/// How deeply an expression may nest, counting every part of it (research R4).
pub const MAX_DEPTH: usize = 256;
