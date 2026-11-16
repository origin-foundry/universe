//! Working out how an expression repeats (FR-008, FR-009; research R6).
//!
//! One bottom-up pass gives every node a [`Form`] with respect to the chosen variable `y`. Each
//! rule holds for exact whole numbers, so the root's exact value (or division-by-zero error)
//! repeats with the period given. Out-of-range errors depend on the size of the numbers and
//! aren't covered.

use crate::arith;
use crate::check::Expr;
use crate::error::{EvalError, PeriodError};
use crate::eval::{binary, unary};
use crate::parse::{Ast, BinOp, Kind, UnOp};
use crate::types::{Pos, Type, Value};

/// What a node is, as a function of `y`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    /// Doesn't depend on `y`, and evaluates without error to this value.
    Const(Value),
    /// Exactly `a·y + b`, with `a ≠ 0`.
    Affine(i64, i64),
    /// Its exact value, or division-by-zero error, is the same at `y` and `y + P`.
    Periodic(i64),
}

impl Form {
    /// The period this form repeats with, if it isn't affine.
    fn period(self) -> Option<i64> {
        match self {
            Form::Const(_) => Some(1),
            Form::Periodic(p) => Some(p),
            Form::Affine(..) => None,
        }
    }
}

impl Expr {
    /// A period P, from 1 to `limit`, in the whole-number variable `var`: for every value `y`,
    /// the expression's exact value (or division-by-zero error) at `y` and at `y + P` is the
    /// same, with the other variables unchanged (FR-008). It is worked out from the expression
    /// itself, and is safe but not always the smallest.
    ///
    /// `var` must be used only through a remainder by a fixed number, possibly after adding,
    /// subtracting or multiplying by fixed numbers; anything else that involves it is
    /// [`PeriodError::NotRepeating`] at the part responsible (FR-009).
    ///
    /// ```
    /// use uni_expr::{Expr, PeriodError, Pos, Type, VarDecl};
    ///
    /// let vars = [VarDecl { name: "turn".into(), ty: Type::Int }];
    /// let rule = Expr::check("turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)", &vars, Type::Bool)?;
    /// assert_eq!(rule.period("turn", 100_000), Ok(400));
    ///
    /// let drifting = Expr::check("turn / 7 % 2 == 0", &vars, Type::Bool)?;
    /// assert_eq!(drifting.period("turn", 100_000), Err(PeriodError::NotRepeating { at: Pos { line: 1, column: 6 } }));
    /// # Ok::<(), uni_expr::CheckError>(())
    /// ```
    pub fn period(&self, var: &str, limit: i64) -> Result<i64, PeriodError> {
        let Some(index) = self
            .vars()
            .iter()
            .position(|v| v.name == var && v.ty == Type::Int)
        else {
            return Err(PeriodError::NotAWholeNumber {
                name: var.to_string(),
            });
        };
        if limit < 1 {
            return Err(PeriodError::BadLimit { limit });
        }
        let analysis = Analysis { var: index, limit };
        let root = self.root();
        match analysis.form(root)?.period() {
            Some(p) => Ok(p),
            None => Err(PeriodError::NotRepeating { at: blame(root) }),
        }
    }
}

/// Where to point for a node: its operator or `if`, or its start.
fn blame(ast: &Ast) -> Pos {
    match &ast.kind {
        Kind::Unary { at, .. } | Kind::Binary { at, .. } | Kind::If { at, .. } => *at,
        _ => ast.start,
    }
}

struct Analysis {
    /// The index of the variable that counts up.
    var: usize,
    limit: i64,
}

impl Analysis {
    fn form(&self, ast: &Ast) -> Result<Form, PeriodError> {
        let not_repeating = Err(PeriodError::NotRepeating { at: blame(ast) });
        match &ast.kind {
            Kind::Int(v) => Ok(Form::Const(Value::Int(*v))),
            Kind::Bool(b) => Ok(Form::Const(Value::Bool(*b))),
            Kind::Var(index) if *index == self.var => Ok(Form::Affine(1, 0)),
            // Other variables don't change as `y` counts up.
            Kind::Var(_) => Ok(Form::Periodic(1)),
            Kind::Name(_) => unreachable!("unresolved name in a checked expression"),
            Kind::Unary { op, at, operand } => match self.form(operand)? {
                Form::Const(v) => Ok(fold(unary(*op, v, *at))),
                Form::Affine(a, b) if *op == UnOp::Neg => {
                    self.affine(arith::neg(a), arith::neg(b), *at)
                }
                Form::Affine(..) => not_repeating,
                Form::Periodic(p) => Ok(Form::Periodic(p)),
            },
            Kind::Binary { op, at, lhs, rhs } => {
                let left = self.form(lhs)?;
                let right = self.form(rhs)?;
                self.binary(*op, *at, left, right)
            }
            Kind::If {
                cond, then, els, ..
            } => {
                let forms = [self.form(cond)?, self.form(then)?, self.form(els)?];
                match forms {
                    [Form::Const(Value::Bool(c)), Form::Const(t), Form::Const(e)] => {
                        Ok(Form::Const(if c { t } else { e }))
                    }
                    _ => {
                        let mut p = 1;
                        for form in forms {
                            let Some(q) = form.period() else {
                                return not_repeating;
                            };
                            p = self.lcm(p, q)?;
                        }
                        Ok(Form::Periodic(p))
                    }
                }
            }
        }
    }

