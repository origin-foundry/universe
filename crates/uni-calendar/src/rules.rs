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
