use uni_time::Instant;

use crate::{CalendarDef, CalendarError, Clock, DefinitionError};

type R<T> = Result<T, CalendarError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calendar {
    def: CalendarDef,
}

impl Calendar {
    pub fn new(_def: CalendarDef) -> Result<Calendar, DefinitionError> {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    pub fn clock(&self) -> Clock {
        todo!()
    }

    pub fn era_name(&self) -> &str {
        todo!()
    }

    pub fn epoch(&self) -> Instant {
        todo!()
    }

    pub fn units(&self) -> Vec<&str> {
        todo!()
    }

    pub fn era_unit(&self) -> &str {
        todo!()
    }

    pub fn units_per(&self, _outer: &str, _inner: &str) -> R<i64> {
        let _ = &self.def;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use uni_time::{Instant, Ratio};

    use super::*;
    use crate::{
        EraDef, FundamentalDef, GroupingDef, Limit, MemberDef, MemberUnitDef, SubdivisionDef,
    };

    fn s(text: &str) -> String {
        text.to_string()
    }

    fn one(name: &str, count: i64) -> GroupingDef {
        GroupingDef {
            name: s(name),
            member_unit: None,
            members: vec![MemberDef { name: None, count }],
            first_number: None,
        }
    }

    fn sub(name: &str, parts: i64) -> SubdivisionDef {
        SubdivisionDef {
            name: s(name),
            parts,
            first_number: None,
        }
    }

    /// Aster, as in the spec, built here because unit tests can't see `tests/common`.
    fn aster() -> CalendarDef {
        let counts = [56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55];
        let names = [
            "Prime", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth", "Ninth",
            "Tenth", "Eleventh", "Twelfth",
        ];
        CalendarDef {
            name: s("Aster"),
            clock: Clock::Gct,
            groupings: vec![
                one("age", 10),
                one("decade", 10),
                GroupingDef {
                    name: s("year"),
                    member_unit: Some(MemberUnitDef {
                        name: s("month"),
                        first_number: None,
                    }),
                    members: names
                        .iter()
                        .zip(counts)
                        .map(|(name, count)| MemberDef {
                            name: Some(s(name)),
                            count,
                        })
                        .collect(),
                    first_number: None,
                },
            ],
            fundamental: FundamentalDef {
                name: s("sol"),
                length: Ratio::new(88_775_244, 1000).unwrap(),
                first_number: None,
            },
            subdivisions: vec![sub("hour", 20), sub("minute", 100)],
            era: EraDef {
                name: s("After Landing"),
                unit: s("year"),
                epoch: Instant::ZERO,
            },
        }
    }

    /// Only a fundamental unit, numbered by the era.
    fn bare(length: Ratio) -> CalendarDef {
        CalendarDef {
            name: s("Bare"),
            clock: Clock::Gct,
            groupings: vec![],
            fundamental: FundamentalDef {
                name: s("tick"),
                length,
                first_number: None,
            },
            subdivisions: vec![],
            era: EraDef {
                name: s("Count"),
                unit: s("tick"),
                epoch: Instant::ZERO,
            },
        }
    }

    fn refused(def: CalendarDef) -> DefinitionError {
        Calendar::new(def).expect_err("should be refused")
    }

    fn secs(num: i128, den: i128) -> Ratio {
        Ratio::new(num, den).unwrap()
    }

    fn beyond(unit: &str, limit: Limit) -> DefinitionError {
        DefinitionError::BeyondLimits {
            unit: s(unit),
            limit,
        }
    }

    #[test]
    fn non_positive_lengths() {
        for length in [Ratio::ZERO, secs(-1, 1)] {
            let mut def = aster();
            def.fundamental.length = length;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveLength { unit: s("sol") }
            );
        }
    }

    #[test]
    fn grouping_without_members() {
        let mut def = aster();
        def.groupings[1].members.clear();
        assert_eq!(
            refused(def),
            DefinitionError::NoMembers { unit: s("decade") }
        );
    }

    #[test]
    fn non_positive_counts_name_the_member() {
        for count in [0, -3] {
            let mut def = aster();
            def.groupings[2].members[2].count = count;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveCount {
                    unit: s("year"),
                    member: s("Third")
                }
            );
            let mut def = aster();
            def.groupings[0].members[0].count = count;
            assert_eq!(
                refused(def),
                DefinitionError::NonPositiveCount {
                    unit: s("age"),
                    member: s("1")
                }
            );
        }
    }

    #[test]
    fn several_members_need_a_member_unit() {
        let mut def = aster();
        def.groupings[1].members.push(MemberDef {
            name: None,
            count: 5,
        });
        assert_eq!(
            refused(def),
            DefinitionError::MembersNeedUnit { unit: s("decade") }
        );
    }

    #[test]
    fn subdivision_into_no_parts() {
        let mut def = aster();
        def.subdivisions[1].parts = 0;
        assert_eq!(
            refused(def),
            DefinitionError::TooFewParts { unit: s("minute") }
        );
    }

