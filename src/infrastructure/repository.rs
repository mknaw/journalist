use crate::domain::{DateRange, Entry};
use anyhow::Result;
use chrono::NaiveDate;

pub trait EntryRepository {
    fn load(&self, date: NaiveDate) -> Result<Option<Entry>>;
    fn save(&self, entry: Entry) -> Result<()>;
    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>>;
}
