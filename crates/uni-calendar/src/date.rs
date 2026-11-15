use uni_time::{Instant, Ratio};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Date {
    pub values: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstantDate {
    pub above: Vec<i64>,
    pub date: Date,
    pub remainder: Ratio,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DateStart {
    pub instant: Instant,
    pub rounded_by: Option<Ratio>,
}
