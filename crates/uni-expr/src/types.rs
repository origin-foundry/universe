use std::fmt;

/// The type of a value or an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// A whole number (64-bit, signed).
    Int,
    /// True or false.
    Bool,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Type::Int => "whole number",
            Type::Bool => "true/false",
        })
    }
}

/// A value: what a variable holds and what an expression gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    Int(i64),
    Bool(bool),
}

impl Value {
    /// This value's type.
    pub fn ty(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::Bool(_) => Type::Bool,
        }
    }
}

/// A variable an expression may read: its name and type, declared by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarDecl {
    pub name: String,
    pub ty: Type,
}

/// A position in an expression's text: line and column, both from 1, columns counted in
/// characters (not bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub line: u32,
    pub column: u32,
}

impl Pos {
    /// The start of the text.
    pub(crate) const START: Pos = Pos { line: 1, column: 1 };
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_display_in_words() {
        assert_eq!(Type::Int.to_string(), "whole number");
        assert_eq!(Type::Bool.to_string(), "true/false");
    }

    #[test]
    fn values_know_their_type() {
        assert_eq!(Value::Int(-3).ty(), Type::Int);
        assert_eq!(Value::Bool(true).ty(), Type::Bool);
    }

    #[test]
    fn positions_display_as_line_and_column() {
        assert_eq!(
            Pos {
                line: 2,
                column: 10
            }
            .to_string(),
            "2:10"
        );
        assert_eq!(Pos::START.to_string(), "1:1");
    }
}
