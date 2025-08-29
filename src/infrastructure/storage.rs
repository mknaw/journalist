use crate::entities::{DateRange, Entry};
use anyhow::Result;
use chrono::NaiveDate;

/// Trait for querying journal entries from storage
pub trait EntryStorage {
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
}

/// Trait for managing journal metadata and analytics
pub trait MetadataStorage {
    /// Get writing statistics for a date range
    fn get_writing_stats(&self, range: DateRange) -> Result<WritingStats>;
    
    /// Get most frequently used words/phrases
    fn get_common_terms(&self, limit: usize) -> Result<Vec<TermFrequency>>;
    
    /// Track cross-references between entries
    fn get_related_entries(&self, date: NaiveDate) -> Result<Vec<NaiveDate>>;
    
    /// Update metadata after entry changes
    fn refresh_metadata(&self, date: NaiveDate, entry: &Entry) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct WritingStats {
    pub total_entries: u64,
    pub total_words: u64,
    pub total_tasks: u64,
    pub total_events: u64,
    pub total_notes: u64,
    pub avg_words_per_entry: f64,
    pub most_productive_day: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct TermFrequency {
    pub term: String,
    pub frequency: u64,
    pub first_seen: NaiveDate,
    pub last_seen: NaiveDate,
}

/// Combined storage interface that includes both entry and metadata operations
pub trait JournalStorage: EntryStorage + MetadataStorage {
    /// Initialize the storage backend (create tables, indexes, etc.)
    fn initialize(&self) -> Result<()>;
    
    /// Get storage backend information
    fn backend_info(&self) -> &str;
    
    /// Perform maintenance operations (vacuum, optimize, etc.)
    fn maintenance(&self) -> Result<()>;
}