//! The lexer: text to tokens with their positions (research R2).

use crate::error::CheckError;
use crate::types::Pos;

/// A token's kind. Literals keep their digits as text, so the parser can fold a leading `-`
/// into them before deciding whether they fit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tok {
    Int(String),
    Name(String),
    If,
    Then,
    Else,
    True,
    False,
    OrOr,
    AndAnd,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    LParen,
    RParen,
}

impl Tok {
    /// The token as written, for messages.
    pub(crate) fn text(&self) -> String {
        let fixed = match self {
            Tok::Int(text) | Tok::Name(text) => return text.clone(),
            Tok::If => "if",
            Tok::Then => "then",
            Tok::Else => "else",
            Tok::True => "true",
            Tok::False => "false",
            Tok::OrOr => "||",
            Tok::AndAnd => "&&",
            Tok::EqEq => "==",
            Tok::NotEq => "!=",
            Tok::Lt => "<",
            Tok::Le => "<=",
            Tok::Gt => ">",
            Tok::Ge => ">=",
            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Star => "*",
            Tok::Slash => "/",
            Tok::Percent => "%",
            Tok::Bang => "!",
            Tok::LParen => "(",
            Tok::RParen => ")",
        };
        fixed.to_string()
    }
}

/// The words that can't be variable names.
pub(crate) const KEYWORDS: [&str; 5] = ["if", "then", "else", "true", "false"];

/// Whether `c` can start a name.
pub(crate) fn name_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Whether `c` can continue a name.
pub(crate) fn name_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub tok: Tok,
    pub at: Pos,
}

/// The tokens of a text, and the position just past its last character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Lexed {
    pub tokens: Vec<Token>,
    pub end: Pos,
}

