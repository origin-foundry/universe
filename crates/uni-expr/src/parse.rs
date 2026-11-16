//! The parser: tokens to a tree (research R2, R4).
//!
//! Precedence climbing over the contract's grammar. Each nested parenthesis, unary operator
//! and `if` counts one level of the parser's own recursion, and every node one level of the
//! tree; both are held to [`MAX_DEPTH`], so nothing later can exhaust the stack.

use crate::MAX_DEPTH;
use crate::error::CheckError;
use crate::lex::{Lexed, Tok, Token};
use crate::types::Pos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl BinOp {
    pub(crate) fn symbol(self) -> &'static str {
        match self {
            BinOp::Or => "||",
            BinOp::And => "&&",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
        }
    }
}

/// How tightly comparisons bind; they don't chain.
const COMPARISON: u8 = 3;

/// The binary operator a token stands for, with its precedence (higher binds tighter).
fn binary_op(tok: &Tok) -> Option<(BinOp, u8)> {
    Some(match tok {
        Tok::OrOr => (BinOp::Or, 1),
        Tok::AndAnd => (BinOp::And, 2),
        Tok::EqEq => (BinOp::Eq, COMPARISON),
        Tok::NotEq => (BinOp::Ne, COMPARISON),
        Tok::Lt => (BinOp::Lt, COMPARISON),
        Tok::Le => (BinOp::Le, COMPARISON),
        Tok::Gt => (BinOp::Gt, COMPARISON),
        Tok::Ge => (BinOp::Ge, COMPARISON),
        Tok::Plus => (BinOp::Add, 4),
        Tok::Minus => (BinOp::Sub, 4),
        Tok::Star => (BinOp::Mul, 5),
        Tok::Slash => (BinOp::Div, 5),
        Tok::Percent => (BinOp::Rem, 5),
        _ => return None,
    })
}

/// An expression's tree. Every node keeps where it starts; operators also keep where the
/// operator is, which is where evaluation errors point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ast {
    pub kind: Kind,
    pub start: Pos,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Int(i64),
    Bool(bool),
    /// A name as written; checking replaces it with `Var`.
    Name(String),
    /// A variable, by its index in the declarations.
    Var(usize),
    Unary {
        op: UnOp,
        at: Pos,
        operand: Box<Ast>,
    },
    Binary {
        op: BinOp,
        at: Pos,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
    If {
        /// Where the `if` is.
        at: Pos,
        cond: Box<Ast>,
        then: Box<Ast>,
        els: Box<Ast>,
    },
}

/// Parses a whole text's tokens into one expression.
pub(crate) fn parse(lexed: Lexed) -> Result<Ast, CheckError> {
    let mut parser = Parser {
        tokens: lexed.tokens,
        next: 0,
        end: lexed.end,
        depth: 0,
    };
    let (ast, _) = parser.binary(0)?;
    match parser.peek() {
        None => Ok(ast),
        Some(_) => Err(parser.unexpected("an operator or the end")),
    }
}

/// A subtree and its depth.
type Parsed = Result<(Ast, usize), CheckError>;

