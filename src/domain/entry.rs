use crate::domain::bullet::{Event, Note, Task};
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub date: NaiveDate,
    pub tasks: Vec<Task>,
    pub events: Vec<Event>,
    pub notes: Vec<Note>,
    pub priorities: Vec<Task>,
    pub inspirations: Vec<Note>,
    pub insights: Vec<Note>,
    pub missteps: Vec<Note>,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            tasks: Vec::new(),
            events: Vec::new(),
            notes: Vec::new(),
            priorities: Vec::new(),
            inspirations: Vec::new(),
            insights: Vec::new(),
            missteps: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) -> &mut Self {
        self.tasks.push(task);
        self
    }

    pub fn add_event(&mut self, event: Event) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn add_note(&mut self, note: Note) -> &mut Self {
        self.notes.push(note);
        self
    }

    pub fn add_priority(&mut self, task: Task) -> &mut Self {
        self.priorities.push(task);
        self
    }

    pub fn add_inspiration(&mut self, note: Note) -> &mut Self {
        self.inspirations.push(note);
        self
    }

    pub fn add_insight(&mut self, note: Note) -> &mut Self {
        self.insights.push(note);
        self
    }

    pub fn add_misstep(&mut self, note: Note) -> &mut Self {
        self.missteps.push(note);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
            && self.events.is_empty()
            && self.notes.is_empty()
            && self.priorities.is_empty()
            && self.inspirations.is_empty()
            && self.insights.is_empty()
            && self.missteps.is_empty()
    }
}
