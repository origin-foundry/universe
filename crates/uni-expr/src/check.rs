//! Checking: declarations, names and types (research R3).

use crate::error::CheckError;
use crate::parse::Ast;
use crate::types::{Type, VarDecl};

/// A checked expression, ready to evaluate. Made only by [`Expr::check`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr {
    source: String,
    vars: Vec<VarDecl>,
    ty: Type,
    root: Ast,
}

impl Expr {
    /// Parses and type-checks `source` against `vars`, requiring a result of type `result`.
    pub fn check(source: &str, vars: &[VarDecl], result: Type) -> Result<Expr, CheckError> {
        let _ = (source, vars, result);
        todo!()
    }

    /// The text exactly as given to [`Expr::check`].
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The type of the result.
    pub fn ty(&self) -> Type {
        self.ty
    }

    /// The declarations this expression was checked against.
    pub fn vars(&self) -> &[VarDecl] {
        &self.vars
    }

    pub(crate) fn root(&self) -> &Ast {
        &self.root
    }
}
