//! Working out how an expression repeats (research R6).

use crate::check::Expr;
use crate::error::PeriodError;

impl Expr {
    /// A period in the whole-number variable `var`, from 1 to `limit`.
    pub fn period(&self, var: &str, limit: i64) -> Result<i64, PeriodError> {
        let _ = (var, limit);
        todo!()
    }
}
