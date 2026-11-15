use uni_time::{Instant, Ratio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clock {
    Gct,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDef {
    pub name: String,
    pub clock: Clock,
    pub groupings: Vec<GroupingDef>,
    pub fundamental: FundamentalDef,
    pub subdivisions: Vec<SubdivisionDef>,
    pub era: EraDef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundamentalDef {
    pub name: String,
    pub length: Ratio,
    pub first_number: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupingDef {
    pub name: String,
    pub member_unit: Option<MemberUnitDef>,
    pub members: Vec<MemberDef>,
    pub first_number: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberUnitDef {
    pub name: String,
    pub first_number: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberDef {
    pub name: Option<String>,
    pub count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubdivisionDef {
    pub name: String,
    pub parts: i64,
    pub first_number: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EraDef {
    pub name: String,
    pub unit: String,
    pub epoch: Instant,
}
