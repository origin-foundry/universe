//! Story 4: exact and safe on any input (spec 005, research R8).
//!
//! Random well-typed trees are rendered as text, checked, and evaluated; an independent
//! reference evaluator over unlimited-size integers says what the answer should be.

use num_bigint::BigInt;
use proptest::prelude::*;
use uni_expr::{CheckError, EvalError, Expr, Pos, Type, Value, VarDecl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
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

impl Op {
    fn symbol(self) -> &'static str {
        match self {
            Op::Or => "||",
            Op::And => "&&",
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Rem => "%",
        }
    }

    fn precedence(self) -> u8 {
        match self {
            Op::Or => 1,
            Op::And => 2,
            Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => 3,
            Op::Add | Op::Sub => 4,
            Op::Mul | Op::Div | Op::Rem => 5,
        }
    }
}

/// A generated expression tree, well typed by construction.
#[derive(Clone, Debug)]
enum T {
    Int(i64),
    Bool(bool),
    Year,
    N,
    Neg(Box<T>),
    Not(Box<T>),
    Bin(Op, Box<T>, Box<T>),
    If(Box<T>, Box<T>, Box<T>),
}

const UNARY: u8 = 6;
const ATOM: u8 = 7;

impl T {
    fn bin(op: Op, l: T, r: T) -> T {
        T::Bin(op, Box::new(l), Box::new(r))
    }

    fn precedence(&self) -> u8 {
        match self {
            T::Int(v) if *v < 0 => UNARY,
            T::Int(_) | T::Bool(_) | T::Year | T::N => ATOM,
            T::Neg(_) | T::Not(_) => UNARY,
            T::Bin(op, ..) => op.precedence(),
            T::If(..) => 0,
        }
    }

    /// Every compound part in parentheses.
    fn full(&self) -> String {
        match self {
            T::Int(v) if *v < 0 => format!("({v})"),
            T::Int(v) => v.to_string(),
            T::Bool(b) => b.to_string(),
            T::Year => "year".into(),
            T::N => "n".into(),
            T::Neg(x) => format!("(-{})", x.full()),
            T::Not(x) => format!("(!{})", x.full()),
            T::Bin(op, l, r) => format!("({} {} {})", l.full(), op.symbol(), r.full()),
            T::If(c, t, e) => format!("(if {} then {} else {})", c.full(), t.full(), e.full()),
        }
    }

    /// As few parentheses as precedence allows.
    fn minimal(&self) -> String {
        fn at_least(x: &T, need: u8) -> String {
            if x.precedence() < need {
                format!("({})", x.minimal())
            } else {
                x.minimal()
            }
        }
        match self {
            T::Neg(x) => format!("-{}", at_least(x, UNARY)),
            T::Not(x) => format!("!{}", at_least(x, UNARY)),
            T::Bin(op, l, r) => {
                let p = op.precedence();
                // Comparisons don't chain, so neither side may be a bare comparison.
                let left = if p == 3 { p + 1 } else { p };
                format!(
                    "{} {} {}",
                    at_least(l, left),
                    op.symbol(),
                    at_least(r, p + 1)
                )
            }
            T::If(c, t, e) => format!(
                "if {} then {} else {}",
                c.minimal(),
                t.minimal(),
                e.minimal()
            ),
            _ => self.full(),
        }
    }

    /// Every `year` replaced by `(a·year + b) % m`, so periods are common.
    fn wrap_year(&self, a: i64, b: i64, m: i64) -> T {
        let w = |x: &T| Box::new(x.wrap_year(a, b, m));
        match self {
            T::Year => T::bin(
                Op::Rem,
                T::bin(Op::Add, T::bin(Op::Mul, T::Int(a), T::Year), T::Int(b)),
                T::Int(m),
            ),
            T::Neg(x) => T::Neg(w(x)),
            T::Not(x) => T::Not(w(x)),
            T::Bin(op, l, r) => T::Bin(*op, w(l), w(r)),
            T::If(c, t, e) => T::If(w(c), w(t), w(e)),
            leaf => leaf.clone(),
        }
    }
}

