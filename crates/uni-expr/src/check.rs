//! Checking: declarations, names and types (research R1, R3).

use crate::error::CheckError;
use crate::lex::{KEYWORDS, lex, name_continue, name_start};
use crate::parse::{Ast, BinOp, Kind, UnOp, parse};
use crate::types::{Pos, Type, VarDecl};

/// A checked expression, ready to evaluate. Made only by [`Expr::check`], so one that exists
/// has passed every check: evaluating it can fail only for the reasons in
/// [`EvalError`](crate::EvalError).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr {
    source: String,
    vars: Vec<VarDecl>,
    ty: Type,
    root: Ast,
}

impl Expr {
    /// Parses and type-checks `source` against the variables `vars`, requiring a result of
    /// type `result` (FR-002, FR-003).
    ///
    /// The declarations are checked first; then the text, refused at the first fault with its
    /// reason and its line and column.
    ///
    /// ```
    /// use uni_expr::{CheckError, Expr, Pos, Type, VarDecl};
    ///
    /// let vars = [VarDecl { name: "turn".into(), ty: Type::Int }];
    /// let rule = Expr::check("turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)", &vars, Type::Bool)?;
    /// assert_eq!(rule.source(), "turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)");
    ///
    /// assert_eq!(
    ///     Expr::check("phase == 2", &vars, Type::Bool),
    ///     Err(CheckError::UnknownVariable { name: "phase".into(), at: Pos { line: 1, column: 1 } }),
    /// );
    /// # Ok::<(), CheckError>(())
    /// ```
    pub fn check(source: &str, vars: &[VarDecl], result: Type) -> Result<Expr, CheckError> {
        check_declarations(vars)?;
        let mut root = parse(lex(source)?)?;
        let ty = resolve(&mut root, vars)?;
        require("result", result, ty, root.start)?;
        Ok(Expr {
            source: source.to_string(),
            vars: vars.to_vec(),
            ty,
            root,
        })
    }

    /// The text exactly as given to [`Expr::check`] (FR-004).
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The type of the result.
    pub fn ty(&self) -> Type {
        self.ty
    }

    /// The declarations this expression was checked against, in order.
    pub fn vars(&self) -> &[VarDecl] {
        &self.vars
    }

    pub(crate) fn root(&self) -> &Ast {
        &self.root
    }
}

/// Every name must be a valid name, not a keyword, and declared once.
fn check_declarations(vars: &[VarDecl]) -> Result<(), CheckError> {
    for (i, var) in vars.iter().enumerate() {
        let name = var.name.as_str();
        let mut chars = name.chars();
        let reason = match chars.next() {
            None => Some("a name can't be empty"),
            Some(first) if !name_start(first) => Some("a name must start with a letter or `_`"),
            _ if !chars.all(name_continue) => {
                Some("a name may contain only letters, digits and `_`")
            }
            _ if KEYWORDS.contains(&name) => Some("it's a keyword"),
            _ if vars[..i].iter().any(|v| v.name == name) => Some("it's declared more than once"),
            _ => None,
        };
        if let Some(reason) = reason {
            return Err(CheckError::InvalidVariable {
                name: var.name.clone(),
                reason,
            });
        }
    }
    Ok(())
}

fn require(context: &'static str, expected: Type, found: Type, at: Pos) -> Result<(), CheckError> {
    if expected == found {
        Ok(())
    } else {
        Err(CheckError::TypeMismatch {
            context,
            expected,
            found,
            at,
        })
    }
}

