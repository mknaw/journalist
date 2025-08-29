use crate::domain::{Entry, Event, Note, Task, TaskState};

pub struct TuiRenderer;

impl TuiRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_entry(&self, entry: &Entry) {
        println!("=== {} ===", entry.date);

        if !entry.tasks.is_empty() {
            println!("\n• TASKS");
            for task in &entry.tasks {
                let symbol = match task.state {
                    TaskState::Pending => "•",
                    TaskState::Completed => "X",
                    TaskState::Migrated => ">",
                    TaskState::Scheduled => "<",
                };
                println!("  {} {}", symbol, task.content.as_str());
            }
        }

        if !entry.events.is_empty() {
            println!("\n○ EVENTS");
            for event in &entry.events {
                println!("  ○ {}", event.content.as_str());
            }
        }

        if !entry.notes.is_empty() {
            println!("\n— NOTES");
            for note in &entry.notes {
                println!("  — {}", note.content.as_str());
            }
        }

        if !entry.priorities.is_empty() {
            println!("\n★ PRIORITY");
            for task in &entry.priorities {
                println!("  ★ {}", task.content.as_str());
            }
        }

        if !entry.inspirations.is_empty() {
            println!("\n! INSPIRATION");
            for note in &entry.inspirations {
                println!("  ! {}", note.content.as_str());
            }
        }

        if !entry.insights.is_empty() {
            println!("\n$ INSIGHTS");
            for note in &entry.insights {
                println!("  $ {}", note.content.as_str());
            }
        }

        if !entry.missteps.is_empty() {
            println!("\nv MISSTEPS");
            for note in &entry.missteps {
                println!("  v {}", note.content.as_str());
            }
        }
    }

    pub fn render_calendar_view(&self) {
        // TODO: Implement calendar grid view
        todo!("Implement calendar rendering with ratatui")
    }

    pub fn render_week_view(&self) {
        // TODO: Implement week view
        todo!("Implement week view rendering")
    }
}