    fn binary(&self, op: BinOp, at: Pos, left: Form, right: Form) -> Result<Form, PeriodError> {
        use Form::{Affine, Const};
        match (op, left, right) {
            (_, Const(x), Const(y)) => Ok(fold(binary(op, x, y, at))),
            (BinOp::Add, Affine(a, b), Const(Value::Int(c)))
            | (BinOp::Add, Const(Value::Int(c)), Affine(a, b)) => {
                self.affine(Some(a), arith::add(b, c), at)
            }
            (BinOp::Add, Affine(a1, b1), Affine(a2, b2)) => {
                self.affine(arith::add(a1, a2), arith::add(b1, b2), at)
            }
            (BinOp::Sub, Affine(a, b), Const(Value::Int(c))) => {
                self.affine(Some(a), arith::sub(b, c), at)
            }
            (BinOp::Sub, Const(Value::Int(c)), Affine(a, b)) => {
                self.affine(arith::neg(a), arith::sub(c, b), at)
            }
            (BinOp::Sub, Affine(a1, b1), Affine(a2, b2)) => {
                self.affine(arith::sub(a1, a2), arith::sub(b1, b2), at)
            }
            (BinOp::Mul, Affine(a, b), Const(Value::Int(c)))
            | (BinOp::Mul, Const(Value::Int(c)), Affine(a, b)) => {
                self.affine(arith::mul(a, c), arith::mul(b, c), at)
            }
            // a·(y + m) + b ≡ a·y + b (mod m); a remainder by zero always fails.
            (BinOp::Rem, Affine(..), Const(Value::Int(0))) => Ok(Form::Periodic(1)),
            (BinOp::Rem, Affine(..), Const(Value::Int(m))) => match m.checked_abs() {
                Some(p) => self.within_limit(p).map(Form::Periodic),
                None => Err(PeriodError::OverLimit { limit: self.limit }),
            },
            _ => match (left.period(), right.period()) {
                (Some(p), Some(q)) => Ok(Form::Periodic(self.lcm(p, q)?)),
                _ => Err(PeriodError::NotRepeating { at }),
            },
        }
    }

    /// `a·y + b`, or a constant when `a` is 0; not repeating if a coefficient overflowed.
    fn affine(&self, a: Option<i64>, b: Option<i64>, at: Pos) -> Result<Form, PeriodError> {
        match (a, b) {
            (Some(0), Some(b)) => Ok(Form::Const(Value::Int(b))),
            (Some(a), Some(b)) => Ok(Form::Affine(a, b)),
            _ => Err(PeriodError::NotRepeating { at }),
        }
    }

    fn within_limit(&self, p: i64) -> Result<i64, PeriodError> {
        if p > self.limit {
            Err(PeriodError::OverLimit { limit: self.limit })
        } else {
            Ok(p)
        }
    }

