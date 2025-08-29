use crate::application::Config;
use crate::entities::{DateRange, Journal, ViewScope};
use crate::infrastructure::{FileSystemRepository, HookRegistry, SimpleLoggerHook};
use chrono::{Datelike, Local, NaiveDate};

pub struct JournalApp {
    journal: Journal,
    config: Config,
    current_date: NaiveDate,
    current_view: ViewScope,
}

impl JournalApp {
    pub fn new() -> Self {
        Self::with_default_plugins()
    }

    pub fn with_default_plugins() -> Self {
        let config = Config::from_env();

        // Set up hook registry with default plugins
        let mut hook_registry = HookRegistry::new();
        hook_registry.register(SimpleLoggerHook);

        let repository = FileSystemRepository::with_hooks(
            config.data_dir.clone(),
            config.journal_dir.clone(),
            hook_registry,
        );
        let journal = Journal::new(Box::new(repository));
        let current_date = Local::now().naive_local().date();
        let current_view = ViewScope::Day(current_date);

        Self {
            journal,
            config,
            current_date,
            current_view,
        }
    }

    pub fn without_plugins() -> Self {
        let config = Config::from_env();
        let repository =
            FileSystemRepository::new(config.data_dir.clone(), config.journal_dir.clone());
        let journal = Journal::new(Box::new(repository));
        let current_date = Local::now().naive_local().date();
        let current_view = ViewScope::Day(current_date);

        Self {
            journal,
            config,
            current_date,
            current_view,
        }
    }

    pub fn run_tui(&mut self) -> anyhow::Result<()> {
        // TODO: Implement TUI main loop
        println!("Starting Journalism TUI...");

        // For now, just show today's entry
        let entry = self.journal.get_entry(self.current_date)?;
        if let Some(entry) = entry {
            println!(
                "Entry for {}: {} items",
                entry.date,
                entry.tasks.len() + entry.events.len() + entry.notes.len()
            );
        } else {
            println!("No entry for {} yet", self.current_date);
        }

        todo!("Implement TUI interface")
    }

    pub fn navigate_to_date(&mut self, date: NaiveDate) {
        self.current_date = date;
        self.current_view = ViewScope::Day(date);
    }

    pub fn switch_to_week_view(&mut self) {
        // TODO: Calculate start of week
        todo!("Implement week view navigation")
    }

    pub fn switch_to_month_view(&mut self) {
        let (year, month) = (self.current_date.year(), self.current_date.month());
        self.current_view = ViewScope::Month(NaiveDate::from_ymd_opt(year, month, 1).unwrap());
    }

    pub fn edit_entry_for_date(&mut self, date: NaiveDate) -> anyhow::Result<()> {
        use std::process::Command;

        // Get or create entry
        let entry = self.journal.get_entry_mut(date)?;

        // Create temporary file path (same as final path for simplicity)
        let temp_path = self
            .config
            .data_dir
            .join(date.format("%Y").to_string())
            .join(date.format("%m").to_string())
            .join(date.format("%d").to_string())
            .join("entry.md");

        // Ensure parent directory exists
        if let Some(parent) = temp_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write current content if entry exists, otherwise create empty file
        if entry.is_empty() {
            // Create template for new entries
            let template = format!(
                "# Tasks\n\n# Events\n\n# Notes\n\n# Priority\n\n# Inspiration\n\n# Insights\n\n# Missteps\n\n"
            );
            std::fs::write(&temp_path, template)?;
        } else {
            // Save current entry content
            self.journal.save_entry(date)?;
        }

        // Launch editor
        let status = Command::new(&self.config.editor).arg(&temp_path).status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Editor exited with error: {}", status));
        }

        // TODO: Reload entry from file after editing
        println!("Entry saved for {}", date);

        Ok(())
    }

    pub fn get_current_range(&self) -> DateRange {
        match self.current_view {
            ViewScope::Day(date) => DateRange::day(date),
            ViewScope::Week(start) => DateRange::week(start),
            ViewScope::Month(start) => DateRange::month(start.year(), start.month()),
        }
    }
}
