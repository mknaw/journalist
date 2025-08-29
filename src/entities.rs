use crate::infrastructure::EntryRepository;
use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Bullet Journal Domain Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulletContent(pub String);

impl BulletContent {
    pub fn new(content: impl Into<String>) -> Self {
        Self(content.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BulletContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Completed,
    Migrated,
    Scheduled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub content: BulletContent,
    pub state: TaskState,
}

impl Task {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
            state: TaskState::Pending,
        }
    }

    pub fn with_state(content: impl Into<String>, state: TaskState) -> Self {
        Self {
            content: BulletContent::new(content),
            state,
        }
    }

    pub fn complete(mut self) -> Self {
        self.state = TaskState::Completed;
        self
    }

    pub fn migrate(mut self) -> Self {
        self.state = TaskState::Migrated;
        self
    }

    pub fn schedule(mut self) -> Self {
        self.state = TaskState::Scheduled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub content: BulletContent,
}

impl Event {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub content: BulletContent,
}

impl Note {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
        }
    }
}

// ============================================================================
// Entry
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub date: NaiveDate,
    pub tasks: Vec<Task>,
    pub events: Vec<Event>,
    pub notes: Vec<Note>,
    pub priorities: Vec<Task>,
    pub inspirations: Vec<Note>,
    pub insights: Vec<Note>,
    pub missteps: Vec<Note>,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            tasks: Vec::new(),
            events: Vec::new(),
            notes: Vec::new(),
            priorities: Vec::new(),
            inspirations: Vec::new(),
            insights: Vec::new(),
            missteps: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub fn add_event(&mut self, event: Event) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn add_note(&mut self, note: Note) -> &mut Self {
        self.notes.push(note);
        self
    }

    pub fn add_priority(&mut self, task: Task) -> &mut Self {
        self.priorities.push(task);
        self
    }

    pub fn add_inspiration(&mut self, note: Note) -> &mut Self {
        self.inspirations.push(note);
        self
    }

    pub fn add_insight(&mut self, note: Note) -> &mut Self {
        self.insights.push(note);
        self
    }

    pub fn add_misstep(&mut self, note: Note) -> &mut Self {
        self.missteps.push(note);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
            && self.events.is_empty()
            && self.notes.is_empty()
            && self.priorities.is_empty()
            && self.inspirations.is_empty()
            && self.insights.is_empty()
            && self.missteps.is_empty()
    }
}

// ============================================================================
// Date Range and View Scope
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ViewScope {
    Day(NaiveDate),
    Week(NaiveDate),  // Start of week
    Month(NaiveDate), // Start of month
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub scope: ViewScope,
}

impl DateRange {
    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> NaiveDate {
        self.end
    }

    pub fn day(date: NaiveDate) -> Self {
        Self {
            start: date,
            end: date,
            scope: ViewScope::Day(date),
        }
    }

    pub fn week(start_of_week: NaiveDate) -> Self {
        let end = start_of_week + chrono::Duration::days(6);
        Self {
            start: start_of_week,
            end,
            scope: ViewScope::Week(start_of_week),
        }
    }

    pub fn month(year: i32, month: u32) -> Self {
        let start = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid year/month");
        let end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .expect("Invalid date calculation")
        .pred_opt()
        .expect("Invalid month end calculation");

        Self {
            start,
            end,
            scope: ViewScope::Month(start),
        }
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    pub fn days(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start;
        let end = self.end;
        (0..=(end - start).num_days()).map(move |i| start + chrono::Duration::days(i))
    }
}

// ============================================================================
// Journal
// ============================================================================

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