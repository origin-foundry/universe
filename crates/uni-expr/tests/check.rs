//! Story 1: checking an expression (spec 005, quickstart.md).

use uni_expr::{CheckError, Expr, MAX_DEPTH, Pos, Type, VarDecl};

const ASTER: &str = "turn % 5 == 0 && (turn % 100 != 0 || turn % 500 == 0)";
/// The Gregorian rule, with its variable called `turn` like the other tests' (ADR 0009).
const GREGORIAN: &str = "turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)";

fn turn() -> Vec<VarDecl> {
    vec![VarDecl {
        name: "turn".into(),
        ty: Type::Int,
    }]
}

fn pos(line: u32, column: u32) -> Pos {
    Pos { line, column }
}

fn refuse(source: &str, result: Type) -> CheckError {
    Expr::check(source, &turn(), result).expect_err(source)
}

fn mismatch(context: &'static str, expected: Type, found: Type, at: Pos) -> CheckError {
    CheckError::TypeMismatch {
        context,
        expected,
        found,
        at,
    }
}

#[test]
fn adjustment_rules_are_accepted_and_keep_their_text() {
    for source in [
        ASTER,
        GREGORIAN,
        "if turn % 2 == 0 then turn / 2 % 3 == 0 else false",
    ] {
        let expr = Expr::check(source, &turn(), Type::Bool).expect(source);
        assert_eq!(expr.source(), source);
        assert_eq!(expr.ty(), Type::Bool);
        assert_eq!(expr.vars(), turn().as_slice());
    }
}

#[test]
fn a_result_of_the_wrong_type_is_refused() {
    assert_eq!(
        refuse("turn % 4", Type::Bool),
        mismatch("result", Type::Bool, Type::Int, pos(1, 1))
    );
    assert_eq!(
        refuse("turn == 4", Type::Int),
        mismatch("result", Type::Int, Type::Bool, pos(1, 1))
    );
}

#[test]
fn an_unfinished_expression_is_refused_at_its_end() {
    assert_eq!(
        refuse("turn % 4 == 0 &&", Type::Bool),
        CheckError::Unexpected {
            found: None,
            expected: "a value",
            at: pos(1, 17),
        }
    );
}

#[test]
fn an_unknown_variable_is_refused() {
    assert_eq!(
        refuse("phase == 2", Type::Bool),
        CheckError::UnknownVariable {
            name: "phase".into(),
            at: pos(1, 1),
        }
    );
}

#[test]
fn operands_of_the_wrong_type_are_refused_where_they_start() {
    assert_eq!(
        refuse("turn + true", Type::Int),
        mismatch("+", Type::Int, Type::Bool, pos(1, 8))
    );
    assert_eq!(
        refuse("!turn", Type::Bool),
        mismatch("!", Type::Bool, Type::Int, pos(1, 2))
    );
    assert_eq!(
        refuse("if turn then 1 else 2", Type::Int),
        mismatch("if", Type::Bool, Type::Int, pos(1, 4))
    );
    assert_eq!(
        refuse("if true then 1 else false", Type::Int),
        mismatch("then/else", Type::Int, Type::Bool, pos(1, 21))
    );
    assert_eq!(
        refuse("1 == true", Type::Bool),
        mismatch("==", Type::Int, Type::Bool, pos(1, 6))
    );
    assert_eq!(
        refuse("true && 1", Type::Bool),
        mismatch("&&", Type::Bool, Type::Int, pos(1, 9))
    );
    assert_eq!(
        refuse("true < 1", Type::Bool),
        mismatch("<", Type::Int, Type::Bool, pos(1, 1))
    );
    assert_eq!(
        refuse("-true", Type::Int),
        mismatch("-", Type::Int, Type::Bool, pos(1, 2))
    );
}

#[test]
fn positions_count_lines() {
    assert_eq!(
        refuse("turn % 4 == 0 &&\n  turn + true", Type::Bool),
        mismatch("+", Type::Int, Type::Bool, pos(2, 10))
    );
}

#[test]
fn comparisons_do_not_chain() {
    assert_eq!(
        refuse("1 < turn < 5", Type::Bool),
        CheckError::ChainedComparison { at: pos(1, 10) }
    );
    assert_eq!(
        refuse("turn == 1 == true", Type::Bool),
        CheckError::ChainedComparison { at: pos(1, 11) }
    );
    // Parentheses make it a single comparison again (and here a type error).
    assert_eq!(
        refuse("(1 < turn) < 5", Type::Bool),
        mismatch("<", Type::Int, Type::Bool, pos(1, 1))
    );
}