struct Parser {
    tokens: Vec<Token>,
    next: usize,
    end: Pos,
    /// The parser's own recursion: open parentheses, unary operators and `if`s.
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.next)
    }

    fn unexpected(&self, expected: &'static str) -> CheckError {
        match self.peek() {
            Some(token) => CheckError::Unexpected {
                found: Some(token.tok.text()),
                expected,
                at: token.at,
            },
            None => CheckError::Unexpected {
                found: None,
                expected,
                at: self.end,
            },
        }
    }

    fn expect(&mut self, tok: &Tok, expected: &'static str) -> Result<(), CheckError> {
        match self.peek() {
            Some(token) if token.tok == *tok => {
                self.next += 1;
                Ok(())
            }
            _ => Err(self.unexpected(expected)),
        }
    }

    /// Runs `f` one level deeper in the parser's recursion.
    fn nested(&mut self, at: Pos, f: impl FnOnce(&mut Self) -> Parsed) -> Parsed {
        if self.depth >= MAX_DEPTH {
            return Err(too_deep(at));
        }
        self.depth += 1;
        let parsed = f(self);
        self.depth -= 1;
        parsed
    }

    /// Operators binding at least as tightly as `min`, grouping left to right.
    fn binary(&mut self, min: u8) -> Parsed {
        let (mut lhs, mut depth) = self.unary()?;
        let mut compared = false;
        while let Some(token) = self.peek() {
            let Some((op, prec)) = binary_op(&token.tok) else {
                break;
            };
            if prec < min {
                break;
            }
            let at = token.at;
            if prec == COMPARISON {
                if compared {
                    return Err(CheckError::ChainedComparison { at });
                }
                compared = true;
            } else if prec < COMPARISON {
                compared = false;
            }
            self.next += 1;
            let (rhs, rhs_depth) = self.binary(prec + 1)?;
            depth = node_depth(depth.max(rhs_depth), at)?;
            lhs = Ast {
                start: lhs.start,
                kind: Kind::Binary {
                    op,
                    at,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
        Ok((lhs, depth))
    }

    fn unary(&mut self) -> Parsed {
        let Some(token) = self.peek() else {
            return self.atom();
        };
        let at = token.at;
        let op = match token.tok {
            Tok::Minus => UnOp::Neg,
            Tok::Bang => UnOp::Not,
            _ => return self.atom(),
        };
        self.next += 1;
        if op == UnOp::Neg
            && let Some(Token {
                tok: Tok::Int(digits),
                ..
            }) = self.peek()
        {
            // Folded into the literal, so the smallest whole number can be written.
            let value = literal(digits, true, at)?;
            self.next += 1;
            return Ok((
                Ast {
                    kind: Kind::Int(value),
                    start: at,
                },
                1,
            ));
        }
        let (operand, depth) = self.nested(at, |p| p.unary())?;
        Ok((
            Ast {
                kind: Kind::Unary {
                    op,
                    at,
                    operand: Box::new(operand),
                },
                start: at,
            },
            node_depth(depth, at)?,
        ))
    }

    fn atom(&mut self) -> Parsed {
        let Some(token) = self.peek() else {
            return Err(self.unexpected("a value"));
        };
        let at = token.at;
        let kind = match &token.tok {
            Tok::Int(digits) => Kind::Int(literal(digits, false, at)?),
            Tok::True => Kind::Bool(true),
            Tok::False => Kind::Bool(false),
            Tok::Name(name) => Kind::Name(name.clone()),
            Tok::LParen => {
                self.next += 1;
                let (mut inner, depth) = self.nested(at, |p| {
                    let inner = p.binary(0)?;
                    p.expect(&Tok::RParen, "`)`")?;
                    Ok(inner)
                })?;
                // The part starts at its opening parenthesis.
                inner.start = at;
                return Ok((inner, depth));
            }
            Tok::If => {
                self.next += 1;
                return self.nested(at, |p| {
                    let (cond, c) = p.binary(0)?;
                    p.expect(&Tok::Then, "`then`")?;
                    let (then, t) = p.binary(0)?;
                    p.expect(&Tok::Else, "`else`")?;
                    let (els, e) = p.binary(0)?;
                    Ok((
                        Ast {
                            kind: Kind::If {
                                at,
                                cond: Box::new(cond),
                                then: Box::new(then),
                                els: Box::new(els),
                            },
                            start: at,
                        },
                        node_depth(c.max(t).max(e), at)?,
                    ))
                });
            }
            _ => return Err(self.unexpected("a value")),
        };
        self.next += 1;
        Ok((Ast { kind, start: at }, 1))
    }
}

fn too_deep(at: Pos) -> CheckError {
    CheckError::TooDeep {
        limit: MAX_DEPTH,
        at,
    }
}

/// The depth of a node whose deepest child is `child` levels deep.
fn node_depth(child: usize, at: Pos) -> Result<usize, CheckError> {
    let depth = child + 1;
    if depth > MAX_DEPTH {
        Err(too_deep(at))
    } else {
        Ok(depth)
    }
}

/// A decimal literal's value, negated when a `-` was folded into it.
fn literal(digits: &str, negative: bool, at: Pos) -> Result<i64, CheckError> {
    let out_of_range = CheckError::LiteralOutOfRange { at };
    let mut magnitude: u64 = 0;
    for d in digits.bytes() {
        magnitude = magnitude
            .checked_mul(10)
            .and_then(|m| m.checked_add(u64::from(d - b'0')))
            .ok_or_else(|| out_of_range.clone())?;
    }
    if negative {
        0i64.checked_sub_unsigned(magnitude).ok_or(out_of_range)
    } else {
        i64::try_from(magnitude).map_err(|_| out_of_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lex;

    fn parsed(source: &str) -> Result<Ast, CheckError> {
        parse(lex(source)?)
    }

    /// The tree as an s-expression, to check grouping.
    fn sexp(ast: &Ast) -> String {
        match &ast.kind {
            Kind::Int(v) => v.to_string(),
            Kind::Bool(b) => b.to_string(),
            Kind::Name(n) => n.clone(),
            Kind::Var(i) => format!("${i}"),
            Kind::Unary { op, operand, .. } => {
                let op = if *op == UnOp::Neg { "-" } else { "!" };
                format!("({op} {})", sexp(operand))
            }
            Kind::Binary { op, lhs, rhs, .. } => {
                format!("({} {} {})", op.symbol(), sexp(lhs), sexp(rhs))
            }
            Kind::If {
                cond, then, els, ..
            } => {
                format!("(if {} {} {})", sexp(cond), sexp(then), sexp(els))
            }
        }
    }

    fn grouping(source: &str) -> String {
        sexp(&parsed(source).expect(source))
    }

    fn pos(line: u32, column: u32) -> Pos {
        Pos { line, column }
    }

    #[test]
    fn precedence_and_grouping() {
        for (source, tree) in [
            ("1 + 2 * 3", "(+ 1 (* 2 3))"),
            ("(1 + 2) * 3", "(* (+ 1 2) 3)"),
            ("a - b - c", "(- (- a b) c)"),
            ("a / b % c * d", "(* (% (/ a b) c) d)"),
            ("-a * b", "(* (- a) b)"),
            ("- a - b", "(- (- a) b)"),
            ("!a && b", "(&& (! a) b)"),
            ("a || b && c", "(|| a (&& b c))"),
            ("a && b || c", "(|| (&& a b) c)"),
            ("a + 1 < b * 2 && c", "(&& (< (+ a 1) (* b 2)) c)"),
            ("if c then 1 else 2 + 3", "(if c 1 (+ 2 3))"),
            ("1 + if c then 1 else 2 + 3", "(+ 1 (if c 1 (+ 2 3)))"),
            ("if a then if b then 1 else 2 else 3", "(if a (if b 1 2) 3)"),
            ("!!a", "(! (! a))"),
            ("- -5", "(- -5)"),
            ("-5 % 3", "(% -5 3)"),
            ("a < b == true", "(== (< a b) true)"),
        ] {
            if source == "a < b == true" {
                // Comparisons don't chain, whichever operators they are.
                assert_eq!(
                    parsed(source),
                    Err(CheckError::ChainedComparison { at: pos(1, 7) })
                );
                continue;
            }
            assert_eq!(grouping(source), tree, "{source}");
        }
    }

    #[test]
    fn nodes_keep_their_positions() {
        let ast = parsed("(a +\n b) / -c").unwrap();
        assert_eq!(ast.start, pos(1, 1));
        let Kind::Binary { op, at, lhs, rhs } = ast.kind else {
            panic!("{ast:?}");
        };
        assert_eq!((op, at), (BinOp::Div, pos(2, 5)));
        assert_eq!(lhs.start, pos(1, 1));
        let Kind::Binary { at: plus, .. } = lhs.kind else {
            panic!("{lhs:?}");
        };
        assert_eq!(plus, pos(1, 4));
        assert_eq!(rhs.start, pos(2, 7));
    }

    #[test]
    fn negative_literals_are_folded() {
        assert_eq!(
            parsed("-9223372036854775808").unwrap().kind,
            Kind::Int(i64::MIN)
        );
        assert_eq!(
            parsed("-9223372036854775809"),
            Err(CheckError::LiteralOutOfRange { at: pos(1, 1) })
        );
        assert_eq!(parsed("0").unwrap().kind, Kind::Int(0));
        assert_eq!(parsed("-0").unwrap().kind, Kind::Int(0));
        assert_eq!(
            parsed(&"9".repeat(500)),
            Err(CheckError::LiteralOutOfRange { at: pos(1, 1) })
        );
    }

    #[test]
    fn depth_counts_parentheses_unary_operators_and_nodes() {
        let parens = |k: usize| format!("{}1{}", "(".repeat(k), ")".repeat(k));
        assert!(parsed(&parens(MAX_DEPTH)).is_ok());
        assert_eq!(parsed(&parens(MAX_DEPTH + 1)), Err(too_deep(pos(1, 257))));
        let nots = |k: usize| format!("{}a", "!".repeat(k));
        assert!(parsed(&nots(MAX_DEPTH - 1)).is_ok());
        assert!(matches!(
            parsed(&nots(MAX_DEPTH)),
            Err(CheckError::TooDeep { .. })
        ));
        let chain = |k: usize| format!("a{}", "+a".repeat(k));
        assert!(parsed(&chain(MAX_DEPTH - 1)).is_ok());
        assert!(matches!(
            parsed(&chain(MAX_DEPTH)),
            Err(CheckError::TooDeep { .. })
        ));
        // Deep text with nothing at the end still stops at the limit, not at the end.
        assert!(matches!(
            parsed(&"(".repeat(5000)),
            Err(CheckError::TooDeep { .. })
        ));
    }
}
