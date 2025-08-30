use crate::entities::Entry;
use crate::infrastructure::storage::JournalStorage;
use crate::infrastructure::{DuckDbStorage, WriteContext, WriteHook};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Plugin that syncs entry writes to DuckDB storage
pub struct DuckDbSyncHook {
    storage: Arc<DuckDbStorage>,
}

impl DuckDbSyncHook {
    pub fn new(journal_dir: &PathBuf) -> Result<Self> {
        let db_path = journal_dir.join("journal.db");
        let storage =
            Arc::new(DuckDbStorage::new(db_path).context("Failed to initialize DuckDB storage")?);

        Ok(Self { storage })
    }

    pub fn with_storage(storage: Arc<DuckDbStorage>) -> Self {
        Self { storage }
    }
}

impl WriteHook for DuckDbSyncHook {
    fn on_entry_written(&self, _context: &WriteContext, entry: &Entry) -> Result<()> {
        self.storage
            .save_entry(entry)
            .context("Failed to sync entry to DuckDB")?;

        // Refresh metadata for the entry
        self.storage
            .refresh_metadata(entry.date, entry)
            .context("Failed to refresh metadata in DuckDB")?;

        Ok(())
    }

    fn name(&self) -> &str {
        "DuckDB Sync"
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

