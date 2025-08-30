// TODO I cannot for the life of me figure out how to exec
// `PRAGMA create_fts_index('bullets', 'id', 'content');`
// through the rust duckdb bindings... very whack.

use crate::entities::{Bullet, BulletType, DateRange, Entry, TaskState};
use crate::infrastructure::repository::EntryRepository;
use crate::infrastructure::storage::JournalStorage;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params};
use log::{debug, info};
use std::collections::HashMap;
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
        let db_path = db_path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let conn = Connection::open(db_path)?;
        debug!("DuckDB connection opened");

        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.initialize()?;
        info!("DuckDB storage initialized successfully");
        Ok(storage)
    }
}

impl JournalStorage for DuckDbStorage {
    fn initialize(&self) -> Result<()> {
        debug!("Setting up migration system");
        self.set_up_migration_system()?;
        debug!("Running migrations");
        self.run_migrations()?;
        debug!("Storage initialization complete");
        Ok(())
    }

    fn backend_info(&self) -> &str {
        "DuckDB Storage Backend v1.0"
    }

    fn maintenance(&self) -> Result<()> {
        debug!("Starting database maintenance operations");
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM; ANALYZE;")
            .context("Failed to perform maintenance operations")?;
        info!("Database maintenance completed successfully");
        Ok(())
    }

    fn load_entry(&self, date: NaiveDate) -> Result<Option<Entry>> {
        debug!("Loading entry for date: {}", date);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT content, type, task_state FROM bullets WHERE date = ? ORDER BY id")
            .context("Failed to prepare select statement")?;

        let date_str = date.format("%Y-%m-%d").to_string();
        debug!("Querying bullets for date: {}", date_str);
        let rows = stmt.query_map(params![date_str], |row| {
            let content: String = row.get(0)?;
            let type_str: String = row.get(1)?;
            let task_state_str: Option<String> = row.get(2)?;
            Ok((content, type_str, task_state_str))
        })?;

        let mut entry = Entry::new(date);
        let mut has_bullets = false;

        for row in rows {
            let (content, type_str, task_state_str) = row?;
            has_bullets = true;

            let bullet_type = match type_str.as_str() {
                "task" => BulletType::Task,
                "event" => BulletType::Event,
                "note" => BulletType::Note,
                "priority" => BulletType::Priority,
                "inspiration" => BulletType::Inspiration,
                "insight" => BulletType::Insight,
                "misstep" => BulletType::Misstep,
                _ => continue,
            };

            let task_state = task_state_str.and_then(|s| match s.as_str() {
                "pending" => Some(TaskState::Pending),
                "completed" => Some(TaskState::Completed),
                "migrated" => Some(TaskState::Migrated),
                "scheduled" => Some(TaskState::Scheduled),
                _ => None,
            });

            let bullet = Bullet {
                content,
                bullet_type,
                task_state,
            };

            entry.add_bullet(bullet);
        }

        if has_bullets {
            debug!(
                "Loaded entry for {} with {} bullets",
                date,
                entry.total_bullets()
            );
            Ok(Some(entry))
        } else {
            debug!("No bullets found for date: {}", date);
            Ok(None)
        }
    }

    fn load_entries(&self, range: DateRange) -> Result<Vec<Entry>> {
        debug!(
            "Loading entries for range: {} to {}",
            range.start(),
            range.end()
        );
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT date, content, type, task_state FROM bullets WHERE date BETWEEN ? AND ? ORDER BY date, id"
        ).context("Failed to prepare select statement")?;

        let rows = stmt.query_map(
            params![
                range.start().format("%Y-%m-%d").to_string(),
                range.end().format("%Y-%m-%d").to_string()
            ],
            |row| {
                let date_str: String = row.get(0)?;
                let content: String = row.get(1)?;
                let type_str: String = row.get(2)?;
                let task_state_str: Option<String> = row.get(3)?;
                Ok((date_str, content, type_str, task_state_str))
            },
        )?;

        let mut entries_map: HashMap<NaiveDate, Entry> = HashMap::new();

        for row in rows {
            let (date_str, content, type_str, task_state_str) = row?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from database")?;

            let bullet_type = match type_str.as_str() {
                "task" => BulletType::Task,
                "event" => BulletType::Event,
                "note" => BulletType::Note,
                "priority" => BulletType::Priority,
                "inspiration" => BulletType::Inspiration,
                "insight" => BulletType::Insight,
                "misstep" => BulletType::Misstep,
                _ => continue,
            };

            let task_state = task_state_str.and_then(|s| match s.as_str() {
                "pending" => Some(TaskState::Pending),
                "completed" => Some(TaskState::Completed),
                "migrated" => Some(TaskState::Migrated),
                "scheduled" => Some(TaskState::Scheduled),
                _ => None,
            });

            let bullet = Bullet {
                content,
                bullet_type,
                task_state,
            };

            entries_map
                .entry(date)
                .or_insert_with(|| Entry::new(date))
                .add_bullet(bullet);
        }

