# Journalist - TUI Bullet Journal

A terminal-based bullet journal application with calendar view, inspired by Ryder Carroll's Bullet Journal system.

## Project Structure

This is a Rust project using ratatui for the terminal interface.

### Core Components

- **TUI Calendar View**: Navigate between days/weeks/months
- **Entry Structure**: Markdown files following bullet journal format
- **External Editor**: Opens `$EDITOR` for editing entries (like `git commit -e`)

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

### File Structure

The journal uses a structured directory layout separating data from indexes:

```
$JOURNAL_DIR/                    # Default: ~/.local/share/journalist
├── data/                        # Entry storage
│   ├── 2024/
│   │   ├── 01/
│   │   │   ├── 01/
│   │   │   │   └── entry.md
│   │   │   ├── 02/
│   │   │   │   └── entry.md
│   │   │   └── ...
│   │   └── 12/
│   └── 2025/
└── indexes/                     # Generated indexes and metadata
    ├── reverse_index.json       # (example plugin output)
    ├── embeddings/              # (example plugin output)
    └── tags/                    # (example plugin output)
```

### Configuration

- **Default Location**: `~/.local/share/journalist/` (follows XDG Base Directory Specification)
- **Environment Variable**: `JOURNAL_DIR` - override default storage location
- **Data Directory**: `$JOURNAL_DIR/data/` - contains all entry files
- **Indexes Directory**: `$JOURNAL_DIR/indexes/` - contains generated indexes and plugin outputs
- **File Naming**: Each day uses a single `entry.md` file
- **Sparse Storage**: Days without entries have no corresponding files

### Path Examples

```
~/.local/share/journalist/data/2024/03/15/entry.md     # March 15, 2024
~/.local/share/journalist/data/2024/12/31/entry.md     # December 31, 2024
~/.local/share/journalist/indexes/reverse_index.json  # Generated index
```

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
    pub indexes_dir: PathBuf,     // Path to indexes directory
    pub content: String,          // Raw markdown content written
}
```

#### Example Plugin Use Cases

- **Reverse Index**: Update search indexes when entries are modified
- **Vector Embeddings**: Generate embeddings for semantic search
- **Tag Extraction**: Parse and index hashtags or mentions
- **Cross-References**: Build links between related entries
- **Backup**: Sync entries to external storage
- **Analytics**: Track writing patterns and statistics

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
- External editor integration via `$EDITOR`
- Markdown-based entry storage
- Bullet journal rapid logging system
