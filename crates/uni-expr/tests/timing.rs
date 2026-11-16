//! SC-004: evaluating a leap rule takes under a microsecond. Run by hand in release mode:
//! `cargo test -p uni-expr --release -- --ignored`.

use std::time::{Duration, Instant};

use uni_expr::{Expr, Type, Value, VarDecl};

#[test]
#[ignore = "timing; run in release mode"]
fn a_million_evaluations_take_under_a_second() {
    let vars = [VarDecl {
        name: "year".into(),
        ty: Type::Int,
    }];
    let rule = Expr::check(
        "year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)",
        &vars,
        Type::Bool,
    )
    .unwrap();
    let start = Instant::now();
    let mut leap_years = 0u32;
    for year in -500_000..500_000 {
        if rule.eval(&[Value::Int(year)]) == Ok(Value::Bool(true)) {
            leap_years += 1;
        }
    }
    let elapsed = start.elapsed();
    // 97 leap years in every 400; the range is exactly 2,500 such cycles.
    assert_eq!(leap_years, 242_500);
    println!("1,000,000 evaluations took {elapsed:?}");
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
