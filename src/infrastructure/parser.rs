use crate::domain::{Entry, Event, Note, Task, TaskState};
use anyhow::Result;
use chrono::NaiveDate;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, date: NaiveDate, content: &str) -> Result<Entry> {
        let mut entry = Entry::new(date);

        // TODO: Implement markdown parsing
        // Parse sections like:
        // # Tasks
        // Complete project proposal
        // Review code changes
        //
        // # Events
        // Team meeting at 2pm
        // etc.

        todo!("Implement markdown parsing")
    }

    pub fn serialize(&self, entry: &Entry) -> Result<String> {
        let mut content = String::new();

        if !entry.tasks.is_empty() {
            content.push_str("# Tasks\n");
            for task in &entry.tasks {
                content.push_str(&format!("{}\n", task.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.events.is_empty() {
            content.push_str("# Events\n");
            for event in &entry.events {
                content.push_str(&format!("{}\n", event.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.notes.is_empty() {
            content.push_str("# Notes\n");
            for note in &entry.notes {
                content.push_str(&format!("{}\n", note.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.priorities.is_empty() {
            content.push_str("# Priority\n");
            for task in &entry.priorities {
                content.push_str(&format!("{}\n", task.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.inspirations.is_empty() {
            content.push_str("# Inspiration\n");
            for note in &entry.inspirations {
                content.push_str(&format!("{}\n", note.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.insights.is_empty() {
            content.push_str("# Insights\n");
            for note in &entry.insights {
                content.push_str(&format!("{}\n", note.content.as_str()));
            }
            content.push('\n');
        }

        if !entry.missteps.is_empty() {
            content.push_str("# Missteps\n");
            for note in &entry.missteps {
                content.push_str(&format!("{}\n", note.content.as_str()));
            }
            content.push('\n');
        }

        Ok(content)
    }
}