        let mut entries: Vec<Entry> = entries_map.into_values().collect();
        entries.sort_by_key(|e| e.date);
        debug!(
            "Loaded {} entries in range {} to {}",
            entries.len(),
            range.start(),
            range.end()
        );
        Ok(entries)
    }

    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>> {
        debug!(
            "Listing dates for range: {} to {}",
            range.start(),
            range.end()
        );
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT DISTINCT date FROM bullets WHERE date BETWEEN ? AND ? ORDER BY date")
            .context("Failed to prepare select statement")?;

        let rows = stmt.query_map(
            params![
                range.start().format("%Y-%m-%d").to_string(),
                range.end().format("%Y-%m-%d").to_string()
            ],
            |row| {
                let date_str: String = row.get(0)?;
                Ok(date_str)
            },
        )?;

        let mut dates = Vec::new();
        for date_str in rows {
            let date_str = date_str?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from database")?;
            dates.push(date);
        }

        debug!(
            "Found {} dates in range {} to {}",
            dates.len(),
            range.start(),
            range.end()
        );
        Ok(dates)
    }

    fn save_entry(&self, entry: &Entry) -> Result<()> {
        debug!(
            "Saving entry for date: {} with {} total bullets",
            entry.date,
            entry.total_bullets()
        );
        let conn = self.conn.lock().unwrap();
        let date_str = entry.date.format("%Y-%m-%d").to_string();

        // Delete existing bullets for this date
        debug!("Deleting existing bullets for date: {}", date_str);
        conn.execute("DELETE FROM bullets WHERE date = ?", params![date_str])
            .context("Failed to delete existing bullets")?;

        // Insert all bullets for this entry
        let mut stmt = conn
            .prepare("INSERT INTO bullets (date, content, type, task_state) VALUES (?, ?, ?, ?)")
            .context("Failed to prepare insert statement")?;

        let mut bullet_count = 0;
        for (bullet_type, bullets) in &entry.bullets {
            debug!(
                "Inserting {} bullets of type: {}",
                bullets.len(),
                bullet_type
            );
            for bullet in bullets {
                let task_state_str = bullet.task_state.as_ref().map(|s| s.to_string());
                debug!(
                    "Inserting bullet: {} (type: {}, state: {:?})",
                    bullet.content, bullet_type, task_state_str
                );
                stmt.execute(params![
                    date_str,
                    bullet.content,
                    bullet_type.to_string(),
                    task_state_str
                ])
                .context("Failed to insert bullet")?;
                bullet_count += 1;
            }
        }

        info!(
            "Successfully saved {} bullets for date: {}",
            bullet_count, entry.date
        );
        Ok(())
    }

    fn delete_entry(&self, date: NaiveDate) -> Result<()> {
        debug!("Deleting entry for date: {}", date);
        let conn = self.conn.lock().unwrap();
        let date_str = date.format("%Y-%m-%d").to_string();

        let affected_rows = conn
            .execute("DELETE FROM bullets WHERE date = ?", params![date_str])
            .context("Failed to delete entry")?;

        info!("Deleted {} bullets for date: {}", affected_rows, date);
        Ok(())
    }

    fn search_entries(&self, query: &str) -> Result<Vec<Entry>> {
        debug!("Searching entries with query: '{}'", query);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT date FROM bullets 
             WHERE content MATCH ? 
             ORDER BY date DESC",
            )
            .context("Failed to prepare search statement")?;

        let rows = stmt.query_map(params![query], |row| {
            let date_str: String = row.get(0)?;
            Ok(date_str)
        })?;

        let mut entries = Vec::new();
        for date_str in rows {
            let date_str = date_str?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse search result date")?;
            if let Some(entry) = self.load_entry(date)? {
                entries.push(entry);
            }
        }

        info!("Search for '{}' returned {} entries", query, entries.len());
        Ok(entries)
    }

    fn count_entries(&self) -> Result<u64> {
        debug!("Counting total entries");
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(DISTINCT date) FROM bullets", [], |row| {
            row.get(0)
        })?;

        debug!("Total entries count: {}", count);
        Ok(count as u64)
    }

    fn find_entries_with_tasks(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_type("task", range)
    }

    fn find_entries_with_events(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_type("event", range)
    }

    fn find_entries_with_priorities(&self, range: DateRange) -> Result<Vec<Entry>> {
        self.find_entries_by_type("priority", range)
    }

    // TODO probably will want to do something with this!
    fn refresh_metadata(&self, _date: NaiveDate, _entry: &Entry) -> Result<()> {
        Ok(())
    }
}

