use crate::entities::{DateRange, Entry};
use crate::infrastructure::{EntryRepository, HookRegistry, WriteContext, MarkdownParser};
use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;

pub struct FileSystemRepository {
    data_dir: PathBuf,
    journal_dir: PathBuf,
    parser: MarkdownParser,
    hook_registry: HookRegistry,
}

impl FileSystemRepository {
    pub fn new(data_dir: PathBuf, journal_dir: PathBuf) -> Self {
        Self {
            data_dir,
            journal_dir,
            parser: MarkdownParser::new(),
            hook_registry: HookRegistry::new(),
        }
    }

    pub fn with_hooks(
        data_dir: PathBuf,
        journal_dir: PathBuf,
        hook_registry: HookRegistry,
    ) -> Self {
        Self {
            data_dir,
            journal_dir,
            parser: MarkdownParser::new(),
            hook_registry,
        }
    }

    fn entry_path(&self, date: NaiveDate) -> PathBuf {
        self.data_dir
            .join(date.format("%Y").to_string())
            .join(date.format("%m").to_string())
            .join(date.format("%d").to_string())
            .join("entry.md")
    }
}

impl EntryRepository for FileSystemRepository {
    fn load(&self, date: NaiveDate) -> Result<Option<Entry>> {
        let path = self.entry_path(date);

        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let entry = self.parser.parse(date, &content)?;
        Ok(Some(entry))
    }

    fn save(&self, entry: Entry) -> Result<()> {
        let path = self.entry_path(entry.date);

        // Create data directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Ensure journal directory exists
        std::fs::create_dir_all(&self.journal_dir)?;

        let content = self.parser.serialize(&entry)?;
        std::fs::write(&path, &content)?;

        // Call write hooks after successful write
        let context = WriteContext {
            date: entry.date,
            entry_path: path,
            journal_dir: self.journal_dir.clone(),
            content,
        };

        self.hook_registry.execute_write_hooks(&context, &entry)?;

        Ok(())
    }

    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>> {
        let mut dates = Vec::new();

        for date in range.days() {
            let path = self.entry_path(date);
            if path.exists() {
                dates.push(date);
            }
        }

        Ok(dates)
    }
}
