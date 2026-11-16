//! Leap rules on the era's unit: checking them, their period, and the cycle table that both
//! conversion directions search (spec 006, research R3 to R5).

use crate::calendar::Unit;
use crate::{CalendarDef, CalendarError, DefinitionError};

/// The cycle of the era's unit when it is a grouping: one rule period of instances, each with
/// its variant (research R4). A calendar without rules has a one-instance cycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cycle {
    /// How many rules the era's unit has.
    pub(crate) rules: usize,
    /// The rule period `P`: instances of the era's unit per cycle.
    pub(crate) period: i64,
    /// One per distinct set of holding rules.
    pub(crate) variants: Vec<Variant>,
    /// For each residue `r` in `0 … P − 1`, the index of its variant in `variants`.
    pub(crate) residues: Vec<u32>,
    /// For each `r` in `0 … P`, fundamental units from the start of a cycle to the start of
    /// instance `r`; the last is the cycle's length.
    pub(crate) starts: Vec<i64>,
}

/// What one instance of the era's unit looks like after the rules that hold in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Variant {
    /// Each member's count after the changes, in the unit below the grouping.
    pub(crate) counts: Vec<i64>,
    /// Each member's start in fundamental units from the instance's start; one more entry,
    /// the instance's length.
    pub(crate) starts: Vec<i64>,
    /// The numbers (from 1) of the rules that hold.
    pub(crate) holding: Vec<usize>,
}

impl Cycle {
    /// Checks the rules of `def` (research R5) and builds the cycle, or `None` when the era's
    /// unit isn't a grouping. `units` is the checked chain with its lengths, `era` the era's
    /// unit's index in it.
    pub(crate) fn build(
        _def: &CalendarDef,
        _units: &[Unit],
        _era: usize,
    ) -> Result<Option<Cycle>, DefinitionError> {
        todo!()
    }

    /// The variant of the instance whose count from the epoch is `count`.
    pub(crate) fn variant(&self, _count: i64) -> &Variant {
        todo!()
    }

    /// The instance of the era's unit that fixed number `fixed` falls in: its count from the
    /// epoch, the position in fundamental units inside it, and its variant.
    pub(crate) fn locate(&self, _fixed: i64) -> Result<(i64, i64, &Variant), CalendarError> {
        todo!()
    }

    /// Where the instance whose count from the epoch is `count` starts, in fundamental units
    /// from the epoch.
    pub(crate) fn start(&self, _count: i64) -> Result<i64, CalendarError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::tests::aster;
    use crate::{Calendar, LeapRuleDef};

    fn rule(condition: &str, member: &str, change: i64) -> LeapRuleDef {
        LeapRuleDef {
            condition: condition.to_string(),
            member: member.to_string(),
            change,
        }
    }

    const ASTER_RULE: &str = "year % 5 == 0 && (year % 100 != 0 || year % 500 == 0)";
    const COMMON: [i64; 12] = [56, 56, 55, 56, 56, 55, 56, 56, 55, 56, 56, 55];

    fn cycle_of(rules: Vec<LeapRuleDef>) -> super::Cycle {
        let mut def = aster();
        def.groupings[2].rules = rules;
        Calendar::new(def)
            .unwrap()
            .cycle
            .expect("the year is a grouping")
    }

    #[test]
    fn aster_cycle_table() {
        let cycle = cycle_of(vec![rule(ASTER_RULE, "Twelfth", 1)]);
        assert_eq!(cycle.rules, 1);
        assert_eq!(cycle.period, 500);
        assert_eq!(cycle.residues.len(), 500);
        assert_eq!(cycle.starts.len(), 501);
        assert_eq!(cycle.variants.len(), 2);
        let mut leap = COMMON;
        leap[11] = 56;
        let counts = |r: i64| &cycle.variant(r).counts;
        assert_eq!(counts(0), &COMMON); // year 1
        assert_eq!(counts(4), &leap); // year 5
        assert_eq!(counts(99), &COMMON); // year 100
        assert_eq!(counts(499), &leap); // year 500
        assert_eq!(cycle.variant(4).holding, [1]);
        assert_eq!(cycle.variant(0).holding, Vec::<usize>::new());
        assert_eq!(*cycle.variant(4).starts.last().unwrap(), 669);
        assert_eq!(cycle.variant(4).starts[11], 668 - 55);
        // 500 years of 668 sols plus 96 leap days.
        assert_eq!(cycle.starts[500], 334_096);
        assert_eq!(cycle.starts[5], 3_341);
        // Residues repeat: counts −1 and 499 are both year 500's variant (years 0 and 500).
        assert_eq!(cycle.variant(-1), cycle.variant(499));
        assert_eq!(cycle.variant(-500), cycle.variant(0));
    }

    #[test]
    fn two_rules_on_one_member_add_up_in_one_variant() {
        let cycle = cycle_of(vec![
            rule(ASTER_RULE, "Twelfth", 1),
            rule("year % 25 == 0", "Twelfth", 2),
        ]);
        assert_eq!(cycle.period, 500);
        // Common, Aster's rule alone, both, and the second alone (years 100, 200, 300, 400).
        assert_eq!(cycle.variants.len(), 4);
        let year_25 = cycle.variant(24);
        assert_eq!(year_25.counts[11], 58);
        assert_eq!(year_25.holding, [1, 2]);
        assert_eq!(cycle.variant(99).counts[11], 57);
        assert_eq!(cycle.variant(99).holding, [2]);
    }

    #[test]
    fn a_calendar_without_rules_has_a_one_year_cycle() {
        let cycle = Calendar::new(aster()).unwrap().cycle.unwrap();
        assert_eq!(cycle.rules, 0);
        assert_eq!(cycle.period, 1);
        assert_eq!(cycle.starts, [0, 668]);
        assert_eq!(cycle.variants.len(), 1);
        assert_eq!(cycle.variants[0].counts, COMMON);
        assert_eq!(cycle.variants[0].holding, Vec::<usize>::new());
        assert_eq!(cycle.variant(-12_345).counts, COMMON);
    }

    #[test]
    fn locating_fixed_numbers_in_the_cycle() {
        let cycle = cycle_of(vec![rule(ASTER_RULE, "Twelfth", 1)]);
        // Fixed number 3,340 is the last sol of year 5 (count 4); 3,341 starts year 6.
        assert_eq!(cycle.locate(3_340).map(|(c, p, _)| (c, p)), Ok((4, 668)));
        assert_eq!(cycle.locate(3_341).map(|(c, p, _)| (c, p)), Ok((5, 0)));
        // One sol before the epoch: the last sol of year 0 (count −1), a leap year.
        assert_eq!(cycle.locate(-1).map(|(c, p, _)| (c, p)), Ok((-1, 668)));
        assert_eq!(cycle.start(500), Ok(334_096));
        assert_eq!(cycle.start(-1), Ok(-669));
        assert_eq!(cycle.start(5), Ok(3_341));
        assert!(cycle.start(i64::MAX / 2).is_err());
    }
}
