//! Evaluation (FR-005 to FR-007).

use crate::check::Expr;
use crate::error::EvalError;
use crate::types::Value;

impl Expr {
    /// Evaluates with one value per declared variable, in declaration order.
    pub fn eval(&self, values: &[Value]) -> Result<Value, EvalError> {
        let _ = (self.root(), values);
        todo!()
    }
}
