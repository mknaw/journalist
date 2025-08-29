use crate::entities::{DateRange, Entry};
use crate::infrastructure::storage::{EntryStorage, JournalStorage, MetadataStorage, TermFrequency, WritingStats};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params, OptionalExt};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct DuckDbStorage {
    conn: Mutex<Connection>,
}

// Mark DuckDbStorage as Send + Sync since we've wrapped the connection in a Mutex
unsafe impl Send for DuckDbStorage {}
unsafe impl Sync for DuckDbStorage {}

impl DuckDbStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open DuckDB connection")?;
        
        let storage = Self { conn: Mutex::new(conn) };
        storage.initialize()?;
        Ok(storage)
    }
    
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory DuckDB connection")?;
        
        let storage = Self { conn: Mutex::new(conn) };
        storage.initialize()?;
        Ok(storage)
    }

    fn serialize_entry(&self, entry: &Entry) -> Result<String> {
        serde_json::to_string(entry)
            .context("Failed to serialize entry to JSON")
    }

    fn deserialize_entry(&self, json: &str) -> Result<Entry> {
        serde_json::from_str(json)
            .context("Failed to deserialize entry from JSON")
    }
}

impl JournalStorage for DuckDbStorage {
    fn initialize(&self) -> Result<()> {
        self.setup_migration_system()?;
        self.run_migrations()?;
        Ok(())
    }

    fn backend_info(&self) -> &str {
        "DuckDB Storage Backend v1.0"
    }

    fn maintenance(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM; ANALYZE;")
            .context("Failed to perform maintenance operations")?;
        Ok(())
    }
}

impl EntryStorage for DuckDbStorage {
    fn load_entry(&self, date: NaiveDate) -> Result<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT content FROM entries WHERE date = ?")
            .context("Failed to prepare select statement")?;
        
        let mut rows = stmt.query(params![date.format("%Y-%m-%d").to_string()])?;
        
        if let Some(row) = rows.next()? {
            let content: String = row.get(0)?;
            let entry = self.deserialize_entry(&content)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    fn load_entries(&self, range: DateRange) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content FROM entries WHERE date BETWEEN ? AND ? ORDER BY date"
        ).context("Failed to prepare select statement")?;
        
        let rows = stmt.query_map(params![
            range.start().format("%Y-%m-%d").to_string(),
            range.end().format("%Y-%m-%d").to_string()
        ], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;

        let mut entries = Vec::new();
        for content in rows {
            let content = content?;
            let entry = self.deserialize_entry(&content)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }

    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT date FROM entries WHERE date BETWEEN ? AND ? ORDER BY date"
        ).context("Failed to prepare select statement")?;
        
        let rows = stmt.query_map(params![
            range.start().format("%Y-%m-%d").to_string(),
            range.end().format("%Y-%m-%d").to_string()
        ], |row| {
            let date_str: String = row.get(0)?;
            Ok(date_str)
        })?;

        let mut dates = Vec::new();
        for date_str in rows {
            let date_str = date_str?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from database")?;
            dates.push(date);
        }
        
        Ok(dates)
    }

    fn save_entry(&self, entry: &Entry) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let content = self.serialize_entry(entry)?;
        let date_str = entry.date.format("%Y-%m-%d").to_string();
        
        // Calculate counts
        let word_count = self.count_words(&content);
        let task_count = entry.tasks.len() as i32;
        let event_count = entry.events.len() as i32;
        let note_count = entry.notes.len() as i32;
        let priority_count = entry.priorities.len() as i32;
        let inspiration_count = entry.inspirations.len() as i32;
        let insight_count = entry.insights.len() as i32;
        let misstep_count = entry.missteps.len() as i32;

        conn.execute(r#"
            INSERT OR REPLACE INTO entries (
                date, content, word_count, task_count, event_count, note_count,
                priority_count, inspiration_count, insight_count, misstep_count,
                updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#, params![
            date_str, content, word_count, task_count, event_count, note_count,
            priority_count, inspiration_count, insight_count, misstep_count
        ]).context("Failed to save entry")?;

        // Update search index
        conn.execute(
            "INSERT OR REPLACE INTO entry_search (date, content_fts) VALUES (?, ?)",
            params![date_str, content]
        ).context("Failed to update search index")?;

        Ok(())
    }

    fn delete_entry(&self, date: NaiveDate) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let date_str = date.format("%Y-%m-%d").to_string();
        
        conn.execute("DELETE FROM entries WHERE date = ?", params![date_str])
            .context("Failed to delete entry")?;
        
        Ok(())
    }

    fn search_entries(&self, query: &str) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT e.content FROM entries e 
             JOIN entry_search s ON e.date = s.date 
             WHERE s.content_fts LIKE ? 
             ORDER BY e.date DESC"
        ).context("Failed to prepare search statement")?;
        
        let search_pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![search_pattern], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;

        let mut entries = Vec::new();
        for content in rows {
            let content = content?;
            let entry = self.deserialize_entry(&content)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }

    fn count_entries(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM entries", [], |row| {
            row.get(0)
        })?;
        
        Ok(count as u64)
    }

    fn find_entries_with_tasks(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_count("task_count", range)
    }

    fn find_entries_with_events(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_count("event_count", range)
    }

    fn find_entries_with_priorities(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_count("priority_count", range)
    }
}