fn literal() -> impl Strategy<Value = i64> {
    prop_oneof![
        8 => -1000i64..=1000,
        1 => prop::sample::select(vec![
            i64::MAX,
            i64::MAX - 1,
            i64::MIN,
            i64::MIN + 1,
            4_611_686_018_427_387_904,
            -4_611_686_018_427_387_904,
            3_037_000_499,
            -3_037_000_500,
        ]),
    ]
}

/// Whole-number and true/false trees up to `depth` levels below the root.
fn trees(depth: usize) -> (BoxedStrategy<T>, BoxedStrategy<T>) {
    let mut ints = prop_oneof![
        2 => literal().prop_map(T::Int),
        2 => Just(T::Year),
        1 => Just(T::N),
    ]
    .boxed();
    let mut bools = any::<bool>().prop_map(T::Bool).boxed();
    for _ in 0..depth {
        let (i, b) = (ints.clone(), bools.clone());
        let int_ops = prop::sample::select(vec![Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Rem]);
        let cmp_ops = prop::sample::select(vec![Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge]);
        let bool_ops = prop::sample::select(vec![Op::And, Op::Or, Op::Eq, Op::Ne]);
        let next_ints = prop_oneof![
            3 => i.clone(),
            1 => i.clone().prop_map(|x| T::Neg(Box::new(x))),
            4 => (int_ops, i.clone(), i.clone()).prop_map(|(op, l, r)| T::bin(op, l, r)),
            1 => (b.clone(), i.clone(), i.clone())
                .prop_map(|(c, t, e)| T::If(Box::new(c), Box::new(t), Box::new(e))),
        ]
        .boxed();
        let next_bools = prop_oneof![
            2 => b.clone(),
            1 => b.clone().prop_map(|x| T::Not(Box::new(x))),
            4 => (cmp_ops, i.clone(), i.clone()).prop_map(|(op, l, r)| T::bin(op, l, r)),
            2 => (bool_ops, b.clone(), b.clone()).prop_map(|(op, l, r)| T::bin(op, l, r)),
            1 => (b.clone(), b.clone(), b.clone())
                .prop_map(|(c, t, e)| T::If(Box::new(c), Box::new(t), Box::new(e))),
        ]
        .boxed();
        ints = next_ints;
        bools = next_bools;
    }
    (ints, bools)
}

/// A typed tree: `(tree, its type)`.
fn typed_tree() -> impl Strategy<Value = (T, Type)> {
    let (ints, bools) = trees(8);
    prop_oneof![
        ints.prop_map(|t| (t, Type::Int)),
        bools.prop_map(|t| (t, Type::Bool)),
    ]
}

