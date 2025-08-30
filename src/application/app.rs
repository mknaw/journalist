use crate::application::Config;
use crate::entities::{DateRange, Journal, ViewScope};
use crate::infrastructure::storage::JournalStorage;
use crate::infrastructure::{DuckDbStorage, MarkdownParser};
use chrono::{Datelike, Local, NaiveDate};
use std::io::Write;

pub struct JournalApp {
    pub journal: Journal,
    storage: DuckDbStorage,
    parser: MarkdownParser,
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
        let db_path = config.journal_dir.join("journal.db");

        let storage = DuckDbStorage::new(&db_path).expect("Failed to initialize DuckDB storage");
        let journal = Journal::new(Box::new(storage));
        let current_date = Local::now().naive_local().date();
        let current_view = ViewScope::Day(current_date);

        Self {
            journal,
            storage: DuckDbStorage::new(&db_path).expect("Failed to initialize storage reference"),
            parser: MarkdownParser::new(),
            config,
            current_date,
            current_view,
        }
    }

    pub fn without_plugins() -> Self {
        let config = Config::from_env();
        let db_path = config.journal_dir.join("journal.db");

        let storage = DuckDbStorage::new(&db_path).expect("Failed to initialize DuckDB storage");
        let journal = Journal::new(Box::new(storage));
        let current_date = Local::now().naive_local().date();
        let current_view = ViewScope::Day(current_date);

        Self {
            journal,
            storage: DuckDbStorage::new(&db_path).expect("Failed to initialize storage reference"),
            parser: MarkdownParser::new(),
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
            println!("Entry for {}: {} items", entry.date, entry.total_bullets());
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
        use tempfile::NamedTempFile;

        // Get existing entry or create new one
        let existing_entry = self.storage.load_entry(date)?;

        // Create temp file with .md extension for editor syntax highlighting
        let mut temp_file = NamedTempFile::with_suffix(".md")?;

        // Write current content or template to temp file
        let content = if let Some(ref entry) = existing_entry {
            self.parser.serialize_for_editing(entry)?
        } else {
            MarkdownParser::empty_template()
        };

        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        // Launch editor with temp file
        let status = Command::new(&self.config.editor)
            .arg(temp_file.path())
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Editor exited with error: {}", status));
        }

        // Read edited content from temp file
        let edited_content = std::fs::read_to_string(temp_file.path())?;

        // Parse and save to DuckDB
        let updated_entry = self.parser.parse(date, &edited_content)?;
        self.storage.save_entry(&updated_entry)?;

        // Update journal's in-memory cache
        self.journal.entries.insert(date, updated_entry);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::BulletType;
    use crate::infrastructure::MarkdownParser;
    use crate::infrastructure::test_utils::test_harness::TestStorage;

    #[test]
    fn test_editor_workflow_new_entry() {
        let test_storage = TestStorage::new();
        let storage = test_storage.storage();
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Simulate what happens when editing a new entry

        // 1. Check if entry exists (it shouldn't)
        let existing_entry = storage.load_entry(date).unwrap();
        assert!(existing_entry.is_none());

        // 2. Create template content for temp file
        let template_content = MarkdownParser::empty_template();

        // 3. Verify template contains all sections
        assert!(template_content.contains("# Tasks"));
        assert!(template_content.contains("# Events"));
        assert!(template_content.contains("# Notes"));
        assert!(template_content.contains("# Priority"));
        assert!(template_content.contains("# Inspiration"));
        assert!(template_content.contains("# Insights"));
        assert!(template_content.contains("# Missteps"));

        // 4. Simulate user editing the file (adding content)
        let edited_content = "# Tasks\nComplete unit tests\nAdd documentation\n\n# Events\nTeam standup at 9am\n\n# Notes\nLearned about temp files\n";

        // 5. Parse the edited content
        let updated_entry = parser.parse(date, edited_content).unwrap();

        // 6. Save to storage
        storage.save_entry(&updated_entry).unwrap();

        // 7. Verify it was saved correctly
        let saved_entry = storage.load_entry(date).unwrap().unwrap();
        assert_eq!(saved_entry.date, date);
        assert_eq!(saved_entry.total_bullets(), 4);

        let tasks = saved_entry.get_bullets(&BulletType::Task);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].content, "Complete unit tests");
        assert_eq!(tasks[1].content, "Add documentation");

        let events = saved_entry.get_bullets(&BulletType::Event);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].content, "Team standup at 9am");
    }

    #[test]
    fn test_editor_workflow_existing_entry() {
        let test_storage = TestStorage::new();
        let storage = test_storage.storage();
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // 1. Create and save an initial entry
        let _initial_entry = test_storage.create_sample_entry(date).unwrap();

        // 2. Load existing entry for editing
        let existing_entry = storage.load_entry(date).unwrap().unwrap();
        assert_eq!(existing_entry.total_bullets(), 3);

        // 3. Serialize to markdown for temp file (should show ALL headers for editing)
        let temp_file_content = parser.serialize_for_editing(&existing_entry).unwrap();

        // Should include ALL headers, even empty ones, for better editing UX
        assert!(temp_file_content.contains("# Tasks"));
        assert!(temp_file_content.contains("# Events"));
        assert!(temp_file_content.contains("# Notes"));
        assert!(temp_file_content.contains("# Priority"));
        assert!(temp_file_content.contains("# Inspiration"));
        assert!(temp_file_content.contains("# Insights"));
        assert!(temp_file_content.contains("# Missteps"));

        // Should also include the existing content
        assert!(temp_file_content.contains("Sample task"));
        assert!(temp_file_content.contains("Sample event"));
        assert!(temp_file_content.contains("Sample note"));

        // 4. Simulate user editing (adding and modifying content)
        let edited_content = "# Tasks\nSample task\nNew task added\n\n# Events\nSample event\n\n# Notes\nSample note\nAdded some notes\n";

        // 5. Parse the edited content
        let updated_entry = parser.parse(date, edited_content).unwrap();

        // 6. Save back to storage
        storage.save_entry(&updated_entry).unwrap();

        // 7. Verify changes were saved
        let final_entry = storage.load_entry(date).unwrap().unwrap();
        assert_eq!(final_entry.total_bullets(), 5);

        let tasks = final_entry.get_bullets(&BulletType::Task);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].content, "Sample task");
        assert_eq!(tasks[1].content, "New task added");

        let notes = final_entry.get_bullets(&BulletType::Note);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].content, "Sample note");
        assert_eq!(notes[1].content, "Added some notes");
    }

    #[test]
    fn test_editor_workflow_round_trip_preservation() {
        let test_storage = TestStorage::new();
        let storage = test_storage.storage();
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Create a complex entry with all bullet types
        let complex_entry = test_storage.create_complex_entry(date).unwrap();

        // Load entry (simulating editing workflow)
        let loaded_entry = storage.load_entry(date).unwrap().unwrap();

        // Serialize to temp file content
        let temp_content = parser.serialize(&loaded_entry).unwrap();

        // Parse back from temp file content (simulating no changes by user)
        let reparsed_entry = parser.parse(date, &temp_content).unwrap();

        // Save back to storage
        storage.save_entry(&reparsed_entry).unwrap();

        // Load final result
        let final_entry = storage.load_entry(date).unwrap().unwrap();

        // Should be identical to original
        assert_eq!(final_entry.total_bullets(), complex_entry.total_bullets());

        for bullet_type in [
            BulletType::Task,
            BulletType::Event,
            BulletType::Note,
            BulletType::Priority,
            BulletType::Inspiration,
            BulletType::Insight,
            BulletType::Misstep,
        ] {
            let original_bullets = complex_entry.get_bullets(&bullet_type);
            let final_bullets = final_entry.get_bullets(&bullet_type);

            assert_eq!(
                original_bullets.len(),
                final_bullets.len(),
                "Bullet count mismatch for {:?}",
                bullet_type
            );

            for (orig, final_bullet) in original_bullets.iter().zip(final_bullets.iter()) {
                assert_eq!(
                    orig.content, final_bullet.content,
                    "Content mismatch for {:?}",
                    bullet_type
                );
            }
        }
    }

    #[test]
    fn test_empty_template_structure() {
        let template = MarkdownParser::empty_template();

        // Verify the template structure matches what the parser expects
        let parser = MarkdownParser::new();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        // Parsing the empty template should create an empty entry
        let entry = parser.parse(date, &template).unwrap();
        assert_eq!(entry.total_bullets(), 0);

        // But serializing an empty entry should produce an empty string
        let serialized = parser.serialize(&entry).unwrap();
        assert_eq!(serialized, "");

        // This is expected behavior: template != serialized empty entry
        // Template provides structure for editing, serialized empty is minimal
        assert_ne!(template, serialized);
    }
}
