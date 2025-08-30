use crate::entities::Entry;
use crate::infrastructure::{WriteContext, WriteHook};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

/// Example plugin that logs all write operations to a file
pub struct SimpleLoggerHook;

impl WriteHook for SimpleLoggerHook {
    fn on_entry_written(&self, context: &WriteContext, _entry: &Entry) -> Result<()> {
        let log_path = context.journal_dir.join("write_log.txt");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        writeln!(
            file,
            "[{}] Entry written for {} - Path: {} - Content length: {} characters",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            context.date,
            context.entry_path.display(),
            context.content.len()
        )?;

        Ok(())
    }

    fn name(&self) -> &str {
        "Simple Logger"
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}