/// Splits `source` into tokens. Whitespace (any `char::is_whitespace`) separates tokens; a line
/// ends at `\n`.
pub(crate) fn lex(source: &str) -> Result<Lexed, CheckError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut at = Pos::START;
    let mut i = 0;
    // Moves past `n` characters, none of them a line break.
    let advance = |at: &mut Pos, n: usize| {
        let n = u32::try_from(n).unwrap_or(u32::MAX);
        at.column = at.column.saturating_add(n);
    };
    while let Some(&c) = chars.get(i) {
        if c == '\n' {
            at.line = at.line.saturating_add(1);
            at.column = 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            advance(&mut at, 1);
            i += 1;
            continue;
        }
        let start = at;
        let next = chars.get(i + 1).copied();
        let (tok, len) = if c.is_ascii_digit() {
            let len = chars[i..].iter().take_while(|c| c.is_ascii_digit()).count();
            (Tok::Int(chars[i..i + len].iter().collect()), len)
        } else if name_start(c) {
            let len = 1 + chars[i + 1..]
                .iter()
                .take_while(|&&c| name_continue(c))
                .count();
            let word: String = chars[i..i + len].iter().collect();
            let tok = match word.as_str() {
                "if" => Tok::If,
                "then" => Tok::Then,
                "else" => Tok::Else,
                "true" => Tok::True,
                "false" => Tok::False,
                _ => Tok::Name(word),
            };
            (tok, len)
        } else {
            match (c, next) {
                ('|', Some('|')) => (Tok::OrOr, 2),
                ('&', Some('&')) => (Tok::AndAnd, 2),
                ('=', Some('=')) => (Tok::EqEq, 2),
                ('!', Some('=')) => (Tok::NotEq, 2),
                ('<', Some('=')) => (Tok::Le, 2),
                ('>', Some('=')) => (Tok::Ge, 2),
                ('<', _) => (Tok::Lt, 1),
                ('>', _) => (Tok::Gt, 1),
                ('!', _) => (Tok::Bang, 1),
                ('+', _) => (Tok::Plus, 1),
                ('-', _) => (Tok::Minus, 1),
                ('*', _) => (Tok::Star, 1),
                ('/', _) => (Tok::Slash, 1),
                ('%', _) => (Tok::Percent, 1),
                ('(', _) => (Tok::LParen, 1),
                (')', _) => (Tok::RParen, 1),
                _ => return Err(CheckError::InvalidCharacter { found: c, at }),
            }
        };
        tokens.push(Token { tok, at: start });
        advance(&mut at, len);
        i += len;
    }
    Ok(Lexed { tokens, end: at })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: u32, column: u32) -> Pos {
        Pos { line, column }
    }

    fn toks(source: &str) -> Vec<(Tok, Pos)> {
        lex(source)
            .expect(source)
            .tokens
            .into_iter()
            .map(|t| (t.tok, t.at))
            .collect()
    }

    #[test]
    fn every_operator() {
        let source = "|| && == != <= >= < > + - * / % ! ( )";
        let expected = [
            Tok::OrOr,
            Tok::AndAnd,
            Tok::EqEq,
            Tok::NotEq,
            Tok::Le,
            Tok::Ge,
            Tok::Lt,
            Tok::Gt,
            Tok::Plus,
            Tok::Minus,
            Tok::Star,
            Tok::Slash,
            Tok::Percent,
            Tok::Bang,
            Tok::LParen,
            Tok::RParen,
        ];
        let got = toks(source);
        assert_eq!(got.len(), expected.len());
        let mut column = 1;
        for ((tok, at), want) in got.into_iter().zip(expected) {
            assert_eq!(at, pos(1, column));
            column += u32::try_from(want.text().len()).unwrap() + 1;
            assert_eq!(tok, want);
        }
    }

    #[test]
    fn operators_without_spaces() {
        let got: Vec<Tok> = toks("a<=b!=!c>-1").into_iter().map(|t| t.0).collect();
        assert_eq!(
            got,
            [
                Tok::Name("a".into()),
                Tok::Le,
                Tok::Name("b".into()),
                Tok::NotEq,
                Tok::Bang,
                Tok::Name("c".into()),
                Tok::Gt,
                Tok::Minus,
                Tok::Int("1".into()),
            ]
        );
    }

    #[test]
    fn keywords_names_and_literals() {
        assert_eq!(
            toks("if then else true false iffy _x x2 007"),
            [
                (Tok::If, pos(1, 1)),
                (Tok::Then, pos(1, 4)),
                (Tok::Else, pos(1, 9)),
                (Tok::True, pos(1, 14)),
                (Tok::False, pos(1, 19)),
                (Tok::Name("iffy".into()), pos(1, 25)),
                (Tok::Name("_x".into()), pos(1, 30)),
                (Tok::Name("x2".into()), pos(1, 33)),
                (Tok::Int("007".into()), pos(1, 36)),
            ]
        );
        // A literal followed by letters is two tokens; the parser refuses the pair.
        assert_eq!(
            toks("2x"),
            [
                (Tok::Int("2".into()), pos(1, 1)),
                (Tok::Name("x".into()), pos(1, 2)),
            ]
        );
    }

    #[test]
    fn whitespace_and_later_lines() {
        assert_eq!(
            toks("a\t+\r\n  b\n\nc"),
            [
                (Tok::Name("a".into()), pos(1, 1)),
                (Tok::Plus, pos(1, 3)),
                (Tok::Name("b".into()), pos(2, 3)),
                (Tok::Name("c".into()), pos(4, 1)),
            ]
        );
        assert_eq!(lex("a \n").unwrap().end, pos(2, 1));
        assert_eq!(lex("year").unwrap().end, pos(1, 5));
        assert_eq!(lex("").unwrap().end, pos(1, 1));
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        assert_eq!(
            toks("année + 日付"),
            [
                (Tok::Name("année".into()), pos(1, 1)),
                (Tok::Plus, pos(1, 7)),
                (Tok::Name("日付".into()), pos(1, 9)),
            ]
        );
    }

    #[test]
    fn invalid_characters() {
        for (source, found, column) in [
            ("year # 2", '#', 6),
            ("a = b", '=', 3),
            ("a & b", '&', 3),
            ("a | b", '|', 3),
            ("½", '½', 1),
            ("1.5", '.', 2),
        ] {
            assert_eq!(
                lex(source),
                Err(CheckError::InvalidCharacter {
                    found,
                    at: pos(1, column)
                }),
                "{source}"
            );
        }
    }
}
