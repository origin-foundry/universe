//! Hand-worked tests for the spec's Aster calendar (US1 to US3).

mod common;

use common::*;
use uni_calendar::{Calendar, CalendarError};

fn calendar() -> Calendar {
    Calendar::new(aster(*EPOCH)).expect("Aster is valid")
}

mod define {
    use super::*;

    #[test]
    fn aster_is_accepted_with_its_chain() {
        let cal = calendar();
        assert_eq!(cal.name(), "Aster");
        assert_eq!(cal.era_name(), "After Landing");
        assert_eq!(cal.epoch(), *EPOCH);
        assert_eq!(cal.clock(), uni_calendar::Clock::Gct);
        assert_eq!(
            cal.units(),
            ["age", "decade", "year", "month", "sol", "hour", "minute"]
        );
        assert_eq!(cal.era_unit(), "year");
    }

    #[test]
    fn fixed_counts_between_units() {
        let cal = calendar();
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
        assert_eq!(cal.units_per("decade", "sol"), Ok(6_680));
        assert_eq!(cal.units_per("age", "year"), Ok(100));
        assert_eq!(cal.units_per("sol", "minute"), Ok(2_000));
        assert_eq!(cal.units_per("year", "month"), Ok(12));
        assert_eq!(cal.units_per("decade", "month"), Ok(120));
        assert_eq!(cal.units_per("year", "minute"), Ok(668 * 2_000));
        assert_eq!(cal.units_per("sol", "sol"), Ok(1));
    }

    #[test]
    fn counts_that_vary_or_name_no_unit_are_errors() {
        let cal = calendar();
        assert_eq!(
            cal.units_per("month", "sol"),
            Err(CalendarError::NotFixed {
                unit: "month".to_string(),
                of: "sol".to_string()
            })
        );
        assert_eq!(
            cal.units_per("week", "sol"),
            Err(CalendarError::UnknownUnit {
                name: "week".to_string()
            })
        );
        assert_eq!(
            cal.units_per("sol", "week"),
            Err(CalendarError::UnknownUnit {
                name: "week".to_string()
            })
        );
    }

    #[test]
    fn any_sol_length_is_accepted_without_warning() {
        // FR-002a, US1 scenario 7: a calendar needn't match the planet's rotation.
        let mut def = aster(*EPOCH);
        def.fundamental.length = secs(100_000, 1);
        let cal = Calendar::new(def).expect("accepted");
        assert_eq!(cal.units_per("year", "sol"), Ok(668));
    }
}
