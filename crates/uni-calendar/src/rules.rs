//! Leap rules on the era's unit: checking them, their period, and the cycle table that both
//! conversion directions search (spec 006, research R3 to R5).

use std::collections::HashMap;

use uni_expr::{Expr, Type, Value, VarDecl};

use crate::calendar::{Kind, RULE_PERIOD_LIMIT, Unit};
use crate::{CalendarDef, CalendarError, DefinitionError, GroupingDef, Limit};

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

/// A checked rule: its condition and period, the index of the member it changes, and the
/// change.
struct Rule {
    condition: Expr,
    period: i64,
    member: usize,
    change: i64,
}

impl Cycle {
    /// Checks the rules of `def` (research R5) and builds the cycle, or `None` when the era's
    /// unit isn't a grouping. `units` is the checked chain with its lengths, `era` the era's
    /// unit's index in it.
    pub(crate) fn build(
        def: &CalendarDef,
        units: &[Unit],
        era: usize,
    ) -> Result<Option<Cycle>, DefinitionError> {
        let name = &units[era].name;
        // 1. Rules only on the era's unit, which must be a grouping. Unit names are unique, so
        // a grouping with the era's unit's name is the era's unit.
        if let Some(g) = def
            .groupings
            .iter()
            .find(|g| !g.rules.is_empty() && &g.name != name)
        {
            return Err(DefinitionError::RulesNotOnEraUnit {
                unit: g.name.clone(),
            });
        }
        let (Some(grouping), Kind::Grouping { length, .. }) = (
            def.groupings.iter().find(|g| &g.name == name),
            &units[era].kind,
        ) else {
            return Ok(None);
        };
        // 2 and 3. Each rule, then their combined period.
        let rules = check_rules(grouping)?;
        let mut period: i64 = 1;
        for rule in &rules {
            period = lcm(period, rule.period);
            if period > RULE_PERIOD_LIMIT {
                return Err(DefinitionError::RulesPeriodOverLimit {
                    unit: name.clone(),
                    limit: RULE_PERIOD_LIMIT,
                });
            }
        }
        // 4. The table, year by year. The members count the unit below the grouping, whose
        // length in fundamental units is the grouping's length over the members' total.
        let base: Vec<i64> = grouping.members.iter().map(|m| m.count).collect();
        let below = length / base.iter().sum::<i64>(); // the sum is at most `length`
        let too_long = || DefinitionError::BeyondLimits {
            unit: name.clone(),
            limit: Limit::CycleTooLong,
        };
        let size = usize::try_from(period).expect("at most the limit");
        let mut variants: Vec<Variant> = Vec::new();
        let mut seen: HashMap<Vec<usize>, u32> = HashMap::new();
        let mut residues = Vec::with_capacity(size);
        let mut starts = Vec::with_capacity(size + 1);
        starts.push(0i64);
        for year in 1..=period {
            let mut holding = Vec::new();
            for (i, rule) in rules.iter().enumerate() {
                match rule.condition.eval(&[Value::Int(year)]) {
                    Ok(Value::Bool(true)) => holding.push(i + 1),
                    Ok(_) => {}
                    Err(error) => {
                        return Err(DefinitionError::RuleFails {
                            unit: name.clone(),
                            rule: i + 1,
                            year,
                            error,
                        });
                    }
                }
            }
            let index = match seen.get(&holding) {
                Some(&index) => index,
                None => {
                    let variant =
                        Variant::new(grouping, &base, &rules, holding.clone(), year, below)
                            .map_err(|e| e.unwrap_or_else(too_long))?;
                    variants.push(variant);
                    let index = u32::try_from(variants.len() - 1).expect("at most one per year");
                    seen.insert(holding, index);
                    index
                }
            };
            residues.push(index);
            let total = starts[starts.len() - 1]
                .checked_add(variants[index as usize].length())
                .ok_or_else(too_long)?;
            starts.push(total);
        }
        Ok(Some(Cycle {
            rules: rules.len(),
            period,
            variants,
            residues,
            starts,
        }))
    }

    /// The cycle's length in fundamental units.
    fn length(&self) -> i64 {
        self.starts[self.starts.len() - 1]
    }

    /// The variant of the instance whose count from the epoch is `count`.
    pub(crate) fn variant(&self, count: i64) -> &Variant {
        let r = usize::try_from(count.rem_euclid(self.period)).expect("below the period");
        &self.variants[self.residues[r] as usize]
    }

