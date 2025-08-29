use crate::domain::Entry;
use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;

/// Context provided to write hooks
#[derive(Debug, Clone)]
pub struct WriteContext {
    pub date: NaiveDate,
    pub entry_path: PathBuf,
    pub indexes_dir: PathBuf,
    pub content: String,
}

/// Trait for plugins that respond to entry write events
pub trait WriteHook: Send + Sync {
    /// Called after an entry has been successfully written to disk
    fn on_entry_written(&self, context: &WriteContext, entry: &Entry) -> Result<()>;

    /// Human-readable name for this hook
    fn name(&self) -> &str;

    /// Whether this hook should be enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }
}

/// Registry for managing write hooks
pub struct HookRegistry {
    hooks: Vec<Box<dyn WriteHook>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a new write hook
    pub fn register<H>(&mut self, hook: H)
    where
        H: WriteHook + 'static,
    {
        self.hooks.push(Box::new(hook));
    }

    /// Execute all registered hooks for an entry write
    pub fn execute_write_hooks(&self, context: &WriteContext, entry: &Entry) -> Result<()> {
        for hook in &self.hooks {
            if let Err(e) = hook.on_entry_written(context, entry) {
                eprintln!("Warning: Hook '{}' failed: {}", hook.name(), e);
                // Continue with other hooks even if one fails
            }
        }
        Ok(())
    }

    /// List all registered hooks
    pub fn list_hooks(&self) -> Vec<&str> {
        self.hooks.iter().map(|h| h.name()).collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