impl MetadataStorage for DuckDbStorage {
    fn get_writing_stats(&self, range: DateRange) -> Result<WritingStats> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(r#"
            SELECT 
                COUNT(*) as total_entries,
                COALESCE(SUM(word_count), 0) as total_words,
                COALESCE(SUM(task_count), 0) as total_tasks,
                COALESCE(SUM(event_count), 0) as total_events,
                COALESCE(SUM(note_count + priority_count + inspiration_count + insight_count + misstep_count), 0) as total_notes,
                COALESCE(AVG(word_count), 0) as avg_words
            FROM entries 
            WHERE date BETWEEN ? AND ?
        "#).context("Failed to prepare stats query")?;
        
        let row = stmt.query_row(params![
            range.start().format("%Y-%m-%d").to_string(),
            range.end().format("%Y-%m-%d").to_string()
        ], |row| {
            Ok((
                row.get::<_, i64>(0)? as u64,  // total_entries
                row.get::<_, i64>(1)? as u64,  // total_words
                row.get::<_, i64>(2)? as u64,  // total_tasks
                row.get::<_, i64>(3)? as u64,  // total_events
                row.get::<_, i64>(4)? as u64,  // total_notes
                row.get::<_, f64>(5)?,         // avg_words
            ))
        })?;

        // Find most productive day
        let most_productive_day = conn.query_row(r#"
            SELECT date FROM entries 
            WHERE date BETWEEN ? AND ? 
            ORDER BY (task_count + event_count + note_count + priority_count + inspiration_count + insight_count + misstep_count) DESC 
            LIMIT 1
        "#, params![
            range.start().format("%Y-%m-%d").to_string(),
            range.end().format("%Y-%m-%d").to_string()
        ], |row| {
            let date_str: String = row.get(0)?;
            Ok(date_str)
        }).optional()?;

        let most_productive_day = most_productive_day
            .map(|date_str| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"))
            .transpose()
            .context("Failed to parse most productive day")?;

        Ok(WritingStats {
            total_entries: row.0,
            total_words: row.1,
            total_tasks: row.2,
            total_events: row.3,
            total_notes: row.4,
            avg_words_per_entry: row.5,
            most_productive_day,
        })
    }