#[test]
fn literals_must_fit_in_64_bits() {
    assert_eq!(
        refuse("99999999999999999999", Type::Int),
        CheckError::LiteralOutOfRange { at: pos(1, 1) }
    );
    assert_eq!(
        refuse("9223372036854775808", Type::Int),
        CheckError::LiteralOutOfRange { at: pos(1, 1) }
    );
    assert_eq!(
        refuse("turn + 9223372036854775808", Type::Int),
        CheckError::LiteralOutOfRange { at: pos(1, 8) }
    );
    assert!(Expr::check("-9223372036854775808", &turn(), Type::Int).is_ok());
    assert!(Expr::check("9223372036854775807", &turn(), Type::Int).is_ok());
}

#[test]
fn stray_characters_and_tokens_are_refused() {
    assert_eq!(
        refuse("turn # 2", Type::Int),
        CheckError::InvalidCharacter {
            found: '#',
            at: pos(1, 6),
        }
    );
    assert_eq!(
        refuse("turn = 2", Type::Bool),
        CheckError::InvalidCharacter {
            found: '=',
            at: pos(1, 6),
        }
    );
    assert_eq!(
        refuse("(turn", Type::Int),
        CheckError::Unexpected {
            found: None,
            expected: "`)`",
            at: pos(1, 6),
        }
    );
    assert_eq!(
        refuse("turn)", Type::Int),
        CheckError::Unexpected {
            found: Some(")".into()),
            expected: "an operator or the end",
            at: pos(1, 5),
        }
    );
    assert_eq!(
        refuse("", Type::Int),
        CheckError::Unexpected {
            found: None,
            expected: "a value",
            at: pos(1, 1),
        }
    );
    assert_eq!(
        refuse("if true 1 else 2", Type::Int),
        CheckError::Unexpected {
            found: Some("1".into()),
            expected: "`then`",
            at: pos(1, 9),
        }
    );
    assert_eq!(
        refuse("if true then 1", Type::Int),
        CheckError::Unexpected {
            found: None,
            expected: "`else`",
            at: pos(1, 15),
        }
    );
}

#[test]
fn names_in_any_script_count_characters() {
    let vars = [VarDecl {
        name: "année".into(),
        ty: Type::Int,
    }];
    assert!(Expr::check("année % 4 == 0", &vars, Type::Bool).is_ok());
    assert_eq!(
        Expr::check("année + true", &vars, Type::Int),
        Err(mismatch("+", Type::Int, Type::Bool, pos(1, 9)))
    );
}

#[test]
fn declarations_are_checked() {
    let int = |name: &str| VarDecl {
        name: name.into(),
        ty: Type::Int,
    };
    for (vars, name) in [
        (vec![int("if")], "if"),
        (vec![int("true")], "true"),
        (vec![int("2x")], "2x"),
        (vec![int("")], ""),
        (vec![int("a-b")], "a-b"),
        (vec![int("turn"), int("turn")], "turn"),
    ] {
        match Expr::check("1", &vars, Type::Int) {
            Err(CheckError::InvalidVariable { name: got, reason }) => {
                assert_eq!(got, name);
                assert!(!reason.is_empty());
            }
            other => panic!("{name:?}: {other:?}"),
        }
    }
    assert!(Expr::check("_x + x_2", &[int("_x"), int("x_2")], Type::Int).is_ok());
}

#[test]
fn nesting_is_limited() {
    let parens = |k: usize| format!("{}1{}", "(".repeat(k), ")".repeat(k));
    assert!(Expr::check(&parens(200), &turn(), Type::Int).is_ok());
    assert!(matches!(
        Expr::check(&parens(257), &turn(), Type::Int),
        Err(CheckError::TooDeep {
            limit: MAX_DEPTH,
            ..
        })
    ));
    let chain = format!("turn{}", " + 1".repeat(300));
    assert!(matches!(
        Expr::check(&chain, &turn(), Type::Int),
        Err(CheckError::TooDeep { limit: 256, .. })
    ));
    let negations = format!("{}turn", "- ".repeat(300));
    assert!(matches!(
        Expr::check(&negations, &turn(), Type::Int),
        Err(CheckError::TooDeep { limit: 256, .. })
    ));
    let short_chain = format!("turn{}", " + 1".repeat(200));
    assert!(Expr::check(&short_chain, &turn(), Type::Int).is_ok());
}
