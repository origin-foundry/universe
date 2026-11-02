//! fixture: f64 and 1.0 in a doc comment
// f32, 2.5e3 in a line comment
/* f64 in a /* nested */ block comment 3.0 */
pub const S: &str = "f64 1.0";
pub const R: &str = r#"f32 "quoted" 4.5"#;
pub const C: char = '1';
pub fn t(p: ((u8, u8), u8)) -> u8 { p.0.1 }
pub fn range() -> usize { (1..3).len() }
pub fn life<'a>(x: &'a str) -> &'a str { x }