    /// The least common multiple of two periods (both from 1 to the limit), refused as soon
    /// as it passes the limit.
    fn lcm(&self, p: i64, q: i64) -> Result<i64, PeriodError> {
        let over = PeriodError::OverLimit { limit: self.limit };
        let multiple = arith::mul(p / gcd(p, q), q).ok_or(over)?;
        self.within_limit(multiple)
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// A folded constant; one that fails fails the same way for every `y`.
fn fold(result: Result<Value, EvalError>) -> Form {
    match result {
        Ok(v) => Form::Const(v),
        Err(_) => Form::Periodic(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VarDecl;

    fn form(source: &str, limit: i64) -> Result<Form, PeriodError> {
        let vars = [
            VarDecl {
                name: "turn".into(),
                ty: Type::Int,
            },
            VarDecl {
                name: "n".into(),
                ty: Type::Int,
            },
        ];
        let expr = Expr::check(source, &vars, Type::Int)
            .or_else(|_| Expr::check(source, &vars, Type::Bool))
            .expect(source);
        Analysis { var: 0, limit }.form(expr.root())
    }

    fn at(column: u32) -> Pos {
        Pos { line: 1, column }
    }

    #[test]
    fn constants_fold() {
        assert_eq!(form("2 * 3 - 1", 10), Ok(Form::Const(Value::Int(5))));
        assert_eq!(form("-(4 % 3)", 10), Ok(Form::Const(Value::Int(-1))));
        assert_eq!(form("7 == 7", 10), Ok(Form::Const(Value::Bool(true))));
        assert_eq!(
            form("if 1 == 2 then 3 else 4", 10),
            Ok(Form::Const(Value::Int(4)))
        );
        // A fold that fails fails the same way for every turn.
        assert_eq!(form("1 / 0", 10), Ok(Form::Periodic(1)));
        assert_eq!(form("9223372036854775807 + 1", 10), Ok(Form::Periodic(1)));
        assert_eq!(form("-(-9223372036854775808)", 10), Ok(Form::Periodic(1)));
    }

    #[test]
    fn affine_parts_combine() {
        assert_eq!(form("turn", 10), Ok(Form::Affine(1, 0)));
        assert_eq!(form("3 * turn - 2", 10), Ok(Form::Affine(3, -2)));
        assert_eq!(form("5 - 2 * (turn + 1)", 10), Ok(Form::Affine(-2, 3)));
        assert_eq!(form("turn * 4 + turn", 10), Ok(Form::Affine(5, 0)));
        assert_eq!(form("-turn - -turn", 10), Ok(Form::Const(Value::Int(0))));
        assert_eq!(form("turn * 0 + 6", 10), Ok(Form::Const(Value::Int(6))));
    }

    #[test]
    fn coefficient_overflow_stops_repeating() {
        assert_eq!(
            form("turn * 4611686018427387904 * 4", 10),
            Err(PeriodError::NotRepeating { at: at(28) })
        );
        assert_eq!(
            form("turn + 9223372036854775807 + 1", 10),
            Err(PeriodError::NotRepeating { at: at(28) })
        );
        assert_eq!(
            form("-(turn - 9223372036854775807 - 1)", 10),
            Err(PeriodError::NotRepeating { at: at(1) })
        );
    }

    #[test]
    fn remainders_by_constants_repeat() {
        assert_eq!(form("(2 * turn + 1) % 6", 100), Ok(Form::Periodic(6)));
        assert_eq!(form("(2 * turn + 1) % -6", 100), Ok(Form::Periodic(6)));
        assert_eq!(form("turn % 0", 100), Ok(Form::Periodic(1)));
        assert_eq!(form("turn % (5 - 5)", 100), Ok(Form::Periodic(1)));
        assert_eq!(
            form("turn % 101", 100),
            Err(PeriodError::OverLimit { limit: 100 })
        );
    }

    #[test]
    fn periodic_parts_take_the_lcm() {
        assert_eq!(form("turn % 4 + turn % 6", 100), Ok(Form::Periodic(12)));
        assert_eq!(form("turn % 4 * n", 100), Ok(Form::Periodic(4)));
        assert_eq!(
            form("(turn % 9) / 2 - turn % 6", 100),
            Ok(Form::Periodic(18))
        );
        assert_eq!(form("turn % 8 == turn % 12", 100), Ok(Form::Periodic(24)));
        assert_eq!(
            form("if turn % 2 == 0 then turn % 5 else n", 100),
            Ok(Form::Periodic(10))
        );
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(7, 1), 1);
    }

    #[test]
    fn over_the_limit_as_soon_as_it_appears() {
        assert_eq!(
            form("turn % 7 + turn % 11 + turn % 13", 100),
            Err(PeriodError::OverLimit { limit: 100 })
        );
        // The left side passes the limit before the `/` on the right is reached.
        assert_eq!(
            form("turn % 7 * (turn % 11) + turn / 2", 50),
            Err(PeriodError::OverLimit { limit: 50 })
        );
        assert_eq!(
            form("turn % 7 * (turn % 11) + turn / 2", 100),
            Err(PeriodError::NotRepeating { at: at(31) })
        );
        let lcm = |p, q, limit| Analysis { var: 0, limit }.lcm(p, q);
        assert_eq!(
            lcm(i64::MAX, i64::MAX - 1, i64::MAX),
            Err(PeriodError::OverLimit { limit: i64::MAX })
        );
        assert_eq!(lcm(6, 4, 12), Ok(12));
    }
}