/// Resolves names to declaration indices and gives the node's type, refusing at the first
/// fault from left to right.
fn resolve(ast: &mut Ast, vars: &[VarDecl]) -> Result<Type, CheckError> {
    let start = ast.start;
    match &mut ast.kind {
        Kind::Int(_) => Ok(Type::Int),
        Kind::Bool(_) => Ok(Type::Bool),
        Kind::Name(name) => match vars.iter().position(|v| v.name == *name) {
            Some(index) => {
                let ty = vars[index].ty;
                ast.kind = Kind::Var(index);
                Ok(ty)
            }
            None => Err(CheckError::UnknownVariable {
                name: name.clone(),
                at: start,
            }),
        },
        Kind::Var(index) => Ok(vars[*index].ty),
        Kind::Unary { op, operand, .. } => {
            let (context, ty) = match op {
                UnOp::Neg => ("-", Type::Int),
                UnOp::Not => ("!", Type::Bool),
            };
            let found = resolve(operand, vars)?;
            require(context, ty, found, operand.start)?;
            Ok(ty)
        }
        Kind::Binary { op, lhs, rhs, .. } => {
            let context = op.symbol();
            let (operands, result) = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                    (Some(Type::Int), Type::Int)
                }
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => (Some(Type::Int), Type::Bool),
                BinOp::And | BinOp::Or => (Some(Type::Bool), Type::Bool),
                // Either type, as long as both sides match.
                BinOp::Eq | BinOp::Ne => (None, Type::Bool),
            };
            let left = resolve(lhs, vars)?;
            if let Some(ty) = operands {
                require(context, ty, left, lhs.start)?;
            }
            let right = resolve(rhs, vars)?;
            require(context, operands.unwrap_or(left), right, rhs.start)?;
            Ok(result)
        }
        Kind::If {
            cond, then, els, ..
        } => {
            let condition = resolve(cond, vars)?;
            require("if", Type::Bool, condition, cond.start)?;
            let ty = resolve(then, vars)?;
            let other = resolve(els, vars)?;
            require("then/else", ty, other, els.start)?;
            Ok(ty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vec<VarDecl> {
        vec![
            VarDecl {
                name: "turn".into(),
                ty: Type::Int,
            },
            VarDecl {
                name: "flag".into(),
                ty: Type::Bool,
            },
        ]
    }

    fn type_of(source: &str) -> Result<Type, CheckError> {
        let mut ast = parse(lex(source)?)?;
        resolve(&mut ast, &vars())
    }

    #[test]
    fn every_node_kind_has_its_type() {
        for (source, ty) in [
            ("1", Type::Int),
            ("true", Type::Bool),
            ("false", Type::Bool),
            ("turn", Type::Int),
            ("flag", Type::Bool),
            ("-turn", Type::Int),
            ("!flag", Type::Bool),
            ("turn + 1", Type::Int),
            ("turn - 1", Type::Int),
            ("turn * 1", Type::Int),
            ("turn / 1", Type::Int),
            ("turn % 1", Type::Int),
            ("turn < 1", Type::Bool),
            ("turn <= 1", Type::Bool),
            ("turn > 1", Type::Bool),
            ("turn >= 1", Type::Bool),
            ("turn == 1", Type::Bool),
            ("flag != true", Type::Bool),
            ("flag && true", Type::Bool),
            ("flag || false", Type::Bool),
            ("if flag then turn else 0", Type::Int),
            ("if flag then flag else false", Type::Bool),
        ] {
            assert_eq!(type_of(source), Ok(ty), "{source}");
        }
    }

    #[test]
    fn mismatches_point_at_the_operand() {
        let at = |column| Pos { line: 1, column };
        let mismatch = |context, expected, found, column| {
            Err(CheckError::TypeMismatch {
                context,
                expected,
                found,
                at: at(column),
            })
        };
        assert_eq!(type_of("flag * 2"), mismatch("*", Type::Int, Type::Bool, 1));
        assert_eq!(
            type_of("flag == 2"),
            mismatch("==", Type::Bool, Type::Int, 9)
        );
        assert_eq!(
            type_of("turn || flag"),
            mismatch("||", Type::Bool, Type::Int, 1)
        );
        assert_eq!(
            type_of("turn >= (flag)"),
            mismatch(">=", Type::Int, Type::Bool, 9)
        );
    }

    #[test]
    fn variables_resolve_to_their_index() {
        let mut ast = parse(lex("flag && turn > 0").unwrap()).unwrap();
        resolve(&mut ast, &vars()).unwrap();
        let Kind::Binary { lhs, rhs, .. } = ast.kind else {
            panic!("{ast:?}");
        };
        assert_eq!(lhs.kind, Kind::Var(1));
        let Kind::Binary { lhs, .. } = rhs.kind else {
            panic!("{rhs:?}");
        };
        assert_eq!(lhs.kind, Kind::Var(0));
    }

    #[test]
    fn declarations_name_their_fault() {
        let int = |name: &str| VarDecl {
            name: name.into(),
            ty: Type::Int,
        };
        let reason = |vars: &[VarDecl]| match check_declarations(vars) {
            Err(CheckError::InvalidVariable { reason, .. }) => reason,
            other => panic!("{other:?}"),
        };
        assert_eq!(reason(&[int("")]), "a name can't be empty");
        assert_eq!(
            reason(&[int("2x")]),
            "a name must start with a letter or `_`"
        );
        assert_eq!(
            reason(&[int("a b")]),
            "a name may contain only letters, digits and `_`"
        );
        assert_eq!(reason(&[int("else")]), "it's a keyword");
        assert_eq!(
            reason(&[int("n"), int("turn"), int("n")]),
            "it's declared more than once"
        );
        assert_eq!(check_declarations(&[int("ano"), int("年")]), Ok(()));
    }
}