    /// The instance of the era's unit that fixed number `fixed` falls in: its count from the
    /// epoch, the position in fundamental units inside it, and its variant.
    pub(crate) fn locate(&self, fixed: i64) -> Result<(i64, i64, &Variant), CalendarError> {
        let length = self.length();
        let (cycle, rest) = (fixed.div_euclid(length), fixed.rem_euclid(length));
        // The last start at or before `rest`; starts[0] is 0 and starts[P] is past it.
        let r = self.starts.partition_point(|&s| s <= rest) - 1;
        let count = cycle
            .checked_mul(self.period)
            .and_then(|c| c.checked_add(i64::try_from(r).ok()?))
            .ok_or(CalendarError::OutOfRange)?;
        Ok((
            count,
            rest - self.starts[r],
            &self.variants[self.residues[r] as usize],
        ))
    }

    /// Where the instance whose count from the epoch is `count` starts, in fundamental units
    /// from the epoch.
    pub(crate) fn start(&self, count: i64) -> Result<i64, CalendarError> {
        let r = usize::try_from(count.rem_euclid(self.period)).expect("below the period");
        count
            .div_euclid(self.period)
            .checked_mul(self.length())
            .and_then(|f| f.checked_add(self.starts[r]))
            .ok_or(CalendarError::OutOfRange)
    }
}

impl Variant {
    /// The instance's length in fundamental units.
    pub(crate) fn length(&self) -> i64 {
        self.starts[self.starts.len() - 1]
    }

    /// The variant where the rules numbered in `holding` hold, first met in instance `year`.
    /// `Err(None)` means a total doesn't fit in `i64`.
    fn new(
        grouping: &GroupingDef,
        base: &[i64],
        rules: &[Rule],
        holding: Vec<usize>,
        year: i64,
        below: i64,
    ) -> Result<Variant, Option<DefinitionError>> {
        let mut counts = base.to_vec();
        for &h in &holding {
            let rule = &rules[h - 1];
            counts[rule.member] = counts[rule.member].checked_add(rule.change).ok_or(None)?;
        }
        if let Some((j, &length)) = counts.iter().enumerate().find(|(_, c)| **c < 1) {
            return Err(Some(DefinitionError::MemberTooShort {
                unit: grouping.name.clone(),
                member: label(grouping, j),
                year,
                length,
            }));
        }
        let mut starts = Vec::with_capacity(counts.len() + 1);
        let mut total: i64 = 0;
        starts.push(total);
        for &count in &counts {
            total = count
                .checked_mul(below)
                .and_then(|l| total.checked_add(l))
                .ok_or(None)?;
            starts.push(total);
        }
        Ok(Variant {
            counts,
            starts,
            holding,
        })
    }
}

/// Checks each rule in order (research R5, step 2): its condition, member, change and period.
fn check_rules(grouping: &GroupingDef) -> Result<Vec<Rule>, DefinitionError> {
    let unit = &grouping.name;
    let vars = [VarDecl {
        name: unit.clone(),
        ty: Type::Int,
    }];
    let mut rules = Vec::with_capacity(grouping.rules.len());
    for (i, def) in grouping.rules.iter().enumerate() {
        let rule = i + 1;
        let condition = Expr::check(&def.condition, &vars, Type::Bool).map_err(|error| {
            DefinitionError::RuleCondition {
                unit: unit.clone(),
                rule,
                error,
            }
        })?;
        let member = (0..grouping.members.len())
            .find(|&j| label(grouping, j) == def.member)
            .ok_or_else(|| DefinitionError::RuleUnknownMember {
                unit: unit.clone(),
                rule,
                member: def.member.clone(),
            })?;
        if def.change == 0 {
            return Err(DefinitionError::RuleZeroChange {
                unit: unit.clone(),
                rule,
            });
        }
        let period = condition.period(unit, RULE_PERIOD_LIMIT).map_err(|error| {
            DefinitionError::RulePeriod {
                unit: unit.clone(),
                rule,
                error,
            }
        })?;
        rules.push(Rule {
            condition,
            period,
            member,
            change: def.change,
        });
    }
    Ok(rules)
}

/// A member's label: its name, or its number from 1 when it has none.
fn label(grouping: &GroupingDef, j: usize) -> String {
    grouping.members[j]
        .name
        .clone()
        .unwrap_or_else(|| (j + 1).to_string())
}

/// The least common multiple of two positive numbers, each at most the period limit, so the
/// result fits easily.
fn lcm(a: i64, b: i64) -> i64 {
    let (mut x, mut y) = (a, b);
    while y != 0 {
        (x, y) = (y, x % y);
    }
    a / x * b
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
