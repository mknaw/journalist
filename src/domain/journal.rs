use crate::domain::{DateRange, Entry};
use crate::infrastructure::EntryRepository;
use anyhow::Result;
use chrono::NaiveDate;
use std::collections::HashMap;

pub struct Journal {
    entries: HashMap<NaiveDate, Entry>,
    repository: Box<dyn EntryRepository>,
}

impl Journal {
    pub fn new(repository: Box<dyn EntryRepository>) -> Self {
        Self {
            entries: HashMap::new(),
            repository,
        }
    }

    pub fn get_entry(&mut self, date: NaiveDate) -> Result<Option<&Entry>> {
        if !self.entries.contains_key(&date) {
            if let Some(entry) = self.repository.load(date)? {
                self.entries.insert(date, entry);
            }
        }

        Ok(self.entries.get(&date))
    }

    pub fn get_entry_mut(&mut self, date: NaiveDate) -> Result<&mut Entry> {
        if !self.entries.contains_key(&date) {
            let entry = self
                .repository
                .load(date)?
                .unwrap_or_else(|| Entry::new(date));
            self.entries.insert(date, entry);
        }

        Ok(self.entries.get_mut(&date).unwrap())
    }

    pub fn save_entry(&mut self, date: NaiveDate) -> Result<()> {
        if let Some(entry) = self.entries.get(&date) {
            self.repository.save(entry.clone())?;
        }
        Ok(())
    }

    pub fn get_entries_in_range(&mut self, range: DateRange) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();

        for date in range.days() {
            if let Some(entry) = self.get_entry(date)? {
                entries.push(entry.clone());
            }
        }

        Ok(entries)
    }

    pub fn list_dates_in_range(&self, range: DateRange) -> Result<Vec<NaiveDate>> {
        self.repository.list_dates(range)
    }
}
