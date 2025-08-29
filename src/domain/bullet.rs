use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct BulletContent(pub String);

impl BulletContent {
    pub fn new(content: impl Into<String>) -> Self {
        Self(content.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BulletContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    Pending,
    Completed,
    Migrated,
    Scheduled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub content: BulletContent,
    pub state: TaskState,
}

impl Task {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
            state: TaskState::Pending,
        }
    }

    pub fn with_state(content: impl Into<String>, state: TaskState) -> Self {
        Self {
            content: BulletContent::new(content),
            state,
        }
    }

    pub fn complete(mut self) -> Self {
        self.state = TaskState::Completed;
        self
    }

    pub fn migrate(mut self) -> Self {
        self.state = TaskState::Migrated;
        self
    }

    pub fn schedule(mut self) -> Self {
        self.state = TaskState::Scheduled;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub content: BulletContent,
}

impl Event {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub content: BulletContent,
}

impl Note {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: BulletContent::new(content),
        }
    }
}
