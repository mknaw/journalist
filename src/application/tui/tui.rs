use crate::entities::{Entry, Bullet, BulletType, TaskState};

pub struct TuiRenderer;

impl TuiRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_entry(&self, entry: &Entry) {
        println!("=== {} ===", entry.date);

        let sections = [
            (BulletType::Task, "• TASKS", "•"),
            (BulletType::Event, "○ EVENTS", "○"),
            (BulletType::Note, "— NOTES", "—"),
            (BulletType::Priority, "★ PRIORITY", "★"),
            (BulletType::Inspiration, "! INSPIRATION", "!"),
            (BulletType::Insight, "$ INSIGHTS", "$"),
            (BulletType::Misstep, "v MISSTEPS", "v"),
        ];

        for (bullet_type, section_name, default_symbol) in sections {
            let bullets = entry.get_bullets(&bullet_type);
            if !bullets.is_empty() {
                println!("\n{}", section_name);
                for bullet in bullets {
                    let symbol = if matches!(bullet_type, BulletType::Task | BulletType::Priority) {
                        match bullet.task_state {
                            Some(TaskState::Pending) => "•",
                            Some(TaskState::Completed) => "X",
                            Some(TaskState::Migrated) => ">",
                            Some(TaskState::Scheduled) => "<",
                            None => default_symbol,
                        }
                    } else {
                        default_symbol
                    };
                    println!("  {} {}", symbol, bullet.content);
                }
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