impl DuckDbStorage {
    fn set_up_migration_system(&self) -> Result<()> {
        debug!("Setting up migration system");
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#,
        )
        .context("Failed to create migrations table")?;
        debug!("Migration system table created/verified");
        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        debug!("Running database migrations");
        let migrations = self.discover_migrations()?;
        let applied = self.get_applied_migrations()?;

        debug!(
            "Found {} total migrations, {} already applied",
            migrations.len(),
            applied.len()
        );

        for (version, name, sql_content) in migrations {
            if !applied.contains(&version) {
                info!("Applying migration {}: {}", version, name);
                self.apply_migration(version, &name, &sql_content)
                    .with_context(|| format!("Failed to apply migration {}: {}", version, name))?;
            } else {
                debug!("Migration {} already applied, skipping", version);
            }
        }

        info!("All migrations completed successfully");
        Ok(())
    }

    fn discover_migrations(&self) -> Result<Vec<(i32, String, String)>> {
        let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
        debug!("Looking for migrations in: {:?}", migrations_dir);

        if !migrations_dir.exists() {
            debug!("Migrations directory does not exist, skipping");
            return Ok(vec![]);
        }

        let mut migrations = Vec::new();
        let entries =
            fs::read_dir(&migrations_dir).context("Failed to read migrations directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if let Some(version_str) = filename.split('_').next() {
                        if let Ok(version) = version_str.parse::<i32>() {
                            let name = filename
                                .strip_suffix(".sql")
                                .unwrap_or(filename)
                                .to_string();
                            let content = fs::read_to_string(&path).with_context(|| {
                                format!("Failed to read migration file: {}", path.display())
                            })?;
                            migrations.push((version, name, content));
                        }
                    }
                }
            }
        }

        migrations.sort_by_key(|(version, _, _)| *version);
        debug!("Discovered {} migration files", migrations.len());
        Ok(migrations)
    }

    fn get_applied_migrations(&self) -> Result<std::collections::HashSet<i32>> {
        debug!("Querying applied migrations from database");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT version FROM migrations ORDER BY version")
            .context("Failed to prepare migration query")?;

        let rows = stmt.query_map([], |row| {
            let version: i32 = row.get(0)?;
            Ok(version)
        })?;

        let mut applied = std::collections::HashSet::new();
        for version in rows {
            let v = version?;
            applied.insert(v);
            debug!("Found applied migration version: {}", v);
        }

        debug!("Total applied migrations: {}", applied.len());
        Ok(applied)
    }

    fn apply_migration(&self, version: i32, name: &str, sql_content: &str) -> Result<()> {
        debug!("Applying migration {} ({})", version, name);
        let conn = self.conn.lock().unwrap();

        // Execute the migration SQL
        debug!("Executing migration SQL for {}", name);
        conn.execute_batch(sql_content)
            .with_context(|| format!("Failed to execute migration SQL for {}", name))?;
        debug!("Migration SQL executed successfully for {}", name);

        // Record the migration as applied
        debug!("Recording migration {} as applied", version);
        conn.execute(
            "INSERT INTO migrations (version, name) VALUES (?, ?)",
            params![version, name],
        )
        .with_context(|| format!("Failed to record migration {} as applied", name))?;

        info!("Migration {} ({}) applied successfully", version, name);
        Ok(())
    }

    fn find_entries_by_type(&self, bullet_type: &str, range: DateRange) -> Result<Vec<Entry>> {
        debug!(
            "Finding entries with bullet type '{}' in range {} to {}",
            bullet_type,
            range.start(),
            range.end()
        );
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT date FROM bullets WHERE type = ? AND date BETWEEN ? AND ? ORDER BY date"
        ).context("Failed to prepare type-based query")?;

        let rows = stmt.query_map(
            params![
                bullet_type,
                range.start().format("%Y-%m-%d").to_string(),
                range.end().format("%Y-%m-%d").to_string()
            ],
            |row| {
                let date_str: String = row.get(0)?;
                Ok(date_str)
            },
        )?;

        let mut entries = Vec::new();
        for date_str in rows {
            let date_str = date_str?;
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("Failed to parse date from type search")?;
            if let Some(entry) = self.load_entry(date)? {
                entries.push(entry);
            }
        }

        debug!(
            "Found {} entries with bullet type '{}'",
            entries.len(),
            bullet_type
        );
        Ok(entries)
    }
}

// Bridge implementation for backwards compatibility with Journal
impl EntryRepository for DuckDbStorage {
    fn load(&self, date: NaiveDate) -> Result<Option<Entry>> {
        self.load_entry(date)
    }

    fn save(&self, entry: Entry) -> Result<()> {
        self.save_entry(&entry)
    }

    fn list_dates(&self, range: DateRange) -> Result<Vec<NaiveDate>> {
        JournalStorage::list_dates(self, range)
    }
}
