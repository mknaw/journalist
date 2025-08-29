# Journalism - TUI Bullet Journal

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
