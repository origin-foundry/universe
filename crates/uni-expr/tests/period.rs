//! Story 3: working out how an expression repeats (spec 005, research R6).

use uni_expr::{Expr, PeriodError, Pos, Type, Value, VarDecl};

const LIMIT: i64 = 100_000;

fn vars() -> Vec<VarDecl> {
    vec![
        VarDecl {
            name: "turn".into(),
            ty: Type::Int,
        },
        VarDecl {
            name: "n".into(),
            ty: Type::Int,
        },
        VarDecl {
            name: "flag".into(),
            ty: Type::Bool,
        },
    ]
}

fn checked(source: &str) -> Expr {
    Expr::check(source, &vars(), Type::Bool).expect(source)
}

fn period(source: &str, limit: i64) -> Result<i64, PeriodError> {
    checked(source).period("turn", limit)
}

fn at(column: u32) -> Pos {
    Pos { line: 1, column }
}

/// Story 3 scenario 7: the value (or division-by-zero error) at y and y + P is the same, for
/// every y from −2P to 2P and a few values of the other variables.
fn assert_repeats(expr: &Expr, p: i64) {
    for n in [-3, 0, 2, 7] {
        for flag in [false, true] {
            for y in -2 * p..=2 * p {
                let at = |y| expr.eval(&[Value::Int(y), Value::Int(n), Value::Bool(flag)]);
                assert_eq!(at(y), at(y + p), "{} at {y}", expr.source());
            }
        }
    }
}

#[test]
fn periods_of_repeating_expressions() {
    for (source, p) in [
        ("turn % 5 == 0 && (turn % 100 != 0 || turn % 500 == 0)", 500),
        ("turn % 4 == 0 && (turn % 100 != 0 || turn % 400 == 0)", 400),
        ("(11 * turn + 14) % 30 < 11", 30),
        ("true", 1),
        ("3 > 2", 1),
        ("turn % 4 == 0 || turn % 6 == 0", 12),
        ("(turn + 3) % 4 == 0", 4),
        ("(-2 * turn) % 6 == 0", 6),
        ("turn % -4 == 0", 4),
        ("turn % 4 == n", 4),
        ("flag && turn % 3 == 0", 3),
        ("if turn % 2 == 0 then turn % 3 == 0 else n > 0", 6),
        ("(turn - turn) / 0 == 1", 1),
        ("(3 * turn - 2 * turn - turn + 5) % 3 == 2", 1),
        ("turn % 2 == 0 && 100 / (turn % 7) == 3", 14),
    ] {
        let expr = checked(source);
        assert_eq!(expr.period("turn", LIMIT), Ok(p), "{source}");
        assert_repeats(&expr, p);
    }
}

#[test]
fn parts_that_stop_repeating_are_named() {
    for (source, column) in [
        ("turn / 7 % 2 == 0", 6),
        ("turn * turn % 3 == 0", 6),
        ("turn > 100", 6),
        ("turn % (turn % 3 + 2) == 0", 6),
        ("(if turn % 2 == 0 then turn else 0) % 4 == 0", 2),
        ("turn + n % 2 == 0", 6),
        ("turn == turn", 6),
        ("turn % n == 0", 6),
    ] {
        assert_eq!(
            period(source, LIMIT),
            Err(PeriodError::NotRepeating { at: at(column) }),
            "{source}"
        );
    }
}

#[test]
fn periods_over_the_limit_are_refused() {
    let source = "turn % 7 == 0 && turn % 11 == 0 && turn % 13 == 0";
    assert_eq!(
        period(source, 1_000),
        Err(PeriodError::OverLimit { limit: 1_000 })
    );
    assert_eq!(period(source, 1_001), Ok(1_001));
    assert_eq!(
        period("turn % 100001 == 0", LIMIT),
        Err(PeriodError::OverLimit { limit: LIMIT })
    );
    assert_eq!(
        period("turn % -9223372036854775808 == 0", i64::MAX),
        Err(PeriodError::OverLimit { limit: i64::MAX })
    );
}

#[test]
fn the_variable_and_limit_are_checked() {
    let expr = checked("turn % 4 == 0");
    assert_eq!(
        expr.period("phase", LIMIT),
        Err(PeriodError::NotAWholeNumber {
            name: "phase".into()
        })
    );
    assert_eq!(
        expr.period("flag", LIMIT),
        Err(PeriodError::NotAWholeNumber {
            name: "flag".into()
        })
    );
    assert_eq!(
        expr.period("turn", 0),
        Err(PeriodError::BadLimit { limit: 0 })
    );
    assert_eq!(
        expr.period("turn", -5),
        Err(PeriodError::BadLimit { limit: -5 })
    );
    // Any whole-number variable can be the one that counts up.
    assert_eq!(expr.period("n", LIMIT), Ok(1));
    assert_eq!(checked("n % 6 == turn % 4").period("n", LIMIT), Ok(6));
}

#[test]
fn whole_number_results_repeat_too() {
    let int = |source: &str| Expr::check(source, &vars(), Type::Int).expect(source);
    assert_eq!(int("turn % 7 + n").period("turn", LIMIT), Ok(7));
    assert_eq!(
        int("turn + 1").period("turn", LIMIT),
        Err(PeriodError::NotRepeating { at: at(6) })
    );
    assert_eq!(
        int("turn").period("turn", LIMIT),
        Err(PeriodError::NotRepeating { at: at(1) })
    );
}