    #[test]
    fn duplicate_unit_names() {
        let mut def = aster();
        def.subdivisions[1].name = s("hour");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("hour") }
        );
        let mut def = aster();
        def.groupings[2].member_unit.as_mut().unwrap().name = s("decade");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("decade") }
        );
        let mut def = aster();
        def.fundamental.name = s("age");
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateUnit { name: s("age") }
        );
    }

    #[test]
    fn duplicate_member_names() {
        let mut def = aster();
        def.groupings[2].members[1].name = Some(s("Prime"));
        assert_eq!(
            refused(def),
            DefinitionError::DuplicateMember {
                unit: s("year"),
                name: s("Prime")
            }
        );
    }

    #[test]
    fn era_unit_must_be_at_or_above_the_fundamental_unit() {
        let mut def = aster();
        def.era.unit = s("week");
        assert_eq!(
            refused(def),
            DefinitionError::EraUnitUnknown { unit: s("week") }
        );
        let mut def = aster();
        def.era.unit = s("hour");
        assert_eq!(
            refused(def),
            DefinitionError::EraUnitBelowFundamental { unit: s("hour") }
        );
    }

    #[test]
    fn first_numbers_only_below_the_era_unit() {
        let mut def = aster();
        def.groupings[2].first_number = Some(0);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("year") }
        );
        let mut def = aster();
        def.groupings[1].first_number = Some(1);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("decade") }
        );
        let mut def = bare(Ratio::ONE);
        def.fundamental.first_number = Some(0);
        assert_eq!(
            refused(def),
            DefinitionError::FirstNumberNotAllowed { unit: s("tick") }
        );
        // Below the era's unit, any whole number is fine.
        let mut def = aster();
        def.groupings[2].member_unit.as_mut().unwrap().first_number = Some(-4);
        def.fundamental.first_number = Some(0);
        def.subdivisions[0].first_number = Some(1);
        assert!(Calendar::new(def).is_ok());
    }

    #[test]
    fn empty_names() {
        let mut def = aster();
        def.name.clear();
        assert_eq!(
            refused(def),
            DefinitionError::EmptyName { what: "calendar" }
        );
        let mut def = aster();
        def.era.name.clear();
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "era" });
        let mut def = aster();
        def.subdivisions[0].name.clear();
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "unit" });
        let mut def = aster();
        def.groupings[2].members[0].name = Some(String::new());
        assert_eq!(refused(def), DefinitionError::EmptyName { what: "member" });
    }

    #[test]
    fn length_too_fine() {
        // 1 + 1/10¹⁹ s is 10⁹ + 1/10¹⁰ ns: a denominator of 10¹⁰.
        let ten_19 = 10_000_000_000_000_000_000;
        let def = bare(secs(ten_19 + 1, ten_19));
        assert_eq!(refused(def), beyond("tick", Limit::LengthTooFine));
        // A denominator of exactly 10⁹ is fine: 1/10¹⁸ s more than a second.
        let ten_18 = 1_000_000_000_000_000_000;
        assert!(Calendar::new(bare(secs(ten_18 + 1, ten_18))).is_ok());
    }

    #[test]
    fn smallest_unit_shorter_than_a_nanosecond() {
        let mut def = bare(secs(1, 1_000_000_000));
        def.subdivisions = vec![sub("half", 2)];
        assert_eq!(refused(def), beyond("half", Limit::ShorterThanNanosecond));
        let def = bare(secs(1, 2_000_000_000));
        assert_eq!(refused(def), beyond("tick", Limit::ShorterThanNanosecond));
    }

    #[test]
    fn too_many_ticks() {
        let mut def = bare(secs(1_000_000 * 31_557_600, 1));
        def.subdivisions = vec![sub("minute", 1 << 60)];
        assert_eq!(refused(def), beyond("tick", Limit::TooManyTicks));
    }

    #[test]
    fn grouping_too_long() {
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![one("big", 1 << 32), one("bigger", 1 << 32)];
        def.era.unit = s("big");
        assert_eq!(refused(def), beyond("big", Limit::GroupingTooLong));
        // Members that add up past i64 overflow too.
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![GroupingDef {
            name: s("wide"),
            member_unit: Some(MemberUnitDef {
                name: s("part"),
                first_number: None,
            }),
            members: vec![
                MemberDef {
                    name: None,
                    count: i64::MAX,
                },
                MemberDef {
                    name: None,
                    count: 1,
                },
            ],
            first_number: None,
        }];
        def.era.unit = s("wide");
        assert_eq!(refused(def), beyond("wide", Limit::GroupingTooLong));
    }

    #[test]
    fn accepted_edge_shapes() {
        // Only a fundamental unit, which its era numbers.
        let cal = Calendar::new(bare(Ratio::ONE)).unwrap();
        assert_eq!(cal.units(), ["tick"]);
        assert_eq!(cal.era_unit(), "tick");
        // A year of one 400-sol month.
        let mut def = bare(Ratio::ONE);
        def.groupings = vec![GroupingDef {
            name: s("year"),
            member_unit: Some(MemberUnitDef {
                name: s("month"),
                first_number: None,
            }),
            members: vec![MemberDef {
                name: Some(s("Only")),
                count: 400,
            }],
            first_number: None,
        }];
        def.era.unit = s("year");
        let cal = Calendar::new(def).unwrap();
        assert_eq!(cal.units(), ["year", "month", "tick"]);
        assert_eq!(cal.units_per("year", "tick"), Ok(400));
        // A 1 ns fundamental unit, and one of a million years.
        assert!(Calendar::new(bare(secs(1, 1_000_000_000))).is_ok());
        assert!(Calendar::new(bare(secs(31_557_600_000_000, 1))).is_ok());
    }

    #[test]
    fn aster_is_accepted() {
        let cal = Calendar::new(aster()).unwrap();
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
    }
}
