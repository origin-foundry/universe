//! Story 2: evaluating an expression (spec 005).

use uni_expr::{EvalError, Expr, Pos, Type, Value, VarDecl};

const ASTER: &str = "year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)";
const GREGORIAN: &str = "year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)";

fn year() -> Vec<VarDecl> {
    vec![VarDecl {
        name: "year".into(),
        ty: Type::Int,
    }]
}

fn checked(source: &str, result: Type) -> Expr {
    Expr::check(source, &year(), result).expect(source)
}

fn at(column: u32) -> Pos {
    Pos { line: 1, column }
}

fn eval(source: &str, result: Type, y: i64) -> Result<Value, EvalError> {
    checked(source, result).eval(&[Value::Int(y)])
}

#[test]
fn leap_rules() {
    let aster = checked(ASTER, Type::Bool);
    for (y, leap) in [
        (5, true),
        (100, false),
        (500, true),
        (0, true),
        (-5, true),
        (1, false),
    ] {
        assert_eq!(aster.eval(&[Value::Int(y)]), Ok(Value::Bool(leap)), "{y}");
    }
    let gregorian = checked(GREGORIAN, Type::Bool);
    for (y, leap) in [
        (2000, true),
        (1900, false),
        (2024, true),
        (2023, false),
        (0, true),
        (-4, true),
        (-100, false),
    ] {
        assert_eq!(
            gregorian.eval(&[Value::Int(y)]),
            Ok(Value::Bool(leap)),
            "{y}"
        );
    }
}

#[test]
fn division_is_floored() {
    for (source, value) in [("-7 % 4", 1), ("-7 / 4", -2), ("7 % -4", -1), ("7 / -4", -2)] {
        assert_eq!(eval(source, Type::Int, 0), Ok(Value::Int(value)), "{source}");
    }
}

#[test]
fn every_operator() {
    for (source, value) in [
        ("year + 2", Value::Int(9)),
        ("year - 2", Value::Int(5)),
        ("year * 2", Value::Int(14)),
        ("-year", Value::Int(-7)),
        ("year < 7", Value::Bool(false)),
        ("year <= 7", Value::Bool(true)),
        ("year > 6", Value::Bool(true)),
        ("year >= 8", Value::Bool(false)),
        ("year != 7", Value::Bool(false)),
        ("!(year == 7)", Value::Bool(false)),
        ("(year == 7) == true", Value::Bool(true)),
        ("true != false", Value::Bool(true)),
        ("if year > 0 then year else -year", Value::Int(7)),
    ] {
        let ty = value.ty();
        assert_eq!(eval(source, ty, 7), Ok(value), "{source}");
    }
}

#[test]
fn if_takes_one_branch() {
    let rule = checked(
        "if year % 2 == 0 then year / 2 % 3 == 0 else false",
        Type::Bool,
    );
    for (y, value) in [(6, true), (4, false), (3, false)] {
        assert_eq!(rule.eval(&[Value::Int(y)]), Ok(Value::Bool(value)), "{y}");
    }
}

#[test]
fn division_by_zero_points_at_the_operator() {
    let rule = checked("100 / (year % 7) == 3", Type::Bool);
    assert_eq!(
        rule.eval(&[Value::Int(14)]),
        Err(EvalError::DivisionByZero { at: at(5) })
    );
    assert_eq!(rule.eval(&[Value::Int(1)]), Ok(Value::Bool(false)));
    assert_eq!(
        eval("year % 0", Type::Int, 3),
        Err(EvalError::DivisionByZero { at: at(6) })
    );
}

#[test]
fn results_outside_64_bits_are_refused() {
    assert_eq!(
        eval("year * 4611686018427387904", Type::Int, 2),
        Err(EvalError::OutOfRange { at: at(6) })
    );
    assert_eq!(
        eval("-year", Type::Int, i64::MIN),
        Err(EvalError::OutOfRange { at: at(1) })
    );
    assert_eq!(
        eval("year / -1", Type::Int, i64::MIN),
        Err(EvalError::OutOfRange { at: at(6) })
    );
    assert_eq!(
        eval("year % -1", Type::Int, i64::MIN),
        Ok(Value::Int(0))
    );
    assert_eq!(
        eval("year + 1", Type::Int, i64::MAX),
        Err(EvalError::OutOfRange { at: at(6) })
    );
    assert_eq!(
        eval("year - 1", Type::Int, i64::MIN),
        Err(EvalError::OutOfRange { at: at(6) })
    );
}

#[test]
fn only_what_is_needed_is_evaluated() {
    assert_eq!(
        eval("year % 2 == 0 && 100 / (year % 7) == 3", Type::Bool, 7),
        Ok(Value::Bool(false))
    );
    assert_eq!(
        eval("year % 2 == 1 || 1 / 0 == 0", Type::Bool, 7),
        Ok(Value::Bool(true))
    );
    assert_eq!(
        eval("if true then 1 else 1 / 0", Type::Int, 7),
        Ok(Value::Int(1))
    );
    assert_eq!(
        eval("if false then 1 / 0 else 2", Type::Int, 7),
        Ok(Value::Int(2))
    );
    // When the left side doesn't decide, the right side is evaluated, errors and all.
    assert_eq!(
        eval("year % 2 == 1 && 1 / 0 == 0", Type::Bool, 7),
        Err(EvalError::DivisionByZero { at: at(20) })
    );
}

#[test]
fn values_follow_the_declarations() {
    let vars = [
        VarDecl {
            name: "year".into(),
            ty: Type::Int,
        },
        VarDecl {
            name: "leap".into(),
            ty: Type::Bool,
        },
    ];
    let expr = Expr::check("if leap then year + 1 else year", &vars, Type::Int).unwrap();
    assert_eq!(
        expr.eval(&[Value::Int(4), Value::Bool(true)]),
        Ok(Value::Int(5))
    );
    assert_eq!(
        expr.eval(&[Value::Int(4), Value::Bool(false)]),
        Ok(Value::Int(4))
    );
    for values in [
        vec![Value::Int(4)],
        vec![Value::Int(4), Value::Bool(true), Value::Int(1)],
        vec![Value::Bool(true), Value::Bool(true)],
        vec![Value::Bool(true), Value::Int(4)],
        vec![],
    ] {
        assert_eq!(
            expr.eval(&values),
            Err(EvalError::Inputs {
                expected: 2,
                given: values.len()
            }),
            "{values:?}"
        );
    }
}
