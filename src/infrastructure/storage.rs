use crate::entities::{DateRange, Entry};
use anyhow::Result;
use chrono::NaiveDate;

/// Combined storage interface that includes both entry and metadata operations
pub trait JournalStorage {
    /// Initialize the storage backend (create tables, indexes, etc.)
    fn initialize(&self) -> Result<()>;

    /// Get storage backend information
    fn backend_info(&self) -> &str;

    /// Perform maintenance operations (vacuum, optimize, etc.)
    fn maintenance(&self) -> Result<()>;
    ///
    /// Load a single entry by date
    fn load_entry(&self, date: NaiveDate) -> Result<Option<Entry>>;

    /// Load multiple entries within a date range
    fn load_entries(&self, range: DateRange) -> Result<Vec<Entry>>;

    /// List all dates that have entries within a range
    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>>;

    /// Save or update an entry
    fn save_entry(&self, entry: &Entry) -> Result<()>;

    /// Delete an entry by date
    fn delete_entry(&self, date: NaiveDate) -> Result<()>;

    /// Search entries by text content
    fn search_entries(&self, query: &str) -> Result<Vec<Entry>>;

    /// Count total number of entries
    fn count_entries(&self) -> Result<u64>;

    /// Get entries with specific bullet types
    fn find_entries_with_tasks(&self, range: DateRange) -> Result<Vec<Entry>>;
    fn find_entries_with_events(&self, range: DateRange) -> Result<Vec<Entry>>;
    fn find_entries_with_priorities(&self, range: DateRange) -> Result<Vec<Entry>>;

    fn refresh_metadata(&self, date: NaiveDate, entry: &Entry) -> Result<()>;
}