fn variable_value() -> impl Strategy<Value = i64> {
    prop_oneof![
        4 => -1000i64..=1000,
        1 => -1_000_000_000_000i64..=1_000_000_000_000,
        1 => any::<i64>(),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ref {
    Int(BigInt),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RefError {
    DivisionByZero,
    OutOfRange,
}

/// Floored quotient and remainder, by definition: `q = ⌊a / b⌋`, `r = a − b·q`.
fn floored(a: &BigInt, b: &BigInt) -> (BigInt, BigInt) {
    let zero = BigInt::from(0);
    let mut q = a / b; // truncates towards zero
    let mut r = a - b * &q;
    if r != zero && ((r < zero) != (*b < zero)) {
        q -= 1;
        r += b;
    }
    (q, r)
}

/// The reference evaluator: unlimited-size integers; any whole number it computes that
/// doesn't fit in 64 bits is out of range.
fn reference(t: &T, year: i64, n: i64) -> Result<Ref, RefError> {
    let fits = |v: BigInt| {
        if v >= BigInt::from(i64::MIN) && v <= BigInt::from(i64::MAX) {
            Ok(Ref::Int(v))
        } else {
            Err(RefError::OutOfRange)
        }
    };
    let int = |t: &T| match reference(t, year, n)? {
        Ref::Int(v) => Ok(v),
        Ref::Bool(_) => unreachable!("well typed"),
    };
    let boolean = |t: &T| match reference(t, year, n)? {
        Ref::Bool(b) => Ok(b),
        Ref::Int(_) => unreachable!("well typed"),
    };
    match t {
        T::Int(v) => Ok(Ref::Int(BigInt::from(*v))),
        T::Bool(b) => Ok(Ref::Bool(*b)),
        T::Year => Ok(Ref::Int(BigInt::from(year))),
        T::N => Ok(Ref::Int(BigInt::from(n))),
        T::Neg(x) => fits(-int(x)?),
        T::Not(x) => Ok(Ref::Bool(!boolean(x)?)),
        T::If(c, a, b) => {
            if boolean(c)? {
                reference(a, year, n)
            } else {
                reference(b, year, n)
            }
        }
        T::Bin(Op::And, l, r) => Ok(Ref::Bool(boolean(l)? && boolean(r)?)),
        T::Bin(Op::Or, l, r) => Ok(Ref::Bool(boolean(l)? || boolean(r)?)),
        T::Bin(op @ (Op::Eq | Op::Ne), l, r) => {
            let (a, b) = (reference(l, year, n)?, reference(r, year, n)?);
            Ok(Ref::Bool((a == b) == (*op == Op::Eq)))
        }
        T::Bin(op, l, r) => {
            let (a, b) = (int(l)?, int(r)?);
            match op {
                Op::Lt => Ok(Ref::Bool(a < b)),
                Op::Le => Ok(Ref::Bool(a <= b)),
                Op::Gt => Ok(Ref::Bool(a > b)),
                Op::Ge => Ok(Ref::Bool(a >= b)),
                Op::Add => fits(a + b),
                Op::Sub => fits(a - b),
                Op::Mul => fits(a * b),
                Op::Div | Op::Rem if b == BigInt::from(0) => Err(RefError::DivisionByZero),
                Op::Div => fits(floored(&a, &b).0),
                Op::Rem => fits(floored(&a, &b).1),
                Op::And | Op::Or | Op::Eq | Op::Ne => unreachable!("handled above"),
            }
        }
    }
}

fn vars() -> Vec<VarDecl> {
    vec![
        VarDecl {
            name: "year".into(),
            ty: Type::Int,
        },
        VarDecl {
            name: "n".into(),
            ty: Type::Int,
        },
    ]
}

/// The language's result in the reference's terms.
fn as_ref(result: Result<Value, EvalError>) -> Result<Ref, RefError> {
    match result {
        Ok(Value::Int(v)) => Ok(Ref::Int(BigInt::from(v))),
        Ok(Value::Bool(b)) => Ok(Ref::Bool(b)),
        Err(EvalError::DivisionByZero { .. }) => Err(RefError::DivisionByZero),
        Err(EvalError::OutOfRange { .. }) => Err(RefError::OutOfRange),
        Err(e @ EvalError::Inputs { .. }) => panic!("{e}"),
    }
}

/// Every position a fault in `source` may point at: each character's, and one past the end.
fn positions(source: &str) -> Vec<Pos> {
    let mut at = Pos { line: 1, column: 1 };
    let mut all = Vec::new();
    for c in source.chars() {
        all.push(at);
        if c == '\n' {
            at = Pos {
                line: at.line + 1,
                column: 1,
            };
        } else {
            at.column += 1;
        }
    }
    all.push(at);
    all
}

fn error_position(error: &CheckError) -> Option<Pos> {
    match error {
        CheckError::InvalidVariable { .. } => None,
        CheckError::InvalidCharacter { at, .. }
        | CheckError::Unexpected { at, .. }
        | CheckError::LiteralOutOfRange { at }
        | CheckError::ChainedComparison { at }
        | CheckError::TooDeep { at, .. }
        | CheckError::UnknownVariable { at, .. }
        | CheckError::TypeMismatch { at, .. } => Some(*at),
    }
}

/// Text that looks a little like expressions, and text that doesn't.
fn any_text() -> impl Strategy<Value = String> {
    let alphabet: Vec<char> =
        "()+-*/%!<>=&| \n\r\tyearnifthenelsetruefalse0123456789#._éé日\u{2028}"
            .chars()
            .collect();
    prop_oneof![
        prop::collection::vec(any::<char>(), 0..2000).prop_map(|v| v.into_iter().collect()),
        prop::collection::vec(prop::sample::select(alphabet), 0..2000)
            .prop_map(|v| v.into_iter().collect()),
        (0usize..=5000, 0usize..4).prop_map(|(k, shape)| match shape {
            0 => "(".repeat(k),
            1 => format!("{}1{}", "(".repeat(k), ")".repeat(k)),
            2 => format!("{}year", "- ".repeat(k)),
            _ => format!("{}1", "1 + ".repeat(k)),
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Story 4 scenario 1: evaluation matches the reference, including when it reports out of
    /// range or division by zero; both renderings check and agree.
    #[test]
    fn evaluation_matches_the_reference(
        (tree, ty) in typed_tree(),
        year in variable_value(),
        n in variable_value(),
    ) {
        let expected = reference(&tree, year, n);
        for source in [tree.full(), tree.minimal()] {
            let expr = Expr::check(&source, &vars(), ty)
                .map_err(|e| TestCaseError::fail(format!("{source}: {e}")))?;
            let got = as_ref(expr.eval(&[Value::Int(year), Value::Int(n)]));
            prop_assert_eq!(&got, &expected, "{} at year {}, n {}", source, year, n);
        }
    }

    /// Story 4 scenario 2: wherever a period is worked out, the reference gives the same value
    /// or the same division by zero at y and y + P, unless either is out of range.
    #[test]
    fn periods_hold(
        (tree, ty) in typed_tree(),
        wrap in prop::option::of((-3i64..=3, -10i64..=10, -12i64..=12)),
        y in -1_000_000i64..=1_000_000,
        n in variable_value(),
    ) {
        let tree = match wrap {
            Some((a, b, m)) => tree.wrap_year(a, b, m),
            None => tree,
        };
        let source = tree.minimal();
        let expr = Expr::check(&source, &vars(), ty)
            .map_err(|e| TestCaseError::fail(format!("{source}: {e}")))?;
        if let Ok(p) = expr.period("year", 10_000) {
            prop_assert!((1..=10_000).contains(&p));
            let here = reference(&tree, y, n);
            let there = reference(&tree, y + p, n);
            if here != Err(RefError::OutOfRange) && there != Err(RefError::OutOfRange) {
                prop_assert_eq!(here, there, "{} at {} and {} + {}", source, y, y, p);
            }
        }
    }

    /// Story 4 scenario 3: any text is checked without panicking, and a refusal points inside
    /// the text or just past its end. Whatever checks can be evaluated and analysed safely.
    #[test]
    fn any_text_is_answered(source in any_text(), want_bool in any::<bool>(), y in any::<i64>()) {
        let ty = if want_bool { Type::Bool } else { Type::Int };
        match Expr::check(&source, &vars(), ty) {
            Ok(expr) => {
                prop_assert_eq!(expr.source(), source.as_str());
                let _ = expr.eval(&[Value::Int(y), Value::Int(y)]);
                let _ = expr.period("year", 10_000);
            }
            Err(error) => {
                let at = error_position(&error).expect("declarations are valid");
                prop_assert!(positions(&source).contains(&at), "{:?} in {:?}", error, source);
            }
        }
    }
}
