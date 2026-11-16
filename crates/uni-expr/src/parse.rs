//! The parser: tokens to a tree (research R2, R4).

use crate::types::Pos;

/// A parsed expression; `check` resolves names to declaration indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ast {
    pub start: Pos,
}
