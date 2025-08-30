# Journo - TUI Bullet Journal

A terminal-based bullet journal application with calendar view, inspired by Ryder Carroll's Bullet Journal system.

## Project Structure

This is a Rust project using ratatui for the terminal interface.

### Core Components

- **TUI Calendar View**: Navigate between days/weeks/months
- **Entry Structure**: Markdown format for bullet journal entries (stored in DuckDB)
- **External Editor**: Opens `$EDITOR` for editing entries via temp files (like `git commit -e`)

### Bullet Journal Structure

The system implements these modules:

- **Index**: Table of contents for collections and logs
- **Future Log**: 6-month overview for future events/tasks
- **Monthly Log**: Calendar view and task list for current month
- **Daily Log**: Day-to-day rapid logging

### Bullet Symbols

```
• Task (to-do item)
○ Event (something that happened)  
— Note (information to remember)
★ Priority (important items)
! Inspiration (ideas worth exploring)
$ Insight (valuable realization or lesson learned)
v Misstep (mistake or approach to avoid)

Task States:
X Completed task
> Migrated task (moved to another day/month)
< Scheduled task (assigned to specific future date)
```

### Entry File Format

Daily entries are stored as markdown files with structured headers. Each section header corresponds to a bullet type, and individual lines under each header are interpreted as bullets of that type for TUI display.

**Markdown Structure:**
```markdown
# Tasks
Complete project proposal
Review code changes

# Events
Team meeting at 2pm
Lunch with client

# Notes
New framework release announced
Server maintenance scheduled

# Priority
Submit quarterly report

# Inspiration
Idea for improving user onboarding

# Insights
Async patterns reduced response time by 40%

# Missteps
Forgot to backup before major refactoring
```

The bullet symbols are used only for TUI display - the actual markdown uses headers to organize content by type.

## Data Storage

### DuckDB-Only Architecture

The journal uses a single DuckDB database file for all storage, eliminating filesystem complexity:

```
$JOURNAL_DIR/                    # Default: ~/.local/share/journo
└── journal.db                   # Single DuckDB database file
```

### Storage Benefits

- **Single Source of Truth**: All data stored in one DuckDB file
- **Easy Syncing**: Single file sync with tools like syncthing between hosts
- **Full-Text Search**: Native DuckDB search capabilities across all entries
- **Analytics**: Built-in aggregation and statistical analysis
- **No File Conflicts**: Eliminates directory structure sync issues
- **Atomic Operations**: Database transactions ensure data consistency

### Editor Integration

When editing entries:
1. **Existing Entry**: Query DuckDB → format to markdown → write to temp file → launch editor
2. **New Entry**: Create markdown template → write to temp file → launch editor  
3. **Save Process**: Parse edited content → save directly to DuckDB → delete temp file

### Configuration

- **Default Location**: `~/.local/share/journo/` (follows XDG Base Directory Specification)
- **Environment Variable**: `JOURNAL_DIR` - override default storage location
- **Database File**: `$JOURNAL_DIR/journal.db` - Single DuckDB database file
- **Temp Files**: Created on-demand with `.md` extension for editor syntax highlighting
- **Migration Support**: Automatic schema migrations for database upgrades

### Storage Features

- Full-text search across entries
- Writing statistics and analytics
- Cross-references between entries
- Term frequency analysis
- Bullet type aggregation and filtering
- Sparse storage (no records for empty days)

## Plugin System

### Write Hooks

The application supports a plugin system that triggers hooks when entries are written:

#### Hook Interface

```rust
pub trait WriteHook: Send + Sync {
    /// Called after an entry has been successfully written to disk
    fn on_entry_written(&self, context: &WriteContext, entry: &Entry) -> anyhow::Result<()>;
    
    /// Human-readable name for this hook
    fn name(&self) -> &str;
    
    /// Whether this hook should be enabled by default
    fn enabled_by_default(&self) -> bool { true }
}
```

#### Write Context

Hooks receive context about the write operation:

```rust
pub struct WriteContext {
    pub date: NaiveDate,
    pub entry_path: PathBuf,      // Path to the written entry file
    pub journal_dir: PathBuf,     // Path to journal directory (contains journal.db)
    pub content: String,          // Raw markdown content written
}
```

#### Database Integration

Since all data is stored directly in DuckDB, there's no separate sync plugin needed. Entry saves automatically:
- Update the DuckDB database directly
- Refresh search indexes for full-text search
- Update metadata and statistics
- Maintain transactional consistency

#### Example Plugin Use Cases

- **Vector Embeddings**: Generate embeddings for semantic search
- **Tag Extraction**: Parse and index hashtags or mentions  
- **Cross-References**: Build links between related entries
- **Backup**: Sync entries to external storage
- **Analytics**: Extended writing pattern analysis beyond built-in stats

## Development Commands

```bash
# Build and run
cargo run

# Development with auto-reload
cargo watch -x run

# Run tests
cargo test

# Lint and format
cargo clippy
cargo fmt
```

## Features

- Calendar navigation (day/week/month views)
- Week-level aggregation (concatenates structured entries from each day)
- External editor integration via `$EDITOR` with temp files
- DuckDB-based storage for reliability and sync simplicity
- Full-text search and analytics
- Bullet journal rapid logging system
