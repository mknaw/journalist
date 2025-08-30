use crate::infrastructure::EntryRepository;
use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Bullet Journal Domain Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum BulletType {
    Task,
    Event,
    Note,
    Priority,
    Inspiration,
    Insight,
    Misstep,
}

impl fmt::Display for BulletType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BulletType::Task => write!(f, "task"),
            BulletType::Event => write!(f, "event"),
            BulletType::Note => write!(f, "note"),
            BulletType::Priority => write!(f, "priority"),
            BulletType::Inspiration => write!(f, "inspiration"),
            BulletType::Insight => write!(f, "insight"),
            BulletType::Misstep => write!(f, "misstep"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub enum TaskState {
    Pending,
    Completed,
    Migrated,
    Scheduled,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Pending => write!(f, "pending"),
            TaskState::Completed => write!(f, "completed"),
            TaskState::Migrated => write!(f, "migrated"),
            TaskState::Scheduled => write!(f, "scheduled"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bullet {
    pub content: String,
    pub bullet_type: BulletType,
    pub task_state: Option<TaskState>,
}

impl Bullet {
    pub fn new(content: impl Into<String>, bullet_type: BulletType) -> Self {
        Self {
            content: content.into(),
            bullet_type,
            task_state: match bullet_type {
                BulletType::Task | BulletType::Priority => Some(TaskState::Pending),
                _ => None,
            },
        }
    }

    pub fn with_task_state(content: impl Into<String>, bullet_type: BulletType, state: TaskState) -> Self {
        Self {
            content: content.into(),
            bullet_type,
            task_state: Some(state),
        }
    }

    pub fn complete(mut self) -> Self {
        if matches!(self.bullet_type, BulletType::Task | BulletType::Priority) {
            self.task_state = Some(TaskState::Completed);
        }
        self
    }

    pub fn migrate(mut self) -> Self {
        if matches!(self.bullet_type, BulletType::Task | BulletType::Priority) {
            self.task_state = Some(TaskState::Migrated);
        }
        self
    }

    pub fn schedule(mut self) -> Self {
        if matches!(self.bullet_type, BulletType::Task | BulletType::Priority) {
            self.task_state = Some(TaskState::Scheduled);
        }
        self
    }
}

// ============================================================================
// Entry
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub date: NaiveDate,
    pub bullets: HashMap<BulletType, Vec<Bullet>>,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            bullets: HashMap::new(),
        }
    }

    pub fn add_bullet(&mut self, bullet: Bullet) -> &mut Self {
        self.bullets
            .entry(bullet.bullet_type.clone())
            .or_insert_with(Vec::new)
            .push(bullet);
        self
    }

    pub fn get_bullets(&self, bullet_type: &BulletType) -> &[Bullet] {
        self.bullets.get(bullet_type).map_or(&[], |bullets| bullets.as_slice())
    }

    pub fn get_bullets_mut(&mut self, bullet_type: &BulletType) -> &mut Vec<Bullet> {
        self.bullets.entry(bullet_type.clone()).or_insert_with(Vec::new)
    }

    pub fn all_bullets(&self) -> impl Iterator<Item = &Bullet> {
        self.bullets.values().flatten()
    }

    pub fn is_empty(&self) -> bool {
        self.bullets.values().all(|bullets| bullets.is_empty())
    }

    pub fn bullet_count(&self, bullet_type: &BulletType) -> usize {
        self.bullets.get(bullet_type).map_or(0, |bullets| bullets.len())
    }

    pub fn total_bullets(&self) -> usize {
        self.bullets.values().map(|bullets| bullets.len()).sum()
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
    pub entries: HashMap<NaiveDate, Entry>,
    pub repository: Box<dyn EntryRepository>,
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