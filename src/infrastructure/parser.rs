use crate::entities::{Entry, Bullet, BulletType, TaskState};
use anyhow::Result;
use chrono::NaiveDate;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, date: NaiveDate, content: &str) -> Result<Entry> {
        let mut entry = Entry::new(date);
        let mut current_bullet_type: Option<BulletType> = None;

        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') {
                current_bullet_type = match line.to_lowercase().as_str() {
                    "# tasks" => Some(BulletType::Task),
                    "# events" => Some(BulletType::Event),
                    "# notes" => Some(BulletType::Note),
                    "# priority" => Some(BulletType::Priority),
                    "# inspiration" => Some(BulletType::Inspiration),
                    "# insights" => Some(BulletType::Insight),
                    "# missteps" => Some(BulletType::Misstep),
                    _ => None,
                };
                continue;
            }

            if let Some(bullet_type) = current_bullet_type {
                let bullet = Bullet::new(line, bullet_type);
                entry.add_bullet(bullet);
            }
        }

        Ok(entry)
    }

    pub fn serialize(&self, entry: &Entry) -> Result<String> {
        let mut content = String::new();

        let sections = [
            (BulletType::Task, "# Tasks"),
            (BulletType::Event, "# Events"),
            (BulletType::Note, "# Notes"),
            (BulletType::Priority, "# Priority"),
            (BulletType::Inspiration, "# Inspiration"),
            (BulletType::Insight, "# Insights"),
            (BulletType::Misstep, "# Missteps"),
        ];

        for (bullet_type, section_header) in sections {
            let bullets = entry.get_bullets(&bullet_type);
            if !bullets.is_empty() {
                content.push_str(&format!("{}\n", section_header));
                for bullet in bullets {
                    content.push_str(&format!("{}\n", bullet.content));
                }
                content.push('\n');
            }
        }

        Ok(content)
    }
}
