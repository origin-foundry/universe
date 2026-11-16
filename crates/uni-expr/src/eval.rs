//! Evaluation (FR-005 to FR-007; research R5).

use crate::arith;
use crate::check::Expr;
use crate::error::EvalError;
use crate::parse::{Ast, BinOp, Kind, UnOp};
use crate::types::{Pos, Value};

impl Expr {
    /// Evaluates with one value per declared variable, in declaration order (FR-005 to
    /// FR-007).
    ///
    /// Arithmetic is exact: `/` and `%` are floored, and a division by zero or a result outside
    /// 64 bits is an error at the operator. `&&`, `||` and `if` evaluate only what they need.
    /// Values that don't match the declarations in number or type give
    /// [`EvalError::Inputs`].
    ///
    /// ```
    /// use uni_expr::{EvalError, Expr, Pos, Type, Value, VarDecl};
    ///
    /// let vars = [VarDecl { name: "year".into(), ty: Type::Int }];
    /// let rule = Expr::check("year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)", &vars, Type::Bool)?;
    /// assert_eq!(rule.eval(&[Value::Int(2000)]), Ok(Value::Bool(true)));
    /// assert_eq!(rule.eval(&[Value::Int(1900)]), Ok(Value::Bool(false)));
    ///
    /// let ratio = Expr::check("100 / (year % 7)", &vars, Type::Int)?;
    /// assert_eq!(ratio.eval(&[Value::Int(14)]), Err(EvalError::DivisionByZero { at: Pos { line: 1, column: 5 } }));
    /// # Ok::<(), uni_expr::CheckError>(())
    /// ```
    pub fn eval(&self, values: &[Value]) -> Result<Value, EvalError> {
        let vars = self.vars();
        if values.len() != vars.len() || values.iter().zip(vars).any(|(v, d)| v.ty() != d.ty) {
            return Err(EvalError::Inputs {
                expected: vars.len(),
                given: values.len(),
            });
        }
        eval(self.root(), values)
    }
}

fn eval(ast: &Ast, values: &[Value]) -> Result<Value, EvalError> {
    match &ast.kind {
        Kind::Int(v) => Ok(Value::Int(*v)),
        Kind::Bool(b) => Ok(Value::Bool(*b)),
        Kind::Var(index) => Ok(values[*index]),
        // Checking replaces every name; an unresolved one can't reach evaluation.
        Kind::Name(_) => unreachable!("unresolved name in a checked expression"),
        Kind::Unary { op, at, operand } => unary(*op, eval(operand, values)?, *at),
        Kind::Binary { op, at, lhs, rhs } => {
            let left = eval(lhs, values)?;
            match (op, left) {
                (BinOp::And, Value::Bool(false)) => Ok(left),
                (BinOp::Or, Value::Bool(true)) => Ok(left),
                _ => binary(*op, left, eval(rhs, values)?, *at),
            }
        }
        Kind::If {
            cond, then, els, ..
        } => match eval(cond, values)? {
            Value::Bool(true) => eval(then, values),
            _ => eval(els, values),
        },
    }
}

/// A unary operator applied to a value of the type checking guarantees.
pub(crate) fn unary(op: UnOp, value: Value, at: Pos) -> Result<Value, EvalError> {
    match (op, value) {
        (UnOp::Neg, Value::Int(v)) => arith::neg(v)
            .map(Value::Int)
            .ok_or(EvalError::OutOfRange { at }),
        (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        _ => unreachable!("type checking guarantees the operand's type"),
    }
}

/// A binary operator applied to both its operands (no short-circuiting here).
pub(crate) fn binary(op: BinOp, left: Value, right: Value, at: Pos) -> Result<Value, EvalError> {
    let int = |result: Option<i64>, divisor: Option<i64>| match result {
        Some(v) => Ok(Value::Int(v)),
        None if divisor == Some(0) => Err(EvalError::DivisionByZero { at }),
        None => Err(EvalError::OutOfRange { at }),
    };
    Ok(match (left, right) {
        (Value::Int(a), Value::Int(b)) => match op {
            BinOp::Add => return int(arith::add(a, b), None),
            BinOp::Sub => return int(arith::sub(a, b), None),
            BinOp::Mul => return int(arith::mul(a, b), None),
            BinOp::Div => return int(arith::div(a, b), Some(b)),
            BinOp::Rem => return int(arith::rem(a, b), Some(b)),
            BinOp::Eq => Value::Bool(a == b),
            BinOp::Ne => Value::Bool(a != b),
            BinOp::Lt => Value::Bool(a < b),
            BinOp::Le => Value::Bool(a <= b),
            BinOp::Gt => Value::Bool(a > b),
            BinOp::Ge => Value::Bool(a >= b),
            BinOp::And | BinOp::Or => unreachable!("type checking guarantees true/false"),
        },
        (Value::Bool(a), Value::Bool(b)) => match op {
            BinOp::And => Value::Bool(a && b),
            BinOp::Or => Value::Bool(a || b),
            BinOp::Eq => Value::Bool(a == b),
            BinOp::Ne => Value::Bool(a != b),
            _ => unreachable!("type checking guarantees whole numbers"),
        },
        _ => unreachable!("type checking guarantees matching operands"),
    })
}
