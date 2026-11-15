use uni_time::Instant;

use crate::{Calendar, CalendarError, Date, DateStart, InstantDate};

type R<T> = Result<T, CalendarError>;

impl Calendar {
    pub fn fixed_from_instant(&self, _instant: Instant) -> R<i64> {
        todo!()
    }

    pub fn start_of_fixed(&self, _fixed: i64) -> R<DateStart> {
        todo!()
    }

    pub fn instant_to_date(&self, _instant: Instant) -> R<InstantDate> {
        todo!()
    }

    pub fn date_to_instant(&self, _date: &Date) -> R<DateStart> {
        todo!()
    }
}