    fn get_common_terms(&self, limit: usize) -> Result<Vec<TermFrequency>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT term, frequency, first_seen, last_seen FROM term_frequency ORDER BY frequency DESC LIMIT ?"
        ).context("Failed to prepare term frequency query")?;
        
        let rows = stmt.query_map(params![limit], |row| {
            let first_seen_str: String = row.get(2)?;
            let last_seen_str: String = row.get(3)?;
            
            let first_seen = NaiveDate::parse_from_str(&first_seen_str, "%Y-%m-%d")
                .map_err(|e| duckdb::Error::FromSqlConversionFailure(2, duckdb::types::Type::Text, Box::new(e)))?;
            let last_seen = NaiveDate::parse_from_str(&last_seen_str, "%Y-%m-%d")
                .map_err(|e| duckdb::Error::FromSqlConversionFailure(3, duckdb::types::Type::Text, Box::new(e)))?;
            
            Ok(TermFrequency {
                term: row.get(0)?,
                frequency: row.get::<_, i64>(1)? as u64,
                first_seen,
                last_seen,
            })
        })?;

        let mut terms = Vec::new();
        for term in rows {
            terms.push(term?);
        }
        
        Ok(terms)
    }

    fn get_related_entries(&self, date: NaiveDate) -> Result<Vec<NaiveDate>> {
        let conn = self.conn.lock().unwrap();
        let date_str = date.format("%Y-%m-%d").to_string();
        let mut stmt = conn.prepare(r#"
            SELECT DISTINCT target_date FROM cross_references WHERE source_date = ?
            UNION
            SELECT DISTINCT source_date FROM cross_references WHERE target_date = ?
        "#).context("Failed to prepare related entries query")?;
        
        let rows = stmt.query_map(params![date_str, date_str], |row| {
            let date_str: String = row.get(0)?;
            Ok(date_str)
        })?;

        let mut dates = Vec::new();
        for date_str in rows {
            let date_str = date_str?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse related entry date")?;
            dates.push(date);
        }
        
        Ok(dates)
    }

    fn refresh_metadata(&self, _date: NaiveDate, _entry: &Entry) -> Result<()> {
        // This would implement term extraction and cross-reference detection
        // For now, just a placeholder
        Ok(())
    }
}

impl DuckDbStorage {
    fn setup_migration_system(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#).context("Failed to create migrations table")?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        let migrations = self.discover_migrations()?;
        let applied = self.get_applied_migrations()?;
        
        for (version, name, sql_content) in migrations {
            if !applied.contains(&version) {
                self.apply_migration(version, &name, &sql_content)
                    .with_context(|| format!("Failed to apply migration {}: {}", version, name))?;
            }
        }
        
        Ok(())
    }
    
    fn discover_migrations(&self) -> Result<Vec<(i32, String, String)>> {
        let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
        
        if !migrations_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut migrations = Vec::new();
        let entries = fs::read_dir(&migrations_dir)
            .context("Failed to read migrations directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(version_str) = filename.split('_').next() {
                        if let Ok(version) = version_str.parse::<i32>() {
                            let name = filename.strip_suffix(".sql").unwrap_or(filename).to_string();
                            let content = fs::read_to_string(&path)
                                .with_context(|| format!("Failed to read migration file: {}", path.display()))?;
                            migrations.push((version, name, content));
                        }
                    }
                }
            }
        }
        
        migrations.sort_by_key(|(version, _, _)| *version);
        Ok(migrations)
    }
    
    fn get_applied_migrations(&self) -> Result<std::collections::HashSet<i32>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT version FROM migrations ORDER BY version")
            .context("Failed to prepare migration query")?;
        
        let rows = stmt.query_map([], |row| {
            let version: i32 = row.get(0)?;
            Ok(version)
        })?;
        
        let mut applied = std::collections::HashSet::new();
        for version in rows {
            applied.insert(version?);
        }
        
        Ok(applied)
    }
    
    fn apply_migration(&self, version: i32, name: &str, sql_content: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Execute the migration SQL
        conn.execute_batch(sql_content)
            .with_context(|| format!("Failed to execute migration SQL for {}", name))?;
        
        // Record the migration as applied
        conn.execute(
            "INSERT INTO migrations (version, name) VALUES (?, ?)",
            params![version, name]
        ).with_context(|| format!("Failed to record migration {} as applied", name))?;
        
        Ok(())
    }

    fn count_words(&self, content: &str) -> i32 {
        content.split_whitespace().count() as i32
    }

    fn find_entries_by_count(&self, count_column: &str, range: DateRange) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let query = format!(
            "SELECT content FROM entries WHERE {} > 0 AND date BETWEEN ? AND ? ORDER BY date",
            count_column
        );
        
        let mut stmt = conn.prepare(&query)
            .context("Failed to prepare count-based query")?;
        
        let rows = stmt.query_map(params![
            range.start().format("%Y-%m-%d").to_string(),
            range.end().format("%Y-%m-%d").to_string()
        ], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;

        let mut entries = Vec::new();
        for content in rows {
            let content = content?;
            let entry = self.deserialize_entry(&content)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
}