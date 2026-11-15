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
