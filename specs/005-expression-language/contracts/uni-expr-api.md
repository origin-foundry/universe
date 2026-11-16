# Contract: `uni-expr` public API

The Rust API that leap rules (spec 006) and later features build on. Names are fixed by this contract; doc comments, private helpers and the internal tree are up to the build. Field rules and messages are in [data-model.md](../data-model.md).

## Types and values

```rust
pub enum Type { Int, Bool }                              // Display: "whole number", "true/false"
pub enum Value { Int(i64), Bool(bool) }
pub struct VarDecl { pub name: String, pub ty: Type }
pub struct Pos { pub line: u32, pub column: u32 }        // both from 1; columns in chars
// all derive: Clone, Copy (except VarDecl: Clone only), Debug, PartialEq, Eq, Hash

pub const MAX_DEPTH: usize = 256;                        // research R4
```

## Expressions

```rust
impl Expr {
    /// Parses and type-checks `source` against `vars`, requiring a result of type `result`
    /// (FR-002, FR-003).
    pub fn check(source: &str, vars: &[VarDecl], result: Type) -> Result<Expr, CheckError>;
    pub fn source(&self) -> &str;                        // FR-004
    pub fn ty(&self) -> Type;
    pub fn vars(&self) -> &[VarDecl];

    /// Evaluates with one value per declared variable, in declaration order (FR-005 to FR-007).
    pub fn eval(&self, values: &[Value]) -> Result<Value, EvalError>;

    /// A period P, 1 ≤ P ≤ limit, in the whole-number variable `var` (FR-008, FR-009;
    /// research R6). Safe, not necessarily the smallest.
    pub fn period(&self, var: &str, limit: i64) -> Result<i64, PeriodError>;
}
// Expr derives: Clone, Debug, PartialEq, Eq
```

## Grammar

```text
expr    = or
or      = and { "||" and }
and     = cmp { "&&" cmp }
cmp     = sum [ ("==" | "!=" | "<" | "<=" | ">" | ">=") sum ]     no chaining
sum     = product { ("+" | "-") product }
product = unary { ("*" | "/" | "%") unary }
unary   = ("-" | "!") unary | atom
atom    = INT | "true" | "false" | NAME | "(" expr ")"
        | "if" expr "then" expr "else" expr
INT     = digit { digit }                   fits i64; "-" directly before it is folded in
NAME    = (letter | "_") { letter | digit | "_" }, not a keyword     any script
keyword = "if" | "then" | "else" | "true" | "false"
```

Whitespace (any Unicode whitespace, including line breaks) may appear between tokens. `/` and `%` are floored: `/` rounds towards minus infinity and `%` takes the sign of the divisor.

## Errors

```rust
pub enum CheckError {
    InvalidVariable { name: String, reason: &'static str },
    InvalidCharacter { found: char, at: Pos },
    Unexpected { found: Option<String>, expected: &'static str, at: Pos },
    LiteralOutOfRange { at: Pos },
    ChainedComparison { at: Pos },
    TooDeep { limit: usize, at: Pos },
    UnknownVariable { name: String, at: Pos },
    TypeMismatch { context: &'static str, expected: Type, found: Type, at: Pos },
}
pub enum EvalError {
    DivisionByZero { at: Pos },
    OutOfRange { at: Pos },
    Inputs { expected: usize, given: usize },
}
pub enum PeriodError {
    NotRepeating { at: Pos },
    OverLimit { limit: i64 },
    NotAWholeNumber { name: String },
    BadLimit { limit: i64 },
}
// all: Clone, Debug, PartialEq, Eq; Display (messages in data-model.md) + std::error::Error
```

## Example

```rust
let vars = [VarDecl { name: "year".into(), ty: Type::Int }];
let rule = Expr::check("year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)", &vars, Type::Bool)?;
assert_eq!(rule.eval(&[Value::Int(2000)])?, Value::Bool(true));
assert_eq!(rule.eval(&[Value::Int(1900)])?, Value::Bool(false));
assert_eq!(rule.period("year", 100_000)?, 400);
```
